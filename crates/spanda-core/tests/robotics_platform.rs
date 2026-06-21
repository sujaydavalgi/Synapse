//! Robotics platform integration tests.
//!
use spanda_core::{check, compile, run, RunOptions};

#[test]
fn mission_with_steps_parses_and_runs() {
    let source = r#"
robot DeliveryBot {
  actuator wheels: DifferentialDrive;

  safety { max_speed = 1.0 m/s; }

  mission Delivery {
    duration: 20 min;
    navigate;
    deliver;
    return_home;
  }

  behavior execute_mission() {
    mission.start();
    let _ = mission.state();
    let step = mission.advance();
    let _ = step;
    navigation.goal("Dock at charger");
    let _ = navigation.path();
    let _ = navigation.navigate();
    mission.complete();
    wheels.stop();
  }
}
"#;
    check(source).expect("mission with steps should type-check");
    let result = run(source, RunOptions::default()).expect("mission program should run");
    assert!(
        result.logs.iter().any(|l| l.contains("mission 'Delivery'")),
        "expected mission setup log, got: {:?}",
        result.logs
    );
}

#[test]
fn fleet_decl_validates_members() {
    let source = r#"
robot Picker1 {
  actuator wheels: DifferentialDrive;
  behavior idle() { wheels.stop(); }
}

robot Picker2 {
  actuator wheels: DifferentialDrive;
  behavior idle() { wheels.stop(); }
}

fleet Warehouse {
  Picker1;
  Picker2;
}

robot Coordinator {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let count = fleet.members("Warehouse");
    let _ = count;
    wheels.stop();
  }
}
"#;
    check(source).expect("fleet declaration should type-check");
    run(source, RunOptions::default()).expect("fleet program should run");
}

#[test]
fn fleet_unknown_member_rejected() {
    let source = r#"
robot Picker1 {
  actuator wheels: DifferentialDrive;
  behavior idle() { wheels.stop(); }
}

fleet Warehouse {
  MissingBot;
}
"#;
    let err = check(source).expect_err("unknown fleet member should fail type-check");
    assert!(
        err.diagnostics()
            .iter()
            .any(|d| d.message.contains("unknown robot 'MissingBot'")),
        "expected fleet member error, got: {:?}",
        err.diagnostics()
    );
}

#[test]
fn program_safety_zone_parses() {
    let source = r#"
safety_zone HumanArea {
  max_speed 0.5 m/s;
}

robot ZoneBot {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  safety {
    max_speed = 1.0 m/s;
    zone HumanArea circle at (0.0 m, 0.0 m) radius 2.0 m;
    stop_if robot.in_zone("HumanArea");
  }

  behavior patrol() {
    wheels.drive(linear: 0.2 m/s, angular: 0.0 rad/s);
  }
}
"#;
    compile(source).expect("program safety_zone should compile");
    let result = run(source, RunOptions::default()).expect("safety zone program should run");
    assert!(
        result
            .logs
            .iter()
            .any(|l| l.contains("safety_zone 'HumanArea'")),
        "expected safety_zone log, got: {:?}",
        result.logs
    );
}

#[test]
fn fusion_returns_confidence() {
    let source = r#"
robot FusionBot {
  sensor camera: Camera on "/camera";
  sensor lidar: Lidar on "/scan";
  sensor imu: IMU;
  actuator wheels: DifferentialDrive;

  observe { camera; lidar; imu; }

  behavior run() {
    let fused = fusion.read();
    let _ = fused.count;
    let _ = fused.pose;
    wheels.stop();
  }
}
"#;
    check(source).expect("fusion confidence fields should type-check");
    run(source, RunOptions::default()).expect("fusion program should run");
}

#[test]
fn program_safety_zone_caps_motion() {
    let source = r#"
safety_zone HumanArea {
  max_speed 0.5 m/s;
}

robot ZoneBot {
  actuator wheels: DifferentialDrive;

  safety {
    max_speed = 1.0 m/s;
    zone HumanArea circle at (0.0 m, 0.0 m) radius 2.0 m;
  }

  behavior drive() {
    wheels.drive(linear: 0.8 m/s, angular: 0.0 rad/s);
  }
}
"#;
    run(source, RunOptions::default()).expect("zone cap program should run");
}

#[test]
fn legacy_mission_duration_still_works() {
    let source = r#"
robot LegacyRover {
  actuator wheels: DifferentialDrive;
  mission { duration: 15 min; }
  behavior run() { wheels.stop(); }
}
"#;
    check(source).expect("legacy mission duration should still type-check");
}

#[test]
fn certify_metadata_parses() {
    let source = r#"
certify ISO13849;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}
"#;
    check(source).expect("certify metadata should type-check");
}

#[test]
fn certify_unknown_standard_rejected() {
    let source = r#"
certify UNKNOWN;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}
"#;
    assert!(check(source).is_err());
}

#[test]
fn certify_duplicate_rejected() {
    let source = r#"
certify ISO13849;
certify ISO13849;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}
"#;
    assert!(check(source).is_err());
}
