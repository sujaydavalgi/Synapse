//! Runtime recovery action dispatch integration tests.

use spanda_interpreter::{run_program, RunOptions};
use spanda_lexer::tokenize;
use spanda_parser::parse;

#[test]
fn recovery_auto_triggers_during_run_on_health_fault() {
    let source = r#"
hardware H {
    sensors [GPS, Lidar];
    actuators [DifferentialDrive];
}

recovery_policy RoverRecovery {
    on gps.failed {
        enter degraded_mode;
        reduce_speed 0.4 m/s;
    }
}

robot Rover {
    sensor gps: GPS;
    sensor lidar: Lidar;
    actuator wheels: DifferentialDrive;
    mode degraded { }
    safety { max_speed = 1.0 m/s; }
    behavior patrol() {
        loop every 50ms {
            let _ = gps.read();
        }
    }
}
"#;
    std::env::set_var("SPANDA_OPERATOR_APPROVAL", "1");
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
            .any(|l| l.contains("recovery: auto-triggered")),
        "expected recovery auto-trigger log, got: {:?}",
        result.logs
    );
}

#[test]
fn recovery_policy_dispatches_degraded_mode_on_health_fault() {
    // Description:
    //     Recovery policy dispatches degraded mode on health fault.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_interpreter::recovery_runtime::recovery_policy_dispatches_degraded_mode_on_health_fault();

    let source = r#"
hardware H {
    sensors [GPS, Lidar];
    actuators [DifferentialDrive];
}

recovery_policy RoverRecovery {
    on gps.failed {
        enter degraded_mode;
        reduce_speed 0.4 m/s;
    }
}

on anomaly NavigationFault severity High {
    enter degraded_mode;
}

anomaly_detector NavigationFault {
    expected gps.accuracy <= 3 m;
}

robot Rover {
    sensor gps: GPS;
    sensor lidar: Lidar;
    actuator wheels: DifferentialDrive;
    mode degraded { }
    safety { max_speed = 1.0 m/s; }
    behavior patrol() {
        loop every 50ms {
            let _ = gps.read();
        }
    }
}
"#;
    std::env::set_var("SPANDA_OPERATOR_APPROVAL", "1");
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
            .any(|l| l.contains("recovery:") || l.contains("mode: entered")),
        "expected recovery or mode dispatch logs, got: {:?}",
        result.logs
    );
}

#[test]
fn approval_topic_grants_high_risk_recovery() {
    // Description:
    //     Approval topic grants high risk recovery.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_interpreter::recovery_runtime::approval_topic_grants_high_risk_recovery();

    let source = r#"
hardware H {
    sensors [GPS, Lidar];
    actuators [DifferentialDrive];
}

recovery_policy OperatorResume {
    on gps {
        resume mission;
    }
}

robot Rover {
    topic recovery_approval: Approval subscribe on "/recovery/approval";
    sensor gps: GPS;
    sensor lidar: Lidar;
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }
    recover from SensorFailure { }
    behavior patrol() {
        loop every 50ms {
            let _ = gps.read();
        }
    }
}
"#;
    std::env::remove_var("SPANDA_OPERATOR_APPROVAL");
    std::env::remove_var("SPANDA_GRANT_RECOVERY_APPROVAL");
    let program = parse(tokenize(source).unwrap()).unwrap();
    let result = run_program(
        &program,
        RunOptions {
            inject_health_faults: true,
            max_loop_iterations: 5,
            inbound_comm_messages: vec![("/recovery/approval".into(), "resume mission".into())],
            ..Default::default()
        },
    )
    .expect("run");
    assert!(
        result.logs.iter().any(|l| {
            l.contains("recovery: operator approval granted")
                || l.contains("recovery: recorded action 'resume mission'")
        }),
        "expected approval grant or resume mission dispatch, got: {:?}",
        result.logs
    );
}
