//! Native tonic gRPC client — optional `grpc` feature on `spanda-sdk`.
//!
use crate::error::{SpandaError, SpandaResult};
use serde_json::Value;
use tonic::transport::Channel;

pub mod spanda_v1 {
    tonic::include_proto!("spanda.v1");
}

use spanda_v1::control_center_client::ControlCenterClient;
use spanda_v1::{EntityBodyRequest, EntityIdRequest, JsonBodyRequest, QueryRequest};

/// Async gRPC client for Control Center (`spanda.v1.ControlCenter`).
pub struct GrpcClient {
    inner: ControlCenterClient<Channel>,
    api_key: Option<String>,
}

impl GrpcClient {
    /// Connect to a gRPC endpoint (for example `http://127.0.0.1:50051`).
    pub async fn connect(endpoint: impl Into<String>) -> SpandaResult<Self> {
        let api_key = std::env::var("SPANDA_API_KEY")
            .ok()
            .filter(|key| !key.is_empty());
        let channel = Channel::from_shared(endpoint.into())
            .map_err(|e| SpandaError::connection(e.to_string()))?
            .connect()
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Ok(Self {
            inner: ControlCenterClient::new(channel),
            api_key,
        })
    }

    fn bearer_metadata(&self) -> Option<tonic::metadata::MetadataValue<tonic::metadata::Ascii>> {
        self.api_key
            .as_ref()
            .and_then(|key| tonic::metadata::MetadataValue::try_from(format!("Bearer {key}")).ok())
    }

    fn with_bearer<T>(&self, mut request: tonic::Request<T>) -> tonic::Request<T> {
        if let Some(value) = self.bearer_metadata() {
            request.metadata_mut().insert("authorization", value);
        }
        request
    }

    /// Blocking connect helper for scripts without an async runtime.
    pub fn connect_blocking(endpoint: impl Into<String>) -> SpandaResult<Self> {
        tokio::runtime::Runtime::new()
            .map_err(|e| SpandaError::connection(e.to_string()))?
            .block_on(Self::connect(endpoint))
    }

    fn parse_json(raw: String) -> SpandaResult<Value> {
        serde_json::from_str(&raw).map_err(|e| SpandaError::validation(e.to_string()))
    }

    fn program_body(file: &str, extra: Value) -> String {
        let mut body = serde_json::json!({ "file": file });
        if let Some(obj) = body.as_object_mut() {
            if let Some(extra_obj) = extra.as_object() {
                for (key, value) in extra_obj {
                    obj.insert(key.clone(), value.clone());
                }
            }
        }
        body.to_string()
    }

    /// Evaluate program readiness via `EvaluateProgramReadiness`.
    pub async fn readiness(&mut self, file: &str) -> SpandaResult<Value> {
        let body = Self::program_body(file, Value::Null);
        let resp = self
            .inner
            .evaluate_program_readiness(JsonBodyRequest { body_json: body })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Evaluate program assurance via `EvaluateProgramAssure`.
    pub async fn assure(&mut self, file: &str) -> SpandaResult<Value> {
        let body = Self::program_body(file, Value::Null);
        let resp = self
            .inner
            .evaluate_program_assure(JsonBodyRequest { body_json: body })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Run program simulation via `RunProgramSimulation`.
    pub async fn run_simulation(&mut self, file: &str, execute: bool) -> SpandaResult<Value> {
        let body = Self::program_body(file, serde_json::json!({ "execute": execute }));
        let resp = self
            .inner
            .run_program_simulation(JsonBodyRequest { body_json: body })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Replay or inspect a mission trace via `ReplayProgram`.
    pub async fn replay(
        &mut self,
        file: &str,
        deterministic: bool,
        playback: bool,
    ) -> SpandaResult<Value> {
        let body = Self::program_body(
            file,
            serde_json::json!({
                "deterministic": deterministic,
                "playback": playback,
            }),
        );
        let resp = self
            .inner
            .replay_program(JsonBodyRequest { body_json: body })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// List unified entities via `ListEntities`.
    pub async fn list_entities(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .list_entities(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Fetch one entity via `GetEntity`.
    pub async fn get_entity(&mut self, entity_id: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_entity(EntityIdRequest {
                entity_id: entity_id.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Fetch the entity graph via `GetEntityGraph`.
    pub async fn entity_graph(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_entity_graph(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Unified traceability via `GetEntityTraceability`.
    pub async fn entity_traceability(
        &mut self,
        entity_id: Option<&str>,
        capability: Option<&str>,
        device_id: Option<&str>,
    ) -> SpandaResult<Value> {
        let mut parts = Vec::new();
        if let Some(id) = entity_id {
            parts.push(format!("entity_id={id}"));
        }
        if let Some(cap) = capability {
            parts.push(format!("capability={cap}"));
        }
        if let Some(dev) = device_id {
            parts.push(format!("device_id={dev}"));
        }
        let query = parts.join("&");
        let resp = self
            .inner
            .get_entity_traceability(QueryRequest { query })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Query entities via `QueryEntities`.
    pub async fn query_entities(&mut self, body: &Value) -> SpandaResult<Value> {
        let resp = self
            .inner
            .query_entities(JsonBodyRequest {
                body_json: body.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Relationship edges via `GetEntityRelationships`.
    pub async fn entity_relationships(&mut self, entity_id: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_entity_relationships(EntityIdRequest {
                entity_id: entity_id.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Health snapshot via `GetEntityHealth`.
    pub async fn entity_health(&mut self, entity_id: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_entity_health(EntityIdRequest {
                entity_id: entity_id.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Readiness snapshot via `GetEntityReadiness`.
    pub async fn entity_readiness(&mut self, entity_id: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_entity_readiness(EntityIdRequest {
                entity_id: entity_id.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Trust evaluation via `GetEntityTrust`.
    pub async fn entity_trust(&mut self, entity_id: &str) -> SpandaResult<Value> {
        let resp = self
            .inner
            .get_entity_trust(EntityIdRequest {
                entity_id: entity_id.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Unified verification via `VerifyEntity`.
    pub async fn entity_verify(&mut self, entity_id: &str, body: &Value) -> SpandaResult<Value> {
        let resp = self
            .inner
            .verify_entity(EntityBodyRequest {
                entity_id: entity_id.to_string(),
                body_json: body.to_string(),
            })
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Register or update an entity via `RegisterEntity` (Bearer API key required).
    pub async fn register_entity(&mut self, body: &Value) -> SpandaResult<Value> {
        let request = self.with_bearer(tonic::Request::new(JsonBodyRequest {
            body_json: body.to_string(),
        }));
        let resp = self
            .inner
            .register_entity(request)
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Add or remove tags via `TagEntity` (Bearer API key required).
    pub async fn tag_entity(&mut self, entity_id: &str, body: &Value) -> SpandaResult<Value> {
        let request = self.with_bearer(tonic::Request::new(EntityBodyRequest {
            entity_id: entity_id.to_string(),
            body_json: body.to_string(),
        }));
        let resp = self
            .inner
            .tag_entity(request)
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Relate two entities via `RelateEntities` (Bearer API key required).
    pub async fn relate_entities(&mut self, body: &Value) -> SpandaResult<Value> {
        let request = self.with_bearer(tonic::Request::new(JsonBodyRequest {
            body_json: body.to_string(),
        }));
        let resp = self
            .inner
            .relate_entities(request)
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// Sync overlay to TOML via `SyncEntities` (Bearer API key required).
    pub async fn sync_entities(&mut self) -> SpandaResult<Value> {
        let request = self.with_bearer(tonic::Request::new(spanda_v1::Empty {}));
        let resp = self
            .inner
            .sync_entities(request)
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }

    /// List devices via `ListDevices`.
    pub async fn list_devices(&mut self) -> SpandaResult<Value> {
        let resp = self
            .inner
            .list_devices(spanda_v1::Empty {})
            .await
            .map_err(|e| SpandaError::connection(e.to_string()))?;
        Self::parse_json(resp.into_inner().json)
    }
}
