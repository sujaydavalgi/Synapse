//! On-device fleet peer relay HTTP server.
//!
use crate::mesh::mesh_registry_path;
use crate::remote::{
    load_fleet_agent_registry, lookup_fleet_agent, relay_peer_delivery, FleetAgentEntry,
    PeerRelayRequest, PeerRelayResponse,
};
use crate::PeerDelivery;
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FleetAgentState {
    pub robot_name: String,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub program: Option<String>,
    #[serde(default)]
    pub last_peer_messages: Vec<String>,
    #[serde(default)]
    pub last_recovery_commands: Vec<String>,
    #[serde(default)]
    pub recovery_active: Option<String>,
    #[serde(default)]
    pub recovery_actions_applied: Vec<String>,
    #[serde(default)]
    pub mission_paused: bool,
    #[serde(default)]
    pub recovery_mode: Option<String>,
    #[serde(default)]
    pub recovery_validation: Option<String>,
    #[serde(default)]
    pub last_recovery_evidence: Option<serde_json::Value>,
    #[serde(default)]
    pub recovery_speed_cap: Option<f64>,
    #[serde(default)]
    pub recovery_engine: Option<String>,
    #[serde(default)]
    pub last_recovery_runtime_logs: Vec<String>,
    #[serde(default)]
    pub last_continuity_commands: Vec<String>,
    #[serde(default)]
    pub continuity_active: Option<String>,
    #[serde(default)]
    pub continuity_successor: Option<String>,
    #[serde(default)]
    pub continuity_mode: Option<String>,
    #[serde(default)]
    pub continuity_validation: Option<String>,
    #[serde(default)]
    pub last_continuity_evidence: Option<serde_json::Value>,
    #[serde(default)]
    pub continuity_engine: Option<String>,
    #[serde(default)]
    pub last_continuity_runtime_logs: Vec<String>,
    #[serde(default)]
    pub mission_progress_percent: Option<f64>,
    #[serde(default)]
    pub mission_handoff_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub program_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hardware_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firmware_version: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<String>,
}

pub fn default_fleet_agent_state_path() -> PathBuf {
    // Description:
    //     Default fleet agent state path.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: PathBuf
    //         Return value from `default_fleet_agent_state_path`.
    //
    // Example:

    //     let result = spanda_fleet::agent::default_fleet_agent_state_path();

    PathBuf::from(".spanda/fleet-agent-state.json")
}

pub fn fleet_agent_state_path_for(robot_name: &str) -> PathBuf {
    // Description:

    //     Fleet agent state path for.

    //

    // Inputs:

    //     robot_name: &str

    //         Caller-supplied robot name.

    //

    // Outputs:

    //     result: PathBuf

    //         Return value from `fleet_agent_state_path_for`.

    //

    // Example:

    //     let result = spanda_fleet::agent::fleet_agent_state_path_for(robot_name);
    let safe_name = robot_name.replace(['/', '\\'], "_");
    PathBuf::from(format!(".spanda/fleet-agent-state/{safe_name}.json"))
}

pub fn load_fleet_agent_state(path: &Path) -> FleetAgentState {
    // Description:
    //     Load fleet agent state.
    //
    // Inputs:
    //     path: &Path
    //         Caller-supplied path.
    //
    // Outputs:
    //     result: FleetAgentState
    //         Return value from `load_fleet_agent_state`.
    //
    // Example:

    //     let result = spanda_fleet::agent::load_fleet_agent_state(path);

    if !path.exists() {
        return FleetAgentState::default();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save_fleet_agent_state(path: &Path, state: &FleetAgentState) -> Result<(), String> {
    // Description:
    //     Save fleet agent state.
    //
    // Inputs:
    //     path: &Path
    //         Caller-supplied path.
    //     state: &FleetAgentState
    //         Caller-supplied state.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `save_fleet_agent_state`.
    //
    // Example:

    //     let result = spanda_fleet::agent::save_fleet_agent_state(path, state);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}

fn unauthorized(request: &HttpRequest, state: &FleetAgentState) -> bool {
    // Description:
    //     Unauthorized.
    //
    // Inputs:
    //     request: &HttpRequest
    //         Caller-supplied request.
    //     state: &FleetAgentState
    //         Caller-supplied state.
    //
    // Outputs:
    //     result: bool
    //         Return value from `unauthorized`.
    //
    // Example:

    //     let result = spanda_fleet::agent::unauthorized(reques, state);

    match (&state.token, &request.authorization) {
        (Some(expected), Some(provided)) => expected != provided,
        (Some(_), None) => true,
        _ => false,
    }
}

fn clear_fleet_agent_on_identity_change(state: &mut FleetAgentState, new_robot_name: &str) {
    // Description:

    //     Clear fleet agent on identity change.

    //

    // Inputs:

    //     state: &mut FleetAgentState

    //         Caller-supplied state.

    //     new_robot_name: &str

    //         Caller-supplied new robot name.

    //

    // Outputs:

    //     None.

    //

    // Example:

    //     let result = spanda_fleet::agent::clear_fleet_agent_on_identity_change(state, new_robot_name);
    if !state.robot_name.is_empty() && state.robot_name != new_robot_name {
        state.last_peer_messages.clear();
        state.token = None;
        state.last_recovery_commands.clear();
        state.recovery_active = None;
        state.recovery_actions_applied.clear();
        state.mission_paused = false;
        state.recovery_mode = None;
        state.recovery_validation = None;
        state.last_recovery_evidence = None;
        state.recovery_speed_cap = None;
        state.recovery_engine = None;
        state.last_recovery_runtime_logs.clear();
        state.last_continuity_commands.clear();
        state.continuity_active = None;
        state.continuity_successor = None;
        state.continuity_mode = None;
        state.continuity_validation = None;
        state.last_continuity_evidence = None;
        state.continuity_engine = None;
        state.last_continuity_runtime_logs.clear();
        state.mission_progress_percent = None;
        state.mission_handoff_from = None;
    }
}

pub(crate) fn apply_continuity_takeover(
    state: &mut FleetAgentState,
    report: &spanda_runtime::TakeoverReport,
    request: &spanda_deploy_http::FleetContinuityRequest,
) {
    state.continuity_active = Some(format!("takeover:{}", report.successor));
    state.continuity_successor = Some(report.successor.clone());
    state.continuity_mode = Some(format!("{:?}", report.mode));
    state.mission_progress_percent = request.progress_percent;
    state.mission_handoff_from = Some(request.failed_robot.clone());

    if state.robot_name == request.failed_robot {
        state.mission_paused = true;
    }
    if state.robot_name == report.successor {
        state.mission_paused = false;
        if let Some(progress) = request.progress_percent {
            state.mission_progress_percent = Some(progress);
        }
    }
}

pub(crate) fn apply_recovery_action(state: &mut FleetAgentState, action: &str) {
    // Description:
    //     Apply recovery action.
    //
    // Inputs:
    //     state: &mut FleetAgentState
    //         Caller-supplied state.
    //     action: &str
    //         Caller-supplied action.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_fleet::agent::apply_recovery_action(state, action);

    state.recovery_active = Some(action.to_string());
    state.recovery_actions_applied.push(action.to_string());
    let lower = action.to_ascii_lowercase();
    if lower.contains("pause") && lower.contains("mission") {
        state.mission_paused = true;
    }
    if lower.contains("degraded") {
        state.recovery_mode = Some("degraded".into());
    } else if lower.contains("safe") && lower.contains("mode") {
        state.recovery_mode = Some("safe".into());
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

    //     let result = spanda_fleet::agent::query_flag(path, key);

    path.split('?').nth(1).is_some_and(|query| {
        query.split('&').any(|pair| {
            pair == key
                || pair == format!("{key}=true")
                || pair == format!("{key}=1")
                || pair.starts_with(&format!("{key}=true"))
        })
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

    //     let result = spanda_fleet::agent::readiness_path_base(path);

    path.split('?').next().unwrap_or(path)
}

pub fn handle_fleet_agent_request(
    state: &mut FleetAgentState,
    request: HttpRequest,
) -> HttpResponse {
    // Description:

    //     Handle fleet agent request.

    //

    // Inputs:

    //     state: &mut FleetAgentState

    //         Caller-supplied state.

    //     request: HttpRequest

    //         Caller-supplied request.

    //

    // Outputs:

    //     result: HttpResponse

    //         Return value from `handle_fleet_agent_request`.

    //

    // Example:

    //     let result = spanda_fleet::agent::handle_fleet_agent_request(state, reques);
    if unauthorized(&request, state) {
        return HttpResponse {
            status: 401,
            body: r#"{"ok":false,"error":"unauthorized"}"#.into(),
        };
    }

    match (request.method.as_str(), readiness_path_base(&request.path)) {
        ("GET", "/v1/health") => HttpResponse {
            status: 200,
            body: r#"{"ok":true,"agent":"spanda-fleet-agent","version":"0.1.0"}"#.into(),
        },
        ("GET", "/v1/readiness") => {
            let Some(program) = state.program.as_deref() else {
                return HttpResponse {
                    status: 503,
                    body: r#"{"ok":false,"error":"no program deployed on fleet agent"}"#.into(),
                };
            };
            let include_runtime = query_flag(&request.path, "runtime");
            let inject = query_flag(&request.path, "inject_health_faults");
            match spanda_runtime::readiness_runtime::readiness_runtime()
                .evaluate_agent_readiness_json(
                    program,
                    Some(&state.robot_name),
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
        ("POST", "/v1/recovery/ack") => {
            state.recovery_active = None;
            state.mission_paused = false;
            state.recovery_mode = None;
            state.recovery_validation = None;
            state.last_recovery_evidence = None;
            state.recovery_speed_cap = None;
            state.recovery_engine = None;
            state.last_recovery_runtime_logs.clear();
            HttpResponse {
                status: 200,
                body: r#"{"ok":true}"#.into(),
            }
        }
        ("POST", "/v1/continuity/ack") => {
            state.continuity_active = None;
            state.continuity_mode = None;
            state.continuity_validation = None;
            state.last_continuity_evidence = None;
            state.continuity_engine = None;
            state.last_continuity_runtime_logs.clear();
            HttpResponse {
                status: 200,
                body: r#"{"ok":true}"#.into(),
            }
        }
        ("POST", "/v1/recovery/execute") => {
            let Ok(payload) = serde_json::from_str::<serde_json::Value>(&request.body) else {
                return HttpResponse {
                    status: 400,
                    body: r#"{"ok":false,"error":"invalid recovery payload"}"#.into(),
                };
            };
            let Some(action) = payload.get("action").and_then(|v| v.as_str()) else {
                return HttpResponse {
                    status: 400,
                    body: r#"{"ok":false,"error":"action field required"}"#.into(),
                };
            };
            if action.trim().is_empty() {
                return HttpResponse {
                    status: 400,
                    body: r#"{"ok":false,"error":"action must not be empty"}"#.into(),
                };
            };
            crate::recovery_agent::handle_fleet_recovery_command(state, action);
            HttpResponse {
                status: 200,
                body: serde_json::to_string(&serde_json::json!({
                    "ok": true,
                    "recovery_active": state.recovery_active,
                    "recovery_validation": state.recovery_validation,
                    "recovery_engine": state.recovery_engine,
                    "recovery_speed_cap": state.recovery_speed_cap,
                    "recovery_actions_applied": state.recovery_actions_applied,
                    "mission_paused": state.mission_paused,
                    "recovery_mode": state.recovery_mode,
                }))
                .unwrap_or_else(|_| r#"{"ok":true}"#.into()),
            }
        }
        ("POST", "/v1/continuity/execute") => {
            let body = request.body.clone();
            let Ok(takeover) =
                serde_json::from_str::<spanda_deploy_http::FleetContinuityRequest>(&body)
            else {
                return HttpResponse {
                    status: 400,
                    body: r#"{"ok":false,"error":"invalid continuity payload"}"#.into(),
                };
            };
            if takeover.failed_robot.trim().is_empty() {
                return HttpResponse {
                    status: 400,
                    body: r#"{"ok":false,"error":"failed_robot required"}"#.into(),
                };
            }
            crate::continuity_agent::handle_fleet_takeover_command(state, &body);
            HttpResponse {
                status: 200,
                body: serde_json::to_string(&serde_json::json!({
                    "ok": true,
                    "continuity_active": state.continuity_active,
                    "continuity_successor": state.continuity_successor,
                    "continuity_validation": state.continuity_validation,
                    "continuity_engine": state.continuity_engine,
                    "mission_paused": state.mission_paused,
                    "mission_progress_percent": state.mission_progress_percent,
                    "mission_handoff_from": state.mission_handoff_from,
                }))
                .unwrap_or_else(|_| r#"{"ok":true}"#.into()),
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
                "robot_name": state.robot_name,
                "last_peer_messages": state.last_peer_messages,
                "last_recovery_commands": state.last_recovery_commands,
                "recovery_active": state.recovery_active,
                "recovery_actions_applied": state.recovery_actions_applied,
                "mission_paused": state.mission_paused,
                "recovery_mode": state.recovery_mode,
                "recovery_validation": state.recovery_validation,
                "recovery_engine": state.recovery_engine,
                "recovery_speed_cap": state.recovery_speed_cap,
                "last_recovery_runtime_logs": state.last_recovery_runtime_logs,
                "last_recovery_evidence": state.last_recovery_evidence,
                "last_continuity_commands": state.last_continuity_commands,
                "continuity_active": state.continuity_active,
                "continuity_successor": state.continuity_successor,
                "continuity_mode": state.continuity_mode,
                "continuity_validation": state.continuity_validation,
                "continuity_engine": state.continuity_engine,
                "last_continuity_runtime_logs": state.last_continuity_runtime_logs,
                "last_continuity_evidence": state.last_continuity_evidence,
                "mission_progress_percent": state.mission_progress_percent,
                "mission_handoff_from": state.mission_handoff_from,
                "has_program": state.program.is_some(),
                "program_hash": state.program_hash,
                "hardware_profile": state.hardware_profile,
                "firmware_version": state.firmware_version,
                "packages": state.packages,
                "healthy": true,
            }))
            .unwrap_or_else(|_| "{}".into()),
        },
        ("POST", "/v1/peer") => {
            let Ok(payload) = serde_json::from_str::<PeerRelayRequest>(&request.body) else {
                return HttpResponse {
                    status: 400,
                    body: r#"{"ok":false,"error":"invalid peer payload"}"#.into(),
                };
            };

            // Forward peer deliveries destined for another registered robot agent.
            if !payload.to_robot.is_empty()
                && !state.robot_name.is_empty()
                && payload.to_robot != state.robot_name
            {
                let registry = load_fleet_agent_registry(&mesh_registry_path());
                if let Some(entry) = lookup_fleet_agent(&registry, &payload.to_robot) {
                    let delivery = PeerDelivery {
                        from_robot: payload.from_robot.clone(),
                        to_robot: payload.to_robot.clone(),
                        topic: payload.topic.clone(),
                        step: payload.step.clone(),
                        delivered: false,
                    };
                    return match relay_peer_delivery(entry, &delivery) {
                        Ok(resp) => HttpResponse {
                            status: if resp.ok { 200 } else { 502 },
                            body: serde_json::to_string(&resp).unwrap_or_else(|_| "{}".into()),
                        },
                        Err(err) => HttpResponse {
                            status: 502,
                            body: serde_json::to_string(&PeerRelayResponse {
                                ok: false,
                                to_robot: payload.to_robot,
                                topic: payload.topic,
                                step: payload.step,
                                error: Some(err),
                            })
                            .unwrap_or_else(|_| "{}".into()),
                        },
                    };
                }
            }

            if !state.robot_name.is_empty() && payload.to_robot != state.robot_name {
                return HttpResponse {
                    status: 400,
                    body: r#"{"ok":false,"error":"robot mismatch"}"#.into(),
                };
            }
            if state.robot_name.is_empty() {
                return HttpResponse {
                    status: 500,
                    body: r#"{"ok":false,"error":"fleet agent missing robot identity"}"#.into(),
                };
            }
            let message = format!(
                "{}->{}:{}={}",
                payload.from_robot, payload.to_robot, payload.topic, payload.step
            );
            state.last_peer_messages.push(message);
            if payload.topic == "fleet_recovery" {
                crate::recovery_agent::handle_fleet_recovery_command(state, &payload.step);
            }
            if payload.topic == "fleet_takeover" {
                crate::continuity_agent::handle_fleet_takeover_command(state, &payload.step);
            }
            HttpResponse {
                status: 200,
                body: serde_json::to_string(&PeerRelayResponse {
                    ok: true,
                    to_robot: payload.to_robot,
                    topic: payload.topic,
                    step: payload.step,
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
    state: Arc<Mutex<FleetAgentState>>,
    state_path: PathBuf,
    mut stream: TcpStream,
    tls: Option<Arc<rustls::ServerConfig>>,
) {
    // Description:
    //     Handle connection.
    //
    // Inputs:
    //     state: Arc<Mutex<FleetAgentState>>
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

    //     let result = spanda_fleet::agent::handle_connection(state, state_path, strea, ls);

    if let Some(server_config) = tls {
        let shared = Arc::clone(&state);
        let path = state_path.clone();
        let _ = serve_tls_connection(&server_config, stream, move |request| {
            let mut locked = shared.lock().expect("fleet agent state lock");
            let response = handle_fleet_agent_request(&mut locked, request);
            let _ = save_fleet_agent_state(&path, &locked);
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
        let mut locked = state.lock().expect("fleet agent state lock");
        let response = handle_fleet_agent_request(&mut locked, request);
        let _ = save_fleet_agent_state(&state_path, &locked);
        response
    };
    let _ = write_plain_response(&mut stream, &response);
}

pub fn run_fleet_agent_server(
    bind: &str,
    robot_name: &str,
    token: Option<String>,
    state_path: &Path,
    tls: Option<DeployAgentTls>,
) -> Result<(), String> {
    // Description:

    //     Run fleet agent server.

    //

    // Inputs:

    //     bind: &str

    //         Caller-supplied bind.

    //     robot_name: &str

    //         Caller-supplied robot name.

    //     token: Option<String>

    //         Caller-supplied token.

    //     state_path: &Path

    //         Caller-supplied state path.

    //     ls: Option<DeployAgentTls>

    //         Caller-supplied ls.

    //

    // Outputs:

    //     result: Result<(), String>

    //         Return value from `run_fleet_agent_server`.

    //

    // Example:

    //     let result = spanda_fleet::agent::run_fleet_agent_server(bind, robot_name, oken, state_path, ls);
    let mut state = load_fleet_agent_state(state_path);
    clear_fleet_agent_on_identity_change(&mut state, robot_name);
    state.robot_name = robot_name.to_string();
    state.token = token.or(state.token);
    save_fleet_agent_state(state_path, &state)?;
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
    eprintln!("Spanda fleet agent listening on {scheme}://{bind} for robot {robot_name}");
    for connection in listener.incoming() {
        let Ok(stream) = connection else { continue };
        let shared_clone = Arc::clone(&shared);
        let path_clone = state_path.to_path_buf();
        let tls_clone = server_config.clone();
        thread::spawn(move || handle_connection(shared_clone, path_clone, stream, tls_clone));
    }
    Ok(())
}

pub fn spawn_test_fleet_agent(
    robot_name: &str,
    token: Option<String>,
) -> Result<(u16, thread::JoinHandle<()>), String> {
    // Description:
    //     Spawn test fleet agent.
    //
    // Inputs:
    //     robot_name: &str
    //         Caller-supplied robot name.
    //     token: Option<String>
    //         Caller-supplied token.
    //
    // Outputs:
    //     result: Result<(u16, thread::JoinHandle<()>), String>
    //         Return value from `spawn_test_fleet_agent`.
    //
    // Example:

    //     let result = spanda_fleet::agent::spawn_test_fleet_agent(robot_name, oken);

    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let state = FleetAgentState {
        robot_name: robot_name.to_string(),
        token,
        ..FleetAgentState::default()
    };
    let shared = Arc::new(Mutex::new(state));
    let state_path = fleet_agent_state_path_for(robot_name);
    let handle = thread::spawn(move || {
        for connection in listener.incoming() {
            let Ok(stream) = connection else { continue };
            handle_connection(Arc::clone(&shared), state_path.clone(), stream, None);
        }
    });
    Ok((port, handle))
}

pub fn fleet_entry_for_port(robot_name: &str, port: u16, token: Option<String>) -> FleetAgentEntry {
    // Description:
    //     Fleet entry for port.
    //
    // Inputs:
    //     robot_name: &str
    //         Caller-supplied robot name.
    //     por: u16
    //         Caller-supplied por.
    //     token: Option<String>
    //         Caller-supplied token.
    //
    // Outputs:
    //     result: FleetAgentEntry
    //         Return value from `fleet_entry_for_port`.
    //
    // Example:

    //     let result = spanda_fleet::agent::fleet_entry_for_port(robot_name, por, oken);

    FleetAgentEntry {
        robot_name: robot_name.to_string(),
        url: format!("http://127.0.0.1:{port}"),
        token,
    }
}

#[cfg(test)]
mod agent_state_path_tests {
    use super::*;

    #[test]
    fn distinct_robots_use_distinct_paths() {
        // Description:
        //     Distinct robots use distinct paths.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_fleet::agent::distinct_robots_use_distinct_paths();

        assert_ne!(
            fleet_agent_state_path_for("ScoutB"),
            fleet_agent_state_path_for("ScoutC")
        );
    }
}
