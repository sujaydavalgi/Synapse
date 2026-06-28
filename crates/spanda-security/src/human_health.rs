//! Opt-in human health telemetry privacy gate for wearables and readiness.
//!
use serde::{Deserialize, Serialize};

/// TOML `[security.human_health]` section from blueprint security config.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HumanHealthSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub require_consent: bool,
    #[serde(default = "default_true")]
    pub audit_health_reads: bool,
    #[serde(default)]
    pub retention_days: u32,
}

fn default_true() -> bool {
    true
}

/// Resolved gate combining config opt-in and runtime environment flag.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HumanHealthGate {
    pub config_enabled: bool,
    pub env_enabled: bool,
    pub active: bool,
    pub require_consent: bool,
    pub audit_health_reads: bool,
    pub retention_days: u32,
}

impl HumanHealthGate {
    pub fn resolve(settings: &HumanHealthSettings) -> Self {
        let env_enabled = std::env::var("SPANDA_HUMAN_HEALTH_ENABLED")
            .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let config_enabled = settings.enabled;
        Self {
            config_enabled,
            env_enabled,
            active: config_enabled && env_enabled,
            require_consent: settings.require_consent,
            audit_health_reads: settings.audit_health_reads,
            retention_days: settings.retention_days,
        }
    }

    pub fn allows_health_telemetry_read(&self) -> bool {
        self.active
    }
}
