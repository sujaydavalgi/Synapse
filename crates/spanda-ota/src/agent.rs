//! On-device Spanda deploy agent HTTP server.

use crate::bundle::{verify_deploy_bundle, DeployArtifactBundle};
use crate::remote::DeployAgentEntry;
use crate::types::{CertificationProofSummary, DeployAssignment};
use serde::{Deserialize, Serialize};
use spanda_deploy_http::{
    parse_http_request, read_plain_request, serve_tls_connection, write_plain_response,
    DeployAgentTls, HttpRequest, HttpResponse,
};
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentState {
    pub target: String,
    pub current_version: String,
    pub previous_version: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
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
    #[serde(default)]
    pub require_hash: bool,
    #[serde(default)]
    pub require_signature: bool,
    #[serde(default)]
    pub require_certify: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trusted_public_key: Option<String>,
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
struct RolloutResponse {
    ok: bool,
    target: String,
    version: String,
    previous_version: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

pub fn default_agent_state_path() -> PathBuf {
    // Description:
    //     Default agent state path.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: PathBuf
    //         Return value from `default_agent_state_path`.
    //
    // Example:

    //     let result = spanda_ota::agent::default_agent_state_path();

    PathBuf::from(".spanda/agent-state.json")
}

pub fn agent_state_path_for(target: &str) -> PathBuf {
    // Description:

    //     Agent state path for.

    //

    // Inputs:

    //     arge: &str

    //         Caller-supplied arge.

    //

    // Outputs:

    //     result: PathBuf

    //         Return value from `agent_state_path_for`.

    //

    // Example:

    //     let result = spanda_ota::agent::agent_state_path_for(arge);
    let safe_target = target.replace(['/', '\\', '@', ':'], "_");
    PathBuf::from(format!(".spanda/agent-state/{safe_target}.json"))
}

pub fn load_agent_state(path: &Path) -> AgentState {
    // Description:
    //     Load agent state.
    //
    // Inputs:
    //     path: &Path
    //         Caller-supplied path.
    //
    // Outputs:
    //     result: AgentState
    //         Return value from `load_agent_state`.
    //
    // Example:

    //     let result = spanda_ota::agent::load_agent_state(path);

    if !path.exists() {
        return AgentState::default();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save_agent_state(path: &Path, state: &AgentState) -> Result<(), String> {
    // Description:
    //     Save agent state.
    //
    // Inputs:
    //     path: &Path
    //         Caller-supplied path.
    //     state: &AgentState
    //         Caller-supplied state.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `save_agent_state`.
    //
    // Example:

    //     let result = spanda_ota::agent::save_agent_state(path, state);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}

fn apply_attestation_env(state: &mut AgentState) {
    if let Ok(contract) = std::env::var("SPANDA_ATTESTATION_CONTRACT") {
        let trimmed = contract.trim();
        if !trimmed.is_empty() {
            state.attestation_contract = Some(trimmed.to_string());
        }
    }
    if let Ok(boot_state) = std::env::var("SPANDA_BOOT_STATE") {
        let trimmed = boot_state.trim();
        if !trimmed.is_empty() {
            state.boot_state = Some(trimmed.to_string());
        }
    }
    if std::env::var("SPANDA_ATTESTATION_VERIFIED")
        .ok()
        .map(|value| value == "1")
        .unwrap_or(false)
    {
        state.attestation_verified = Some(true);
    }
}

fn unauthorized(request: &HttpRequest, state: &AgentState) -> bool {
    // Description:
    //     Unauthorized.
    //
    // Inputs:
    //     request: &HttpRequest
    //         Caller-supplied request.
    //     state: &AgentState
    //         Caller-supplied state.
    //
    // Outputs:
    //     result: bool
    //         Return value from `unauthorized`.
    //
    // Example:

    //     let result = spanda_ota::agent::unauthorized(reques, state);

    match (&state.token, &request.authorization) {
        (Some(expected), Some(provided)) => expected != provided,
        (Some(_), None) => true,
        _ => false,
    }
}

fn clear_agent_deployment_on_identity_change(state: &mut AgentState, new_target: &str) {
    // Description:

    //     Clear agent deployment on identity change.

    //

    // Inputs:

    //     state: &mut AgentState

    //         Caller-supplied state.

    //     new_targe: &str

    //         Caller-supplied new targe.

    //

    // Outputs:

    //     None.

    //

    // Example:

    //     let result = spanda_ota::agent::clear_agent_deployment_on_identity_change(state, new_targe);
    if !state.target.is_empty() && state.target != new_target {
        state.current_version = "0.0.0".into();
        state.previous_version = None;
        state.program = None;
        state.program_hash = None;
        state.require_hash = false;
        state.require_signature = false;
        state.require_certify = false;
        state.trusted_public_key = None;
        state.token = None;
    }
}

fn query_flag(path: &str, key: &str) -> bool {
    // Description:
    //     Query flag.
    //
    // Inputs:
    //     path: &str
    //         Caller-supplied path.
    //     key: &str
    //         Caller-supplied key.
    //
    // Outputs:
    //     result: bool
    //         Return value from `query_flag`.
    //
    // Example:

    //     let result = spanda_ota::agent::query_flag(path, key);

    path.split('?').nth(1).is_some_and(|query| {
        query
            .split('&')
            .any(|pair| pair == key || pair == format!("{key}=true") || pair == format!("{key}=1"))
    })
}

fn readiness_path_base(path: &str) -> &str {
    // Description:
    //     Readiness path base.
    //
    // Inputs:
    //     path: &str
    //         Caller-supplied path.
    //
    // Outputs:
    //     result: &str
    //         Return value from `readiness_path_base`.
    //
    // Example:

    //     let result = spanda_ota::agent::readiness_path_base(path);

    path.split('?').next().unwrap_or(path)
}

pub fn handle_agent_request(state: &mut AgentState, request: HttpRequest) -> HttpResponse {
    // Description:

    //     Handle agent request.

    //

    // Inputs:

    //     state: &mut AgentState

    //         Caller-supplied state.

    //     request: HttpRequest

    //         Caller-supplied request.

    //

    // Outputs:

    //     result: HttpResponse

    //         Return value from `handle_agent_request`.

    //

    // Example:

    //     let result = spanda_ota::agent::handle_agent_request(state, reques);
    if unauthorized(&request, state) {
        return HttpResponse {
            status: 401,
            body: r#"{"ok":false,"error":"unauthorized"}"#.into(),
        };
    }

    match (request.method.as_str(), readiness_path_base(&request.path)) {
        ("GET", "/v1/health") => HttpResponse {
            status: 200,
            body: r#"{"ok":true,"agent":"spanda-deploy-agent","version":"0.1.0"}"#.into(),
        },
        ("GET", "/v1/readiness") => {
            let Some(program) = state.program.as_deref() else {
                return HttpResponse {
                    status: 503,
                    body: r#"{"ok":false,"error":"no program deployed on agent"}"#.into(),
                };
            };
            let target = if state.target.is_empty() {
                None
            } else {
                Some(state.target.as_str())
            };
            let include_runtime = query_flag(&request.path, "runtime");
            let inject = query_flag(&request.path, "inject_health_faults");
            match spanda_runtime::readiness_runtime::readiness_runtime()
                .evaluate_agent_readiness_json(program, target, include_runtime, inject)
            {
                Ok(body) => HttpResponse { status: 200, body },
                Err(err) => HttpResponse {
                    status: 500,
                    body: format!(r#"{{"ok":false,"error":"{err}"}}"#),
                },
            }
        }
        ("POST", "/v1/program") => {
            let Ok(payload) = serde_json::from_str::<serde_json::Value>(&request.body) else {
                return HttpResponse {
                    status: 400,
                    body: r#"{"ok":false,"error":"invalid program payload"}"#.into(),
                };
            };
            let Some(program) = payload.get("program").and_then(|v| v.as_str()) else {
                return HttpResponse {
                    status: 400,
                    body: r#"{"ok":false,"error":"program field required"}"#.into(),
                };
            };
            state.program = Some(program.to_string());
            HttpResponse {
                status: 200,
                body: r#"{"ok":true}"#.into(),
            }
        }
        ("GET", "/v1/status") => HttpResponse {
            status: 200,
            body: serde_json::to_string(&serde_json::json!({
                "ok": true,
                "target": state.target,
                "current_version": state.current_version,
                "previous_version": state.previous_version,
                "program": state.program,
                "program_hash": state.program_hash,
                "hardware_profile": state.hardware_profile,
                "firmware_version": state.firmware_version,
                "packages": state.packages,
                "attestation_contract": state.attestation_contract,
                "attestation_verified": state.attestation_verified,
                "boot_state": state.boot_state,
                "healthy": true,
            }))
            .unwrap_or_else(|_| "{}".into()),
        },
        ("POST", "/v1/rollout") => {
            let Ok(payload) = serde_json::from_str::<RolloutRequest>(&request.body) else {
                return HttpResponse {
                    status: 400,
                    body: r#"{"ok":false,"error":"invalid rollout payload"}"#.into(),
                };
            };
            if !state.target.is_empty() && payload.target != state.target {
                return HttpResponse {
                    status: 400,
                    body: r#"{"ok":false,"error":"target mismatch"}"#.into(),
                };
            }
            if state.require_hash && payload.program_hash.is_none() {
                return HttpResponse {
                    status: 400,
                    body: r#"{"ok":false,"error":"program_hash required"}"#.into(),
                };
            }
            if state.require_signature {
                let Some(trusted) = state.trusted_public_key.as_deref() else {
                    return HttpResponse {
                        status: 500,
                        body: r#"{"ok":false,"error":"agent missing trusted public key"}"#.into(),
                    };
                };
                let bundle = DeployArtifactBundle {
                    version: payload.version.clone(),
                    program: payload.program.clone().unwrap_or_default(),
                    program_hash: payload.program_hash.clone(),
                    assignments: payload.assignments.clone(),
                    certifications: payload.certifications.clone(),
                    signature: payload.artifact_signature.clone(),
                    public_key: payload.artifact_public_key.clone(),
                };
                if !verify_deploy_bundle(&bundle, trusted) {
                    return HttpResponse {
                        status: 400,
                        body: r#"{"ok":false,"error":"invalid artifact signature"}"#.into(),
                    };
                }
            }
            if state.require_certify {
                let proof_ok = payload
                    .certification_proof
                    .as_ref()
                    .is_some_and(|proof| proof.passed_strict);
                if !proof_ok {
                    return HttpResponse {
                        status: 400,
                        body: r#"{"ok":false,"error":"strict certification proof required"}"#
                            .into(),
                    };
                }
            }
            if !state.current_version.is_empty() {
                state.previous_version = Some(state.current_version.clone());
            }
            state.current_version = payload.version.clone();
            state.program = payload.program.clone();
            state.program_hash = payload.program_hash.clone();
            if let Some(hw) = payload.hardware_profile.as_ref() {
                state.hardware_profile = Some(hw.clone());
            } else if let Some(assignment) = payload.assignments.iter().find(|a| {
                crate::service::deploy_target_key(&a.robot_name, &a.hardware) == state.target
            }) {
                state.hardware_profile = Some(assignment.hardware.clone());
            }
            if let Some(fw) = payload.firmware_version.as_ref() {
                state.firmware_version = Some(fw.clone());
            }
            if !payload.packages.is_empty() {
                state.packages = payload.packages.clone();
            }
            HttpResponse {
                status: 200,
                body: serde_json::to_string(&RolloutResponse {
                    ok: true,
                    target: state.target.clone(),
                    version: state.current_version.clone(),
                    previous_version: state.previous_version.clone(),
                    error: None,
                })
                .unwrap_or_else(|_| "{}".into()),
            }
        }
        ("POST", "/v1/rollback") => {
            let Some(previous) = state.previous_version.clone() else {
                return HttpResponse {
                    status: 409,
                    body: r#"{"ok":false,"error":"no previous version"}"#.into(),
                };
            };
            let current = state.current_version.clone();
            state.current_version = previous.clone();
            state.previous_version = Some(current);
            HttpResponse {
                status: 200,
                body: serde_json::to_string(&RolloutResponse {
                    ok: true,
                    target: state.target.clone(),
                    version: state.current_version.clone(),
                    previous_version: state.previous_version.clone(),
                    error: None,
                })
                .unwrap_or_else(|_| "{}".into()),
            }
        }
        _ => HttpResponse {
            status: 404,
            body: r#"{"ok":false,"error":"not found"}"#.into(),
        },
    }
}

fn handle_connection(
    state: Arc<Mutex<AgentState>>,
    state_path: PathBuf,
    mut stream: TcpStream,
    tls: Option<Arc<rustls::ServerConfig>>,
) {
    // Description:
    //     Handle connection.
    //
    // Inputs:
    //     state: Arc<Mutex<AgentState>>
    //         Caller-supplied state.
    //     state_path: PathBuf
    //         Caller-supplied state path.
    //     strea: TcpStream
    //         Caller-supplied strea.
    //     ls: Option<Arc<rustls::ServerConfig>>
    //         Caller-supplied ls.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_ota::agent::handle_connection(state, state_path, strea, ls);

    let respond = |locked: &mut AgentState, request: HttpRequest| -> HttpResponse {
        let response = handle_agent_request(locked, request);
        let _ = save_agent_state(&state_path, locked);
        response
    };

    if let Some(server_config) = tls {
        let shared = Arc::clone(&state);
        let path = state_path.clone();
        let _ = serve_tls_connection(&server_config, stream, move |request| {
            let mut locked = shared.lock().expect("agent state lock");
            let response = handle_agent_request(&mut locked, request);
            let _ = save_agent_state(&path, &locked);
            response
        });
        return;
    }

    let raw = match read_plain_request(&mut stream) {
        Ok(raw) => raw,
        Err(_) => {
            let _ = write_plain_response(
                &mut stream,
                &HttpResponse {
                    status: 400,
                    body: r#"{"ok":false,"error":"bad request"}"#.into(),
                },
            );
            return;
        }
    };
    let Ok(request) = parse_http_request(&raw) else {
        let _ = write_plain_response(
            &mut stream,
            &HttpResponse {
                status: 400,
                body: r#"{"ok":false,"error":"bad request"}"#.into(),
            },
        );
        return;
    };
    let response = {
        let mut locked = state.lock().expect("agent state lock");
        respond(&mut locked, request)
    };
    let _ = write_plain_response(&mut stream, &response);
}

/// Configuration for starting the on-device deploy agent HTTP server.
#[derive(Debug, Clone)]
pub struct DeployAgentServerOptions {
    pub bind: String,
    pub target: String,
    pub token: Option<String>,
    pub state_path: PathBuf,
    pub tls: Option<DeployAgentTls>,
    pub require_hash: bool,
    pub require_signature: bool,
    pub require_certify: bool,
    pub trusted_public_key: Option<String>,
}

pub fn run_deploy_agent_server(options: &DeployAgentServerOptions) -> Result<(), String> {
    // Description:
    //     Run deploy agent server.
    //
    // Inputs:
    //     options: &DeployAgentServerOptions
    //         Caller-supplied options.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `run_deploy_agent_server`.
    //
    // Example:

    //     let result = spanda_ota::agent::run_deploy_agent_server(options);

    let DeployAgentServerOptions {
        bind,
        target,
        token,
        state_path,
        tls,
        require_hash,
        require_signature,
        require_certify,
        trusted_public_key,
    } = options;
    let bind = bind.as_str();
    let target = target.as_str();
    let state_path = state_path.as_path();
    let mut state = load_agent_state(state_path);
    clear_agent_deployment_on_identity_change(&mut state, target);
    state.target = target.to_string();
    state.token = token.clone().or(state.token);
    state.require_hash = *require_hash || state.require_hash;
    state.require_signature = *require_signature || state.require_signature;
    state.require_certify = *require_certify || state.require_certify;
    state.trusted_public_key = trusted_public_key.clone().or(state.trusted_public_key);
    apply_attestation_env(&mut state);
    save_agent_state(state_path, &state)?;
    let listener = TcpListener::bind(bind).map_err(|e| format!("bind {bind} failed: {e}"))?;
    let shared = Arc::new(Mutex::new(state));
    let server_config = tls
        .as_ref()
        .map(spanda_deploy_http::build_deploy_server_config)
        .transpose()?;
    let scheme = if server_config.is_some() {
        "https"
    } else {
        "http"
    };
    eprintln!("Spanda deploy agent listening on {scheme}://{bind} for target {target}");
    for connection in listener.incoming() {
        let Ok(stream) = connection else { continue };
        let shared_clone = Arc::clone(&shared);
        let path_clone = state_path.to_path_buf();
        let tls_clone = server_config.clone();
        thread::spawn(move || handle_connection(shared_clone, path_clone, stream, tls_clone));
    }
    Ok(())
}

pub fn spawn_test_agent(
    target: &str,
    token: Option<String>,
) -> Result<(u16, thread::JoinHandle<()>), String> {
    // Description:
    //     Spawn test agent.
    //
    // Inputs:
    //     arge: &str
    //         Caller-supplied arge.
    //     token: Option<String>
    //         Caller-supplied token.
    //
    // Outputs:
    //     result: Result<(u16, thread::JoinHandle<()>), String>
    //         Return value from `spawn_test_agent`.
    //
    // Example:

    //     let result = spanda_ota::agent::spawn_test_agent(arge, oken);

    spawn_test_agent_with_options(target, token, false)
}

pub fn spawn_test_agent_with_options(
    target: &str,
    token: Option<String>,
    require_certify: bool,
) -> Result<(u16, thread::JoinHandle<()>), String> {
    // Description:

    //     Spawn test agent with options.

    //

    // Inputs:

    //     arge: &str

    //         Caller-supplied arge.

    //     token: Option<String>

    //         Caller-supplied token.

    //     require_certify: bool

    //         Caller-supplied require certify.

    //

    // Outputs:

    //     result: Result<(u16, thread::JoinHandle<()>), String>

    //         Return value from `spawn_test_agent_with_options`.

    //

    // Example:

    //     let result = spanda_ota::agent::spawn_test_agent_with_options(arge, oken, require_certify);
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let state = {
        let mut state = AgentState {
            target: target.to_string(),
            current_version: "0.0.0".into(),
            previous_version: None,
            token,
            require_certify,
            ..AgentState::default()
        };
        apply_attestation_env(&mut state);
        state
    };
    let shared = Arc::new(Mutex::new(state));
    let state_path = agent_state_path_for(target);
    let handle = thread::spawn(move || {
        for connection in listener.incoming() {
            let Ok(stream) = connection else { continue };
            handle_connection(Arc::clone(&shared), state_path.clone(), stream, None);
        }
    });
    Ok((port, handle))
}

pub fn agent_entry_for_port(target: &str, port: u16, token: Option<String>) -> DeployAgentEntry {
    // Description:
    //     Agent entry for port.
    //
    // Inputs:
    //     arge: &str
    //         Caller-supplied arge.
    //     por: u16
    //         Caller-supplied por.
    //     token: Option<String>
    //         Caller-supplied token.
    //
    // Outputs:
    //     result: DeployAgentEntry
    //         Return value from `agent_entry_for_port`.
    //
    // Example:

    //     let result = spanda_ota::agent::agent_entry_for_port(arge, por, oken);

    DeployAgentEntry {
        target: target.to_string(),
        url: format!("http://127.0.0.1:{port}"),
        token,
    }
}

#[cfg(test)]
mod agent_state_path_tests {
    use super::*;

    #[test]
    fn distinct_targets_use_distinct_paths() {
        // Description:
        //     Distinct targets use distinct paths.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_ota::agent::distinct_targets_use_distinct_paths();

        assert_ne!(
            agent_state_path_for("Rover@JetsonOrin"),
            agent_state_path_for("Scout@Pi4")
        );
    }
}
