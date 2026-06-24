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
        #[serde(skip_serializing_if = "Option::is_none", default)]
        session_id: Option<String>,
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
        #[serde(skip_serializing_if = "Option::is_none", default)]
        session_id: Option<String>,
    },
    /// Task heartbeat sample (throttled in the runtime).
    #[serde(rename = "heartbeat")]
    Heartbeat {
        task_name: String,
        timestamp_ms: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        robot_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        session_id: Option<String>,
    },
    /// IoT device or fleet agent liveness sample.
    #[serde(rename = "device_heartbeat")]
    DeviceHeartbeat {
        device_id: String,
        timestamp_ms: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        robot_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        protocol: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        session_id: Option<String>,
    },
    /// Health status transition.
    #[serde(rename = "health")]
    Health {
        target: String,
        status: String,
        timestamp_ms: f64,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        session_id: Option<String>,
    },
    /// Run session boundary (start/end) linking source and mission trace.
    #[serde(rename = "session")]
    Session {
        session_id: String,
        phase: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mission_trace_path: Option<String>,
        timestamp_ms: f64,
    },
    /// End-of-run scheduler telemetry snapshot (`--metrics-json` payload).
    #[serde(rename = "runtime_metrics")]
    RuntimeMetrics {
        session_id: String,
        metrics: serde_json::Value,
        timestamp_ms: f64,
    },
}

impl TelemetryEvent {
    pub fn timestamp_ms(&self) -> f64 {
        match self {
            Self::Device { timestamp_ms, .. }
            | Self::Sensor { timestamp_ms, .. }
            |             Self::Heartbeat { timestamp_ms, .. }
            | Self::DeviceHeartbeat { timestamp_ms, .. }
            | Self::Health { timestamp_ms, .. }
            | Self::Session { timestamp_ms, .. }
            | Self::RuntimeMetrics { timestamp_ms, .. } => *timestamp_ms,
        }
    }

    pub fn session_id(&self) -> Option<&str> {
        match self {
            Self::Device { session_id, .. }
            | Self::Sensor { session_id, .. }
            | Self::Heartbeat { session_id, .. }
            | Self::DeviceHeartbeat { session_id, .. }
            | Self::Health { session_id, .. } => session_id.as_deref(),
            Self::Session { session_id, .. } | Self::RuntimeMetrics { session_id, .. } => {
                Some(session_id.as_str())
            }
        }
    }
}

/// Latest heartbeat per task and device for fast liveness queries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatIndex {
    pub tasks: std::collections::HashMap<String, f64>,
    #[serde(default)]
    pub devices: std::collections::HashMap<String, f64>,
}
