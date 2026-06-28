//! WebSocket event stream client for real-time Control Center events.
//!
use crate::error::{SpandaError, SpandaResult};

/// Real-time event types emitted by Control Center.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpandaEventKind {
    HealthChanged,
    ReadinessChanged,
    MissionStarted,
    MissionPaused,
    RecoveryTriggered,
    DeviceOffline,
    TamperDetected,
    KillSwitchTriggered,
    Unknown(String),
}

impl SpandaEventKind {
    pub fn parse(raw: &str) -> Self {
        match raw {
            "health_changed" => Self::HealthChanged,
            "readiness_changed" => Self::ReadinessChanged,
            "mission_started" => Self::MissionStarted,
            "mission_paused" => Self::MissionPaused,
            "recovery_triggered" => Self::RecoveryTriggered,
            "device_offline" => Self::DeviceOffline,
            "tamper_detected" => Self::TamperDetected,
            "kill_switch_triggered" => Self::KillSwitchTriggered,
            other => Self::Unknown(other.to_string()),
        }
    }
}

/// WebSocket telemetry stream handle.
///
/// Connect to `WS /v1/stream/telemetry` on the Control Center server.
/// Full streaming requires an async runtime; use language-specific SDKs
/// (Python `TelemetryStream`, TypeScript `EventStream`) for production use.
#[derive(Debug, Clone)]
pub struct EventStream {
    pub ws_url: String,
}

impl EventStream {
    pub fn local() -> Self {
        let http = std::env::var("SPANDA_CONTROL_CENTER_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8080".into());
        let ws_url = http
            .replacen("http://", "ws://", 1)
            .replacen("https://", "wss://", 1);
        Self {
            ws_url: format!("{ws_url}/v1/stream/telemetry"),
        }
    }

    pub fn url(&self) -> &str {
        &self.ws_url
    }

    pub fn connect_hint(&self) -> SpandaResult<String> {
        Ok(format!("Connect WebSocket client to {}", self.ws_url))
    }

    pub fn parse_event_type(payload: &str) -> SpandaResult<SpandaEventKind> {
        let value: serde_json::Value =
            serde_json::from_str(payload).map_err(|e| SpandaError::validation(e.to_string()))?;
        let kind = value
            .get("type")
            .or_else(|| value.get("event"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        Ok(SpandaEventKind::parse(kind))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_event_kinds() {
        assert_eq!(
            SpandaEventKind::parse("recovery_triggered"),
            SpandaEventKind::RecoveryTriggered
        );
    }
}
