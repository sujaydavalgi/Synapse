//! Deployment gate policy tests (production provenance hardening).

use spanda_package::{
    dependency::{LockedDependency, LockedSource},
    lockfile::{LockPackageInfo, Lockfile, LOCKFILE_FILENAME},
    MANIFEST_FILENAME,
};
use spanda_readiness::{evaluate_deployment_gates, DeploymentGatePolicy, ReadinessOptions};
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());
static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

const ROVER: &str = include_str!("../../../examples/showcase/capability_verification/rover.sd");

fn parse_source(source: &str) -> spanda_ast::nodes::Program {
    let tokens = spanda_lexer::tokenize(source).expect("tokenize");
    spanda_parser::parse(tokens).expect("parse")
}

fn minimal_program() -> spanda_ast::nodes::Program {
    parse_source(ROVER)
}

fn temp_project() -> std::path::PathBuf {
    let id = TEMP_COUNTER.fetch_add(1, Ordering::SeqCst);
    let root =
        std::env::temp_dir().join(format!("spanda-deploy-gate-{}-{}", std::process::id(), id));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create temp project");
    root
}

#[test]
fn production_policy_fails_official_name_path_override() {
    let root = temp_project();
    let evil = root.join("evil-mqtt");
    fs::create_dir_all(&evil).unwrap();
    fs::write(
        root.join(MANIFEST_FILENAME),
        format!(
            r#"
[package]
name = "demo"
version = "0.1.0"

[dependencies]
spanda-mqtt = {{ path = "{}" }}
"#,
            evil.display()
        ),
    )
    .unwrap();
    let program = minimal_program();
    let options = ReadinessOptions {
        source_path: Some(root.join("main.sd")),
        project_root: Some(root.clone()),
        ..ReadinessOptions::default()
    };
    let report = evaluate_deployment_gates(
        &program,
        ROVER,
        &options,
        &DeploymentGatePolicy::production(),
    );
    let gate = report
        .gates
        .iter()
        .find(|gate| gate.name == "official_provenance")
        .expect("official_provenance gate");
    assert!(!gate.passed);
    assert!(gate.message.contains("spanda-mqtt"));
}

#[test]
fn production_policy_requires_registry_signature_env() {
    let root = temp_project();
    fs::write(
        root.join(MANIFEST_FILENAME),
        r#"
[package]
name = "demo"
version = "0.1.0"
"#,
    )
    .unwrap();
    let _guard = ENV_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    unsafe {
        std::env::remove_var("SPANDA_REGISTRY_REQUIRE_SIGNATURE");
    }
    let program = minimal_program();
    let options = ReadinessOptions {
        source_path: Some(root.join("main.sd")),
        project_root: Some(root.clone()),
        ..ReadinessOptions::default()
    };
    let report = evaluate_deployment_gates(
        &program,
        ROVER,
        &options,
        &DeploymentGatePolicy::production(),
    );
    let gate = report
        .gates
        .iter()
        .find(|gate| gate.name == "registry_signatures")
        .expect("registry_signatures gate");
    assert!(!gate.passed);
    assert!(gate.message.contains("SPANDA_REGISTRY_REQUIRE_SIGNATURE"));
}

#[test]
fn production_policy_verifies_hosted_registry_lockfile_when_env_set() {
    let index_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../registry/index.json");
    if !index_path.is_file() {
        return;
    }
    let root = temp_project();
    fs::write(
        root.join(MANIFEST_FILENAME),
        r#"
[package]
name = "demo"
version = "0.1.0"

[dependencies]
spanda-mqtt = "0.1.0"
"#,
    )
    .unwrap();
    let lockfile = Lockfile {
        version: 1,
        package: LockPackageInfo {
            name: "demo".into(),
            version: "0.1.0".into(),
        },
        dependencies: [(
            "spanda-mqtt".into(),
            LockedDependency {
                name: "spanda-mqtt".into(),
                version: "0.1.0".into(),
                source: LockedSource::Registry {
                    registry: "spanda".into(),
                },
                checksum: None,
            },
        )]
        .into(),
    };
    lockfile
        .save(&root.join(LOCKFILE_FILENAME))
        .expect("save lockfile");

    let _guard = ENV_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    unsafe {
        std::env::set_var("SPANDA_REGISTRY_REQUIRE_SIGNATURE", "1");
    }

    let program = minimal_program();
    let options = ReadinessOptions {
        source_path: Some(root.join("main.sd")),
        project_root: Some(root.clone()),
        ..ReadinessOptions::default()
    };
    let report = evaluate_deployment_gates(
        &program,
        ROVER,
        &options,
        &DeploymentGatePolicy::production(),
    );
    let gate = report
        .gates
        .iter()
        .find(|gate| gate.name == "registry_signatures")
        .expect("registry_signatures gate");
    assert!(gate.passed, "{}", gate.message);
}
