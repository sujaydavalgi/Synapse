//! Tests for GPS/GNSS positioning and wireless connectivity language features.
//!
use spanda_core::{
    check, verify_compatibility, verify_compatibility_target, CompatSeverity, VerifyOptions,
};

#[test]
fn gps_gnss_sensor_types_parse() {
    let source = r#"
robot Rover {
  sensor gps: GPS on "/gps";
  sensor gnss: GNSS on "/gnss";
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}
"#;
    check(source).expect("GPS and GNSS sensors should type-check");
}

#[test]
fn positioning_types_and_geo_builtin() {
    let source = r#"
robot Rover {
  sensor gps: GPS on "/gps";
  actuator wheels: DifferentialDrive;

  behavior run() {
    let home: GeoPoint = geo(30.2672, -97.7431);
    let fix: GpsFix;
    wheels.stop();
  }
}
"#;
    check(source).expect("positioning types and geo() should type-check");
}

#[test]
fn hardware_connectivity_block_parses() {
    let source = r#"
hardware RoverV2 {
  connectivity [
    WiFi6,
    Bluetooth5,
    LTE,
    GPS
  ];
  sensors [ GPS, IMU ];
  actuators [ DifferentialDrive ];
}

robot Rover {
  sensor gps: GPS;
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}

deploy Rover to RoverV2;
"#;
    check(source).expect("hardware connectivity block should parse");
    let report = verify_compatibility_target(source, None).expect("verify should run");
    assert!(report.compatible);
}

#[test]
fn requires_connectivity_validation() {
    let source = r#"
requires_connectivity {
  gps: required;
  wifi: optional;
  cellular: required;
  latency <= 100ms;
  bandwidth >= 5 Mbps;
  packet_loss <= 1%;
}

hardware RoverV2 {
  connectivity [ WiFi6, Bluetooth5, LTE, GPS ];
  sensors [ GPS, IMU ];
  actuators [ DifferentialDrive ];
  network { bandwidth: 100 Mbps; latency: 15 ms; }
}

robot Rover {
  sensor gps: GPS;
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}

deploy Rover to RoverV2;
"#;
    check(source).expect("requires_connectivity should parse");
    let report = verify_compatibility_target(source, None).expect("verify should run");
    assert!(report.compatible);
}

#[test]
fn requires_connectivity_missing_cellular_fails() {
    let source = r#"
requires_connectivity {
  cellular: required;
}

hardware Tiny {
  connectivity [ WiFi ];
  sensors [ IMU ];
  actuators [ DifferentialDrive ];
}

robot Rover {
  sensor imu: IMU;
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}

deploy Rover to Tiny;
"#;
    check(source).expect("should type-check");
    let report = verify_compatibility_target(source, None).expect("verify should run");
    assert!(!report.compatible);
    assert!(report
        .items
        .iter()
        .any(|i| { i.severity == CompatSeverity::Error && i.message.contains("cellular") }));
}

#[test]
fn geofence_parsing_and_validation() {
    let source = r#"
geofence SafeZone {
  center: geo(30.2672, -97.7431);
  radius: 100.0;
}

robot Rover {
  sensor gps: GPS on "/gps";
  actuator wheels: DifferentialDrive;

  on geofence SafeZone exited {
    stop_all_actuators();
  }

  behavior run() { wheels.stop(); }
}
"#;
    check(source).expect("geofence should parse and type-check");
}

#[test]
fn connectivity_triggers_parse() {
    let source = r#"
robot Rover {
  sensor gps: GPS on "/gps";
  actuator wheels: DifferentialDrive;

  mode degraded { wheels.stop(); }

  on gps.lost { enter degraded; }
  on gps.acquired { wheels.stop(); }
  on network.disconnected { wheels.stop(); }
  on cellular.roaming { wheels.stop(); }
  on bluetooth.device_connected { wheels.stop(); }
  on gps.fix { wheels.stop(); }

  behavior run() { wheels.stop(); }
}
"#;
    check(source).expect("connectivity triggers should parse");
}

#[test]
fn connectivity_policy_parsing() {
    let source = r#"
connectivity_policy RoverNetwork {
  preferred: wifi;
  fallback: cellular;
  emergency: bluetooth;
  switch_if latency > 200ms;
  switch_if packet_loss > 5%;
}

robot Rover {
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}
"#;
    check(source).expect("connectivity policy should parse");
}

#[test]
fn bluetooth_and_ble_service_syntax() {
    let source = r#"
ble_service HeartRateSensor {
  uuid: "180D";
}

robot Rover {
  bluetooth {
    scan for devices where name matches /^sensor-/;
    pair trusted_only;
  }
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}
"#;
    check(source).expect("bluetooth and ble_service should parse");
}

#[test]
fn connectivity_modes_parse() {
    let source = r#"
robot Rover {
  actuator wheels: DifferentialDrive;

  mode offline {
    wheels.stop();
  }

  mode weak_signal {
    wheels.stop();
  }

  on network.disconnected { enter offline_mode; }

  behavior run() { wheels.stop(); }
}
"#;
    check(source).expect("connectivity-aware modes should parse");
}

#[test]
fn simulator_connectivity_faults() {
    let source = r#"
hardware RoverV2 {
  connectivity [ WiFi6, LTE, GPS ];
  sensors [ GPS, IMU ];
  actuators [ DifferentialDrive ];
}

robot Rover {
  sensor gps: GPS;
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}

deploy Rover to RoverV2;

simulate_compatibility {
  fault GPSLost;
  fault NetworkLatencySpike;
}
"#;
    check(source).expect("simulator faults should parse");
    let report = verify_compatibility(
        source,
        &VerifyOptions {
            simulate: true,
            ..Default::default()
        },
    )
    .expect("verify with simulate");
    assert!(!report.compatible);
}

#[test]
fn geofence_runtime_in_geofence() {
    let source = r#"
geofence Home {
  center: geo(30.0, -97.0);
  radius: 500.0;
}

robot Rover {
  sensor gps: GPS on "/fix";
  actuator wheels: DifferentialDrive;
  behavior run() {
    let inside = robot.in_geofence("Home");
    let _ = inside;
    wheels.stop();
  }
}
"#;
    check(source).expect("in_geofence should type-check");
}

#[test]
fn robot_connectivity_link_method() {
    let source = r#"
connectivity_policy Net {
  preferred: wifi;
  fallback: cellular;
}

robot Rover {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let link = robot.connectivity_link();
    let _ = link;
    wheels.stop();
  }
}
"#;
    check(source).expect("connectivity_link should type-check");
}

#[test]
fn security_connectivity_capabilities() {
    let source = r#"
robot Rover {
  permissions [
    gps.read,
    network.status,
    network.failover
  ];
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}
"#;
    check(source).expect("connectivity capabilities should parse");
}

#[test]
fn robot_sim_identity_method() {
    let source = r#"
robot Rover {
  permissions [ cellular.connect ];
  actuator wheels: DifferentialDrive;
  behavior run() {
    let sim = robot.sim_identity();
    let _ = sim;
    wheels.stop();
  }
}
"#;
    check(source).expect("sim_identity should type-check");
}
