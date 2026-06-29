//! Interpreter fleet recovery coordination integration tests.

use spanda_interpreter::{run_program, RunOptions};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_runtime::assurance_runtime::platform_assurance_runtime;

fn assurance_run_options() -> RunOptions {
    RunOptions {
        assurance_runtime: Some(platform_assurance_runtime()),
        ..Default::default()
    }
}

#[test]
fn fleet_recovery_publishes_mesh_command() {
    let source = r#"
hardware H {
    sensors [GPS];
    actuators [DifferentialDrive];
}

fleet PatrolFleet {
    RoverAlpha;
    RoverBeta;
}

on anomaly FleetFault severity High {
    reassign mission;
}

anomaly_detector FleetFault {
    expected gps.accuracy <= 3 m;
}

robot RoverAlpha {
    sensor gps: GPS;
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }
    behavior patrol() {
        loop every 50ms {
            let _ = gps.read();
        }
    }
}

robot RoverBeta {
    sensor gps: GPS;
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }
    behavior patrol() { }
}
"#;
    let program = parse(tokenize(source).unwrap()).unwrap();
    let result = run_program(
        &program,
        RunOptions {
            inject_health_faults: true,
            max_loop_iterations: 3,
            ..assurance_run_options()
        },
    )
    .expect("run");
    assert!(
        result.logs.iter().any(|l| l.contains("fleet_recovery:")),
        "expected fleet recovery coordination log, got: {:?}",
        result.logs
    );
}

#[test]
fn fleet_recovery_relays_to_mesh_coordinator() {
    use spanda_fleet::{
        register_fleet_agent, spawn_test_fleet_agent, spawn_test_fleet_mesh, FleetAgentRegistry,
    };
    use std::thread;
    use std::time::Duration;

    let (port_a, _a) = spawn_test_fleet_agent("RoverAlpha", None).expect("spawn A");
    let (port_b, _b) = spawn_test_fleet_agent("RoverBeta", None).expect("spawn B");
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
    std::env::set_var(
        "SPANDA_FLEET_MESH_URL",
        format!("http://127.0.0.1:{mesh_port}"),
    );

    let source = r#"
hardware H { sensors [GPS]; actuators [DifferentialDrive]; }
fleet PatrolFleet { RoverAlpha; RoverBeta; }
on anomaly FleetFault severity High { reassign mission; }
anomaly_detector FleetFault { expected gps.accuracy <= 3 m; }
robot RoverAlpha {
    sensor gps: GPS;
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }
    behavior patrol() { loop every 50ms { let _ = gps.read(); } }
}
robot RoverBeta {
    sensor gps: GPS;
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }
    behavior patrol() { }
}
"#;
    let program = parse(tokenize(source).unwrap()).unwrap();
    let result = run_program(
        &program,
        RunOptions {
            inject_health_faults: true,
            max_loop_iterations: 3,
            ..assurance_run_options()
        },
    )
    .expect("run");
    std::env::remove_var("SPANDA_FLEET_MESH_URL");
    assert!(
        result
            .logs
            .iter()
            .any(|l| l.contains("fleet_mesh: recovery")),
        "expected mesh recovery relay log, got: {:?}",
        result.logs
    );
    assert!(
        result
            .logs
            .iter()
            .any(|l| l.contains("fleet_mesh: takeover")),
        "expected mesh continuity relay on reassign, got: {:?}",
        result.logs
    );
}
