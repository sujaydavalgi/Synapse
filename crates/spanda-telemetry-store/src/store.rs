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
use spanda_runtime::telemetry::RuntimeTelemetry;

static SESSION_ENABLED: AtomicBool = AtomicBool::new(false);
static ACTIVE_SESSION_ID: OnceLock<Mutex<Option<String>>> = OnceLock::new();
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
    if !enabled {
        clear_active_session();
    }
}

fn active_session_slot() -> &'static Mutex<Option<String>> {
    ACTIVE_SESSION_ID.get_or_init(|| Mutex::new(None))
}

/// Begin a telemetry session for correlating events with mission traces.
pub fn begin_run_session(source: Option<&str>) -> TelemetryStoreResult<String> {
    if !persist_enabled() {
        return Ok(String::new());
    }
    let session_id = new_session_id(source);
    *active_session_slot().lock().unwrap() = Some(session_id.clone());
    append_event(TelemetryEvent::Session {
        session_id: session_id.clone(),
        phase: "start".into(),
        source: source.map(str::to_string),
        mission_trace_path: None,
        timestamp_ms: wall_timestamp_ms(),
    })?;
    Ok(session_id)
}

/// End the active telemetry session and optionally link a mission trace path.
pub fn end_run_session(
    mission_trace_path: Option<&str>,
    metrics: Option<&RuntimeTelemetry>,
    timestamp_ms: f64,
) -> TelemetryStoreResult<()> {
    if !persist_enabled() {
        return Ok(());
    }
    let session_id = active_session_slot().lock().unwrap().take();
    let Some(session_id) = session_id else {
        return Ok(());
    };
    append_event(TelemetryEvent::Session {
        session_id: session_id.clone(),
        phase: "end".into(),
        source: None,
        mission_trace_path: mission_trace_path.map(str::to_string),
        timestamp_ms,
    })?;
    if let Some(metrics) = metrics {
        let payload = serde_json::to_value(metrics)
            .map_err(|error| TelemetryStoreError::Serialization(error.to_string()))?;
        append_event(TelemetryEvent::RuntimeMetrics {
            session_id,
            metrics: payload,
            timestamp_ms,
        })?;
    }
    Ok(())
}

fn active_session_id() -> Option<String> {
    active_session_slot().lock().unwrap().clone()
}

fn stamp_active_session_id(event: &mut TelemetryEvent) {
    let Some(session_id) = active_session_id() else {
        return;
    };
    match event {
        TelemetryEvent::Device { session_id: slot, .. }
        | TelemetryEvent::Sensor { session_id: slot, .. }
        | TelemetryEvent::Heartbeat { session_id: slot, .. }
        | TelemetryEvent::DeviceHeartbeat { session_id: slot, .. }
        | TelemetryEvent::Health { session_id: slot, .. } => {
            if slot.is_none() {
                *slot = Some(session_id);
            }
        }
        TelemetryEvent::Session { .. } | TelemetryEvent::RuntimeMetrics { .. } => {}
    }
}

fn clear_active_session() {
    if let Some(slot) = ACTIVE_SESSION_ID.get() {
        *slot.lock().unwrap() = None;
    }
}

fn new_session_id(source: Option<&str>) -> String {
    let stem = source
        .and_then(|path| Path::new(path).file_stem())
        .and_then(|name| name.to_str())
        .unwrap_or("program");
    format!("{stem}-{}", wall_timestamp_ms())
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
pub fn append_event(mut event: TelemetryEvent) -> TelemetryStoreResult<()> {
    if !persist_enabled() {
        return Ok(());
    }
    stamp_active_session_id(&mut event);
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
        session_id: None,
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
        session_id: None,
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

/// Record a device or fleet agent heartbeat when persistence is enabled.
pub fn record_device_heartbeat(
    device_id: impl Into<String>,
    timestamp_ms: f64,
    robot_id: Option<&str>,
    protocol: Option<&str>,
    history_interval_ms: f64,
) -> TelemetryStoreResult<()> {
    if !persist_enabled() {
        return Ok(());
    }
    let device_id = device_id.into();
    let mut store = global_store().lock().unwrap();
    store.touch_device_heartbeat(
        &device_id,
        timestamp_ms,
        history_interval_ms,
        robot_id,
        protocol,
    )
}

/// Return true when a telemetry metric name represents a liveness heartbeat.
pub fn is_heartbeat_metric(metric: &str) -> bool {
    matches!(
        metric.to_ascii_lowercase().as_str(),
        "heartbeat" | "liveness" | "alive" | "ping"
    )
}

/// Record a health status transition when persistence is enabled.
pub fn record_health_event(
    target: impl Into<String>,
    status: impl Into<String>,
    timestamp_ms: f64,
) -> TelemetryStoreResult<()> {
    append_event(TelemetryEvent::Health {
        target: target.into(),
        status: status.into(),
        timestamp_ms,
        session_id: None,
    })
}

/// Record a topic publish as device telemetry keyed by robot and topic path.
pub fn record_topic_publish(
    robot_id: Option<&str>,
    topic_path: &str,
    value: &RuntimeValue,
    timestamp_ms: f64,
) -> TelemetryStoreResult<()> {
    record_device_telemetry(
        robot_id.unwrap_or("robot"),
        topic_path,
        value,
        timestamp_ms,
        robot_id,
    )
}

/// Query filters for listing stored events.
#[derive(Debug, Clone, Default)]
pub struct TelemetryQuery {
    pub device_id: Option<String>,
    pub sensor_id: Option<String>,
    pub task_name: Option<String>,
    pub kind: Option<String>,
    pub session_id: Option<String>,
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
    pub device_heartbeat_events: usize,
    pub health_events: usize,
    pub session_events: usize,
    pub runtime_metrics_events: usize,
    pub tracked_tasks: usize,
    pub tracked_devices: usize,
}

/// Append-only JSONL store with heartbeat index sidecar.
#[derive(Debug)]
pub struct PersistentTelemetryStore {
    store_path: PathBuf,
    heartbeat_path: PathBuf,
    heartbeat_index: HeartbeatIndex,
    last_heartbeat_history: HashMap<String, f64>,
    last_device_heartbeat_history: HashMap<String, f64>,
    max_events: Option<usize>,
}

impl PersistentTelemetryStore {
    pub fn open(store_path: PathBuf, heartbeat_path: PathBuf) -> Self {
        Self::open_with_max_events(store_path, heartbeat_path, resolve_max_events())
    }

    pub fn open_with_max_events(
        store_path: PathBuf,
        heartbeat_path: PathBuf,
        max_events: Option<usize>,
    ) -> Self {
        let heartbeat_index = read_heartbeat_index(&heartbeat_path).unwrap_or_default();
        Self {
            store_path,
            heartbeat_path,
            heartbeat_index,
            last_heartbeat_history: HashMap::new(),
            last_device_heartbeat_history: HashMap::new(),
            max_events,
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
        self.maybe_compact()?;
        Ok(())
    }

    fn maybe_compact(&mut self) -> TelemetryStoreResult<()> {
        let Some(max_events) = self.max_events else {
            return Ok(());
        };
        let events = self.read_all()?;
        if events.len() <= max_events {
            return Ok(());
        }
        let start = events.len() - max_events;
        let trimmed: Vec<TelemetryEvent> = events.into_iter().skip(start).collect();
        self.rewrite_all(&trimmed)
    }

    fn rewrite_all(&self, events: &[TelemetryEvent]) -> TelemetryStoreResult<()> {
        if let Some(parent) = self.store_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut body = String::new();
        for event in events {
            body.push_str(&serde_json::to_string(event)
                .map_err(|error| TelemetryStoreError::Serialization(error.to_string()))?);
            body.push('\n');
        }
        fs::write(&self.store_path, body)?;
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
            let mut event = TelemetryEvent::Heartbeat {
                task_name: task_name.to_string(),
                timestamp_ms,
                robot_id: robot_id.map(str::to_string),
                session_id: None,
            };
            stamp_active_session_id(&mut event);
            self.append(event)?;
        }
        Ok(())
    }

    pub fn touch_device_heartbeat(
        &mut self,
        device_id: &str,
        timestamp_ms: f64,
        history_interval_ms: f64,
        robot_id: Option<&str>,
        protocol: Option<&str>,
    ) -> TelemetryStoreResult<()> {
        self.heartbeat_index
            .devices
            .insert(device_id.to_string(), timestamp_ms);
        write_heartbeat_index(&self.heartbeat_path, &self.heartbeat_index)?;

        let last = self
            .last_device_heartbeat_history
            .get(device_id)
            .copied()
            .unwrap_or(f64::MIN);
        if timestamp_ms - last >= history_interval_ms {
            self.last_device_heartbeat_history
                .insert(device_id.to_string(), timestamp_ms);
            let mut event = TelemetryEvent::DeviceHeartbeat {
                device_id: device_id.to_string(),
                timestamp_ms,
                robot_id: robot_id.map(str::to_string),
                protocol: protocol.map(str::to_string),
                session_id: None,
            };
            stamp_active_session_id(&mut event);
            self.append(event)?;
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
        let all = self.read_all()?;
        let session_window = query
            .session_id
            .as_ref()
            .and_then(|session_id| session_time_window(&all, session_id));
        let mut events: Vec<TelemetryEvent> = all
            .into_iter()
            .filter(|event| {
                if let Some(expected) = &query.session_id {
                    match event.session_id() {
                        Some(actual) if actual == expected => {}
                        Some(_) => return false,
                        None => {
                            let Some((start_ms, end_ms)) = session_window else {
                                return false;
                            };
                            let ts = event.timestamp_ms();
                            if ts < start_ms || ts > end_ms {
                                return false;
                            }
                        }
                    }
                }
                matches_query(event, query)
            })
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
            tracked_devices: self.heartbeat_index.devices.len(),
            ..TelemetryStats::default()
        };
        for event in events {
            match event {
                TelemetryEvent::Device { .. } => stats.device_events += 1,
                TelemetryEvent::Sensor { .. } => stats.sensor_events += 1,
                TelemetryEvent::Heartbeat { .. } => stats.heartbeat_events += 1,
                TelemetryEvent::DeviceHeartbeat { .. } => stats.device_heartbeat_events += 1,
                TelemetryEvent::Health { .. } => stats.health_events += 1,
                TelemetryEvent::Session { .. } => stats.session_events += 1,
                TelemetryEvent::RuntimeMetrics { .. } => stats.runtime_metrics_events += 1,
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

fn resolve_max_events() -> Option<usize> {
    std::env::var("SPANDA_TELEMETRY_MAX_EVENTS")
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|max| *max > 0)
}

fn session_time_window(events: &[TelemetryEvent], session_id: &str) -> Option<(f64, f64)> {
    let mut start_ms = None;
    let mut end_ms = None;
    for event in events {
        if let TelemetryEvent::Session {
            session_id: id,
            phase,
            timestamp_ms,
            ..
        } = event
        {
            if id != session_id {
                continue;
            }
            if phase == "start" {
                start_ms = Some(*timestamp_ms);
            } else if phase == "end" {
                end_ms = Some(*timestamp_ms);
            }
        }
    }
    Some((start_ms?, end_ms?))
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
            TelemetryEvent::DeviceHeartbeat { .. } => "device_heartbeat",
            TelemetryEvent::Health { .. } => "health",
            TelemetryEvent::Session { .. } => "session",
            TelemetryEvent::RuntimeMetrics { .. } => "runtime_metrics",
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
        TelemetryEvent::DeviceHeartbeat { device_id, .. } => {
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
        TelemetryEvent::Session { .. } | TelemetryEvent::RuntimeMetrics { .. } => {
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
