//! Report formatting for mission assurance CLI outputs.
//!
use crate::recovery::RecoveryReport;
use spanda_readiness::ReportFormat;

use crate::anomaly::AnomalyReport;
use crate::diagnosis::DiagnosisReport;
use crate::evidence::AssuranceReport;
use crate::mission::MissionAssuranceReport;
use crate::mitigation::MitigationReport;
use crate::prognostics::PrognosticsReport;
use crate::resilience::ResilienceReport;
use crate::state::StateAssuranceReport;

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Format assurance report for CLI output.
pub fn format_assurance(report: &AssuranceReport, format: ReportFormat) -> String {
    match format {
        ReportFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        ReportFormat::Markdown => format!(
            "# Assurance Report\n\n**Passed:** {}\n\n**Cases:** {}\n",
            report.passed,
            report.cases.len()
        ),
        ReportFormat::Html => format!(
            "<!DOCTYPE html><html><body><h1>Assurance Report</h1><p>Passed: {}</p></body></html>",
            report.passed
        ),
        ReportFormat::Text => format!(
            "Assurance Report\nPassed: {}\nCases: {}\n",
            report.passed,
            report.cases.len()
        ),
    }
}

/// Format anomaly scan report.
pub fn format_anomaly(report: &AnomalyReport, format: ReportFormat) -> String {
    match format {
        ReportFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        ReportFormat::Markdown => {
            let mut out = format!("# Anomaly Report\n\n**Passed:** {}\n\n", report.passed);
            if !report.anomalies.is_empty() {
                out.push_str("## Anomalies\n\n");
                for a in &report.anomalies {
                    out.push_str(&format!(
                        "- {} {} (expected {}, observed {})\n",
                        a.detector, a.metric, a.expected, a.observed
                    ));
                }
            }
            out
        }
        ReportFormat::Html => format!(
            "<!DOCTYPE html><html><body><h1>Anomaly Report</h1><p>Passed: {}</p></body></html>",
            report.passed
        ),
        ReportFormat::Text => {
            let mut out = format!("Anomaly Report\nPassed: {}\n", report.passed);
            for model in &report.learned {
                out.push_str(&format!(
                    "* learned {} via {}\n",
                    model.detector, model.backend
                ));
            }
            for a in &report.anomalies {
                out.push_str(&format!(
                    "* {} {} expected {} observed {}\n",
                    a.detector, a.metric, a.expected, a.observed
                ));
            }
            out
        }
    }
}

/// Format diagnosis report.
pub fn format_diagnosis(report: &DiagnosisReport, format: ReportFormat) -> String {
    match format {
        ReportFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        ReportFormat::Markdown => format!("# Diagnosis Report\n\n**Passed:** {}\n", report.passed),
        ReportFormat::Html => format!(
            "<!DOCTYPE html><html><body><h1>Diagnosis Report</h1><p>Passed: {}</p></body></html>",
            report.passed
        ),
        ReportFormat::Text => format!("Diagnosis Report\nPassed: {}\n", report.passed),
    }
}

/// Format prognostics report.
pub fn format_prognostics(report: &PrognosticsReport, format: ReportFormat) -> String {
    match format {
        ReportFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        ReportFormat::Markdown => format!(
            "# Prognostics Report\n\n**Passed:** {}\n\n**Warnings:** {}\n",
            report.passed,
            report.warnings.len()
        ),
        ReportFormat::Html => format!(
            "<!DOCTYPE html><html><body><h1>Prognostics Report</h1><p>Passed: {}</p></body></html>",
            report.passed
        ),
        ReportFormat::Text => format!(
            "Prognostics Report\nPassed: {}\nWarnings: {}\n",
            report.passed,
            report.warnings.len()
        ),
    }
}

/// Format mitigation plan report.
pub fn format_mitigation(report: &MitigationReport, format: ReportFormat) -> String {
    match format {
        ReportFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        ReportFormat::Markdown => {
            format!("# Mitigation Plan\n\n**Plans:** {}\n", report.plans.len())
        }
        ReportFormat::Html => format!(
            "<!DOCTYPE html><html><body><h1>Mitigation Plan</h1><p>Plans: {}</p></body></html>",
            report.plans.len()
        ),
        ReportFormat::Text => format!("Mitigation Plan\nPlans: {}\n", report.plans.len()),
    }
}

/// Format mission assurance report.
pub fn format_mission_assurance(report: &MissionAssuranceReport, format: ReportFormat) -> String {
    match format {
        ReportFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        ReportFormat::Markdown => format!(
            "# Mission Assurance Report\n\n**Passed:** {}\n\n**Plans:** {}\n",
            report.passed,
            report.plans.len()
        ),
        ReportFormat::Html => format!(
            "<!DOCTYPE html><html><body><h1>Mission Assurance</h1><p>Passed: {}</p></body></html>",
            html_escape(&report.passed.to_string())
        ),
        ReportFormat::Text => format!(
            "Mission Assurance Report\nPassed: {}\nPlans: {}\n",
            report.passed,
            report.plans.len()
        ),
    }
}

/// Format resilience check report.
pub fn format_resilience(report: &ResilienceReport, format: ReportFormat) -> String {
    match format {
        ReportFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        ReportFormat::Markdown => format!(
            "# Resilience Report\n\n**Passed:** {}\n\n**Readiness score:** {}\n",
            report.passed, report.readiness_score
        ),
        ReportFormat::Html => format!(
            "<!DOCTYPE html><html><body><h1>Resilience Report</h1><p>Passed: {}</p></body></html>",
            report.passed
        ),
        ReportFormat::Text => format!(
            "Resilience Report\nPassed: {}\nReadiness score: {}\n",
            report.passed, report.readiness_score
        ),
    }
}

/// Format state estimation assurance report.
pub fn format_state(report: &StateAssuranceReport, format: ReportFormat) -> String {
    match format {
        ReportFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        ReportFormat::Markdown => format!(
            "# State Estimation Report\n\n**Passed:** {}\n\n**Estimators:** {}\n\n**Belief estimates:** {}\n",
            report.passed,
            report.estimators.len(),
            report.belief.estimates.len()
        ),
        ReportFormat::Html => format!(
            "<!DOCTYPE html><html><body><h1>State Estimation</h1><p>Passed: {}</p></body></html>",
            report.passed
        ),
        ReportFormat::Text => {
            let mut out = format!(
                "State Estimation Report\nPassed: {}\nEstimators: {}\n",
                report.passed,
                report.estimators.len()
            );
            for est in &report.estimators {
                out.push_str(&format!(
                    "* {} inputs [{}]\n",
                    est.estimator,
                    est.inputs.join(", ")
                ));
            }
            for issue in &report.issues {
                out.push_str(&format!("! {issue}\n"));
            }
            out
        }
    }
}

/// Format recovery framework report for CLI output.
pub fn format_recovery(report: &RecoveryReport, format: ReportFormat) -> String {
    match format {
        ReportFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        ReportFormat::Markdown => {
            let mut out = format!(
                "# Recovery Report\n\n**Passed:** {}\n\n**Recovery Ready:** {}\n**Risk:** {}\n\n",
                report.passed,
                if report.readiness.recovery_ready {
                    "YES"
                } else {
                    "NO"
                },
                report.readiness.risk
            );
            for plan in &report.plans {
                out.push_str(&format!(
                    "## {}\n\n**Issue:** {}\n**Diagnosis:** {}\n**Risk:** {}\n\n",
                    plan.name, plan.failure, plan.diagnosis, plan.risk
                ));
                for action in &plan.actions {
                    out.push_str(&format!("- {}\n", action.description));
                }
                out.push('\n');
            }
            for result in &report.results {
                out.push_str(&format!(
                    "### Outcome: {:?}\n**Safety Validation:** {}\n\n",
                    result.status, result.evidence.safety_validation
                ));
            }
            out
        }
        ReportFormat::Html => format!(
            "<!DOCTYPE html><html><body><h1>Recovery Report</h1><p>Passed: {}</p><p>Recovery Ready: {}</p></body></html>",
            report.passed,
            if report.readiness.recovery_ready {
                "YES"
            } else {
                "NO"
            }
        ),
        ReportFormat::Text => {
            let mut out = String::new();
            if let Some(plan) = report.plans.first() {
                out.push_str(&format!("Issue:\n{}\n\n", plan.failure));
                out.push_str(&format!("Diagnosis:\n{}\n\n", plan.diagnosis));
                if let Some(action) = plan.actions.first() {
                    out.push_str(&format!("Recovery:\n{}\n\n", action.description));
                }
                out.push_str(&format!("Risk:\n{}\n\n", plan.risk));
            }
            if let Some(result) = report.results.first() {
                out.push_str(&format!(
                    "Safety Validation:\n{}\n\nOutcome:\n{:?}\n",
                    result.evidence.safety_validation, result.status
                ));
            }
            out.push_str(&format!(
                "Recovery Ready: {}\nSuccess Rate: {:.0}%\n",
                if report.readiness.recovery_ready {
                    "YES"
                } else {
                    "NO"
                },
                report.assurance.success_rate * 100.0
            ));
            out
        }
    }
}
