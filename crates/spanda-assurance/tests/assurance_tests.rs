//! Integration tests for mission assurance analysis.

use spanda_assurance::{
    assure_program, build_assurance_report, build_assurance_report_with_config, check_resilience,
    evaluate_prognostics, extract_knowledge_base, scan_anomalies, verify_mission_assurance,
};
use spanda_lexer::tokenize;
use spanda_parser::parse;

fn parse_source(source: &str) -> spanda_ast::nodes::Program {
    // Description:
    //     Parse source.
    //
    // Inputs:
    //     source: &str
    //         Caller-supplied source.
    //
    // Outputs:
    //     result: spanda_ast::nodes::Program
    //         Return value from `parse_source`.
    //
    // Example:

    //     let result = spanda_assurance::assurance_tests::parse_source(source);

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
    // Description:
    //     Parses and analyzes knowledge model.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_assurance::assurance_tests::parses_and_analyzes_knowledge_model();

    let program = parse_source(ROVER);
    let kb = extract_knowledge_base(&program);
    assert_eq!(kb.models.len(), 1);
    assert!(!kb.models[0].components.is_empty());
}

#[test]
fn anomaly_scan_finds_detectors() {
    // Description:
    //     Anomaly scan finds detectors.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_assurance::assurance_tests::anomaly_scan_finds_detectors();

    let program = parse_source(ROVER);
    let report = scan_anomalies(&program);
    assert_eq!(report.detectors.len(), 1);
    assert!(!report.handlers.is_empty());
}

#[test]
fn assurance_report_links_evidence() {
    // Description:
    //     Assurance report links evidence.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_assurance::assurance_tests::assurance_report_links_evidence();

    let program = parse_source(ROVER);
    let report = build_assurance_report(&program, "test.sd");
    assert_eq!(report.cases.len(), 1);
    assert!(!report.cases[0].evidence.is_empty());
}

#[test]
fn prognostics_evaluates_rules() {
    // Description:
    //     Prognostics evaluates rules.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_assurance::assurance_tests::prognostics_evaluates_rules();

    let program = parse_source(ROVER);
    let report = evaluate_prognostics(&program);
    assert_eq!(report.models.len(), 1);
}

#[test]
fn resilience_check_runs() {
    // Description:
    //     Resilience check runs.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_assurance::assurance_tests::resilience_check_runs();

    let program = parse_source(ROVER);
    let report = check_resilience(&program);
    assert!(report.readiness_score > 0 || !report.recovery.is_empty());
}

#[test]
fn showcase_assurance_passes() {
    // Description:
    //     Showcase assurance passes.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_assurance::assurance_tests::showcase_assurance_passes();

    let source = include_str!("../../../examples/showcase/assurance/rover.sd");
    let program = parse_source(source);
    let summary = assure_program(&program, "rover.sd");
    assert!(
        summary.passed,
        "expected showcase assurance to pass: {:?}",
        summary.issues
    );
}

#[test]
fn assure_program_composes() {
    // Description:
    //     Assure program composes.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_assurance::assurance_tests::assure_program_composes();

    let program = parse_source(ROVER);
    let summary = assure_program(&program, "test.sd");
    assert!(!summary.assurance.cases.is_empty());
}

#[test]
fn mission_assurance_parses_plans() {
    // Description:
    //     Mission assurance parses plans.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_assurance::assurance_tests::mission_assurance_parses_plans();

    let source = r#"
hardware H { sensors [GPS]; actuators [DifferentialDrive]; }
mission_plan P { step a; constraint battery.level >= 10%; }
robot R { sensor gps: GPS; actuator w: DifferentialDrive; safety { max_speed = 1 m/s; } behavior b() {} }
"#;
    let program = parse_source(source);
    let report = verify_mission_assurance(&program);
    assert_eq!(report.plans.len(), 1);
}

#[test]
fn learned_models_detect_package_import() {
    // Description:
    //     Learned models detect package import.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_assurance::assurance_tests::learned_models_detect_package_import();

    use spanda_assurance::learned_models;
    let source = r#"
import assurance.anomaly;

anomaly_detector NavML {
    expected gps.accuracy <= 3 m;
}

robot R {
    sensor gps: GPS;
    actuator w: DifferentialDrive;
    safety { max_speed = 1 m/s; }
    behavior b() {}
}
"#;
    let program = parse_source(source);
    let models = learned_models(&program);
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].detector, "NavML");
    assert!(models[0].backend.contains("anomaly"));
}

#[test]
fn learned_models_detect_explicit_backend() {
    // Description:
    //     Learned models detect explicit backend.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_assurance::assurance_tests::learned_models_detect_explicit_backend();

    use spanda_assurance::learned_models;
    let source = r#"
anomaly_detector NavML {
    learned backend assurance.anomaly;
    expected gps.accuracy <= 3 m;
}

robot R {
    sensor gps: GPS;
    actuator w: DifferentialDrive;
    safety { max_speed = 1 m/s; }
    behavior b() {}
}
"#;
    let program = parse_source(source);
    let models = learned_models(&program);
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].backend, "assurance.anomaly");
}

#[test]
fn anomaly_scan_includes_learned_models() {
    // Description:
    //     Anomaly scan includes learned models.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_assurance::assurance_tests::anomaly_scan_includes_learned_models();

    use spanda_assurance::scan_anomalies;
    let source = r#"
anomaly_detector NavML {
    learned backend assurance.anomaly;
    expected gps.accuracy <= 3 m;
}

on anomaly NavML severity High {
    enter safe_mode;
}

robot R {
    sensor gps: GPS;
    actuator w: DifferentialDrive;
    safety { max_speed = 1 m/s; }
    behavior b() {}
}
"#;
    let program = parse_source(source);
    let report = scan_anomalies(&program);
    assert_eq!(report.learned.len(), 1);
}

#[test]
fn state_assurance_evaluates_estimators() {
    // Description:
    //     State assurance evaluates estimators.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_assurance::assurance_tests::state_assurance_evaluates_estimators();

    use spanda_assurance::evaluate_state_assurance;
    let source = r#"
state_estimator RoverState {
    inputs [gps.fix, lidar.read];
    output StateEstimate;
}

robot R {
    sensor gps: GPS;
    sensor lidar: Lidar;
    actuator w: DifferentialDrive;
    safety { max_speed = 1 m/s; }
    behavior b() {}
}
"#;
    let program = parse_source(source);
    let report = evaluate_state_assurance(&program);
    assert_eq!(report.estimators.len(), 1);
    assert!(report.passed);
    assert_eq!(report.belief.estimates.len(), 1);
    assert!(report.estimators[0]
        .fused
        .as_ref()
        .is_some_and(|f| f.value.contains("weighted")));
}

#[test]
fn assurance_report_includes_device_traceability_from_config() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../spanda-config/tests/fixtures/warehouse");
    let cfg = spanda_config::ConfigResolver::new()
        .resolve_from_dir(&root)
        .expect("resolve");
    let program = parse_source(ROVER);
    let report = build_assurance_report_with_config(&program, "rover.sd", Some(&cfg));
    assert!(report.traceability.rows.iter().any(|row| row.contains("device:camera-front-001")));
}
