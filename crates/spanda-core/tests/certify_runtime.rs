//! Runtime certification gate tests.

use spanda_core::{compile, enforce_certification_runtime, run, RunOptions};

#[test]
fn enforce_certify_blocks_deploy_without_metadata() {
    let source = r#"
hardware Tiny { actuators [ DifferentialDrive ]; }
robot Rover {
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}
deploy Rover to Tiny;
"#;
    let program = compile(source).expect("compile").program;
    let err = enforce_certification_runtime(&program, true).expect_err("should block");
    assert!(err.to_string().contains("certification runtime gate"));
}

#[test]
fn certified_example_passes_runtime_gate() {
    let source = include_str!("../../../examples/robotics/certified_deployment.sd");
    let program = compile(source).expect("compile").program;
    enforce_certification_runtime(&program, true).expect("certified example should pass");
}

#[test]
fn run_with_enforce_certify_flag_fails_for_ota_example() {
    let source = include_str!("../../../examples/robotics/ota_deployment.sd");
    let result = run(
        source,
        RunOptions {
            enforce_certify: true,
            max_loop_iterations: 1,
            ..Default::default()
        },
    );
    assert!(result.is_err());
}
