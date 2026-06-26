//! WebSocket telemetry stream for Control Center (`/v1/stream/telemetry`).
//!
use crate::state::SharedState;
use serde_json::json;
use spanda_telemetry_store::global_store;
use std::io::{Cursor, Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use tungstenite::{accept, Message, WebSocket};

/// True when the parsed HTTP request is a WebSocket upgrade for telemetry streaming.
pub fn is_telemetry_stream_upgrade(raw: &str, path: &str) -> bool {
    if path != "/v1/stream/telemetry" {
        return false;
    }
    let lower = raw.to_ascii_lowercase();
    lower.contains("upgrade: websocket") && lower.contains("connection:")
}

/// Serve live telemetry, API traces, and alerts over WebSocket.
pub fn serve_telemetry_websocket(
    stream: TcpStream,
    prefix: &[u8],
    state: SharedState,
) -> Result<(), String> {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));
    let prefixed = PrefixedReader {
        cursor: Cursor::new(prefix.to_vec()),
        inner: stream,
    };
    let mut websocket = accept(prefixed).map_err(|error| error.to_string())?;

    send_json(
        &mut websocket,
        &json!({
            "type": "hello",
            "version": "v1",
            "stream": "telemetry",
        }),
    )?;

    let mut telemetry_offset = 0usize;
    let mut last_trace_count = 0usize;
    let mut last_alert_count = 0usize;
    let deadline = std::time::Instant::now() + stream_duration();

    while std::time::Instant::now() < deadline {
        drain_client_messages(&mut websocket)?;

        if let Ok(guard) = state.lock() {
            let traces = guard.trace_log.list_owned();
            if traces.len() > last_trace_count {
                for record in traces.iter().skip(last_trace_count) {
                    send_json(
                        &mut websocket,
                        &json!({ "type": "trace", "record": record }),
                    )?;
                }
                last_trace_count = traces.len();
            }
            let alerts = guard.alert_store.list_owned();
            if alerts.len() > last_alert_count {
                for alert in alerts.iter().skip(last_alert_count) {
                    send_json(&mut websocket, &json!({ "type": "alert", "alert": alert }))?;
                }
                last_alert_count = alerts.len();
            }
        }

        if let Ok(store) = global_store().lock() {
            let events = store
                .read_all()
                .map_err(|error| error.to_string())?;
            if telemetry_offset < events.len() {
                for event in events.iter().skip(telemetry_offset) {
                    send_json(
                        &mut websocket,
                        &json!({ "type": "telemetry", "event": event }),
                    )?;
                }
                telemetry_offset = events.len();
            }
        }

        std::thread::sleep(Duration::from_millis(250));
    }

    let _ = websocket.close(None);
    Ok(())
}

fn send_json(websocket: &mut WebSocket<PrefixedReader<TcpStream>>, value: &serde_json::Value) -> Result<(), String> {
    let text = serde_json::to_string(value).map_err(|error| error.to_string())?;
    websocket
        .send(Message::Text(text))
        .map_err(|error| error.to_string())
}

fn drain_client_messages(websocket: &mut WebSocket<PrefixedReader<TcpStream>>) -> Result<(), String> {
    loop {
        match websocket.read() {
            Ok(Message::Close(_)) => return Err("client closed websocket".into()),
            Ok(Message::Ping(payload)) => {
                websocket
                    .send(Message::Pong(payload))
                    .map_err(|error| error.to_string())?;
            }
            Ok(Message::Pong(_)) | Ok(Message::Frame(_)) => {}
            Ok(Message::Text(_)) | Ok(Message::Binary(_)) => {}
            Err(tungstenite::Error::Io(error))
                if error.kind() == std::io::ErrorKind::WouldBlock
                    || error.kind() == std::io::ErrorKind::TimedOut =>
            {
                break;
            }
            Err(tungstenite::Error::AlreadyClosed) => return Err("websocket closed".into()),
            Err(_) => break,
        }
    }
    Ok(())
}

fn stream_duration() -> Duration {
    std::env::var("SPANDA_WS_STREAM_SECONDS")
        .ok()
        .and_then(|value| value.parse().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(5))
}

struct PrefixedReader<R: Read> {
    cursor: Cursor<Vec<u8>>,
    inner: R,
}

impl<R: Read> Read for PrefixedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let from_prefix = self.cursor.read(buf)?;
        if from_prefix > 0 {
            return Ok(from_prefix);
        }
        self.inner.read(buf)
    }
}

impl<R: Read + Write> Write for PrefixedReader<R> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_upgrade_request() {
        let raw = "GET /v1/stream/telemetry HTTP/1.1\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\r\n";
        assert!(is_telemetry_stream_upgrade(raw, "/v1/stream/telemetry"));
        assert!(!is_telemetry_stream_upgrade(raw, "/v1/health"));
    }
}
