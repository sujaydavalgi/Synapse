//! Fleet mesh telemetry ingest and aggregated OTLP export.

use crate::mesh::FleetMeshState;
use serde::{Deserialize, Serialize};
use spanda_deploy_http::{http_request, HttpResponse};
use spanda_runtime::fleet_telemetry_runtime::fleet_telemetry_runtime;

/// OTLP ingest payload from a fleet agent or robot runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetTelemetryIngestRequest {
    pub robot_id: String,
    pub otlp_json: String,
}

/// Result of ingesting one robot telemetry shard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetTelemetryIngestResponse {
    pub ok: bool,
    pub robots: u32,
}

/// Handle `POST /v1/fleet/telemetry/ingest` on the mesh coordinator.
pub fn handle_fleet_telemetry_ingest_post(body: &str, state: &mut FleetMeshState) -> HttpResponse {
    let payload: FleetTelemetryIngestRequest = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(_) => {
            return HttpResponse {
                status: 400,
                body: r#"{"ok":false,"error":"invalid fleet telemetry ingest payload"}"#.into(),
            };
        }
    };
    if payload.robot_id.trim().is_empty() || payload.otlp_json.trim().is_empty() {
        return HttpResponse {
            status: 400,
            body: r#"{"ok":false,"error":"robot_id and otlp_json are required"}"#.into(),
        };
    }
    state
        .telemetry_shards
        .insert(payload.robot_id, payload.otlp_json);
    state.telemetry_ingest_total = state.telemetry_ingest_total.saturating_add(1);
    let response = FleetTelemetryIngestResponse {
        ok: true,
        robots: state.telemetry_shards.len() as u32,
    };
    HttpResponse {
        status: 200,
        body: serde_json::to_string(&response).unwrap_or_else(|_| r#"{"ok":false}"#.into()),
    }
}

/// Handle `GET /v1/fleet/telemetry` — merged OTLP/JSON across ingested robots.
pub fn handle_fleet_telemetry_get(state: &FleetMeshState) -> HttpResponse {
    // Delegate merge to the injected fleet telemetry runtime.
    let shards: std::collections::HashMap<String, String> =
        state.telemetry_shards.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    match fleet_telemetry_runtime().merge_fleet_otlp_json(&shards) {
        Ok(body) => HttpResponse { status: 200, body },
        Err(error) => HttpResponse {
            status: 500,
            body: format!(r#"{{"ok":false,"error":"{error}"}}"#),
        },
    }
}

/// Fetch merged fleet OTLP/JSON from a mesh coordinator.
pub fn fetch_fleet_telemetry(mesh_url: &str, token: Option<&str>) -> Result<String, String> {
    let url = if mesh_url.ends_with('/') {
        format!("{mesh_url}v1/fleet/telemetry")
    } else {
        format!("{mesh_url}/v1/fleet/telemetry")
    };
    let response = http_request("GET", &url, None, token)?;
    if (200..300).contains(&response.status) {
        return Ok(response.body);
    }
    Err(format!(
        "fleet telemetry HTTP {}: {}",
        response.status, response.body
    ))
}

/// Ingest a robot OTLP snapshot into the mesh coordinator.
pub fn ingest_fleet_telemetry(
    mesh_url: &str,
    robot_id: &str,
    otlp_json: &str,
    token: Option<&str>,
) -> Result<FleetTelemetryIngestResponse, String> {
    let url = if mesh_url.ends_with('/') {
        format!("{mesh_url}v1/fleet/telemetry/ingest")
    } else {
        format!("{mesh_url}/v1/fleet/telemetry/ingest")
    };
    let payload = FleetTelemetryIngestRequest {
        robot_id: robot_id.to_string(),
        otlp_json: otlp_json.to_string(),
    };
    let body = serde_json::to_string(&payload).map_err(|error| error.to_string())?;
    let response = http_request("POST", &url, Some(&body), token)?;
    if (200..300).contains(&response.status) {
        return serde_json::from_str(&response.body)
            .map_err(|error| format!("invalid fleet telemetry ingest JSON: {error}"));
    }
    Err(format!(
        "fleet telemetry ingest HTTP {}: {}",
        response.status, response.body
    ))
}
