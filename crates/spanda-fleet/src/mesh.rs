//! Multi-host fleet mesh coordinator for centralized peer relay routing.
//!
use crate::continuity_mesh::handle_fleet_continuity_post;
use crate::recovery_mesh::handle_fleet_recovery_post;
use crate::remote::{
    default_fleet_agents_path, load_fleet_agent_registry, relay_peer_deliveries, FleetAgentRegistry,
};
use crate::tamper_mesh::{handle_fleet_tamper_get, handle_fleet_tamper_ingest_post};
use crate::telemetry_mesh::{handle_fleet_telemetry_get, handle_fleet_telemetry_ingest_post};
use crate::PeerDelivery;
use serde::{Deserialize, Serialize};
use spanda_deploy_http::FleetContinuityResponse;
use spanda_deploy_http::FleetRecoveryResponse;
use spanda_deploy_http::{
    http_request, parse_http_request, read_plain_request, serve_tls_connection,
    write_plain_response, DeployAgentTls, HttpRequest, HttpResponse,
};
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
    // Description:
    //     Default fleet mesh state path.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: PathBuf
    //         Return value from `default_fleet_mesh_state_path`.
    //
    // Example:

    //     let result = spanda_fleet::mesh::default_fleet_mesh_state_path();

    PathBuf::from(".spanda/fleet-mesh-state.json")
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetMeshState {
    pub relayed_total: u32,
    pub failed_total: u32,
    pub recovery_relayed_total: u32,
    pub recovery_failed_total: u32,
    pub continuity_relayed_total: u32,
    pub continuity_failed_total: u32,
    pub telemetry_ingest_total: u32,
    pub tamper_ingest_total: u32,
    #[serde(default)]
    pub telemetry_shards: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub tamper_shards: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub tamper_fleet_name: String,
    #[serde(default)]
    pub token: Option<String>,
}

#[derive(Clone)]
pub enum MeshRegistryBacking {
    Path(Arc<PathBuf>),
    Memory(Arc<FleetAgentRegistry>),
}

fn unauthorized(request: &HttpRequest, state: &FleetMeshState) -> bool {
    // Description:
    //     Unauthorized.
    //
    // Inputs:
    //     request: &HttpRequest
    //         Caller-supplied request.
    //     state: &FleetMeshState
    //         Caller-supplied state.
    //
    // Outputs:
    //     result: bool
    //         Return value from `unauthorized`.
    //
    // Example:

    //     let result = spanda_fleet::mesh::unauthorized(reques, state);

    match (&state.token, &request.authorization) {
        (Some(expected), Some(provided)) => expected != provided,
        (Some(_), None) => true,
        _ => false,
    }
}

fn load_registry(backing: &MeshRegistryBacking) -> FleetAgentRegistry {
    // Description:
    //     Load registry.
    //
    // Inputs:
    //     backing: &MeshRegistryBacking
    //         Caller-supplied backing.
    //
    // Outputs:
    //     result: FleetAgentRegistry
    //         Return value from `load_registry`.
    //
    // Example:

    //     let result = spanda_fleet::mesh::load_registry(backing);

    match backing {
        MeshRegistryBacking::Path(path) => load_fleet_agent_registry(path),
        MeshRegistryBacking::Memory(registry) => (**registry).clone(),
    }
}

fn mesh_relay_http_response(relayed: u32, failed: u32) -> HttpResponse {
    // Description:
    //     Mesh relay http response.
    //
    // Inputs:
    //     relayed: u32
    //         Caller-supplied relayed.
    //     failed: u32
    //         Caller-supplied failed.
    //
    // Outputs:
    //     result: HttpResponse
    //         Return value from `mesh_relay_http_response`.
    //
    // Example:

    //     let result = spanda_fleet::mesh::mesh_relay_http_response(relayed, failed);

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

fn finish_mesh_relay(
    state: &mut FleetMeshState,
    registry_backing: &MeshRegistryBacking,
    deliveries: &[PeerDelivery],
) -> HttpResponse {
    // Description:
    //     Finish mesh relay.
    //
    // Inputs:
    //     state: &mut FleetMeshState
    //         Caller-supplied state.
    //     registry_backing: &MeshRegistryBacking
    //         Caller-supplied registry backing.
    //     deliveries: &[PeerDelivery]
    //         Caller-supplied deliveries.
    //
    // Outputs:
    //     result: HttpResponse
    //         Return value from `finish_mesh_relay`.
    //
    // Example:

    //     let result = spanda_fleet::mesh::finish_mesh_relay(state, registry_backing, deliveries);

    let registry = load_registry(registry_backing);
    let (relayed, failed) = relay_peer_deliveries(deliveries, &registry);
    state.relayed_total += relayed;
    state.failed_total += failed;
    mesh_relay_http_response(relayed, failed)
}

pub fn handle_fleet_mesh_request(
    state: &mut FleetMeshState,
    registry_backing: &MeshRegistryBacking,
    request: HttpRequest,
) -> HttpResponse {
    // Description:

    //     Handle fleet mesh request.

    //

    // Inputs:

    //     state: &mut FleetMeshState

    //         Caller-supplied state.

    //     registry_backing: &MeshRegistryBacking

    //         Caller-supplied registry backing.

    //     request: HttpRequest

    //         Caller-supplied request.

    //

    // Outputs:

    //     result: HttpResponse

    //         Return value from `handle_fleet_mesh_request`.

    //

    // Example:

    //     let result = spanda_fleet::mesh::handle_fleet_mesh_request(state, registry_backing, reques);
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
                    "recovery_relayed_total": state.recovery_relayed_total,
                    "recovery_failed_total": state.recovery_failed_total,
                    "continuity_relayed_total": state.continuity_relayed_total,
                    "continuity_failed_total": state.continuity_failed_total,
                    "telemetry_ingest_total": state.telemetry_ingest_total,
                    "telemetry_robots": state.telemetry_shards.len(),
                    "tamper_ingest_total": state.tamper_ingest_total,
                    "tamper_robots": state.tamper_shards.len(),
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
            finish_mesh_relay(state, registry_backing, &payload.deliveries)
        }
        ("POST", "/v1/fleet/recovery") => {
            let registry = load_registry(registry_backing);
            let response = handle_fleet_recovery_post(&request.body, &registry);
            if let Ok(payload) = serde_json::from_str::<FleetRecoveryResponse>(&response.body) {
                state.recovery_relayed_total += payload.relayed;
                state.recovery_failed_total += payload.failed;
            }
            response
        }
        ("POST", "/v1/fleet/continuity") => {
            let registry = load_registry(registry_backing);
            let response = handle_fleet_continuity_post(&request.body, &registry);
            if let Ok(payload) = serde_json::from_str::<FleetContinuityResponse>(&response.body) {
                state.continuity_relayed_total += payload.relayed;
                state.continuity_failed_total += payload.failed;
            }
            response
        }
        ("GET", "/v1/fleet/telemetry") => handle_fleet_telemetry_get(state),
        ("POST", "/v1/fleet/telemetry/ingest") => {
            handle_fleet_telemetry_ingest_post(&request.body, state)
        }
        ("GET", path) if path.starts_with("/v1/fleet/tamper") => {
            handle_fleet_tamper_get(path, state)
        }
        ("POST", "/v1/fleet/tamper/ingest") => {
            handle_fleet_tamper_ingest_post(&request.body, state)
        }
        _ => HttpResponse {
            status: 404,
            body: r#"{"ok":false,"error":"not found"}"#.into(),
        },
    }
}

fn dispatch_mesh_request(
    state: Arc<Mutex<FleetMeshState>>,
    registry_backing: &MeshRegistryBacking,
    request: HttpRequest,
) -> HttpResponse {
    // Description:

    //     Dispatch mesh request.

    //

    // Inputs:

    //     state: Arc<Mutex<FleetMeshState>>

    //         Caller-supplied state.

    //     registry_backing: &MeshRegistryBacking

    //         Caller-supplied registry backing.

    //     request: HttpRequest

    //         Caller-supplied request.

    //

    // Outputs:

    //     result: HttpResponse

    //         Return value from `dispatch_mesh_request`.

    //

    // Example:

    //     let result = spanda_fleet::mesh::dispatch_mesh_request(state, registry_backing, reques);
    if request.method == "POST" && request.path == "/v1/mesh/relay" {
        let deliveries = {
            let locked = state.lock().expect("fleet mesh state lock");
            if unauthorized(&request, &locked) {
                return HttpResponse {
                    status: 401,
                    body: r#"{"ok":false,"error":"unauthorized"}"#.into(),
                };
            }
            let Ok(payload) = serde_json::from_str::<MeshRelayRequest>(&request.body) else {
                return HttpResponse {
                    status: 400,
                    body: r#"{"ok":false,"error":"invalid mesh relay payload"}"#.into(),
                };
            };
            payload.deliveries
        };
        let registry = load_registry(registry_backing);
        let (relayed, failed) = relay_peer_deliveries(&deliveries, &registry);
        let mut locked = state.lock().expect("fleet mesh state lock");
        locked.relayed_total += relayed;
        locked.failed_total += failed;
        return mesh_relay_http_response(relayed, failed);
    }

    if request.method == "POST" && request.path == "/v1/fleet/recovery" {
        let locked = state.lock().expect("fleet mesh state lock");
        if unauthorized(&request, &locked) {
            return HttpResponse {
                status: 401,
                body: r#"{"ok":false,"error":"unauthorized"}"#.into(),
            };
        }
        drop(locked);
        let registry = load_registry(registry_backing);
        let response = handle_fleet_recovery_post(&request.body, &registry);
        if let Ok(payload) = serde_json::from_str::<FleetRecoveryResponse>(&response.body) {
            let mut locked = state.lock().expect("fleet mesh state lock");
            locked.recovery_relayed_total += payload.relayed;
            locked.recovery_failed_total += payload.failed;
        }
        return response;
    }

    if request.method == "POST" && request.path == "/v1/fleet/continuity" {
        let locked = state.lock().expect("fleet mesh state lock");
        if unauthorized(&request, &locked) {
            return HttpResponse {
                status: 401,
                body: r#"{"ok":false,"error":"unauthorized"}"#.into(),
            };
        }
        drop(locked);
        let registry = load_registry(registry_backing);
        let response = handle_fleet_continuity_post(&request.body, &registry);
        if let Ok(payload) = serde_json::from_str::<FleetContinuityResponse>(&response.body) {
            let mut locked = state.lock().expect("fleet mesh state lock");
            locked.continuity_relayed_total += payload.relayed;
            locked.continuity_failed_total += payload.failed;
        }
        return response;
    }

    if request.method == "GET" && request.path == "/v1/fleet/telemetry" {
        let locked = state.lock().expect("fleet mesh state lock");
        if unauthorized(&request, &locked) {
            return HttpResponse {
                status: 401,
                body: r#"{"ok":false,"error":"unauthorized"}"#.into(),
            };
        }
        return handle_fleet_telemetry_get(&locked);
    }

    if request.method == "POST" && request.path == "/v1/fleet/telemetry/ingest" {
        let mut locked = state.lock().expect("fleet mesh state lock");
        if unauthorized(&request, &locked) {
            return HttpResponse {
                status: 401,
                body: r#"{"ok":false,"error":"unauthorized"}"#.into(),
            };
        }
        return handle_fleet_telemetry_ingest_post(&request.body, &mut locked);
    }

    let mut locked = state.lock().expect("fleet mesh state lock");
    handle_fleet_mesh_request(&mut locked, registry_backing, request)
}

fn handle_connection(
    state: Arc<Mutex<FleetMeshState>>,
    registry_backing: MeshRegistryBacking,
    mut stream: TcpStream,
    tls: Option<Arc<rustls::ServerConfig>>,
) {
    // Description:
    //     Handle connection.
    //
    // Inputs:
    //     state: Arc<Mutex<FleetMeshState>>
    //         Caller-supplied state.
    //     registry_backing: MeshRegistryBacking
    //         Caller-supplied registry backing.
    //     strea: TcpStream
    //         Caller-supplied strea.
    //     ls: Option<Arc<rustls::ServerConfig>>
    //         Caller-supplied ls.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_fleet::mesh::handle_connection(state, registry_backing, strea, ls);

    if let Some(server_config) = tls {
        let shared = Arc::clone(&state);
        let backing = registry_backing.clone();
        let _ = serve_tls_connection(&server_config, stream, move |request| {
            dispatch_mesh_request(Arc::clone(&shared), &backing, request)
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
    let response = dispatch_mesh_request(state, &registry_backing, request);
    let _ = write_plain_response(&mut stream, &response);
}

pub fn run_fleet_mesh_coordinator(
    bind: &str,
    registry_path: &Path,
    token: Option<String>,
    tls: Option<DeployAgentTls>,
) -> Result<(), String> {
    // Description:

    //     Run fleet mesh coordinator.

    //

    // Inputs:

    //     bind: &str

    //         Caller-supplied bind.

    //     registry_path: &Path

    //         Caller-supplied registry path.

    //     token: Option<String>

    //         Caller-supplied token.

    //     ls: Option<DeployAgentTls>

    //         Caller-supplied ls.

    //

    // Outputs:

    //     result: Result<(), String>

    //         Return value from `run_fleet_mesh_coordinator`.

    //

    // Example:

    //     let result = spanda_fleet::mesh::run_fleet_mesh_coordinator(bind, registry_path, oken, ls);
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
        let shared_state = Arc::clone(&shared_state);
        let registry_backing = registry_backing.clone();
        let server_config = server_config.clone();
        thread::spawn(move || {
            handle_connection(shared_state, registry_backing, stream, server_config);
        });
    }
    Ok(())
}

pub fn relay_deliveries_via_mesh(
    mesh_url: &str,
    deliveries: &[PeerDelivery],
    token: Option<&str>,
) -> Result<MeshRelayResponse, String> {
    // Description:

    //     Relay deliveries via mesh.

    //

    // Inputs:

    //     mesh_url: &str

    //         Caller-supplied mesh url.

    //     deliveries: &[PeerDelivery]

    //         Caller-supplied deliveries.

    //     token: Option<&str>

    //         Caller-supplied token.

    //

    // Outputs:

    //     result: Result<MeshRelayResponse, String>

    //         Return value from `relay_deliveries_via_mesh`.

    //

    // Example:

    //     let result = spanda_fleet::mesh::relay_deliveries_via_mesh(esh_url, deliveries, oken);
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
    // Description:
    //     Mesh registry path.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: PathBuf
    //         Return value from `mesh_registry_path`.
    //
    // Example:

    //     let result = spanda_fleet::mesh::mesh_registry_path();

    std::env::var("SPANDA_FLEET_AGENTS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_fleet_agents_path())
}

pub fn spawn_test_fleet_mesh(
    registry: &FleetAgentRegistry,
) -> Result<(u16, thread::JoinHandle<()>), String> {
    // Description:
    //     Spawn test fleet mesh.
    //
    // Inputs:
    //     registry: &FleetAgentRegistry
    //         Caller-supplied registry.
    //
    // Outputs:
    //     result: Result<(u16, thread::JoinHandle<()>), String>
    //         Return value from `spawn_test_fleet_mesh`.
    //
    // Example:

    //     let result = spanda_fleet::mesh::spawn_test_fleet_mesh(registry);

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
