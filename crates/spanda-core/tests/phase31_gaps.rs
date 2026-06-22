//! Phase 31 gap-closure tests: health_policy runtime, behavior I/O types, agent audit.

use spanda_core::{check, run, RunOptions};
use spanda_driver::check_with_registry;

#[test]
fn health_policy_applies_on_critical_health() {
    let source = r#"
health_check RoverHealth for robot Rover {
    check gps.status == Healthy;
}

health_policy SafetyPolicy {
    on Degraded {
        enter degraded_mode;
    }
}

robot Rover {
    sensor gps: GPS;
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }

    every 50ms {
        let tick = true;
    }
}
"#;
    let result = run(
        source,
        RunOptions {
            inject_health_faults: true,
            max_loop_iterations: 3,
            ..Default::default()
        },
    )
    .expect("health policy run");
    assert!(
        result.logs.iter().any(|l| l.contains("health_policy: applying")),
        "expected health policy reaction, got {:?}",
        result.logs
    );
    assert!(
        result
            .logs
            .iter()
            .any(|l| l.contains("mode: entered 'degraded")),
        "expected degraded mode entry, got {:?}",
        result.logs
    );
}

#[test]
fn behavior_return_type_mismatch_fails_check() {
    let source = r#"
robot Rover {
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }

    behavior status() -> Bool {
        return 1;
    }
}
"#;
    let err = check(source).expect_err("return type mismatch should fail");
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
fn agent_plan_requires_safe_action_return() {
    let source = r#"
robot Rover {
    ai_model planner: LLM { provider: "mock"; model: "test"; }
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }

    agent Nav {
        uses planner;
        can [ propose_motion ];

        plan {
            return 1;
        }
    }

    behavior run() {
        Nav.plan();
    }
}
"#;
    let err = check_with_registry(source, &Default::default())
        .expect_err("agent plan should require SafeAction return");
    let text = err
        .diagnostics()
        .into_iter()
        .map(|d| d.message)
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        text.contains("SafeAction"),
        "expected SafeAction return error, got {text}"
    );
}
