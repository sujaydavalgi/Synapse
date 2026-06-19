use spanda_core::{check, compile, run, RunOptions};

#[test]
fn unqualified_enum_variant_in_match() {
    let source = r#"
enum RobotState {
  Idle,
  Navigating,
  EmergencyStop
}

robot Rover {
  actuator wheels: DifferentialDrive;

  behavior run() {
    let state = Idle;
    match state {
      Idle => wheels.stop();
      Navigating => wheels.drive(linear: 0.3 m/s, angular: 0.0 rad/s);
      EmergencyStop => emergency_stop;
    };
  }
}
"#;
    compile(source).expect("enum variant program should compile");
    run(source, RunOptions::default()).expect("enum variant match should run");
}

#[test]
fn qualified_enum_variant_reference() {
    let source = r#"
enum Mode {
  Idle,
  Active
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let state = Mode.Active;
    match state {
      Idle => wheels.stop();
      Active => wheels.drive(linear: 0.1 m/s, angular: 0.0 rad/s);
    };
  }
}
"#;
    check(source).expect("qualified enum variant should type-check");
}

#[test]
fn duplicate_enum_variant_name_rejected() {
    let source = r#"
enum A { Idle, Go }
enum B { Idle, Stop }
robot R { actuator wheels: DifferentialDrive; }
"#;
    let err = check(source).expect_err("duplicate variant should fail");
    assert!(
        err.diagnostics()
            .iter()
            .any(|d| d.message.contains("Enum variant 'Idle' already declared")),
        "got {:?}",
        err.diagnostics()
    );
}
