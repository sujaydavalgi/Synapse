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
    http_response, parse_http_request, read_plain_request, serve_tls_connection,
    write_plain_response, DeployAgentTls, HttpRequest, HttpResponse,
};
use std::fs;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetAgentState {
    pub robot_name: String,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub last_peer_messages: Vec<String>,
}

pub fn default_fleet_agent_state_path() -> PathBuf {
    PathBuf::from(".spanda/fleet-agent-state.json")
}

pub fn fleet_agent_state_path_for(robot_name: &str) -> PathBuf {
    // Keep one state file per robot so concurrent agents do not clobber identity.
    let safe_name = robot_name.replace(['/', '\\'], "_");
    PathBuf::from(format!(".spanda/fleet-agent-state/{safe_name}.json"))
}

pub fn load_fleet_agent_state(path: &Path) -> FleetAgentState {
    if !path.exists() {
        return FleetAgentState::default();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save_fleet_agent_state(path: &Path, state: &FleetAgentState) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}

fn unauthorized(request: &HttpRequest, state: &FleetAgentState) -> bool {
    match (&state.token, &request.authorization) {
        (Some(expected), Some(provided)) => expected != provided,
        (Some(_), None) => true,
        _ => false,
    }
}

pub fn handle_fleet_agent_request(
    state: &mut FleetAgentState,
    request: HttpRequest,
) -> HttpResponse {
    // Route fleet peer relay protocol requests.
    if unauthorized(&request, state) {
        return HttpResponse {
            status: 401,
            body: r#"{"ok":false,"error":"unauthorized"}"#.into(),
        };
    }

    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/v1/health") => HttpResponse {
            status: 200,
            body: r#"{"ok":true,"agent":"spanda-fleet-agent","version":"0.1.0"}"#.into(),
        },
        ("GET", "/v1/status") => HttpResponse {
            status: 200,
            body: serde_json::to_string(&serde_json::json!({
                "ok": true,
                "robot_name": state.robot_name,
                "last_peer_messages": state.last_peer_messages,
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
                state.robot_name = payload.to_robot.clone();
            }
            let message = format!(
                "{}->{}:{}={}",
                payload.from_robot, payload.to_robot, payload.topic, payload.step
            );
            state.last_peer_messages.push(message);
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
            let _ = stream
                .write_all(http_response(400, r#"{"ok":false,"error":"bad request"}"#).as_bytes());
            return;
        }
    };
    let Ok(request) = parse_http_request(&raw) else {
        let _ = stream
            .write_all(http_response(400, r#"{"ok":false,"error":"bad request"}"#).as_bytes());
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
    // Run the fleet peer relay agent until interrupted.
    let mut state = load_fleet_agent_state(state_path);
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
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let state = FleetAgentState {
        robot_name: robot_name.to_string(),
        token,
        ..FleetAgentState::default()
    };
    let shared = Arc::new(Mutex::new(state));
    let handle = thread::spawn(move || {
        for connection in listener.incoming() {
            let Ok(stream) = connection else { continue };
            handle_connection(
                Arc::clone(&shared),
                PathBuf::from(".spanda/test-fleet-agent-state.json"),
                stream,
                None,
            );
        }
    });
    Ok((port, handle))
}

pub fn fleet_entry_for_port(robot_name: &str, port: u16, token: Option<String>) -> FleetAgentEntry {
    FleetAgentEntry {
        robot_name: robot_name.to_string(),
        url: format!("http://127.0.0.1:{port}"),
        token,
    }
}
