//! Ensure spanda-package does not depend on spanda-core (Phase 4 cycle break).
//!
#[test]
fn package_manifest_has_no_spanda_core_dependency() {
    let manifest = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"),
    )
    .expect("read Cargo.toml");
    assert!(
        !manifest.contains("spanda-core"),
        "spanda-package must not depend on spanda-core"
    );
}

#[test]
fn permissive_permissions_use_hardware_catalog() {
    let perms = spanda_package::validation::ApplicationPermissions::permissive();
    assert!(perms.hardware_targets.iter().any(|t| t == "JetsonOrin"));
}
