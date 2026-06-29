//! Native gRPC server (tonic) for Control Center CLI parity.
//!
use crate::state::SharedState;
use spanda_security::RbacContext;
use tonic::{transport::Server, Request, Response, Status};

pub mod spanda_v1 {
    tonic::include_proto!("spanda.v1");
}

use spanda_v1::control_center_server::{ControlCenter, ControlCenterServer};
use spanda_v1::{
    ApprovalBodyRequest, DeviceBodyRequest, DeviceIdRequest, DriftRequest, Empty,
    EntityBodyRequest, EntityIdRequest, HealthResponse, IncidentBodyRequest, IncidentIdRequest,
    JsonBodyRequest, JsonResponse, QueryRequest, ReadinessRequest, TrustPackageRequest,
};

struct GrpcControlCenter {
    state: SharedState,
}

#[allow(clippy::result_large_err)]
impl GrpcControlCenter {
    fn bearer_token<T>(request: &Request<T>) -> Option<String> {
        request
            .metadata()
            .get("authorization")
            .or_else(|| request.metadata().get("x-api-key"))
            .and_then(|value| value.to_str().ok())
            .map(|raw| {
                let trimmed = raw.trim();
                trimmed
                    .strip_prefix("Bearer ")
                    .unwrap_or(trimmed)
                    .trim()
                    .to_string()
            })
    }

    fn rbac_from_request<T>(&self, request: &Request<T>) -> Option<RbacContext> {
        let token = Self::bearer_token(request);
        let guard = self.state.lock().ok()?;
        guard.api_keys.authenticate(token.as_deref())
    }

    fn with_state<F>(&self, f: F) -> Result<JsonResponse, Status>
    where
        F: FnOnce(&crate::state::ControlCenterState) -> String,
    {
        let guard = self
            .state
            .lock()
            .map_err(|e| Status::internal(e.to_string()))?;
        let json = f(&guard);
        Ok(JsonResponse { json })
    }

    fn with_state_mut<F>(&self, f: F) -> Result<JsonResponse, Status>
    where
        F: FnOnce(&mut crate::state::ControlCenterState) -> String,
    {
        let mut guard = self
            .state
            .lock()
            .map_err(|e| Status::internal(e.to_string()))?;
        let json = f(&mut guard);
        Ok(JsonResponse { json })
    }

    fn guard_request<T>(&self, request: &Request<T>) -> Result<(), Status> {
        if let Some(version) = request.metadata().get("x-spanda-api-version") {
            let value = version
                .to_str()
                .map_err(|_| Status::invalid_argument("invalid x-spanda-api-version metadata"))?;
            if !value.trim().is_empty() && value.trim() != crate::versioning::SUPPORTED_API_VERSION
            {
                return Err(Status::invalid_argument(format!(
                    "unsupported api version '{value}'; supported: {}",
                    crate::versioning::SUPPORTED_API_VERSION
                )));
            }
        }
        let token = Self::bearer_token(request);
        let guard = self
            .state
            .lock()
            .map_err(|e| Status::internal(e.to_string()))?;
        let rbac_ctx = guard.api_keys.authenticate(token.as_deref());
        if let Some(context) = &rbac_ctx {
            if !spanda_security::tenant_matches(&guard.tenant_id, &context.tenant_id) {
                return Err(Status::permission_denied("tenant mismatch"));
            }
        }
        let rate_key = rbac_ctx
            .as_ref()
            .map(|context| context.key_id.clone())
            .unwrap_or_else(|| "anonymous".to_string());
        guard.rate_limiter.check(&rate_key).map_err(|retry| {
            Status::resource_exhausted(format!("rate limit exceeded; retry after {retry}s"))
        })
    }

    fn audit_grpc_response(&self, rpc: &str, json: &str, ctx: Option<&RbacContext>) {
        if let Ok(mut guard) = self.state.lock() {
            crate::audit_log::record_grpc_mutation(&mut guard, rpc, json, ctx);
        }
    }

    fn respond_mutation(
        &self,
        rpc: &str,
        json: String,
        ctx: Option<RbacContext>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.audit_grpc_response(rpc, &json, ctx.as_ref());
        Ok(Response::new(JsonResponse { json }))
    }
}

#[tonic::async_trait]
impl ControlCenter for GrpcControlCenter {
    async fn health(&self, _request: Request<Empty>) -> Result<Response<HealthResponse>, Status> {
        Ok(Response::new(HealthResponse {
            status: format!(
                "ok;{};rpcs={}",
                crate::grpc_policy::PROTO_SEMVER,
                crate::grpc_policy::control_center_rpc_count()
            ),
        }))
    }

    async fn get_tenant(&self, request: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        self.with_state(crate::handlers::tenant_info_json)
            .map(Response::new)
    }

    async fn get_dashboard(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        self.with_state(|state| {
            let registry = state.device_registry();
            let fleet =
                spanda_fleet::load_fleet_agent_registry(&spanda_fleet::default_fleet_agents_path());
            let json = serde_json::json!({
                "version": "v1",
                "device_pool": registry.pool_summary(),
                "fleet_agent_count": fleet.agents.len(),
                "alert_count": state.alert_store.list().len(),
            });
            serde_json::to_string(&json).unwrap_or_else(|_| "{}".into())
        })
        .map(Response::new)
    }

    async fn list_devices(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        self.with_state(crate::handlers::devices_list_json)
            .map(Response::new)
    }

    async fn list_audit_mutations(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        self.with_state(|state| crate::handlers::mutation_audit_list_json(state, ctx.as_ref()))
            .map(Response::new)
    }

    async fn get_device(
        &self,
        request: Request<DeviceIdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let device_id = request.into_inner().device_id;
        self.with_state(|state| crate::handlers::device_get_json(state, &device_id))
            .map(Response::new)
    }

    async fn patch_device(
        &self,
        request: Request<DeviceBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let inner = request.into_inner();
        let json = {
            let mut guard = self
                .state
                .lock()
                .map_err(|e| Status::internal(e.to_string()))?;
            crate::handlers::device_patch_json(
                &mut guard,
                &inner.device_id,
                &inner.body_json,
                ctx.as_ref(),
            )
        };
        self.respond_mutation("PatchDevice", json, ctx)
    }

    async fn device_provision(
        &self,
        request: Request<DeviceBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let inner = request.into_inner();
        self.with_state_mut(|state| {
            crate::handlers::device_provision_json(
                state,
                &inner.device_id,
                &inner.body_json,
                ctx.as_ref(),
            )
        })
        .map(Response::new)
    }

    async fn device_assign(
        &self,
        request: Request<DeviceBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let inner = request.into_inner();
        self.with_state_mut(|state| {
            crate::handlers::device_assign_json(
                state,
                &inner.device_id,
                &inner.body_json,
                ctx.as_ref(),
            )
        })
        .map(Response::new)
    }

    async fn device_quarantine(
        &self,
        request: Request<DeviceIdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let device_id = request.into_inner().device_id;
        self.with_state_mut(|state| {
            crate::handlers::device_quarantine_json(state, &device_id, ctx.as_ref())
        })
        .map(Response::new)
    }

    async fn device_trust(
        &self,
        request: Request<DeviceIdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let device_id = request.into_inner().device_id;
        self.with_state_mut(|state| {
            crate::handlers::device_trust_json(state, &device_id, ctx.as_ref())
        })
        .map(Response::new)
    }

    async fn list_fleet_agents(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        Ok(Response::new(JsonResponse {
            json: crate::handlers::fleet_agents_json(),
        }))
    }

    async fn evaluate_readiness(
        &self,
        request: Request<ReadinessRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let body = request.into_inner().body_json;
        self.with_state(|state| crate::handlers::readiness_run_json(state, &body))
            .map(Response::new)
    }

    async fn get_sre_summary(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        self.with_state(crate::handlers::sre_summary_json)
            .map(Response::new)
    }

    async fn list_sre_incidents(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        self.with_state(crate::handlers::sre_incidents_list_json)
            .map(Response::new)
    }

    async fn create_sre_incident(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        if !spanda_security::ApiKeyStore::check(ctx.as_ref(), spanda_security::RbacAction::Operate)
        {
            return Err(Status::permission_denied("bearer token required"));
        }
        let body = request.into_inner().body_json;
        let response = self.with_state_mut(|state| {
            crate::handlers::sre_incidents_create_json(state, &body, ctx.as_ref())
        })?;
        self.audit_grpc_response("CreateSreIncident", &response.json, ctx.as_ref());
        Ok(Response::new(response))
    }

    async fn ack_sre_incident(
        &self,
        request: Request<IncidentBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        if !spanda_security::ApiKeyStore::check(ctx.as_ref(), spanda_security::RbacAction::Operate)
        {
            return Err(Status::permission_denied("bearer token required"));
        }
        let inner = request.into_inner();
        let response = self.with_state_mut(|state| {
            crate::handlers::sre_incident_ack_json(
                state,
                &inner.incident_id,
                &inner.body_json,
                ctx.as_ref(),
            )
        })?;
        self.audit_grpc_response("AckSreIncident", &response.json, ctx.as_ref());
        Ok(Response::new(response))
    }

    async fn resolve_sre_incident(
        &self,
        request: Request<IncidentIdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        if !spanda_security::ApiKeyStore::check(ctx.as_ref(), spanda_security::RbacAction::Operate)
        {
            return Err(Status::permission_denied("bearer token required"));
        }
        let incident_id = request.into_inner().incident_id;
        let response = self.with_state_mut(|state| {
            crate::handlers::sre_incident_resolve_json(state, &incident_id, ctx.as_ref())
        })?;
        self.audit_grpc_response("ResolveSreIncident", &response.json, ctx.as_ref());
        Ok(Response::new(response))
    }

    async fn get_trust_package(
        &self,
        request: Request<TrustPackageRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let package_name = request.into_inner().package_name;
        let query = format!("name={package_name}");
        Ok(Response::new(JsonResponse {
            json: crate::handlers::trust_package_json(&query),
        }))
    }

    async fn get_open_api(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        Ok(Response::new(JsonResponse {
            json: crate::handlers::openapi_json(),
        }))
    }

    async fn get_health_summary(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        self.with_state(crate::handlers::health_summary_json)
            .map(Response::new)
    }

    async fn get_assurance_summary(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        self.with_state(crate::handlers::assurance_summary_json)
            .map(Response::new)
    }

    async fn get_diagnosis_summary(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        self.with_state(crate::handlers::diagnosis_summary_json)
            .map(Response::new)
    }

    async fn get_executive_scorecard(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        self.with_state(crate::handlers::executive_scorecard_json)
            .map(Response::new)
    }

    async fn query_digital_thread(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let query = request.into_inner().query;
        self.with_state(|state| crate::handlers::digital_thread_query_json(state, &query))
            .map(Response::new)
    }

    async fn get_ota_status(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        Ok(Response::new(JsonResponse {
            json: crate::handlers::ota_status_json(),
        }))
    }

    async fn get_otlp_metrics(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        self.with_state(crate::handlers::otlp_metrics_json)
            .map(Response::new)
    }

    async fn get_observability_backend(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        Ok(Response::new(JsonResponse {
            json: crate::handlers::observability_backend_json(),
        }))
    }

    async fn discover_devices(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let query = request.into_inner().query;
        Ok(Response::new(JsonResponse {
            json: crate::handlers::discovery_run_json(&query),
        }))
    }

    async fn run_discovery(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let body = request.into_inner().body_json;
        self.with_state_mut(|state| {
            crate::handlers::discovery_post_json(state, &body, ctx.as_ref())
        })
        .map(Response::new)
    }

    async fn provision_device(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let body = request.into_inner().body_json;
        self.with_state_mut(|state| crate::handlers::provision_run_json(state, &body, ctx.as_ref()))
            .map(Response::new)
    }

    async fn plan_ota(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let body = request.into_inner().body_json;
        let json = self
            .with_state(|state| crate::handlers::ota_plan_json(state, &body, ctx.as_ref()))?
            .json;
        self.respond_mutation("PlanOta", json, ctx)
    }

    async fn execute_ota(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let body = request.into_inner().body_json;
        let json = self
            .with_state(|state| crate::handlers::ota_execute_json(state, &body, ctx.as_ref()))?
            .json;
        self.respond_mutation("ExecuteOta", json, ctx)
    }

    async fn operator_quarantine(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let body = request.into_inner().body_json;
        self.with_state_mut(|state| {
            crate::handlers::operator_quarantine_json(state, &body, ctx.as_ref())
        })
        .map(Response::new)
    }

    async fn operator_mission_approve(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let body = request.into_inner().body_json;
        let json = crate::handlers::operator_mission_approve_json(&body, ctx.as_ref());
        self.respond_mutation("OperatorMissionApprove", json, ctx)
    }

    async fn export_compliance(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let query = request.into_inner().query;
        self.with_state(|state| {
            crate::handlers::compliance_export_json(state, &query, ctx.as_ref())
        })
        .map(Response::new)
    }

    async fn list_robots(&self, request: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        self.with_state(crate::handlers::robots_list_json)
            .map(Response::new)
    }

    async fn list_fleets(&self, request: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        self.with_state(crate::handlers::fleets_list_json)
            .map(Response::new)
    }

    async fn list_alerts(&self, request: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        self.with_state(crate::handlers::alerts_list_json)
            .map(Response::new)
    }

    async fn list_config_snapshots(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        Ok(Response::new(JsonResponse {
            json: crate::handlers::config_snapshots_list_json(),
        }))
    }

    async fn save_config_snapshot(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let body = request.into_inner().body_json;
        self.with_state(|state| {
            crate::handlers::config_snapshots_save_json(state, &body, ctx.as_ref())
        })
        .map(Response::new)
    }

    async fn test_alert(&self, request: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let json = {
            let mut guard = self
                .state
                .lock()
                .map_err(|e| Status::internal(e.to_string()))?;
            crate::handlers::alerts_test_json(&mut guard, ctx.as_ref())
        };
        self.respond_mutation("TestAlert", json, ctx)
    }

    async fn get_device_tree(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        self.with_state(crate::handlers::device_tree_json)
            .map(Response::new)
    }

    async fn get_device_reports(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        self.with_state(crate::handlers::device_reports_json)
            .map(Response::new)
    }

    async fn get_failover_chains(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        self.with_state(crate::handlers::failover_chains_json)
            .map(Response::new)
    }

    async fn list_secrets(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        self.with_state(|state| crate::handlers::secrets_list_json(state, ctx.as_ref()))
            .map(Response::new)
    }

    async fn get_rbac_matrix(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        Ok(Response::new(JsonResponse {
            json: crate::handlers::rbac_matrix_json(),
        }))
    }

    async fn get_analytics_readiness(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let query = request.into_inner().query;
        self.with_state(|state| crate::handlers::analytics_readiness_json(state, &query))
            .map(Response::new)
    }

    async fn export_reports(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let query = request.into_inner().query;
        self.with_state(|state| crate::handlers::reports_export_json(state, &query, ctx.as_ref()))
            .map(Response::new)
    }

    async fn get_observability_traces(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        self.with_state(crate::handlers::observability_traces_json)
            .map(Response::new)
    }

    async fn get_otlp_traces(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        self.with_state(crate::handlers::otlp_traces_json)
            .map(Response::new)
    }

    async fn export_otlp_traces(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let query = request.into_inner().query;
        self.with_state(|state| {
            crate::handlers::otlp_traces_export_json(state, &query, ctx.as_ref())
        })
        .map(Response::new)
    }

    async fn export_otlp_metrics(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let query = request.into_inner().query;
        self.with_state(|state| {
            crate::handlers::otlp_metrics_export_json(state, &query, ctx.as_ref())
        })
        .map(Response::new)
    }

    async fn detect_drift(
        &self,
        request: Request<DriftRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let baseline_id = request.into_inner().baseline_id;
        self.with_state(|state| {
            let query = format!("baseline_id={baseline_id}");
            crate::e3::drift_report(state, &query).body
        })
        .map(Response::new)
    }

    async fn list_compliance_evidence(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        Ok(Response::new(JsonResponse {
            json: crate::handlers::compliance_evidence_list_json(ctx.as_ref()),
        }))
    }

    async fn list_config_approvals(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        Ok(Response::new(JsonResponse {
            json: crate::handlers::config_approvals_list_json(),
        }))
    }

    async fn submit_config_approval(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let body = request.into_inner().body_json;
        let json = crate::handlers::config_approvals_submit_json(&body, ctx.as_ref());
        self.respond_mutation("SubmitConfigApproval", json, ctx)
    }

    async fn approve_config_approval(
        &self,
        request: Request<ApprovalBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let inner = request.into_inner();
        let json = self
            .with_state_mut(|state| {
                crate::handlers::config_approvals_approve_json(
                    state,
                    &inner.approval_id,
                    &inner.body_json,
                    ctx.as_ref(),
                )
            })?
            .json;
        self.respond_mutation("ApproveConfigApproval", json, ctx)
    }

    async fn reject_config_approval(
        &self,
        request: Request<ApprovalBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let inner = request.into_inner();
        let json = self
            .with_state_mut(|state| {
                crate::handlers::config_approvals_reject_json(
                    state,
                    &inner.approval_id,
                    &inner.body_json,
                    ctx.as_ref(),
                )
            })?
            .json;
        self.respond_mutation("RejectConfigApproval", json, ctx)
    }

    async fn evaluate_program_readiness(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let body = request.into_inner().body_json;
        self.with_state(|state| crate::sdk_ops::program_readiness_json(state, &body))
            .map(Response::new)
    }

    async fn evaluate_program_assure(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let body = request.into_inner().body_json;
        self.with_state(|state| crate::sdk_ops::program_assure_json(state, &body))
            .map(Response::new)
    }

    async fn evaluate_program_diagnose(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let body = request.into_inner().body_json;
        self.with_state(|state| crate::sdk_ops::program_diagnose_json(state, &body))
            .map(Response::new)
    }

    async fn evaluate_program_heal(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let body = request.into_inner().body_json;
        self.with_state(|state| crate::sdk_ops::program_heal_json(state, &body))
            .map(Response::new)
    }

    async fn verify_program_hardware(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let body = request.into_inner().body_json;
        self.with_state(|state| crate::sdk_ops::program_verify_hardware_json(state, &body))
            .map(Response::new)
    }

    async fn verify_program_capabilities(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let body = request.into_inner().body_json;
        self.with_state(|state| crate::sdk_ops::program_verify_capabilities_json(state, &body))
            .map(Response::new)
    }

    async fn verify_program_mission(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let body = request.into_inner().body_json;
        self.with_state(|state| crate::sdk_ops::program_verify_mission_json(state, &body))
            .map(Response::new)
    }

    async fn run_program_simulation(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let body = request.into_inner().body_json;
        self.with_state(|state| crate::sdk_ops::program_simulation_json(state, &body))
            .map(Response::new)
    }

    async fn replay_program(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let body = request.into_inner().body_json;
        self.with_state(|state| crate::sdk_ops::program_replay_json(state, &body))
            .map(Response::new)
    }

    async fn get_trust_program(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let query = request.into_inner().query;
        self.with_state(|state| crate::sdk_ops::trust_program_json(state, &query))
            .map(Response::new)
    }

    async fn list_entities(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        self.with_state(crate::sdk_ops::list_entities_json)
            .map(Response::new)
    }

    async fn get_entity(
        &self,
        request: Request<EntityIdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let entity_id = request.into_inner().entity_id;
        self.with_state(|state| crate::sdk_ops::get_entity_json(state, &entity_id))
            .map(Response::new)
    }

    async fn get_entity_health(
        &self,
        request: Request<EntityIdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let entity_id = request.into_inner().entity_id;
        self.with_state(|state| crate::sdk_ops::entity_health_json(state, &entity_id))
            .map(Response::new)
    }

    async fn get_entity_trust(
        &self,
        request: Request<EntityIdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let entity_id = request.into_inner().entity_id;
        self.with_state(|state| crate::sdk_ops::entity_trust_json(state, &entity_id))
            .map(Response::new)
    }

    async fn get_entity_graph(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        self.with_state(crate::sdk_ops::entity_graph_json)
            .map(Response::new)
    }

    async fn get_entity_traceability(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let query = request.into_inner().query;
        self.with_state(|state| crate::sdk_ops::entity_traceability_json(state, &query))
            .map(Response::new)
    }

    async fn query_entities(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let body = request.into_inner().body_json;
        self.with_state(|state| crate::sdk_ops::entity_query_json(state, &body))
            .map(Response::new)
    }

    async fn get_entity_relationships(
        &self,
        request: Request<EntityIdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let entity_id = request.into_inner().entity_id;
        self.with_state(|state| crate::sdk_ops::entity_relationships_json(state, &entity_id))
            .map(Response::new)
    }

    async fn get_entity_readiness(
        &self,
        request: Request<EntityIdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let entity_id = request.into_inner().entity_id;
        self.with_state_mut(|state| crate::sdk_ops::entity_readiness_json(state, &entity_id))
            .map(Response::new)
    }

    async fn verify_entity(
        &self,
        request: Request<EntityBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let inner = request.into_inner();
        self.with_state(|state| {
            crate::sdk_ops::entity_verify_json(state, &inner.entity_id, &inner.body_json)
        })
        .map(Response::new)
    }

    async fn register_entity(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let body = request.into_inner().body_json;
        let json = self
            .with_state_mut(|state| {
                crate::entity_mutations::entity_register_json(state, &body, ctx.as_ref())
            })?
            .json;
        self.respond_mutation("RegisterEntity", json, ctx)
    }

    async fn tag_entity(
        &self,
        request: Request<EntityBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let inner = request.into_inner();
        let json = self
            .with_state_mut(|state| {
                crate::entity_mutations::entity_tag_json(
                    state,
                    &inner.entity_id,
                    &inner.body_json,
                    ctx.as_ref(),
                )
            })?
            .json;
        self.respond_mutation("TagEntity", json, ctx)
    }

    async fn relate_entities(
        &self,
        request: Request<JsonBodyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let body = request.into_inner().body_json;
        let json = self
            .with_state_mut(|state| {
                crate::entity_mutations::entity_relate_json(state, &body, ctx.as_ref())
            })?
            .json;
        self.respond_mutation("RelateEntities", json, ctx)
    }

    async fn sync_entities(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        self.guard_request(&request)?;
        let ctx = self.rbac_from_request(&request);
        let json = self
            .with_state_mut(|state| crate::entity_mutations::entity_sync_json(state, ctx.as_ref()))?
            .json;
        self.respond_mutation("SyncEntities", json, ctx)
    }
}

/// Start tonic gRPC server on `bind` (blocks the current thread's tokio runtime).
pub async fn serve_grpc(bind: String, state: SharedState) -> Result<(), String> {
    // Serve ControlCenter gRPC on a dedicated listener.
    //
    // Parameters:
    // - `bind` — socket address (for example `127.0.0.1:50051`)
    // - `state` — shared Control Center state
    //
    // Returns:
    // Ok when the server stops, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // serve_grpc("127.0.0.1:50051".into(), state).await?;

    let service = GrpcControlCenter { state };
    Server::builder()
        .add_service(ControlCenterServer::new(service))
        .serve(
            bind.parse()
                .map_err(|e: std::net::AddrParseError| e.to_string())?,
        )
        .await
        .map_err(|e| e.to_string())
}

/// Spawn gRPC server on a background thread with its own tokio runtime.
pub fn spawn_grpc_server(bind: String, state: SharedState) {
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("grpc tokio runtime");
        if let Err(error) = runtime.block_on(serve_grpc(bind.clone(), state)) {
            eprintln!("gRPC server on {bind} stopped: {error}");
        }
    });
}
