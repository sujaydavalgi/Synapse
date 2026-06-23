//! Report formatting for readiness and safety outputs.

use crate::auditor::SafetyAuditReport;
use crate::failure::FailureAnalysisReport;
use crate::mission::MissionVerificationReport;
use crate::root_cause::RootCauseReport;
use crate::safety_report::SafetyCaseReport;
use crate::types::FleetReadinessReport;
use crate::types::{ReadinessReport, ReportFormat, TwinReadinessStatus};

/// Format a readiness report for CLI output.
pub fn format_readiness(report: &ReadinessReport, format: ReportFormat) -> String {
    match format {
        ReportFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        ReportFormat::Markdown => format_readiness_markdown(report),
        ReportFormat::Html => format_readiness_html(report),
        ReportFormat::Text => format_readiness_text(report),
    }
}

fn format_readiness_text(report: &ReadinessReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Mission Ready: {}\n",
        if report.mission_ready { "YES" } else { "NO" }
    ));
    out.push_str(&format!(
        "Score: {}/{}\n",
        report.score.total, report.score.maximum
    ));
    out.push_str(&format!("Status: {:?}\n", report.status));
    if !report.issues.is_empty() {
        out.push_str("\nIssues:\n");
        for issue in &report.issues {
            out.push_str(&format!("* {}\n", issue.message));
        }
    }
    out
}

fn format_readiness_markdown(report: &ReadinessReport) -> String {
    let mut out = String::new();
    out.push_str("# Readiness Report\n\n");
    out.push_str(&format!(
        "**Mission Ready:** {}\n\n",
        if report.mission_ready { "YES" } else { "NO" }
    ));
    out.push_str(&format!(
        "**Score:** {}/{}\n\n",
        report.score.total, report.score.maximum
    ));
    if !report.issues.is_empty() {
        out.push_str("## Issues\n\n");
        for issue in &report.issues {
            out.push_str(&format!("- {}\n", issue.message));
        }
    }
    out
}

fn format_readiness_html(report: &ReadinessReport) -> String {
    let issues: String = report
        .issues
        .iter()
        .map(|i| format!("<li>{}</li>", html_escape(&i.message)))
        .collect();
    format!(
        "<!DOCTYPE html><html><head><title>Readiness Report</title></head><body>\
         <h1>Readiness Report</h1>\
         <p><strong>Mission Ready:</strong> {}</p>\
         <p><strong>Score:</strong> {}/{}</p>\
         <h2>Issues</h2><ul>{issues}</ul>\
         </body></html>",
        if report.mission_ready { "YES" } else { "NO" },
        report.score.total,
        report.score.maximum
    )
}

/// Format safety case report.
pub fn format_safety_report(report: &SafetyCaseReport, format: ReportFormat) -> String {
    match format {
        ReportFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        ReportFormat::Markdown => {
            let mut out = String::from("# Safety Case Report\n\n");
            out.push_str(&format!(
                "**Deployable:** {}\n\n",
                if report.deployable { "YES" } else { "NO" }
            ));
            if !report.known_risks.is_empty() {
                out.push_str("## Known Risks\n\n");
                for risk in &report.known_risks {
                    out.push_str(&format!("- {risk}\n"));
                }
            }
            out
        }
        ReportFormat::Html => format!(
            "<!DOCTYPE html><html><head><title>Safety Report</title></head><body>\
             <h1>Safety Case Report</h1><p>Deployable: {}</p></body></html>",
            if report.deployable { "YES" } else { "NO" }
        ),
        ReportFormat::Text => format!(
            "Deployable: {}\nKnown risks: {}\n",
            if report.deployable { "YES" } else { "NO" },
            report.known_risks.len()
        ),
    }
}

/// Format failure analysis report.
pub fn format_failure_analysis(report: &FailureAnalysisReport) -> String {
    let mut out = String::new();
    for impact in &report.impacts {
        out.push_str(&format!("If {} fails:\n", impact.component));
        out.push_str(&format!("  {}\n", impact.mitigation));
        out.push('\n');
    }
    out
}

/// Format fleet readiness report.
pub fn format_fleet_readiness(report: &FleetReadinessReport) -> String {
    format!(
        "Fleet Score: {}/100\nHealthy Robots:\n{}\nDegraded Robots:\n{}\nMission Capacity:\n{}%\n",
        report.fleet_score,
        report.healthy_robots,
        report.degraded_robots,
        report.mission_capacity_percent
    )
}

/// Format mission verification reports.
pub fn format_mission_verification(reports: &[MissionVerificationReport]) -> String {
    let mut out = String::new();
    for r in reports {
        out.push_str(&format!(
            "Mission {:?} on {:?}: {}\n",
            r.mission_name,
            r.robot,
            if r.achievable {
                "ACHIEVABLE"
            } else {
                "BLOCKED"
            }
        ));
        for issue in &r.issues {
            out.push_str(&format!("  - {issue}\n"));
        }
    }
    out
}

/// Format root cause report.
pub fn format_root_cause(report: &RootCauseReport) -> String {
    let mut out = format!(
        "Root Cause\n{}\n\nContributing Factors\n",
        report.root_cause
    );
    for f in &report.contributing_factors {
        out.push_str(&format!("* {f}\n"));
    }
    out.push_str("\nTimeline\n");
    for e in report.timeline.iter().take(20) {
        out.push_str(&format!(
            "  T+{:.0}ms {} — {}\n",
            e.sim_time_ms, e.event, e.detail
        ));
    }
    out.push_str("\nRecommended Actions\n");
    for a in &report.recommended_actions {
        out.push_str(&format!("* {a}\n"));
    }
    out
}

/// Format safety audit report.
pub fn format_audit(report: &SafetyAuditReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Critical: {}  High: {}  Medium: {}  Low: {}\n",
        report.critical_count, report.high_count, report.medium_count, report.low_count
    ));
    for f in &report.findings {
        out.push_str(&format!(
            "[{:?}] {} — {}\n",
            f.severity, f.category, f.message
        ));
    }
    out
}

/// Format twin readiness status.
pub fn format_twin_readiness(status: &TwinReadinessStatus) -> String {
    format!(
        "Physical Ready: {}\nTwin Ready: {}\nConfiguration Drift: {}\nCapability Drift: {}\nHealth Drift: {}\n",
        status.physical_ready,
        status.twin_ready,
        status.configuration_drift.len(),
        status.capability_drift.len(),
        status.health_drift.len()
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
