//! SpandaClient — thin REST v1 client for Control Center.
//!
use crate::error::{SpandaError, SpandaResult};
use crate::types::{
    AssuranceReport, DiagnosisReport, Entity, HealthReport, PackageTrustReport, ReadinessReport,
    RecoveryReport, ReplayResult, SimulationResult, TrustReport,
};
use serde_json::{json, Value};
use std::env;
use std::time::Duration;

/// Authentication configuration for SDK clients.
#[derive(Debug, Clone, Default)]
pub struct AuthConfig {
    pub api_key: Option<String>,
}

/// Builder for [`SpandaClient`].
#[derive(Debug, Clone)]
pub struct SpandaClientBuilder {
    base_url: String,
    auth: AuthConfig,
    timeout: Duration,
}

impl Default for SpandaClientBuilder {
    fn default() -> Self {
        Self {
            base_url: env::var("SPANDA_CONTROL_CENTER_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8080".into()),
            auth: AuthConfig {
                api_key: env::var("SPANDA_API_KEY").ok(),
            },
            timeout: Duration::from_secs(30),
        }
    }
}

impl SpandaClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.auth.api_key = Some(key.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn build(self) -> SpandaClient {
        SpandaClient {
            base_url: self.base_url.trim_end_matches('/').to_string(),
            auth: self.auth,
            timeout: self.timeout,
        }
    }
}

/// Official Spanda SDK client — delegates to Control Center REST API v1.
#[derive(Debug, Clone)]
pub struct SpandaClient {
    base_url: String,
    auth: AuthConfig,
    timeout: Duration,
}

impl SpandaClient {
    /// Connect to the local Control Center (`http://127.0.0.1:8080`).
    pub fn local() -> Self {
        Self::builder().build()
    }

    pub fn builder() -> SpandaClientBuilder {
        SpandaClientBuilder::new()
    }

    pub fn with_url(base_url: impl Into<String>) -> Self {
        Self::builder().base_url(base_url).build()
    }

    fn correlation_id() -> String {
        format!(
            "rust-sdk-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..12]
        )
    }

    fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<&Value>,
        auth: bool,
    ) -> SpandaResult<Value> {
        let url = format!("{}{}", self.base_url, path);
        let agent = ureq::AgentBuilder::new().timeout(self.timeout).build();
        let mut req = match method {
            "GET" => agent.get(&url),
            "POST" => agent.post(&url),
            "PATCH" => agent.patch(&url),
            _ => {
                return Err(SpandaError::validation(format!(
                    "unsupported method {method}"
                )))
            }
        };
        req = req.set("Accept", "application/json");
        req = req.set("X-Correlation-ID", &Self::correlation_id());
        if auth {
            if let Some(key) = &self.auth.api_key {
                req = req.set("Authorization", &format!("Bearer {key}"));
            }
        }
        if let Some(payload) = body {
            req = req.set("Content-Type", "application/json");
            let resp = req
                .send_json(payload)
                .map_err(|e| SpandaError::connection(e.to_string()))?;
            return Self::parse_response(resp);
        }
        let resp = req
            .call()
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_response(resp)
    }

    fn parse_response(resp: ureq::Response) -> SpandaResult<Value> {
        let status = resp.status();
        let body: Value = resp.into_json().unwrap_or(json!({}));
        if (200..300).contains(&status) {
            return Ok(body);
        }
        let message = body
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("request failed")
            .to_string();
        Err(SpandaError::from_status(status, message))
    }

    fn program_body(file: &str) -> Value {
        json!({ "file": file })
    }

    /// Evaluate operational readiness for a program file.
    pub fn readiness(&self, file_or_project: &str) -> SpandaResult<ReadinessReport> {
        let body = Self::program_body(file_or_project);
        let value = self.request("POST", "/v1/programs/readiness", Some(&body), false)?;
        Ok(ReadinessReport::from_api(value))
    }

    /// Run mission assurance for a program file.
    pub fn assure(&self, file_or_project: &str) -> SpandaResult<AssuranceReport> {
        let body = Self::program_body(file_or_project);
        let value = self.request("POST", "/v1/programs/assure", Some(&body), false)?;
        Ok(AssuranceReport::from_api(value))
    }

    /// Diagnose a program or `.trace` file.
    pub fn diagnose(&self, trace_or_file: &str) -> SpandaResult<DiagnosisReport> {
        let body = Self::program_body(trace_or_file);
        let value = self.request("POST", "/v1/programs/diagnose", Some(&body), false)?;
        Ok(DiagnosisReport::from_api(value))
    }

    /// Evaluate recovery options for a program.
    pub fn heal(&self, target: &str) -> SpandaResult<RecoveryReport> {
        let body = Self::program_body(target);
        let value = self.request("POST", "/v1/programs/recovery/heal", Some(&body), false)?;
        Ok(RecoveryReport { raw: value })
    }

    /// Verify hardware compatibility for a program.
    pub fn verify_hardware(&self, project: &str) -> SpandaResult<Value> {
        let body = Self::program_body(project);
        self.request("POST", "/v1/programs/verify/hardware", Some(&body), false)
    }

    /// Verify robot capabilities for a program.
    pub fn verify_capabilities(&self, project: &str) -> SpandaResult<Value> {
        let body = json!({ "file": project, "traceability": true });
        self.request(
            "POST",
            "/v1/programs/verify/capabilities",
            Some(&body),
            false,
        )
    }

    /// List unified entities (all platform objects).
    pub fn list_entities(&self) -> SpandaResult<Vec<Entity>> {
        let value = self.request("GET", "/v1/entities", None, false)?;
        Self::parse_entity_list(value)
    }

    /// Query entities with filter parameters.
    pub fn query_entities(&self, query: &Value) -> SpandaResult<Value> {
        self.request("POST", "/v1/entities/query", Some(query), false)
    }

    /// Fetch the full entity graph for traversal and visualization.
    pub fn entity_graph(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/entities/graph", None, false)
    }

    /// Get a single entity by id.
    pub fn get_entity(&self, id: &str) -> SpandaResult<Entity> {
        let value = self.request("GET", &format!("/v1/entities/{id}"), None, false)?;
        let raw = value.get("entity").cloned().unwrap_or(value);
        Ok(Self::parse_entity(raw, id))
    }

    /// Relationship edges, impact set, and dependency chain for an entity.
    pub fn entity_relationships(&self, id: &str) -> SpandaResult<Value> {
        self.request(
            "GET",
            &format!("/v1/entities/{id}/relationships"),
            None,
            false,
        )
    }

    /// Health snapshot for any entity kind.
    pub fn entity_health(&self, id: &str) -> SpandaResult<HealthReport> {
        let value = self.request("GET", &format!("/v1/entities/{id}/health"), None, false)?;
        Ok(HealthReport { raw: value })
    }

    /// Readiness snapshot for any entity kind.
    pub fn entity_readiness(&self, id: &str) -> SpandaResult<Value> {
        self.request("GET", &format!("/v1/entities/{id}/readiness"), None, false)
    }

    /// Trust evaluation for any entity kind.
    pub fn entity_trust(&self, id: &str) -> SpandaResult<TrustReport> {
        let value = self.request("GET", &format!("/v1/entities/{id}/trust"), None, false)?;
        Ok(TrustReport { raw: value })
    }

    /// Register or update an entity in the runtime mutation overlay.
    pub fn register_entity(&self, body: &Value) -> SpandaResult<Value> {
        self.request("POST", "/v1/entities/register", Some(body), true)
    }

    /// Add or remove tags on an entity overlay record.
    pub fn tag_entity(&self, id: &str, body: &Value) -> SpandaResult<Value> {
        self.request("POST", &format!("/v1/entities/{id}/tags"), Some(body), true)
    }

    /// Relate two entities in the mutation overlay.
    pub fn relate_entities(&self, body: &Value) -> SpandaResult<Value> {
        self.request("POST", "/v1/entities/relationships", Some(body), true)
    }

    /// Sync mutation overlay entities back to TOML fragments.
    pub fn sync_entities(&self) -> SpandaResult<Value> {
        self.request("POST", "/v1/entities/sync", None, true)
    }

    fn parse_entity_list(value: Value) -> SpandaResult<Vec<Entity>> {
        let entities = value
            .get("entities")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(entities
            .into_iter()
            .filter_map(|raw| {
                let id = raw.get("id")?.as_str()?.to_string();
                Some(Self::parse_entity(raw, &id))
            })
            .collect())
    }

    fn parse_entity(raw: Value, fallback_id: &str) -> Entity {
        let entity_id = raw
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or(fallback_id)
            .to_string();
        Entity {
            id: entity_id,
            kind: raw.get("kind").and_then(|v| v.as_str()).map(String::from),
            entity_type: raw
                .get("entity_type")
                .and_then(|v| v.as_str())
                .map(String::from),
            display_name: raw
                .get("display_name")
                .and_then(|v| v.as_str())
                .map(String::from),
            health_status: raw
                .get("health_status")
                .and_then(|v| v.as_str())
                .map(String::from),
            readiness_status: raw
                .get("readiness_status")
                .and_then(|v| v.as_str())
                .map(String::from),
            trust_status: raw
                .get("trust_status")
                .and_then(|v| v.as_str())
                .map(String::from),
            lifecycle_state: raw
                .get("lifecycle_state")
                .and_then(|v| v.as_str())
                .map(String::from),
            raw,
        }
    }

    /// List devices in the device pool.
    pub fn list_devices(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/devices", None, true)
    }

    /// Provision a device (requires auth).
    pub fn provision_device(&self, device_id: &str, body: &Value) -> SpandaResult<Value> {
        self.request(
            "POST",
            &format!("/v1/devices/{device_id}/provision"),
            Some(body),
            true,
        )
    }

    /// Plan or execute simulation for a program (`execute: true` runs the driver).
    pub fn run_simulation(&self, project: &str, execute: bool) -> SpandaResult<SimulationResult> {
        let body = json!({ "file": project, "execute": execute });
        let value = self.request("POST", "/v1/programs/simulation", Some(&body), false)?;
        Ok(SimulationResult { raw: value })
    }

    /// Load or verify mission trace replay (`deterministic` / `playback` flags).
    pub fn replay_with_options(
        &self,
        trace: &str,
        deterministic: bool,
        playback: bool,
    ) -> SpandaResult<ReplayResult> {
        let body = json!({
            "file": trace,
            "deterministic": deterministic,
            "playback": playback,
        });
        let value = self.request("POST", "/v1/programs/replay", Some(&body), false)?;
        Ok(ReplayResult { raw: value })
    }

    /// Load mission trace replay metadata (inspect only).
    pub fn replay(&self, trace: &str) -> SpandaResult<ReplayResult> {
        self.replay_with_options(trace, false, false)
    }

    /// Get health for an entity.
    pub fn get_health(&self, entity_id: &str) -> SpandaResult<HealthReport> {
        let value = self.request(
            "GET",
            &format!("/v1/entities/{entity_id}/health"),
            None,
            false,
        )?;
        Ok(HealthReport { raw: value })
    }

    /// Get trust metadata for an entity.
    pub fn get_trust(&self, entity_id: &str) -> SpandaResult<TrustReport> {
        let value = self.request(
            "GET",
            &format!("/v1/entities/{entity_id}/trust"),
            None,
            false,
        )?;
        Ok(TrustReport { raw: value })
    }

    /// Evaluate package trust.
    pub fn get_package_trust(
        &self,
        package: &str,
        version: Option<&str>,
    ) -> SpandaResult<PackageTrustReport> {
        let mut path = format!("/v1/trust/package?name={package}");
        if let Some(v) = version {
            path.push_str(&format!("&version={v}"));
        }
        let value = self.request("GET", &path, None, false)?;
        Ok(PackageTrustReport { raw: value })
    }

    /// Service liveness check.
    pub fn health_check(&self) -> SpandaResult<Value> {
        self.request("GET", "/v1/health", None, false)
    }

    /// Call JSON-RPC gateway (`POST /v1/rpc`) with a gRPC-style method name.
    pub fn rpc(&self, method: &str, params: Option<&Value>) -> SpandaResult<Value> {
        let body = json!({
            "method": method,
            "params": params.unwrap_or(&json!({})),
        });
        self.request("POST", "/v1/rpc", Some(&body), false)
            .and_then(|value| {
                value
                    .get("result")
                    .cloned()
                    .ok_or_else(|| SpandaError::validation("rpc response missing result"))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_client_uses_default_url() {
        let client = SpandaClient::local();
        assert!(client.base_url.contains("127.0.0.1"));
    }

    #[test]
    fn program_body_includes_file() {
        let body = SpandaClient::program_body("rover.sd");
        assert_eq!(body["file"], "rover.sd");
    }
}
