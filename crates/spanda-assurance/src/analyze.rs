//! Top-level mission assurance composition.
//!
use spanda_ast::nodes::Program;

use crate::anomaly::{scan_anomalies, AnomalyReport};
use crate::diagnosis::{diagnose_program, DiagnosisReport};
use crate::evidence::{build_assurance_report, AssuranceReport};
use crate::knowledge::validate_knowledge_models;
use crate::mission::{verify_mission_assurance, MissionAssuranceReport};
use crate::mitigation::extract_mitigations;
use crate::modes::validate_modes;
use crate::prognostics::{evaluate_prognostics, PrognosticsReport};
use crate::resilience::{check_resilience, ResilienceReport};
use crate::state::validate_state_estimators;

/// Composite mission assurance summary.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MissionAssuranceSummary {
    pub assurance: AssuranceReport,
    pub anomalies: AnomalyReport,
    pub prognostics: PrognosticsReport,
    pub resilience: ResilienceReport,
    pub mission: MissionAssuranceReport,
    pub issues: Vec<String>,
    pub passed: bool,
}

/// Run full mission assurance analysis on a program.
pub fn assure_program(program: &Program, source_label: &str) -> MissionAssuranceSummary {
    let assurance = build_assurance_report(program, source_label);
    let anomalies = scan_anomalies(program);
    let prognostics = evaluate_prognostics(program);
    let resilience = check_resilience(program);
    let mission = verify_mission_assurance(program);

    let mut issues = Vec::new();
    issues.extend(validate_knowledge_models(program));
    issues.extend(validate_state_estimators(program));
    issues.extend(validate_modes(program));

    let passed = assurance.passed
        && anomalies.passed
        && prognostics.passed
        && resilience.passed
        && mission.passed
        && issues.is_empty();

    MissionAssuranceSummary {
        assurance,
        anomalies,
        prognostics,
        resilience,
        mission,
        issues,
        passed,
    }
}

/// Re-export mitigation report builder for CLI.
pub fn mitigation_report(program: &Program) -> crate::mitigation::MitigationReport {
    extract_mitigations(program)
}

/// Re-export diagnosis for program-only path.
pub fn diagnosis_report(program: &Program) -> DiagnosisReport {
    diagnose_program(program)
}
