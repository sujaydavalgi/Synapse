//! Stable plugin host API surface (no internal crate exposure).

use crate::capability::{enforce_capability, CapabilitySet};
use crate::error::PluginResult;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginApiSurface {
    Entity,
    Readiness,
    Assurance,
    Diagnosis,
    Recovery,
    Health,
    Trust,
    Telemetry,
    Report,
}

impl PluginApiSurface {
    pub fn read_capability(self) -> &'static str {
        match self {
            Self::Entity => "entity.read",
            Self::Readiness => "readiness.read",
            Self::Assurance => "assurance.read",
            Self::Diagnosis => "diagnosis.read",
            Self::Recovery => "recovery.read",
            Self::Health => "health.read",
            Self::Trust => "trust.read",
            Self::Telemetry => "telemetry.read",
            Self::Report => "report.generate",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PluginApiContext {
    pub plugin_name: String,
    capabilities: CapabilitySet,
}

impl PluginApiContext {
    pub fn new(plugin_name: impl Into<String>, capabilities: CapabilitySet) -> Self {
        Self {
            plugin_name: plugin_name.into(),
            capabilities,
        }
    }

    pub fn capabilities(&self) -> &CapabilitySet {
        &self.capabilities
    }

    pub fn plugin_name(&self) -> &str {
        &self.plugin_name
    }

    pub(crate) fn gate(&self, surface: PluginApiSurface) -> PluginResult<()> {
        enforce_capability(&self.capabilities, surface.read_capability())
    }

    pub(crate) fn gate_entity(&self) -> PluginResult<()> {
        self.gate_surface(PluginApiSurface::Entity)
    }

    pub(crate) fn gate_readiness(&self) -> PluginResult<()> {
        self.gate_surface(PluginApiSurface::Readiness)
    }

    pub(crate) fn gate_assurance(&self) -> PluginResult<()> {
        self.gate_surface(PluginApiSurface::Assurance)
    }

    pub(crate) fn gate_diagnosis(&self) -> PluginResult<()> {
        self.gate_surface(PluginApiSurface::Diagnosis)
    }

    pub(crate) fn gate_recovery(&self) -> PluginResult<()> {
        self.gate_surface(PluginApiSurface::Recovery)
    }

    pub(crate) fn gate_health(&self) -> PluginResult<()> {
        self.gate_surface(PluginApiSurface::Health)
    }

    pub(crate) fn gate_trust(&self) -> PluginResult<()> {
        self.gate_surface(PluginApiSurface::Trust)
    }

    pub(crate) fn gate_telemetry(&self) -> PluginResult<()> {
        self.gate_surface(PluginApiSurface::Telemetry)
    }

    pub(crate) fn gate_report(&self) -> PluginResult<()> {
        self.gate_surface(PluginApiSurface::Report)
    }

    fn gate_surface(&self, surface: PluginApiSurface) -> PluginResult<()> {
        enforce_capability(&self.capabilities, surface.read_capability())
    }

    pub fn entity_read(&self, entity_id: &str) -> PluginResult<Value> {
        self.gate_surface(PluginApiSurface::Entity)?;
        Ok(json!({
            "entity_id": entity_id,
            "source": "entity-api",
            "plugin": self.plugin_name,
        }))
    }

    pub fn readiness_read(&self, mission_id: &str) -> PluginResult<Value> {
        self.gate_surface(PluginApiSurface::Readiness)?;
        Ok(json!({
            "mission_id": mission_id,
            "status": "unknown",
            "source": "readiness-api",
        }))
    }

    pub fn assurance_read(&self, subject: &str) -> PluginResult<Value> {
        self.gate_surface(PluginApiSurface::Assurance)?;
        Ok(json!({ "subject": subject, "source": "assurance-api" }))
    }

    pub fn diagnosis_read(&self, target: &str) -> PluginResult<Value> {
        self.gate_surface(PluginApiSurface::Diagnosis)?;
        Ok(json!({ "target": target, "source": "diagnosis-api" }))
    }

    pub fn recovery_read(&self, incident_id: &str) -> PluginResult<Value> {
        self.gate_surface(PluginApiSurface::Recovery)?;
        Ok(json!({ "incident_id": incident_id, "source": "recovery-api" }))
    }

    pub fn health_read(&self, entity_id: &str) -> PluginResult<Value> {
        self.gate_surface(PluginApiSurface::Health)?;
        Ok(json!({ "entity_id": entity_id, "source": "health-api" }))
    }

    pub fn trust_read(&self, subject: &str) -> PluginResult<Value> {
        self.gate_surface(PluginApiSurface::Trust)?;
        Ok(json!({ "subject": subject, "source": "trust-api" }))
    }

    pub fn telemetry_read(&self, stream: &str) -> PluginResult<Value> {
        self.gate_surface(PluginApiSurface::Telemetry)?;
        Ok(json!({ "stream": stream, "source": "telemetry-api" }))
    }

    pub fn report_generate(&self, template: &str, data: Value) -> PluginResult<Value> {
        self.gate_surface(PluginApiSurface::Report)?;
        Ok(json!({
            "template": template,
            "data": data,
            "source": "report-api",
        }))
    }
}
