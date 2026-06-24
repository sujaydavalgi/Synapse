//! Persistent append-only telemetry storage for devices, sensors, and heartbeats.
//!
//! Events are written to `.spanda/telemetry-store.jsonl` (default) or
//! `.spanda/telemetry-store.db` when `SPANDA_TELEMETRY_BACKEND=sqlite`, with a
//! heartbeat index sidecar or SQLite table. Enable with `--persist-telemetry` or
//! `SPANDA_TELEMETRY_STORE=1`.

pub mod error;
pub mod memory;
pub mod otlp;
pub mod prometheus;
pub mod record;
pub mod serve;
#[cfg(feature = "sqlite")]
pub mod sqlite;
pub mod store;

pub use error::{TelemetryStoreError, TelemetryStoreResult};
pub use memory::{
    memory_append_json_line, memory_clear, memory_render_otlp_json, memory_render_prometheus,
    memory_stats, MemoryTelemetryStore,
};
pub use record::{HeartbeatIndex, TelemetryEvent};
pub use otlp::{render_otlp_from_events, render_otlp_json};
pub use prometheus::{render_prometheus, render_prometheus_from_events};
pub use serve::{run_telemetry_server, TelemetryServeOptions};
pub use store::{
    append_event, begin_run_session, configure_session_persist, default_heartbeat_index_path,
    default_store_path, end_run_session, env_persist_enabled, global_store, is_heartbeat_metric,
    persist_enabled, record_device_heartbeat, record_device_telemetry, record_health_event,
    record_sensor_reading, record_task_heartbeat, record_topic_publish, resolve_heartbeat_index_path,
    resolve_store_path, stats_from_events, wall_timestamp_ms, PersistentTelemetryStore,
    TelemetryQuery, TelemetrySessionSummary, TelemetryStats, TelemetryStoreInfo,
};
#[cfg(feature = "sqlite")]
pub use sqlite::{default_sqlite_store_path, env_backend_sqlite, resolve_sqlite_path};
