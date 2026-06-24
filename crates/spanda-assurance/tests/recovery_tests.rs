//! Integration tests for self-healing and recovery framework.

use spanda_assurance::{
    evaluate_recovery, extract_recovery_policies, format_recovery, simulate_failure_recovery,
    RecoveryContext, RecoveryLevel, RecoveryStatus,
};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_readiness::ReportFormat;

fn parse_source(source: &str) -> spanda_ast::nodes::Program {
    parse(tokenize(source).unwrap()).unwrap()
}

const SELF_HEALING: &str = include_str!("../../../examples/showcase/self_healing/rover.sd");

#[test]
fn recovery_policy_parses_from_showcase() {
    let program = parse_source(SELF_HEALING);
    let policies = extract_recovery_policies(&program);
    assert!(!policies.is_empty());
    assert!(policies[0].triggers.iter().any(|(c, _)| c.contains("gps")));
}

#[test]
fn heal_workflow_produces_passing_report() {
    let program = parse_source(SELF_HEALING);
    let report = evaluate_recovery(&program, None);
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
    let program = parse_source(SELF_HEALING);
    let report = simulate_failure_recovery(&program, "gps");
    assert_eq!(report.plans[0].diagnosis, "Satellite lock lost");
    assert!(!report.results.is_empty());
}

#[test]
fn recovery_readiness_evaluated() {
    let program = parse_source(SELF_HEALING);
    let ctx = RecoveryContext {
        issue: "gps.failed".into(),
        diagnosis: None,
        classification: None,
        level: RecoveryLevel::Level3AutomaticWithValidation,
    };
    let report = evaluate_recovery(&program, Some(&ctx));
    assert!(report.readiness.readiness_score > 0 || report.readiness.blockers.is_empty());
}
