//! Remote OTA rollout via HTTP deploy agents.

use crate::bundle::DeployArtifactBundle;
use crate::service::{deploy_target_key, plan_rollout};
use crate::types::{
    CertificationProofSummary, DeployAssignment, DeployPlan, RolloutOptions, RolloutResult,
    RolloutStep, RolloutStepStatus, RolloutStrategy,
};
use serde::{Deserialize, Serialize};
use spanda_deploy_http::{http_request, parse_http_url, HttpResponse};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployAgentEntry {
    pub target: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployAgentRegistry {
    pub agents: Vec<DeployAgentEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentStatusResponse {
    pub ok: bool,
    pub target: String,
    pub current_version: String,
    pub previous_version: Option<String>,
    pub healthy: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub program_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hardware_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firmware_version: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub robot_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attestation_contract: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attestation_verified: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boot_state: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RolloutRequest {
    target: String,
    version: String,
    program: Option<String>,
    program_hash: Option<String>,
    #[serde(default)]
    assignments: Vec<DeployAssignment>,
    #[serde(default)]
    certifications: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    certification_proof: Option<CertificationProofSummary>,
    artifact_signature: Option<String>,
    artifact_public_key: Option<String>,
    #[serde(default)]
    packages: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hardware_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    firmware_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentRolloutResponse {
    pub ok: bool,
    pub target: String,
    pub version: String,
    pub previous_version: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

pub fn default_agents_path() -> PathBuf {
    // Description:
    //     Default agents path.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: PathBuf
    //         Return value from `default_agents_path`.
    //
    // Example:

    //     let result = spanda_ota::remote::default_agents_path();

    PathBuf::from(".spanda/deploy-agents.json")
}

/// Deploy agent registry path (`SPANDA_DEPLOY_AGENTS` override).
pub fn agents_registry_path() -> PathBuf {
    std::env::var("SPANDA_DEPLOY_AGENTS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_agents_path())
}

pub fn load_agent_registry(path: &Path) -> DeployAgentRegistry {
    // Description:
    //     Load agent registry.
    //
    // Inputs:
    //     path: &Path
    //         Caller-supplied path.
    //
    // Outputs:
    //     result: DeployAgentRegistry
    //         Return value from `load_agent_registry`.
    //
    // Example:

    //     let result = spanda_ota::remote::load_agent_registry(path);

    if !path.exists() {
        return DeployAgentRegistry::default();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save_agent_registry(path: &Path, registry: &DeployAgentRegistry) -> Result<(), String> {
    // Description:
    //     Save agent registry.
    //
    // Inputs:
    //     path: &Path
    //         Caller-supplied path.
    //     registry: &DeployAgentRegistry
    //         Caller-supplied registry.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `save_agent_registry`.
    //
    // Example:

    //     let result = spanda_ota::remote::save_agent_registry(path, registry);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(registry).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}

pub fn register_agent(
    registry: &mut DeployAgentRegistry,
    target: String,
    url: String,
    token: Option<String>,
) -> Result<(), String> {
    // Description:

    //     Register agent.

    //

    // Inputs:

    //     registry: &mut DeployAgentRegistry

    //         Caller-supplied registry.

    //     arge: String

    //         Caller-supplied arge.

    //     url: String

    //         Caller-supplied url.

    //     token: Option<String>

    //         Caller-supplied token.

    //

    // Outputs:

    //     result: Result<(), String>

    //         Return value from `register_agent`.

    //

    // Example:

    //     let result = spanda_ota::remote::register_agent(registry, arge, rl, oken);
    parse_http_url(&url)?;
    registry.agents.retain(|entry| entry.target != target);
    registry
        .agents
        .push(DeployAgentEntry { target, url, token });
    registry.agents.sort_by(|a, b| a.target.cmp(&b.target));
    Ok(())
}

pub fn lookup_agent<'a>(
    registry: &'a DeployAgentRegistry,
    target: &str,
) -> Option<&'a DeployAgentEntry> {
    registry.agents.iter().find(|entry| entry.target == target)
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

    //     let result = spanda_ota::remote::agent_endpoint(base_url, path);

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

    //     let result = spanda_ota::remote::decode_response(response);

    if response.status >= 400 {
        return Err(format!("agent HTTP {}: {}", response.status, response.body));
    }
    serde_json::from_str(&response.body).map_err(|e| format!("invalid agent JSON: {e}"))
}

pub fn agent_health(entry: &DeployAgentEntry) -> Result<bool, String> {
    // Description:
    //     Agent health.
    //
    // Inputs:
    //     entry: &DeployAgentEntry
    //         Caller-supplied entry.
    //
    // Outputs:
    //     result: Result<bool, String>
    //         Return value from `agent_health`.
    //
    // Example:

    //     let result = spanda_ota::remote::agent_health(entry);

    let url = agent_endpoint(&entry.url, "/v1/health")?;
    let response = http_request("GET", &url, None, entry.token.as_deref())?;
    let body: serde_json::Value = decode_response(response)?;
    let ok = body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    if ok {
        // Record heartbeat via the injected device telemetry sink.
        let sink = spanda_runtime::device_telemetry_sink::device_telemetry_sink();
        if sink.persist_enabled() {
            let device_id = entry.target.clone();
            let _ = sink.record_device_heartbeat(
                &device_id,
                sink.wall_timestamp_ms(),
                None,
                Some("deploy-agent"),
                5000.0,
            );
        }
    }
    Ok(ok)
}

/// Fetch live readiness report from a deploy agent (`GET /v1/readiness`).
pub fn agent_readiness(
    entry: &DeployAgentEntry,
    runtime: bool,
    inject_health_faults: bool,
) -> Result<serde_json::Value, String> {
    // Description:
    //     Agent readiness.
    //
    // Inputs:
    //     entry: &DeployAgentEntry
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

    //     let result = spanda_ota::remote::agent_readiness(entry, runtime, inject_health_faults);

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

pub fn agent_upload_program(entry: &DeployAgentEntry, program: &str) -> Result<(), String> {
    // Description:
    //     Agent upload program.
    //
    // Inputs:
    //     entry: &DeployAgentEntry
    //         Caller-supplied entry.
    //     progra: &str
    //         Caller-supplied progra.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `agent_upload_program`.
    //
    // Example:

    //     let result = spanda_ota::remote::agent_upload_program(entry, progra);

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

pub fn agent_status(entry: &DeployAgentEntry) -> Result<AgentStatusResponse, String> {
    // Description:
    //     Agent status.
    //
    // Inputs:
    //     entry: &DeployAgentEntry
    //         Caller-supplied entry.
    //
    // Outputs:
    //     result: Result<AgentStatusResponse, String>
    //         Return value from `agent_status`.
    //
    // Example:

    //     let result = spanda_ota::remote::agent_status(entry);

    let url = agent_endpoint(&entry.url, "/v1/status")?;
    let response = http_request("GET", &url, None, entry.token.as_deref())?;
    decode_response(response)
}

pub fn agent_rollout(
    entry: &DeployAgentEntry,
    bundle: &DeployArtifactBundle,
    certification_proof: Option<&CertificationProofSummary>,
) -> Result<AgentRolloutResponse, String> {
    // Description:
    //     Agent rollout.
    //
    // Inputs:
    //     entry: &DeployAgentEntry
    //         Caller-supplied entry.
    //     bundle: &DeployArtifactBundle
    //         Caller-supplied bundle.
    //     certification_proof: Option<&CertificationProofSummary>
    //         Caller-supplied certification proof.
    //
    // Outputs:
    //     result: Result<AgentRolloutResponse, String>
    //         Return value from `agent_rollout`.
    //
    // Example:

    //     let result = spanda_ota::remote::agent_rollout(entry, bundle, certification_proof);

    let url = agent_endpoint(&entry.url, "/v1/rollout")?;
    let payload = serde_json::to_string(&RolloutRequest {
        target: entry.target.clone(),
        version: bundle.version.clone(),
        program: Some(bundle.program.clone()),
        program_hash: bundle.program_hash.clone(),
        assignments: bundle.assignments.clone(),
        certifications: bundle.certifications.clone(),
        certification_proof: certification_proof.cloned(),
        artifact_signature: bundle.signature.clone(),
        artifact_public_key: bundle.public_key.clone(),
        packages: Vec::new(),
        hardware_profile: None,
        firmware_version: None,
    })
    .map_err(|e| e.to_string())?;
    let response = http_request("POST", &url, Some(&payload), entry.token.as_deref())?;
    decode_response(response)
}

pub fn agent_rollback(entry: &DeployAgentEntry) -> Result<AgentRolloutResponse, String> {
    // Description:
    //     Agent rollback.
    //
    // Inputs:
    //     entry: &DeployAgentEntry
    //         Caller-supplied entry.
    //
    // Outputs:
    //     result: Result<AgentRolloutResponse, String>
    //         Return value from `agent_rollback`.
    //
    // Example:

    //     let result = spanda_ota::remote::agent_rollback(entry);

    let url = agent_endpoint(&entry.url, "/v1/rollback")?;
    let payload = serde_json::to_string(&serde_json::json!({
        "target": entry.target,
    }))
    .map_err(|e| e.to_string())?;
    let response = http_request("POST", &url, Some(&payload), entry.token.as_deref())?;
    decode_response(response)
}

fn readiness_mission_ready(body: &serde_json::Value) -> bool {
    body.get("mission_ready")
        .and_then(|value| value.as_bool())
        .or_else(|| {
            body.get("readiness")
                .and_then(|readiness| readiness.get("mission_ready"))
                .and_then(|value| value.as_bool())
        })
        .unwrap_or(false)
}

fn rollout_step_after_readiness_gate(
    agent: &DeployAgentEntry,
    options: &RolloutOptions,
    step: &RolloutStep,
) -> RolloutStep {
    if !options.rollback_on_readiness_fail {
        return RolloutStep {
            status: RolloutStepStatus::Deployed,
            ..step.clone()
        };
    }
    let readiness_ok = match agent_readiness(
        agent,
        options.readiness_runtime,
        options.readiness_inject_faults,
    ) {
        Ok(body) => readiness_mission_ready(&body),
        Err(_) => false,
    };
    if readiness_ok {
        return RolloutStep {
            status: RolloutStepStatus::Deployed,
            ..step.clone()
        };
    }
    let rolled_back = agent_rollback(agent)
        .map(|response| response.ok)
        .unwrap_or(false);
    RolloutStep {
        status: if rolled_back {
            RolloutStepStatus::RolledBack
        } else {
            RolloutStepStatus::Failed
        },
        ..step.clone()
    }
}

pub fn execute_remote_rollout(
    plan: &DeployPlan,
    options: &RolloutOptions,
    registry: &DeployAgentRegistry,
    bundle: &DeployArtifactBundle,
) -> RolloutResult {
    // Description:

    //     Execute remote rollout.

    //

    // Inputs:

    //     plan: &DeployPlan

    //         Caller-supplied plan.

    //     options: &RolloutOptions

    //         Caller-supplied options.

    //     registry: &DeployAgentRegistry

    //         Caller-supplied registry.

    //     bundle: &DeployArtifactBundle

    //         Caller-supplied bundle.

    //

    // Outputs:

    //     result: RolloutResult

    //         Return value from `execute_remote_rollout`.

    //

    // Example:

    //     let result = spanda_ota::remote::execute_remote_rollout(plan, options, registry, bundle);
    let local = plan_rollout(plan, options);
    if options.dry_run {
        return local;
    }

    let mut steps = Vec::new();
    let mut success = true;
    for step in &local.steps {
        if step.status == RolloutStepStatus::Skipped {
            steps.push(step.clone());
            continue;
        }
        let key = deploy_target_key(&step.robot_name, &step.hardware);
        let Some(agent) = lookup_agent(registry, &key) else {
            steps.push(RolloutStep {
                status: RolloutStepStatus::Failed,
                ..step.clone()
            });
            success = false;
            continue;
        };
        match agent_rollout(agent, bundle, plan.certification_proof.as_ref()) {
            Ok(resp) if resp.ok => {
                let gated = rollout_step_after_readiness_gate(agent, options, step);
                if gated.status != RolloutStepStatus::Deployed {
                    success = false;
                }
                steps.push(gated);
            }
            Ok(resp) => {
                steps.push(RolloutStep {
                    status: RolloutStepStatus::Failed,
                    version: resp.version,
                    ..step.clone()
                });
                success = false;
            }
            Err(_) => {
                steps.push(RolloutStep {
                    status: RolloutStepStatus::Failed,
                    ..step.clone()
                });
                success = false;
            }
        }
    }

    RolloutResult {
        strategy: options.strategy,
        version: options.version.clone(),
        dry_run: false,
        steps,
        success,
    }
}

pub fn execute_remote_rollback(plan: &DeployPlan, registry: &DeployAgentRegistry) -> RolloutResult {
    // Description:
    //     Execute remote rollback.
    //
    // Inputs:
    //     plan: &DeployPlan
    //         Caller-supplied plan.
    //     registry: &DeployAgentRegistry
    //         Caller-supplied registry.
    //
    // Outputs:
    //     result: RolloutResult
    //         Return value from `execute_remote_rollback`.
    //
    // Example:

    //     let result = spanda_ota::remote::execute_remote_rollback(plan, registry);

    let mut steps = Vec::new();
    let mut success = false;
    for assignment in &plan.assignments {
        let key = deploy_target_key(&assignment.robot_name, &assignment.hardware);
        let step = RolloutStep {
            robot_name: assignment.robot_name.clone(),
            hardware: assignment.hardware.clone(),
            status: RolloutStepStatus::Skipped,
            version: "unknown".into(),
            phase_percent: None,
        };
        let Some(agent) = lookup_agent(registry, &key) else {
            steps.push(step);
            continue;
        };
        match agent_rollback(agent) {
            Ok(resp) if resp.ok => {
                success = true;
                steps.push(RolloutStep {
                    status: RolloutStepStatus::RolledBack,
                    version: resp.version,
                    ..step
                });
            }
            Ok(resp) => steps.push(RolloutStep {
                status: RolloutStepStatus::Failed,
                version: resp.version,
                ..step
            }),
            Err(_) => steps.push(RolloutStep {
                status: RolloutStepStatus::Failed,
                ..step
            }),
        }
    }
    RolloutResult {
        strategy: RolloutStrategy::All,
        version: "rollback".into(),
        dry_run: false,
        steps,
        success,
    }
}

pub fn missing_remote_targets(plan: &DeployPlan, registry: &DeployAgentRegistry) -> Vec<String> {
    // Description:
    //     Missing remote targets.
    //
    // Inputs:
    //     plan: &DeployPlan
    //         Caller-supplied plan.
    //     registry: &DeployAgentRegistry
    //         Caller-supplied registry.
    //
    // Outputs:
    //     result: Vec<String>
    //         Return value from `missing_remote_targets`.
    //
    // Example:

    //     let result = spanda_ota::remote::missing_remote_targets(plan, registry);

    let mut missing = Vec::new();
    for assignment in &plan.assignments {
        let key = deploy_target_key(&assignment.robot_name, &assignment.hardware);
        if lookup_agent(registry, &key).is_none() {
            missing.push(key);
        }
    }
    missing
}

pub fn registry_by_target(registry: &DeployAgentRegistry) -> HashMap<String, DeployAgentEntry> {
    // Description:
    //     Registry by target.
    //
    // Inputs:
    //     registry: &DeployAgentRegistry
    //         Caller-supplied registry.
    //
    // Outputs:
    //     result: HashMap<String, DeployAgentEntry>
    //         Return value from `registry_by_target`.
    //
    // Example:

    //     let result = spanda_ota::remote::registry_by_target(registry);

    registry
        .agents
        .iter()
        .cloned()
        .map(|entry| (entry.target.clone(), entry))
        .collect()
}
