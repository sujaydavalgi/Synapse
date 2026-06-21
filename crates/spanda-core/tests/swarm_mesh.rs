//! Swarm coordinator mesh relay integration tests.

use spanda_core::{
    compile, coordinate_swarms_mesh, register_fleet_agent, spawn_test_fleet_agent,
    spawn_test_fleet_mesh, SwarmState, FleetAgentRegistry,
};
use std::thread;
use std::time::Duration;

#[test]
fn swarm_leader_follow_relays_via_mesh() {
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
  mission Patrol { navigate; inspect; }
}
robot ScoutB {
  mission Patrol { navigate; inspect; }
}
fleet Recon { ScoutA; ScoutB; }
swarm ReconLeader {
  fleet Recon;
  policy leader_follow;
}
"#;
    let program = compile(source).expect("compile").program;
    thread::sleep(Duration::from_millis(30));
    let mut state = SwarmState::default();
    let result = coordinate_swarms_mesh(
        &program,
        "swarm_leader.sd",
        &mut state,
        &format!("http://127.0.0.1:{mesh_port}"),
        None,
    );
    assert!(result.success);
    let leader = result
        .swarms
        .iter()
        .find(|swarm| swarm.policy == "leader_follow")
        .expect("leader_follow swarm");
    assert_eq!(leader.remote_relayed, 1);
    assert!(leader.coordination_mode.ends_with("_mesh"));
}
