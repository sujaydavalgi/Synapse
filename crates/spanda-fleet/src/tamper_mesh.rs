//! Fleet mesh tamper trace ingest and live correlation.

use crate::mesh::FleetMeshState;
use spanda_deploy_http::{FleetTamperIngestRequest, HttpResponse};
use spanda_tamper::{
    correlate_fleet_tamper_traces, format_fleet_tamper_report, FleetTamperReport, MissionTrace,
    TamperFormat,
};

/// Handle `POST /v1/fleet/tamper/ingest` on the mesh coordinator.
pub fn handle_fleet_tamper_ingest_post(body: &str, state: &mut FleetMeshState) -> HttpResponse {
    let payload: FleetTamperIngestRequest = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(_) => {
            return HttpResponse {
                status: 400,
                body: r#"{"ok":false,"error":"invalid fleet tamper ingest payload"}"#.into(),
            };
        }
    };
    if payload.robot_id.trim().is_empty() || payload.trace_json.trim().is_empty() {
        return HttpResponse {
            status: 400,
            body: r#"{"ok":false,"error":"robot_id and trace_json are required"}"#.into(),
        };
    }
    state
        .tamper_shards
        .insert(payload.robot_id, payload.trace_json);
    if let Some(fleet_name) = payload.fleet_name {
        state.tamper_fleet_name = fleet_name;
    }
    state.tamper_ingest_total = state.tamper_ingest_total.saturating_add(1);
    let response = spanda_deploy_http::FleetTamperIngestResponse {
        ok: true,
        robots: state.tamper_shards.len() as u32,
    };
    HttpResponse {
        status: 200,
        body: serde_json::to_string(&response).unwrap_or_else(|_| r#"{"ok":false}"#.into()),
    }
}

/// Handle `GET /v1/fleet/tamper` — correlate ingested member traces.
pub fn handle_fleet_tamper_get(path: &str, state: &FleetMeshState) -> HttpResponse {
    let fleet_name = parse_fleet_query(path).unwrap_or_else(|| state.tamper_fleet_name.clone());
    match correlate_mesh_tamper_shards(&fleet_name, state) {
        Ok(report) => HttpResponse {
            status: 200,
            body: serde_json::to_string(&report).unwrap_or_else(|_| "{}".into()),
        },
        Err(error) => HttpResponse {
            status: 400,
            body: format!(r#"{{"ok":false,"error":"{error}"}}"#),
        },
    }
}

/// Correlate ingested tamper shards stored on the mesh coordinator.
pub fn correlate_mesh_tamper_shards(
    fleet_name: &str,
    state: &FleetMeshState,
) -> Result<FleetTamperReport, String> {
    if state.tamper_shards.is_empty() {
        return Err("no tamper trace shards ingested".into());
    }
    let mut traces = Vec::new();
    for (robot_id, trace_json) in &state.tamper_shards {
        let trace: MissionTrace =
            serde_json::from_str(trace_json).map_err(|error| format!("parse {robot_id}: {error}"))?;
        let label = format!("{robot_id}.trace");
        traces.push((robot_id.clone(), trace, label));
    }
    Ok(correlate_fleet_tamper_traces(fleet_name, &traces))
}

/// Fetch and format a live fleet tamper report from a mesh coordinator.
pub fn fetch_live_fleet_tamper_report(
    mesh_url: &str,
    fleet_name: &str,
    token: Option<&str>,
    json: bool,
) -> Result<String, String> {
    let body = spanda_deploy_http::fetch_fleet_tamper_report(mesh_url, fleet_name, token)?;
    if json {
        return Ok(body);
    }
    let report: FleetTamperReport =
        serde_json::from_str(&body).map_err(|error| format!("invalid fleet tamper JSON: {error}"))?;
    Ok(format_fleet_tamper_report(
        &report,
        TamperFormat::Text,
    ))
}

fn parse_fleet_query(path: &str) -> Option<String> {
    let (_, query) = path.split_once('?')?;
    for pair in query.split('&') {
        if let Some(value) = pair.strip_prefix("fleet=") {
            return Some(value.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ingests_and_correlates_tamper_shards() {
        let mut state = FleetMeshState::default();
        let trace = r#"{
            "version": 1,
            "source": "rover.sd",
            "deterministic": true,
            "frames": [{
                "sim_time_ms": 120.0,
                "event": "security_audit",
                "payload": {"kind": "agent_capability_denied", "agent": "Intruder"}
            }]
        }"#;
        let ingest = handle_fleet_tamper_ingest_post(
            &serde_json::to_string(&FleetTamperIngestRequest {
                robot_id: "RoverAlpha".into(),
                trace_json: trace.into(),
                fleet_name: Some("PatrolFleet".into()),
            })
            .unwrap(),
            &mut state,
        );
        assert_eq!(ingest.status, 200);
        let report = correlate_mesh_tamper_shards("PatrolFleet", &state).expect("correlate");
        assert_eq!(report.fleet, "PatrolFleet");
        assert_eq!(report.members.len(), 1);
    }
}
