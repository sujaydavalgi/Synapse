//! REST v1 route handlers for Spanda Control Center.
//!
use crate::correlation::{correlation_from_headers, new_correlation_id};
use crate::e3;
use crate::e4;
use crate::observability;
use crate::state::ControlCenterState;
use serde::Serialize;
use spanda_config::{
    default_snapshots_dir, export_device_mapping_json, failover_chains, generate_device_reports,
    ingest_discovery_matches, list_config_snapshots, readiness_impact, run_discovery_transports,
    run_provision_workflow, save_config_snapshot, AssignDeviceOptions, DeviceDiscoveryTransport,
    DeviceLifecycleState, DiscoveryOptions, SubnetDiscoveryTransport,
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
    let correlation_id = correlation_from_headers(raw_headers).unwrap_or_else(new_correlation_id);
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
        e3::record_trace(
            state,
            &correlation_id,
            &request.method,
            path,
            response.status,
            started_ms,
            None,
        );
        return (response, correlation_id);
    }
    if path == "/" || path == "/control-center" {
        let response = html_ok(include_str!("static/control-center.html"));
        e3::record_trace(
            state,
            &correlation_id,
            &request.method,
            path,
            response.status,
            started_ms,
            None,
        );
        return (response, correlation_id);
    }
    if path == "/v1/openapi.json" {
        let response = e3::openapi_spec();
        e3::record_trace(
            state,
            &correlation_id,
            &request.method,
            path,
            response.status,
            started_ms,
            None,
        );
        return (response, correlation_id);
    }
    if !path.starts_with("/v1/") {
        let response = not_found();
        e3::record_trace(
            state,
            &correlation_id,
            &request.method,
            path,
            response.status,
            started_ms,
            None,
        );
        return (response, correlation_id);
    }
    let ctx = state
        .api_keys
        .authenticate(request.authorization.as_deref());
    if ctx.is_some() && !ApiKeyStore::check_tenant(ctx.as_ref(), &state.tenant_id) {
        let response = tenant_forbidden();
        e3::record_trace(
            state,
            &correlation_id,
            &request.method,
            path,
            response.status,
            started_ms,
            ctx.as_ref(),
        );
        return (response, correlation_id);
    }
    if let Some(response) = crate::versioning::enforce_api_version(
        crate::versioning::api_version_from_headers(raw_headers).as_deref(),
    ) {
        e3::record_trace(
            state,
            &correlation_id,
            &request.method,
            path,
            response.status,
            started_ms,
            ctx.as_ref(),
        );
        return (response, correlation_id);
    }
    let skip_rate_limit = path == "/v1/health" || path == "/v1/version";
    if !skip_rate_limit {
        let rate_key = ctx
            .as_ref()
            .map(|context| context.key_id.clone())
            .unwrap_or_else(|| "anonymous".to_string());
        if let Err(retry_after) = state.rate_limiter.check(&rate_key) {
            let response = rate_limited(retry_after);
            e3::record_trace(
                state,
                &correlation_id,
                &request.method,
                path,
                response.status,
                started_ms,
                ctx.as_ref(),
            );
            return (response, correlation_id);
        }
    }
    if path.starts_with("/v1/devices/") && request.method == "PATCH" {
        let id = path.trim_start_matches("/v1/devices/");
        let response = device_patch(state, id, &request.body, ctx.as_ref());
        e3::record_trace(
            state,
            &correlation_id,
            &request.method,
            path,
            response.status,
            started_ms,
            ctx.as_ref(),
        );
        return (response, correlation_id);
    }
    if let Some(response) = route_device_subresource(
        state,
        path,
        &request.method,
        &request.body,
        query,
        ctx.as_ref(),
    ) {
        e3::record_trace(
            state,
            &correlation_id,
            &request.method,
            path,
            response.status,
            started_ms,
            ctx.as_ref(),
        );
        return (response, correlation_id);
    }
    if let Some(response) =
        route_config_approval(state, path, &request.method, &request.body, ctx.as_ref())
    {
        e3::record_trace(
            state,
            &correlation_id,
            &request.method,
            path,
            response.status,
            started_ms,
            ctx.as_ref(),
        );
        return (response, correlation_id);
    }
    if let Some(response) =
        route_sre_incident(state, path, &request.method, &request.body, ctx.as_ref())
    {
        e3::record_trace(
            state,
            &correlation_id,
            &request.method,
            path,
            response.status,
            started_ms,
            ctx.as_ref(),
        );
        return (response, correlation_id);
    }
    if let Some(response) = route_hri_session(
        state,
        path,
        &request.method,
        &request.body,
        started_ms,
        ctx.as_ref(),
    ) {
        e3::record_trace(
            state,
            &correlation_id,
            &request.method,
            path,
            response.status,
            started_ms,
            ctx.as_ref(),
        );
        return (response, correlation_id);
    }
    if let Some(response) = route_sdk_entities(
        state,
        path,
        &request.method,
        query,
        &request.body,
        ctx.as_ref(),
    ) {
        e3::record_trace(
            state,
            &correlation_id,
            &request.method,
            path,
            response.status,
            started_ms,
            ctx.as_ref(),
        );
        return (response, correlation_id);
    }
    if let Some(response) = route_humans(state, path, &request.method) {
        e3::record_trace(
            state,
            &correlation_id,
            &request.method,
            path,
            response.status,
            started_ms,
            ctx.as_ref(),
        );
        return (response, correlation_id);
    }
    let response = match (path, request.method.as_str()) {
        ("/v1/tenant", "GET") => tenant_info(state),
        ("/v1/audit/mutations", "GET") => mutation_audit_list(state, ctx.as_ref()),
        ("/v1/audit/mutations/export", "GET") => mutation_audit_export(state, query, ctx.as_ref()),
        ("/v1/dashboard", "GET") => dashboard(state),
        ("/v1/version", "GET") => api_version_info(),
        ("/v1/robots", "GET") => robots_list(state),
        ("/v1/fleets", "GET") => fleets_list(state),
        ("/v1/device-tree", "GET") => device_tree_get(state),
        ("/v1/readiness/run", "POST") => readiness_run(state, &request.body),
        ("/v1/device-reports", "GET") => device_reports_get(state),
        ("/v1/failover/chains", "GET") => failover_chains_get(state),
        ("/v1/devices", "GET") => devices_list(state),
        ("/v1/fleet/agents", "GET") => fleet_agents(),
        ("/v1/alerts", "GET") => alerts_list(state),
        ("/v1/alerts/test", "POST") => alerts_test(state, ctx.as_ref()),
        ("/v1/secrets", "GET") => secrets_list(state, ctx.as_ref()),
        ("/v1/rbac/matrix", "GET") => rbac_matrix(),
        ("/v1/provision", "POST") => provision_run(state, &request.body, ctx.as_ref()),
        ("/v1/config/snapshots", "GET") => config_snapshots_list(),
        ("/v1/config/snapshots", "POST") => {
            config_snapshots_save(state, &request.body, ctx.as_ref())
        }
        ("/v1/discovery", "GET") => discovery_run(query),
        ("/v1/health/summary", "GET") => health_summary(state),
        ("/v1/assurance/summary", "GET") => assurance_summary(state),
        ("/v1/diagnosis/summary", "GET") => diagnosis_summary(state),
        ("/v1/drift", "GET") => e3::drift_report(state, query),
        ("/v1/drift/scans", "GET") => crate::drift_scheduler::drift_scans_list(state),
        ("/v1/drift/scan", "POST") => {
            crate::drift_scheduler::drift_scan_run(state, &request.body, ctx.as_ref())
        }
        ("/v1/ota/status", "GET") => e3::ota_status(),
        ("/v1/ota/plan", "POST") => e3::ota_plan(state, &request.body, ctx.as_ref()),
        ("/v1/ota/execute", "POST") => e3::ota_execute(state, &request.body, ctx.as_ref()),
        ("/v1/trust/package", "GET") => e3::trust_package(query),
        ("/v1/trust/program", "GET") => crate::sdk_ops::trust_program(state, query),
        ("/v1/programs/readiness", "POST") => {
            crate::sdk_ops::program_readiness(state, &request.body)
        }
        ("/v1/programs/assure", "POST") => crate::sdk_ops::program_assure(state, &request.body),
        ("/v1/programs/diagnose", "POST") => crate::sdk_ops::program_diagnose(state, &request.body),
        ("/v1/programs/recovery/heal", "POST") => {
            crate::sdk_ops::program_heal(state, &request.body)
        }
        ("/v1/programs/verify/hardware", "POST") => {
            crate::sdk_ops::program_verify_hardware(state, &request.body)
        }
        ("/v1/programs/verify/capabilities", "POST") => {
            crate::sdk_ops::program_verify_capabilities(state, &request.body)
        }
        ("/v1/programs/verify/mission", "POST") => {
            crate::sdk_ops::program_verify_mission(state, &request.body)
        }
        ("/v1/programs/simulation", "POST") => {
            crate::sdk_ops::program_simulation(state, &request.body)
        }
        ("/v1/programs/replay", "POST") => crate::sdk_ops::program_replay(state, &request.body),
        ("/v1/sre/summary", "GET") => e3::sre_summary(state),
        ("/v1/integrations/pagerduty/webhook", "POST") => crate::integrations::pagerduty_webhook(
            state,
            &request.body,
            &parse_header_pairs(raw_headers),
            ctx.as_ref(),
        ),
        ("/v1/observability/traces", "GET") => e3::observability_traces(state),
        ("/v1/observability/otlp/traces", "GET") => observability::otlp_traces_preview(state),
        ("/v1/observability/otlp/metrics", "GET") => observability::otlp_metrics_preview(state),
        ("/v1/observability/backend", "GET") => observability::backend_info(),
        ("/v1/observability/otlp/export", "POST") => {
            observability::otlp_traces_export(state, query, ctx.as_ref())
        }
        ("/v1/observability/otlp/export-metrics", "POST") => {
            observability::otlp_metrics_export(state, query, ctx.as_ref())
        }
        ("/v1/operator/quarantine", "POST") => {
            e3::operator_quarantine(state, &request.body, ctx.as_ref())
        }
        ("/v1/operator/mission/approvals", "GET") => e3::mission_approvals_list(state),
        ("/v1/operator/mission/approve", "POST") => {
            e3::operator_mission_approve(&request.body, ctx.as_ref())
        }
        ("/v1/rpc", "POST") => e3::rpc_gateway(state, &request.body),
        ("/v1/compliance/export", "GET") => e4::compliance_export(state, query, None, ctx.as_ref()),
        ("/v1/compliance/export", "POST") => {
            e4::compliance_export(state, query, Some(&request.body), ctx.as_ref())
        }
        ("/v1/compliance/evidence", "GET") => e4::compliance_evidence_list(ctx.as_ref()),
        ("/v1/digital-thread/query", "GET") => e4::digital_thread_query(state, query),
        ("/v1/executive/scorecard", "GET") => e4::executive_scorecard(state),
        ("/v1/analytics/readiness", "GET") => e4::analytics_readiness(state, query),
        ("/v1/reports/export", "GET") => e4::reports_export(state, query, ctx.as_ref()),
        ("/v1/reports/schedules", "GET") => crate::report_scheduler::report_schedules_list(state),
        ("/v1/reports/schedules", "POST") => {
            crate::report_scheduler::report_schedules_create(state, &request.body, ctx.as_ref())
        }
        ("/v1/compliance/profiles", "GET") => e4::compliance_profiles_catalog(),
        _ => not_found(),
    };
    e3::record_trace(
        state,
        &correlation_id,
        &request.method,
        path,
        response.status,
        started_ms,
        ctx.as_ref(),
    );
    (response, correlation_id)
}

pub(crate) fn tenant_forbidden() -> HttpResponse {
    HttpResponse {
        status: 403,
        body: serde_json::json!({
            "ok": false,
            "error": "tenant mismatch",
        })
        .to_string(),
    }
}

fn tenant_info(state: &ControlCenterState) -> HttpResponse {
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "tenant_id": state.tenant_id,
        "isolation": "api_keys must match SPANDA_TENANT_ID on this Control Center instance",
    }))
}

fn api_version_info() -> HttpResponse {
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "api_version": crate::versioning::SUPPORTED_API_VERSION,
        "supported_versions": [crate::versioning::SUPPORTED_API_VERSION],
        "grpc": crate::grpc_policy::policy_json(),
        "rate_limit_per_minute": std::env::var("SPANDA_API_RATE_LIMIT_PER_MINUTE")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(0),
        "policy": "Breaking REST changes require a new /v2/ path prefix. gRPC uses proto semver in grpc.proto_semver; send X-Spanda-Api-Version: v1 or omit the header.",
    }))
}

pub(crate) fn rate_limited(retry_after_secs: u64) -> HttpResponse {
    HttpResponse {
        status: 429,
        body: serde_json::json!({
            "ok": false,
            "error": "rate limit exceeded",
            "retry_after_secs": retry_after_secs,
        })
        .to_string(),
    }
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

pub(crate) fn record_alert(state: &mut ControlCenterState, mut alert: Alert) {
    let now = now_ms();
    let window = spanda_ops::alert_dedup_window_ms(alert.severity);
    if state.alert_store.is_duplicate_within(&alert, now, window) {
        return;
    }
    let incident = state.incident_store.maybe_open_from_alert(&alert);
    let incident_id = incident.as_ref().map(|value| value.id.as_str());
    state
        .alert_dispatcher
        .dispatch_with_incident(&mut alert, incident_id);
    state.alert_store.push(alert.clone());
    let _ = crate::persistence::persist_runtime_state(state);
}

fn alerts_test(state: &mut ControlCenterState, ctx: Option<&RbacContext>) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Operate) {
        return unauthorized();
    }
    let alert = Alert {
        id: format!("test-{}", now_ms()),
        alert_type: AlertType::Custom,
        severity: AlertSeverity::Info,
        message: "Control Center alert test".into(),
        source: "control-center".into(),
        timestamp_ms: now_ms(),
        delivered_via: vec![],
    };
    record_alert(state, alert.clone());
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

fn mutation_audit_list(state: &ControlCenterState, ctx: Option<&RbacContext>) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Deploy) {
        return unauthorized();
    }
    let export = state
        .mutation_audit
        .export_json()
        .unwrap_or_else(|_| "{\"records\":[]}".into());
    let parsed: serde_json::Value =
        serde_json::from_str(&export).unwrap_or_else(|_| serde_json::json!({ "records": [] }));
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "audit": parsed,
        "persist_path": crate::audit_log::default_mutation_audit_path().to_string_lossy(),
        "record_count": state.mutation_audit.record_count(),
    }))
}

fn mutation_audit_export(
    state: &ControlCenterState,
    query: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Deploy) {
        return unauthorized();
    }
    let params = parse_query(query);
    let format = params.get("format").map(String::as_str).unwrap_or("jsonl");
    let path = crate::audit_log::default_mutation_audit_path();
    let body = match format {
        "cef" => crate::audit_log::export_mutation_audit_cef(&path),
        "jsonl" => crate::audit_log::export_mutation_audit_jsonl(&path),
        _ => return bad_request("format must be cef or jsonl"),
    };
    match body {
        Ok(content) => HttpResponse {
            status: 200,
            body: serde_json::json!({
                "version": API_VERSION,
                "format": format,
                "record_count": state.mutation_audit.record_count(),
                "content": content,
            })
            .to_string(),
        },
        Err(error) => bad_request(&error),
    }
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
        let alert = Alert {
            id: format!("provision-{}", now_ms()),
            alert_type: AlertType::ReadinessFailed,
            severity: AlertSeverity::Critical,
            message: format!("provisioning failed for device '{device_id}'"),
            source: "provisioning".into(),
            timestamp_ms: now_ms(),
            delivered_via: vec![],
        };
        record_alert(state, alert);
        if let Some(resolved) = state.resolved.as_mut() {
            if let Some(device) = resolved
                .device_registry
                .devices
                .iter_mut()
                .find(|d| d.id == device_id)
            {
                device.lifecycle_state =
                    Some(DeviceLifecycleState::Quarantined.as_str().to_string());
            }
        }
    } else if let Some(resolved) = state.resolved.as_mut() {
        if let Some(device) = resolved
            .device_registry
            .devices
            .iter_mut()
            .find(|d| d.id == device_id)
        {
            device.lifecycle_state = Some(DeviceLifecycleState::Active.as_str().to_string());
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
    let encrypt = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|value| value.get("encrypt").and_then(|flag| flag.as_bool()));
    let dir = default_snapshots_dir();
    match save_config_snapshot(resolved, &dir, label, encrypt) {
        Ok(meta) => json_ok(&serde_json::json!({
            "version": API_VERSION,
            "ok": true,
            "snapshot": meta,
        })),
        Err(e) => bad_request(&e.to_string()),
    }
}

fn config_approvals_list() -> HttpResponse {
    let path = spanda_config::default_approvals_path();
    let queue = spanda_config::load_approval_queue(&path).unwrap_or_default();
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "approvals": queue.requests,
    }))
}

fn config_approvals_submit(body: &str, ctx: Option<&RbacContext>) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Deploy) {
        return unauthorized();
    }
    let payload: serde_json::Value = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(error) => return bad_request(&error.to_string()),
    };
    let Some(snapshot_id) = payload.get("snapshot_id").and_then(|v| v.as_str()) else {
        return bad_request("missing snapshot_id");
    };
    let note = payload
        .get("note")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let required_approvals = payload
        .get("required_approvals")
        .and_then(|value| value.as_u64())
        .map(|value| value as u32);
    let requester = ctx
        .map(|context| context.key_id.clone())
        .unwrap_or_else(|| "anonymous".into());
    let path = spanda_config::default_approvals_path();
    let mut queue = spanda_config::load_approval_queue(&path).unwrap_or_default();
    match spanda_config::submit_config_approval(
        &mut queue,
        snapshot_id,
        &requester,
        note,
        required_approvals,
    ) {
        Ok(request) => {
            let _ = spanda_config::save_approval_queue(&path, &queue);
            json_ok(&serde_json::json!({
                "version": API_VERSION,
                "ok": true,
                "approval": request,
            }))
        }
        Err(error) => bad_request(&error.to_string()),
    }
}

fn config_approvals_resolve(
    state: &mut ControlCenterState,
    request_id: &str,
    approve: bool,
    body: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Approve) {
        return unauthorized();
    }
    let resolver = ctx
        .map(|context| context.key_id.clone())
        .unwrap_or_else(|| "anonymous".into());
    let note = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|value| {
            value
                .get("note")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        });
    let path = spanda_config::default_approvals_path();
    let mut queue = spanda_config::load_approval_queue(&path).unwrap_or_default();
    let result = if approve {
        spanda_config::approve_config_request(&mut queue, request_id, &resolver, note)
    } else {
        spanda_config::reject_config_request(&mut queue, request_id, &resolver, note)
    };
    match result {
        Ok(request) => {
            let _ = spanda_config::save_approval_queue(&path, &queue);
            let quorum = serde_json::json!({
                "required": request.required_approvals,
                "received": request.approvals.len(),
                "met": spanda_config::approval_quorum_met(&request),
            });
            let mut publish = None;
            if approve && request.status == spanda_config::ConfigApprovalStatus::Approved {
                match spanda_config::publish_config_snapshot(
                    &request.snapshot_id,
                    &spanda_config::default_snapshots_dir(),
                    state.project_root().as_deref(),
                ) {
                    Ok((resolved, publish_result)) => {
                        if let Err(error) = state
                            .apply_published_config(resolved, publish_result.reloaded_from_disk)
                        {
                            return bad_request(&error);
                        }
                        publish = Some(publish_result);
                    }
                    Err(error) => return bad_request(&error.to_string()),
                }
            }
            json_ok(&serde_json::json!({
                "version": API_VERSION,
                "ok": true,
                "approval": request,
                "quorum": quorum,
                "publish": publish,
            }))
        }
        Err(error) => bad_request(&error.to_string()),
    }
}

fn discovery_run(query: &str) -> HttpResponse {
    let params = parse_query(query);
    let transport = params
        .get("transport")
        .map(String::as_str)
        .unwrap_or("subnet");
    let options = DiscoveryOptions {
        subnet: params.get("subnet").cloned(),
        timeout_ms: params.get("timeout_ms").and_then(|v| v.parse().ok()),
        transports: vec![transport.to_string()],
    };
    let results = run_discovery_transports(&options);
    let discovery = results
        .into_iter()
        .next()
        .and_then(Result::ok)
        .or_else(|| SubnetDiscoveryTransport.discover(&options).ok());
    match discovery {
        Some(discovery) => json_ok(&serde_json::json!({
            "version": API_VERSION,
            "discovery": discovery,
            "installed_packages": spanda_config::list_installed_discovery_packages(),
            "tls": spanda_config::discovery_tls_summary(),
        })),
        None => bad_request("discovery failed"),
    }
}

fn route_config_approval(
    state: &mut ControlCenterState,
    path: &str,
    method: &str,
    body: &str,
    ctx: Option<&RbacContext>,
) -> Option<HttpResponse> {
    if path == "/v1/config/approvals" && method == "GET" {
        return Some(config_approvals_list());
    }
    if path == "/v1/config/approvals" && method == "POST" {
        return Some(config_approvals_submit(body, ctx));
    }
    let rest = path.strip_prefix("/v1/config/approvals/")?;
    let (request_id, action) = rest.split_once('/').unwrap_or((rest, ""));
    match (action, method) {
        ("approve", "POST") => Some(config_approvals_resolve(state, request_id, true, body, ctx)),
        ("reject", "POST") => Some(config_approvals_resolve(
            state, request_id, false, body, ctx,
        )),
        _ => None,
    }
}

fn route_sre_incident(
    state: &mut ControlCenterState,
    path: &str,
    method: &str,
    body: &str,
    ctx: Option<&RbacContext>,
) -> Option<HttpResponse> {
    if path == "/v1/sre/incidents" && method == "GET" {
        return Some(e3::sre_incidents_list(state));
    }
    if path == "/v1/sre/incidents" && method == "POST" {
        return Some(e3::sre_incidents_create(state, body, ctx));
    }
    let rest = path.strip_prefix("/v1/sre/incidents/")?;
    let (incident_id, action) = rest.split_once('/').unwrap_or((rest, ""));
    match (action, method) {
        ("ack", "POST") => Some(e3::sre_incident_ack(state, incident_id, body, ctx)),
        ("resolve", "POST") => Some(e3::sre_incident_resolve(state, incident_id, ctx)),
        _ => None,
    }
}

fn route_hri_session(
    state: &mut ControlCenterState,
    path: &str,
    method: &str,
    body: &str,
    now_ms: f64,
    ctx: Option<&RbacContext>,
) -> Option<HttpResponse> {
    if path == "/v1/hri/sessions" && method == "GET" {
        return Some(crate::hri::hri_sessions_list(state));
    }
    if path == "/v1/hri/collaboration" && method == "GET" {
        return Some(crate::hri::hri_collaboration_graph(state));
    }
    if path == "/v1/hri/context" && method == "GET" {
        return Some(crate::hri::hri_context_snapshot(state));
    }
    if path == "/v1/hri/sessions" && method == "POST" {
        return Some(crate::hri::hri_sessions_create(state, body, ctx));
    }
    let rest = path.strip_prefix("/v1/hri/sessions/")?;
    let (session_id, action) = rest.split_once('/').unwrap_or((rest, ""));
    match (action, method) {
        ("annotate", "POST") => Some(crate::hri::hri_session_annotate(
            state, session_id, body, ctx, now_ms,
        )),
        ("replay", "GET") => Some(crate::hri::hri_session_replay(state, session_id)),
        _ => None,
    }
}

fn route_sdk_entities(
    state: &mut ControlCenterState,
    path: &str,
    method: &str,
    query: &str,
    body: &str,
    ctx: Option<&RbacContext>,
) -> Option<HttpResponse> {
    if path == "/v1/entities/register" && method == "POST" {
        return Some(crate::entity_mutations::entity_register(state, body, ctx));
    }
    if path == "/v1/entities/relationships" && method == "POST" {
        return Some(crate::entity_mutations::entity_relate(state, body, ctx));
    }
    if path == "/v1/entities/sync" && method == "POST" {
        return Some(crate::entity_mutations::entity_sync(state, ctx));
    }
    if path == "/v1/entities/graph" && method == "GET" {
        return Some(crate::sdk_ops::entity_graph(state));
    }
    if path == "/v1/entities/traceability" && method == "GET" {
        return Some(crate::sdk_ops::entity_traceability(state, query));
    }
    if path == "/v1/entities/query" && method == "POST" {
        return Some(crate::sdk_ops::entity_query(state, body));
    }
    if path == "/v1/entities" && method == "GET" {
        let filter = crate::sdk_ops::entity_query_from_params(query);
        return Some(crate::sdk_ops::list_entities_filtered(state, &filter));
    }
    let rest = path.strip_prefix("/v1/entities/")?;
    if rest.contains('/') {
        let (entity_id, action) = rest.split_once('/')?;
        return match (action, method) {
            ("health", "GET") => Some(crate::sdk_ops::entity_health(state, entity_id)),
            ("readiness", "GET") => Some(crate::sdk_ops::entity_readiness(state, entity_id)),
            ("trust", "GET") => Some(crate::sdk_ops::entity_trust(state, entity_id)),
            ("relationships", "GET") => {
                Some(crate::sdk_ops::entity_relationships(state, entity_id))
            }
            ("tags", "POST") => Some(crate::entity_mutations::entity_tag(
                state, entity_id, body, ctx,
            )),
            _ => None,
        };
    }
    if method == "GET" {
        return Some(crate::sdk_ops::get_entity(state, rest));
    }
    None
}

fn route_humans(state: &ControlCenterState, path: &str, method: &str) -> Option<HttpResponse> {
    if path == "/v1/humans" && method == "GET" {
        return Some(crate::humans::humans_list(state));
    }
    if path == "/v1/wearables" && method == "GET" {
        return Some(crate::humans::wearables_list(state));
    }
    if path == "/v1/human-health/policy" && method == "GET" {
        return Some(crate::humans::human_health_policy(state));
    }
    if path == "/v1/humans/readiness" && method == "GET" {
        return Some(crate::humans::humans_readiness_team(state));
    }
    if path == "/v1/humans/twins" && method == "GET" {
        return Some(crate::humans::humans_twins_list(state));
    }
    let rest = path.strip_prefix("/v1/humans/")?;
    let (human_id, action) = rest.split_once('/').unwrap_or((rest, ""));
    match (action, method) {
        ("readiness", "GET") => Some(crate::humans::human_readiness_get(state, human_id)),
        _ => None,
    }
}

fn route_device_subresource(
    state: &mut ControlCenterState,
    path: &str,
    method: &str,
    body: &str,
    _query: &str,
    ctx: Option<&RbacContext>,
) -> Option<HttpResponse> {
    let rest = path.strip_prefix("/v1/devices/")?;
    if rest == "discover" && method == "POST" {
        return Some(discovery_post(state, body, ctx));
    }
    let (device_id, action) = rest.split_once('/').unwrap_or((rest, ""));
    match (action, method) {
        ("", "GET") => Some(device_get(state, device_id)),
        ("provision", "POST") => Some(device_provision(state, device_id, body, ctx)),
        ("assign", "POST") => Some(device_assign(state, device_id, body, ctx)),
        ("quarantine", "POST") => Some(device_quarantine(state, device_id, ctx)),
        ("trust", "POST") => Some(device_trust(state, device_id, ctx)),
        _ => None,
    }
}

fn device_get(state: &ControlCenterState, device_id: &str) -> HttpResponse {
    let registry = state.device_registry();
    let Some(device) = registry.get(device_id) else {
        return HttpResponse {
            status: 404,
            body: serde_json::json!({ "ok": false, "error": "device not found" }).to_string(),
        };
    };
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "device": device,
    }))
}

fn discovery_post(
    state: &mut ControlCenterState,
    body: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Provision) {
        return unauthorized();
    }
    let mut options: DiscoveryOptions = serde_json::from_str(body).unwrap_or_default();
    if options.subnet.is_none() {
        options.subnet = spanda_config::default_discovery_subnet();
    }
    let results: Vec<_> = run_discovery_transports(&options)
        .into_iter()
        .filter_map(Result::ok)
        .collect();
    let registered = if let Some(resolved) = state.resolved.as_mut() {
        ingest_discovery_matches(&mut resolved.device_registry, &results)
    } else {
        Vec::new()
    };
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "results": results,
        "registered": registered,
        "installed_packages": spanda_config::list_installed_discovery_packages(),
    }))
}

fn device_provision(
    state: &mut ControlCenterState,
    device_id: &str,
    body: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    let payload = serde_json::json!({
        "device_id": device_id,
    });
    let merged = if body.trim().is_empty() {
        payload.to_string()
    } else if let Ok(mut v) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(obj) = v.as_object_mut() {
            obj.entry("device_id")
                .or_insert(serde_json::json!(device_id));
        }
        v.to_string()
    } else {
        payload.to_string()
    };
    provision_run(state, &merged, ctx)
}

fn device_assign(
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
    let Some(robot_id) = payload.get("robot_id").and_then(|v| v.as_str()) else {
        return bad_request("missing robot_id");
    };
    let options = AssignDeviceOptions {
        robot_id: robot_id.to_string(),
        logical_name: payload
            .get("logical_name")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        redundant_group: payload
            .get("redundant_group")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        failover_priority: payload
            .get("failover_priority")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32),
    };
    let result = {
        let Some(resolved) = state.resolved.as_mut() else {
            return bad_request("no resolved configuration loaded");
        };
        resolved.device_registry.assign_device(device_id, &options)
    };
    match result {
        Ok(result) => {
            let persist = state.persist_device(device_id).ok();
            let mapping = state
                .resolved
                .as_ref()
                .map(|r| export_device_mapping_json(&r.device_registry, &r.logical_map));
            json_ok(&serde_json::json!({
                "version": API_VERSION,
                "result": result,
                "mapping": mapping,
                "persisted": persist,
            }))
        }
        Err(e) => bad_request(&e),
    }
}

fn device_quarantine(
    state: &mut ControlCenterState,
    device_id: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Provision) {
        return unauthorized();
    }
    let result = {
        let Some(resolved) = state.resolved.as_mut() else {
            return bad_request("no resolved configuration loaded");
        };
        resolved.device_registry.quarantine_device(device_id)
    };
    match result {
        Ok(result) => {
            let persist = state.persist_device(device_id).ok();
            json_ok(&serde_json::json!({
                "version": API_VERSION,
                "result": result,
                "persisted": persist,
            }))
        }
        Err(e) => bad_request(&e),
    }
}

fn device_trust(
    state: &mut ControlCenterState,
    device_id: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Approve) {
        return unauthorized();
    }
    let result = {
        let Some(resolved) = state.resolved.as_mut() else {
            return bad_request("no resolved configuration loaded");
        };
        resolved.device_registry.trust_device(device_id)
    };
    match result {
        Ok(result) => {
            let persist = state.persist_device(device_id).ok();
            json_ok(&serde_json::json!({
                "version": API_VERSION,
                "result": result,
                "persisted": persist,
            }))
        }
        Err(e) => bad_request(&e),
    }
}

fn robots_list(state: &ControlCenterState) -> HttpResponse {
    let Some(resolved) = state.resolved.as_ref() else {
        return json_ok(&serde_json::json!({
            "version": API_VERSION,
            "robots": [],
        }));
    };
    let robots: Vec<_> = resolved
        .device_tree
        .fleet
        .as_ref()
        .map(|f| {
            f.robots
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "model": r.model,
                        "hardware_profile": r.hardware_profile,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "robots": robots,
    }))
}

fn fleets_list(state: &ControlCenterState) -> HttpResponse {
    let Some(resolved) = state.resolved.as_ref() else {
        return json_ok(&serde_json::json!({
            "version": API_VERSION,
            "fleets": [],
        }));
    };
    let fleets = resolved
        .device_tree
        .fleet
        .as_ref()
        .map(|f| {
            vec![serde_json::json!({
                "id": f.id,
                "robot_count": f.robots.len(),
            })]
        })
        .unwrap_or_default();
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "fleets": fleets,
    }))
}

fn device_tree_get(state: &ControlCenterState) -> HttpResponse {
    let Some(resolved) = state.resolved.as_ref() else {
        return json_ok(&serde_json::json!({
            "version": API_VERSION,
            "loaded": false,
        }));
    };
    let mapping = export_device_mapping_json(&resolved.device_registry, &resolved.logical_map);
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "loaded": true,
        "hierarchy": resolved.device_tree.hierarchy_lines(),
        "mapping": mapping,
    }))
}

fn readiness_run(state: &ControlCenterState, _body: &str) -> HttpResponse {
    let registry = state.device_registry();
    let impact = readiness_impact(&registry, now_ms());
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "mission_ready": impact.blocked_count == 0,
        "impact": impact,
    }))
}

fn device_reports_get(state: &ControlCenterState) -> HttpResponse {
    let Some(resolved) = state.resolved.as_ref() else {
        return bad_request("no resolved configuration loaded");
    };
    let reports =
        generate_device_reports(&resolved.device_registry, &resolved.logical_map, now_ms());
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "reports": reports,
    }))
}

fn failover_chains_get(state: &ControlCenterState) -> HttpResponse {
    let registry = state.device_registry();
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "chains": failover_chains(&registry),
    }))
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

pub(crate) fn parse_header_pairs(raw_headers: &str) -> Vec<(String, String)> {
    raw_headers
        .lines()
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            Some((name.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

pub(crate) fn json_ok<T: Serialize>(value: &T) -> HttpResponse {
    let body = serde_json::to_string(value).unwrap_or_else(|_| "{}".into());
    HttpResponse { status: 200, body }
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
        429 => "Too Many Requests",
        _ => "Error",
    };
    let retry_header = if response.status == 429 {
        serde_json::from_str::<serde_json::Value>(&response.body)
            .ok()
            .and_then(|body| {
                body.get("retry_after_secs")
                    .and_then(|value| value.as_u64())
            })
            .map(|seconds| format!("Retry-After: {seconds}\r\n"))
            .unwrap_or_default()
    } else {
        String::new()
    };
    let correlation_header = correlation_id
        .map(|id| format!("X-Correlation-ID: {id}\r\n"))
        .unwrap_or_default();
    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: Authorization, Content-Type, X-Correlation-ID, X-Spanda-Api-Version\r\n{correlation_header}{retry_header}Access-Control-Expose-Headers: X-Correlation-ID\r\n\r\n{}",
        response.status,
        status_text,
        content_type,
        response.body.len(),
        response.body
    )
}

/// JSON body for gRPC `GetTenant` (parity with `GET /v1/tenant`).
pub fn tenant_info_json(state: &ControlCenterState) -> String {
    tenant_info(state).body
}

/// JSON body for gRPC `ListAuditMutations` (parity with `GET /v1/audit/mutations`).
pub fn mutation_audit_list_json(state: &ControlCenterState, ctx: Option<&RbacContext>) -> String {
    mutation_audit_list(state, ctx).body
}

/// JSON body for gRPC `ListDevices` (parity with `GET /v1/devices`).
pub fn devices_list_json(state: &ControlCenterState) -> String {
    devices_list(state).body
}

/// JSON body for gRPC `GetDevice` (parity with `GET /v1/devices/{id}`).
pub fn device_get_json(state: &ControlCenterState, device_id: &str) -> String {
    device_get(state, device_id).body
}

/// JSON body for gRPC `PatchDevice` (parity with `PATCH /v1/devices/{id}`).
pub fn device_patch_json(
    state: &mut ControlCenterState,
    device_id: &str,
    body: &str,
    ctx: Option<&RbacContext>,
) -> String {
    device_patch(state, device_id, body, ctx).body
}

/// JSON body for gRPC `DeviceProvision` (parity with `POST /v1/devices/{id}/provision`).
pub fn device_provision_json(
    state: &mut ControlCenterState,
    device_id: &str,
    body: &str,
    ctx: Option<&RbacContext>,
) -> String {
    device_provision(state, device_id, body, ctx).body
}

/// JSON body for gRPC `DeviceAssign` (parity with `POST /v1/devices/{id}/assign`).
pub fn device_assign_json(
    state: &mut ControlCenterState,
    device_id: &str,
    body: &str,
    ctx: Option<&RbacContext>,
) -> String {
    device_assign(state, device_id, body, ctx).body
}

/// JSON body for gRPC `DeviceQuarantine` (parity with `POST /v1/devices/{id}/quarantine`).
pub fn device_quarantine_json(
    state: &mut ControlCenterState,
    device_id: &str,
    ctx: Option<&RbacContext>,
) -> String {
    device_quarantine(state, device_id, ctx).body
}

/// JSON body for gRPC `DeviceTrust` (parity with `POST /v1/devices/{id}/trust`).
pub fn device_trust_json(
    state: &mut ControlCenterState,
    device_id: &str,
    ctx: Option<&RbacContext>,
) -> String {
    device_trust(state, device_id, ctx).body
}

/// JSON body for gRPC `ListFleetAgents` (parity with `GET /v1/fleet/agents`).
pub fn fleet_agents_json() -> String {
    fleet_agents().body
}

/// JSON body for gRPC `EvaluateReadiness` (parity with `POST /v1/readiness/run`).
pub fn readiness_run_json(state: &ControlCenterState, body: &str) -> String {
    readiness_run(state, body).body
}

/// JSON body for gRPC `GetSreSummary` (parity with `GET /v1/sre/summary`).
pub fn sre_summary_json(state: &ControlCenterState) -> String {
    e3::sre_summary(state).body
}

/// JSON body for gRPC `ListSreIncidents` (parity with `GET /v1/sre/incidents`).
pub fn sre_incidents_list_json(state: &ControlCenterState) -> String {
    e3::sre_incidents_list(state).body
}

/// JSON body for gRPC `CreateSreIncident` (parity with `POST /v1/sre/incidents`).
pub fn sre_incidents_create_json(
    state: &mut ControlCenterState,
    body: &str,
    ctx: Option<&RbacContext>,
) -> String {
    e3::sre_incidents_create(state, body, ctx).body
}

/// JSON body for gRPC `AckSreIncident` (parity with `POST /v1/sre/incidents/{id}/ack`).
pub fn sre_incident_ack_json(
    state: &mut ControlCenterState,
    incident_id: &str,
    body: &str,
    ctx: Option<&RbacContext>,
) -> String {
    e3::sre_incident_ack(state, incident_id, body, ctx).body
}

/// JSON body for gRPC `ResolveSreIncident` (parity with `POST /v1/sre/incidents/{id}/resolve`).
pub fn sre_incident_resolve_json(
    state: &mut ControlCenterState,
    incident_id: &str,
    ctx: Option<&RbacContext>,
) -> String {
    e3::sre_incident_resolve(state, incident_id, ctx).body
}

/// JSON body for gRPC `GetTrustPackage` (parity with `GET /v1/trust/package`).
pub fn trust_package_json(query: &str) -> String {
    e3::trust_package(query).body
}

/// JSON body for gRPC `GetOpenApi` (parity with `GET /v1/openapi.json`).
pub fn openapi_json() -> String {
    e3::openapi_spec().body
}

/// JSON body for gRPC `GetHealthSummary` (parity with `GET /v1/health/summary`).
pub fn health_summary_json(state: &ControlCenterState) -> String {
    health_summary(state).body
}

/// JSON body for gRPC `GetAssuranceSummary` (parity with `GET /v1/assurance/summary`).
pub fn assurance_summary_json(state: &ControlCenterState) -> String {
    assurance_summary(state).body
}

/// JSON body for gRPC `GetDiagnosisSummary` (parity with `GET /v1/diagnosis/summary`).
pub fn diagnosis_summary_json(state: &ControlCenterState) -> String {
    diagnosis_summary(state).body
}

/// JSON body for gRPC `GetExecutiveScorecard` (parity with `GET /v1/executive/scorecard`).
pub fn executive_scorecard_json(state: &ControlCenterState) -> String {
    e4::executive_scorecard(state).body
}

/// JSON body for gRPC `QueryDigitalThread` (parity with `GET /v1/digital-thread/query`).
pub fn digital_thread_query_json(state: &ControlCenterState, query: &str) -> String {
    e4::digital_thread_query(state, query).body
}

/// JSON body for gRPC `GetOtaStatus` (parity with `GET /v1/ota/status`).
pub fn ota_status_json() -> String {
    e3::ota_status().body
}

/// JSON body for gRPC `GetObservabilityBackend` (parity with `GET /v1/observability/backend`).
pub fn observability_backend_json() -> String {
    observability::backend_info().body
}

/// JSON body for gRPC `GetOtlpMetrics` (parity with `GET /v1/observability/otlp/metrics`).
pub fn otlp_metrics_json(state: &ControlCenterState) -> String {
    crate::observability::otlp_metrics_preview(state).body
}

/// JSON body for gRPC `DiscoverDevices` (parity with `GET /v1/discovery`).
pub fn discovery_run_json(query: &str) -> String {
    discovery_run(query).body
}

/// JSON body for gRPC `RunDiscovery` (parity with `POST /v1/devices/discover`).
pub fn discovery_post_json(
    state: &mut ControlCenterState,
    body: &str,
    ctx: Option<&RbacContext>,
) -> String {
    discovery_post(state, body, ctx).body
}

/// JSON body for gRPC `ProvisionDevice` (parity with `POST /v1/provision`).
pub fn provision_run_json(
    state: &mut ControlCenterState,
    body: &str,
    ctx: Option<&RbacContext>,
) -> String {
    provision_run(state, body, ctx).body
}

/// JSON body for gRPC `PlanOta` (parity with `POST /v1/ota/plan`).
pub fn ota_plan_json(state: &ControlCenterState, body: &str, ctx: Option<&RbacContext>) -> String {
    e3::ota_plan(state, body, ctx).body
}

/// JSON body for gRPC `ExecuteOta` (parity with `POST /v1/ota/execute`).
pub fn ota_execute_json(
    state: &ControlCenterState,
    body: &str,
    ctx: Option<&RbacContext>,
) -> String {
    e3::ota_execute(state, body, ctx).body
}

/// JSON body for gRPC `ListRobots` (parity with `GET /v1/robots`).
pub fn robots_list_json(state: &ControlCenterState) -> String {
    robots_list(state).body
}

/// JSON body for gRPC `ListFleets` (parity with `GET /v1/fleets`).
pub fn fleets_list_json(state: &ControlCenterState) -> String {
    fleets_list(state).body
}

/// JSON body for gRPC `ListAlerts` (parity with `GET /v1/alerts`).
pub fn alerts_list_json(state: &ControlCenterState) -> String {
    alerts_list(state).body
}

/// JSON body for gRPC `ListConfigSnapshots` (parity with `GET /v1/config/snapshots`).
pub fn config_snapshots_list_json() -> String {
    config_snapshots_list().body
}

/// JSON body for gRPC `SaveConfigSnapshot` (parity with `POST /v1/config/snapshots`).
pub fn config_snapshots_save_json(
    state: &ControlCenterState,
    body: &str,
    ctx: Option<&RbacContext>,
) -> String {
    config_snapshots_save(state, body, ctx).body
}

/// JSON body for gRPC `ListConfigApprovals` (parity with `GET /v1/config/approvals`).
pub fn config_approvals_list_json() -> String {
    config_approvals_list().body
}

/// JSON body for gRPC `SubmitConfigApproval` (parity with `POST /v1/config/approvals`).
pub fn config_approvals_submit_json(body: &str, ctx: Option<&RbacContext>) -> String {
    config_approvals_submit(body, ctx).body
}

/// JSON body for gRPC `ApproveConfigApproval` (parity with `POST /v1/config/approvals/{id}/approve`).
pub fn config_approvals_approve_json(
    state: &mut ControlCenterState,
    approval_id: &str,
    body: &str,
    ctx: Option<&RbacContext>,
) -> String {
    config_approvals_resolve(state, approval_id, true, body, ctx).body
}

/// JSON body for gRPC `RejectConfigApproval` (parity with `POST /v1/config/approvals/{id}/reject`).
pub fn config_approvals_reject_json(
    state: &mut ControlCenterState,
    approval_id: &str,
    body: &str,
    ctx: Option<&RbacContext>,
) -> String {
    config_approvals_resolve(state, approval_id, false, body, ctx).body
}

/// JSON body for gRPC `TestAlert` (parity with `POST /v1/alerts/test`).
pub fn alerts_test_json(state: &mut ControlCenterState, ctx: Option<&RbacContext>) -> String {
    alerts_test(state, ctx).body
}

/// JSON body for gRPC `GetDeviceTree` (parity with `GET /v1/device-tree`).
pub fn device_tree_json(state: &ControlCenterState) -> String {
    device_tree_get(state).body
}

/// JSON body for gRPC `GetDeviceReports` (parity with `GET /v1/device-reports`).
pub fn device_reports_json(state: &ControlCenterState) -> String {
    device_reports_get(state).body
}

/// JSON body for gRPC `GetFailoverChains` (parity with `GET /v1/failover/chains`).
pub fn failover_chains_json(state: &ControlCenterState) -> String {
    failover_chains_get(state).body
}

/// JSON body for gRPC `ListSecrets` (parity with `GET /v1/secrets`).
pub fn secrets_list_json(state: &ControlCenterState, ctx: Option<&RbacContext>) -> String {
    secrets_list(state, ctx).body
}

/// JSON body for gRPC `GetRbacMatrix` (parity with `GET /v1/rbac/matrix`).
pub fn rbac_matrix_json() -> String {
    rbac_matrix().body
}

/// JSON body for gRPC `GetAnalyticsReadiness` (parity with `GET /v1/analytics/readiness`).
pub fn analytics_readiness_json(state: &ControlCenterState, query: &str) -> String {
    e4::analytics_readiness(state, query).body
}

/// JSON body for gRPC `ExportReports` (parity with `GET /v1/reports/export`).
pub fn reports_export_json(
    state: &ControlCenterState,
    query: &str,
    ctx: Option<&RbacContext>,
) -> String {
    e4::reports_export(state, query, ctx).body
}

/// JSON body for gRPC `GetObservabilityTraces` (parity with `GET /v1/observability/traces`).
pub fn observability_traces_json(state: &ControlCenterState) -> String {
    e3::observability_traces(state).body
}

/// JSON body for gRPC `GetOtlpTraces` (parity with `GET /v1/observability/otlp/traces`).
pub fn otlp_traces_json(state: &ControlCenterState) -> String {
    observability::otlp_traces_preview(state).body
}

/// JSON body for gRPC `ExportOtlpTraces` (parity with `POST /v1/observability/otlp/export`).
pub fn otlp_traces_export_json(
    state: &ControlCenterState,
    query: &str,
    ctx: Option<&RbacContext>,
) -> String {
    observability::otlp_traces_export(state, query, ctx).body
}

/// JSON body for gRPC `ExportOtlpMetrics` (parity with `POST /v1/observability/otlp/export-metrics`).
pub fn otlp_metrics_export_json(
    state: &ControlCenterState,
    query: &str,
    ctx: Option<&RbacContext>,
) -> String {
    observability::otlp_metrics_export(state, query, ctx).body
}

/// JSON body for gRPC `OperatorQuarantine` (parity with `POST /v1/operator/quarantine`).
pub fn operator_quarantine_json(
    state: &mut ControlCenterState,
    body: &str,
    ctx: Option<&RbacContext>,
) -> String {
    e3::operator_quarantine(state, body, ctx).body
}

/// JSON body for gRPC `OperatorMissionApprove` (parity with `POST /v1/operator/mission/approve`).
pub fn operator_mission_approve_json(body: &str, ctx: Option<&RbacContext>) -> String {
    e3::operator_mission_approve(body, ctx).body
}

/// JSON body for gRPC `ExportCompliance` (parity with `GET /v1/compliance/export`).
pub fn compliance_export_json(
    state: &ControlCenterState,
    query: &str,
    ctx: Option<&RbacContext>,
) -> String {
    e4::compliance_export(state, query, None, ctx).body
}

/// JSON body for gRPC `ListComplianceEvidence` (parity with `GET /v1/compliance/evidence`).
pub fn compliance_evidence_list_json(ctx: Option<&RbacContext>) -> String {
    e4::compliance_evidence_list(ctx).body
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
