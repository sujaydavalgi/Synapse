//! Real platform API host backing for plugin `PluginApiContext`.

use crate::api::PluginApiContext;
use crate::capability::CapabilitySet;
use crate::error::PluginResult;
use serde_json::{json, Value};
use spanda_config::ResolvedSystemConfig;
use spanda_readiness::{EntityHealthOptions, EntityReadinessOptions};
use spanda_trust::EntityTrustOptions;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct PluginApiHost {
    pub project_root: PathBuf,
    pub resolved: Option<Arc<ResolvedSystemConfig>>,
}

impl PluginApiHost {
    pub fn from_project_root(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
            resolved: None,
        }
    }

    pub fn with_resolved(mut self, resolved: Arc<ResolvedSystemConfig>) -> Self {
        self.resolved = Some(resolved);
        self
    }

    pub fn context_for(&self, plugin_name: &str, capabilities: CapabilitySet) -> PluginApiHostContext {
        PluginApiHostContext {
            inner: PluginApiContext::new(plugin_name, capabilities),
            host: self.clone(),
        }
    }
}

pub struct PluginApiHostContext {
    inner: PluginApiContext,
    host: PluginApiHost,
}

impl PluginApiHostContext {
    pub fn entity_read(&self, entity_id: &str) -> PluginResult<Value> {
        self.inner.gate_entity()?;
        let registry = self
            .host
            .resolved
            .as_ref()
            .map(|r| r.entity_registry())
            .unwrap_or_default();
        let entity = registry.get(entity_id).cloned();
        Ok(json!({
            "entity_id": entity_id,
            "entity": entity,
            "source": "entity-api",
            "plugin": self.inner.plugin_name(),
        }))
    }

    pub fn readiness_read(&self, entity_id: &str) -> PluginResult<Value> {
        self.inner.gate_readiness()?;
        let Some(resolved) = self.host.resolved.as_ref() else {
            return Ok(json!({
                "entity_id": entity_id,
                "status": "unknown",
                "source": "readiness-api",
                "detail": "no resolved system config",
            }));
        };
        let registry = resolved.entity_registry();
        let mut options = EntityReadinessOptions::default();
        let report = spanda_readiness::evaluate_entity_readiness(
            entity_id,
            &registry,
            resolved,
            &mut options,
        );
        Ok(json!({
            "entity_id": entity_id,
            "report": report,
            "source": "readiness-api",
        }))
    }

    pub fn health_read(&self, entity_id: &str) -> PluginResult<Value> {
        self.inner.gate_health()?;
        let Some(resolved) = self.host.resolved.as_ref() else {
            return Ok(json!({
                "entity_id": entity_id,
                "source": "health-api",
                "detail": "no resolved system config",
            }));
        };
        let registry = resolved.entity_registry();
        let options = EntityHealthOptions::default();
        let report =
            spanda_readiness::evaluate_entity_health(entity_id, &registry, resolved, &options);
        Ok(json!({
            "entity_id": entity_id,
            "report": report,
            "source": "health-api",
        }))
    }

    pub fn trust_read(&self, entity_id: &str) -> PluginResult<Value> {
        self.inner.gate_trust()?;
        let Some(resolved) = self.host.resolved.as_ref() else {
            return Ok(json!({
                "entity_id": entity_id,
                "source": "trust-api",
                "detail": "no resolved system config",
            }));
        };
        let registry = resolved.entity_registry();
        let options = EntityTrustOptions::default();
        let report = spanda_trust::evaluate_entity_trust(entity_id, &registry, resolved, &options);
        Ok(json!({
            "entity_id": entity_id,
            "report": report,
            "source": "trust-api",
        }))
    }

    pub fn assurance_read(&self, subject: &str) -> PluginResult<Value> {
        self.inner.gate_assurance()?;
        Ok(json!({ "subject": subject, "source": "assurance-api", "status": "available" }))
    }

    pub fn diagnosis_read(&self, target: &str) -> PluginResult<Value> {
        self.inner.gate_diagnosis()?;
        Ok(json!({ "target": target, "source": "diagnosis-api", "status": "available" }))
    }

    pub fn recovery_read(&self, incident_id: &str) -> PluginResult<Value> {
        self.inner.gate_recovery()?;
        Ok(json!({ "incident_id": incident_id, "source": "recovery-api", "status": "available" }))
    }

    pub fn telemetry_read(&self, stream: &str) -> PluginResult<Value> {
        self.inner.gate_telemetry()?;
        let stats = spanda_telemetry_store::memory_stats().ok();
        Ok(json!({
            "stream": stream,
            "source": "telemetry-api",
            "stats": stats,
        }))
    }

    pub fn report_generate(&self, template: &str, data: Value) -> PluginResult<Value> {
        self.inner.gate_report()?;
        Ok(json!({
            "template": template,
            "data": data,
            "source": "report-api",
            "generated": true,
        }))
    }
}
