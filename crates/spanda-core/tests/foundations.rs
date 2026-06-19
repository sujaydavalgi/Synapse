use spanda_core::{check, compile, run, RunOptions};

#[test]
fn module_struct_enum_trait_and_match() {
    let source = r#"
module navigation;

import sensors.lidar;
import motion.drive;

struct Pose {
  x: Distance;
  y: Distance;
  heading: Angle;
}

enum RobotState {
  Idle,
  Navigating,
  EmergencyStop
}

trait Navigator {
  fn plan(goal: Pose) -> Path;
}

robot Rover {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  safety {
    max_speed = 1.0 m/s;
  }

  behavior run() {
    let state = "Idle";
    match state {
      Idle => wheels.stop();
      Navigating => wheels.drive(linear: 0.3 m/s, angular: 0.0 rad/s);
      EmergencyStop => emergency_stop;
    };
  }
}
"#;
    compile(source).expect("compile foundations example");
    check(source).expect("type-check foundations example");
}

#[test]
fn agent_capabilities_task_state_machine_event_twin() {
    let source = r#"
robot DeliveryBot {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  safety {
    max_speed = 1.0 m/s;
    stop_if lidar.nearest_distance < 0.5 m;
  }

  event ObstacleDetected;

  on ObstacleDetected {
    wheels.stop();
  }

  twin DeliveryTwin {
    mirror pose;
    replay true;
  }

  ai_model planner: LLM {
    provider: "mock";
    model: "safe-planner";
    temperature: 0.1;
  }

  agent Navigator {
    uses planner;
    tools [lidar, wheels];
    memory short_term;
    skill path_planning;
    goal "Deliver safely";
    can [ read(lidar), propose_motion ];

    plan {
      let scan = lidar.read();
      let proposal = planner.reason(prompt: "Plan safe motion", input: scan);
      let action = safety.validate(proposal);
      wheels.execute(action);
    }
  }

  state_machine Delivery {
    state Idle;
    state Navigate;
    state Deliver;
    transition Idle -> Navigate;
    transition Navigate -> Deliver;
  }

  task control_loop every 20ms requires lidar.nearest_distance > 0.3 m {
    Navigator.plan();
  }
}
"#;
    compile(source).expect("compile autonomous primitives");
    run(source, RunOptions { max_loop_iterations: 3, ..Default::default() })
        .expect("run autonomous primitives");
}

#[test]
fn behavior_contracts_type_check() {
    let source = r#"
robot R {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  behavior move() requires lidar.nearest_distance > 0.5 m ensures true {
    wheels.drive(linear: 0.2 m/s, angular: 0.0 rad/s);
  }
}
"#;
    check(source).expect("contracts should type-check");
}

#[test]
fn enter_stmt_transitions_state_machine() {
    let source = r#"
robot Bot {
  state_machine Flow {
    state Idle;
    state Loading;
    transition Idle -> Loading;
  }

  behavior run() {
    enter Loading;
  }
}
"#;
    let result = run(
        source,
        RunOptions {
            max_loop_iterations: 1,
            ..Default::default()
        },
    )
    .expect("enter should run");
    assert!(
        result
            .logs
            .iter()
            .any(|l| l.contains("state_machine Flow: Idle -> Loading")),
        "expected transition log, got: {:?}",
        result.logs
    );
}

#[test]
fn enter_stmt_rejects_invalid_transition() {
    let source = r#"
robot Bot {
  state_machine Flow {
    state Idle;
    state Loading;
    transition Idle -> Loading;
  }

  behavior run() {
    enter Idle;
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
    .expect_err("enter to current state without transition should fail");
    assert!(
        err.to_string().contains("No valid transition"),
        "expected transition error, got: {err}"
    );
}

#[test]
fn emit_event_dispatches_handler() {
    let source = r#"
robot Bot {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  safety {
    max_speed = 1.0 m/s;
  }

  event ObstacleDetected;

  on ObstacleDetected {
    wheels.stop();
  }

  behavior run() {
    emit ObstacleDetected;
  }
}
"#;
    let result = run(
        source,
        RunOptions {
            max_loop_iterations: 1,
            ..Default::default()
        },
    )
    .expect("emit should run");
    assert!(
        result.logs.iter().any(|l| l.contains("emit ObstacleDetected")),
        "expected emit log, got: {:?}",
        result.logs
    );
    assert!(
        result.logs.iter().any(|l| l.contains("wheels.stop")),
        "expected handler to run wheels.stop(), got: {:?}",
        result.logs
    );
}
