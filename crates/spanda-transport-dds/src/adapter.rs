//! DDS `TransportAdapter` implementation with optional live UDP multicast bridge.
//!
use spanda_runtime::RuntimeValue;
use spanda_security::policy::EncryptionMode;
use spanda_transport::{
    AdapterMessage, StubTransportState, TransportAdapter, TransportConfig,
};

use crate::LiveDdsBridge;

/// DDS transport adapter with stub state and optional live multicast forwarding.
#[derive(Debug, Default)]
pub struct DdsTransportAdapterLive {
    state: StubTransportState,
    live: Option<LiveDdsBridge>,
}

/// Alias used by routing comm bus and provider bootstrap.
pub type DdsTransportAdapter = DdsTransportAdapterLive;

impl TransportAdapter for DdsTransportAdapterLive {
    fn kind(&self) -> spanda_ast::comm_decl::TransportKind {
        spanda_ast::comm_decl::TransportKind::Dds
    }

    fn connect(&mut self, config: &TransportConfig) -> Result<(), String> {
        config.security.validate(self.kind().as_str())?;

        // Require a negotiated TLS session when encryption is enabled.
        if config.security.encryption != EncryptionMode::None && !config.tls.negotiated {
            return Err("dds adapter requires negotiated TLS session".into());
        }

        self.state.connected = true;
        self.state.config = config.clone();

        // Connect a live DDS domain when SPANDA_LIVE_DDS is set.
        if std::env::var("SPANDA_LIVE_DDS").ok().as_deref() == Some("1") {
            let domain = config.domain_id.unwrap_or(0);
            self.live = LiveDdsBridge::connect(domain).ok();
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
