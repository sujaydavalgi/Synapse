//! HTTP server for Spanda Control Center.
//!
use crate::handlers::{encode_response, handle_request};
use crate::state::{shared_state, SharedState};
use crate::ws::{is_telemetry_stream_upgrade, serve_telemetry_websocket};
use spanda_deploy_http::parse_http_request;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Options for `spanda control-center serve`.
#[derive(Debug, Clone)]
pub struct ControlCenterOptions {
    pub bind: String,
    pub grpc_bind: Option<String>,
    pub config_path: Option<PathBuf>,
    pub program_path: Option<PathBuf>,
    pub once: bool,
    pub timeout_ms: u64,
}

impl Default for ControlCenterOptions {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1:8080".into(),
            grpc_bind: None,
            config_path: None,
            program_path: None,
            once: false,
            timeout_ms: 0,
        }
    }
}

/// Run the Control Center API until interrupted or `--once` is set.
pub fn run_control_center_server(options: &ControlCenterOptions) -> Result<(), String> {
    let state = shared_state();
    {
        let mut guard = state.lock().map_err(|e| e.to_string())?;
        if let Some(path) = options.config_path.clone() {
            guard.config_path = Some(path);
            guard.reload_config()?;
        }
        if let Some(path) = options.program_path.clone() {
            guard.program_path = Some(path);
        }
    }

    let listener = TcpListener::bind(&options.bind)
        .map_err(|e| format!("bind {} failed: {e}", options.bind))?;
    eprintln!("Spanda Control Center listening on http://{}", options.bind);
    {
        let guard = state.lock().map_err(|e| e.to_string())?;
        if guard.api_keys.keys.is_empty() {
            eprintln!("  warning: no API keys configured — mutations will return 401");
            eprintln!("  generate a key: spanda control-center api-key generate --export");
        } else {
            eprintln!(
                "  API keys loaded: {} (mutations require Bearer token)",
                guard.api_keys.keys.len()
            );
        }
    }
    eprintln!("  GET  /                  Control Center UI");
    eprintln!("  GET  /v1/health         liveness");
    eprintln!("  GET  /v1/version        API version policy");
    eprintln!("  GET  /v1/dashboard      operations summary");
    eprintln!("  GET  /v1/devices        device pool");
    eprintln!("  GET  /v1/fleet/agents   registered fleet agents");
    eprintln!("  GET  /v1/alerts         alert history");
    eprintln!("  POST /v1/alerts/test    test alert (Bearer token)");
    eprintln!("  GET  /v1/tenant          active tenant (SPANDA_TENANT_ID)");
    eprintln!("  GET  /v1/version         API versioning policy");
    eprintln!("  GET  /v1/audit/mutations mutation audit trail (Bearer token)");
    eprintln!("  GET  /v1/openapi.json   OpenAPI 3.1 spec");
    eprintln!("  GET  /v1/drift          operational drift (?baseline_id=)");
    eprintln!("  GET  /v1/sre/summary    SRE availability and incident rollup");
    eprintln!("  GET  /v1/sre/incidents  incident workflow list");
    eprintln!("  POST /v1/sre/incidents  open incident (Bearer token)");
    eprintln!("  POST /v1/ota/plan       canary / phased / blue-green rollout");
    eprintln!("  POST /v1/ota/execute    remote fleet rollout (dry-run supported)");
    eprintln!("  POST /v1/rpc            gRPC-compatible JSON gateway");
    eprintln!("  GET  /v1/digital-thread/query  capability-to-device trace");
    eprintln!("  GET  /v1/entities/traceability unified entity + program trace");
    eprintln!("  GET  /v1/compliance/export     accreditation bundle");
    eprintln!("  WS   /v1/stream/telemetry        live telemetry + traces");
    eprintln!("  GET  /v1/observability/backend     OTLP collector endpoint summary");
    eprintln!("  GET  /v1/observability/otlp/metrics  OTLP metrics preview");
    eprintln!("  POST /v1/observability/otlp/export-metrics  push metrics to collector");
    eprintln!("  POST /v1/observability/otlp/export  push traces to Jaeger");
    if let Some(grpc_bind) = &options.grpc_bind {
        crate::grpc::spawn_grpc_server(grpc_bind.clone(), Arc::clone(&state));
        eprintln!(
            "  gRPC tonic server on {grpc_bind} ({} RPCs — Control Center service)",
            crate::grpc_policy::control_center_rpc_count()
        );
    }
    crate::drift_scheduler::spawn_drift_scheduler(Arc::clone(&state));
    crate::report_scheduler::spawn_report_scheduler(Arc::clone(&state));
    crate::slo_burn_scheduler::spawn_slo_burn_monitor(Arc::clone(&state));
    if std::env::var("SPANDA_DRIFT_SCAN_INTERVAL_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|secs| *secs > 0)
        .is_some()
    {
        eprintln!("  drift scan scheduler active (SPANDA_DRIFT_SCAN_INTERVAL_SECS)");
        eprintln!("  GET  /v1/drift/scans     drift scan history");
        eprintln!("  POST /v1/drift/scan      trigger drift scan (Bearer token)");
    }
    if std::env::var("SPANDA_SRE_BURN_SCAN_INTERVAL_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|secs| *secs > 0)
        .is_some()
    {
        eprintln!("  SLO burn-rate monitor active (SPANDA_SRE_BURN_SCAN_INTERVAL_SECS)");
    }

    if options.once {
        let (stream, _) = listener
            .accept()
            .map_err(|e| format!("accept failed: {e}"))?;
        return serve_connection(stream, Arc::clone(&state), options.timeout_ms);
    }

    for connection in listener.incoming() {
        let Ok(stream) = connection else { continue };
        let state_clone = Arc::clone(&state);
        let timeout = options.timeout_ms;
        thread::spawn(move || {
            let _ = serve_connection(stream, state_clone, timeout);
        });
    }
    Ok(())
}

fn serve_connection(
    mut stream: TcpStream,
    state: SharedState,
    timeout_ms: u64,
) -> Result<(), String> {
    if timeout_ms > 0 {
        let _ = stream.set_read_timeout(Some(Duration::from_millis(timeout_ms)));
    }
    let mut buf = [0u8; 8192];
    let read = stream
        .read(&mut buf)
        .map_err(|e| format!("read request failed: {e}"))?;
    let raw = String::from_utf8_lossy(&buf[..read]);
    let request = parse_http_request(&raw)?;
    let path = request.path.split('?').next().unwrap_or(&request.path);

    if is_telemetry_stream_upgrade(&raw, path) {
        return serve_telemetry_websocket(stream, &buf[..read], state);
    }

    let mut guard = state.lock().map_err(|e| e.to_string())?;
    let (response, correlation_id) = handle_request(&mut guard, &request, &raw);
    let content_type = if path == "/" || path == "/control-center" {
        "text/html; charset=utf-8"
    } else {
        "application/json"
    };
    let encoded = encode_response(&response, content_type, Some(&correlation_id));
    stream
        .write_all(encoded.as_bytes())
        .map_err(|e| format!("write response failed: {e}"))?;
    stream
        .shutdown(Shutdown::Write)
        .map_err(|e| format!("shutdown failed: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::time::Duration;

    #[test]
    fn serve_once_health() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let state = shared_state();
        let server = thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                let _ = serve_connection(stream, state, 5000);
            }
        });
        let mut client = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        client
            .write_all(b"GET /v1/health HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .unwrap();
        let mut body = String::new();
        client.read_to_string(&mut body).unwrap();
        assert!(body.contains("spanda-control-center"));
        server.join().unwrap();
    }
}
