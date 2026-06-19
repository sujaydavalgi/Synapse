use spanda_core::{check, run, RunOptions};

#[test]
fn capability_denied_when_agent_lacks_read() {
    let source = r#"
robot R {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  safety {
    max_speed = 1.0 m/s;
  }

  ai_model planner: LLM {
    provider: "mock";
    model: "test";
    temperature: 0.1;
  }

  agent NoRead {
    uses planner;
    tools [lidar, wheels];
    goal "test";
    can [ propose_motion ];

    plan {
      let scan = lidar.read();
      let proposal = planner.reason(prompt: "go", input: scan);
      let action = safety.validate(proposal);
      wheels.execute(action);
    }
  }

  behavior run() {
    NoRead.plan();
  }
}
"#;
    let err = run(
        source,
        RunOptions {
            max_loop_iterations: 1,
            ..Default::default()
        },
    )
    .expect_err("agent without read(lidar) should fail at runtime");
    assert!(
        err.to_string().contains("lacks capability read(lidar)"),
        "expected capability error, got: {err}"
    );
}

#[test]
fn capability_denied_when_agent_lacks_propose_motion() {
    let source = r#"
robot R {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  safety {
    max_speed = 1.0 m/s;
  }

  ai_model planner: LLM {
    provider: "mock";
    model: "test";
    temperature: 0.1;
  }

  agent NoMotion {
    uses planner;
    tools [lidar, wheels];
    goal "test";
    can [ read(lidar) ];

    plan {
      let scan = lidar.read();
      let proposal = planner.reason(prompt: "go", input: scan);
      let action = safety.validate(proposal);
      wheels.execute(action);
    }
  }

  behavior run() {
    NoMotion.plan();
  }
}
"#;
    let err = run(
        source,
        RunOptions {
            max_loop_iterations: 1,
            ..Default::default()
        },
    )
    .expect_err("agent without propose_motion should fail at runtime");
    assert!(
        err.to_string().contains("lacks capability propose_motion"),
        "expected capability error, got: {err}"
    );
}

#[test]
fn behavior_requires_contract_fails_at_runtime() {
    let source = r#"
robot R {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  behavior move() requires lidar.nearest_distance > 100.0 m {
    wheels.drive(linear: 0.2 m/s, angular: 0.0 rad/s);
  }
}
"#;
    let err = run(
        source,
        RunOptions {
            max_loop_iterations: 1,
            ..Default::default()
        },
    )
    .expect_err("requires should fail when obstacle is closer than 100m");
    assert!(
        err.to_string().contains("requires contract failed"),
        "expected requires error, got: {err}"
    );
}

#[test]
fn behavior_ensures_contract_fails_at_runtime() {
    let source = r#"
robot R {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  behavior move() requires true ensures lidar.nearest_distance > 100.0 m {
    wheels.drive(linear: 0.2 m/s, angular: 0.0 rad/s);
  }
}
"#;
    let err = run(
        source,
        RunOptions {
            max_loop_iterations: 1,
            ..Default::default()
        },
    )
    .expect_err("ensures should fail after body runs");
    assert!(
        err.to_string().contains("ensures contract failed"),
        "expected ensures error, got: {err}"
    );
}

#[test]
fn task_requires_skips_iteration_without_aborting() {
    let source = r#"
robot R {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  task tick every 20ms requires lidar.nearest_distance > 100.0 m {
    wheels.drive(linear: 0.5 m/s, angular: 0.0 rad/s);
  }
}
"#;
    let result = run(
        source,
        RunOptions {
            max_loop_iterations: 3,
            ..Default::default()
        },
    )
    .expect("task should skip iterations when requires fails");
    assert!(
        result
            .logs
            .iter()
            .any(|l| l.contains("task requires contract failed")),
        "expected skip log, got: {:?}",
        result.logs
    );
    assert_eq!(
        result.state.velocity.linear, 0.0,
        "motion should not run when requires fails"
    );
}

#[test]
fn task_ensures_contract_fails_at_runtime() {
    let source = r#"
robot R {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  task tick every 20ms requires true ensures lidar.nearest_distance > 100.0 m {
    wheels.stop();
  }
}
"#;
    let err = run(
        source,
        RunOptions {
            max_loop_iterations: 1,
            ..Default::default()
        },
    )
    .expect_err("task ensures should fail");
    assert!(
        err.to_string().contains("task ensures contract failed"),
        "expected ensures error, got: {err}"
    );
}

#[test]
fn twin_unknown_mirror_field_rejected_at_typecheck() {
    let source = r#"
robot R {
  twin T {
    mirror telemetry;
    replay false;
  }
}
"#;
    let err = check(source).expect_err("unknown twin mirror field should fail typecheck");
    assert!(
        err.diagnostics()
            .iter()
            .any(|d| d.message.contains("Unknown twin mirror field")),
        "expected twin mirror error, got: {:?}",
        err.diagnostics()
    );
}

#[test]
fn match_non_exhaustive_enum_rejected_at_typecheck() {
    let source = r#"
enum Mode {
  Idle,
  Active,
  Stop
}

robot R {
  actuator wheels: DifferentialDrive;

  behavior run() {
    let mode = "Idle";
    match mode {
      Idle => wheels.stop();
    };
  }
}
"#;
    let err = check(source).expect_err("non-exhaustive match should fail typecheck");
    assert!(
        err.diagnostics()
            .iter()
            .any(|d| d.message.contains("Non-exhaustive match")),
        "expected exhaustiveness error, got: {:?}",
        err.diagnostics()
    );
}
