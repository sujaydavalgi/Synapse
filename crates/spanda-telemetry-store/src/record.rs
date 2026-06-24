//! Persisted telemetry event records.

use serde::{Deserialize, Serialize};

/// A single persisted telemetry, sensor, heartbeat, or health event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TelemetryEvent {
    /// Device or IoT metric sample.
    #[serde(rename = "device")]
    Device {
        device_id: String,
        metric: String,
        value: serde_json::Value,
        timestamp_ms: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        robot_id: Option<String>,
    },
    /// Robot sensor reading.
    #[serde(rename = "sensor")]
    Sensor {
        sensor_id: String,
        sensor_type: String,
        value: serde_json::Value,
        timestamp_ms: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        robot_id: Option<String>,
    },
    /// Task heartbeat sample (throttled in the runtime).
    #[serde(rename = "heartbeat")]
    Heartbeat {
        task_name: String,
        timestamp_ms: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        robot_id: Option<String>,
    },
    /// Health status transition.
    #[serde(rename = "health")]
    Health {
        target: String,
        status: String,
        timestamp_ms: f64,
    },
}

impl TelemetryEvent {
    pub fn timestamp_ms(&self) -> f64 {
        match self {
            Self::Device { timestamp_ms, .. }
            | Self::Sensor { timestamp_ms, .. }
            | Self::Heartbeat { timestamp_ms, .. }
            | Self::Health { timestamp_ms, .. } => *timestamp_ms,
        }
    }
}

/// Latest heartbeat per task for fast liveness queries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatIndex {
    pub tasks: std::collections::HashMap<String, f64>,
}
