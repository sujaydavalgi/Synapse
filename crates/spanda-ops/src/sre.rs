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
}
