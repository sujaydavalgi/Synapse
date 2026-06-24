//! Integration tests for mission assurance analysis.

use spanda_assurance::{
    assure_program, build_assurance_report, check_resilience, evaluate_prognostics,
    extract_knowledge_base, scan_anomalies, verify_mission_assurance,
};
use spanda_lexer::tokenize;
use spanda_parser::parse;

fn parse_source(source: &str) -> spanda_ast::nodes::Program {
    parse(tokenize(source).unwrap()).unwrap()
}

const ROVER: &str = r#"
hardware RoverV1 {
    sensors [GPS, Camera, Lidar];
    actuators [DifferentialDrive];
    connectivity [WiFi];
}

knowledge_model RoverModel {
    component gps;
    component camera;
    dependency navigation requires [gps, camera];
}

anomaly_detector NavAnomaly {
    expected gps.accuracy <= 3 m;
}

on anomaly NavAnomaly severity High {
    enter degraded_mode;
}

prognostics BatteryPrognostics {
    predict battery.remaining_useful_life;
}

mitigation GPSLost {
    if gps.failed {
        enter degraded_mode;
    }
}

assurance_case RoverCase {
    evidence hardware_verification;
    evidence health_checks;
}

health_check RoverHealth for robot Rover {
    check battery.level >= 20%;
}

robot Rover {
    sensor gps: GPS;
    sensor camera: Camera;
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }
    behavior patrol() { wheels.drive(0.3 m/s); }
}
"#;

#[test]
fn parses_and_analyzes_knowledge_model() {
    let program = parse_source(ROVER);
    let kb = extract_knowledge_base(&program);
    assert_eq!(kb.models.len(), 1);
    assert!(!kb.models[0].components.is_empty());
}

#[test]
fn anomaly_scan_finds_detectors() {
    let program = parse_source(ROVER);
    let report = scan_anomalies(&program);
    assert_eq!(report.detectors.len(), 1);
    assert!(!report.handlers.is_empty());
}

#[test]
fn assurance_report_links_evidence() {
    let program = parse_source(ROVER);
    let report = build_assurance_report(&program, "test.sd");
    assert_eq!(report.cases.len(), 1);
    assert!(!report.cases[0].evidence.is_empty());
}

#[test]
fn prognostics_evaluates_rules() {
    let program = parse_source(ROVER);
    let report = evaluate_prognostics(&program);
    assert_eq!(report.models.len(), 1);
}

#[test]
fn resilience_check_runs() {
    let program = parse_source(ROVER);
    let report = check_resilience(&program);
    assert!(report.readiness_score > 0 || !report.recovery.is_empty());
}

#[test]
fn assure_program_composes() {
    let program = parse_source(ROVER);
    let summary = assure_program(&program, "test.sd");
    assert!(!summary.assurance.cases.is_empty());
}

#[test]
fn mission_assurance_parses_plans() {
    let source = r#"
hardware H { sensors [GPS]; actuators [DifferentialDrive]; }
mission_plan P { step a; constraint battery.level >= 10%; }
robot R { sensor gps: GPS; actuator w: DifferentialDrive; safety { max_speed = 1 m/s; } behavior b() {} }
"#;
    let program = parse_source(source);
    let report = verify_mission_assurance(&program);
    assert_eq!(report.plans.len(), 1);
}
