//! Phase E3 handlers — drift, OTA, trust, SRE, operator workflows, RPC gateway.
//!
use crate::correlation::TraceRecord;
use crate::handlers::{bad_request, json_ok, now_ms, parse_query, unauthorized};
use crate::observability::maybe_auto_push_latest_span;
use crate::state::ControlCenterState;
use serde::Deserialize;
use spanda_certify::build_certification_proof_summary;
use spanda_config::{
    default_mission_approvals_path, default_snapshots_dir, detect_operational_drift_full,
    load_config_snapshot, load_mission_approval_queue, merge_mission_approval_seeds,
    resolve_mission_approval, save_mission_approval_queue, DeviceLifecycleState,
    MissionApprovalStatus,
};
use spanda_deploy_http::HttpResponse;
use spanda_ota::{
    apply_rollout, build_deploy_bundle, build_deploy_plan_from_program, default_state_path,
    execute_remote_rollout, load_agent_registry, load_deploy_state, plan_rollout,
    save_deploy_state, validate_rollout_certification, CertificationProofSummary, DeployAssignment,
    DeployPlan, RolloutOptions, RolloutStrategy,
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
    #[serde(default)]
    rollback_on_readiness_fail: bool,
    #[serde(default)]
    readiness_runtime: bool,
    #[serde(default)]
    readiness_inject_faults: bool,
    #[serde(default)]
    require_certify: bool,
}

#[derive(Deserialize)]
struct OperatorMissionRequest {
    mission_id: String,
    #[serde(default)]
    approval_id: Option<String>,
    #[serde(default)]
    approved: bool,
    #[serde(default)]
    note: Option<String>,
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
            let program = state
                .program_path
                .as_ref()
                .and_then(|path| crate::program::parse_program_file(path).ok())
                .map(|(program, _, _)| program);
            let agent_findings = program
                .as_ref()
                .map(|program| {
                    crate::drift_collect::collect_agent_drift_findings(
                        program,
                        current,
                        state.program_path.as_deref(),
                    )
                })
                .unwrap_or_default();
            let report =
                detect_operational_drift_full(&base, current, program.as_ref(), &agent_findings);
            json_ok(&serde_json::json!({
                "version": "v1",
                "report": report,
                "agent_findings": agent_findings.len(),
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

fn production_require_certify(request_flag: bool) -> bool {
    request_flag
        || std::env::var("SPANDA_OTA_REQUIRE_CERTIFY")
            .ok()
            .is_some_and(|value| {
                value == "1"
                    || value.eq_ignore_ascii_case("true")
                    || value.eq_ignore_ascii_case("yes")
            })
        || std::env::var("SPANDA_PRODUCTION_POLICY")
            .ok()
            .is_some_and(|value| value.eq_ignore_ascii_case("production"))
}

fn enrich_ota_plan(state: &ControlCenterState, plan: &mut DeployPlan) {
    let Some(program_path) = state.program_path.as_ref() else {
        return;
    };
    let Ok((program, _, _)) = crate::program::parse_program_file(program_path) else {
        return;
    };
    if plan.assignments.is_empty() {
        let program_path_str = program_path.to_string_lossy();
        let built = build_deploy_plan_from_program(&program, &program_path_str, &plan.version);
        plan.assignments = built.assignments;
        plan.certifications = built.certifications;
        plan.program = built.program;
        plan.program_hash = built.program_hash;
    }
    if plan.certification_proof.is_none() {
        let program_path_str = program_path.to_string_lossy();
        let proof = build_certification_proof_summary(&program, &program_path_str);
        plan.certification_proof = Some(CertificationProofSummary {
            passed: proof.passed,
            passed_strict: proof.passed_strict,
            summary: proof.summary,
            error_count: proof.error_count,
            warning_count: proof.warning_count,
        });
    }
}

fn parse_ota_plan_request(
    state: &ControlCenterState,
    body: &str,
) -> Result<(DeployPlan, RolloutOptions), HttpResponse> {
    let req: OtaPlanRequest = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => return Err(bad_request(&e.to_string())),
    };
    let strategy = match req.strategy.to_ascii_lowercase().as_str() {
        "canary" => RolloutStrategy::Canary,
        "staged" | "phased" => RolloutStrategy::Staged,
        "blue_green" | "bluegreen" => RolloutStrategy::BlueGreen,
        _ => RolloutStrategy::All,
    };
    let mut plan = DeployPlan {
        program: "control-center".into(),
        version: req.version.clone(),
        program_hash: None,
        assignments: req.assignments,
        certifications: vec![],
        certification_proof: None,
    };
    enrich_ota_plan(state, &mut plan);
    let rollback_on_readiness_fail = req.rollback_on_readiness_fail
        || std::env::var("SPANDA_OTA_ROLLBACK_ON_READINESS_FAIL")
            .ok()
            .map(|value| {
                value == "1"
                    || value.eq_ignore_ascii_case("true")
                    || value.eq_ignore_ascii_case("yes")
            })
            .unwrap_or(false);
    let options = RolloutOptions {
        strategy,
        canary_percent: req.canary_percent.unwrap_or(10),
        version: req.version,
        dry_run: req.dry_run,
        require_certify: production_require_certify(req.require_certify),
        rollback_on_readiness_fail,
        readiness_runtime: req.readiness_runtime,
        readiness_inject_faults: req.readiness_inject_faults,
        ..RolloutOptions::default()
    };
    Ok((plan, options))
}

pub fn ota_plan(state: &ControlCenterState, body: &str, ctx: Option<&RbacContext>) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Deploy) {
        return unauthorized();
    }
    let (plan, options) = match parse_ota_plan_request(state, body) {
        Ok(v) => v,
        Err(response) => return response,
    };
    if let Err(error) = validate_rollout_certification(&plan, &options) {
        return bad_request(&error);
    }
    let result = plan_rollout(&plan, &options);
    json_ok(&serde_json::json!({
        "version": "v1",
        "rollout": result,
        "require_certify": options.require_certify,
    }))
}

pub fn ota_execute(
    state: &ControlCenterState,
    body: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Deploy) {
        return unauthorized();
    }
    let (plan, options) = match parse_ota_plan_request(state, body) {
        Ok(v) => v,
        Err(response) => return response,
    };
    if let Err(error) = validate_rollout_certification(&plan, &options) {
        return bad_request(&error);
    }
    if options.dry_run {
        let result = plan_rollout(&plan, &options);
        return json_ok(&serde_json::json!({
            "version": "v1",
            "dry_run": true,
            "rollout": result,
        }));
    }
    let registry = load_agent_registry(&spanda_ota::agents_registry_path());
    let bundle = build_deploy_bundle(&plan);
    let result = execute_remote_rollout(&plan, &options, &registry, &bundle);
    if result.success {
        let path = default_state_path();
        let mut deploy_state = load_deploy_state(&path);
        apply_rollout(&mut deploy_state, &result);
        if let Err(error) = save_deploy_state(&path, &deploy_state) {
            return bad_request(&error);
        }
    }
    json_ok(&serde_json::json!({
        "version": "v1",
        "executed": true,
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
        .filter(|a| {
            format!("{:?}", a.severity)
                .to_ascii_lowercase()
                .contains("critical")
        })
        .count();
    let traces = state.trace_log.list_owned();
    let incidents = state.incident_store.list_owned();
    let availability = if pool.total == 0 {
        100.0
    } else {
        ((pool.healthy + pool.assigned) as f64 / pool.total as f64) * 100.0
    };
    let health_trends = spanda_ops::health_trends_summary(
        pool.degraded as usize,
        pool.failed as usize,
        pool.offline as usize,
        pool.total as usize,
    );
    let readiness_trends = state.program_path.as_ref().and_then(|path| {
        let history = spanda_readiness::load_readiness_history(
            &spanda_readiness::default_readiness_history_path(),
        );
        let label = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("program.sd")
            .to_string();
        let report = spanda_readiness::analyze_readiness_trends(&history, &label, Some(7), 80);
        serde_json::to_value(&report).ok()
    });
    json_ok(&serde_json::json!({
        "version": "v1",
        "availability_percent": availability,
        "devices_total": pool.total,
        "devices_healthy": pool.healthy,
        "alerts_total": alerts.len(),
        "alerts_critical": critical,
        "traces_recorded": traces.len(),
        "incidents_total": incidents.len(),
        "incidents_open": state.incident_store.open_count(),
        "incidents_acknowledged": state.incident_store.acknowledged_count(),
        "mttr_hint_ms": state.incident_store.mttr_hint_ms(),
        "mtbf_hint_ms": spanda_ops::mtbf_hint_ms(&alerts),
        "health_trends": health_trends,
        "readiness_trends": readiness_trends,
        "slo": spanda_ops::slo_status(availability),
        "burn_rate": spanda_ops::slo_burn_rate_summary(&alerts),
    }))
}

pub fn sre_incidents_list(state: &ControlCenterState) -> HttpResponse {
    json_ok(&serde_json::json!({
        "version": "v1",
        "incidents": state.incident_store.list_owned(),
    }))
}

pub fn sre_incidents_create(
    state: &mut ControlCenterState,
    body: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Operate) {
        return unauthorized();
    }
    let payload: serde_json::Value = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(error) => return bad_request(&error.to_string()),
    };
    let Some(title) = payload.get("title").and_then(|value| value.as_str()) else {
        return bad_request("missing title");
    };
    let description = payload
        .get("description")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let severity = match payload
        .get("severity")
        .and_then(|value| value.as_str())
        .unwrap_or("warning")
    {
        "critical" => spanda_ops::IncidentSeverity::Critical,
        "info" => spanda_ops::IncidentSeverity::Info,
        _ => spanda_ops::IncidentSeverity::Warning,
    };
    let source_alert_id = payload
        .get("source_alert_id")
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let incident = state.incident_store.create(
        title.to_string(),
        description.to_string(),
        severity,
        source_alert_id,
    );
    let _ = crate::persistence::persist_runtime_state(state);
    json_ok(&serde_json::json!({
        "ok": true,
        "incident": incident,
    }))
}

pub fn sre_incident_ack(
    state: &mut ControlCenterState,
    incident_id: &str,
    body: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Operate) {
        return unauthorized();
    }
    let assignee = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|value| {
            value
                .get("assignee")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        });
    let Some(incident) = state.incident_store.acknowledge(incident_id, assignee) else {
        return bad_request("incident not found or already resolved");
    };
    let pagerduty_sync =
        spanda_ops::pagerduty::sync_incident_status_to_pagerduty(&incident, "acknowledge");
    let _ = crate::persistence::persist_runtime_state(state);
    json_ok(&serde_json::json!({
        "ok": true,
        "incident": incident,
        "pagerduty_sync": pagerduty_sync,
    }))
}

pub fn sre_incident_resolve(
    state: &mut ControlCenterState,
    incident_id: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Operate) {
        return unauthorized();
    }
    let Some(incident) = state.incident_store.resolve(incident_id) else {
        return bad_request("incident not found");
    };
    let pagerduty_sync =
        spanda_ops::pagerduty::sync_incident_status_to_pagerduty(&incident, "resolve");
    let _ = crate::persistence::persist_runtime_state(state);
    json_ok(&serde_json::json!({
        "ok": true,
        "incident": incident,
        "pagerduty_sync": pagerduty_sync,
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

pub fn mission_approvals_list(state: &ControlCenterState) -> HttpResponse {
    let path = default_mission_approvals_path();
    let mut queue = load_mission_approval_queue(&path).unwrap_or_default();
    if let Some(resolved) = state.resolved.as_ref() {
        merge_mission_approval_seeds(
            &mut queue,
            &resolved.human_registry.mission_approvals,
            now_ms(),
        );
        let _ = save_mission_approval_queue(&path, &queue);
    }
    json_ok(&serde_json::json!({
        "version": "v1",
        "approvals": queue.requests,
        "count": queue.requests.len(),
        "pending": queue.requests.iter().filter(|r| r.status == MissionApprovalStatus::Pending).count(),
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
    let path = default_mission_approvals_path();
    let mut queue = load_mission_approval_queue(&path).unwrap_or_default();
    let resolver = ctx
        .map(|c| c.key_id.clone())
        .unwrap_or_else(|| "operator".into());
    let lookup = req
        .approval_id
        .as_deref()
        .unwrap_or(req.mission_id.as_str());
    let record =
        match resolve_mission_approval(&mut queue, lookup, req.approved, &resolver, now_ms()) {
            Ok(record) => record,
            Err(_) => {
                queue.requests.push(spanda_config::MissionApprovalRecord {
                    id: format!("mission-{}", req.mission_id),
                    mission_id: req.mission_id.clone(),
                    requested_by: None,
                    status: if req.approved {
                        MissionApprovalStatus::Approved
                    } else {
                        MissionApprovalStatus::Rejected
                    },
                    created_at_ms: now_ms(),
                    resolved_at_ms: Some(now_ms()),
                    resolver: Some(resolver.clone()),
                    note: req.note.clone(),
                });
                queue.requests.last().unwrap().clone()
            }
        };
    let _ = save_mission_approval_queue(&path, &queue);
    json_ok(&serde_json::json!({
        "version": "v1",
        "ok": true,
        "mission_id": req.mission_id,
        "approval_id": record.id,
        "approved": req.approved,
        "status": if req.approved { "approved" } else { "rejected" },
        "record": record,
    }))
}

pub fn rpc_gateway(state: &mut ControlCenterState, body: &str) -> HttpResponse {
    let req: RpcRequest = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => return bad_request(&e.to_string()),
    };
    let params = &req.params;
    let body_json = params
        .get("body_json")
        .and_then(|v| v.as_str())
        .unwrap_or("{}");
    let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("");
    let entity_id = params
        .get("entity_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
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
        "spanda.v1.ControlCenter/ListEntities" => {
            serde_json::from_str(&crate::sdk_ops::list_entities_json(state)).unwrap_or_default()
        }
        "spanda.v1.ControlCenter/GetEntity" => {
            serde_json::from_str(&crate::sdk_ops::get_entity_json(state, entity_id))
                .unwrap_or_default()
        }
        "spanda.v1.ControlCenter/GetEntityHealth" => {
            serde_json::from_str(&crate::sdk_ops::entity_health_json(state, entity_id))
                .unwrap_or_default()
        }
        "spanda.v1.ControlCenter/GetEntityTrust" => {
            serde_json::from_str(&crate::sdk_ops::entity_trust_json(state, entity_id))
                .unwrap_or_default()
        }
        "spanda.v1.ControlCenter/EvaluateProgramReadiness" => {
            serde_json::from_str(&crate::sdk_ops::program_readiness_json(state, body_json))
                .unwrap_or_default()
        }
        "spanda.v1.ControlCenter/EvaluateProgramAssure" => {
            serde_json::from_str(&crate::sdk_ops::program_assure_json(state, body_json))
                .unwrap_or_default()
        }
        "spanda.v1.ControlCenter/EvaluateProgramDiagnose" => {
            serde_json::from_str(&crate::sdk_ops::program_diagnose_json(state, body_json))
                .unwrap_or_default()
        }
        "spanda.v1.ControlCenter/EvaluateProgramHeal" => {
            serde_json::from_str(&crate::sdk_ops::program_heal_json(state, body_json))
                .unwrap_or_default()
        }
        "spanda.v1.ControlCenter/VerifyProgramHardware" => serde_json::from_str(
            &crate::sdk_ops::program_verify_hardware_json(state, body_json),
        )
        .unwrap_or_default(),
        "spanda.v1.ControlCenter/VerifyProgramCapabilities" => serde_json::from_str(
            &crate::sdk_ops::program_verify_capabilities_json(state, body_json),
        )
        .unwrap_or_default(),
        "spanda.v1.ControlCenter/VerifyProgramMission" => serde_json::from_str(
            &crate::sdk_ops::program_verify_mission_json(state, body_json),
        )
        .unwrap_or_default(),
        "spanda.v1.ControlCenter/RunProgramSimulation" => {
            serde_json::from_str(&crate::sdk_ops::program_simulation_json(state, body_json))
                .unwrap_or_default()
        }
        "spanda.v1.ControlCenter/ReplayProgram" => {
            serde_json::from_str(&crate::sdk_ops::program_replay_json(state, body_json))
                .unwrap_or_default()
        }
        "spanda.v1.ControlCenter/GetTrustProgram" => {
            serde_json::from_str(&crate::sdk_ops::trust_program_json(state, query))
                .unwrap_or_default()
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
    ctx: Option<&RbacContext>,
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
    crate::audit_log::maybe_record_mutation(state, method, path, status, ctx, correlation_id);
    let _ = crate::persistence::persist_runtime_state(state);
}

pub fn openapi_spec() -> HttpResponse {
    HttpResponse {
        status: 200,
        body: include_str!("static/openapi.json").to_string(),
    }
}
