//! In-memory telemetry store for WASM and embedded hosts without filesystem access.

use crate::error::{TelemetryStoreError, TelemetryStoreResult};
use crate::otlp::render_otlp_from_events;
use crate::prometheus::render_prometheus_from_events;
use crate::record::{HeartbeatIndex, TelemetryEvent};
use crate::store::{stats_from_events, TelemetryStats};
use std::sync::{Mutex, OnceLock};

static MEMORY_STORE: OnceLock<Mutex<MemoryTelemetryStore>> = OnceLock::new();

/// Thread-safe in-memory telemetry buffer.
#[derive(Debug, Default)]
pub struct MemoryTelemetryStore {
    events: Vec<TelemetryEvent>,
    heartbeat_index: HeartbeatIndex,
}

impl MemoryTelemetryStore {
    /// Return the process-global in-memory telemetry store.
    pub fn global() -> &'static Mutex<Self> {
        MEMORY_STORE.get_or_init(|| Mutex::new(Self::default()))
    }

    /// Clear all buffered events and heartbeat indexes.
    pub fn clear(&mut self) {
        self.events.clear();
        self.heartbeat_index = HeartbeatIndex::default();
    }

    /// Append a parsed telemetry event and update heartbeat indexes when relevant.
    pub fn append(&mut self, event: TelemetryEvent) -> TelemetryStoreResult<()> {
        match &event {
            TelemetryEvent::Heartbeat { task_name, timestamp_ms, .. } => {
                self.heartbeat_index.tasks.insert(task_name.clone(), *timestamp_ms);
            }
            TelemetryEvent::DeviceHeartbeat {
                device_id,
                timestamp_ms,
                ..
            } => {
                self.heartbeat_index
                    .devices
                    .insert(device_id.clone(), *timestamp_ms);
            }
            _ => {}
        }
        self.events.push(event);
        Ok(())
    }

    /// Parse one JSONL line and append it to the buffer.
    pub fn append_json_line(&mut self, line: &str) -> TelemetryStoreResult<()> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(());
        }
        let event: TelemetryEvent = serde_json::from_str(trimmed)
            .map_err(|error| TelemetryStoreError::Serialization(error.to_string()))?;
        self.append(event)
    }

    /// Return a copy of all buffered events.
    pub fn read_all(&self) -> Vec<TelemetryEvent> {
        self.events.clone()
    }

    /// Return the latest heartbeat index derived from appended events.
    pub fn heartbeat_index(&self) -> HeartbeatIndex {
        self.heartbeat_index.clone()
    }

    /// Compute aggregate event counts for the buffer.
    pub fn stats(&self) -> TelemetryStats {
        stats_from_events(&self.events, &self.heartbeat_index)
    }
}

/// Clear the global in-memory telemetry buffer.
pub fn memory_clear() -> TelemetryStoreResult<()> {
    MemoryTelemetryStore::global()
        .lock()
        .map_err(|_| TelemetryStoreError::LockPoisoned)
        .map(|mut store| store.clear())
}

/// Append a JSONL telemetry event line to the global in-memory buffer.
pub fn memory_append_json_line(line: &str) -> TelemetryStoreResult<()> {
    MemoryTelemetryStore::global()
        .lock()
        .map_err(|_| TelemetryStoreError::LockPoisoned)?
        .append_json_line(line)
}

/// Return aggregate stats for the global in-memory buffer.
pub fn memory_stats() -> TelemetryStoreResult<TelemetryStats> {
    Ok(MemoryTelemetryStore::global()
        .lock()
        .map_err(|_| TelemetryStoreError::LockPoisoned)?
        .stats())
}

/// Render Prometheus text exposition for the global in-memory buffer.
pub fn memory_render_prometheus() -> TelemetryStoreResult<String> {
    let store = MemoryTelemetryStore::global()
        .lock()
        .map_err(|_| TelemetryStoreError::LockPoisoned)?;
    let events = store.read_all();
    let index = store.heartbeat_index();
    let stats = stats_from_events(&events, &index);
    Ok(render_prometheus_from_events(&events, &stats, &index))
}

/// Render OTLP/JSON metrics for the global in-memory buffer.
pub fn memory_render_otlp_json() -> TelemetryStoreResult<String> {
    let store = MemoryTelemetryStore::global()
        .lock()
        .map_err(|_| TelemetryStoreError::LockPoisoned)?;
    let events = store.read_all();
    let index = store.heartbeat_index();
    let stats = stats_from_events(&events, &index);
    render_otlp_from_events(&events, &stats, &index, "memory")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_stats_and_render() {
        let _ = memory_clear();
        memory_append_json_line(
            r#"{"kind":"sensor","sensor_id":"lidar","sensor_type":"Lidar","value":{"kind":"scan"},"timestamp_ms":1}"#,
        )
        .unwrap();
        let stats = memory_stats().unwrap();
        assert_eq!(stats.sensor_events, 1);
        let prom = memory_render_prometheus().unwrap();
        assert!(prom.contains("spanda_telemetry_events_total"));
        let otlp = memory_render_otlp_json().unwrap();
        assert!(otlp.contains("resourceMetrics"));
        memory_clear().unwrap();
    }
}
