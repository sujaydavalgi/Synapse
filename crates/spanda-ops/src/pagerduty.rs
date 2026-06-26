//! PagerDuty Events API v2 compatible alert payload formatting.
//!
use crate::alerting::Alert;
use serde_json::json;

/// Format an alert as a PagerDuty Events API v2 JSON body.
pub fn pagerduty_events_payload(alert: &Alert, routing_key: &str) -> String {
    let severity = match alert.severity {
        crate::alerting::AlertSeverity::Critical => "critical",
        crate::alerting::AlertSeverity::Warning => "warning",
        _ => "info",
    };
    json!({
        "routing_key": routing_key,
        "event_action": "trigger",
        "payload": {
            "summary": alert.message,
            "severity": severity,
            "source": alert.source,
            "custom_details": {
                "alert_id": alert.id,
                "alert_type": format!("{:?}", alert.alert_type),
            }
        }
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerting::{AlertSeverity, AlertType};

    #[test]
    fn pagerduty_payload_contains_routing_key() {
        let alert = Alert {
            id: "a1".into(),
            alert_type: AlertType::Crash,
            severity: AlertSeverity::Critical,
            message: "process crash".into(),
            source: "runtime".into(),
            timestamp_ms: 1.0,
            delivered_via: vec![],
        };
        let body = pagerduty_events_payload(&alert, "test-routing-key");
        assert!(body.contains("test-routing-key"));
        assert!(body.contains("process crash"));
    }
}
