//! Prometheus text exposition for the persistent telemetry store.

use crate::error::TelemetryStoreResult;
use crate::record::{HeartbeatIndex, TelemetryEvent};
use crate::store::{stats_from_events, PersistentTelemetryStore, TelemetryStats};
use serde_json::Value;

/// Render store contents in Prometheus text exposition format.
pub fn render_prometheus(store: &PersistentTelemetryStore) -> TelemetryStoreResult<String> {
    let stats = store.stats()?;
    let index = store.heartbeat_index();
    let events = store.read_all()?;
    Ok(render_prometheus_from_events(&events, &stats, &index))
}

/// Render Prometheus text exposition for an in-memory event slice.
pub fn render_prometheus_from_events(
    events: &[TelemetryEvent],
    stats: &TelemetryStats,
    index: &HeartbeatIndex,
) -> String {
    let mut out = String::new();

    append_event_totals(&mut out, stats);
    append_heartbeat_gauges(&mut out, index);
    append_latest_runtime_metrics(&mut out, events);
    append_latest_device_metrics(&mut out, events);
    append_health_gauges(&mut out, events);

    out
}

/// Render Prometheus text exposition using stats derived from events.
pub fn render_prometheus_stats_from_events(
    events: &[TelemetryEvent],
    index: &HeartbeatIndex,
) -> String {
    let stats = stats_from_events(events, index);
    render_prometheus_from_events(events, &stats, index)
}

fn append_event_totals(out: &mut String, stats: &TelemetryStats) {
    write_help_type(
        out,
        "spanda_telemetry_events_total",
        "gauge",
        "Telemetry events in the persistent store by kind.",
    );
    for (kind, count) in [
        ("device", stats.device_events),
        ("sensor", stats.sensor_events),
        ("heartbeat", stats.heartbeat_events),
        ("device_heartbeat", stats.device_heartbeat_events),
        ("health", stats.health_events),
        ("session", stats.session_events),
        ("runtime_metrics", stats.runtime_metrics_events),
    ] {
        write_metric(
            out,
            "spanda_telemetry_events_total",
            &[("kind", kind)],
            count as f64,
        );
    }
    write_help_type(
        out,
        "spanda_telemetry_tracked_total",
        "gauge",
        "Tracked tasks and devices in the heartbeat index.",
    );
    write_metric(
        out,
        "spanda_telemetry_tracked_total",
        &[("target", "task")],
        stats.tracked_tasks as f64,
    );
    write_metric(
        out,
        "spanda_telemetry_tracked_total",
        &[("target", "device")],
        stats.tracked_devices as f64,
    );
}

fn append_heartbeat_gauges(
    out: &mut String,
    index: &crate::record::HeartbeatIndex,
) {
    write_help_type(
        out,
        "spanda_task_heartbeat_last_timestamp_ms",
        "gauge",
        "Latest task heartbeat timestamp in simulation milliseconds.",
    );
    for (task, timestamp_ms) in &index.tasks {
        write_metric(
            out,
            "spanda_task_heartbeat_last_timestamp_ms",
            &[("task", task.as_str())],
            *timestamp_ms,
        );
    }
    write_help_type(
        out,
        "spanda_device_heartbeat_last_timestamp_ms",
        "gauge",
        "Latest device heartbeat timestamp in simulation milliseconds.",
    );
    for (device, timestamp_ms) in &index.devices {
        write_metric(
            out,
            "spanda_device_heartbeat_last_timestamp_ms",
            &[("device", device.as_str())],
            *timestamp_ms,
        );
    }
}

fn append_latest_runtime_metrics(out: &mut String, events: &[TelemetryEvent]) {
    let Some(metrics) = events
        .iter()
        .rev()
        .find_map(|event| match event {
            TelemetryEvent::RuntimeMetrics { metrics, .. } => Some(metrics),
            _ => None,
        })
    else {
        return;
    };

    write_help_type(
        out,
        "spanda_scheduler_ticks",
        "gauge",
        "Scheduler ticks from the latest runtime_metrics snapshot.",
    );
    write_metric(
        out,
        "spanda_scheduler_ticks",
        &[],
        json_f64(metrics, &["scheduler", "scheduler_ticks"]),
    );
    write_help_type(
        out,
        "spanda_scheduler_emergency_stops",
        "gauge",
        "Emergency stops from the latest runtime_metrics snapshot.",
    );
    write_metric(
        out,
        "spanda_scheduler_emergency_stops",
        &[],
        json_f64(metrics, &["scheduler", "emergency_stops"]),
    );
    write_help_type(
        out,
        "spanda_task_ticks",
        "gauge",
        "Task ticks from the latest runtime_metrics snapshot.",
    );
    if let Some(tasks) = metrics.get("tasks").and_then(Value::as_object) {
        for (task, body) in tasks {
            write_metric(
                out,
                "spanda_task_ticks",
                &[("task", task.as_str())],
                json_f64(body, &["ticks"]),
            );
        }
    }
    write_help_type(
        out,
        "spanda_task_missed_deadlines",
        "gauge",
        "Missed task deadlines from the latest runtime_metrics snapshot.",
    );
    if let Some(tasks) = metrics.get("tasks").and_then(Value::as_object) {
        for (task, body) in tasks {
            write_metric(
                out,
                "spanda_task_missed_deadlines",
                &[("task", task.as_str())],
                json_f64(body, &["missed_deadlines"]),
            );
        }
    }
}

fn append_latest_device_metrics(out: &mut String, events: &[TelemetryEvent]) {
    write_help_type(
        out,
        "spanda_device_metric_value",
        "gauge",
        "Latest numeric device metric sample from the telemetry store.",
    );
    let mut latest: std::collections::HashMap<(String, String), f64> =
        std::collections::HashMap::new();
    for event in events {
        if let TelemetryEvent::Device {
            device_id,
            metric,
            value,
            ..
        } = event
        {
            if let Some(number) = numeric_json_value(value) {
                latest.insert((device_id.clone(), metric.clone()), number);
            }
        }
    }
    for ((device_id, metric), value) in latest {
        write_metric(
            out,
            "spanda_device_metric_value",
            &[("device", device_id.as_str()), ("metric", metric.as_str())],
            value,
        );
    }
}

fn append_health_gauges(out: &mut String, events: &[TelemetryEvent]) {
    write_help_type(
        out,
        "spanda_health_status",
        "gauge",
        "Latest health status (1=Healthy, 0.5=Degraded, 0=Unhealthy).",
    );
    let mut latest: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for event in events {
        if let TelemetryEvent::Health { target, status, .. } = event {
            latest.insert(target.clone(), health_score(status));
        }
    }
    for (target, score) in latest {
        write_metric(
            out,
            "spanda_health_status",
            &[("target", target.as_str())],
            score,
        );
    }
}

fn health_score(status: &str) -> f64 {
    match status.to_ascii_lowercase().as_str() {
        "healthy" | "ok" => 1.0,
        "degraded" | "warn" | "warning" => 0.5,
        _ => 0.0,
    }
}

fn numeric_json_value(value: &Value) -> Option<f64> {
    if let Some(number) = value.as_f64() {
        return Some(number);
    }
    if let Some(object) = value.as_object() {
        if let Some(inner) = object.get("value").and_then(Value::as_f64) {
            return Some(inner);
        }
    }
    None
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

fn write_help_type(out: &mut String, name: &str, metric_type: &str, help: &str) {
    out.push_str("# HELP ");
    out.push_str(name);
    out.push_str(" ");
    out.push_str(help);
    out.push('\n');
    out.push_str("# TYPE ");
    out.push_str(name);
    out.push(' ');
    out.push_str(metric_type);
    out.push('\n');
}

fn write_metric(out: &mut String, name: &str, labels: &[(&str, &str)], value: f64) {
    out.push_str(name);
    if !labels.is_empty() {
        out.push('{');
        for (index, (key, value)) in labels.iter().enumerate() {
            if index > 0 {
                out.push(',');
            }
            out.push_str(key);
            out.push_str("=\"");
            out.push_str(&escape_label(value));
            out.push('"');
        }
        out.push('}');
    }
    out.push(' ');
    if value.is_finite() {
        out.push_str(&value.to_string());
    } else {
        out.push_str("0");
    }
    out.push('\n');
}

fn escape_label(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::TelemetryEvent;
    use tempfile::tempdir;

    #[test]
    fn render_prometheus_includes_stats_and_runtime_metrics() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("telemetry.jsonl");
        let heartbeat_path = dir.path().join("heartbeats.json");
        let mut store = PersistentTelemetryStore::open(store_path, heartbeat_path);
        store
            .append(TelemetryEvent::Device {
                device_id: "Rover".into(),
                metric: "battery".into(),
                value: serde_json::json!({"kind":"number","value":82.0}),
                timestamp_ms: 1.0,
            robot_id: Some("Rover".into()),
            session_id: None,
        })
        .unwrap();
        store
            .append(TelemetryEvent::RuntimeMetrics {
                session_id: "run-1".into(),
                metrics: serde_json::json!({
                    "scheduler": {"scheduler_ticks": 12.0, "emergency_stops": 1.0},
                    "tasks": {"control": {"ticks": 5.0, "missed_deadlines": 2.0}}
                }),
                timestamp_ms: 2.0,
            })
            .unwrap();
        store
            .touch_heartbeat("control", 42.0, 5000.0, Some("Rover"))
            .unwrap();

        let body = render_prometheus(&store).unwrap();
        assert!(body.contains("spanda_telemetry_events_total{kind=\"device\"} 1"));
        assert!(body.contains("spanda_task_heartbeat_last_timestamp_ms{task=\"control\"} 42"));
        assert!(body.contains("spanda_scheduler_ticks 12"));
        assert!(body.contains("spanda_task_ticks{task=\"control\"} 5"));
        assert!(body.contains("spanda_device_metric_value{device=\"Rover\",metric=\"battery\"} 82"));
    }
}
