//! Shared SDK response types mirroring OpenAPI / domain schemas.
//!
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Operational readiness report envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessReport {
    pub score: Option<u32>,
    pub status: Option<String>,
    pub mission_ready: Option<bool>,
    #[serde(flatten)]
    pub raw: Value,
}

impl ReadinessReport {
    pub fn from_api(value: Value) -> Self {
        let report = value.get("report").cloned().unwrap_or(value.clone());
        let score = report
            .get("score")
            .and_then(|s| s.get("total"))
            .or_else(|| report.get("score"))
            .and_then(|v| v.as_u64())
            .map(|n| n as u32);
        let status = report
            .get("status")
            .and_then(|v| v.as_str())
            .map(String::from);
        let mission_ready = report
            .get("mission_ready")
            .and_then(|v| v.as_bool());
        Self {
            score,
            status,
            mission_ready,
            raw: value,
        }
    }
}

/// Mission assurance report envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssuranceReport {
    pub passed: Option<bool>,
    #[serde(flatten)]
    pub raw: Value,
}

impl AssuranceReport {
    pub fn from_api(value: Value) -> Self {
        let report = value.get("report").cloned().unwrap_or(value.clone());
        let passed = report.get("passed").and_then(|v| v.as_bool());
        Self { passed, raw: value }
    }
}

/// Diagnosis report envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisReport {
    pub passed: Option<bool>,
    #[serde(flatten)]
    pub raw: Value,
}

impl DiagnosisReport {
    pub fn from_api(value: Value) -> Self {
        let report = value.get("report").cloned().unwrap_or(value.clone());
        let passed = report.get("passed").and_then(|v| v.as_bool());
        Self { passed, raw: value }
    }
}

/// Recovery / heal report envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryReport {
    #[serde(flatten)]
    pub raw: Value,
}

/// Entity health snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    #[serde(flatten)]
    pub raw: Value,
}

/// Trust evaluation report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustReport {
    #[serde(flatten)]
    pub raw: Value,
}

/// Unified entity record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readiness_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<String>,
    #[serde(flatten)]
    pub raw: Value,
}

/// Device pool entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    #[serde(flatten)]
    pub raw: Value,
}

/// Simulation planning result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    #[serde(flatten)]
    pub raw: Value,
}

/// Mission trace replay metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    #[serde(flatten)]
    pub raw: Value,
}

/// Package trust report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageTrustReport {
    #[serde(flatten)]
    pub raw: Value,
}

/// Audit event record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    #[serde(flatten)]
    pub raw: Value,
}

/// Hardware verification report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    #[serde(flatten)]
    pub raw: Value,
}

/// Capability verification report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    #[serde(flatten)]
    pub raw: Value,
}

/// Mission verification report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    #[serde(flatten)]
    pub raw: Value,
}

/// Traceability matrix rows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceabilityMatrix {
    #[serde(flatten)]
    pub raw: Value,
}
