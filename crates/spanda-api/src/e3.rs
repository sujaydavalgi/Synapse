//! Phase E3 handlers — drift, OTA, trust, SRE, operator workflows, RPC gateway.
//!
use crate::correlation::TraceRecord;
use crate::handlers::{bad_request, json_ok, now_ms, parse_query, unauthorized};
use crate::observability::maybe_auto_push_latest_span;
use crate::state::ControlCenterState;
use serde::Deserialize;
use spanda_config::{
    default_snapshots_dir, detect_operational_drift, load_config_snapshot,
    DeviceLifecycleState,
};
use spanda_deploy_http::HttpResponse;
use spanda_ota::{
    default_state_path, load_deploy_state, plan_rollout, DeployAssignment, DeployPlan,
    RolloutOptions, RolloutStrategy,
};
use spanda_package::evaluate_package_trust;
use spanda_security::{ApiKeyStore, RbacAction, RbacContext};

#[derive(Deserialize)]
struct OtaPlanRequest {
    strategy: String,
    version: String,
    #[serde(default)]
    canary_percent: Option<u8>,
    #[serde(default)]
    dry_run: bool,
    #[serde(default)]
    assignments: Vec<DeployAssignment>,
}

#[derive(Deserialize)]
struct OperatorMissionRequest {
    mission_id: String,
    #[serde(default)]
    approved: bool,
}

#[derive(Deserialize)]
struct OperatorQuarantineRequest {
    device_id: String,
    #[serde(default)]
    reason: Option<String>,
}

#[derive(Deserialize)]
struct RpcRequest {
    method: String,
    #[serde(default)]
    #[allow(dead_code)]
    params: serde_json::Value,
}

pub fn drift_report(state: &ControlCenterState, query: &str) -> HttpResponse {
    let baseline_id = parse_query(query).get("baseline_id").cloned();
    let Some(current) = state.resolved.as_ref() else {
        return bad_request("no resolved configuration; use --config");
    };
    let baseline = if let Some(id) = baseline_id {
        load_config_snapshot(&default_snapshots_dir(), &id)
            .map(|s| s.resolved)
            .map_err(|e| e.to_string())
    } else {
        Err("missing baseline_id query parameter".into())
    };
    match baseline {
        Ok(base) => {
            let report = detect_operational_drift(&base, current);
            json_ok(&serde_json::json!({
                "version": "v1",
                "report": report,
            }))
        }
        Err(e) => bad_request(&e),
    }
}

pub fn ota_status() -> HttpResponse {
    let path = default_state_path();
    let state = load_deploy_state(&path);
    json_ok(&serde_json::json!({
        "version": "v1",
        "state": state,
    }))
}

pub fn ota_plan(body: &str, ctx: Option<&RbacContext>) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Deploy) {
        return unauthorized();
    }
    let req: OtaPlanRequest = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => return bad_request(&e.to_string()),
    };
    let strategy = match req.strategy.to_ascii_lowercase().as_str() {
        "canary" => RolloutStrategy::Canary,
        "staged" | "phased" => RolloutStrategy::Staged,
        "blue_green" | "bluegreen" => RolloutStrategy::BlueGreen,
        _ => RolloutStrategy::All,
    };
    let plan = DeployPlan {
        program: "control-center".into(),
        version: req.version.clone(),
        program_hash: None,
        assignments: req.assignments,
        certifications: vec![],
        certification_proof: None,
    };
    let options = RolloutOptions {
        strategy,
        canary_percent: req.canary_percent.unwrap_or(10),
        version: req.version,
        dry_run: req.dry_run,
        ..RolloutOptions::default()
    };
    let result = plan_rollout(&plan, &options);
    json_ok(&serde_json::json!({
        "version": "v1",
        "rollout": result,
    }))
}

pub fn trust_package(query: &str) -> HttpResponse {
    let params = parse_query(query);
    let Some(name) = params.get("name") else {
        return bad_request("missing name query parameter");
    };
    let version = params.get("version").map(String::as_str);
    let report = evaluate_package_trust(name, version, None);
    json_ok(&serde_json::json!({
        "version": "v1",
        "trust": report,
    }))
}

pub fn sre_summary(state: &ControlCenterState) -> HttpResponse {
    let pool = state.device_registry().pool_summary();
    let alerts = state.alert_store.list_owned();
    let critical = alerts
        .iter()
        .filter(|a| format!("{:?}", a.severity).to_ascii_lowercase().contains("critical"))
        .count();
    let traces = state.trace_log.list_owned();
    json_ok(&serde_json::json!({
        "version": "v1",
        "availability_percent": if pool.total == 0 {
            100.0
        } else {
            ((pool.healthy + pool.assigned) as f64 / pool.total as f64) * 100.0
        },
        "devices_total": pool.total,
        "devices_healthy": pool.healthy,
        "alerts_total": alerts.len(),
        "alerts_critical": critical,
        "traces_recorded": traces.len(),
        "mttr_hint_ms": null,
    }))
}

pub fn observability_traces(state: &ControlCenterState) -> HttpResponse {
    json_ok(&serde_json::json!({
        "version": "v1",
        "traces": state.trace_log.list_owned(),
    }))
}

pub fn operator_quarantine(
    state: &mut ControlCenterState,
    body: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Operate) {
        return unauthorized();
    }
    let req: OperatorQuarantineRequest = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => return bad_request(&e.to_string()),
    };
    let mut registry = state.device_registry();
    if let Err(e) = registry.set_lifecycle(&req.device_id, DeviceLifecycleState::Quarantined) {
        return bad_request(&e);
    }
    if let Some(resolved) = state.resolved.as_mut() {
        resolved.device_registry = registry;
    }
    json_ok(&serde_json::json!({
        "version": "v1",
        "ok": true,
        "device_id": req.device_id,
        "lifecycle_state": "quarantined",
        "reason": req.reason,
    }))
}

pub fn operator_mission_approve(body: &str, ctx: Option<&RbacContext>) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Approve) {
        return unauthorized();
    }
    let req: OperatorMissionRequest = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => return bad_request(&e.to_string()),
    };
    json_ok(&serde_json::json!({
        "version": "v1",
        "ok": true,
        "mission_id": req.mission_id,
        "approved": req.approved,
        "status": if req.approved { "approved" } else { "rejected" },
    }))
}

pub fn rpc_gateway(state: &mut ControlCenterState, body: &str) -> HttpResponse {
    let req: RpcRequest = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => return bad_request(&e.to_string()),
    };
    let response = match req.method.as_str() {
        "spanda.v1.SpandaService/GetHealth" => serde_json::json!({
            "ok": true,
            "service": "spanda-control-center",
        }),
        "spanda.v1.SpandaService/GetDashboard" => {
            let pool = state.device_registry().pool_summary();
            serde_json::json!({ "device_pool": pool })
        }
        "spanda.v1.SpandaService/GetSreSummary" => {
            let resp = sre_summary(state);
            serde_json::from_str(&resp.body).unwrap_or(serde_json::json!({}))
        }
        _ => {
            return bad_request(&format!("unknown rpc method: {}", req.method));
        }
    };
    json_ok(&serde_json::json!({
        "version": "v1",
        "method": req.method,
        "result": response,
    }))
}

pub fn record_trace(
    state: &mut ControlCenterState,
    correlation_id: &str,
    method: &str,
    path: &str,
    status: u16,
    started_ms: f64,
) {
    state.trace_log.push(TraceRecord {
        correlation_id: correlation_id.to_string(),
        method: method.to_string(),
        path: path.to_string(),
        status,
        timestamp_ms: started_ms,
        duration_ms: Some(now_ms() - started_ms),
    });
    if let Some(record) = state.trace_log.list_owned().last() {
        maybe_auto_push_latest_span(record);
    }
}

pub fn openapi_spec() -> HttpResponse {
    HttpResponse {
        status: 200,
        body: include_str!("static/openapi.json").to_string(),
    }
}
