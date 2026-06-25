//! Scorecard composition from readiness, assurance, security, and verification engines.

use serde::{Deserialize, Serialize};
use spanda_assurance::{assure_program_with_config, evaluate_recovery_coverage};
use spanda_ast::nodes::Program;
use spanda_capability::evaluate_health_checks;
use spanda_config::ResolvedSystemConfig;
use spanda_hardware::{verify_program_compatibility, VerifyOptions};
use spanda_readiness::{
    audit_program, evaluate_readiness, evaluate_safety_coverage, verify_mission, ReadinessOptions,
};
use spanda_threat::analyze_threat_model;
use std::sync::Arc;

/// Output format for scorecard rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScorecardFormat {
    #[default]
    Text,
    Json,
    Markdown,
}

/// Options for scorecard evaluation.
#[derive(Debug, Clone, Default)]
pub struct ScorecardOptions {
    pub system_config: Option<Arc<ResolvedSystemConfig>>,
}

/// Per-pillar scorecard category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScorecardCategory {
    pub name: String,
    pub score: u32,
    pub weight: u32,
    pub weighted: u32,
    pub detail: String,
}

/// Executive scorecard rollup for a mission program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScorecardReport {
    pub program: String,
    pub categories: Vec<ScorecardCategory>,
    pub overall_score: u32,
    pub tier: String,
    pub recommendations: Vec<String>,
}

/// Evaluate an autonomous systems scorecard by composing existing engines.
pub fn evaluate_scorecard(
    program: &Program,
    source_label: &str,
    options: &ScorecardOptions,
) -> ScorecardReport {
    // Roll up readiness, safety, security, verification, assurance, resilience, and health scores.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    // - `source_label` — file label
    // - `options` — optional resolved configuration
    //
    // Returns:
    // Weighted scorecard with category breakdown and recommendations.
    //
    // Options:
    // `ScorecardOptions::system_config`.
    //
    // Example:
    // let report = evaluate_scorecard(&program, "rover.sd", &ScorecardOptions::default());

    let config = options.system_config.as_deref();
    let readiness_options = ReadinessOptions {
        system_config: options.system_config.clone(),
        ..ReadinessOptions::default()
    };
    let readiness = evaluate_readiness(program, &readiness_options);
    let safety_coverage = evaluate_safety_coverage(program, source_label);
    let safety_audit = audit_program(program, source_label);
    let threat = analyze_threat_model(program, source_label);
    let hardware = verify_program_compatibility(program, &VerifyOptions::default());
    let missions = verify_mission(program, None);
    let assurance = assure_program_with_config(program, source_label, config);
    let recovery = evaluate_recovery_coverage(program, source_label);
    let health = evaluate_health_checks(program);

    let readiness_score = pct(readiness.score.total, readiness.score.maximum);
    let safety_score = safety_coverage
        .overall_coverage_pct
        .saturating_sub(safety_audit.critical_count.saturating_mul(20))
        .saturating_sub(safety_audit.high_count.saturating_mul(8))
        .min(100);
    let security_score = 100u32.saturating_sub(threat.risk_score);
    let verification_score = verification_score(&hardware, &missions);
    let assurance_score = if assurance.passed {
        100
    } else {
        100u32.saturating_sub((assurance.issues.len() as u32).saturating_mul(8))
    };
    let resilience_score = recovery.coverage_pct;
    let health_score = health_score_from_report(&health);

    let categories = vec![
        category("readiness", readiness_score, 20, &format!(
            "mission_ready={} score {}/{}",
            readiness.mission_ready, readiness.score.total, readiness.score.maximum
        )),
        category("safety", safety_score, 20, &format!(
            "coverage {}% audit critical={} high={}",
            safety_coverage.overall_coverage_pct,
            safety_audit.critical_count,
            safety_audit.high_count
        )),
        category("health", health_score, 15, &format!(
            "checks={} overall={:?}",
            health.checks.len(),
            health.overall
        )),
        category("security", security_score, 15, &format!(
            "threat risk {}/100 assessments={}",
            threat.risk_score,
            threat.assessments.len()
        )),
        category("verification", verification_score, 10, &format!(
            "hardware_compatible={} missions={}",
            hardware.compatible,
            missions.len()
        )),
        category("assurance", assurance_score, 10, &format!(
            "passed={} issues={}",
            assurance.passed,
            assurance.issues.len()
        )),
        category("resilience", resilience_score, 10, &format!(
            "recovery coverage {}%",
            recovery.coverage_pct
        )),
    ];

    let overall_score = categories.iter().map(|entry| entry.weighted).sum();
    let tier = score_tier(overall_score);
    let mut recommendations = Vec::new();
    if !readiness.mission_ready {
        recommendations.push("Run `spanda explain readiness` and resolve blockers".into());
    }
    if safety_audit.critical_count > 0 {
        recommendations.push("Resolve critical safety audit findings before deploy".into());
    }
    if threat.risk_score >= 30 {
        recommendations.push("Review `spanda threat-model` mitigations".into());
    }
    if !hardware.compatible {
        recommendations.push("Run `spanda verify` for hardware compatibility".into());
    }
    if !assurance.passed {
        recommendations.push("Run `spanda assure` and address assurance gaps".into());
    }
    if recovery.coverage_pct < 80 {
        recommendations.push("Improve recovery coverage with `spanda recovery-coverage`".into());
    }
    recommendations.extend(safety_coverage.recommendations.into_iter().take(2));
    ScorecardReport {
        program: source_label.into(),
        categories,
        overall_score,
        tier,
        recommendations,
    }
}

fn category(name: &str, score: u32, weight: u32, detail: &str) -> ScorecardCategory {
    let weighted = score.saturating_mul(weight) / 100;
    ScorecardCategory {
        name: name.into(),
        score,
        weight,
        weighted,
        detail: detail.into(),
    }
}

fn pct(value: u32, maximum: u32) -> u32 {
    if maximum == 0 {
        return 0;
    }
    ((value as f64 / maximum as f64) * 100.0).round() as u32
}

fn verification_score(
    hardware: &spanda_hardware::CompatibilityReport,
    missions: &[spanda_readiness::MissionVerificationReport],
) -> u32 {
    let mut score = 100u32;
    if !hardware.compatible {
        score = score.saturating_sub(30);
    }
    score = score.saturating_sub(
        hardware
            .items
            .iter()
            .filter(|item| item.severity == spanda_hardware::CompatSeverity::Error)
            .count() as u32
            * 10,
    );
    let failed_missions = missions.iter().filter(|m| !m.achievable).count() as u32;
    score.saturating_sub(failed_missions.saturating_mul(15))
}

fn health_score_from_report(health: &spanda_capability::HealthReport) -> u32 {
    if health.checks.is_empty() {
        return if health.policies.is_empty() { 60 } else { 75 };
    }
    let bad = health
        .checks
        .iter()
        .filter(|check| {
            matches!(
                check.status,
                spanda_capability::HealthStatus::Critical
                    | spanda_capability::HealthStatus::Failed
                    | spanda_capability::HealthStatus::Unsafe
            )
        })
        .count() as u32;
    100u32.saturating_sub(bad.saturating_mul(15))
}

fn score_tier(score: u32) -> String {
    if score >= 90 {
        "excellent".into()
    } else if score >= 75 {
        "good".into()
    } else if score >= 60 {
        "acceptable".into()
    } else {
        "needs_attention".into()
    }
}

/// Format a scorecard report for CLI output.
pub fn format_scorecard(report: &ScorecardReport, format: ScorecardFormat) -> String {
    // Render scorecard as text, JSON, or markdown.
    //
    // Parameters:
    // - `report` — scorecard report
    // - `format` — output format
    //
    // Returns:
    // Formatted scorecard string.
    //
    // Options:
    // None.
    //
    // Example:
    // let text = format_scorecard(&report, ScorecardFormat::Text);

    match format {
        ScorecardFormat::Json => {
            serde_json::to_string_pretty(report).unwrap_or_else(|e| e.to_string())
        }
        ScorecardFormat::Markdown => {
            let mut lines = vec![
                format!("# Scorecard: {}", report.program),
                format!(
                    "**Overall:** {}/100 ({})",
                    report.overall_score, report.tier
                ),
                String::new(),
                "| Category | Score | Weight |".into(),
                "|----------|-------|--------|".into(),
            ];
            for category in &report.categories {
                lines.push(format!(
                    "| {} | {} | {}% |",
                    category.name, category.score, category.weight
                ));
            }
            if !report.recommendations.is_empty() {
                lines.push(String::new());
                lines.push("## Recommendations".into());
                for rec in &report.recommendations {
                    lines.push(format!("- {rec}"));
                }
            }
            lines.join("\n")
        }
        ScorecardFormat::Text => {
            let mut lines = vec![
                format!("Scorecard: {}", report.program),
                format!(
                    "Overall: {}/100 ({})",
                    report.overall_score, report.tier
                ),
                "Categories:".into(),
            ];
            for category in &report.categories {
                lines.push(format!(
                    "  {} — {}/100 (weight {}%) — {}",
                    category.name, category.score, category.weight, category.detail
                ));
            }
            if !report.recommendations.is_empty() {
                lines.push("Recommendations:".into());
                for rec in &report.recommendations {
                    lines.push(format!("  - {rec}"));
                }
            }
            lines.join("\n")
        }
    }
}
