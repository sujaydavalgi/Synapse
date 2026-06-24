use spanda_telemetry_store::{
    PersistentTelemetryStore, TelemetryEvent, TelemetryQuery,
};
use tempfile::tempdir;

#[test]
fn persistent_store_sqlite_round_trip() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("telemetry.db");
    let heartbeat_path = dir.path().join("telemetry-heartbeats.json");
    std::env::set_var("SPANDA_TELEMETRY_BACKEND", "sqlite");
    let mut store = PersistentTelemetryStore::open(store_path, heartbeat_path);
    store
        .append(TelemetryEvent::Device {
            device_id: "rover".into(),
            metric: "/telemetry".into(),
            value: serde_json::json!(1.0),
            timestamp_ms: 100.0,
            robot_id: Some("Rover".into()),
            session_id: Some("run-1".into()),
        })
        .unwrap();
    let events = store
        .query(&TelemetryQuery {
            kind: Some("device".into()),
            session_id: Some("run-1".into()),
            ..TelemetryQuery::default()
        })
        .unwrap();
    assert_eq!(events.len(), 1);
    assert!(store.sqlite_backend());
    std::env::remove_var("SPANDA_TELEMETRY_BACKEND");
}
