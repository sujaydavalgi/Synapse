//! Append-only JSONL telemetry store with a heartbeat index sidecar.

use crate::error::{TelemetryStoreError, TelemetryStoreResult};
use crate::record::{HeartbeatIndex, TelemetryEvent};
use serde::Serialize;
use serde_json::Value as JsonValue;
use spanda_runtime::serialize::runtime_to_json_string;
use spanda_runtime::value::RuntimeValue;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static SESSION_ENABLED: AtomicBool = AtomicBool::new(false);
static GLOBAL_STORE: OnceLock<Mutex<PersistentTelemetryStore>> = OnceLock::new();

/// Default append-only event log under the project `.spanda/` directory.
pub fn default_store_path() -> PathBuf {
    PathBuf::from(".spanda/telemetry-store.jsonl")
}

/// Default heartbeat index sidecar.
pub fn default_heartbeat_index_path() -> PathBuf {
    PathBuf::from(".spanda/telemetry-heartbeats.json")
}

/// Resolve store path from `SPANDA_TELEMETRY_STORE_PATH` or the default.
pub fn resolve_store_path() -> PathBuf {
    std::env::var("SPANDA_TELEMETRY_STORE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_store_path())
}

/// Resolve heartbeat index path from env or sibling of the store file.
pub fn resolve_heartbeat_index_path(store_path: &Path) -> PathBuf {
    std::env::var("SPANDA_TELEMETRY_HEARTBEAT_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            store_path
                .parent()
                .map(|dir| dir.join("telemetry-heartbeats.json"))
                .unwrap_or_else(default_heartbeat_index_path)
        })
}

/// Return true when `SPANDA_TELEMETRY_STORE` is `1` or `true`.
pub fn env_persist_enabled() -> bool {
    std::env::var("SPANDA_TELEMETRY_STORE")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Enable or disable persistence for the current process session.
pub fn configure_session_persist(enabled: bool) {
    SESSION_ENABLED.store(enabled, Ordering::SeqCst);
}

/// Return true when persistence is enabled via env or the current session.
pub fn persist_enabled() -> bool {
    SESSION_ENABLED.load(Ordering::SeqCst) || env_persist_enabled()
}

/// Shared process-global store handle.
pub fn global_store() -> &'static Mutex<PersistentTelemetryStore> {
    GLOBAL_STORE.get_or_init(|| {
        Mutex::new(PersistentTelemetryStore::open(
            resolve_store_path(),
            resolve_heartbeat_index_path(&resolve_store_path()),
        ))
    })
}

/// Append an event when persistence is enabled.
pub fn append_event(event: TelemetryEvent) -> TelemetryStoreResult<()> {
    if !persist_enabled() {
        return Ok(());
    }
    global_store().lock().unwrap().append(event)
}

/// Record device telemetry when persistence is enabled.
pub fn record_device_telemetry(
    device_id: impl Into<String>,
    metric: impl Into<String>,
    value: &RuntimeValue,
    timestamp_ms: f64,
    robot_id: Option<&str>,
) -> TelemetryStoreResult<()> {
    if !persist_enabled() {
        return Ok(());
    }
    let value = runtime_value_json(value)?;
    append_event(TelemetryEvent::Device {
        device_id: device_id.into(),
        metric: metric.into(),
        value,
        timestamp_ms,
        robot_id: robot_id.map(str::to_string),
    })
}

/// Record a sensor reading when persistence is enabled.
pub fn record_sensor_reading(
    sensor_id: impl Into<String>,
    sensor_type: impl Into<String>,
    value: &RuntimeValue,
    timestamp_ms: f64,
    robot_id: Option<&str>,
) -> TelemetryStoreResult<()> {
    if !persist_enabled() {
        return Ok(());
    }
    let value = runtime_value_json(value)?;
    append_event(TelemetryEvent::Sensor {
        sensor_id: sensor_id.into(),
        sensor_type: sensor_type.into(),
        value,
        timestamp_ms,
        robot_id: robot_id.map(str::to_string),
    })
}

/// Record a task heartbeat and update the latest index.
pub fn record_task_heartbeat(
    task_name: impl Into<String>,
    timestamp_ms: f64,
    robot_id: Option<&str>,
    history_interval_ms: f64,
) -> TelemetryStoreResult<()> {
    if !persist_enabled() {
        return Ok(());
    }
    let task_name = task_name.into();
    let mut store = global_store().lock().unwrap();
    store.touch_heartbeat(&task_name, timestamp_ms, history_interval_ms, robot_id)
}

/// Query filters for listing stored events.
#[derive(Debug, Clone, Default)]
pub struct TelemetryQuery {
    pub device_id: Option<String>,
    pub sensor_id: Option<String>,
    pub task_name: Option<String>,
    pub kind: Option<String>,
    pub since_ms: Option<f64>,
    pub limit: Option<usize>,
}

/// Aggregate counts by event kind.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct TelemetryStats {
    pub total_events: usize,
    pub device_events: usize,
    pub sensor_events: usize,
    pub heartbeat_events: usize,
    pub health_events: usize,
    pub tracked_tasks: usize,
}

/// Append-only JSONL store with heartbeat index sidecar.
#[derive(Debug)]
pub struct PersistentTelemetryStore {
    store_path: PathBuf,
    heartbeat_path: PathBuf,
    heartbeat_index: HeartbeatIndex,
    last_heartbeat_history: HashMap<String, f64>,
}

impl PersistentTelemetryStore {
    pub fn open(store_path: PathBuf, heartbeat_path: PathBuf) -> Self {
        let heartbeat_index = read_heartbeat_index(&heartbeat_path).unwrap_or_default();
        Self {
            store_path,
            heartbeat_path,
            heartbeat_index,
            last_heartbeat_history: HashMap::new(),
        }
    }

    pub fn store_path(&self) -> &Path {
        &self.store_path
    }

    pub fn heartbeat_path(&self) -> &Path {
        &self.heartbeat_path
    }

    pub fn append(&mut self, event: TelemetryEvent) -> TelemetryStoreResult<()> {
        if let Some(parent) = self.store_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let line = serde_json::to_string(&event)
            .map_err(|error| TelemetryStoreError::Serialization(error.to_string()))?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.store_path)?;
        writeln!(file, "{line}")?;
        Ok(())
    }

    pub fn touch_heartbeat(
        &mut self,
        task_name: &str,
        timestamp_ms: f64,
        history_interval_ms: f64,
        robot_id: Option<&str>,
    ) -> TelemetryStoreResult<()> {
        self.heartbeat_index
            .tasks
            .insert(task_name.to_string(), timestamp_ms);
        write_heartbeat_index(&self.heartbeat_path, &self.heartbeat_index)?;

        let last = self
            .last_heartbeat_history
            .get(task_name)
            .copied()
            .unwrap_or(f64::MIN);
        if timestamp_ms - last >= history_interval_ms {
            self.last_heartbeat_history
                .insert(task_name.to_string(), timestamp_ms);
            self.append(TelemetryEvent::Heartbeat {
                task_name: task_name.to_string(),
                timestamp_ms,
                robot_id: robot_id.map(str::to_string),
            })?;
        }
        Ok(())
    }

    pub fn read_all(&self) -> TelemetryStoreResult<Vec<TelemetryEvent>> {
        if !self.store_path.exists() {
            return Ok(Vec::new());
        }
        let file = fs::File::open(&self.store_path)?;
        let reader = BufReader::new(file);
        let mut events = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let event: TelemetryEvent = serde_json::from_str(trimmed)
                .map_err(|error| TelemetryStoreError::Serialization(error.to_string()))?;
            events.push(event);
        }
        Ok(events)
    }

    pub fn query(&self, query: &TelemetryQuery) -> TelemetryStoreResult<Vec<TelemetryEvent>> {
        let mut events: Vec<TelemetryEvent> = self
            .read_all()?
            .into_iter()
            .filter(|event| matches_query(event, query))
            .collect();
        if let Some(limit) = query.limit {
            if events.len() > limit {
                let start = events.len() - limit;
                events = events.split_off(start);
            }
        }
        Ok(events)
    }

    pub fn latest_device(
        &self,
        device_id: &str,
        metric: &str,
    ) -> TelemetryStoreResult<Option<TelemetryEvent>> {
        Ok(self.read_all()?.into_iter().rev().find(|event| {
            matches!(
                event,
                TelemetryEvent::Device {
                    device_id: id,
                    metric: name,
                    ..
                } if id == device_id && name == metric
            )
        }))
    }

    pub fn latest_sensor(&self, sensor_id: &str) -> TelemetryStoreResult<Option<TelemetryEvent>> {
        Ok(self.read_all()?.into_iter().rev().find(|event| {
            matches!(
                event,
                TelemetryEvent::Sensor {
                    sensor_id: id, ..
                } if id == sensor_id
            )
        }))
    }

    pub fn heartbeat_index(&self) -> &HeartbeatIndex {
        &self.heartbeat_index
    }

    pub fn stats(&self) -> TelemetryStoreResult<TelemetryStats> {
        let events = self.read_all()?;
        let mut stats = TelemetryStats {
            total_events: events.len(),
            tracked_tasks: self.heartbeat_index.tasks.len(),
            ..TelemetryStats::default()
        };
        for event in events {
            match event {
                TelemetryEvent::Device { .. } => stats.device_events += 1,
                TelemetryEvent::Sensor { .. } => stats.sensor_events += 1,
                TelemetryEvent::Heartbeat { .. } => stats.heartbeat_events += 1,
                TelemetryEvent::Health { .. } => stats.health_events += 1,
            }
        }
        Ok(stats)
    }
}

fn runtime_value_json(value: &RuntimeValue) -> TelemetryStoreResult<JsonValue> {
    let json = runtime_to_json_string(value)
        .map_err(|error| TelemetryStoreError::Serialization(error.to_string()))?;
    serde_json::from_str(&json)
        .map_err(|error| TelemetryStoreError::Serialization(error.to_string()))
}

fn read_heartbeat_index(path: &Path) -> TelemetryStoreResult<HeartbeatIndex> {
    if !path.exists() {
        return Ok(HeartbeatIndex::default());
    }
    let text = fs::read_to_string(path)?;
    serde_json::from_str(&text).map_err(|error| TelemetryStoreError::Serialization(error.to_string()))
}

fn write_heartbeat_index(path: &Path, index: &HeartbeatIndex) -> TelemetryStoreResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(index)
        .map_err(|error| TelemetryStoreError::Serialization(error.to_string()))?;
    fs::write(path, text)?;
    Ok(())
}

fn matches_query(event: &TelemetryEvent, query: &TelemetryQuery) -> bool {
    if let Some(since_ms) = query.since_ms {
        if event.timestamp_ms() < since_ms {
            return false;
        }
    }
    if let Some(kind) = &query.kind {
        let event_kind = match event {
            TelemetryEvent::Device { .. } => "device",
            TelemetryEvent::Sensor { .. } => "sensor",
            TelemetryEvent::Heartbeat { .. } => "heartbeat",
            TelemetryEvent::Health { .. } => "health",
        };
        if event_kind != kind.as_str() {
            return false;
        }
    }
    match event {
        TelemetryEvent::Device { device_id, .. } => {
            query.sensor_id.is_none()
                && query.task_name.is_none()
                && query
                    .device_id
                    .as_ref()
                    .is_none_or(|expected| expected == device_id)
        }
        TelemetryEvent::Sensor { sensor_id, .. } => {
            query.device_id.is_none()
                && query.task_name.is_none()
                && query
                    .sensor_id
                    .as_ref()
                    .is_none_or(|expected| expected == sensor_id)
        }
        TelemetryEvent::Heartbeat { task_name, .. } => {
            query.device_id.is_none()
                && query.sensor_id.is_none()
                && query
                    .task_name
                    .as_ref()
                    .is_none_or(|expected| expected == task_name)
        }
        TelemetryEvent::Health { .. } => {
            query.device_id.is_none() && query.sensor_id.is_none() && query.task_name.is_none()
        }
    }
}

/// Wall-clock timestamp in milliseconds for store events without sim time.
pub fn wall_timestamp_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as f64)
        .unwrap_or(0.0)
}
