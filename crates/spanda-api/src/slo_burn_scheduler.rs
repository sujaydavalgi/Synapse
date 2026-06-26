//! Background SLO burn-rate monitoring with alert dispatch on fast burn.
//!
use crate::handlers::{now_ms, record_alert};
use crate::state::ControlCenterState;
use spanda_ops::{Alert, AlertSeverity, AlertType, slo_burn_rate_summary};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn recent_fast_burn_alert(state: &ControlCenterState, window_ms: f64) -> bool {
    let cutoff = now_ms() - window_ms;
    state.alert_store.list().iter().any(|alert| {
        alert.alert_type == AlertType::HealthCritical
            && alert.source == "slo-burn-monitor"
            && alert.timestamp_ms >= cutoff
            && alert.message.contains("fast burn")
    })
}

pub fn check_and_alert_fast_burn(state: &mut ControlCenterState) -> bool {
    let alerts = state.alert_store.list_owned();
    let summary = slo_burn_rate_summary(&alerts);
    let fast_burn = summary
        .get("fast_burn")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    if !fast_burn {
        return false;
    }
    let window_hours = summary
        .get("window_hours")
        .and_then(|value| value.as_f64())
        .unwrap_or(1.0);
    if recent_fast_burn_alert(state, window_hours * 3_600_000.0) {
        return false;
    }
    let rate = summary
        .get("rate")
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0);
    let recent = summary
        .get("recent_fault_alerts")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    let alert = Alert {
        id: format!("slo-burn-{}", now_ms()),
        alert_type: AlertType::HealthCritical,
        severity: AlertSeverity::Critical,
        message: format!(
            "SLO error budget fast burn detected (rate={rate:.2}, recent_fault_alerts={recent})"
        ),
        source: "slo-burn-monitor".into(),
        timestamp_ms: now_ms(),
        delivered_via: vec![],
    };
    record_alert(state, alert);
    true
}

pub fn spawn_slo_burn_monitor(state: Arc<Mutex<ControlCenterState>>) {
    let interval_secs = std::env::var("SPANDA_SRE_BURN_SCAN_INTERVAL_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    if interval_secs == 0 {
        return;
    }
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(interval_secs));
            if let Ok(mut guard) = state.lock() {
                let _ = check_and_alert_fast_burn(&mut guard);
            }
        }
    });
}
