//! SQLite-backed telemetry storage with indexed queries.

use crate::error::{TelemetryStoreError, TelemetryStoreResult};
use crate::record::{HeartbeatIndex, TelemetryEvent};
use crate::store::{matches_query, session_time_window, TelemetryQuery, TelemetryStats};
use rusqlite::{params, Connection};
use std::fs;
use std::path::{Path, PathBuf};

/// Default SQLite database path under `.spanda/`.
pub fn default_sqlite_store_path() -> PathBuf {
    PathBuf::from(".spanda/telemetry-store.db")
}

/// Resolve SQLite path from `SPANDA_TELEMETRY_STORE_PATH` or the default `.db` file.
pub fn resolve_sqlite_path() -> PathBuf {
    std::env::var("SPANDA_TELEMETRY_STORE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_sqlite_store_path())
}

/// Return true when `SPANDA_TELEMETRY_BACKEND=sqlite`.
pub fn env_backend_sqlite() -> bool {
    std::env::var("SPANDA_TELEMETRY_BACKEND")
        .map(|value| value.eq_ignore_ascii_case("sqlite"))
        .unwrap_or(false)
}

/// Open a SQLite telemetry database and ensure schema indexes exist.
pub fn open_connection(path: &Path) -> TelemetryStoreResult<Connection> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path).map_err(|error| TelemetryStoreError::Database(error.to_string()))?;
    init_schema(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> TelemetryStoreResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS telemetry_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            kind TEXT NOT NULL,
            timestamp_ms REAL NOT NULL,
            session_id TEXT,
            device_id TEXT,
            sensor_id TEXT,
            task_name TEXT,
            metric TEXT,
            robot_id TEXT,
            payload TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_events_kind ON telemetry_events(kind);
        CREATE INDEX IF NOT EXISTS idx_events_session ON telemetry_events(session_id);
        CREATE INDEX IF NOT EXISTS idx_events_timestamp ON telemetry_events(timestamp_ms);
        CREATE INDEX IF NOT EXISTS idx_events_device ON telemetry_events(device_id);
        CREATE INDEX IF NOT EXISTS idx_events_sensor ON telemetry_events(sensor_id);
        CREATE INDEX IF NOT EXISTS idx_events_task ON telemetry_events(task_name);
        CREATE TABLE IF NOT EXISTS heartbeat_liveness (
            target_kind TEXT NOT NULL,
            target_id TEXT NOT NULL,
            timestamp_ms REAL NOT NULL,
            PRIMARY KEY (target_kind, target_id)
        );",
    )
    .map_err(|error| TelemetryStoreError::Database(error.to_string()))?;
    Ok(())
}

pub fn append_event(conn: &Connection, event: &TelemetryEvent) -> TelemetryStoreResult<()> {
    let kind = event_kind(event);
    let (device_id, sensor_id, task_name, metric, session_id, robot_id) = index_fields(event);
    let payload = serde_json::to_string(event)
        .map_err(|error| TelemetryStoreError::Serialization(error.to_string()))?;
    conn.execute(
        "INSERT INTO telemetry_events
         (kind, timestamp_ms, session_id, device_id, sensor_id, task_name, metric, robot_id, payload)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            kind,
            event.timestamp_ms(),
            session_id,
            device_id,
            sensor_id,
            task_name,
            metric,
            robot_id,
            payload,
        ],
    )
    .map_err(|error| TelemetryStoreError::Database(error.to_string()))?;
    Ok(())
}

pub fn read_all(conn: &Connection) -> TelemetryStoreResult<Vec<TelemetryEvent>> {
    let mut stmt = conn
        .prepare("SELECT payload FROM telemetry_events ORDER BY id ASC")
        .map_err(|error| TelemetryStoreError::Database(error.to_string()))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| TelemetryStoreError::Database(error.to_string()))?;
    let mut events = Vec::new();
    for row in rows {
        let payload = row.map_err(|error| TelemetryStoreError::Database(error.to_string()))?;
        let event: TelemetryEvent = serde_json::from_str(&payload)
            .map_err(|error| TelemetryStoreError::Serialization(error.to_string()))?;
        events.push(event);
    }
    Ok(events)
}

pub fn query(conn: &Connection, query: &TelemetryQuery) -> TelemetryStoreResult<Vec<TelemetryEvent>> {
    let all = read_all(conn)?;
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
    conn: &Connection,
    device_id: &str,
    metric: &str,
) -> TelemetryStoreResult<Option<TelemetryEvent>> {
    let mut stmt = conn
        .prepare(
            "SELECT payload FROM telemetry_events
             WHERE kind = 'device' AND device_id = ?1 AND metric = ?2
             ORDER BY id DESC LIMIT 1",
        )
        .map_err(|error| TelemetryStoreError::Database(error.to_string()))?;
    let mut rows = stmt
        .query_map(params![device_id, metric], |row| row.get::<_, String>(0))
        .map_err(|error| TelemetryStoreError::Database(error.to_string()))?;
    if let Some(row) = rows.next() {
        let payload = row.map_err(|error| TelemetryStoreError::Database(error.to_string()))?;
        let event: TelemetryEvent = serde_json::from_str(&payload)
            .map_err(|error| TelemetryStoreError::Serialization(error.to_string()))?;
        return Ok(Some(event));
    }
    Ok(None)
}

pub fn latest_sensor(conn: &Connection, sensor_id: &str) -> TelemetryStoreResult<Option<TelemetryEvent>> {
    let mut stmt = conn
        .prepare(
            "SELECT payload FROM telemetry_events
             WHERE kind = 'sensor' AND sensor_id = ?1
             ORDER BY id DESC LIMIT 1",
        )
        .map_err(|error| TelemetryStoreError::Database(error.to_string()))?;
    let mut rows = stmt
        .query_map(params![sensor_id], |row| row.get::<_, String>(0))
        .map_err(|error| TelemetryStoreError::Database(error.to_string()))?;
    if let Some(row) = rows.next() {
        let payload = row.map_err(|error| TelemetryStoreError::Database(error.to_string()))?;
        let event: TelemetryEvent = serde_json::from_str(&payload)
            .map_err(|error| TelemetryStoreError::Serialization(error.to_string()))?;
        return Ok(Some(event));
    }
    Ok(None)
}

pub fn stats(conn: &Connection, heartbeat_index: &HeartbeatIndex) -> TelemetryStoreResult<TelemetryStats> {
    let mut stats = TelemetryStats {
        tracked_tasks: heartbeat_index.tasks.len(),
        tracked_devices: heartbeat_index.devices.len(),
        ..TelemetryStats::default()
    };
    let mut stmt = conn
        .prepare("SELECT kind, COUNT(*) FROM telemetry_events GROUP BY kind")
        .map_err(|error| TelemetryStoreError::Database(error.to_string()))?;
    let rows = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
        .map_err(|error| TelemetryStoreError::Database(error.to_string()))?;
    for row in rows {
        let (kind, count) = row.map_err(|error| TelemetryStoreError::Database(error.to_string()))?;
        let count = count as usize;
        stats.total_events += count;
        match kind.as_str() {
            "device" => stats.device_events = count,
            "sensor" => stats.sensor_events = count,
            "heartbeat" => stats.heartbeat_events = count,
            "device_heartbeat" => stats.device_heartbeat_events = count,
            "health" => stats.health_events = count,
            "session" => stats.session_events = count,
            "runtime_metrics" => stats.runtime_metrics_events = count,
            _ => {}
        }
    }
    Ok(stats)
}

pub fn compact(conn: &Connection, max_events: usize) -> TelemetryStoreResult<()> {
    conn.execute(
        "DELETE FROM telemetry_events WHERE id NOT IN (
            SELECT id FROM telemetry_events ORDER BY id DESC LIMIT ?1
        )",
        params![max_events],
    )
    .map_err(|error| TelemetryStoreError::Database(error.to_string()))?;
    Ok(())
}

pub fn read_heartbeat_index(conn: &Connection) -> TelemetryStoreResult<HeartbeatIndex> {
    let mut index = HeartbeatIndex::default();
    let mut stmt = conn
        .prepare("SELECT target_kind, target_id, timestamp_ms FROM heartbeat_liveness")
        .map_err(|error| TelemetryStoreError::Database(error.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
            ))
        })
        .map_err(|error| TelemetryStoreError::Database(error.to_string()))?;
    for row in rows {
        let (kind, id, timestamp_ms) = row.map_err(|error| TelemetryStoreError::Database(error.to_string()))?;
        match kind.as_str() {
            "task" => index.tasks.insert(id, timestamp_ms),
            "device" => index.devices.insert(id, timestamp_ms),
            _ => None,
        };
    }
    Ok(index)
}

pub fn upsert_heartbeat(
    conn: &Connection,
    target_kind: &str,
    target_id: &str,
    timestamp_ms: f64,
) -> TelemetryStoreResult<()> {
    conn.execute(
        "INSERT INTO heartbeat_liveness (target_kind, target_id, timestamp_ms)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(target_kind, target_id) DO UPDATE SET timestamp_ms = excluded.timestamp_ms",
        params![target_kind, target_id, timestamp_ms],
    )
    .map_err(|error| TelemetryStoreError::Database(error.to_string()))?;
    Ok(())
}

fn event_kind(event: &TelemetryEvent) -> &'static str {
    match event {
        TelemetryEvent::Device { .. } => "device",
        TelemetryEvent::Sensor { .. } => "sensor",
        TelemetryEvent::Heartbeat { .. } => "heartbeat",
        TelemetryEvent::DeviceHeartbeat { .. } => "device_heartbeat",
        TelemetryEvent::Health { .. } => "health",
        TelemetryEvent::Session { .. } => "session",
        TelemetryEvent::RuntimeMetrics { .. } => "runtime_metrics",
    }
}

fn index_fields(event: &TelemetryEvent) -> (Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>) {
    match event {
        TelemetryEvent::Device {
            device_id,
            metric,
            robot_id,
            session_id,
            ..
        } => (
            Some(device_id.clone()),
            None,
            None,
            Some(metric.clone()),
            session_id.clone(),
            robot_id.clone(),
        ),
        TelemetryEvent::Sensor {
            sensor_id,
            robot_id,
            session_id,
            ..
        } => (
            None,
            Some(sensor_id.clone()),
            None,
            None,
            session_id.clone(),
            robot_id.clone(),
        ),
        TelemetryEvent::Heartbeat {
            task_name,
            robot_id,
            session_id,
            ..
        } => (
            None,
            None,
            Some(task_name.clone()),
            None,
            session_id.clone(),
            robot_id.clone(),
        ),
        TelemetryEvent::DeviceHeartbeat {
            device_id,
            robot_id,
            session_id,
            ..
        } => (
            Some(device_id.clone()),
            None,
            None,
            None,
            session_id.clone(),
            robot_id.clone(),
        ),
        TelemetryEvent::Health {
            session_id, ..
        } => (None, None, None, None, session_id.clone(), None),
        TelemetryEvent::Session { session_id, .. } | TelemetryEvent::RuntimeMetrics { session_id, .. } => {
            (None, None, None, None, Some(session_id.clone()), None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn sqlite_append_query_and_stats() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("telemetry.db");
        let conn = open_connection(&path).unwrap();
        append_event(
            &conn,
            &TelemetryEvent::Sensor {
                sensor_id: "lidar".into(),
                sensor_type: "Lidar".into(),
                value: serde_json::json!({}),
                timestamp_ms: 10.0,
                robot_id: Some("Rover".into()),
                session_id: Some("run-1".into()),
            },
        )
        .unwrap();
        upsert_heartbeat(&conn, "task", "control", 42.0).unwrap();
        let index = read_heartbeat_index(&conn).unwrap();
        let stats = stats(&conn, &index).unwrap();
        assert_eq!(stats.sensor_events, 1);
        let events = query(
            &conn,
            &TelemetryQuery {
                session_id: Some("run-1".into()),
                kind: Some("sensor".into()),
                ..TelemetryQuery::default()
            },
        )
        .unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn sqlite_compacts_old_rows() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("telemetry.db");
        let conn = open_connection(&path).unwrap();
        for index in 0..3 {
            append_event(
                &conn,
                &TelemetryEvent::Health {
                    target: format!("c-{index}"),
                    status: "Ok".into(),
                    timestamp_ms: index as f64,
                    session_id: None,
                },
            )
            .unwrap();
        }
        compact(&conn, 2).unwrap();
        let events = read_all(&conn).unwrap();
        assert_eq!(events.len(), 2);
    }
}
