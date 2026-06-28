//! Fully resolved system configuration consumed by runtime and verification.
//!
use crate::device_identity::{DeviceIdentityRecord, DeviceRegistry};
use crate::device_tree::DeviceTree;
use crate::human_entities::HumanRegistry;
use crate::layer::ConfigGraph;
use crate::manifest::SpandaManifest;
use crate::mapping::LogicalPhysicalMap;
use crate::validation::ConfigValidationReport;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Final merged configuration for a Spanda autonomous system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedSystemConfig {
    pub project_root: PathBuf,
    pub manifest: SpandaManifest,
    pub raw: toml::Value,
    pub layers_applied: Vec<String>,
    pub fragments_loaded: Vec<String>,
    pub device_tree: DeviceTree,
    pub device_registry: DeviceRegistry,
    pub human_registry: HumanRegistry,
    pub logical_map: LogicalPhysicalMap,
    pub providers: Vec<String>,
    pub packages: Vec<String>,
    pub validation: ConfigValidationReport,
    pub graph: ConfigGraph,
}

impl ResolvedSystemConfig {
    pub fn project_name(&self) -> &str {
        self.manifest
            .project
            .as_ref()
            .map(|p| p.name.as_str())
            .unwrap_or("unknown")
    }

    pub fn fleet_id(&self) -> Option<&str> {
        self.device_tree.fleet.as_ref().map(|f| f.id.as_str())
    }

    pub fn robot_ids(&self) -> Vec<&str> {
        self.device_tree
            .fleet
            .as_ref()
            .map(|f| f.robots.iter().map(|r| r.id.as_str()).collect())
            .unwrap_or_default()
    }

    pub fn human_ids(&self) -> Vec<&str> {
        self.human_registry
            .humans
            .iter()
            .map(|h| h.id.as_str())
            .collect()
    }

    pub fn section(&self, key: &str) -> Option<&toml::Value> {
        self.raw.get(key)
    }

    pub fn health_policy_for(&self, robot_id: &str) -> Option<&toml::Value> {
        self.raw
            .get("health")
            .and_then(|h| h.get("robots"))
            .and_then(|robots| robots.get(robot_id))
    }

    pub fn security_identity_for(&self, device_id: &str) -> Option<&toml::Value> {
        self.raw
            .get("security")
            .and_then(|s| s.get("devices"))
            .and_then(|d| d.get(device_id))
    }

    pub fn readiness_config(&self) -> Option<&toml::Value> {
        self.raw.get("readiness")
    }

    pub fn human_health_gate(&self) -> spanda_security::HumanHealthGate {
        let settings: spanda_security::HumanHealthSettings = self
            .raw
            .get("security")
            .and_then(|security| security.get("human_health"))
            .and_then(|section| toml::from_str(&toml::to_string(section).unwrap_or_default()).ok())
            .unwrap_or_default();
        spanda_security::HumanHealthGate::resolve(&settings)
    }

    pub fn assurance_config(&self) -> Option<&toml::Value> {
        self.raw.get("assurance")
    }

    pub fn recovery_config(&self) -> Option<&toml::Value> {
        self.raw.get("recovery")
    }

    pub fn mission_config(&self) -> Option<&toml::Value> {
        self.raw.get("mission")
    }

    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn raw_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.raw)
    }

    pub fn device_by_logical_name(&self, logical: &str) -> Vec<&DeviceIdentityRecord> {
        self.device_registry.by_logical_name(logical)
    }

    pub fn traceability_rows(&self) -> Vec<crate::device_identity::TraceabilityRow> {
        crate::device_identity::traceability_rows(&self.device_registry)
    }

    /// Unified entity registry projecting all configured platform objects.
    pub fn entity_registry(&self) -> crate::entity::EntityRegistry {
        crate::entity::build_entity_registry(self)
    }
}
