use spanda_telemetry_store::{
    begin_run_session, configure_session_persist, end_run_session, env_persist_enabled,
    persist_enabled, record_sensor_reading, resolve_store_path, PersistentTelemetryStore,
    TelemetryEvent, TelemetryQuery,
};
use tempfile::tempdir;

#[test]
fn append_and_query_device_and_sensor_events() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("telemetry.jsonl");
    let heartbeat_path = dir.path().join("heartbeats.json");
    let mut store = PersistentTelemetryStore::open(store_path.clone(), heartbeat_path);

    store
        .append(TelemetryEvent::Device {
            device_id: "robot-001".into(),
            metric: "battery".into(),
            value: serde_json::json!({"kind":"number","value":82.0}),
            timestamp_ms: 1000.0,
            robot_id: Some("Rover".into()),
            session_id: None,
        })
        .unwrap();
    store
        .append(TelemetryEvent::Sensor {
            sensor_id: "lidar".into(),
            sensor_type: "Lidar".into(),
            value: serde_json::json!({"kind":"scan","nearest_distance":1.2}),
            timestamp_ms: 1100.0,
            robot_id: Some("Rover".into()),
            session_id: None,
        })
        .unwrap();

    let events = store
        .query(&TelemetryQuery {
            device_id: Some("robot-001".into()),
            ..TelemetryQuery::default()
        })
        .unwrap();
    assert_eq!(events.len(), 1);

    let latest = store.latest_sensor("lidar").unwrap().unwrap();
    assert!(matches!(latest, TelemetryEvent::Sensor { .. }));
    assert!(store_path.exists());
}

#[test]
fn heartbeat_index_updates_and_history_is_throttled() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("telemetry.jsonl");
    let heartbeat_path = dir.path().join("heartbeats.json");
    let mut store = PersistentTelemetryStore::open(store_path, heartbeat_path.clone());

    store
        .touch_heartbeat("control", 1000.0, 5000.0, Some("Rover"))
        .unwrap();
    store
        .touch_heartbeat("control", 2000.0, 5000.0, Some("Rover"))
        .unwrap();
    store
        .touch_heartbeat("control", 7000.0, 5000.0, Some("Rover"))
        .unwrap();

    let heartbeats: Vec<_> = store
        .read_all()
        .unwrap()
        .into_iter()
        .filter(|event| matches!(event, TelemetryEvent::Heartbeat { .. }))
        .collect();
    assert_eq!(heartbeats.len(), 2);
    assert_eq!(store.heartbeat_index().tasks.get("control"), Some(&7000.0));
    assert!(heartbeat_path.exists());
}

#[test]
fn persist_enabled_respects_session_and_env() {
    configure_session_persist(false);
    std::env::remove_var("SPANDA_TELEMETRY_STORE");
    assert!(!persist_enabled());

    configure_session_persist(true);
    assert!(persist_enabled());

    configure_session_persist(false);
    std::env::set_var("SPANDA_TELEMETRY_STORE", "1");
    assert!(persist_enabled());
    std::env::set_var("SPANDA_TELEMETRY_STORE", "true");
    assert!(env_persist_enabled());
    std::env::remove_var("SPANDA_TELEMETRY_STORE");
    assert!(!persist_enabled());
}

#[test]
fn record_health_event_appends_to_store() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("telemetry.jsonl");
    let heartbeat_path = dir.path().join("heartbeats.json");
    let mut store = PersistentTelemetryStore::open(store_path, heartbeat_path);
    store
        .append(TelemetryEvent::Health {
            target: "overall".into(),
            status: "Degraded".into(),
            timestamp_ms: 1500.0,
            session_id: None,
        })
        .unwrap();
    let events = store.read_all().unwrap();
    assert_eq!(events.len(), 1);
    assert!(matches!(
        &events[0],
        TelemetryEvent::Health {
            target,
            status,
            timestamp_ms,
            ..
        } if target == "overall" && status == "Degraded" && *timestamp_ms == 1500.0
    ));
}

#[test]
fn device_heartbeat_updates_index_and_history() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("telemetry.jsonl");
    let heartbeat_path = dir.path().join("heartbeats.json");
    let mut store = PersistentTelemetryStore::open(store_path, heartbeat_path.clone());

    store
        .touch_device_heartbeat("temp-1", 1000.0, 5000.0, None, Some("mqtt"))
        .unwrap();
    store
        .touch_device_heartbeat("temp-1", 2000.0, 5000.0, None, Some("mqtt"))
        .unwrap();
    store
        .touch_device_heartbeat("temp-1", 7000.0, 5000.0, None, Some("mqtt"))
        .unwrap();

    let device_heartbeats: Vec<_> = store
        .read_all()
        .unwrap()
        .into_iter()
        .filter(|event| matches!(event, TelemetryEvent::DeviceHeartbeat { .. }))
        .collect();
    assert_eq!(device_heartbeats.len(), 2);
    assert_eq!(
        store.heartbeat_index().devices.get("temp-1"),
        Some(&7000.0)
    );
}

#[test]
fn is_heartbeat_metric_detects_liveness_names() {
    assert!(spanda_telemetry_store::is_heartbeat_metric("heartbeat"));
    assert!(spanda_telemetry_store::is_heartbeat_metric("Liveness"));
    assert!(!spanda_telemetry_store::is_heartbeat_metric("battery"));
}

#[test]
fn record_topic_publish_stores_device_event() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("telemetry.jsonl");
    let heartbeat_path = dir.path().join("heartbeats.json");
    let mut store = PersistentTelemetryStore::open(store_path, heartbeat_path);
    store
        .append(TelemetryEvent::Device {
            device_id: "Rover".into(),
            metric: "/telemetry".into(),
            value: serde_json::json!({"kind":"string","value":"ok"}),
            timestamp_ms: 1200.0,
            robot_id: Some("Rover".into()),
            session_id: None,
        })
        .unwrap();
    let latest = store.latest_device("Rover", "/telemetry").unwrap().unwrap();
    assert!(matches!(
        latest,
        TelemetryEvent::Device {
            device_id,
            metric,
            robot_id: Some(rid),
            ..
        } if device_id == "Rover" && metric == "/telemetry" && rid == "Rover"
    ));
}

#[test]
fn session_query_filters_events_to_run_window() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("telemetry.jsonl");
    let heartbeat_path = dir.path().join("heartbeats.json");
    let mut store = PersistentTelemetryStore::open(store_path, heartbeat_path);
    store
        .append(TelemetryEvent::Session {
            session_id: "run-1".into(),
            phase: "start".into(),
            source: Some("a.sd".into()),
            mission_trace_path: None,
            timestamp_ms: 100.0,
        })
        .unwrap();
    store
        .append(TelemetryEvent::Sensor {
            sensor_id: "lidar".into(),
            sensor_type: "Lidar".into(),
            value: serde_json::json!({}),
            timestamp_ms: 150.0,
            robot_id: None,
            session_id: Some("run-1".into()),
        })
        .unwrap();
    store
        .append(TelemetryEvent::Sensor {
            sensor_id: "lidar".into(),
            sensor_type: "Lidar".into(),
            value: serde_json::json!({}),
            timestamp_ms: 250.0,
            robot_id: None,
            session_id: Some("other-run".into()),
        })
        .unwrap();
    store
        .append(TelemetryEvent::Session {
            session_id: "run-1".into(),
            phase: "end".into(),
            source: None,
            mission_trace_path: Some("a.trace".into()),
            timestamp_ms: 200.0,
        })
        .unwrap();
    let events = store
        .query(&TelemetryQuery {
            session_id: Some("run-1".into()),
            ..TelemetryQuery::default()
        })
        .unwrap();
    assert_eq!(events.len(), 3);
    assert!(matches!(events[0], TelemetryEvent::Session { .. }));
    assert!(matches!(events[1], TelemetryEvent::Sensor { .. }));
    assert!(matches!(&events[2], TelemetryEvent::Session { phase, .. } if phase == "end"));
}

#[test]
fn session_query_falls_back_to_timestamp_window_for_legacy_events() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("telemetry.jsonl");
    let heartbeat_path = dir.path().join("heartbeats.json");
    let mut store = PersistentTelemetryStore::open(store_path, heartbeat_path);
    store
        .append(TelemetryEvent::Session {
            session_id: "legacy-1".into(),
            phase: "start".into(),
            source: None,
            mission_trace_path: None,
            timestamp_ms: 100.0,
        })
        .unwrap();
    store
        .append(TelemetryEvent::Sensor {
            sensor_id: "lidar".into(),
            sensor_type: "Lidar".into(),
            value: serde_json::json!({}),
            timestamp_ms: 150.0,
            robot_id: None,
            session_id: None,
        })
        .unwrap();
    store
        .append(TelemetryEvent::Session {
            session_id: "legacy-1".into(),
            phase: "end".into(),
            source: None,
            mission_trace_path: None,
            timestamp_ms: 200.0,
        })
        .unwrap();
    let events = store
        .query(&TelemetryQuery {
            session_id: Some("legacy-1".into()),
            kind: Some("sensor".into()),
            ..TelemetryQuery::default()
        })
        .unwrap();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], TelemetryEvent::Sensor { .. }));
}

#[test]
fn max_events_env_trims_oldest_entries() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("telemetry.jsonl");
    let heartbeat_path = dir.path().join("heartbeats.json");
    let mut store = PersistentTelemetryStore::open_with_max_events(
        store_path.clone(),
        heartbeat_path,
        Some(2),
    );
    for index in 0..3 {
        store
            .append(TelemetryEvent::Health {
                target: format!("check-{index}"),
                status: "Ok".into(),
                timestamp_ms: index as f64,
                session_id: None,
            })
            .unwrap();
    }
    let events = store.read_all().unwrap();
    assert_eq!(events.len(), 2);
    assert!(matches!(
        &events[0],
        TelemetryEvent::Health { target, .. } if target == "check-1"
    ));
}

#[test]
fn append_stamps_active_session_id_on_recorded_events() {
    let dir = tempdir().unwrap();
    std::env::set_var(
        "SPANDA_TELEMETRY_STORE_PATH",
        dir.path()
            .join("telemetry.jsonl")
            .to_string_lossy()
            .to_string(),
    );
    configure_session_persist(true);
    let session_id = begin_run_session(Some("rover.sd")).unwrap();
    record_sensor_reading(
        "lidar",
        "Lidar",
        &spanda_runtime::value::RuntimeValue::String {
            value: "ok".into(),
        },
        10.0,
        Some("Rover"),
    )
    .unwrap();
    end_run_session(None, None, 20.0).unwrap();
    let store = PersistentTelemetryStore::open(
        resolve_store_path(),
        dir.path().join("heartbeats.json"),
    );
    let sensor = store
        .query(&TelemetryQuery {
            kind: Some("sensor".into()),
            session_id: Some(session_id.clone()),
            ..TelemetryQuery::default()
        })
        .unwrap();
    assert_eq!(sensor.len(), 1);
    assert_eq!(sensor[0].session_id(), Some(session_id.as_str()));
    configure_session_persist(false);
    std::env::remove_var("SPANDA_TELEMETRY_STORE_PATH");
}

#[test]
fn list_sessions_links_mission_trace_and_event_counts() {
    let dir = tempdir().unwrap();
    let store_path = dir.path().join("telemetry.jsonl");
    let heartbeat_path = dir.path().join("heartbeats.json");
    let mut store = PersistentTelemetryStore::open(store_path, heartbeat_path);
    store
        .append(TelemetryEvent::Session {
            session_id: "run-42".into(),
            phase: "start".into(),
            source: Some("rover.sd".into()),
            mission_trace_path: None,
            timestamp_ms: 1.0,
        })
        .unwrap();
    store
        .append(TelemetryEvent::Sensor {
            sensor_id: "lidar".into(),
            sensor_type: "Lidar".into(),
            value: serde_json::json!({}),
            timestamp_ms: 2.0,
            robot_id: None,
            session_id: Some("run-42".into()),
        })
        .unwrap();
    store
        .append(TelemetryEvent::Session {
            session_id: "run-42".into(),
            phase: "end".into(),
            source: None,
            mission_trace_path: Some("mission.trace".into()),
            timestamp_ms: 3.0,
        })
        .unwrap();

    let sessions = store.list_sessions().unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_id, "run-42");
    assert_eq!(
        sessions[0].mission_trace_path.as_deref(),
        Some("mission.trace")
    );
    assert_eq!(sessions[0].event_count, 3);
    assert_eq!(
        store.mission_trace_for_session("run-42").unwrap().as_deref(),
        Some("mission.trace")
    );
}
