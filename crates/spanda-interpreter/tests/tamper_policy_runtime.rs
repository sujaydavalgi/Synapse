//! Runtime tamper policy dispatch integration tests.

use spanda_interpreter::{run_program, RunOptions};
use spanda_lexer::tokenize;
use spanda_parser::parse;

#[test]
fn tamper_policy_dispatches_on_security_fault_injection() {
    let source = r#"
hardware H {
    sensors [GPS];
    actuators [DifferentialDrive];
}

tamper_policy CriticalResponse {
    on tamper severity Critical {
        enter degraded_mode;
        audit.record("critical_tamper_detected");
    }
    on tamper signal InvalidSignature {
        stop_all_actuators();
    }
}

robot Rover {
    sensor gps: GPS;
    actuator wheels: DifferentialDrive;
    mode degraded { }
    safety { max_speed = 1.0 m/s; }
    behavior patrol() {
        wheels.drive(0.1 m/s);
    }
}
"#;
    let program = parse(tokenize(source).unwrap()).unwrap();
    let result = run_program(
        &program,
        RunOptions {
            inject_security_faults: true,
            max_loop_iterations: 2,
            ..Default::default()
        },
    )
    .expect("run");
    assert!(
        result
            .logs
            .iter()
            .any(|line| line.contains("tamper:") && line.contains("action")),
        "expected tamper policy dispatch logs, got: {:?}",
        result.logs
    );
}

#[test]
fn tamper_policy_defers_destructive_action_without_operator_approval() {
    let source = r#"
hardware H {
    sensors [GPS];
    actuators [ DifferentialDrive ];
}

tamper_policy StopOnReplay {
    on tamper signal ReplayAttack {
        stop_all_actuators();
    }
}

robot Rover {
    sensor gps: GPS;
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }
    behavior patrol() {
        wheels.drive(0.1 m/s);
    }
}
"#;
    std::env::remove_var("SPANDA_OPERATOR_APPROVAL");
    let program = parse(tokenize(source).unwrap()).unwrap();
    let result = run_program(
        &program,
        RunOptions {
            inject_security_faults: true,
            max_loop_iterations: 1,
            ..Default::default()
        },
    )
    .expect("run");
    assert!(
        result.logs.iter().any(|line| {
            line.contains("pending operator approval") || line.contains("operator approval required")
        }),
        "expected deferred critical tamper action, got: {:?}",
        result.logs
    );
    std::env::remove_var("SPANDA_OPERATOR_APPROVAL");
}
