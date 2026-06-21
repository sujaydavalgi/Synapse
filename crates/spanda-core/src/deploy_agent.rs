//! On-device Spanda deploy agent HTTP server.

use crate::deploy_bundle::{verify_deploy_bundle, DeployArtifactBundle};
use crate::deploy_http::{
    http_response, parse_http_request, read_plain_request, serve_tls_connection,
    write_plain_response, DeployAgentTls, HttpRequest, HttpResponse,
};
use crate::deploy_remote::DeployAgentEntry;
use crate::deploy_service::DeployAssignment;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
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

pub fn handle_agent_request(state: &mut AgentState, request: HttpRequest) -> HttpResponse {
    // Route deploy agent protocol requests to local state transitions.
    if unauthorized(&request, state) {
        return HttpResponse {
            status: 401,
            body: r#"{"ok":false,"error":"unauthorized"}"#.into(),
        };
    }

    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/v1/health") => HttpResponse {
            status: 200,
            body: r#"{"ok":true,"agent":"spanda-deploy-agent","version":"0.1.0"}"#.into(),
        },
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
            if state.target.is_empty() {
                state.target = payload.target.clone();
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
            let _ = stream.write_all(
                http_response(400, r#"{"ok":false,"error":"bad request"}"#).as_bytes(),
            );
            return;
        }
    };
    let Ok(request) = parse_http_request(&raw) else {
        let _ = stream.write_all(
            http_response(400, r#"{"ok":false,"error":"bad request"}"#).as_bytes(),
        );
        return;
    };
    let response = {
        let mut locked = state.lock().expect("agent state lock");
        respond(&mut locked, request)
    };
    let _ = write_plain_response(&mut stream, &response);
}

pub fn run_deploy_agent_server(
    bind: &str,
    target: &str,
    token: Option<String>,
    state_path: &Path,
    tls: Option<DeployAgentTls>,
    require_hash: bool,
    require_signature: bool,
    trusted_public_key: Option<String>,
) -> Result<(), String> {
    // Run the deploy agent until the listener is interrupted.
    let mut state = load_agent_state(state_path);
    if state.target.is_empty() {
        state.target = target.to_string();
    }
    state.token = token.or(state.token);
    state.require_hash = require_hash || state.require_hash;
    state.require_signature = require_signature || state.require_signature;
    state.trusted_public_key = trusted_public_key.or(state.trusted_public_key);
    save_agent_state(state_path, &state)?;
    let listener = TcpListener::bind(bind).map_err(|e| format!("bind {bind} failed: {e}"))?;
    let shared = Arc::new(Mutex::new(state));
    let server_config = tls
        .as_ref()
        .map(crate::deploy_http::build_deploy_server_config)
        .transpose()?;
    let scheme = if server_config.is_some() { "https" } else { "http" };
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
    // Start a background deploy agent for integration tests.
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let state = AgentState {
        target: target.to_string(),
        current_version: "0.0.0".into(),
        previous_version: None,
        token,
        ..AgentState::default()
    };
    let shared = Arc::new(Mutex::new(state));
    let handle = thread::spawn(move || {
        for connection in listener.incoming() {
            let Ok(stream) = connection else { continue };
            handle_connection(
                Arc::clone(&shared),
                PathBuf::from(".spanda/test-agent-state.json"),
                stream,
                None,
            );
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
