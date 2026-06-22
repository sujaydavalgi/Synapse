//! Phase 32 gap-closure tests: IoT hub, task return types, agent can[] enforcement.

use spanda_core::{check, run, RunOptions};

#[test]
fn task_return_type_mismatch_fails_check() {
    let source = r#"
robot Rover {
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }

    task Monitor every 50ms -> Bool {
        return 1;
    }
}
"#;
    let err = check(source).expect_err("task return mismatch should fail");
    let text = err
        .diagnostics()
        .into_iter()
        .map(|d| d.message)
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        text.contains("Bool") || text.contains("Number"),
        "expected return type error, got {text}"
    );
}

#[test]
fn empty_can_denies_propose_motion_at_runtime() {
    let source = r#"
robot Rover {
    sensor lidar: Lidar on "/scan";
    ai_model planner: LLM { provider: "mock"; model: "test"; }
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }

    agent Nav {
        uses planner;
        can [];

        plan {
            let scan = lidar.read();
            let proposal = planner.reason(prompt: "go", input: scan);
            let action = safety.validate(proposal);
            wheels.execute(action);
        }
    }

    behavior run() {
        Nav.plan();
    }
}
"#;
    let err = run(source, RunOptions::default()).expect_err("empty can should deny motion");
    let text = format!("{err}");
    assert!(
        text.contains("capability") || text.contains("can[]") || text.contains("propose_motion"),
        "expected capability denial, got {text}"
    );
}
