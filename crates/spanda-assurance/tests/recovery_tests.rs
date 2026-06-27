//! Integration tests for self-healing and recovery framework.

use spanda_assurance::{
    evaluate_recovery, extract_recovery_policies, format_recovery, load_merged_recovery_knowledge,
    save_recovery_knowledge_store, simulate_failure_recovery, RecoveryContext,
    RecoveryKnowledgeBase, RecoveryKnowledgeEntry, RecoveryLevel, RecoveryPlanner, RecoveryStatus,
};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_readiness::ReportFormat;

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

    //     let result = spanda_assurance::recovery_tests::parse_source(source);

    parse(tokenize(source).unwrap()).unwrap()
}

const SELF_HEALING: &str = include_str!("../../../examples/showcase/self_healing/rover.sd");

#[test]
fn recovery_policy_parses_from_showcase() {
    // Description:
    //     Recovery policy parses from showcase.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_assurance::recovery_tests::recovery_policy_parses_from_showcase();

    let program = parse_source(SELF_HEALING);
    let policies = extract_recovery_policies(&program);
    assert!(!policies.is_empty());
    assert!(policies[0].triggers.iter().any(|(c, _)| c.contains("gps")));
}

#[test]
fn heal_workflow_produces_passing_report() {
    // Description:
    //     Heal workflow produces passing report.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_assurance::recovery_tests::heal_workflow_produces_passing_report();

    let program = parse_source(SELF_HEALING);
    let report = evaluate_recovery(&program, None, None);
    assert!(!report.plans.is_empty());
    assert!(report
        .results
        .iter()
        .all(|r| r.status != RecoveryStatus::Unsafe));
    let text = format_recovery(&report, ReportFormat::Text);
    assert!(text.contains("Safety Validation"));
}

#[test]
fn inject_gps_failure_simulation() {
    // Description:
    //     Inject gps failure simulation.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_assurance::recovery_tests::inject_gps_failure_simulation();

    let program = parse_source(SELF_HEALING);
    let report = simulate_failure_recovery(&program, "gps", None);
    assert_eq!(report.plans[0].diagnosis, "Satellite lock lost");
    assert!(!report.results.is_empty());
}

#[test]
fn inject_lidar_failure_uses_recovery_policy() {
    let program = parse_source(SELF_HEALING);
    let report = simulate_failure_recovery(&program, "lidar", None);
    assert!(report.passed, "{:?}", report.results);
    assert_eq!(report.plans[0].failure, "lidar.failed");
}

#[test]
fn recovery_readiness_evaluated() {
    // Description:
    //     Recovery readiness evaluated.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_assurance::recovery_tests::recovery_readiness_evaluated();

    let program = parse_source(SELF_HEALING);
    let ctx = RecoveryContext {
        issue: "gps.failed".into(),
        diagnosis: None,
        classification: None,
        level: RecoveryLevel::Level3AutomaticWithValidation,
    };
    let report = evaluate_recovery(&program, Some(&ctx), None);
    assert!(report.readiness.readiness_score > 0 || report.readiness.blockers.is_empty());
}

#[test]
fn merged_knowledge_informs_recovery_plan() {
    // Description:
    //     Merged knowledge informs recovery plan.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_assurance::recovery_tests::merged_knowledge_informs_recovery_plan();

    let program = parse_source(
        "robot Rover { sensor gps: GPS; actuator w: DifferentialDrive; safety { max_speed = 1 m/s; } behavior b() {} }",
    );
    let store = std::path::PathBuf::from(".spanda/recovery_knowledge.json");
    if let Some(parent) = store.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    save_recovery_knowledge_store(
        &store,
        &RecoveryKnowledgeBase {
            entries: vec![RecoveryKnowledgeEntry {
                failure_pattern: "gps".into(),
                recovery_pattern: "switch_to visual_odometry".into(),
                success_rate: 0.92,
                recommendation: "Historical VO fallback".into(),
            }],
        },
    )
    .unwrap();
    let kb = load_merged_recovery_knowledge(&program);
    assert!(kb
        .entries
        .iter()
        .any(|e| e.recovery_pattern.contains("visual")));
    let plan = RecoveryPlanner::plan(
        &program,
        &RecoveryContext {
            issue: "gps.failed".into(),
            diagnosis: None,
            classification: None,
            level: RecoveryLevel::Level2AutomaticLowRisk,
        },
    );
    assert!(plan
        .actions
        .iter()
        .any(|a| a.description.contains("visual_odometry")));
    let _ = std::fs::remove_file(&store);
}

#[test]
fn recovery_diagnostics_flag_high_risk_without_approval() {
    // Description:
    //     Recovery diagnostics flag high risk without approval.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_assurance::recovery_tests::recovery_diagnostics_flag_high_risk_without_approval();

    let program = parse_source(
        r#"
recovery_policy Risky {
    on gps.failed { resume mission; }
}
robot R {
    sensor gps: GPS;
    actuator w: DifferentialDrive;
    safety { max_speed = 1 m/s; }
    behavior b() {}
}
"#,
    );
    let diags = spanda_assurance::collect_recovery_diagnostics(&program);
    assert!(diags.iter().any(|d| d.category == "recovery:approval"));
}

#[test]
fn continuity_diagnostics_suggest_insertable_policy_block() {
    let program = parse_source(
        r#"
fleet Patrol { RoverA; RoverB; }
robot RoverA {
    sensor gps: GPS;
    actuator w: DifferentialDrive;
    safety { max_speed = 1 m/s; }
    behavior b() {}
}
robot RoverB {
    sensor gps: GPS;
    actuator w: DifferentialDrive;
    safety { max_speed = 1 m/s; }
    behavior b() {}
}
"#,
    );
    let diags = spanda_assurance::collect_continuity_diagnostics(&program);
    let policy = diags
        .iter()
        .find(|d| d.category == "continuity:policy")
        .expect("continuity:policy diagnostic");
    assert!(
        policy
            .suggested_fix
            .as_ref()
            .is_some_and(|fix| fix.contains("continuity_policy FleetContinuity")),
        "expected insertable continuity_policy snippet, got {:?}",
        policy.suggested_fix
    );
}

#[test]
fn fleet_showcase_recovery_report_passes() {
    // Description:
    //     Fleet showcase recovery report passes.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_assurance::recovery_tests::fleet_showcase_recovery_report_passes();

    let program = parse_source(include_str!(
        "../../../examples/showcase/fleet_recovery/fleet.sd"
    ));
    let report = evaluate_recovery(&program, None, None);
    assert!(
        report.passed,
        "expected fleet showcase heal to pass, plans={} results={:?}",
        report.plans.len(),
        report.results.iter().map(|r| &r.status).collect::<Vec<_>>()
    );
}

#[test]
fn device_pool_failover_enriches_recovery_plan() {
    use spanda_assurance::enrich_recovery_plan_with_failover;
    use spanda_config::{DeviceIdentityRecord, DeviceRegistry};

    let registry = DeviceRegistry {
        devices: vec![
            DeviceIdentityRecord {
                id: "gps-001".into(),
                device_type: "GPS".into(),
                logical_name: Some("gps".into()),
                redundant_group: Some("gps".into()),
                failover_priority: Some(1),
                lifecycle_state: Some("failed".into()),
                ..Default::default()
            },
            DeviceIdentityRecord {
                id: "gps-002".into(),
                device_type: "GPS".into(),
                redundant_group: Some("gps".into()),
                failover_priority: Some(2),
                lifecycle_state: Some("active".into()),
                ..Default::default()
            },
        ],
    };
    let program = parse_source("robot R { }");
    let mut plan = RecoveryPlanner::plan(
        &program,
        &RecoveryContext {
            issue: "gps-001 failed".into(),
            diagnosis: None,
            classification: None,
            level: RecoveryLevel::Level2AutomaticLowRisk,
        },
    );
    enrich_recovery_plan_with_failover(&mut plan, &registry, "gps-001");
    assert!(plan
        .actions
        .iter()
        .any(|a| a.description.contains("gps-002")));
}
