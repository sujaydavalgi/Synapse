//! Optional live WebSocket broker integration via tungstenite.

use crate::runtime::RuntimeValue;

#[cfg(feature = "live-websocket")]
mod live {
    use super::*;
    use std::collections::{HashMap, VecDeque};
    use std::net::TcpStream;
    use std::sync::Mutex;
    use std::time::Duration;
    use tungstenite::{connect, Message, WebSocket};

    #[derive(serde::Serialize, serde::Deserialize)]
    struct WireEnvelope {
        topic: String,
        payload: String,
    }

    pub struct LiveWebsocketBridge {
        socket: Mutex<WebSocket<TcpStream>>,
        inbound: Mutex<HashMap<String, VecDeque<RuntimeValue>>>,
    }

    impl LiveWebsocketBridge {
        pub fn connect(broker_url: &str) -> Result<Self, String> {
            let (socket, _response) =
                connect(broker_url).map_err(|e| format!("websocket connect failed: {e}"))?;
            Ok(Self {
                socket: Mutex::new(socket),
                inbound: Mutex::new(HashMap::new()),
            })
        }

        fn poll_inbound(&self) {
            let mut guard = match self.socket.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            let _ = guard
                .get_ref()
                .set_read_timeout(Some(Duration::from_millis(50)));
            while let Ok(Message::Text(text)) = guard.read() {
                if let Ok(frame) = serde_json::from_str::<WireEnvelope>(&text) {
                    if let Ok(mut map) = self.inbound.lock() {
                        map.entry(frame.topic)
                            .or_default()
                            .push_back(RuntimeValue::String {
                                value: frame.payload,
                            });
                    }
                }
            }
        }

        pub fn publish(&self, topic: &str, payload: &str) -> Result<(), String> {
            self.poll_inbound();
            let envelope = WireEnvelope {
                topic: topic.to_string(),
                payload: payload.to_string(),
            };
            let text = serde_json::to_string(&envelope)
                .map_err(|e| format!("websocket serialize failed: {e}"))?;
            let mut guard = self
                .socket
                .lock()
                .map_err(|e| format!("websocket lock failed: {e}"))?;
            guard
                .send(Message::Text(text))
                .map_err(|e| format!("websocket send failed: {e}"))
        }

        pub fn subscribe(&self, topic: &str) -> Result<(), String> {
            let envelope = WireEnvelope {
                topic: topic.to_string(),
                payload: "__subscribe__".into(),
            };
            let text = serde_json::to_string(&envelope)
                .map_err(|e| format!("websocket subscribe serialize failed: {e}"))?;
            let mut guard = self
                .socket
                .lock()
                .map_err(|e| format!("websocket lock failed: {e}"))?;
            guard
                .send(Message::Text(text))
                .map_err(|e| format!("websocket subscribe failed: {e}"))
        }

        pub fn receive(&self, topic: &str) -> Option<RuntimeValue> {
            self.poll_inbound();
            let mut map = self.inbound.lock().ok()?;
            map.get_mut(topic).and_then(|q| q.pop_front())
        }
    }
}

/// Live WebSocket bridge handle; inactive unless built with `live-websocket`.
#[derive(Debug, Default)]
pub struct LiveWebsocketBridge {
    #[cfg(feature = "live-websocket")]
    inner: Option<live::LiveWebsocketBridge>,
}

impl LiveWebsocketBridge {
    pub fn connect(broker_url: &str) -> Result<Self, String> {
        #[cfg(feature = "live-websocket")]
        {
            return Ok(Self {
                inner: Some(live::LiveWebsocketBridge::connect(broker_url)?),
            });
        }
        #[cfg(not(feature = "live-websocket"))]
        {
            let _ = broker_url;
            Err("live WebSocket support not enabled (build with --features live-websocket)".into())
        }
    }

    pub fn publish(&self, topic: &str, payload: &str) -> Result<(), String> {
        #[cfg(feature = "live-websocket")]
        if let Some(inner) = &self.inner {
            return inner.publish(topic, payload);
        }
        let _ = (topic, payload);
        Ok(())
    }

    pub fn subscribe(&self, topic: &str) -> Result<(), String> {
        #[cfg(feature = "live-websocket")]
        if let Some(inner) = &self.inner {
            return inner.subscribe(topic);
        }
        let _ = topic;
        Ok(())
    }

    pub fn receive(&self, topic: &str) -> Option<RuntimeValue> {
        #[cfg(feature = "live-websocket")]
        if let Some(inner) = &self.inner {
            return inner.receive(topic);
        }
        let _ = topic;
        None
    }
}
