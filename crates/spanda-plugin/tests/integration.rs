use spanda_plugin::api::PluginApiContext;
use spanda_plugin::capability::{enforce_capability, KNOWN_PLUGIN_CAPABILITIES};
use spanda_plugin::compatibility::validate_spanda_version;
use spanda_plugin::hooks::PluginHook;
use spanda_plugin::loader::{LoadFormat, SandboxPermissions};
use spanda_plugin::manifest::PluginManifest;
use spanda_plugin::registry::{ensure_installable, lookup_plugin_entry, PluginTrustTier};
use spanda_plugin::runtime::{PluginManager, PluginState};
use spanda_plugin::security::{
    sign_plugin_artifact, validate_install_security, verify_plugin_signature,
};

const EXAMPLE_MANIFEST: &str = r#"
[plugin]
name = "spanda-plugin-example"
version = "0.1.0"
publisher = "example"
description = "Example plugin"
license = "Apache-2.0"
type = "readiness"

[compatibility]
spanda_version = ">=0.4.0"
api_version = "v1"

[capabilities]
requires = [
  "entity.read",
  "readiness.read",
  "device.read"
]

[security]
signed = false
sandbox = true
network = false
filesystem = "read-only"

[hooks]
enabled = ["on_install", "on_enable", "on_readiness_completed"]
"#;

fn parse_example() -> PluginManifest {
    PluginManifest::parse_str(EXAMPLE_MANIFEST).expect("example manifest parses")
}

#[test]
fn plugin_manifest_parsing() {
    let manifest = parse_example();
    assert_eq!(manifest.plugin.name, "spanda-plugin-example");
    assert_eq!(manifest.plugin.version, "0.1.0");
    assert_eq!(manifest.capabilities.requires.len(), 3);
    assert!(manifest.security.sandbox);
}

#[test]
fn compatibility_validation() {
    let manifest = parse_example();
    let ok = validate_spanda_version(&manifest, "0.4.0").unwrap();
    assert!(ok.compatible);
    let bad = validate_spanda_version(&manifest, "0.3.0").unwrap();
    assert!(!bad.compatible);
}

#[test]
fn capability_enforcement() {
    let manifest = parse_example();
    let caps = manifest.capability_set();
    assert!(enforce_capability(&caps, "entity.read").is_ok());
    assert!(enforce_capability(&caps, "entity.write").is_err());
}

#[test]
fn signature_validation() {
    let digest = "abc123";
    let signed = sign_plugin_artifact("demo-plugin", "0.1.0", digest, "plugin-test-signing-key");
    assert!(verify_plugin_signature(
        "demo-plugin",
        "0.1.0",
        digest,
        &signed,
        &signed.public_key
    ));
}

#[test]
fn sandbox_permissions_from_manifest() {
    let manifest = parse_example();
    let sandbox = SandboxPermissions::from_manifest(&manifest);
    assert!(sandbox.sandbox);
    assert!(!sandbox.allows_network());
    assert!(!sandbox.allows_filesystem_write());
}

#[test]
fn plugin_lifecycle_install_enable_disable() {
    let dir = tempfile::tempdir().unwrap();
    let plugin_src = dir.path().join("readiness-plugin");
    std::fs::create_dir_all(&plugin_src).unwrap();
    std::fs::write(plugin_src.join("spanda.plugin.toml"), EXAMPLE_MANIFEST).unwrap();

    let project = dir.path().join("project");
    std::fs::create_dir_all(&project).unwrap();

    let mut manager = PluginManager::open(&project, "0.4.0").unwrap();
    let installed = manager
        .store_mut()
        .install_from_dir(&plugin_src, "0.4.0", false)
        .unwrap();
    assert_eq!(installed.state, PluginState::Installed);

    manager.store_mut().enable(&installed.name).unwrap();
    assert_eq!(
        manager.store().get(&installed.name).unwrap().state,
        PluginState::Enabled
    );

    manager.store_mut().disable(&installed.name).unwrap();
    assert_eq!(
        manager.store().get(&installed.name).unwrap().state,
        PluginState::Disabled
    );
}

#[test]
fn plugin_hook_execution() {
    let dir = tempfile::tempdir().unwrap();
    let plugin_src = dir.path().join("hook-plugin");
    std::fs::create_dir_all(&plugin_src).unwrap();
    std::fs::write(plugin_src.join("spanda.plugin.toml"), EXAMPLE_MANIFEST).unwrap();
    let project = dir.path().join("project");
    std::fs::create_dir_all(&project).unwrap();

    let mut manager = PluginManager::open(&project, "0.4.0").unwrap();
    manager
        .store_mut()
        .install_from_dir(&plugin_src, "0.4.0", false)
        .unwrap();
    manager.store_mut().enable("spanda-plugin-example").unwrap();
    let result = manager
        .store_mut()
        .dispatch_hook(
            "spanda-plugin-example",
            PluginHook::OnReadinessCompleted,
            serde_json::json!({"mission": "demo"}),
        )
        .unwrap();
    assert!(result.success);
}

#[test]
fn blocked_plugin_rejection() {
    let entry = lookup_plugin_entry("spanda-plugin-blocked-demo").expect("blocked demo listed");
    assert_eq!(entry.trust_tier(), PluginTrustTier::Blocked);
    assert!(ensure_installable(&entry).is_err());
}

#[test]
fn plugin_api_capability_gate() {
    let manifest = parse_example();
    let ctx = PluginApiContext::new("spanda-plugin-example", manifest.capability_set());
    assert!(ctx.readiness_read("mission-1").is_ok());
    assert!(ctx
        .report_generate("summary", serde_json::json!({}))
        .is_err());
}

#[test]
fn known_capabilities_include_core_set() {
    for cap in [
        "entity.read",
        "device.read",
        "readiness.read",
        "health.read",
        "report.generate",
    ] {
        assert!(KNOWN_PLUGIN_CAPABILITIES.contains(&cap));
    }
}

#[test]
fn security_validation_rejects_incompatible_api() {
    let mut manifest = parse_example();
    manifest.compatibility.api_version = "v99".into();
    let report = validate_install_security(
        &manifest,
        "0.4.0",
        None,
        None,
        PluginTrustTier::Community,
        false,
    )
    .unwrap();
    assert!(!report.approved);
}

#[test]
fn manifest_only_loader_uses_wasm_format() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("spanda.plugin.toml"), EXAMPLE_MANIFEST).unwrap();
    let mut audit = spanda_plugin::audit::PluginAuditLog::default();
    let loaded = spanda_plugin::loader::PluginLoader::load(dir.path(), &mut audit).unwrap();
    assert_eq!(loaded.format, LoadFormat::Wasm);
}

#[test]
fn platform_event_maps_to_readiness_hook() {
    use spanda_plugin::bridge::hook_for_platform_event;
    assert_eq!(
        hook_for_platform_event("ReadinessChanged").map(|h| h.as_str()),
        Some("on_readiness_completed")
    );
}

#[test]
fn cli_command_matching_and_dispatch() {
    let workspace = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let example = workspace.join("examples/plugins/readiness-plugin");
    assert!(example.is_dir(), "missing {}", example.display());
    let root = tempfile::tempdir().unwrap();
    let mut manager = PluginManager::open(root.path(), "0.4.0").unwrap();
    manager
        .store_mut()
        .install_from_dir(&example, "0.4.0", true)
        .unwrap();
    manager.store_mut().enable("spanda-plugin-readiness-example").unwrap();
    assert!(manager
        .try_cli_command("healthcare", "check")
        .is_some());
    let result = manager
        .run_cli_command("healthcare", "check", &[])
        .unwrap()
        .expect("hook result");
    assert!(result.success);
}
