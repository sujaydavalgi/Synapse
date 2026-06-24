//! Runtime wiring for program-level state_estimator declarations.

use spanda_interpreter::{run_program, RunOptions};
use spanda_lexer::tokenize;
use spanda_parser::parse;

#[test]
fn state_estimator_registers_fusion_at_runtime() {
    let source = r#"
hardware H {
    sensors [GPS, Lidar];
    actuators [DifferentialDrive];
}

state_estimator RoverState {
    inputs [gps.fix, lidar.read];
    output StateEstimate;
}

robot Rover {
    sensor gps: GPS;
    sensor lidar: Lidar;
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }
    behavior fuse() {
        let fused = fusion.read();
        let _ = fused.state_estimate;
    }
}
"#;
    let program = parse(tokenize(source).unwrap()).unwrap();
    let result = run_program(&program, RunOptions::default()).expect("run");
    assert!(
        result
            .logs
            .iter()
            .any(|line| line.contains("state_estimator 'RoverState'")),
        "expected state_estimator setup log, got: {:?}",
        result.logs
    );
    assert!(
        result
            .logs
            .iter()
            .any(|line| line.contains("state_estimator: aliased fusion binding")),
        "expected fusion alias log, got: {:?}",
        result.logs
    );
}
