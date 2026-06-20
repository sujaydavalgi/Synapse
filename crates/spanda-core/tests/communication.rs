use spanda_core::{check, compile, run, verify_compatibility_target, CompatSeverity, RunOptions};

#[test]
fn message_parsing_and_registry() {
    let source = r#"
message LidarReading {
  scan: Scan;
  timestamp: String;
  version: 1;
}

robot CommBot {
  topic lidar_scan: LidarReading publish on "/scan";
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  behavior noop() { }
}
"#;
    compile(source).expect("message parsing");
    check(source).expect("message type check");
}

#[test]
fn topic_qos_and_transport() {
    let source = r#"
robot QosBot {
  bus sim;
  topic stream: Scan {
    qos reliable;
    rate 20Hz;
    deadline 50ms;
  } on sim;
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  behavior run() {
    publish stream with lidar.read();
  }
}
"#;
    check(source).expect("topic qos");
}

#[test]
fn service_and_action_typed() {
    let source = r#"
message BatteryStatus { level: String; }

robot ServiceBot {
  service GetBattery {
    request String;
    response BatteryStatus;
  };
  action NavigateTo {
    request Pose;
    feedback String;
    result String;
  };
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  behavior run() {
    let status = call GetBattery();
    let nav = execute NavigateTo(pose(x: 1.0 m, y: 0.0 m, theta: 0.0 rad));
  }
}
"#;
    check(source).expect("typed service and action");
}

#[test]
fn subscribe_and_publish_parens() {
    let source = r#"
robot PubSubBot {
  topic scan_out: Scan publish on "/out";
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  behavior run() {
    subscribe scan_out;
    publish scan_out(lidar.read());
  }
}
"#;
    compile(source).expect("pub/sub compile");
    run(
        source,
        RunOptions {
            max_loop_iterations: 2,
            ..Default::default()
        },
    )
    .expect("pub/sub run");
}

#[test]
fn agent_communication_capabilities() {
    let source = r#"
robot AgentBot {
  topic data: Scan publish on "/data";
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;
  ai_model m: LLM { provider: "mock"; model: "x"; temperature: 0.1; }
  agent Worker {
    uses m;
    can [ subscribe(data), publish(data) ];
    goal "work";
    plan { }
  }
  safety { max_speed = 1.0 m/s; }
  behavior run() { }
}
"#;
    check(source).expect("agent comm capabilities");
}

#[test]
fn robot_peer_and_device_declarations() {
    let source = r#"
robot FleetBot {
  bus local;
  robot RoverA;
  device Lidar: Lidar;
  topic pose: Pose publish on "/pose";
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  behavior run() {
    subscribe RoverA.pose;
    discover robots;
  }
}
"#;
    check(source).expect("peer robot and device");
}

#[test]
fn discover_with_capability_filter() {
    let source = r#"
robot DiscoverBot {
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  behavior run() {
    discover agents where capability includes Planner;
  }
}
"#;
    check(source).expect("discover filter");
}

#[test]
fn event_with_payload_fields() {
    let source = r#"
message Alert { text: String; }

robot EventBot {
  event ObstacleDetected {
    alert: Alert;
  };
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  on ObstacleDetected { wheels.stop(); }
  behavior run() { emit ObstacleDetected; }
}
"#;
    check(source).expect("event fields");
}

#[test]
fn topic_qos_bandwidth_verify() {
    let source = r#"
robot BandwidthBot {
  topic stream: Scan publish on "/stream" {
    qos reliable;
    rate 20Hz;
  };
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  behavior run() { }
}

deploy BandwidthBot to ESP32;
"#;
    check(source).expect("topic qos bandwidth type-check");
    let report = verify_compatibility_target(source, None).expect("verify");
    assert!(
        report.items.iter().any(|i| {
            i.category == "network"
                && i.message.contains("Mbps")
                && i.severity == CompatSeverity::Error
        }),
        "expected topic bandwidth error on ESP32, got: {:?}",
        report.items
    );
}

#[test]
fn transport_ros2_routing_at_runtime() {
    let source = r#"
robot RosBot {
  bus ros2;
  topic cmd: Velocity publish on "/cmd_vel";
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  behavior run() {
    publish cmd with velocity(linear: 0.5 m/s, angular: 0.0 rad/s);
  }
}
"#;
    run(
        source,
        RunOptions {
            max_loop_iterations: 1,
            ..Default::default()
        },
    )
    .expect("ros2 transport runtime");
}

#[test]
fn std_network_import_type_check() {
    let source = r#"
import std.network;

message NetMsg {
  profile: QosProfile;
  endpoint: ServiceEndpoint;
}

robot NetBot {
  topic net: NetMsg publish on "/net";
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  behavior run() { }
}
"#;
    check(source).expect("std.network import");
}

#[test]
fn requires_network_integration() {
    let source = r#"
requires_network {
  bandwidth >= 5 Mbps;
  latency <= 50 ms;
}

robot NetBot {
  topic stream: Scan publish on "/stream" {
    qos reliable;
    rate 20Hz;
  };
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  behavior run() { }
}
"#;
    check(source).expect("network requirements");
}

#[test]
fn simulator_comm_bus_round_trip() {
    let source = r#"
robot SimBot {
  bus sim;
  topic echo: Scan publish on "/echo";
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  behavior run() {
    subscribe echo;
    publish echo(lidar.read());
    receive echo to reading;
  }
}
"#;
    run(
        source,
        RunOptions {
            max_loop_iterations: 1,
            ..Default::default()
        },
    )
    .expect("simulator comm");
}

#[test]
fn rejects_unknown_message_type_on_topic() {
    let source = r#"
robot BadBot {
  topic x: UnknownMsg publish on "/x";
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  behavior run() { }
}
"#;
    assert!(check(source).is_err());
}

#[test]
fn rejects_invalid_capability_target() {
    let source = r#"
robot BadBot {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;
  ai_model m: LLM { provider: "mock"; model: "x"; temperature: 0.1; }
  agent Worker {
    uses m;
    can [ publish(missing_topic) ];
    goal "x";
    plan { }
  }
  safety { max_speed = 1.0 m/s; }
  behavior run() { }
}
"#;
    assert!(check(source).is_err());
}
