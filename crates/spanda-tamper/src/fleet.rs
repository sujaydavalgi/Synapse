//! Fleet-wide tamper correlation across multiple member mission traces.

use crate::diagnosis::{diagnose_tamper_trace, TamperDiagnosisReport};
use crate::detect::{TamperFormat, TamperSeverity, TamperStatus};
use crate::runtime::MissionTrace;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// One fleet member trace entry in a tamper manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetTamperMember {
    pub robot: String,
    pub trace: String,
}

/// Manifest listing fleet member traces for correlation analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetTamperManifest {
    pub fleet: String,
    pub members: Vec<FleetTamperMember>,
}

/// One correlated tamper pattern across fleet members.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetTamperCorrelation {
    pub kind: String,
    pub robots: Vec<String>,
    pub message: String,
    pub confidence: f64,
    pub sim_time_ms: Option<f64>,
}

/// Per-member tamper diagnosis summary inside a fleet report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemberTamperDiagnosis {
    pub robot: String,
    pub trace: String,
    pub tamper_status: TamperStatus,
    pub trust_score: u32,
    pub passed: bool,
    pub diagnosis: TamperDiagnosisReport,
}

/// Fleet-wide tamper correlation report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetTamperReport {
    pub fleet: String,
    pub members: Vec<MemberTamperDiagnosis>,
    pub correlations: Vec<FleetTamperCorrelation>,
    pub fleet_trust_score: u32,
    pub passed: bool,
}

/// Load a fleet tamper manifest from disk.
pub fn load_fleet_tamper_manifest(path: &Path) -> Result<FleetTamperManifest, String> {
    // Deserialize a fleet tamper manifest JSON file.
    //
    // Parameters:
    // - `path` — path to manifest.json
    //
    // Returns:
    // Parsed manifest or I/O / JSON error message.
    //
    // Options:
    // None.
    //
    // Example:
    // let manifest = load_fleet_tamper_manifest(Path::new("manifest.json"))?;

    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&raw).map_err(|error| error.to_string())
}

/// Correlate tamper signals across all fleet member traces in a manifest.
pub fn correlate_fleet_tamper(manifest_path: &Path) -> Result<FleetTamperReport, String> {
    // Diagnose each member trace and detect cross-robot tamper patterns.
    //
    // Parameters:
    // - `manifest_path` — fleet tamper manifest JSON
    //
    // Returns:
    // Fleet tamper correlation report.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = correlate_fleet_tamper(Path::new("manifest.json"))?;

    let manifest = load_fleet_tamper_manifest(manifest_path)?;
    let base_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let mut members = Vec::new();

    for member in &manifest.members {
        let trace_path = resolve_trace_path(base_dir, &member.trace);
        let raw = fs::read_to_string(&trace_path)
            .map_err(|error| format!("read {}: {error}", trace_path.display()))?;
        let trace: MissionTrace =
            serde_json::from_str(&raw).map_err(|error| format!("parse {}: {error}", member.trace))?;
        let label = trace_path.display().to_string();
        let diagnosis = diagnose_tamper_trace(&trace, &label);
        members.push(MemberTamperDiagnosis {
            robot: member.robot.clone(),
            trace: member.trace.clone(),
            tamper_status: diagnosis.tamper_status,
            trust_score: diagnosis.trust_score,
            passed: diagnosis.passed,
            diagnosis,
        });
    }

    let correlations = detect_fleet_correlations(&members);
    let fleet_trust_score = compute_fleet_trust_score(&members, &correlations);
    let passed = members.iter().all(|member| member.passed)
        && correlations.iter().all(|event| event.confidence < 0.85);

    Ok(build_fleet_tamper_report(
        manifest.fleet,
        members,
        correlations,
        fleet_trust_score,
        passed,
    ))
}

/// Build a fleet tamper report from pre-diagnosed member traces.
pub fn build_fleet_tamper_report(
    fleet: String,
    members: Vec<MemberTamperDiagnosis>,
    correlations: Vec<FleetTamperCorrelation>,
    fleet_trust_score: u32,
    passed: bool,
) -> FleetTamperReport {
    // Assemble a fleet tamper correlation report from member diagnoses.
    //
    // Parameters:
    // - `fleet` — fleet label
    // - `members` — per-robot tamper diagnoses
    // - `correlations` — cross-member correlation events
    // - `fleet_trust_score` — rolled-up fleet trust score
    // - `passed` — overall pass/fail
    //
    // Returns:
    // Fleet tamper correlation report.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = build_fleet_tamper_report("PatrolFleet".into(), members, correlations, 75, false);

    FleetTamperReport {
        fleet,
        members,
        correlations,
        fleet_trust_score,
        passed,
    }
}

/// Correlate tamper signals from in-memory member traces (mesh ingest or tests).
pub fn correlate_fleet_tamper_traces(
    fleet: &str,
    traces: &[(String, MissionTrace, String)],
) -> FleetTamperReport {
    // Diagnose each member trace and correlate cross-fleet tamper patterns.
    //
    // Parameters:
    // - `fleet` — fleet label
    // - `traces` — robot id, trace, and source label tuples
    //
    // Returns:
    // Fleet tamper correlation report.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = correlate_fleet_tamper_traces("PatrolFleet", &members);

    let mut members = Vec::new();
    for (robot, trace, label) in traces {
        let diagnosis = diagnose_tamper_trace(trace, label);
        members.push(MemberTamperDiagnosis {
            robot: robot.clone(),
            trace: label.clone(),
            tamper_status: diagnosis.tamper_status,
            trust_score: diagnosis.trust_score,
            passed: diagnosis.passed,
            diagnosis,
        });
    }
    let correlations = detect_fleet_correlations(&members);
    let fleet_trust_score = compute_fleet_trust_score(&members, &correlations);
    let passed = members.iter().all(|member| member.passed)
        && correlations.iter().all(|event| event.confidence < 0.85);
    build_fleet_tamper_report(
        fleet.into(),
        members,
        correlations,
        fleet_trust_score,
        passed,
    )
}

/// Format a fleet tamper report for CLI output.
pub fn format_fleet_tamper_report(report: &FleetTamperReport, format: TamperFormat) -> String {
    // Render fleet tamper report as JSON or human-readable text.
    //
    // Parameters:
    // - `report` — fleet tamper correlation report
    // - `format` — text or JSON output
    //
    // Returns:
    // Formatted report string.
    //
    // Options:
    // None.
    //
    // Example:
    // println!("{}", format_fleet_tamper_report(&report, TamperFormat::Text));

    match format {
        TamperFormat::Json => serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".into()),
        TamperFormat::Text => format_fleet_tamper_text(report),
    }
}

fn format_fleet_tamper_text(report: &FleetTamperReport) -> String {
    let mut lines = vec![
        format!("Fleet tamper correlation: {}", report.fleet),
        format!("Fleet trust score: {}/100", report.fleet_trust_score),
    ];

    lines.push(String::new());
    lines.push("Members:".into());
    for member in &report.members {
        lines.push(format!(
            "  {} ({}) — {:?}, trust {}/100, {}",
            member.robot,
            member.trace,
            member.tamper_status,
            member.trust_score,
            if member.passed { "PASS" } else { "FAIL" }
        ));
    }

    if !report.correlations.is_empty() {
        lines.push(String::new());
        lines.push("Correlations:".into());
        for event in &report.correlations {
            lines.push(format!(
                "  [{}] {} (confidence {:.0}%) — {}",
                event.kind,
                event.robots.join(", "),
                event.confidence * 100.0,
                event.message
            ));
        }
    } else {
        lines.push(String::new());
        lines.push("No cross-fleet tamper correlations.".into());
    }

    lines.push(String::new());
    lines.push(format!(
        "Result: {}",
        if report.passed { "PASS" } else { "FAIL" }
    ));
    lines.join("\n")
}

fn resolve_trace_path(base_dir: &Path, trace: &str) -> PathBuf {
    let path = Path::new(trace);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    }
}

fn detect_fleet_correlations(members: &[MemberTamperDiagnosis]) -> Vec<FleetTamperCorrelation> {
    let mut correlations = Vec::new();
    correlations.extend(detect_simultaneous_tamper(members));
    correlations.extend(detect_shared_agent_intrusion(members));
    correlations.extend(detect_coordinated_denials(members));
    correlations.sort_by(|left, right| {
        left.sim_time_ms
            .partial_cmp(&right.sim_time_ms)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    correlations
}

fn detect_simultaneous_tamper(members: &[MemberTamperDiagnosis]) -> Vec<FleetTamperCorrelation> {
    let mut events: Vec<(String, f64, TamperSeverity)> = Vec::new();
    for member in members {
        for timeline in &member.diagnosis.timeline {
            if timeline.severity >= TamperSeverity::Medium {
                events.push((member.robot.clone(), timeline.sim_time_ms, timeline.severity));
            }
        }
    }

    let mut correlations = Vec::new();
    for index in 0..events.len() {
        for other in (index + 1)..events.len() {
            let (robot_a, time_a, _) = &events[index];
            let (robot_b, time_b, _) = &events[other];
            if robot_a == robot_b {
                continue;
            }
            if (time_a - time_b).abs() > 1_000.0 {
                continue;
            }
            let robots = vec![robot_a.clone(), robot_b.clone()];
            if correlations
                .iter()
                .any(|existing: &FleetTamperCorrelation| existing.robots == robots)
            {
                continue;
            }
            correlations.push(FleetTamperCorrelation {
                kind: "simultaneous_tamper".into(),
                robots,
                message: format!(
                    "Tamper timeline events within {:.0}ms across fleet members",
                    (time_a - time_b).abs()
                ),
                confidence: 0.8,
                sim_time_ms: Some(time_a.min(*time_b)),
            });
        }
    }
    correlations
}

fn detect_shared_agent_intrusion(members: &[MemberTamperDiagnosis]) -> Vec<FleetTamperCorrelation> {
    let mut agent_to_robots: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for member in members {
        for component in &member.diagnosis.affected_components {
            if let Some(agent) = component.strip_prefix("agent:") {
                agent_to_robots
                    .entry(agent.to_string())
                    .or_default()
                    .insert(member.robot.clone());
            }
        }
    }

    agent_to_robots
        .into_iter()
        .filter(|(_, robots)| robots.len() >= 2)
        .map(|(agent, robots)| FleetTamperCorrelation {
            kind: "shared_agent_intrusion".into(),
            robots: robots.into_iter().collect(),
            message: format!("Agent '{agent}' appears in tamper signals on multiple fleet members"),
            confidence: 0.9,
            sim_time_ms: None,
        })
        .collect()
}

fn detect_coordinated_denials(members: &[MemberTamperDiagnosis]) -> Vec<FleetTamperCorrelation> {
    let mut denials: Vec<(String, f64, String)> = Vec::new();

    for member in members {
        for finding in &member.diagnosis.findings {
            if finding.category != "capability_monitor" && finding.category != "runtime_event" {
                continue;
            }
            if finding.severity < TamperSeverity::High {
                continue;
            }
            let sim_time = finding
                .evidence
                .as_deref()
                .map(parse_sim_time_from_evidence)
                .unwrap_or(0.0);
            denials.push((member.robot.clone(), sim_time, finding.message.clone()));
        }
    }

    let mut correlations = Vec::new();
    for index in 0..denials.len() {
        for other in (index + 1)..denials.len() {
            let (robot_a, time_a, _) = &denials[index];
            let (robot_b, time_b, _) = &denials[other];
            if robot_a == robot_b {
                continue;
            }
            if (time_a - time_b).abs() > 2_000.0 {
                continue;
            }
            let robots = vec![robot_a.clone(), robot_b.clone()];
            if correlations
                .iter()
                .any(|existing: &FleetTamperCorrelation| existing.robots == robots)
            {
                continue;
            }
            correlations.push(FleetTamperCorrelation {
                kind: "coordinated_denial".into(),
                robots,
                message: "High-severity capability denials recorded on multiple fleet members"
                    .into(),
                confidence: 0.85,
                sim_time_ms: Some(time_a.min(*time_b)),
            });
        }
    }
    correlations
}

fn compute_fleet_trust_score(
    members: &[MemberTamperDiagnosis],
    correlations: &[FleetTamperCorrelation],
) -> u32 {
    if members.is_empty() {
        return 100;
    }
    let average = members.iter().map(|member| member.trust_score).sum::<u32>() / members.len() as u32;
    let penalty = correlations
        .iter()
        .map(|event| {
            if event.confidence >= 0.9 {
                15
            } else if event.confidence >= 0.8 {
                10
            } else {
                5
            }
        })
        .sum::<u32>();
    average.saturating_sub(penalty)
}

fn parse_sim_time_from_evidence(evidence: &str) -> f64 {
    evidence
        .split("sim_time_ms=")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|value| value.trim_end_matches(',').parse().ok())
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correlates_shared_agent_across_fleet_traces() {
        let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/showcase/fleet_tamper/manifest.json");
        let report = correlate_fleet_tamper(&manifest_path).expect("correlate fleet");
        assert!(!report.passed);
        assert!(report
            .correlations
            .iter()
            .any(|event| event.kind == "shared_agent_intrusion"));
        assert!(report.members.len() >= 2);
    }
}
