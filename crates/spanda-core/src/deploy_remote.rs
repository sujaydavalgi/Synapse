//! Remote OTA rollout via HTTP deploy agents.

use crate::deploy_http::{http_request, parse_http_url, HttpResponse};
use crate::deploy_service::{
    deploy_target_key, DeployPlan, RolloutOptions, RolloutResult, RolloutStep, RolloutStepStatus,
    RolloutStrategy,
};
use serde::{Deserialize, Serialize};
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RolloutRequest {
    target: String,
    version: String,
    program: Option<String>,
    program_hash: Option<String>,
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
    PathBuf::from(".spanda/deploy-agents.json")
}

pub fn load_agent_registry(path: &Path) -> DeployAgentRegistry {
    if !path.exists() {
        return DeployAgentRegistry::default();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save_agent_registry(path: &Path, registry: &DeployAgentRegistry) -> Result<(), String> {
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
    // Validate URL shape before recording the agent endpoint.
    parse_http_url(&url)?;
    registry.agents.retain(|entry| entry.target != target);
    registry.agents.push(DeployAgentEntry { target, url, token });
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
    let parsed = parse_http_url(base_url)?;
    Ok(format!(
        "{}://{}:{}{}",
        parsed.scheme, parsed.host, parsed.port, path
    ))
}

fn decode_response<T: for<'de> Deserialize<'de>>(response: HttpResponse) -> Result<T, String> {
    if response.status >= 400 {
        return Err(format!("agent HTTP {}: {}", response.status, response.body));
    }
    serde_json::from_str(&response.body).map_err(|e| format!("invalid agent JSON: {e}"))
}

pub fn agent_health(entry: &DeployAgentEntry) -> Result<bool, String> {
    let url = agent_endpoint(&entry.url, "/v1/health")?;
    let response = http_request("GET", &url, None, entry.token.as_deref())?;
    let body: serde_json::Value = decode_response(response)?;
    Ok(body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false))
}

pub fn agent_status(entry: &DeployAgentEntry) -> Result<AgentStatusResponse, String> {
    let url = agent_endpoint(&entry.url, "/v1/status")?;
    let response = http_request("GET", &url, None, entry.token.as_deref())?;
    decode_response(response)
}

pub fn agent_rollout(
    entry: &DeployAgentEntry,
    version: &str,
    program: Option<&str>,
    program_hash: Option<&str>,
) -> Result<AgentRolloutResponse, String> {
    let url = agent_endpoint(&entry.url, "/v1/rollout")?;
    let payload = serde_json::to_string(&RolloutRequest {
        target: entry.target.clone(),
        version: version.to_string(),
        program: program.map(str::to_string),
        program_hash: program_hash.map(str::to_string),
    })
    .map_err(|e| e.to_string())?;
    let response = http_request("POST", &url, Some(&payload), entry.token.as_deref())?;
    decode_response(response)
}

pub fn agent_rollback(entry: &DeployAgentEntry) -> Result<AgentRolloutResponse, String> {
    let url = agent_endpoint(&entry.url, "/v1/rollback")?;
    let payload = serde_json::to_string(&serde_json::json!({
        "target": entry.target,
    }))
    .map_err(|e| e.to_string())?;
    let response = http_request("POST", &url, Some(&payload), entry.token.as_deref())?;
    decode_response(response)
}

pub fn execute_remote_rollout(
    plan: &DeployPlan,
    options: &RolloutOptions,
    registry: &DeployAgentRegistry,
) -> RolloutResult {
    // Plan rollout steps locally, then push updates to registered deploy agents.
    let local = crate::deploy_service::plan_rollout(plan, options);
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
        match agent_rollout(
            agent,
            &step.version,
            Some(&plan.program),
            plan.program_hash.as_deref(),
        ) {
            Ok(resp) if resp.ok => steps.push(RolloutStep {
                status: RolloutStepStatus::Deployed,
                ..step.clone()
            }),
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

pub fn execute_remote_rollback(
    plan: &DeployPlan,
    registry: &DeployAgentRegistry,
) -> RolloutResult {
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

pub fn missing_remote_targets(
    plan: &DeployPlan,
    registry: &DeployAgentRegistry,
) -> Vec<String> {
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
    registry
        .agents
        .iter()
        .cloned()
        .map(|entry| (entry.target.clone(), entry))
        .collect()
}
