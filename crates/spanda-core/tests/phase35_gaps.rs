//! Phase 35 gap-closure tests: fleet requirements, kill switch verify severity,
//! debugger task/every entry, ONNX env gate, registry mirror, and IoT live gates.

use spanda_capability::{apply_fleet_health_checks, evaluate_health_checks};
use spanda_debug::DebugOptions;
use spanda_driver::{DebugMachine, DebugStepKind};
use spanda_package::manifest::{PackageManifest, PackageSection};
use spanda_package::publish::{bundle_package, mirror_bundle_to_local_registry};
use spanda_runtime::robotics::FleetRegistry;
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn fleet_requirement_at_least_percent_evaluates() {
    // Description:
    //     Fleet requirement at least percent evaluates.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::phase35_gaps::fleet_requirement_at_least_percent_evaluates();

    let source = r#"
fleet Patrol {
    RoverA;
    RoverB;
}

health_check PatrolHealth for fleet Patrol {
    require at_least 80% robots Healthy;
    check rover.status == Healthy;
}
"#;
    let program =
        spanda_parser::parse(spanda_lexer::tokenize(source).expect("tokenize")).expect("parse");
    let mut report = evaluate_health_checks(&program);
    let mut fleets = FleetRegistry::default();
    fleets.register("Patrol", vec!["RoverA".into(), "RoverB".into()]);
    apply_fleet_health_checks(&mut report, &program, &fleets, &["RoverADegraded".into()]);
    assert!(
        report
            .checks
            .iter()
            .any(|c| c.metric.starts_with("require:")),
        "expected fleet requirement row, got {:?}",
        report.checks
    );
}

#[test]
fn remote_kill_switch_unsigned_policy_is_verification_error() {
    // Description:
    //     Remote kill switch unsigned policy is verification error.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::phase35_gaps::remote_kill_switch_unsigned_policy_is_verification_error();

    let source = r#"
kill_switch EmergencyStop {
    priority: critical;
    remote_signed;
    action { emergency_stop; }
}
"#;
    let program =
        spanda_parser::parse(spanda_lexer::tokenize(source).expect("tokenize")).expect("parse");
    let diags = spanda_capability::collect_verification_diagnostics(&program);
    assert!(
        diags
            .iter()
            .any(|d| d.category == "kill-switch" && d.severity == "error"),
        "expected error severity for remote_signed without identity policy"
    );
}

#[test]
fn debugger_steps_into_task_every_body() {
    // Description:
    //     Debugger steps into task every body.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::phase35_gaps::debugger_steps_into_task_every_body();

    let source = r#"
robot Rover {
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }

    every 50ms {
        wheels.stop();
        wheels.stop();
    }
}
"#;
    let mut machine = DebugMachine::start(source, DebugOptions::default()).expect("debug machine");
    let step_in = machine
        .run_until_pause(DebugStepKind::StepIn)
        .expect("step in");
    assert!(step_in.pauses.iter().any(|p| p.reason.contains("step")));
}

#[test]
fn onnx_provider_uses_mock_without_model_path() {
    // Description:
    //     Onnx provider uses mock without model path.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::phase35_gaps::onnx_provider_uses_mock_without_model_path();

    std::env::remove_var("SPANDA_ONNX_MODEL_PATH");
    assert!(!spanda_ai::live::live_onnx_enabled());
}

#[test]
fn publish_mirrors_bundle_to_local_registry_when_present() {
    // Description:
    //     Publish mirrors bundle to local registry when present.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::phase35_gaps::publish_mirrors_bundle_to_local_registry_when_present();

    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    let registry_root = root.join("registry/packages");
    std::fs::create_dir_all(&registry_root).expect("registry dir");
    std::fs::create_dir_all(root.join("src")).expect("src");
    std::fs::write(
        root.join("spanda.toml"),
        "[package]\nname = \"mirror-test\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(root.join("src/main.sd"), "robot R { behavior run() {} }").unwrap();
    let manifest = PackageManifest {
        package: PackageSection {
            name: "mirror-test".into(),
            version: "0.1.0".into(),
            description: None,
            license: None,
            authors: vec![],
        },
        dependencies: HashMap::new(),
        dev_dependencies: HashMap::new(),
        hardware: Default::default(),
        capabilities: Default::default(),
        requires_hardware: Default::default(),
        safety: Default::default(),
        adapter: Default::default(),
        categories: vec![],
        license_compat: vec![],
        entity_kinds: vec![],
    };
    let report = bundle_package(root, &manifest).expect("bundle");
    let mirrored = mirror_bundle_to_local_registry(root, &manifest, &report.bundle_path)
        .expect("mirror")
        .expect("mirror path");
    assert!(mirrored.exists());
    assert_eq!(
        mirrored,
        PathBuf::from(format!("{}/mirror-test/0.1.0", registry_root.display()))
    );
}

#[test]
fn live_zigbee_env_gate_defaults_off() {
    // Description:
    //     Live zigbee env gate defaults off.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::phase35_gaps::live_zigbee_env_gate_defaults_off();

    std::env::remove_var("SPANDA_LIVE_ZIGBEE");
    assert!(!spanda_providers::iot_live::live_zigbee_enabled());
}
