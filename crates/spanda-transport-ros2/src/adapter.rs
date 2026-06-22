//! ROS2 `TransportAdapter` implementation with rclrs and live-bridge fallbacks.
//!
use spanda_runtime::RuntimeValue;
use spanda_transport::{
    payload_string_for_service, AdapterMessage, StubTransportState, TransportAdapter,
    TransportConfig,
};

use crate::live_bridge;
use crate::rclrs;

fn payload_string(value: &RuntimeValue) -> String {
    match value {
        RuntimeValue::String { value } => value.clone(),
        RuntimeValue::Number { value, .. } => value.to_string(),
        RuntimeValue::Bool { value } => value.to_string(),
        other => format!("{other:?}"),
    }
}

/// ROS2 transport adapter — logs locally; optionally forwards via rclrs or Python bridge.
#[derive(Debug, Default)]
pub struct Ros2TransportAdapter {
    state: StubTransportState,
}

impl TransportAdapter for Ros2TransportAdapter {
    fn kind(&self) -> spanda_ast::comm_decl::TransportKind {
        spanda_ast::comm_decl::TransportKind::Ros2
    }

    fn connect(&mut self, config: &TransportConfig) -> Result<(), String> {
        // Mark the adapter connected and store transport settings.
        self.state.connected = true;
        self.state.config = config.clone();
        Ok(())
    }

    fn disconnect(&mut self) {
        self.state.connected = false;
    }

    fn is_connected(&self) -> bool {
        self.state.connected
    }

    fn publish(&mut self, topic: &str, message_type: &str, value: RuntimeValue) {
        // Record publishes in stub state when connected.
        if self.state.connected {
            self.state.publish(topic, message_type, value.clone());
        }

        // Forward to rclrs or live bridge when available.
        if rclrs::try_rclrs_publish(topic, &value) {
            return;
        }
        let _ = live_bridge::try_ros2_publish(topic, &payload_string(&value));
    }

    fn subscribe(&mut self, topic: &str) {
        // Record subscriptions in stub state when connected.
        if self.state.connected {
            self.state.subscribe(topic);
        }

        // Forward to rclrs or live bridge when available.
        if rclrs::try_rclrs_subscribe(topic) {
            return;
        }
        let _ = live_bridge::try_ros2_subscribe(topic);
    }

    fn receive(&mut self, topic: &str) -> Option<RuntimeValue> {
        if self.state.connected {
            self.state.receive(topic)
        } else {
            None
        }
    }

    fn call_service(
        &mut self,
        service: &str,
        service_type: &str,
        request: Option<RuntimeValue>,
    ) -> RuntimeValue {
        let request_text = request
            .as_ref()
            .map(payload_string_for_service)
            .unwrap_or_else(|| "{}".into());

        // Forward to rclrs or live bridge when available.
        if rclrs::try_rclrs_service_call(service, service_type, &request_text) {
            return StubTransportState::service_result(service_type);
        }
        let _ = live_bridge::try_ros2_service_call(service, service_type, &request_text);
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
