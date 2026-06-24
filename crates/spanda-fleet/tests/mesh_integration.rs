//! Fleet mesh coordinator integration tests.

use spanda_deploy_http::http_request;
use spanda_driver::compile;
use spanda_fleet::{
    default_fleet_agents_path, fleet_entry_for_port, orchestrate_fleets_mesh, register_fleet_agent,
    relay_deliveries_via_mesh, relay_peer_delivery, relay_recovery_via_mesh,
    save_fleet_agent_registry, spawn_test_fleet_agent, spawn_test_fleet_mesh, FleetAgentRegistry,
    FleetRecoveryRequest, PeerDelivery,
};
use std::thread;
use std::time::Duration;

#[test]
fn mesh_coordinator_relays_to_registered_agents() {
    let (port, _agent) = spawn_test_fleet_agent("ScoutB", None).expect("spawn agent");
    let mut registry = FleetAgentRegistry::default();
    register_fleet_agent(
        &mut registry,
        "ScoutB".into(),
        format!("http://127.0.0.1:{port}"),
        None,
    )
    .expect("register");
    let (mesh_port, _mesh) = spawn_test_fleet_mesh(&registry).expect("spawn mesh");
    thread::sleep(Duration::from_millis(30));
    let resp = relay_deliveries_via_mesh(
        &format!("http://127.0.0.1:{mesh_port}"),
        &[PeerDelivery {
            from_robot: "ScoutA".into(),
            to_robot: "ScoutB".into(),
            topic: "mission_step".into(),
            step: "inspect".into(),
            delivered: true,
        }],
        None,
    )
    .expect("mesh relay");
    assert!(resp.ok);
    assert_eq!(resp.relayed, 1);
}

#[test]
fn mesh_coordinator_relays_fleet_recovery_to_agents() {
    let (port_a, _agent_a) = spawn_test_fleet_agent("RoverAlpha", None).expect("spawn A");
    let (port_b, _agent_b) = spawn_test_fleet_agent("RoverBeta", None).expect("spawn B");
    let mut registry = FleetAgentRegistry::default();
    register_fleet_agent(
        &mut registry,
        "RoverAlpha".into(),
        format!("http://127.0.0.1:{port_a}"),
        None,
    )
    .expect("register A");
    register_fleet_agent(
        &mut registry,
        "RoverBeta".into(),
        format!("http://127.0.0.1:{port_b}"),
        None,
    )
    .expect("register B");
    let (mesh_port, _mesh) = spawn_test_fleet_mesh(&registry).expect("spawn mesh");
    thread::sleep(Duration::from_millis(30));
    let resp = relay_recovery_via_mesh(
        &format!("http://127.0.0.1:{mesh_port}"),
        &FleetRecoveryRequest {
            action: "pause mission".into(),
            fleet_name: Some("PatrolFleet".into()),
            from_robot: Some("RoverAlpha".into()),
            members: vec!["RoverAlpha".into(), "RoverBeta".into()],
        },
        None,
    )
    .expect("mesh recovery");
    assert!(resp.ok);
    assert_eq!(resp.relayed, 2);
    let fleet_program = r#"
recovery_policy FleetRecovery {
    on fleet.failed { enter degraded_mode; reduce_speed 0.5 m/s; }
}
robot RoverBeta {
    sensor gps: GPS;
    actuator wheels: DifferentialDrive;
    mode degraded { }
    safety { max_speed = 1.0 m/s; }
    behavior patrol() {}
}
"#;
    let program_payload = serde_json::json!({ "program": fleet_program }).to_string();
    let deploy = http_request(
        "POST",
        &format!("http://127.0.0.1:{port_b}/v1/program"),
        Some(&program_payload),
        None,
    )
    .expect("deploy program");
    assert_eq!(deploy.status, 200);
    std::env::set_var("SPANDA_OPERATOR_APPROVAL", "1");
    let resp2 = relay_recovery_via_mesh(
        &format!("http://127.0.0.1:{mesh_port}"),
        &FleetRecoveryRequest {
            action: "enter degraded_mode".into(),
            fleet_name: Some("PatrolFleet".into()),
            from_robot: Some("RoverAlpha".into()),
            members: vec!["RoverBeta".into()],
        },
        None,
    )
    .expect("mesh recovery after deploy");
    std::env::remove_var("SPANDA_OPERATOR_APPROVAL");
    assert!(resp2.ok);
    let status = http_request(
        "GET",
        &format!("http://127.0.0.1:{port_b}/v1/status"),
        None,
        None,
    )
    .expect("agent status");
    assert!(status.body.contains("enter degraded_mode"));
    assert!(status.body.contains("last_recovery_commands"));
    assert!(status.body.contains("recovery_active"));
    assert!(status.body.contains("recovery_validation"));
    assert!(status.body.contains("\"recovery_engine\":\"interpreter\""));
    assert!(
        status.body.contains("\"PASS\"")
            || status.body.contains("\"PARTIAL\"")
    );
    assert!(status.body.contains("\"recovery_mode\":\"degraded\""));
}

#[test]
fn orchestrate_mesh_mode_reports_distributed_peer_mesh() {
    let (port, _agent) = spawn_test_fleet_agent("ScoutB", None).expect("spawn agent");
    let mut registry = FleetAgentRegistry::default();
    register_fleet_agent(
        &mut registry,
        "ScoutB".into(),
        format!("http://127.0.0.1:{port}"),
        None,
    )
    .expect("register");
    let (mesh_port, _mesh) = spawn_test_fleet_mesh(&registry).expect("spawn mesh");
    let source = r#"
robot ScoutA {
  robot ScoutB;
  mission Patrol { navigate; inspect; }
}
robot ScoutB {
  mission Patrol { navigate; inspect; }
}
fleet Recon { ScoutA; ScoutB; }
"#;
    let program = compile(source).expect("compile").program;
    thread::sleep(Duration::from_millis(30));
    let result = orchestrate_fleets_mesh(
        &program,
        "peer_fleet.sd",
        &format!("http://127.0.0.1:{mesh_port}"),
        None,
    );
    assert!(result.success);
    assert_eq!(result.fleets[0].coordination_mode, "distributed_peer_mesh");
}

#[test]
fn fleet_agent_forwards_to_downstream_peer() {
    // Isolate registry lookup from any SPANDA_FLEET_AGENTS left in the shell environment.
    unsafe {
        std::env::remove_var("SPANDA_FLEET_AGENTS");
    }
    let (port_b, _agent_b) = spawn_test_fleet_agent("ScoutB", None).expect("spawn B");
    let (port_a, _agent_a) = spawn_test_fleet_agent("ScoutA", None).expect("spawn A");
    let path = default_fleet_agents_path();
    let mut registry = FleetAgentRegistry::default();
    register_fleet_agent(
        &mut registry,
        "ScoutB".into(),
        fleet_entry_for_port("ScoutB", port_b, None).url,
        None,
    )
    .expect("register B");
    save_fleet_agent_registry(&path, &registry).expect("save registry");
    thread::sleep(Duration::from_millis(30));
    let delivery = PeerDelivery {
        from_robot: "ScoutA".into(),
        to_robot: "ScoutB".into(),
        topic: "mission_step".into(),
        step: "inspect".into(),
        delivered: false,
    };
    let entry = fleet_entry_for_port("ScoutA", port_a, None);
    let resp = relay_peer_delivery(&entry, &delivery).expect("forward via ScoutA agent");
    assert!(resp.ok);
    let _ = std::fs::remove_file(path);
}
