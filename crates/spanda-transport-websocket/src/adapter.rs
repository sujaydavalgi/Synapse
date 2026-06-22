//! WebSocket `TransportAdapter` implementation with optional live broker bridge.
//!
use spanda_runtime::RuntimeValue;
use spanda_security::policy::EncryptionMode;
use spanda_transport::{
    AdapterMessage, StubTransportState, TransportAdapter, TransportConfig,
};

use crate::LiveWebsocketBridge;

/// WebSocket transport adapter with stub state and optional live session forwarding.
#[derive(Debug, Default)]
pub struct WebsocketTransportAdapterLive {
    state: StubTransportState,
    live: Option<LiveWebsocketBridge>,
}

/// Alias used by routing comm bus and provider bootstrap.
pub type WebsocketTransportAdapter = WebsocketTransportAdapterLive;

impl TransportAdapter for WebsocketTransportAdapterLive {
    fn kind(&self) -> spanda_ast::comm_decl::TransportKind {
        spanda_ast::comm_decl::TransportKind::Websocket
    }

    fn connect(&mut self, config: &TransportConfig) -> Result<(), String> {
        config.security.validate(self.kind().as_str())?;

        // Require a negotiated TLS session when encryption is enabled.
        if config.security.encryption != EncryptionMode::None && !config.tls.negotiated {
            return Err("websocket adapter requires negotiated TLS session".into());
        }

        self.state.connected = true;
        self.state.config = config.clone();

        // Connect a live WebSocket broker when SPANDA_LIVE_WEBSOCKET is set.
        if std::env::var("SPANDA_LIVE_WEBSOCKET").ok().as_deref() == Some("1") {
            if let Some(url) = config.broker_url.as_deref() {
                self.live = LiveWebsocketBridge::connect(url).ok();
            }
        }
        Ok(())
    }

    fn disconnect(&mut self) {
        self.state.connected = false;
        self.live = None;
    }

    fn is_connected(&self) -> bool {
        self.state.connected
    }

    fn publish(&mut self, topic: &str, message_type: &str, value: RuntimeValue) {
        if !self.state.connected {
            return;
        }

        // Forward string payloads to the live bridge when connected.
        if let RuntimeValue::String { value: payload } = &value {
            if let Some(live) = &self.live {
                let _ = live.publish(topic, payload);
            }
        }
        self.state.publish(topic, message_type, value);
    }

    fn subscribe(&mut self, topic: &str) {
        if self.state.connected {
            if let Some(live) = &self.live {
                let _ = live.subscribe(topic);
            }
            self.state.subscribe(topic);
        }
    }

    fn receive(&mut self, topic: &str) -> Option<RuntimeValue> {
        if !self.state.connected {
            return None;
        }

        // Prefer inbound messages from the live bridge.
        if let Some(live) = &self.live {
            if let Some(val) = live.receive(topic) {
                return Some(RuntimeValue::String { value: val });
            }
        }
        self.state.receive(topic)
    }

    fn call_service(
        &mut self,
        _service: &str,
        service_type: &str,
        _request: Option<RuntimeValue>,
    ) -> RuntimeValue {
        StubTransportState::service_result(service_type)
    }

    fn send_action(
        &mut self,
        _action: &str,
        action_type: &str,
        _goal: RuntimeValue,
    ) -> RuntimeValue {
        StubTransportState::action_result(action_type)
    }

    fn published(&self) -> Vec<AdapterMessage> {
        self.state.published.clone()
    }
}
