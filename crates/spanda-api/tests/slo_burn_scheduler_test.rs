//! SLO burn-rate background monitor dispatches deduplicated fast-burn alerts.

use spanda_api::slo_burn_scheduler::check_and_alert_fast_burn;
use spanda_api::ControlCenterState;
use spanda_ops::{Alert, AlertSeverity, AlertType};

fn fault_alert(id: &str, age_ms: f64) -> Alert {
    Alert {
        id: id.into(),
        alert_type: AlertType::Crash,
        severity: AlertSeverity::Critical,
        message: "fault".into(),
        source: "runtime".into(),
        timestamp_ms: spanda_ops::incidents::now_ms() - age_ms,
        delivered_via: vec![],
    }
}

#[test]
fn fast_burn_dispatches_critical_alert_once() {
    std::env::set_var("SPANDA_SRE_BURN_RATE_FAST", "1");
    std::env::set_var("SPANDA_SRE_BURN_WINDOW_HOURS", "1");
    let mut state = ControlCenterState::new();
    for index in 0..3 {
        state.alert_store.push(fault_alert(&format!("fault-{index}"), 100.0));
    }
    assert!(check_and_alert_fast_burn(&mut state));
    let alerts = state.alert_store.list_owned();
    assert!(alerts.iter().any(|alert| alert.source == "slo-burn-monitor"));
    assert!(!check_and_alert_fast_burn(&mut state));
    std::env::remove_var("SPANDA_SRE_BURN_RATE_FAST");
    std::env::remove_var("SPANDA_SRE_BURN_WINDOW_HOURS");
}
