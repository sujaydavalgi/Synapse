//! Multi-host fleet mesh coordinator for centralized peer relay routing.
//!
use crate::remote::{
    default_fleet_agents_path, load_fleet_agent_registry, relay_peer_deliveries, FleetAgentRegistry,
};
use crate::PeerDelivery;
use serde::{Deserialize, Serialize};
use spanda_deploy_http::{
    http_request, http_response, parse_http_request, read_plain_request, serve_tls_connection,
    write_plain_response, DeployAgentTls, HttpRequest, HttpResponse,
};
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshRelayRequest {
    pub deliveries: Vec<PeerDelivery>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshRelayResponse {
    pub ok: bool,
    pub relayed: u32,
    pub failed: u32,
    #[serde(default)]
    pub error: Option<String>,
}

pub fn default_fleet_mesh_state_path() -> PathBuf {
    PathBuf::from(".spanda/fleet-mesh-state.json")
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetMeshState {
    pub relayed_total: u32,
    pub failed_total: u32,
    #[serde(default)]
    pub token: Option<String>,
}

#[derive(Clone)]
pub enum MeshRegistryBacking {
    Path(Arc<PathBuf>),
    Memory(Arc<FleetAgentRegistry>),
}

fn unauthorized(request: &HttpRequest, state: &FleetMeshState) -> bool {
    match (&state.token, &request.authorization) {
        (Some(expected), Some(provided)) => expected != provided,
        (Some(_), None) => true,
        _ => false,
    }
}

fn load_registry(backing: &MeshRegistryBacking) -> FleetAgentRegistry {
    match backing {
        MeshRegistryBacking::Path(path) => load_fleet_agent_registry(path),
        MeshRegistryBacking::Memory(registry) => (**registry).clone(),
    }
}

pub fn handle_fleet_mesh_request(
    state: &mut FleetMeshState,
    registry_backing: &MeshRegistryBacking,
    request: HttpRequest,
) -> HttpResponse {
    // Route mesh coordinator relay requests to registered fleet agents.
    if unauthorized(&request, state) {
        return HttpResponse {
            status: 401,
            body: r#"{"ok":false,"error":"unauthorized"}"#.into(),
        };
    }

    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/v1/health") => HttpResponse {
            status: 200,
            body: r#"{"ok":true,"agent":"spanda-fleet-mesh","version":"0.1.0"}"#.into(),
        },
        ("GET", "/v1/status") => {
            let registry = load_registry(registry_backing);
            HttpResponse {
                status: 200,
                body: serde_json::to_string(&serde_json::json!({
                    "ok": true,
                    "agents": registry.agents.len(),
                    "relayed_total": state.relayed_total,
                    "failed_total": state.failed_total,
                    "healthy": true,
                }))
                .unwrap_or_else(|_| "{}".into()),
            }
        }
        ("POST", "/v1/mesh/relay") => {
            let Ok(payload) = serde_json::from_str::<MeshRelayRequest>(&request.body) else {
                return HttpResponse {
                    status: 400,
                    body: r#"{"ok":false,"error":"invalid mesh relay payload"}"#.into(),
                };
            };
            let registry = load_registry(registry_backing);
            let (relayed, failed) = relay_peer_deliveries(&payload.deliveries, &registry);
            state.relayed_total += relayed;
            state.failed_total += failed;
            let ok = failed == 0;
            HttpResponse {
                status: 200,
                body: serde_json::to_string(&MeshRelayResponse {
                    ok,
                    relayed,
                    failed,
                    error: if failed > 0 {
                        Some(format!("{failed} peer relay(s) failed"))
                    } else {
                        None
                    },
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
    state: Arc<Mutex<FleetMeshState>>,
    registry_backing: MeshRegistryBacking,
    mut stream: TcpStream,
    tls: Option<Arc<rustls::ServerConfig>>,
) {
    if let Some(server_config) = tls {
        let shared = Arc::clone(&state);
        let backing = registry_backing.clone();
        let _ = serve_tls_connection(&server_config, stream, move |request| {
            let mut locked = shared.lock().expect("fleet mesh state lock");
            handle_fleet_mesh_request(&mut locked, &backing, request)
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
        let mut locked = state.lock().expect("fleet mesh state lock");
        handle_fleet_mesh_request(&mut locked, &registry_backing, request)
    };
    let _ = write_plain_response(&mut stream, &response);
}

pub fn run_fleet_mesh_coordinator(
    bind: &str,
    registry_path: &Path,
    token: Option<String>,
    tls: Option<DeployAgentTls>,
) -> Result<(), String> {
    // Run the fleet mesh coordinator until interrupted.
    let registry = load_fleet_agent_registry(registry_path);
    let state = FleetMeshState {
        token,
        ..FleetMeshState::default()
    };
    let listener = TcpListener::bind(bind).map_err(|e| format!("bind {bind} failed: {e}"))?;
    let shared_state = Arc::new(Mutex::new(state));
    let registry_backing = MeshRegistryBacking::Path(Arc::new(registry_path.to_path_buf()));
    let server_config = tls
        .as_ref()
        .map(spanda_deploy_http::build_deploy_server_config)
        .transpose()?;
    let scheme = if server_config.is_some() {
        "https"
    } else {
        "http"
    };
    eprintln!(
        "Spanda fleet mesh listening on {scheme}://{bind} ({} agents)",
        registry.agents.len()
    );
    for connection in listener.incoming() {
        let Ok(stream) = connection else { continue };
        handle_connection(
            Arc::clone(&shared_state),
            registry_backing.clone(),
            stream,
            server_config.clone(),
        );
    }
    Ok(())
}

pub fn relay_deliveries_via_mesh(
    mesh_url: &str,
    deliveries: &[PeerDelivery],
    token: Option<&str>,
) -> Result<MeshRelayResponse, String> {
    // Send peer deliveries to a fleet mesh coordinator endpoint.
    let parsed = spanda_deploy_http::parse_http_url(mesh_url)?;
    let url = format!(
        "{}://{}:{}/v1/mesh/relay",
        parsed.scheme, parsed.host, parsed.port
    );
    let payload = serde_json::to_string(&MeshRelayRequest {
        deliveries: deliveries.to_vec(),
    })
    .map_err(|e| e.to_string())?;
    let response = http_request("POST", &url, Some(&payload), token)?;
    if response.status >= 400 {
        return Err(format!(
            "fleet mesh HTTP {}: {}",
            response.status, response.body
        ));
    }
    serde_json::from_str(&response.body).map_err(|e| format!("invalid fleet mesh JSON: {e}"))
}

pub fn mesh_registry_path() -> PathBuf {
    std::env::var("SPANDA_FLEET_AGENTS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_fleet_agents_path())
}

pub fn spawn_test_fleet_mesh(
    registry: &FleetAgentRegistry,
) -> Result<(u16, thread::JoinHandle<()>), String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let shared_registry = Arc::new(registry.clone());
    let shared_state = Arc::new(Mutex::new(FleetMeshState::default()));
    let registry_backing = MeshRegistryBacking::Memory(shared_registry);
    let handle = thread::spawn(move || {
        for connection in listener.incoming() {
            let Ok(stream) = connection else { continue };
            handle_connection(
                Arc::clone(&shared_state),
                registry_backing.clone(),
                stream,
                None,
            );
        }
    });
    Ok((port, handle))
}
