//! Persistent append-only telemetry storage for devices, sensors, and heartbeats.
//!
//! Events are written to `.spanda/telemetry-store.jsonl` with a heartbeat index
//! sidecar at `.spanda/telemetry-heartbeats.json`. Enable with `--persist-telemetry`
//! or `SPANDA_TELEMETRY_STORE=1`.

pub mod error;
pub mod record;
pub mod store;

pub use error::{TelemetryStoreError, TelemetryStoreResult};
pub use record::{HeartbeatIndex, TelemetryEvent};
pub use store::{
    append_event, configure_session_persist, default_heartbeat_index_path, default_store_path,
    env_persist_enabled, global_store, persist_enabled, record_device_telemetry,
    record_health_event, record_sensor_reading, record_task_heartbeat, resolve_heartbeat_index_path,
    resolve_store_path, wall_timestamp_ms, PersistentTelemetryStore, TelemetryQuery, TelemetryStats,
};
