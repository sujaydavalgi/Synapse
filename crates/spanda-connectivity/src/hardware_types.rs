//! Hardware profile and compatibility report types, shared across connectivity and hardware crates.

use serde::{Deserialize, Serialize};

/// Runtime hardware profile describing available sensors, actuators, and connectivity links.
#[derive(Debug, Clone, PartialEq)]
pub struct HardwareProfile {
    pub name: String,
    pub cpu: Option<String>,
    pub memory_mb: Option<f64>,
    pub storage_mb: Option<f64>,
    pub gpu_tops: Option<f64>,
    pub gpu_required: bool,
    pub sensors: Vec<String>,
    pub actuators: Vec<String>,
    pub connectivity: Vec<String>,
    pub battery_wh: Option<f64>,
    pub network_bandwidth_mbps: Option<f64>,
    pub network_latency_ms: Option<f64>,
    pub packet_loss_pct: Option<f64>,
    pub min_control_period_ms: f64,
    pub power_draw_w: f64,
}

/// Severity level for a hardware compatibility report item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompatSeverity {
    Pass,
    Warning,
    Error,
}

/// Single compatibility check result within a hardware verify report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompatItem {
    pub category: String,
    pub message: String,
    pub severity: CompatSeverity,
    pub line: u32,
    pub column: u32,
}
