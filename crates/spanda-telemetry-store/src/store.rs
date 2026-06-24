//! Append-only telemetry store with JSONL or SQLite backends.

use crate::error::{TelemetryStoreError, TelemetryStoreResult};
use crate::record::{HeartbeatIndex, TelemetryEvent};
#[cfg(feature = "sqlite")]
use crate::sqlite;
#[cfg(feature = "sqlite")]
use rusqlite::Connection;
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
        let (store_path, heartbeat_path) = resolve_store_paths();
        Mutex::new(PersistentTelemetryStore::open(store_path, heartbeat_path))
    })
}

fn resolve_store_paths() -> (PathBuf, PathBuf) {
    #[cfg(feature = "sqlite")]
    if sqlite::env_backend_sqlite() {
        let path = sqlite::resolve_sqlite_path();
        return (path.clone(), resolve_heartbeat_index_path(&path));
    }
    let path = resolve_store_path();
    (path.clone(), resolve_heartbeat_index_path(&path))
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

/// Compute aggregate counts for an event slice and heartbeat index.
pub fn stats_from_events(events: &[TelemetryEvent], index: &HeartbeatIndex) -> TelemetryStats {
    let mut stats = TelemetryStats {
        total_events: events.len(),
        tracked_tasks: index.tasks.len(),
        tracked_devices: index.devices.len(),
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
    stats
}

/// Summary of a persisted run session linked to telemetry events.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TelemetrySessionSummary {
    pub session_id: String,
    pub source: Option<String>,
    pub start_ms: f64,
    pub end_ms: Option<f64>,
    pub mission_trace_path: Option<String>,
    pub event_count: usize,
}

/// Resolved store layout for operator diagnostics.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TelemetryStoreInfo {
    pub backend: String,
    pub store_path: String,
    pub heartbeat_path: Option<String>,
    pub event_count: usize,
    pub max_events: Option<usize>,
    pub migrated_jsonl_backup: Option<String>,
}

/// Append-only store with JSONL or SQLite backend and heartbeat index.
pub struct PersistentTelemetryStore {
    store_path: PathBuf,
    heartbeat_path: PathBuf,
    heartbeat_index: HeartbeatIndex,
    last_heartbeat_history: HashMap<String, f64>,
    last_device_heartbeat_history: HashMap<String, f64>,
    max_events: Option<usize>,
    sqlite_backend: bool,
    #[cfg(feature = "sqlite")]
    sqlite: Option<Connection>,
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
        #[cfg(feature = "sqlite")]
        let sqlite_backend = sqlite::env_backend_sqlite();
        #[cfg(not(feature = "sqlite"))]
        let sqlite_backend = false;
        #[cfg(feature = "sqlite")]
        let sqlite = if sqlite_backend {
            Some(
                sqlite::open_connection(&store_path)
                    .expect("failed to open sqlite telemetry store"),
            )
        } else {
            None
        };
        #[cfg(feature = "sqlite")]
        let heartbeat_index = if let Some(conn) = &sqlite {
            sqlite::read_heartbeat_index(conn).unwrap_or_default()
        } else {
            read_heartbeat_index(&heartbeat_path).unwrap_or_default()
        };
        #[cfg(not(feature = "sqlite"))]
        let heartbeat_index = read_heartbeat_index(&heartbeat_path).unwrap_or_default();
        Self {
            store_path,
            heartbeat_path,
            heartbeat_index,
            last_heartbeat_history: HashMap::new(),
            last_device_heartbeat_history: HashMap::new(),
            max_events,
            sqlite_backend,
            #[cfg(feature = "sqlite")]
            sqlite,
        }
    }

    /// Return true when this store uses the SQLite backend.
    pub fn sqlite_backend(&self) -> bool {
        self.sqlite_backend
    }

    pub fn store_path(&self) -> &Path {
        &self.store_path
    }

    pub fn heartbeat_path(&self) -> &Path {
        &self.heartbeat_path
    }

    pub fn append(&mut self, event: TelemetryEvent) -> TelemetryStoreResult<()> {
        #[cfg(feature = "sqlite")]
        if self.append_sqlite(&event)? {
            return Ok(());
        }
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
        #[cfg(feature = "sqlite")]
        if self.maybe_compact_sqlite(max_events)? {
            return Ok(());
        }
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
        self.persist_heartbeat_index(task_name, timestamp_ms, "task")?;

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
        self.persist_heartbeat_index(device_id, timestamp_ms, "device")?;

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
        #[cfg(feature = "sqlite")]
        if let Some(events) = self.read_all_sqlite()? {
            return Ok(events);
        }
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
        #[cfg(feature = "sqlite")]
        if let Some(events) = self.query_sqlite(query)? {
            return Ok(events);
        }
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
        #[cfg(feature = "sqlite")]
        if let Some(event) = self.latest_device_sqlite(device_id, metric)? {
            return Ok(event);
        }
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
        #[cfg(feature = "sqlite")]
        if let Some(event) = self.latest_sensor_sqlite(sensor_id)? {
            return Ok(event);
        }
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

    pub fn list_sessions(&self) -> TelemetryStoreResult<Vec<TelemetrySessionSummary>> {
        let events = self.read_all()?;
        let mut summaries: HashMap<String, TelemetrySessionSummary> = HashMap::new();
        for event in &events {
            if let TelemetryEvent::Session {
                session_id,
                phase,
                source,
                mission_trace_path,
                timestamp_ms,
            } = event
            {
                let entry = summaries
                    .entry(session_id.clone())
                    .or_insert_with(|| TelemetrySessionSummary {
                        session_id: session_id.clone(),
                        source: None,
                        start_ms: *timestamp_ms,
                        end_ms: None,
                        mission_trace_path: None,
                        event_count: 0,
                    });
                if phase == "start" {
                    entry.start_ms = *timestamp_ms;
                    entry.source = source.clone();
                } else if phase == "end" {
                    entry.end_ms = Some(*timestamp_ms);
                    if mission_trace_path.is_some() {
                        entry.mission_trace_path = mission_trace_path.clone();
                    }
                }
            }
        }

        for summary in summaries.values_mut() {
            let query = TelemetryQuery {
                session_id: Some(summary.session_id.clone()),
                ..TelemetryQuery::default()
            };
            summary.event_count = self.query(&query)?.len();
        }

        let mut sessions: Vec<TelemetrySessionSummary> = summaries.into_values().collect();
        sessions.sort_by(|left, right| {
            right
                .start_ms
                .partial_cmp(&left.start_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(sessions)
    }

    pub fn mission_trace_for_session(
        &self,
        session_id: &str,
    ) -> TelemetryStoreResult<Option<String>> {
        for event in self.read_all()? {
            if let TelemetryEvent::Session {
                session_id: id,
                phase,
                mission_trace_path,
                ..
            } = event
            {
                if id == session_id && phase == "end" {
                    return Ok(mission_trace_path.clone());
                }
            }
        }
        Ok(None)
    }

    pub fn info(&self) -> TelemetryStoreResult<TelemetryStoreInfo> {
        let migrated_jsonl_backup = if self.sqlite_backend {
            let backup = self
                .store_path
                .parent()
                .map(|dir| dir.join("telemetry-store.jsonl.bak"))
                .filter(|path| path.is_file())
                .map(|path| path.display().to_string());
            backup
        } else {
            None
        };
        Ok(TelemetryStoreInfo {
            backend: if self.sqlite_backend {
                "sqlite".into()
            } else {
                "jsonl".into()
            },
            store_path: self.store_path.display().to_string(),
            heartbeat_path: if self.sqlite_backend {
                None
            } else {
                Some(self.heartbeat_path.display().to_string())
            },
            event_count: self.read_all()?.len(),
            max_events: self.max_events,
            migrated_jsonl_backup,
        })
    }

    pub fn stats(&self) -> TelemetryStoreResult<TelemetryStats> {
        #[cfg(feature = "sqlite")]
        if let Some(stats) = self.stats_sqlite()? {
            return Ok(stats);
        }
        let events = self.read_all()?;
        Ok(stats_from_events(&events, &self.heartbeat_index))
    }

    fn persist_heartbeat_index(
        &self,
        target_id: &str,
        timestamp_ms: f64,
        target_kind: &str,
    ) -> TelemetryStoreResult<()> {
        #[cfg(feature = "sqlite")]
        if self.persist_heartbeat_index_sqlite(target_id, timestamp_ms, target_kind)? {
            return Ok(());
        }
        write_heartbeat_index(&self.heartbeat_path, &self.heartbeat_index)
    }
}

#[cfg(feature = "sqlite")]
impl PersistentTelemetryStore {
    fn append_sqlite(&mut self, event: &TelemetryEvent) -> TelemetryStoreResult<bool> {
        if !self.sqlite_backend {
            return Ok(false);
        }
        if let Some(conn) = &self.sqlite {
            sqlite::append_event(conn, event)?;
        }
        self.maybe_compact()?;
        Ok(true)
    }

    fn maybe_compact_sqlite(&mut self, max_events: usize) -> TelemetryStoreResult<bool> {
        if !self.sqlite_backend {
            return Ok(false);
        }
        if let Some(conn) = &self.sqlite {
            let count = sqlite::read_all(conn)?.len();
            if count > max_events {
                sqlite::compact(conn, max_events)?;
            }
        }
        Ok(true)
    }

    fn read_all_sqlite(&self) -> TelemetryStoreResult<Option<Vec<TelemetryEvent>>> {
        if !self.sqlite_backend {
            return Ok(None);
        }
        if let Some(conn) = &self.sqlite {
            return sqlite::read_all(conn).map(Some);
        }
        Ok(Some(Vec::new()))
    }

    fn query_sqlite(
        &self,
        query: &TelemetryQuery,
    ) -> TelemetryStoreResult<Option<Vec<TelemetryEvent>>> {
        if !self.sqlite_backend {
            return Ok(None);
        }
        if let Some(conn) = &self.sqlite {
            return sqlite::query(conn, query).map(Some);
        }
        Ok(Some(Vec::new()))
    }

    fn latest_device_sqlite(
        &self,
        device_id: &str,
        metric: &str,
    ) -> TelemetryStoreResult<Option<Option<TelemetryEvent>>> {
        if !self.sqlite_backend {
            return Ok(None);
        }
        if let Some(conn) = &self.sqlite {
            return sqlite::latest_device(conn, device_id, metric).map(Some);
        }
        Ok(Some(None))
    }

    fn latest_sensor_sqlite(
        &self,
        sensor_id: &str,
    ) -> TelemetryStoreResult<Option<Option<TelemetryEvent>>> {
        if !self.sqlite_backend {
            return Ok(None);
        }
        if let Some(conn) = &self.sqlite {
            return sqlite::latest_sensor(conn, sensor_id).map(Some);
        }
        Ok(Some(None))
    }

    fn stats_sqlite(&self) -> TelemetryStoreResult<Option<TelemetryStats>> {
        if !self.sqlite_backend {
            return Ok(None);
        }
        if let Some(conn) = &self.sqlite {
            return sqlite::stats(conn, &self.heartbeat_index).map(Some);
        }
        Ok(Some(TelemetryStats::default()))
    }

    fn persist_heartbeat_index_sqlite(
        &self,
        target_id: &str,
        timestamp_ms: f64,
        target_kind: &str,
    ) -> TelemetryStoreResult<bool> {
        if !self.sqlite_backend {
            return Ok(false);
        }
        if let Some(conn) = &self.sqlite {
            sqlite::upsert_heartbeat(conn, target_kind, target_id, timestamp_ms)?;
        }
        Ok(true)
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

pub(crate) fn session_time_window(events: &[TelemetryEvent], session_id: &str) -> Option<(f64, f64)> {
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

pub(crate) fn matches_query(event: &TelemetryEvent, query: &TelemetryQuery) -> bool {
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
