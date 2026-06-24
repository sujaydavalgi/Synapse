//! Assurance evidence and assurance case analysis.
//!
use crate::types::{
    AssuranceCase, EvidenceKind, EvidenceRecord, SafetyEvidence, TraceabilityEvidence,
    VerificationEvidence,
};
use spanda_ast::assurance_decl::AssuranceCaseDecl;
use spanda_ast::nodes::Program;
use spanda_capability::{
    capability_traceability, check_minimum_capabilities, evaluate_health_checks,
    hardware_traceability,
};
use spanda_hardware::{verify_program_compatibility, VerifyOptions};
use spanda_readiness::{generate_safety_report, readiness_traceability, SafetyCaseReport};

/// Assurance report combining evidence from all subsystems.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AssuranceReport {
    pub cases: Vec<AssuranceCase>,
    pub verification: VerificationEvidence,
    pub safety: SafetyEvidence,
    pub traceability: TraceabilityEvidence,
    pub safety_case: SafetyCaseReport,
    pub passed: bool,
}

fn evidence_kind(source: &str) -> EvidenceKind {
    match source {
        s if s.contains("hardware") => EvidenceKind::Hardware,
        s if s.contains("capability") || s.contains("traceability") => EvidenceKind::Capability,
        s if s.contains("health") => EvidenceKind::Health,
        s if s.contains("replay") || s.contains("simulation") => EvidenceKind::Replay,
        s if s.contains("safety") => EvidenceKind::Safety,
        _ => EvidenceKind::Verification,
    }
}

/// Build assurance report from program declarations and existing verification.
pub fn build_assurance_report(program: &Program, source_label: &str) -> AssuranceReport {
    let Program::Program {
        assurance_cases, ..
    } = program;

    let cases: Vec<AssuranceCase> = assurance_cases
        .iter()
        .map(|decl| {
            let AssuranceCaseDecl::AssuranceCaseDecl { name, evidence, .. } = decl;
            AssuranceCase {
                name: name.clone(),
                evidence: evidence
                    .iter()
                    .map(|e| EvidenceRecord {
                        source: e.clone(),
                        kind: evidence_kind(e),
                        status: "linked".into(),
                    })
                    .collect(),
            }
        })
        .collect();

    let hw = verify_program_compatibility(program, &VerifyOptions::default());
    let cap = check_minimum_capabilities(program);
    let health = evaluate_health_checks(program);
    let hw_trace = hardware_traceability(program);
    let cap_trace = capability_traceability(program);
    let readiness_trace = readiness_traceability(program);

    let verification = VerificationEvidence {
        compatible: hw.compatible && cap.compatible,
        items: hw
            .items
            .iter()
            .take(10)
            .map(|i| i.message.clone())
            .collect(),
    };

    let safety_case = generate_safety_report(program, source_label);

    let safety = SafetyEvidence {
        rules: safety_case.safety_rules.clone(),
        kill_switches: safety_case.kill_switch_validation.clone(),
    };

    let mut trace_rows: Vec<String> = hw_trace
        .hardware_rows
        .iter()
        .map(|r| format!("hardware: {}", r.hardware_component))
        .collect();
    trace_rows.extend(
        cap_trace
            .capability_rows
            .iter()
            .map(|r| format!("capability: {}", r.capability)),
    );
    trace_rows.extend(
        readiness_trace
            .iter()
            .map(|r| format!("readiness: {}", r.mission_requirement)),
    );

    let traceability = TraceabilityEvidence { rows: trace_rows };

    let passed = verification.compatible
        && health.overall != spanda_capability::HealthStatus::Failed
        && !cases.is_empty();

    AssuranceReport {
        cases,
        verification,
        safety,
        traceability,
        safety_case,
        passed,
    }
}
