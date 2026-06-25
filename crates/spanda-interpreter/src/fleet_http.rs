//! Optional fleet mesh HTTP relay — disabled for wasm32 builds without TLS.

#[cfg(feature = "fleet-http")]
pub use spanda_deploy_http::{
    ingest_fleet_tamper_trace, relay_continuity_via_mesh, relay_recovery_via_mesh,
    FleetContinuityRequest, FleetRecoveryRequest, FleetTamperIngestRequest,
};

#[cfg(not(feature = "fleet-http"))]
use serde::{Deserialize, Serialize};

/// Recovery command posted to the fleet mesh coordinator.
#[cfg(not(feature = "fleet-http"))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetRecoveryRequest {
    pub action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fleet_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_robot: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<String>,
}

/// Result of broadcasting a recovery command to fleet agents.
#[cfg(not(feature = "fleet-http"))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetRecoveryResponse {
    pub ok: bool,
    pub relayed: u32,
    pub failed: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Takeover command posted to the fleet mesh coordinator.
#[cfg(not(feature = "fleet-http"))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetContinuityRequest {
    pub failed_robot: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub successor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mission: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress_percent: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fleet_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_robot: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<String>,
}

/// Result of broadcasting a takeover command to fleet agents.
#[cfg(not(feature = "fleet-http"))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetContinuityResponse {
    pub ok: bool,
    pub relayed: u32,
    pub failed: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Post a recovery command to a running fleet mesh coordinator.
#[cfg(not(feature = "fleet-http"))]
pub fn relay_recovery_via_mesh(
    _mesh_url: &str,
    _request: &FleetRecoveryRequest,
    _token: Option<&str>,
) -> Result<FleetRecoveryResponse, String> {
    Err("fleet mesh HTTP is disabled in this build".into())
}

/// Post a takeover command to a running fleet mesh coordinator.
#[cfg(not(feature = "fleet-http"))]
pub fn relay_continuity_via_mesh(
    _mesh_url: &str,
    _request: &FleetContinuityRequest,
    _token: Option<&str>,
) -> Result<FleetContinuityResponse, String> {
    Err("fleet mesh HTTP is disabled in this build".into())
}

/// Tamper trace shard posted to the fleet mesh coordinator.
#[cfg(not(feature = "fleet-http"))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetTamperIngestRequest {
    pub robot_id: String,
    pub trace_json: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fleet_name: Option<String>,
}

/// Result of ingesting one robot tamper trace shard.
#[cfg(not(feature = "fleet-http"))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetTamperIngestResponse {
    pub ok: bool,
    pub robots: u32,
}

/// Ingest a tamper trace shard into the fleet mesh coordinator.
#[cfg(not(feature = "fleet-http"))]
pub fn ingest_fleet_tamper_trace(
    _mesh_url: &str,
    _request: &FleetTamperIngestRequest,
    _token: Option<&str>,
) -> Result<FleetTamperIngestResponse, String> {
    Err("fleet mesh HTTP is disabled in this build".into())
}
