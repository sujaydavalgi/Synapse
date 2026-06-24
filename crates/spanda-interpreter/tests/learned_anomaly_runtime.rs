//! Runtime learned anomaly backend polling during health transitions.

use spanda_interpreter::{run_program, RunOptions};
use spanda_lexer::tokenize;
use spanda_parser::parse;

#[test]
fn learned_anomaly_backend_triggers_handler_on_health_fault() {
    let source = r#"
hardware H {
    sensors [GPS, Lidar];
    actuators [DifferentialDrive];
}

anomaly_detector NavigationML {
    learned backend assurance.anomaly;
    expected localization.confidence >= 0.80;
}

on anomaly NavigationML severity High {
    enter degraded_mode;
}

robot Rover {
    sensor gps: GPS;
    sensor lidar: Lidar;
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }
    behavior patrol() {
        loop every 50ms {
            let _ = gps.read();
        }
    }
}
"#;
    let program = parse(tokenize(source).unwrap()).unwrap();
    let result = run_program(
        &program,
        RunOptions {
            inject_health_faults: true,
            max_loop_iterations: 3,
            ..Default::default()
        },
    )
    .expect("run");
    assert!(
        result
            .logs
            .iter()
            .any(|line| line.contains("learned anomaly: NavigationML")),
        "expected learned anomaly scan log, got: {:?}",
        result.logs
    );
    assert!(
        result
            .logs
            .iter()
            .any(|line| line.contains("anomaly: applying handler for NavigationML")),
        "expected anomaly handler log, got: {:?}",
        result.logs
    );
}
