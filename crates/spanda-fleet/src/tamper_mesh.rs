//! Fleet mesh tamper trace ingest and live correlation.

use crate::mesh::FleetMeshState;
use spanda_deploy_http::{FleetTamperIngestRequest, HttpResponse};
use spanda_runtime::fleet_tamper_runtime::fleet_tamper_runtime;

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
        Ok(body) => HttpResponse { status: 200, body },
        Err(error) => HttpResponse {
            status: 400,
            body: format!(r#"{{"ok":false,"error":"{error}"}}"#),
        },
    }
}

/// Correlate ingested tamper shards stored on the mesh coordinator, returning JSON.
pub fn correlate_mesh_tamper_shards(
    fleet_name: &str,
    state: &FleetMeshState,
) -> Result<String, String> {
    // Delegate correlation to the injected fleet tamper runtime via JSON shards.
    if state.tamper_shards.is_empty() {
        return Err("no tamper trace shards ingested".into());
    }
    let shards: std::collections::HashMap<String, String> =
        state.tamper_shards.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    fleet_tamper_runtime().correlate_fleet_tamper_traces_json(fleet_name, &shards)
}

/// Fetch and format a live fleet tamper report from a mesh coordinator.
pub fn fetch_live_fleet_tamper_report(
    mesh_url: &str,
    fleet_name: &str,
    token: Option<&str>,
    json: bool,
) -> Result<String, String> {
    // Fetch the raw JSON report from the mesh coordinator.
    let body = spanda_deploy_http::fetch_fleet_tamper_report(mesh_url, fleet_name, token)?;
    if json {
        return Ok(body);
    }
    Ok(fleet_tamper_runtime().format_fleet_tamper_report_json(&body))
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
        let json = correlate_mesh_tamper_shards("PatrolFleet", &state).expect("correlate");
        assert!(!json.is_empty());
    }
}
