//! SRE rollup helpers — SLO targets, MTBF hints, and health trend summaries.
//!
use crate::alerting::{Alert, AlertSeverity, AlertType};
use serde_json::{json, Value};

/// Target availability percent from `SPANDA_SRE_SLO_PERCENT` (default 99.0).
pub fn slo_target_percent() -> f64 {
    std::env::var("SPANDA_SRE_SLO_PERCENT")
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| *value > 0.0 && *value <= 100.0)
        .unwrap_or(99.0)
}

/// SLO status object for Control Center `/v1/sre/summary`.
pub fn slo_status(availability_percent: f64) -> Value {
    let target = slo_target_percent();
    let met = availability_percent >= target;
    json!({
        "target_percent": target,
        "met": met,
        "budget_remaining_percent": availability_percent - target,
        "env": "SPANDA_SRE_SLO_PERCENT",
    })
}

/// Fast-burn threshold multiplier from `SPANDA_SRE_BURN_RATE_FAST` (default 2.0).
pub fn slo_burn_rate_fast_threshold() -> f64 {
    std::env::var("SPANDA_SRE_BURN_RATE_FAST")
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| *value > 0.0)
        .unwrap_or(2.0)
}

/// Sliding window hours for burn-rate from `SPANDA_SRE_BURN_WINDOW_HOURS` (default 1.0).
pub fn slo_burn_rate_window_hours() -> f64 {
    std::env::var("SPANDA_SRE_BURN_WINDOW_HOURS")
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| *value > 0.0)
        .unwrap_or(1.0)
}

fn now_ms() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64() * 1000.0)
        .unwrap_or(0.0)
}

/// Burn-rate rollup from recent fault-class alerts (proxy for error-budget consumption).
pub fn slo_burn_rate_summary(alerts: &[Alert]) -> Value {
    let target = slo_target_percent();
    let window_hours = slo_burn_rate_window_hours();
    let window_ms = window_hours * 3_600_000.0;
    let cutoff = now_ms() - window_ms;
    let recent_fault_alerts = alerts
        .iter()
        .filter(|alert| is_fault_alert(alert) && alert.timestamp_ms >= cutoff)
        .count();
    let error_budget_percent = (100.0 - target).max(0.01);
    let allowed_in_window = (error_budget_percent / 100.0) * window_hours * 10.0;
    let rate = recent_fault_alerts as f64 / allowed_in_window.max(0.1);
    let fast_threshold = slo_burn_rate_fast_threshold();
    json!({
        "window_hours": window_hours,
        "recent_fault_alerts": recent_fault_alerts,
        "allowed_fault_alerts_in_window": allowed_in_window,
        "rate": rate,
        "fast_burn_threshold": fast_threshold,
        "fast_burn": rate >= fast_threshold,
        "env_fast": "SPANDA_SRE_BURN_RATE_FAST",
        "env_window": "SPANDA_SRE_BURN_WINDOW_HOURS",
    })
}

/// Mean time between fault-class alerts (milliseconds) when at least two exist.
pub fn mtbf_hint_ms(alerts: &[Alert]) -> Option<f64> {
    let mut timestamps: Vec<f64> = alerts
        .iter()
        .filter(|alert| is_fault_alert(alert))
        .map(|alert| alert.timestamp_ms)
        .collect();
    if timestamps.len() < 2 {
        return None;
    }
    timestamps.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let gaps: Vec<f64> = timestamps
        .windows(2)
        .map(|window| window[1] - window[0])
        .collect();
    Some(gaps.iter().sum::<f64>() / gaps.len() as f64)
}

/// Device-pool health trend rollup for SRE summary.
pub fn health_trends_summary(
    degraded: usize,
    failed: usize,
    offline: usize,
    total: usize,
) -> Value {
    let denominator = total.max(1) as f64;
    json!({
        "devices_total": total,
        "degraded_percent": (degraded as f64 / denominator) * 100.0,
        "failed_percent": (failed as f64 / denominator) * 100.0,
        "offline_percent": (offline as f64 / denominator) * 100.0,
    })
}

fn is_fault_alert(alert: &Alert) -> bool {
    matches!(
        alert.alert_type,
        AlertType::Crash
            | AlertType::Reboot
            | AlertType::RecoveryFailed
            | AlertType::HealthCritical
            | AlertType::RobotOffline
            | AlertType::MissionFailure
            | AlertType::ConfigDrift
    ) || alert.severity == AlertSeverity::Critical
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerting::AlertType;

    #[test]
    fn slo_met_when_availability_exceeds_target() {
        std::env::set_var("SPANDA_SRE_SLO_PERCENT", "95");
        let status = slo_status(96.0);
        assert_eq!(status["met"], true);
        std::env::remove_var("SPANDA_SRE_SLO_PERCENT");
    }

    #[test]
    fn mtbf_computed_from_fault_alert_gaps() {
        let alerts = vec![
            Alert {
                id: "a1".into(),
                alert_type: AlertType::Crash,
                severity: AlertSeverity::Critical,
                message: "crash".into(),
                source: "rover".into(),
                timestamp_ms: 0.0,
                delivered_via: vec![],
            },
            Alert {
                id: "a2".into(),
                alert_type: AlertType::Crash,
                severity: AlertSeverity::Critical,
                message: "crash".into(),
                source: "rover".into(),
                timestamp_ms: 1000.0,
                delivered_via: vec![],
            },
        ];
        assert_eq!(mtbf_hint_ms(&alerts), Some(1000.0));
    }

    #[test]
    fn burn_rate_flags_fast_consumption() {
        let now = now_ms();
        let alerts = vec![
            Alert {
                id: "b1".into(),
                alert_type: AlertType::Crash,
                severity: AlertSeverity::Critical,
                message: "crash".into(),
                source: "rover".into(),
                timestamp_ms: now - 1_000.0,
                delivered_via: vec![],
            },
            Alert {
                id: "b2".into(),
                alert_type: AlertType::Crash,
                severity: AlertSeverity::Critical,
                message: "crash".into(),
                source: "rover".into(),
                timestamp_ms: now - 2_000.0,
                delivered_via: vec![],
            },
            Alert {
                id: "b3".into(),
                alert_type: AlertType::Crash,
                severity: AlertSeverity::Critical,
                message: "crash".into(),
                source: "rover".into(),
                timestamp_ms: now - 3_000.0,
                delivered_via: vec![],
            },
        ];
        std::env::set_var("SPANDA_SRE_SLO_PERCENT", "99");
        std::env::set_var("SPANDA_SRE_BURN_RATE_FAST", "2.0");
        let summary = slo_burn_rate_summary(&alerts);
        assert!(summary["rate"].as_f64().unwrap_or(0.0) > 0.0);
        assert_eq!(summary["fast_burn"], true);
        std::env::remove_var("SPANDA_SRE_SLO_PERCENT");
        std::env::remove_var("SPANDA_SRE_BURN_RATE_FAST");
    }
}
