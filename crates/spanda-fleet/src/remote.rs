//! Remote fleet peer relay via HTTP fleet agents.
//!
use crate::PeerDelivery;
use serde::{Deserialize, Serialize};
use spanda_deploy_http::{http_request, parse_http_url, HttpResponse};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetAgentEntry {
    pub robot_name: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetAgentRegistry {
    pub agents: Vec<FleetAgentEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerRelayRequest {
    pub from_robot: String,
    pub to_robot: String,
    pub topic: String,
    pub step: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerRelayResponse {
    pub ok: bool,
    pub to_robot: String,
    pub topic: String,
    pub step: String,
    #[serde(default)]
    pub error: Option<String>,
}

pub fn default_fleet_agents_path() -> PathBuf {
    // Description:
    //     Default fleet agents path.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: PathBuf
    //         Return value from `default_fleet_agents_path`.
    //
    // Example:

    //     let result = spanda_fleet::remote::default_fleet_agents_path();

    PathBuf::from(".spanda/fleet-agents.json")
}

pub fn load_fleet_agent_registry(path: &Path) -> FleetAgentRegistry {
    // Description:
    //     Load fleet agent registry.
    //
    // Inputs:
    //     path: &Path
    //         Caller-supplied path.
    //
    // Outputs:
    //     result: FleetAgentRegistry
    //         Return value from `load_fleet_agent_registry`.
    //
    // Example:

    //     let result = spanda_fleet::remote::load_fleet_agent_registry(path);

    if !path.exists() {
        return FleetAgentRegistry::default();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save_fleet_agent_registry(path: &Path, registry: &FleetAgentRegistry) -> Result<(), String> {
    // Description:
    //     Save fleet agent registry.
    //
    // Inputs:
    //     path: &Path
    //         Caller-supplied path.
    //     registry: &FleetAgentRegistry
    //         Caller-supplied registry.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `save_fleet_agent_registry`.
    //
    // Example:

    //     let result = spanda_fleet::remote::save_fleet_agent_registry(path, registry);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(registry).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}

pub fn register_fleet_agent(
    registry: &mut FleetAgentRegistry,
    robot_name: String,
    url: String,
    token: Option<String>,
) -> Result<(), String> {
    // Description:
    //     Register fleet agent.
    //
    // Inputs:
    //     registry: &mut FleetAgentRegistry
    //         Caller-supplied registry.
    //     robot_name: String
    //         Caller-supplied robot name.
    //     url: String
    //         Caller-supplied url.
    //     token: Option<String>
    //         Caller-supplied token.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `register_fleet_agent`.
    //
    // Example:

    //     let result = spanda_fleet::remote::register_fleet_agent(registry, robot_name, rl, oken);

    parse_http_url(&url)?;
    registry
        .agents
        .retain(|entry| entry.robot_name != robot_name);
    registry.agents.push(FleetAgentEntry {
        robot_name,
        url,
        token,
    });
    registry
        .agents
        .sort_by(|a, b| a.robot_name.cmp(&b.robot_name));
    Ok(())
}

pub fn lookup_fleet_agent<'a>(
    registry: &'a FleetAgentRegistry,
    robot_name: &str,
) -> Option<&'a FleetAgentEntry> {
    registry
        .agents
        .iter()
        .find(|entry| entry.robot_name == robot_name)
}

fn agent_endpoint(base_url: &str, path: &str) -> Result<String, String> {
    // Description:
    //     Agent endpoint.
    //
    // Inputs:
    //     base_url: &str
    //         Caller-supplied base url.
    //     path: &str
    //         Caller-supplied path.
    //
    // Outputs:
    //     result: Result<String, String>
    //         Return value from `agent_endpoint`.
    //
    // Example:

    //     let result = spanda_fleet::remote::agent_endpoint(base_url, path);

    let parsed = parse_http_url(base_url)?;
    Ok(format!(
        "{}://{}:{}{}",
        parsed.scheme, parsed.host, parsed.port, path
    ))
}

fn decode_response<T: for<'de> Deserialize<'de>>(response: HttpResponse) -> Result<T, String> {
    // Description:
    //     Decode response.
    //
    // Inputs:
    //     response: HttpResponse
    //         Caller-supplied response.
    //
    // Outputs:
    //     result: Result<T, String>
    //         Return value from `decode_response`.
    //
    // Example:

    //     let result = spanda_fleet::remote::decode_response(response);

    if response.status >= 400 {
        return Err(format!(
            "fleet agent HTTP {}: {}",
            response.status, response.body
        ));
    }
    serde_json::from_str(&response.body).map_err(|e| format!("invalid fleet agent JSON: {e}"))
}

pub fn agent_health(entry: &FleetAgentEntry) -> Result<bool, String> {
    // Description:
    //     Agent health.
    //
    // Inputs:
    //     entry: &FleetAgentEntry
    //         Caller-supplied entry.
    //
    // Outputs:
    //     result: Result<bool, String>
    //         Return value from `agent_health`.
    //
    // Example:

    //     let result = spanda_fleet::remote::agent_health(entry);

    let url = agent_endpoint(&entry.url, "/v1/health")?;
    let response = http_request("GET", &url, None, entry.token.as_deref())?;
    let body: serde_json::Value = decode_response(response)?;
    let ok = body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    if ok {
        // Record heartbeat via the injected device telemetry sink.
        let sink = spanda_runtime::device_telemetry_sink::device_telemetry_sink();
        if sink.persist_enabled() {
            let _ = sink.record_device_heartbeat(
                &entry.robot_name,
                sink.wall_timestamp_ms(),
                Some(entry.robot_name.as_str()),
                Some("fleet-agent"),
                5000.0,
            );
        }
    }
    Ok(ok)
}

/// Fetch live readiness report from a fleet agent (`GET /v1/readiness`).
pub fn agent_readiness(
    entry: &FleetAgentEntry,
    runtime: bool,
    inject_health_faults: bool,
) -> Result<serde_json::Value, String> {
    // Description:
    //     Agent readiness.
    //
    // Inputs:
    //     entry: &FleetAgentEntry
    //         Caller-supplied entry.
    //     runtime: bool
    //         Caller-supplied runtime.
    //     inject_health_faults: bool
    //         Caller-supplied inject health faults.
    //
    // Outputs:
    //     result: Result<serde_json::Value, String>
    //         Return value from `agent_readiness`.
    //
    // Example:

    //     let result = spanda_fleet::remote::agent_readiness(entry, runtime, inject_health_faults);

    let mut url = agent_endpoint(&entry.url, "/v1/readiness")?;
    let mut query = Vec::new();
    if runtime {
        query.push("runtime=true");
    }
    if inject_health_faults {
        query.push("inject_health_faults=true");
    }
    if !query.is_empty() {
        url.push('?');
        url.push_str(&query.join("&"));
    }
    let response = http_request("GET", &url, None, entry.token.as_deref())?;
    decode_response(response)
}

/// Push program source to a fleet agent (`POST /v1/program`).
pub fn agent_upload_program(entry: &FleetAgentEntry, program: &str) -> Result<(), String> {
    // Description:
    //     Agent upload program.
    //
    // Inputs:
    //     entry: &FleetAgentEntry
    //         Caller-supplied entry.
    //     progra: &str
    //         Caller-supplied progra.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `agent_upload_program`.
    //
    // Example:

    //     let result = spanda_fleet::remote::agent_upload_program(entry, progra);

    let url = agent_endpoint(&entry.url, "/v1/program")?;
    let payload = serde_json::json!({ "program": program }).to_string();
    let response = http_request("POST", &url, Some(&payload), entry.token.as_deref())?;
    let body: serde_json::Value = decode_response(response)?;
    if body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        Ok(())
    } else {
        Err(body
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("program upload failed")
            .to_string())
    }
}

pub fn relay_peer_delivery(
    entry: &FleetAgentEntry,
    delivery: &PeerDelivery,
) -> Result<PeerRelayResponse, String> {
    // Description:
    //     Relay peer delivery.
    //
    // Inputs:
    //     entry: &FleetAgentEntry
    //         Caller-supplied entry.
    //     delivery: &PeerDelivery
    //         Caller-supplied delivery.
    //
    // Outputs:
    //     result: Result<PeerRelayResponse, String>
    //         Return value from `relay_peer_delivery`.
    //
    // Example:

    //     let result = spanda_fleet::remote::relay_peer_delivery(entry, delivery);

    let url = agent_endpoint(&entry.url, "/v1/peer")?;
    let payload = serde_json::to_string(&PeerRelayRequest {
        from_robot: delivery.from_robot.clone(),
        to_robot: delivery.to_robot.clone(),
        topic: delivery.topic.clone(),
        step: delivery.step.clone(),
    })
    .map_err(|e| e.to_string())?;
    let response = http_request("POST", &url, Some(&payload), entry.token.as_deref())?;
    decode_response(response)
}

pub fn relay_peer_deliveries(
    deliveries: &[PeerDelivery],
    registry: &FleetAgentRegistry,
) -> (u32, u32) {
    // Description:

    //     Relay peer deliveries.

    //

    // Inputs:

    //     deliveries: &[PeerDelivery]

    //         Caller-supplied deliveries.

    //     registry: &FleetAgentRegistry

    //         Caller-supplied registry.

    //

    // Outputs:

    //     result: (u32, u32)

    //         Return value from `relay_peer_deliveries`.

    //

    // Example:

    //     let result = spanda_fleet::remote::relay_peer_deliveries(deliveries, registry);
    let mut relayed = 0u32;
    let mut failed = 0u32;
    for delivery in deliveries {
        let Some(agent) = lookup_fleet_agent(registry, &delivery.to_robot) else {
            failed += 1;
            continue;
        };
        match relay_peer_delivery(agent, delivery) {
            Ok(resp) if resp.ok => relayed += 1,
            _ => failed += 1,
        }
    }
    (relayed, failed)
}

pub fn registry_by_robot(registry: &FleetAgentRegistry) -> HashMap<String, FleetAgentEntry> {
    // Description:
    //     Registry by robot.
    //
    // Inputs:
    //     registry: &FleetAgentRegistry
    //         Caller-supplied registry.
    //
    // Outputs:
    //     result: HashMap<String, FleetAgentEntry>
    //         Return value from `registry_by_robot`.
    //
    // Example:

    //     let result = spanda_fleet::remote::registry_by_robot(registry);

    registry
        .agents
        .iter()
        .cloned()
        .map(|entry| (entry.robot_name.clone(), entry))
        .collect()
}

/// Fleet agent `/v1/status` payload for drift and operations checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetAgentStatusResponse {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub robot_name: Option<String>,
    #[serde(default)]
    pub healthy: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub program_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hardware_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firmware_version: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<String>,
}

pub fn fleet_agent_status(entry: &FleetAgentEntry) -> Result<FleetAgentStatusResponse, String> {
    // Fetch live status from a fleet peer agent.
    //
    // Parameters:
    // - `entry` — registered fleet agent connection
    //
    // Returns:
    // Parsed `/v1/status` JSON or transport error text.
    //
    // Options:
    // None.
    //
    // Example:
    // let status = fleet_agent_status(&entry)?;

    let url = agent_endpoint(&entry.url, "/v1/status")?;
    let response = http_request("GET", &url, None, entry.token.as_deref())?;
    decode_fleet_status(response)
}

fn decode_fleet_status(response: HttpResponse) -> Result<FleetAgentStatusResponse, String> {
    if response.status >= 400 {
        return Err(format!(
            "fleet agent HTTP {}: {}",
            response.status, response.body
        ));
    }
    serde_json::from_str(&response.body).map_err(|e| e.to_string())
}
