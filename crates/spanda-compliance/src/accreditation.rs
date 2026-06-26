//! Compliance accreditation bundle generation for audit exports.

use crate::evaluate::{evaluate_compliance_profile, ComplianceEvaluationReport, ComplianceViolation};
use crate::profiles::{builtin_profile, ComplianceProfile};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use spanda_ast::nodes::Program;
use spanda_tamper::{evaluate_secure_boot_coverage, tamper_policy_coverage};

/// One evidence item recorded in an accreditation export bundle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplianceEvidenceItem {
    pub id: String,
    pub label: String,
    pub status: String,
    pub detail: String,
}

/// Accreditation export bundle for engineering audit trails (not legal certification).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplianceAccreditationReport {
    pub profile: String,
    pub program: String,
    pub description: String,
    pub accreditation_status: String,
    pub template_notice: String,
    pub audit_export_id: String,
    pub generated_at: String,
    pub passed: bool,
    pub evidence_checklist: Vec<ComplianceEvidenceItem>,
    pub evaluation: ComplianceEvaluationReport,
}

/// Generate an accreditation export bundle for a compliance profile evaluation.
pub fn generate_accreditation_report(
    program: &Program,
    profile_name: &str,
    source_label: &str,
) -> Result<ComplianceAccreditationReport, String> {
    // Build an audit export bundle with evidence checklist and template disclaimer.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    // - `profile_name` — built-in profile name
    // - `source_label` — file label
    //
    // Returns:
    // Accreditation bundle suitable for JSON/Markdown export.
    //
    // Options:
    // None.
    //
    // Example:
    // let bundle = generate_accreditation_report(&program, "defense", "defense_rover.sd")?;

    let profile = builtin_profile(profile_name)
        .ok_or_else(|| format!("Unknown compliance profile '{profile_name}'"))?;
    let evaluation = evaluate_compliance_profile(program, profile_name, source_label)?;
    let evidence_checklist = build_evidence_checklist(program, &profile, &evaluation);
    let audit_export_id = compute_audit_export_id(profile_name, source_label, &evaluation);
    let generated_at = chrono_like_timestamp();

    Ok(ComplianceAccreditationReport {
        profile: profile.name.clone(),
        program: source_label.into(),
        description: profile.description.clone(),
        accreditation_status: "template_only".into(),
        template_notice: profile.template_notice.to_string(),
        audit_export_id,
        generated_at,
        passed: evaluation.passed,
        evidence_checklist,
        evaluation,
    })
}

/// Format an accreditation bundle for CLI output.
pub fn format_accreditation_report(report: &ComplianceAccreditationReport, json: bool) -> String {
    // Render accreditation bundle as JSON or Markdown audit export.
    //
    // Parameters:
    // - `report` — accreditation bundle
    // - `json` — emit JSON when true, else Markdown
    //
    // Returns:
    // Formatted report string.
    //
    // Options:
    // None.
    //
    // Example:
    // println!("{}", format_accreditation_report(&report, false));

    if json {
        return serde_json::to_string_pretty(report).unwrap_or_else(|error| error.to_string());
    }

    let mut lines = vec![
        format!("# Compliance accreditation export — {}", report.profile),
        format!("Program: `{}`", report.program),
        report.description.clone(),
        String::new(),
        format!("**Status:** {}", report.accreditation_status),
        format!("**Audit export ID:** `{}`", report.audit_export_id),
        format!("**Generated:** {}", report.generated_at),
        String::new(),
        format!("> {}", report.template_notice),
        String::new(),
        format!(
            "**Evaluation:** {}",
            if report.passed { "PASS" } else { "FAIL" }
        ),
        String::new(),
        "## Evidence checklist".into(),
    ];

    for item in &report.evidence_checklist {
        lines.push(format!(
            "- [{}] **{}** — {} ({})",
            if item.status == "ok" { "x" } else { " " },
            item.label,
            item.detail,
            item.status
        ));
    }

    if !report.evaluation.violations.is_empty() {
        lines.push(String::new());
        lines.push("## Violations".into());
        for violation in &report.evaluation.violations {
            lines.push(format!(
                "- [{:?}] {} — {}",
                violation.severity, violation.requirement, violation.message
            ));
        }
    }

    lines.join("\n")
}

fn build_evidence_checklist(
    program: &Program,
    profile: &ComplianceProfile,
    evaluation: &ComplianceEvaluationReport,
) -> Vec<ComplianceEvidenceItem> {
    let mut items = Vec::new();
    push_requirement(
        &mut items,
        "kill_switch",
        "Emergency kill switch declared",
        profile.requires_kill_switch,
        !violated(&evaluation.violations, "requires_kill_switch"),
    );
    push_requirement(
        &mut items,
        "readiness",
        "Readiness score meets profile minimum",
        profile.min_readiness_score > 0,
        !violated(&evaluation.violations, "min_readiness_score"),
    );
    push_requirement(
        &mut items,
        "health_checks",
        "Required health checks present",
        profile.min_health_checks > 0,
        !violated(&evaluation.violations, "min_health_checks"),
    );
    push_requirement(
        &mut items,
        "assurance_case",
        "Assurance case evidence declared",
        profile.requires_assurance_case,
        !violated(&evaluation.violations, "requires_assurance_case"),
    );
    push_requirement(
        &mut items,
        "secure_comm",
        "Secure communication posture",
        profile.requires_secure_comm,
        !violated(&evaluation.violations, "requires_secure_comm"),
    );

    let (has_policy, branches) = tamper_policy_coverage(program);
    items.push(ComplianceEvidenceItem {
        id: "tamper_policy".into(),
        label: "Tamper response policy".into(),
        status: if !profile.requires_tamper_policy || (has_policy && branches > 0) {
            "ok".into()
        } else {
            "gap".into()
        },
        detail: format!("{branches} tamper_policy branches"),
    });

    let secure_boot = evaluate_secure_boot_coverage(program, Some(&evaluation.program));
    items.push(ComplianceEvidenceItem {
        id: "secure_boot".into(),
        label: "Secure-boot contract coverage".into(),
        status: if !profile.requires_secure_boot || secure_boot.passed {
            "ok".into()
        } else {
            "gap".into()
        },
        detail: format!(
            "{}/100 score, {} contracts",
            secure_boot.score,
            secure_boot.contracts.len()
        ),
    });

    items.push(ComplianceEvidenceItem {
        id: "template_notice".into(),
        label: "Accreditation disclaimer".into(),
        status: "ok".into(),
        detail: profile.template_notice.to_string(),
    });
    items
}

fn push_requirement(
    items: &mut Vec<ComplianceEvidenceItem>,
    id: &str,
    label: &str,
    required: bool,
    satisfied: bool,
) {
    if !required {
        return;
    }
    items.push(ComplianceEvidenceItem {
        id: id.into(),
        label: label.into(),
        status: if satisfied { "ok".into() } else { "gap".into() },
        detail: if satisfied {
            "requirement satisfied".into()
        } else {
            "requirement not satisfied".into()
        },
    });
}

fn violated(violations: &[ComplianceViolation], requirement: &str) -> bool {
    violations
        .iter()
        .any(|violation| violation.requirement == requirement)
}

fn compute_audit_export_id(
    profile_name: &str,
    source_label: &str,
    evaluation: &ComplianceEvaluationReport,
) -> String {
    let payload = format!(
        "{profile_name}|{source_label}|{}|{}",
        evaluation.passed,
        evaluation.violations.len()
    );
    let digest = Sha256::digest(payload.as_bytes());
    hex::encode(&digest[..8])
}

fn chrono_like_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("unix-{seconds}")
}
