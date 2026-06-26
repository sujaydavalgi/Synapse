//! OTLP/JSON metrics export for Control Center SRE rollups.
//!
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

/// Control Center gauge inputs for OTLP metrics rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct ControlCenterMetrics {
    pub devices_total: u64,
    pub devices_healthy: u64,
    pub alerts_total: u64,
    pub alerts_critical: u64,
    pub traces_recorded: u64,
    pub availability_percent: f64,
}

/// Render Control Center metrics as OTLP/JSON (`ExportMetricsServiceResponse` shape).
pub fn render_otlp_metrics_json(metrics: &ControlCenterMetrics) -> String {
    let now_nano = wall_nanos();
    let gauge_metrics = [
        (
            "spanda.control_center.devices.total",
            metrics.devices_total as f64,
        ),
        (
            "spanda.control_center.devices.healthy",
            metrics.devices_healthy as f64,
        ),
        (
            "spanda.control_center.alerts.total",
            metrics.alerts_total as f64,
        ),
        (
            "spanda.control_center.alerts.critical",
            metrics.alerts_critical as f64,
        ),
        (
            "spanda.control_center.traces.recorded",
            metrics.traces_recorded as f64,
        ),
        (
            "spanda.control_center.availability.percent",
            metrics.availability_percent,
        ),
    ];
    let metrics_json: Vec<Value> = gauge_metrics
        .into_iter()
        .map(|(name, value)| gauge_metric(name, value, now_nano))
        .collect();
    serde_json::to_string(&json!({
        "resourceMetrics": [{
            "resource": {
                "attributes": [
                    attr("service.name", "spanda-control-center"),
                    attr("telemetry.sdk.name", "spanda-ops"),
                ]
            },
            "scopeMetrics": [{
                "scope": { "name": "spanda.control-center.metrics" },
                "metrics": metrics_json,
            }]
        }]
    }))
    .unwrap_or_else(|_| r#"{"resourceMetrics":[]}"#.into())
}

/// Push OTLP/JSON metrics to a collector (`/v1/metrics`).
pub fn push_otlp_metrics(endpoint: &str, body: &str, token: Option<&str>) -> Result<(), String> {
    let response = spanda_deploy_http::http_request("POST", endpoint, Some(body), token)?;
    if (200..300).contains(&response.status) {
        return Ok(());
    }
    Err(format!(
        "OTLP metrics push failed: HTTP {} from {endpoint}",
        response.status
    ))
}

/// Resolve metrics endpoint from env (`SPANDA_OTLP_METRICS_ENDPOINT` or `SPANDA_OTLP_ENDPOINT`).
pub fn env_metrics_endpoint() -> Option<String> {
    if let Ok(value) = std::env::var("SPANDA_OTLP_METRICS_ENDPOINT") {
        if !value.trim().is_empty() {
            return Some(value);
        }
    }
    std::env::var("SPANDA_OTLP_ENDPOINT")
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn gauge_metric(name: &str, value: f64, time_nano: u64) -> Value {
    json!({
        "name": name,
        "gauge": {
            "dataPoints": [{
                "asDouble": value,
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

fn wall_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn otlp_metrics_json_shape() {
        let body = render_otlp_metrics_json(&ControlCenterMetrics {
            devices_total: 4,
            devices_healthy: 3,
            alerts_total: 2,
            alerts_critical: 1,
            traces_recorded: 10,
            availability_percent: 75.0,
        });
        assert!(body.contains("resourceMetrics"));
        assert!(body.contains("spanda.control_center.devices.total"));
        assert!(body.contains("spanda-control-center"));
    }
}
