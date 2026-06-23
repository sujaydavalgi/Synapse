//! Integration tests for operational readiness engine.

use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_readiness::{
    analyze_failure, audit_program, evaluate_fleet_readiness, evaluate_readiness,
    readiness_traceability, verify_approvals, verify_fleet, verify_mission, ReadinessOptions,
};

fn parse_source(source: &str) -> spanda_ast::nodes::Program {
    parse(tokenize(source).expect("tokenize")).expect("parse")
}

const ROVER: &str = include_str!("../../../examples/showcase/capability_verification/rover.sd");

const WAREHOUSE_MISSION: &str = r#"
hardware RoverV1 {
  sensors [ GPS, Lidar ];
  actuators [ DifferentialDrive ];
}

robot Rover {
  uses hardware RoverV1;
  sensor gps: GPS;
  sensor lidar: Lidar;
  actuator wheels: DifferentialDrive;
  exposes capabilities [ gps_navigation, obstacle_avoidance ];
  mission WarehousePatrol {
    requires capabilities [ obstacle_avoidance, gps_navigation ];
    patrol;
  }
  behavior patrol() {
    loop every 100ms { wheels.drive(linear: 0.1 m/s, angular: 0.0 rad/s); }
  }
}
"#;

const APPROVAL_MISSION: &str = r#"
robot GateBot {
  actuator gate: DifferentialDrive;
  topic gate_approval: Approval publish on "/gate/approval";
  mode hold { gate.stop(); }
  mission OpenGate {
    requires approval Operator for: open_gate;
    open_sequence;
  }
  behavior open_sequence() { gate.drive(linear: 0.0 m/s, angular: 0.0 rad/s); }
}
"#;

const FLEET: &str = include_str!("../../../examples/showcase/fleet_readiness/warehouse.sd");

#[test]
fn readiness_engine_produces_score() {
    let program = parse_source(ROVER);
    let report = evaluate_readiness(&program, &ReadinessOptions::default());
    assert!(report.score.total > 0);
    assert!(!report.robots.is_empty());
}

#[test]
fn mission_verification_achievable() {
    let program = parse_source(WAREHOUSE_MISSION);
    let reports = verify_mission(&program, None);
    assert!(!reports.is_empty());
    let report = reports.first().unwrap();
    assert!(report.capabilities_satisfied);
    assert_eq!(report.required_capabilities.len(), 2);
}

#[test]
fn failure_analysis_lists_impacts() {
    let program = parse_source(ROVER);
    let report = analyze_failure(&program);
    assert!(!report.impacts.is_empty());
    assert!(report.impacts.iter().any(|i| i.component == "GPS"));
}

#[test]
fn fleet_readiness_aggregates() {
    let program = parse_source(FLEET);
    let report = evaluate_fleet_readiness(&program, &ReadinessOptions::default());
    assert_eq!(
        report.healthy_robots + report.degraded_robots + report.not_ready_robots,
        2
    );
}

#[test]
fn safety_auditor_flags_missing_kill_switch() {
    let program = parse_source(ROVER);
    let report = audit_program(&program, ROVER);
    assert!(report.critical_count >= 1);
}

#[test]
fn approval_verification_passes_with_topic() {
    let program = parse_source(APPROVAL_MISSION);
    let report = verify_approvals(&program);
    assert!(report.compatible);
}

#[test]
fn fleet_verify_detects_multi_robot() {
    let program = parse_source(FLEET);
    let report = verify_fleet(&program);
    assert!(!report.findings.is_empty());
}

#[test]
fn readiness_traceability_has_rows() {
    let program = parse_source(ROVER);
    let rows = readiness_traceability(&program);
    assert!(!rows.is_empty());
}

#[test]
fn root_cause_diagnose_trace() {
    let trace = include_str!("../../../examples/showcase/root_cause_analysis/mission.trace");
    let path = std::env::temp_dir().join("spanda_test_mission.trace");
    std::fs::write(&path, trace).unwrap();
    let report = spanda_readiness::diagnose_trace(&path).expect("diagnose");
    assert!(!report.root_cause.is_empty());
    assert!(!report.timeline.is_empty());
}
