use spanda_core::{run, RunOptions};

#[test]
fn twin_sync_parses_and_mirrors_telemetry() {
    let source = r#"
robot R {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  twin sync {
    telemetry;
    replay;
  }

  safety { max_speed = 1.0 m/s; }

  task sync every 50ms {
    wheels.drive(linear: 0.1 m/s, angular: 0.0 rad/s);
  }
}
"#;
    let result = run(
        source,
        RunOptions {
            max_loop_iterations: 2,
            ..Default::default()
        },
    )
    .expect("twin sync should run");
    assert!(
        result.logs.iter().any(|l| l.contains("twin sync for R")),
        "expected twin sync init log, got: {:?}",
        result.logs
    );
}

#[test]
fn twin_frame_count_grows_during_task() {
    let source = r#"
robot R {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  twin Shadow {
    mirror pose;
    replay true;
  }

  safety {
    max_speed = 1.0 m/s;
  }

  task sync every 50ms {
    wheels.drive(linear: 0.2 m/s, angular: 0.0 rad/s);
    let frames = Shadow.frame_count();
    if frames < 1 {
      wheels.stop();
    }
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
    .expect("twin task should run");
    assert!(
        result.logs.iter().any(|l| l.contains("replay frames=3")),
        "expected replay buffer to accumulate, got: {:?}",
        result.logs
    );
}

#[test]
fn twin_pose_returns_current_shadow() {
    let source = r#"
robot R {
  actuator wheels: DifferentialDrive;

  twin Shadow {
    mirror pose;
    replay true;
  }

  task sync every 50ms {
    wheels.drive(linear: 0.3 m/s, angular: 0.0 rad/s);
    let shadow_pose = Shadow.pose();
    let _x = shadow_pose.x;
  }
}
"#;
    run(
        source,
        RunOptions {
            max_loop_iterations: 2,
            ..Default::default()
        },
    )
    .expect("twin.pose() should return shadow pose");
}

#[test]
fn twin_replay_returns_historical_frame() {
    let source = r#"
robot R {
  actuator wheels: DifferentialDrive;

  twin Shadow {
    mirror pose;
    replay true;
  }

  task sync every 50ms {
    wheels.drive(linear: 0.3 m/s, angular: 0.0 rad/s);
    let frames = Shadow.frame_count();
    if frames >= 2 {
      let first = Shadow.replay(index: 0, field: pose);
      let _x = first.x;
    }
  }
}
"#;
    run(
        source,
        RunOptions {
            max_loop_iterations: 3,
            ..Default::default()
        },
    )
    .expect("twin.replay() should return historical pose");
}

#[test]
fn twin_replay_disabled_errors_at_runtime() {
    let source = r#"
robot R {
  twin Shadow {
    mirror pose;
    replay false;
  }

  behavior run() {
    Shadow.replay(index: 0, field: pose);
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
    .expect_err("replay on disabled twin should fail");
    assert!(
        err.to_string().contains("replay is disabled"),
        "expected replay disabled error, got: {err}"
    );
}
