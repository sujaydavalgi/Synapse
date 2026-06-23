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
    #[serde(default)]
    pub require_hash: bool,
    #[serde(default)]
    pub require_signature: bool,
    #[serde(default)]
    pub require_certify: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trusted_public_key: Option<String>,
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
    PathBuf::from(".spanda/agent-state.json")
}

pub fn agent_state_path_for(target: &str) -> PathBuf {
    // Keep one state file per deploy target so concurrent agents do not clobber identity.
    let safe_target = target.replace(['/', '\\', '@', ':'], "_");
    PathBuf::from(format!(".spanda/agent-state/{safe_target}.json"))
}

pub fn load_agent_state(path: &Path) -> AgentState {
    if !path.exists() {
        return AgentState::default();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save_agent_state(path: &Path, state: &AgentState) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}

fn unauthorized(request: &HttpRequest, state: &AgentState) -> bool {
    match (&state.token, &request.authorization) {
        (Some(expected), Some(provided)) => expected != provided,
        (Some(_), None) => true,
        _ => false,
    }
}

fn clear_agent_deployment_on_identity_change(state: &mut AgentState, new_target: &str) {
    // Drop stale rollout fields when the on-disk identity does not match startup target.
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
    path.split('?').nth(1).is_some_and(|query| {
        query.split('&').any(|pair| {
            pair == key
                || pair == format!("{key}=true")
                || pair == format!("{key}=1")
        })
    })
}

fn readiness_path_base(path: &str) -> &str {
    path.split('?').next().unwrap_or(path)
}

pub fn handle_agent_request(state: &mut AgentState, request: HttpRequest) -> HttpResponse {
    // Route deploy agent protocol requests to local state transitions.
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
            match spanda_readiness::evaluate_agent_readiness_json(
                program,
                target,
                include_runtime,
                inject,
            ) {
                Ok(body) => HttpResponse { status: 200, body },
                Err(err) => HttpResponse {
                    status: 500,
                    body: format!(r#"{{"ok":false,"error":"{err}"}}"#),
                },
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
    // Run the deploy agent until the listener is interrupted.
    //
    // Parameters:
    // - `options` — bind address, target robot, TLS, and verification policy
    //
    // Returns:
    // Ok when the listener shuts down cleanly, or an error string.
    //
    // Options:
    // None.
    //
    // Example:
    // run_deploy_agent_server(&DeployAgentServerOptions { bind: "0.0.0.0:8787".into(), .. })?;

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
    spawn_test_agent_with_options(target, token, false)
}

pub fn spawn_test_agent_with_options(
    target: &str,
    token: Option<String>,
    require_certify: bool,
) -> Result<(u16, thread::JoinHandle<()>), String> {
    // Start a background deploy agent for integration tests.
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let state = AgentState {
        target: target.to_string(),
        current_version: "0.0.0".into(),
        previous_version: None,
        token,
        require_certify,
        ..AgentState::default()
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
        assert_ne!(
            agent_state_path_for("Rover@JetsonOrin"),
            agent_state_path_for("Scout@Pi4")
        );
    }
}
