//! Pluggable transport adapters for ROS2, MQTT, DDS, and WebSocket.
//!
//! Each adapter records operations for simulation/testing and exposes a uniform
//! interface that real broker/node integrations can implement later.

use crate::comm::{
    CommBus, DiscoverFilter, DiscoverTarget, InMemoryCommBus, PublishedCommMessage,
    SimNetworkConfig, TransportKind,
};
use crate::runtime::RuntimeValue;
use std::collections::{HashMap, VecDeque};

// ── Transport adapter trait ───────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct TransportConfig {
    pub broker_url: Option<String>,
    pub node_name: Option<String>,
    pub namespace: Option<String>,
    pub domain_id: Option<u32>,
    pub client_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AdapterMessage {
    pub topic: String,
    pub message_type: String,
    pub value: RuntimeValue,
}

pub trait TransportAdapter {
    fn kind(&self) -> TransportKind;
    fn connect(&mut self, config: &TransportConfig) -> Result<(), String>;
    fn disconnect(&mut self);
    fn is_connected(&self) -> bool;
    fn publish(&mut self, topic: &str, message_type: &str, value: RuntimeValue);
    fn subscribe(&mut self, topic: &str);
    fn receive(&mut self, topic: &str) -> Option<RuntimeValue>;
    fn call_service(
        &mut self,
        service: &str,
        service_type: &str,
        request: Option<RuntimeValue>,
    ) -> RuntimeValue;
    fn send_action(&mut self, action: &str, action_type: &str, goal: RuntimeValue) -> RuntimeValue;
    fn published(&self) -> Vec<AdapterMessage>;
}

// ── Shared stub internals ─────────────────────────────────────────────────────

#[derive(Debug, Default)]
struct StubTransportState {
    connected: bool,
    config: TransportConfig,
    subscriptions: HashMap<String, VecDeque<RuntimeValue>>,
    published: Vec<AdapterMessage>,
}

impl StubTransportState {
    fn publish(&mut self, topic: &str, message_type: &str, value: RuntimeValue) {
        self.published.push(AdapterMessage {
            topic: topic.to_string(),
            message_type: message_type.to_string(),
            value: value.clone(),
        });
        if let Some(buf) = self.subscriptions.get_mut(topic) {
            buf.push_back(value);
        }
    }

    fn subscribe(&mut self, topic: &str) {
        self.subscriptions.entry(topic.to_string()).or_default();
    }

    fn receive(&mut self, topic: &str) -> Option<RuntimeValue> {
        self.subscriptions
            .get_mut(topic)
            .and_then(|q| q.pop_front())
    }

    fn service_result(service_type: &str) -> RuntimeValue {
        RuntimeValue::Object {
            type_name: service_type.to_string(),
            fields: HashMap::from([("ok".into(), RuntimeValue::Bool { value: true })]),
        }
    }

    fn action_result(action_type: &str) -> RuntimeValue {
        RuntimeValue::Object {
            type_name: action_type.to_string(),
            fields: HashMap::from([("success".into(), RuntimeValue::Bool { value: true })]),
        }
    }
}

macro_rules! stub_adapter {
    ($name:ident, $kind:expr) => {
        #[derive(Debug, Default)]
        pub struct $name {
            state: StubTransportState,
        }

        impl TransportAdapter for $name {
            fn kind(&self) -> TransportKind {
                $kind
            }

            fn connect(&mut self, config: &TransportConfig) -> Result<(), String> {
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
                if self.state.connected {
                    self.state.publish(topic, message_type, value);
                }
            }

            fn subscribe(&mut self, topic: &str) {
                if self.state.connected {
                    self.state.subscribe(topic);
                }
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
    };
}

stub_adapter!(Ros2TransportAdapter, TransportKind::Ros2);
stub_adapter!(MqttTransportAdapter, TransportKind::Mqtt);
stub_adapter!(DdsTransportAdapter, TransportKind::Dds);
stub_adapter!(WebsocketTransportAdapter, TransportKind::Websocket);

// ── Routing comm bus ──────────────────────────────────────────────────────────

/// Routes publish/subscribe/service/action calls to transport-specific adapters
/// while preserving in-memory semantics for simulation and discovery.
#[derive(Debug)]
pub struct RoutingCommBus {
    memory: InMemoryCommBus,
    ros2: Ros2TransportAdapter,
    mqtt: MqttTransportAdapter,
    dds: DdsTransportAdapter,
    websocket: WebsocketTransportAdapter,
    config: TransportConfig,
}

impl Default for RoutingCommBus {
    fn default() -> Self {
        Self::new()
    }
}

impl RoutingCommBus {
    pub fn new() -> Self {
        Self {
            memory: InMemoryCommBus::new(),
            ros2: Ros2TransportAdapter::default(),
            mqtt: MqttTransportAdapter::default(),
            dds: DdsTransportAdapter::default(),
            websocket: WebsocketTransportAdapter::default(),
            config: TransportConfig::default(),
        }
    }

    pub fn configure(&mut self, config: TransportConfig) {
        self.config = config.clone();
        let _ = self.ros2.connect(&config);
        let _ = self.mqtt.connect(&TransportConfig {
            broker_url: config
                .broker_url
                .clone()
                .or(Some("mqtt://localhost:1883".into())),
            client_id: config.client_id.clone().or(Some("spanda".into())),
            ..config.clone()
        });
        let _ = self.dds.connect(&TransportConfig {
            domain_id: config.domain_id.or(Some(0)),
            ..config.clone()
        });
        let _ = self.websocket.connect(&TransportConfig {
            broker_url: config
                .broker_url
                .clone()
                .or(Some("ws://localhost:9090".into())),
            ..config
        });
    }

    pub fn adapter(&self, kind: TransportKind) -> Option<&dyn TransportAdapter> {
        match kind {
            TransportKind::Ros2 => Some(&self.ros2),
            TransportKind::Mqtt => Some(&self.mqtt),
            TransportKind::Dds => Some(&self.dds),
            TransportKind::Websocket => Some(&self.websocket),
            TransportKind::Local | TransportKind::Sim => None,
        }
    }

    pub fn adapter_mut(&mut self, kind: TransportKind) -> Option<&mut dyn TransportAdapter> {
        match kind {
            TransportKind::Ros2 => Some(&mut self.ros2),
            TransportKind::Mqtt => Some(&mut self.mqtt),
            TransportKind::Dds => Some(&mut self.dds),
            TransportKind::Websocket => Some(&mut self.websocket),
            TransportKind::Local | TransportKind::Sim => None,
        }
    }

    pub fn memory(&self) -> &InMemoryCommBus {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut InMemoryCommBus {
        &mut self.memory
    }

    pub fn register_robot(&mut self, name: impl Into<String>) {
        self.memory.register_robot(name);
    }

    pub fn register_agent(&mut self, name: impl Into<String>) {
        self.memory.register_agent(name);
    }

    pub fn register_device(&mut self, name: impl Into<String>) {
        self.memory.register_device(name);
    }
}

impl CommBus for RoutingCommBus {
    fn publish(
        &mut self,
        topic_path: &str,
        message_type: &str,
        value: RuntimeValue,
        transport: TransportKind,
    ) {
        self.memory
            .publish(topic_path, message_type, value.clone(), transport);
        if let Some(adapter) = self.adapter_mut(transport) {
            adapter.publish(topic_path, message_type, value);
        }
    }

    fn subscribe(&mut self, topic_path: &str, handler: &str) {
        self.memory.subscribe(topic_path, handler);
    }

    fn receive(&mut self, topic_path: &str) -> Option<RuntimeValue> {
        self.memory.receive(topic_path)
    }

    fn call_service(
        &mut self,
        service_name: &str,
        service_type: &str,
        request: Option<RuntimeValue>,
    ) -> RuntimeValue {
        self.memory
            .call_service(service_name, service_type, request.clone())
    }

    fn send_action(
        &mut self,
        action_name: &str,
        action_type: &str,
        goal: RuntimeValue,
    ) -> RuntimeValue {
        self.memory.send_action(action_name, action_type, goal)
    }

    fn discover(&self, target: DiscoverTarget, filter: &DiscoverFilter) -> Vec<String> {
        self.memory.discover(target, filter)
    }

    fn published_messages(&self) -> Vec<PublishedCommMessage> {
        self.memory.published_messages()
    }

    fn inject_fault(&mut self, fault: &str) {
        self.memory.inject_fault(fault);
    }

    fn set_network_config(&mut self, config: SimNetworkConfig) {
        self.memory.set_network_config(config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ros2_adapter_publish_when_connected() {
        let mut adapter = Ros2TransportAdapter::default();
        assert!(!adapter.is_connected());
        adapter
            .connect(&TransportConfig {
                node_name: Some("spanda".into()),
                ..Default::default()
            })
            .unwrap();
        adapter.publish("/scan", "Scan", RuntimeValue::Bool { value: true });
        assert_eq!(adapter.published().len(), 1);
        assert_eq!(adapter.published()[0].topic, "/scan");
    }

    #[test]
    fn routing_bus_delegates_ros2_publish() {
        let mut bus = RoutingCommBus::new();
        bus.configure(TransportConfig {
            node_name: Some("bot".into()),
            ..Default::default()
        });
        bus.publish(
            "/cmd_vel",
            "Velocity",
            RuntimeValue::Bool { value: true },
            TransportKind::Ros2,
        );
        assert_eq!(bus.published_messages().len(), 1);
        assert_eq!(bus.ros2.published().len(), 1);
    }

    #[test]
    fn sim_transport_stays_in_memory_only() {
        let mut bus = RoutingCommBus::new();
        bus.publish(
            "/local",
            "String",
            RuntimeValue::Bool { value: true },
            TransportKind::Sim,
        );
        assert_eq!(bus.published_messages().len(), 1);
        assert!(bus.ros2.published().is_empty());
    }
}
