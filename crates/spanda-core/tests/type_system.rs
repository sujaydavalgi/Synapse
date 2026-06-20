use spanda_core::{check, compile, run, RunOptions};

#[test]
fn foundation_types_with_annotations() {
    let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let count: Int = 3;
    let label: String = "rover";
    let active: Bool = true;
    let _ = count;
    wheels.stop();
  }
}
"#;
    check(source).expect("foundation types should type-check");
}

#[test]
fn generic_collections_type_check() {
    let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let goals: Array<Goal> = goals_placeholder;
    let _ = goals;
    wheels.stop();
  }
}
"#;
    let err = check(source).expect_err("undefined goals_placeholder should fail");
    assert!(
        err.diagnostics()
            .iter()
            .any(|d| d.message.contains("Undefined")),
        "got {:?}",
        err.diagnostics()
    );
    let parse_only = r#"
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let goals: Array<Goal>;
    let scan_topic: Topic<LidarScan>;
    let svc: Service<Command, Feedback>;
    wheels.stop();
  }
}
"#;
    check(parse_only).expect("generic type annotations should parse and type-check");
}

#[test]
fn generic_arity_mismatch_fails() {
    let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let bad: Array<Int, Float>;
    wheels.stop();
  }
}
"#;
    let err = check(source).expect_err("Array arity mismatch should fail at parse");
    assert!(
        err.to_string().contains("expects 1")
            || err
                .diagnostics()
                .iter()
                .any(|d| d.message.contains("expects 1")),
        "got {err}"
    );
}

#[test]
fn unit_literals_and_valid_operations() {
    let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let timeout: Duration = 500 ms;
    let speed: Velocity = 1.5 m/s;
    let distance: Distance = 2.0 m;
    let total: Distance = distance + 1.0 m;
    let mixed: Duration = 500 ms + 0.5 s;
    let payload: Mass = 2.5 kg;
    let temp: Temperature = 25 celsius;
    let pressure: Pressure = 101.3 kPa;
    let _ = total;
    let _ = mixed;
    let _ = payload;
    let _ = temp;
    let _ = pressure;
    wheels.stop();
  }
}
"#;
    check(source).expect("valid unit operations should pass");
}

#[test]
fn extended_unit_literals_type_check() {
    let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let d: Distance = 100 cm;
    let v: Velocity = 36 km/h;
    let a: Acceleration = 2 g;
    let angle: Angle = 90 deg;
    let w: AngularVelocity = 180 deg/s;
    let f: Force = 10 N;
    let p: Power = 500 W;
    let volts: Voltage = 12 V;
    let amps: Current = 2 A;
    let _ = d + 1 m;
    wheels.stop();
  }
}
"#;
    check(source).expect("extended units should type-check");
}

#[test]
fn sensor_environmental_units_type_check() {
    let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let humidity: Humidity = 65 %RH;
    let light: Illuminance = 320 lux;
    let co2: Concentration = 800 ppm;
    let noise: SoundLevel = 42 dBA;
    let uv: UvIndex = 6.5 uvi;
    let acidity: Ph = 7.2 pH;
    let ec: Conductivity = 850 uS/cm;
    let pm25: ParticulateMatter = 12 ug/m3;
    let turbidity: Turbidity = 4.5 NTU;
    let salt: Salinity = 35 ppt;
    let dose: Radiation = 0.12 uSv/h;
    let soil: SoilMoisture = 42 %VWC;
    let _ = humidity + 5 rh;
    wheels.stop();
  }
}
"#;
    check(source).expect("sensor environmental units should type-check");
}

#[test]
fn invalid_unit_operation_fails() {
    let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let speed: Velocity = 1.0 m/s;
    let distance: Distance = 2.0 m;
    let bad = speed + distance;
    wheels.stop();
  }
}
"#;
    let err = check(source).expect_err("speed + voltage should fail");
    assert!(
        err.diagnostics()
            .iter()
            .any(|d| d.message.contains("incompatible")),
        "got {:?}",
        err.diagnostics()
    );
}

#[test]
fn distance_plus_duration_fails() {
    let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let d: Distance = 1.0 m;
    let t: Duration = 500 ms;
    let bad = d + t;
    wheels.stop();
  }
}
"#;
    let err = check(source).expect_err("distance + duration should fail");
    assert!(
        err.diagnostics()
            .iter()
            .any(|d| d.message.contains("incompatible")),
        "got {:?}",
        err.diagnostics()
    );
}

#[test]
fn spatial_sensor_and_ai_types_parse() {
    let source = r#"
robot R {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;
  ai_model planner: LLM { provider: "mock"; model: "p"; temperature: 0.1; }
  behavior run() {
    let pose: Pose;
    let path: Path;
    let scan: LidarScan;
    let frame: CameraFrame;
    let prompt: Prompt;
    wheels.stop();
  }
}
"#;
    check(source).expect("spatial/sensor/ai annotations should type-check");
}

#[test]
fn action_proposal_cannot_execute_directly() {
    let source = r#"
robot R {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;
  ai_model planner: LLM { provider: "mock"; model: "p"; temperature: 0.1; }
  behavior run() {
    let proposal: ActionProposal = planner.reason(prompt: "go");
    wheels.execute(proposal);
  }
}
"#;
    let err = check(source).expect_err("ActionProposal execute should fail typecheck");
    assert!(
        err.diagnostics()
            .iter()
            .any(|d| { d.message.contains("ActionProposal") && d.message.contains("execute") }),
        "got {:?}",
        err.diagnostics()
    );
}

#[test]
fn safe_action_can_execute() {
    let source = r#"
robot R {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;
  ai_model planner: LLM { provider: "mock"; model: "p"; temperature: 0.1; }
  safety { max_speed = 1.0 m/s; }
  behavior run() {
    let proposal: ActionProposal = planner.reason(prompt: "go");
    let action: SafeAction = safety.validate(proposal);
    wheels.execute(action);
  }
}
"#;
    check(source).expect("SafeAction execute should type-check");
}

#[test]
fn unknown_type_fails_at_parse() {
    let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let x: NotARealType;
    wheels.stop();
  }
}
"#;
    let err = check(source).expect_err("unknown type should fail");
    assert!(
        err.to_string().contains("Unknown type")
            || err
                .diagnostics()
                .iter()
                .any(|d| d.message.contains("Unknown type")),
        "got {err}"
    );
}

#[test]
fn goal_type_and_agent_goal_injection() {
    let source = r#"
robot R {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  ai_model planner: LLM { provider: "mock"; model: "p"; temperature: 0.1; }
  agent Navigator {
    uses planner;
    tools [lidar, wheels];
    goal "Reach dock";
    can [ read(lidar), propose_motion, plan ];
    plan {
      let mission: Goal = "Reach dock";
      let scan = lidar.read();
      let proposal = planner.reason(prompt: "go", input: scan, goal: mission);
      let trace = proposal.trace;
      let _ = trace;
      let action = safety.validate(proposal);
      wheels.execute(action);
    }
  }
  behavior run() {
    let g: Goal = goal(text: "Deliver package");
    let _ = g;
    Navigator.plan();
  }
}
"#;
    check(source).expect("goal types and agent goal injection should type-check");
    run(
        source,
        RunOptions {
            max_loop_iterations: 1,
            ..Default::default()
        },
    )
    .expect("goal example should run");
}

#[test]
fn goals_example_runs() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/types/goals.sd"
    ))
    .expect("read goals example");
    compile(&source).expect("goals example should compile");
    run(&source, RunOptions::default()).expect("goals example should run");
}

#[test]
fn memory_remember_and_recall() {
    let source = r#"
robot R {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  ai_model planner: LLM { provider: "mock"; model: "p"; temperature: 0.1; }
  agent Navigator {
    uses planner;
    tools [lidar, wheels];
    memory short_term;
    can [ read(lidar), propose_motion, plan ];
    plan {
      let scan = lidar.read();
      remember "last_scan", scan;
      let recalled = recall("last_scan");
      let _ = recalled;
      let proposal = planner.reason(prompt: "go", input: scan);
      let action = safety.validate(proposal);
      wheels.execute(action);
    }
  }
  behavior run() { Navigator.plan(); }
}
"#;
    check(source).expect("remember/recall should type-check");
    run(
        source,
        RunOptions {
            max_loop_iterations: 1,
            ..Default::default()
        },
    )
    .expect("remember/recall should run");
}

#[test]
fn memory_example_runs() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/types/memory.sd"
    ))
    .expect("read memory example");
    compile(&source).expect("memory example should compile");
    run(&source, RunOptions::default()).expect("memory example should run");
}

#[test]
fn verify_block_type_checks_and_runs() {
    let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  safety { max_speed = 2.0 m/s; }
  verify {
    robot.velocity().linear <= 2.0 m/s;
  }
  behavior run() {
    wheels.drive(linear: 0.5 m/s, angular: 0.0 rad/s);
  }
}
"#;
    check(source).expect("verify block should type-check");
    run(source, RunOptions::default()).expect("verify block should pass at runtime");
}

#[test]
fn verify_example_runs() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/types/verify.sd"
    ))
    .expect("read verify example");
    compile(&source).expect("verify example should compile");
    run(&source, RunOptions::default()).expect("verify example should run");
}

#[test]
fn safety_example_runs() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/types/safety.sd"
    ))
    .expect("read safety example");
    compile(&source).expect("safety example should compile");
    run(&source, RunOptions::default()).expect("safety example should run");
}
