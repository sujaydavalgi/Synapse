//! OTLP/JSON metrics export for the persistent telemetry store.

use crate::error::TelemetryStoreResult;
use crate::record::{HeartbeatIndex, TelemetryEvent};
use crate::store::{stats_from_events, PersistentTelemetryStore, TelemetryStats};
use serde_json::{json, Value};

/// Render store metrics as OTLP/JSON (`ExportMetricsServiceResponse` shape).
pub fn render_otlp_json(store: &PersistentTelemetryStore) -> TelemetryStoreResult<String> {
    let stats = store.stats()?;
    let index = store.heartbeat_index();
    let events = store.read_all()?;
    render_otlp_from_events(
        &events,
        &stats,
        &index,
        store.store_path().to_string_lossy().as_ref(),
    )
}

/// Render OTLP/JSON metrics for an in-memory event slice.
pub fn render_otlp_from_events(
    events: &[TelemetryEvent],
    stats: &TelemetryStats,
    index: &HeartbeatIndex,
    store_path: &str,
) -> TelemetryStoreResult<String> {
    let now_nano = wall_nanos();

    let mut metrics: Vec<Value> = Vec::new();
    for (kind, count) in event_kind_counts(stats) {
        metrics.push(gauge_metric(
            "spanda.telemetry.events",
            &[("kind", kind)],
            count as f64,
            now_nano,
        ));
    }
    metrics.push(gauge_metric(
        "spanda.telemetry.tracked",
        &[("target", "task")],
        stats.tracked_tasks as f64,
        now_nano,
    ));
    metrics.push(gauge_metric(
        "spanda.telemetry.tracked",
        &[("target", "device")],
        stats.tracked_devices as f64,
        now_nano,
    ));
    for (task, timestamp_ms) in &index.tasks {
        metrics.push(gauge_metric(
            "spanda.task.heartbeat.last_timestamp_ms",
            &[("task", task.as_str())],
            *timestamp_ms,
            now_nano,
        ));
    }
    for (device, timestamp_ms) in &index.devices {
        metrics.push(gauge_metric(
            "spanda.device.heartbeat.last_timestamp_ms",
            &[("device", device.as_str())],
            *timestamp_ms,
            now_nano,
        ));
    }
    append_runtime_metrics(&mut metrics, events, now_nano);
    append_health_metrics(&mut metrics, events, now_nano);

    let body = json!({
        "resourceMetrics": [{
            "resource": {
                "attributes": [
                    attr("service.name", "spanda"),
                    attr("spanda.store.path", store_path),
                ]
            },
            "scopeMetrics": [{
                "scope": { "name": "spanda.telemetry" },
                "metrics": metrics,
            }]
        }]
    });
    serde_json::to_string_pretty(&body)
        .map_err(|error| crate::error::TelemetryStoreError::Serialization(error.to_string()))
}

/// Render OTLP/JSON metrics using stats derived from events.
pub fn render_otlp_stats_from_events(
    events: &[TelemetryEvent],
    index: &HeartbeatIndex,
    store_path: &str,
) -> TelemetryStoreResult<String> {
    let stats = stats_from_events(events, index);
    render_otlp_from_events(events, &stats, index, store_path)
}

fn event_kind_counts(stats: &TelemetryStats) -> [(&'static str, usize); 7] {
    [
        ("device", stats.device_events),
        ("sensor", stats.sensor_events),
        ("heartbeat", stats.heartbeat_events),
        ("device_heartbeat", stats.device_heartbeat_events),
        ("health", stats.health_events),
        ("session", stats.session_events),
        ("runtime_metrics", stats.runtime_metrics_events),
    ]
}

fn append_runtime_metrics(metrics: &mut Vec<Value>, events: &[TelemetryEvent], now_nano: u64) {
    let Some(payload) = events.iter().rev().find_map(|event| match event {
        TelemetryEvent::RuntimeMetrics { metrics, .. } => Some(metrics),
        _ => None,
    }) else {
        return;
    };
    metrics.push(gauge_metric(
        "spanda.scheduler.ticks",
        &[],
        json_f64(payload, &["scheduler", "scheduler_ticks"]),
        now_nano,
    ));
    if let Some(tasks) = payload.get("tasks").and_then(Value::as_object) {
        for (task, body) in tasks {
            metrics.push(gauge_metric(
                "spanda.task.ticks",
                &[("task", task.as_str())],
                json_f64(body, &["ticks"]),
                now_nano,
            ));
        }
    }
}

fn append_health_metrics(metrics: &mut Vec<Value>, events: &[TelemetryEvent], now_nano: u64) {
    let mut latest: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for event in events {
        if let TelemetryEvent::Health { target, status, .. } = event {
            latest.insert(target.clone(), health_score(status));
        }
    }
    for (target, score) in latest {
        metrics.push(gauge_metric(
            "spanda.health.status",
            &[("target", target.as_str())],
            score,
            now_nano,
        ));
    }
}

fn gauge_metric(name: &str, labels: &[(&str, &str)], value: f64, time_nano: u64) -> Value {
    json!({
        "name": name,
        "gauge": {
            "dataPoints": [{
                "asDouble": value,
                "attributes": labels.iter().map(|(key, value)| attr(key, *value)).collect::<Vec<_>>(),
                "timeUnixNano": time_nano.to_string(),
            }]
        }
    })
}

fn attr(key: &str, value: &str) -> Value {
    json!({
        "key": key,
        "value": { "stringValue": value }
    })
}

fn json_f64(value: &Value, path: &[&str]) -> f64 {
    let mut current = value;
    for segment in path {
        let Some(next) = current.get(*segment) else {
            return 0.0;
        };
        current = next;
    }
    current.as_f64().unwrap_or(0.0)
}

fn health_score(status: &str) -> f64 {
    match status.to_ascii_lowercase().as_str() {
        "healthy" | "ok" => 1.0,
        "degraded" | "warn" | "warning" => 0.5,
        _ => 0.0,
    }
}

fn wall_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::TelemetryEvent;
    use tempfile::tempdir;

    #[test]
    fn render_otlp_json_includes_resource_metrics() {
        let dir = tempdir().unwrap();
        let mut store = PersistentTelemetryStore::open(
            dir.path().join("telemetry.jsonl"),
            dir.path().join("heartbeats.json"),
        );
        store
            .append(TelemetryEvent::Sensor {
                sensor_id: "lidar".into(),
                sensor_type: "Lidar".into(),
                value: serde_json::json!({}),
                timestamp_ms: 1.0,
                robot_id: None,
                session_id: None,
            })
            .unwrap();
        let body = render_otlp_json(&store).unwrap();
        assert!(body.contains("resourceMetrics"));
        assert!(body.contains("spanda.telemetry.events"));
        assert!(body.contains("sensor"));
    }
}
