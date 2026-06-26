//! Microsoft Teams incoming-webhook payload formatting for alerts.
//!
use crate::alerting::Alert;
use serde_json::json;

/// Format an alert as a Teams MessageCard-compatible JSON body.
pub fn teams_webhook_payload(alert: &Alert) -> String {
    json!({
        "@type": "MessageCard",
        "@context": "https://schema.org/extensions",
        "summary": alert.message,
        "themeColor": "D13438",
        "title": format!("Spanda {:?} alert", alert.severity),
        "sections": [{
            "facts": [
                { "name": "Type", "value": format!("{:?}", alert.alert_type) },
                { "name": "Source", "value": alert.source },
                { "name": "Message", "value": alert.message }
            ]
        }]
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerting::{AlertSeverity, AlertType};

    #[test]
    fn teams_payload_contains_message() {
        let alert = Alert {
            id: "a1".into(),
            alert_type: AlertType::Tamper,
            severity: AlertSeverity::Critical,
            message: "tamper detected".into(),
            source: "security".into(),
            timestamp_ms: 1.0,
            delivered_via: vec![],
        };
        let body = teams_webhook_payload(&alert);
        assert!(body.contains("tamper detected"));
    }
}
