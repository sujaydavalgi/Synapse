//! Fleet mesh tamper trace ingest client.

use crate::{http_request, HttpResponse};
use serde::{Deserialize, Serialize};

/// Tamper trace shard posted to a fleet mesh coordinator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetTamperIngestRequest {
    pub robot_id: String,
    pub trace_json: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fleet_name: Option<String>,
}

/// Result of ingesting one robot tamper trace shard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetTamperIngestResponse {
    pub ok: bool,
    pub robots: u32,
}

fn mesh_tamper_url(mesh_url: &str, suffix: &str, query: &str) -> String {
    let path = if suffix.is_empty() {
        "v1/fleet/tamper".to_string()
    } else {
        format!("v1/fleet/tamper/{suffix}")
    };
    let base = if mesh_url.ends_with('/') {
        format!("{mesh_url}{path}")
    } else {
        format!("{mesh_url}/{path}")
    };
    if query.is_empty() {
        base
    } else {
        format!("{base}?{query}")
    }
}

/// Ingest a robot tamper trace snapshot into the mesh coordinator.
pub fn ingest_fleet_tamper_trace(
    mesh_url: &str,
    request: &FleetTamperIngestRequest,
    token: Option<&str>,
) -> Result<FleetTamperIngestResponse, String> {
    // POST a tamper trace shard to the fleet mesh coordinator.
    //
    // Parameters:
    // - `mesh_url` — mesh base URL
    // - `request` — robot id and trace JSON payload
    // - `token` — optional bearer token
    //
    // Returns:
    // Ingest acknowledgement or HTTP error message.
    //
    // Options:
    // None.
    //
    // Example:
    // ingest_fleet_tamper_trace("http://127.0.0.1:8765", &request, None)?;

    let body = serde_json::to_string(request).map_err(|error| error.to_string())?;
    let response = http_request(
        "POST",
        &mesh_tamper_url(mesh_url, "ingest", ""),
        Some(&body),
        token,
    )?;
    parse_ingest_response(response)
}

/// Fetch a correlated fleet tamper report from the mesh coordinator.
pub fn fetch_fleet_tamper_report(
    mesh_url: &str,
    fleet_name: &str,
    token: Option<&str>,
) -> Result<String, String> {
    // GET correlated fleet tamper JSON from the mesh coordinator.
    //
    // Parameters:
    // - `mesh_url` — mesh base URL
    // - `fleet_name` — fleet label for correlation
    // - `token` — optional bearer token
    //
    // Returns:
    // Fleet tamper report JSON body.
    //
    // Options:
    // None.
    //
    // Example:
    // let json = fetch_fleet_tamper_report("http://127.0.0.1:8765", "PatrolFleet", None)?;

    let query = format!("fleet={fleet_name}");
    let response = http_request(
        "GET",
        &mesh_tamper_url(mesh_url, "", &query),
        None,
        token,
    )?;
    if (200..300).contains(&response.status) {
        return Ok(response.body);
    }
    Err(format!(
        "fleet tamper HTTP {}: {}",
        response.status, response.body
    ))
}

fn parse_ingest_response(response: HttpResponse) -> Result<FleetTamperIngestResponse, String> {
    if (200..300).contains(&response.status) {
        return serde_json::from_str(&response.body)
            .map_err(|error| format!("invalid fleet tamper ingest JSON: {error}"));
    }
    Err(format!(
        "fleet tamper ingest HTTP {}: {}",
        response.status, response.body
    ))
}
