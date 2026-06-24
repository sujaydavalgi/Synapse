//! Runtime mission operator approval integration tests.

use spanda_interpreter::{run_program, RunOptions};
use spanda_lexer::tokenize;
use spanda_parser::parse;

#[test]
fn mission_advance_requires_operator_approval() {
    let source = r#"
robot GateBot {
    topic gate_approval: Approval subscribe on "/gate/approval";
    actuator gate: DifferentialDrive;
    mode hold { }
    mission OpenGate {
        requires approval Operator for: open_sequence;
        open_sequence;
    }
    behavior open_sequence() { gate.drive(linear: 0.0 m/s, angular: 0.0 rad/s); }
    behavior runner() {
        mission.start();
        let _ = mission.advance();
    }
}
"#;
    std::env::remove_var("SPANDA_OPERATOR_APPROVAL");
    let program = parse(tokenize(source).unwrap()).unwrap();
    let denied = run_program(
        &program,
        RunOptions {
            entry_behavior: Some("runner".into()),
            max_loop_iterations: 1,
            ..Default::default()
        },
    );
    assert!(
        denied.is_err() || {
            denied
                .as_ref()
                .map(|r| {
                    r.logs
                        .iter()
                        .any(|l| l.contains("operator approval required"))
                })
                .unwrap_or(false)
        }
    );

    let approved = run_program(
        &program,
        RunOptions {
            entry_behavior: Some("runner".into()),
            max_loop_iterations: 1,
            inbound_comm_messages: vec![("/gate/approval".into(), "open_sequence".into())],
            ..Default::default()
        },
    )
    .expect("approved run");
    assert!(
        approved
            .logs
            .iter()
            .any(|l| l.contains("operator approval granted") || l.contains("open_sequence")),
        "expected approval grant or step advance, got: {:?}",
        approved.logs
    );
}
