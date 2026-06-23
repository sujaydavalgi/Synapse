//! Root cause analysis leveraging mission replay traces.

use serde::{Deserialize, Serialize};
use spanda_error::SpandaError;
use spanda_runtime::replay::MissionTrace;
use std::path::Path;

/// Root cause diagnosis from a mission trace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RootCauseReport {
    pub root_cause: String,
    pub contributing_factors: Vec<String>,
    pub timeline: Vec<TimelineEvent>,
    pub recommended_actions: Vec<String>,
}

/// Timeline event extracted from trace frames.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub sim_time_ms: f64,
    pub event: String,
    pub detail: String,
}

/// Diagnose a mission failure from a recorded trace file.
pub fn diagnose_trace(trace_path: &Path) -> Result<RootCauseReport, spanda_error::SpandaError> {
    let trace = MissionTrace::load(trace_path).map_err(SpandaError::from)?;

    let mut timeline = Vec::new();
    let mut failures = Vec::new();
    let mut safety_events = Vec::new();
    let mut health_events = Vec::new();
    let mut provider_failures = Vec::new();

    for frame in &trace.frames {
        let detail = frame.payload.to_string();
        timeline.push(TimelineEvent {
            sim_time_ms: frame.sim_time_ms,
            event: frame.event.clone(),
            detail: detail.clone(),
        });

        let event_lower = frame.event.to_lowercase();
        if event_lower.contains("fail")
            || frame.payload.get("failed") == Some(&serde_json::json!(true))
        {
            failures.push((frame.sim_time_ms, frame.event.clone(), detail.clone()));
        }
        if event_lower.contains("safety") || event_lower.contains("kill") {
            safety_events.push((frame.sim_time_ms, detail.clone()));
        }
        if event_lower.contains("health") {
            health_events.push((frame.sim_time_ms, detail));
        }
        if frame.event == "provider_call"
            && frame.payload.get("failed") == Some(&serde_json::json!(true))
        {
            let module = frame
                .payload
                .get("module")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            provider_failures.push((frame.sim_time_ms, module.to_string()));
        }
    }

    let root_cause = if let Some((_, event, detail)) = failures.first() {
        format!("{event}: {detail}")
    } else if let Some((_, module)) = provider_failures.first() {
        format!("Provider failure: {module}")
    } else if let Some((_, detail)) = safety_events.first() {
        format!("Safety intervention: {detail}")
    } else if trace.frames.is_empty() {
        "Empty trace — no runtime events recorded".into()
    } else {
        "No explicit failure event; inspect timeline for anomalies".into()
    };

    let mut contributing = Vec::new();
    if !provider_failures.is_empty() {
        contributing.push("Provider call failures detected".into());
    }
    if !safety_events.is_empty() {
        contributing.push("Safety rules triggered during mission".into());
    }
    if !health_events.is_empty() {
        contributing.push("Health status changes observed".into());
    }
    if contributing.is_empty() {
        contributing.push("Review trigger and message sequence in timeline".into());
    }

    let mut actions = Vec::new();
    if !provider_failures.is_empty() {
        actions.push("Verify provider connectivity and credentials".into());
    }
    if !safety_events.is_empty() {
        actions.push("Review safety zone and stop_if thresholds".into());
    }
    actions.push("Replay mission with --deterministic for confirmation".into());

    Ok(RootCauseReport {
        root_cause,
        contributing_factors: contributing,
        timeline,
        recommended_actions: actions,
    })
}
