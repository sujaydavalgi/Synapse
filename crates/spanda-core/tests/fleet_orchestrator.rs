//! Fleet orchestrator integration tests.

use spanda_core::{check, compile, orchestrate_fleets, parse_http_url};

#[test]
fn orchestrates_robotics_fleet_example() {
    let source = include_str!("../../../examples/robotics/fleet_management.sd");
    check(source).expect("fleet example should type-check");
    let program = compile(source).expect("compile").program;
    let result = orchestrate_fleets(&program, "fleet_management.sd");
    assert!(result.success);
    assert_eq!(result.fleets.len(), 1);
    assert_eq!(result.fleets[0].fleet_name, "Warehouse");
    assert_eq!(result.fleets[0].members.len(), 2);
}

#[test]
fn peer_fleet_emits_handoff_messages() {
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
    check(source).expect("peer fleet should type-check");
    let program = compile(source).expect("compile").program;
    let result = orchestrate_fleets(&program, "peer_fleet.sd");
    assert!(result.success);
    let fleet = &result.fleets[0];
    assert_eq!(fleet.coordination_mode, "peer_mesh_mission");
    assert!(!fleet.peer_messages.is_empty());
    assert!(!fleet.peer_deliveries.is_empty());
    assert!(fleet.peer_deliveries.iter().all(|d| d.delivered));
}

#[test]
fn deploy_agent_urls_accept_https() {
    let parsed = parse_http_url("https://agent.local:9443/v1").expect("parse https url");
    assert!(parsed.use_tls);
    assert_eq!(parsed.port, 9443);
}
