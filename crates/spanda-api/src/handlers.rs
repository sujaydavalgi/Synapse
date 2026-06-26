//! REST v1 route handlers for Spanda Control Center.
//!
use crate::correlation::{correlation_from_headers, new_correlation_id};
use crate::e3;
use crate::e4;
use crate::state::ControlCenterState;
use serde::Serialize;
use spanda_config::{
    default_snapshots_dir, list_config_snapshots, run_provision_workflow, save_config_snapshot,
    DeviceDiscoveryTransport, DeviceLifecycleState, DiscoveryOptions, MockMdnsDiscoveryTransport,
    SubnetDiscoveryTransport,
};
use spanda_deploy_http::{HttpRequest, HttpResponse};
use spanda_fleet::remote::{default_fleet_agents_path, load_fleet_agent_registry};
use spanda_ops::{Alert, AlertSeverity, AlertType};
use spanda_security::{ApiKeyStore, RbacAction, RbacContext};

const API_VERSION: &str = "v1";

#[derive(Serialize)]
struct HealthPayload {
    ok: bool,
    version: &'static str,
    service: &'static str,
}

#[derive(Serialize)]
struct DashboardPayload {
    version: &'static str,
    device_pool: spanda_config::DevicePoolSummary,
    fleet_agent_count: usize,
    alert_count: usize,
    rbac_roles: usize,
}

pub fn handle_request(
    state: &mut ControlCenterState,
    request: &HttpRequest,
    raw_headers: &str,
) -> (HttpResponse, String) {
    let started_ms = now_ms();
    let correlation_id =
        correlation_from_headers(raw_headers).unwrap_or_else(new_correlation_id);
    let (path, query) = match request.path.split_once('?') {
        Some((p, q)) => (p, q),
        None => (request.path.as_str(), ""),
    };
    if request.method == "OPTIONS" {
        return (cors_preflight(), correlation_id);
    }
    if path == "/health" || path == "/v1/health" || path == "/healthz" {
        let response = json_ok(&HealthPayload {
            ok: true,
            version: API_VERSION,
            service: "spanda-control-center",
        });
        e3::record_trace(state, &correlation_id, &request.method, path, response.status, started_ms);
        return (response, correlation_id);
    }
    if path == "/" || path == "/control-center" {
        let response = html_ok(include_str!("static/control-center.html"));
        e3::record_trace(state, &correlation_id, &request.method, path, response.status, started_ms);
        return (response, correlation_id);
    }
    if path == "/v1/openapi.json" {
        let response = e3::openapi_spec();
        e3::record_trace(state, &correlation_id, &request.method, path, response.status, started_ms);
        return (response, correlation_id);
    }
    if !path.starts_with("/v1/") {
        let response = not_found();
        e3::record_trace(state, &correlation_id, &request.method, path, response.status, started_ms);
        return (response, correlation_id);
    }
    let ctx = state
        .api_keys
        .authenticate(request.authorization.as_deref());
    if path.starts_with("/v1/devices/") && request.method == "PATCH" {
        let id = path.trim_start_matches("/v1/devices/");
        let response = device_patch(state, id, &request.body, ctx.as_ref());
        e3::record_trace(state, &correlation_id, &request.method, path, response.status, started_ms);
        return (response, correlation_id);
    }
    let response = match (path, request.method.as_str()) {
        ("/v1/dashboard", "GET") => dashboard(state),
        ("/v1/devices", "GET") => devices_list(state),
        ("/v1/fleet/agents", "GET") => fleet_agents(),
        ("/v1/alerts", "GET") => alerts_list(state),
        ("/v1/alerts/test", "POST") => alerts_test(state, ctx.as_ref()),
        ("/v1/secrets", "GET") => secrets_list(state, ctx.as_ref()),
        ("/v1/rbac/matrix", "GET") => rbac_matrix(),
        ("/v1/provision", "POST") => provision_run(state, &request.body, ctx.as_ref()),
        ("/v1/config/snapshots", "GET") => config_snapshots_list(),
        ("/v1/config/snapshots", "POST") => config_snapshots_save(state, &request.body, ctx.as_ref()),
        ("/v1/discovery", "GET") => discovery_run(query),
        ("/v1/health/summary", "GET") => health_summary(state),
        ("/v1/assurance/summary", "GET") => assurance_summary(state),
        ("/v1/diagnosis/summary", "GET") => diagnosis_summary(state),
        ("/v1/drift", "GET") => e3::drift_report(state, query),
        ("/v1/ota/status", "GET") => e3::ota_status(),
        ("/v1/ota/plan", "POST") => e3::ota_plan(&request.body, ctx.as_ref()),
        ("/v1/trust/package", "GET") => e3::trust_package(query),
        ("/v1/sre/summary", "GET") => e3::sre_summary(state),
        ("/v1/observability/traces", "GET") => e3::observability_traces(state),
        ("/v1/operator/quarantine", "POST") => {
            e3::operator_quarantine(state, &request.body, ctx.as_ref())
        }
        ("/v1/operator/mission/approve", "POST") => {
            e3::operator_mission_approve(&request.body, ctx.as_ref())
        }
        ("/v1/rpc", "POST") => e3::rpc_gateway(state, &request.body),
        ("/v1/compliance/export", "GET") => {
            e4::compliance_export(state, query, None, ctx.as_ref())
        }
        ("/v1/compliance/export", "POST") => {
            e4::compliance_export(state, query, Some(&request.body), ctx.as_ref())
        }
        ("/v1/digital-thread/query", "GET") => e4::digital_thread_query(state, query),
        ("/v1/executive/scorecard", "GET") => e4::executive_scorecard(state),
        ("/v1/analytics/readiness", "GET") => e4::analytics_readiness(state, query),
        ("/v1/reports/export", "GET") => e4::reports_export(state, query, ctx.as_ref()),
        _ => not_found(),
    };
    e3::record_trace(
        state,
        &correlation_id,
        &request.method,
        path,
        response.status,
        started_ms,
    );
    (response, correlation_id)
}

fn dashboard(state: &ControlCenterState) -> HttpResponse {
    let registry = state.device_registry();
    let fleet = load_fleet_agent_registry(&default_fleet_agents_path());
    json_ok(&DashboardPayload {
        version: API_VERSION,
        device_pool: registry.pool_summary(),
        fleet_agent_count: fleet.agents.len(),
        alert_count: state.alert_store.list().len(),
        rbac_roles: state.api_keys.keys.len(),
    })
}

fn devices_list(state: &ControlCenterState) -> HttpResponse {
    let registry = state.device_registry();
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "devices": registry.pool_entries(),
    }))
}

fn device_patch(
    state: &mut ControlCenterState,
    device_id: &str,
    body: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Provision) {
        return unauthorized();
    }
    let payload: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => return bad_request(&e.to_string()),
    };
    let Some(state_str) = payload.get("lifecycle_state").and_then(|v| v.as_str()) else {
        return bad_request("missing lifecycle_state");
    };
    let lifecycle = DeviceLifecycleState::parse(state_str);
    let mut registry = state.device_registry();
    if let Err(e) = registry.set_lifecycle(device_id, lifecycle) {
        return bad_request(&e);
    }
    if let Some(resolved) = state.resolved.as_mut() {
        resolved.device_registry = registry;
    }
    json_ok(&serde_json::json!({
        "ok": true,
        "device_id": device_id,
        "lifecycle_state": lifecycle.as_str(),
    }))
}

fn fleet_agents() -> HttpResponse {
    let fleet = load_fleet_agent_registry(&default_fleet_agents_path());
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "agents": fleet.agents,
    }))
}

fn alerts_list(state: &ControlCenterState) -> HttpResponse {
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "alerts": state.alert_store.list_owned(),
    }))
}

fn alerts_test(state: &mut ControlCenterState, ctx: Option<&RbacContext>) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Operate) {
        return unauthorized();
    }
    let mut alert = Alert {
        id: format!("test-{}", now_ms()),
        alert_type: AlertType::Custom,
        severity: AlertSeverity::Info,
        message: "Control Center alert test".into(),
        source: "control-center".into(),
        timestamp_ms: now_ms(),
        delivered_via: vec![],
    };
    state.alert_dispatcher.dispatch(&mut alert);
    state.alert_store.push(alert.clone());
    json_ok(&serde_json::json!({
        "ok": true,
        "alert": alert,
    }))
}

fn secrets_list(state: &ControlCenterState, ctx: Option<&RbacContext>) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Deploy) {
        return unauthorized();
    }
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "secrets": state.secret_vault.list_metadata(),
    }))
}

fn rbac_matrix() -> HttpResponse {
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "matrix": spanda_security::permission_matrix(),
    }))
}

fn provision_run(
    state: &mut ControlCenterState,
    body: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Provision) {
        return unauthorized();
    }
    let payload: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => return bad_request(&e.to_string()),
    };
    let Some(device_id) = payload.get("device_id").and_then(|v| v.as_str()) else {
        return bad_request("missing device_id");
    };
    let robot_id = payload.get("robot_id").and_then(|v| v.as_str());
    let registry = state.device_registry();
    let report = run_provision_workflow(device_id, &registry, robot_id);
    if !report.ready {
        let mut alert = Alert {
            id: format!("provision-{}", now_ms()),
            alert_type: AlertType::ReadinessFailed,
            severity: AlertSeverity::Warning,
            message: format!("provisioning failed for device '{device_id}'"),
            source: "provisioning".into(),
            timestamp_ms: now_ms(),
            delivered_via: vec![],
        };
        state.alert_dispatcher.dispatch(&mut alert);
        state.alert_store.push(alert);
        if let Some(resolved) = state.resolved.as_mut() {
            if let Some(device) = resolved
                .device_registry
                .devices
                .iter_mut()
                .find(|d| d.id == device_id)
            {
                device.lifecycle_state = Some(DeviceLifecycleState::Quarantined.as_str().to_string());
            }
        }
    } else if let Some(resolved) = state.resolved.as_mut() {
        if let Some(device) = resolved
            .device_registry
            .devices
            .iter_mut()
            .find(|d| d.id == device_id)
        {
            device.lifecycle_state = Some(DeviceLifecycleState::Healthy.as_str().to_string());
            if let Some(robot) = robot_id {
                device.assigned_robot = Some(robot.to_string());
            }
        }
    }
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "ok": report.ready,
        "report": report,
    }))
}

fn config_snapshots_list() -> HttpResponse {
    let dir = default_snapshots_dir();
    let snapshots = list_config_snapshots(&dir).unwrap_or_default();
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "snapshots": snapshots,
    }))
}

fn config_snapshots_save(
    state: &ControlCenterState,
    body: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Deploy) {
        return unauthorized();
    }
    let Some(resolved) = state.resolved.as_ref() else {
        return bad_request("no resolved configuration loaded; use --config");
    };
    let label = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("label").and_then(|l| l.as_str()).map(str::to_string));
    let dir = default_snapshots_dir();
    match save_config_snapshot(resolved, &dir, label) {
        Ok(meta) => json_ok(&serde_json::json!({
            "version": API_VERSION,
            "ok": true,
            "snapshot": meta,
        })),
        Err(e) => bad_request(&e.to_string()),
    }
}

fn discovery_run(query: &str) -> HttpResponse {
    let params = parse_query(query);
    let transport = params.get("transport").map(String::as_str).unwrap_or("subnet");
    let options = DiscoveryOptions {
        subnet: params.get("subnet").cloned(),
        timeout_ms: params.get("timeout_ms").and_then(|v| v.parse().ok()),
        transports: vec![transport.to_string()],
    };
    let result = match transport {
        "mdns" => MockMdnsDiscoveryTransport.discover(&options),
        _ => SubnetDiscoveryTransport.discover(&options),
    };
    match result {
        Ok(discovery) => json_ok(&serde_json::json!({
            "version": API_VERSION,
            "discovery": discovery,
        })),
        Err(message) => bad_request(&message),
    }
}

fn health_summary(state: &ControlCenterState) -> HttpResponse {
    let pool = state.device_registry().pool_summary();
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "overall_status": if pool.failed > 0 { "critical" } else if pool.degraded > 0 { "degraded" } else { "healthy" },
        "device_pool": pool,
    }))
}

fn assurance_summary(state: &ControlCenterState) -> HttpResponse {
    let Some(resolved) = state.resolved.as_ref() else {
        return json_ok(&serde_json::json!({
            "version": API_VERSION,
            "loaded": false,
        }));
    };
    let policy = spanda_config::assurance_policy(resolved);
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "loaded": true,
        "minimum_score": policy.minimum_score,
        "require_recovery": policy.require_recovery,
        "require_resilience": policy.require_resilience,
    }))
}

fn diagnosis_summary(state: &ControlCenterState) -> HttpResponse {
    let Some(resolved) = state.resolved.as_ref() else {
        return json_ok(&serde_json::json!({
            "version": API_VERSION,
            "loaded": false,
        }));
    };
    let policy = spanda_config::diagnosis_policy(resolved);
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "loaded": true,
        "require_mitigations": policy.require_mitigations,
        "require_anomaly_handlers": policy.require_anomaly_handlers,
    }))
}

pub(crate) fn parse_query(query: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for pair in query.split('&').filter(|s| !s.is_empty()) {
        if let Some((k, v)) = pair.split_once('=') {
            map.insert(k.to_string(), v.to_string());
        }
    }
    map
}

pub(crate) fn json_ok<T: Serialize>(value: &T) -> HttpResponse {
    let body = serde_json::to_string(value).unwrap_or_else(|_| "{}".into());
    HttpResponse {
        status: 200,
        body,
    }
}

fn html_ok(html: &str) -> HttpResponse {
    HttpResponse {
        status: 200,
        body: html.to_string(),
    }
}

pub(crate) fn bad_request(message: &str) -> HttpResponse {
    HttpResponse {
        status: 400,
        body: serde_json::json!({ "ok": false, "error": message }).to_string(),
    }
}

pub(crate) fn unauthorized() -> HttpResponse {
    HttpResponse {
        status: 401,
        body: serde_json::json!({ "ok": false, "error": "unauthorized" }).to_string(),
    }
}

fn not_found() -> HttpResponse {
    HttpResponse {
        status: 404,
        body: serde_json::json!({ "ok": false, "error": "not found" }).to_string(),
    }
}

fn cors_preflight() -> HttpResponse {
    HttpResponse {
        status: 204,
        body: String::new(),
    }
}

pub(crate) fn now_ms() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64() * 1000.0)
        .unwrap_or(0.0)
}

pub fn encode_response(
    response: &HttpResponse,
    content_type: &str,
    correlation_id: Option<&str>,
) -> String {
    if response.body.starts_with("HTTP/1.1") {
        return response.body.clone();
    }
    let status_text = match response.status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        _ => "Error",
    };
    let correlation_header = correlation_id
        .map(|id| format!("X-Correlation-ID: {id}\r\n"))
        .unwrap_or_default();
    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: Authorization, Content-Type, X-Correlation-ID\r\n{}Access-Control-Expose-Headers: X-Correlation-ID\r\n\r\n{}",
        response.status,
        status_text,
        content_type,
        response.body.len(),
        correlation_header,
        response.body
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_endpoint_ok() {
        let mut state = ControlCenterState::new();
        let (response, _) = handle_request(
            &mut state,
            &HttpRequest {
                method: "GET".into(),
                path: "/v1/health".into(),
                body: String::new(),
                authorization: None,
            },
            "",
        );
        assert_eq!(response.status, 200);
        assert!(response.body.contains("spanda-control-center"));
    }
}
