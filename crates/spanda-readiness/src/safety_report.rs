//! Deployable safety case report generation.

use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use spanda_capability::{
    capability_traceability, check_minimum_capabilities, evaluate_health_checks,
    hardware_traceability, health_traceability,
};
use spanda_certify::verify::verify_certification_proof;
use spanda_hardware::{verify_program_compatibility, CompatSeverity, VerifyOptions};

use crate::mission::verify_mission;
use crate::traceability::readiness_traceability;

/// Comprehensive safety evidence report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafetyCaseReport {
    pub program: String,
    pub hardware_verification: serde_json::Value,
    pub capability_verification: serde_json::Value,
    pub health_checks: serde_json::Value,
    pub safety_rules: Vec<String>,
    pub kill_switch_validation: Vec<String>,
    pub connectivity_validation: Vec<String>,
    pub mission_verification: serde_json::Value,
    pub traceability_matrix: serde_json::Value,
    pub test_results: Vec<String>,
    pub known_risks: Vec<String>,
    pub deployable: bool,
}

/// Generate a safety case report for deployment evidence.
pub fn generate_safety_report(program: &Program, source_label: &str) -> SafetyCaseReport {
    let verify_opts = VerifyOptions::default();
    let hw = verify_program_compatibility(program, &verify_opts);
    let minimum = check_minimum_capabilities(program);
    let health = evaluate_health_checks(program);
    let trace = capability_traceability(program);
    let hw_trace = hardware_traceability(program);
    let health_trace = health_traceability(program);
    let missions = verify_mission(program, None);
    let readiness_trace = readiness_traceability(program);

    let mut safety_rules = Vec::new();
    let mut kill_switches = Vec::new();
    let mut connectivity = Vec::new();
    let Program::Program {
        kill_switches: ks_decls,
        robots,
        ..
    } = program;

    for ks in ks_decls {
        let spanda_ast::foundations::KillSwitchDecl::KillSwitchDecl { name, .. } = ks;
        kill_switches.push(format!("Kill switch '{name}' declared"));
    }
    for robot in robots {
        let spanda_ast::nodes::RobotDecl::RobotDecl { safety, .. } = robot;
        if let Some(spanda_ast::nodes::SafetyBlock::SafetyBlock { rules, .. }) = safety {
            for rule in rules {
                safety_rules.push(format!("{rule:?}"));
            }
        }
    }
    for item in &hw.items {
        if item.category == "connectivity" {
            connectivity.push(item.message.clone());
        }
    }

    let certify = verify_certification_proof(program, false);
    let mut known_risks = Vec::new();
    for err in &minimum.errors {
        known_risks.push(err.clone());
    }
    for warn in &trace.warnings {
        known_risks.push(warn.clone());
    }
    for item in certify
        .iter()
        .filter(|i| i.severity == CompatSeverity::Error)
    {
        known_risks.push(item.message.clone());
    }

    let deployable = hw.compatible && minimum.compatible && known_risks.is_empty();

    SafetyCaseReport {
        program: source_label.into(),
        hardware_verification: serde_json::to_value(&hw).unwrap_or_default(),
        capability_verification: serde_json::to_value(&minimum).unwrap_or_default(),
        health_checks: serde_json::json!({
            "report": health,
            "traceability": health_trace,
        }),
        safety_rules,
        kill_switch_validation: kill_switches,
        connectivity_validation: connectivity,
        mission_verification: serde_json::to_value(&missions).unwrap_or_default(),
        traceability_matrix: serde_json::json!({
            "hardware": hw_trace.hardware_rows,
            "capabilities": trace.capability_rows,
            "readiness": readiness_trace,
        }),
        test_results: vec![
            format!("Hardware compatible: {}", hw.compatible),
            format!("Minimum capabilities: {}", minimum.compatible),
            format!("Health checks: {:?}", health.overall),
        ],
        known_risks,
        deployable,
    }
}

/// Generate safety report from source.
pub fn generate_safety_report_source(
    source: &str,
    label: &str,
) -> Result<SafetyCaseReport, spanda_error::SpandaError> {
    let tokens = spanda_lexer::tokenize(source)?;
    let program = spanda_parser::parse(tokens)?;
    Ok(generate_safety_report(&program, label))
}
