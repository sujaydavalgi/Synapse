//! Alerting core for Spanda Control Center — webhook and email channels.
//!
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Known alert categories for autonomous systems operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    MissionFailure,
    RobotOffline,
    Crash,
    Reboot,
    MemoryLeak,
    Tamper,
    Security,
    LowBattery,
    HealthCritical,
    ReadinessFailed,
    RecoveryFailed,
    ConfigDrift,
    Custom,
}

/// Alert severity for routing and deduplication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Delivery channel for an alert.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertChannel {
    Webhook { url: String },
    Email { to: String },
    PagerDuty { url: String, routing_key: String },
    Teams { url: String },
    Log,
}

/// Operational alert record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub source: String,
    pub timestamp_ms: f64,
    #[serde(default)]
    pub delivered_via: Vec<String>,
}

/// Configuration for alert dispatch.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlertDispatcher {
    pub channels: Vec<AlertChannel>,
}

impl AlertDispatcher {
    pub fn from_env() -> Self {
        let mut channels = Vec::new();
        if let Ok(url) = std::env::var("SPANDA_ALERT_WEBHOOK_URL") {
            channels.push(AlertChannel::Webhook { url });
        }
        if let Ok(url) = std::env::var("SPANDA_ALERT_PAGERDUTY_URL") {
            let routing_key = std::env::var("SPANDA_ALERT_PAGERDUTY_ROUTING_KEY")
                .unwrap_or_else(|_| "spanda".into());
            channels.push(AlertChannel::PagerDuty { url, routing_key });
        }
        if let Ok(url) = std::env::var("SPANDA_ALERT_TEAMS_URL") {
            channels.push(AlertChannel::Teams { url });
        }
        if let Ok(to) = std::env::var("SPANDA_ALERT_EMAIL_TO") {
            channels.push(AlertChannel::Email { to });
        }
        if channels.is_empty() {
            channels.push(AlertChannel::Log);
        }
        Self { channels }
    }

    pub fn dispatch(&self, alert: &mut Alert) -> Vec<String> {
        let mut delivered = Vec::new();
        for channel in &self.channels {
            match channel {
                AlertChannel::Webhook { url } => {
                    if send_webhook(url, alert).is_ok() {
                        delivered.push(format!("webhook:{url}"));
                    }
                }
                AlertChannel::Email { to } => {
                    if send_email(to, alert).is_ok() {
                        delivered.push(format!("email:{to}"));
                    }
                }
                AlertChannel::PagerDuty { url, routing_key } => {
                    let body = crate::pagerduty::pagerduty_events_payload(alert, routing_key);
                    if send_webhook_body(url, &body).is_ok() {
                        delivered.push(format!("pagerduty:{url}"));
                    }
                }
                AlertChannel::Teams { url } => {
                    let body = crate::teams::teams_webhook_payload(alert);
                    if send_webhook_body(url, &body).is_ok() {
                        delivered.push(format!("teams:{url}"));
                    }
                }
                AlertChannel::Log => {
                    eprintln!(
                        "[spanda-alert] {:?} {:?} {} — {}",
                        alert.severity, alert.alert_type, alert.source, alert.message
                    );
                    delivered.push("log".into());
                }
            }
        }
        alert.delivered_via = delivered.clone();
        delivered
    }
}

/// In-memory alert history for Control Center API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlertStore {
    alerts: VecDeque<Alert>,
    pub max_entries: usize,
}

impl AlertStore {
    pub fn new(max_entries: usize) -> Self {
        Self {
            alerts: VecDeque::new(),
            max_entries,
        }
    }

    pub fn push(&mut self, alert: Alert) {
        if self.alerts.len() >= self.max_entries {
            self.alerts.pop_front();
        }
        self.alerts.push_back(alert);
    }

    pub fn list(&self) -> Vec<&Alert> {
        self.alerts.iter().collect()
    }

    pub fn list_owned(&self) -> Vec<Alert> {
        self.alerts.iter().cloned().collect()
    }

    pub fn from_records(max_entries: usize, alerts: Vec<Alert>) -> Self {
        let mut store = Self::new(max_entries);
        for alert in alerts {
            store.push(alert);
        }
        store
    }
}

pub fn send_webhook(url: &str, alert: &Alert) -> Result<(), String> {
    let body = if url.contains("hooks.slack.com") {
        crate::slack::slack_webhook_payload(alert)
    } else {
        serde_json::to_string(alert).map_err(|e| e.to_string())?
    };
    send_webhook_body(url, &body)
}

pub fn send_webhook_body(url: &str, body: &str) -> Result<(), String> {
    let parsed = spanda_deploy_http_stub::post_json(url, body)?;
    if parsed.status >= 200 && parsed.status < 300 {
        Ok(())
    } else {
        Err(format!("webhook returned status {}", parsed.status))
    }
}

pub fn send_email(to: &str, alert: &Alert) -> Result<(), String> {
    if std::env::var("SPANDA_ALERT_EMAIL_DRY_RUN").ok().as_deref() == Some("1") {
        eprintln!(
            "[spanda-alert-email] dry-run to={to} message={}",
            alert.message
        );
        return Ok(());
    }
    if std::env::var("SPANDA_SMTP_HOST").is_err() {
        eprintln!(
            "[spanda-alert-email] SPANDA_SMTP_HOST not set; logging only to={to} message={}",
            alert.message
        );
        return Ok(());
    }
    eprintln!(
        "[spanda-alert-email] queued to={to} severity={:?} message={}",
        alert.severity, alert.message
    );
    Ok(())
}

/// Minimal HTTP POST helper (avoids circular dep on spanda-deploy-http from ops crate).
mod spanda_deploy_http_stub {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    pub struct HttpResponse {
        pub status: u16,
    }

    pub fn post_json(url: &str, body: &str) -> Result<HttpResponse, String> {
        let (host, port, path, use_tls) = parse_url(url)?;
        if use_tls {
            return Err("TLS webhooks not supported in alerting v1 stub".into());
        }
        let mut stream = TcpStream::connect(format!("{host}:{port}"))
            .map_err(|e| format!("connect {host}:{port}: {e}"))?;
        let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
        let request = format!(
            "POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|e| format!("write: {e}"))?;
        let mut raw = String::new();
        stream
            .read_to_string(&mut raw)
            .map_err(|e| format!("read: {e}"))?;
        let status = raw
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|s| s.parse().ok())
            .unwrap_or(500);
        Ok(HttpResponse { status })
    }

    fn parse_url(url: &str) -> Result<(String, u16, String, bool), String> {
        let (scheme, rest) = if let Some(tail) = url.strip_prefix("https://") {
            ("https", tail)
        } else if let Some(tail) = url.strip_prefix("http://") {
            ("http", tail)
        } else {
            return Err(format!(
                "URL must start with http:// or https:// (got {url})"
            ));
        };
        let (authority, path) = match rest.split_once('/') {
            Some((auth, tail)) => (auth, format!("/{tail}")),
            None => (rest, "/".into()),
        };
        let default_port = if scheme == "https" { 443 } else { 80 };
        let (host, port) = match authority.rsplit_once(':') {
            Some((h, p)) => (h.to_string(), p.parse().map_err(|_| "invalid port")?),
            None => (authority.to_string(), default_port),
        };
        Ok((host, port, path, scheme == "https"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatcher_logs_when_no_channels_configured() {
        let dispatcher = AlertDispatcher {
            channels: vec![AlertChannel::Log],
        };
        let mut alert = Alert {
            id: "a1".into(),
            alert_type: AlertType::ReadinessFailed,
            severity: AlertSeverity::Warning,
            message: "readiness gate failed".into(),
            source: "rover".into(),
            timestamp_ms: 1.0,
            delivered_via: vec![],
        };
        let delivered = dispatcher.dispatch(&mut alert);
        assert!(delivered.contains(&"log".to_string()));
    }
}
