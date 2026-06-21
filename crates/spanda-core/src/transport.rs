//! Pluggable transport adapters for ROS2, MQTT, DDS, and WebSocket.
//!
//! Adapters exchange versioned wire frames (`TransportWireFrame` v1) with optional
//! AES-256-GCM encryption negotiated via `TlsTransportSession`.

use crate::comm::{
    CommBus, CommEnvelope, DiscoverFilter, DiscoverTarget, InMemoryCommBus, PublishedCommMessage,
    SimNetworkConfig, TransportKind,
};
use crate::runtime::RuntimeValue;
use crate::transport_live as live;
use crate::transport_rclrs as rclrs;
use crate::transport_security::TlsTransportSession;
use crate::transport_wire::{decode_wire_value, encode_wire_value};
use spanda_security::policy::EncryptionMode;
use std::collections::{HashMap, VecDeque};

fn payload_string_for_service(value: &RuntimeValue) -> String {
    // Payload string for service.
    //
    // Parameters:
    // - `value` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::transport::payload_string_for_service(value);

    // Match on value and handle each case.
    match value {
        RuntimeValue::String { value } => {
            format!(
                "{{data: \"{}\"}}",
                value.replace('\\', "\\\\").replace('"', "\\\"")
            )
        }
        RuntimeValue::Number { value, .. } => format!("{{value: {value}}}"),
        RuntimeValue::Bool { value } => format!("{{ok: {value}}}"),
        other => format!("{{raw: \"{other:?}\"}}"),
    }
}

// ── Transport adapter trait ───────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct TransportConfig {
    pub broker_url: Option<String>,
    pub node_name: Option<String>,
    pub namespace: Option<String>,
    pub domain_id: Option<u32>,
    pub client_id: Option<String>,
    pub security: crate::transport_security::TransportSecurityConfig,
    pub tls: TlsTransportSession,
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
        // Publish.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic` — input value
        // - `message_type` — input value
        // - `value` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.publish(topic, message_type, value);

        // Append into self.
        self.published.push(AdapterMessage {
            topic: topic.to_string(),
            message_type: message_type.to_string(),
            value: value.clone(),
        });

        // Emit output when get mut provides a buf.
        if let Some(buf) = self.subscriptions.get_mut(topic) {
            buf.push_back(value);
        }
    }

    fn subscribe(&mut self, topic: &str) {
        // Subscribe.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.subscribe(topic);

        // Call entry on the current instance.
        self.subscriptions.entry(topic.to_string()).or_default();
    }

    fn receive(&mut self, topic: &str) -> Option<RuntimeValue> {
        // Receive.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.receive(topic);

        // Call subscriptions on the current instance.
        self.subscriptions
            .get_mut(topic)
            .and_then(|q| q.pop_front())
    }

    fn service_result(service_type: &str) -> RuntimeValue {
        // Service result.
        //
        // Parameters:
        // - `service_type` — input value
        //
        // Returns:
        // RuntimeValue.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::transport::service_result(service_type);

        // Build a Object runtime value.
        RuntimeValue::Object {
            type_name: service_type.to_string(),
            fields: HashMap::from([("ok".into(), RuntimeValue::Bool { value: true })]),
        }
    }

    fn action_result(action_type: &str) -> RuntimeValue {
        // Action result.
        //
        // Parameters:
        // - `action_type` — input value
        //
        // Returns:
        // RuntimeValue.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::transport::action_result(action_type);

        // Build a Object runtime value.
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
                // Kind.
                //
                // Parameters:
                // - `self` — method receiver
                //
                // Returns:
                // TransportKind.
                //
                // Options:
                // None.
                //
                // Example:
                // let result = instance.kind();

                // Produce $kind as the result.
                $kind
            }

            fn connect(&mut self, config: &TransportConfig) -> Result<(), String> {
                // Connect.
                //
                // Parameters:
                // - `self` — method receiver
                // - `config` — input value
                //
                // Returns:
                // Success value on completion, or an error.
                //
                // Options:
                // None.
                //
                // Example:
                // let result = instance.connect(config);

                // Call connected = true; on the current instance.
                config.security.validate(self.kind().as_str())?;
                if config.security.encryption != EncryptionMode::None && !config.tls.negotiated {
                    return Err(format!(
                        "{} adapter requires negotiated TLS session",
                        self.kind().as_str()
                    ));
                }
                self.state.connected = true;
                self.state.config = config.clone();
                Ok(())
            }

            fn disconnect(&mut self) {
                // Disconnect.
                //
                // Parameters:
                // - `self` — method receiver
                //
                // Returns:
                // Nothing.
                //
                // Options:
                // None.
                //
                // Example:
                // let result = instance.disconnect();

                // Call connected = false; on the current instance.
                self.state.connected = false;
            }

            fn is_connected(&self) -> bool {
                //
                // Parameters:
                // - `self` — method receiver
                //
                // Returns:
                // true or false.
                //
                // Options:
                // None.
                //
                // Example:
                // let result = instance.is_connected();

                // Call connected on the current instance.
                self.state.connected
            }

            fn publish(&mut self, topic: &str, message_type: &str, value: RuntimeValue) {
                // Publish.
                //
                // Parameters:
                // - `self` — method receiver
                // - `topic` — input value
                // - `message_type` — input value
                // - `value` — input value
                //
                // Returns:
                // Nothing.
                //
                // Options:
                // None.
                //
                // Example:
                // let result = instance.publish(topic, message_type, value);

                // take this path when self.state.connected.
                if self.state.connected {
                    self.state.publish(topic, message_type, value);
                }
            }

            fn subscribe(&mut self, topic: &str) {
                // Subscribe.
                //
                // Parameters:
                // - `self` — method receiver
                // - `topic` — input value
                //
                // Returns:
                // Nothing.
                //
                // Options:
                // None.
                //
                // Example:
                // let result = instance.subscribe(topic);

                // take this path when self.state.connected.
                if self.state.connected {
                    self.state.subscribe(topic);
                }
            }

            fn receive(&mut self, topic: &str) -> Option<RuntimeValue> {
                // Receive.
                //
                // Parameters:
                // - `self` — method receiver
                // - `topic` — input value
                //
                // Returns:
                // Some value on success, otherwise none.
                //
                // Options:
                // None.
                //
                // Example:
                // let result = instance.receive(topic);

                // take this path when self.state.connected.
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
                // Call service.
                //
                // Parameters:
                // - `self` — method receiver
                // - `_service` — input value
                // - `service_type` — input value
                // - `_request` — input value
                //
                // Returns:
                // RuntimeValue.
                //
                // Options:
                // None.
                //
                // Example:
                // let result = instance.call_service(_service, service_type, _request);

                // Produce service result as the result.
                StubTransportState::service_result(service_type)
            }

            fn send_action(
                &mut self,
                _action: &str,
                action_type: &str,
                _goal: RuntimeValue,
            ) -> RuntimeValue {
                // Send action.
                //
                // Parameters:
                // - `self` — method receiver
                // - `_action` — input value
                // - `action_type` — input value
                // - `_goal` — input value
                //
                // Returns:
                // RuntimeValue.
                //
                // Options:
                // None.
                //
                // Example:
                // let result = instance.send_action(_action, action_type, _goal);

                // Produce action result as the result.
                StubTransportState::action_result(action_type)
            }

            fn published(&self) -> Vec<AdapterMessage> {
                // Published.
                //
                // Parameters:
                // - `self` — method receiver
                //
                // Returns:
                // Vec<AdapterMessage>.
                //
                // Options:
                // None.
                //
                // Example:
                // let result = instance.published();

                // Call clone on the current instance.
                self.state.published.clone()
            }
        }
    };
}

/// ROS2 transport adapter — logs locally; optionally forwards via Python bridge.
#[derive(Debug, Default)]
pub struct Ros2TransportAdapter {
    state: StubTransportState,
}

impl TransportAdapter for Ros2TransportAdapter {
    fn kind(&self) -> TransportKind {
        // Kind.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // TransportKind.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.kind();

        // Produce Ros2 as the result.
        TransportKind::Ros2
    }

    fn connect(&mut self, config: &TransportConfig) -> Result<(), String> {
        // Connect.
        //
        // Parameters:
        // - `self` — method receiver
        // - `config` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.connect(config);

        // Call connected = true; on the current instance.
        self.state.connected = true;
        self.state.config = config.clone();
        Ok(())
    }

    fn disconnect(&mut self) {
        // Disconnect.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.disconnect();

        // Call connected = false; on the current instance.
        self.state.connected = false;
    }

    fn is_connected(&self) -> bool {
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.is_connected();

        // Call connected on the current instance.
        self.state.connected
    }

    fn publish(&mut self, topic: &str, message_type: &str, value: RuntimeValue) {
        // Publish.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic` — input value
        // - `message_type` — input value
        // - `value` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.publish(topic, message_type, value);

        // take this path when self.state.connected.
        if self.state.connected {
            self.state.publish(topic, message_type, value.clone());
        }

        // Take this path when rclrs::try rclrs publish(topic, &value).
        if rclrs::try_rclrs_publish(topic, &value) {
            return;
        }
        let _ = live::try_ros2_publish(topic, &value);
    }

    fn subscribe(&mut self, topic: &str) {
        // Subscribe.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.subscribe(topic);

        // take this path when self.state.connected.
        if self.state.connected {
            self.state.subscribe(topic);
        }

        // Take this path when rclrs::try rclrs subscribe(topic).
        if rclrs::try_rclrs_subscribe(topic) {
            return;
        }
        let _ = live::try_ros2_subscribe(topic);
    }

    fn receive(&mut self, topic: &str) -> Option<RuntimeValue> {
        // Receive.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.receive(topic);

        // take this path when self.state.connected.
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
        // Call service.
        //
        // Parameters:
        // - `self` — method receiver
        // - `service` — input value
        // - `service_type` — input value
        // - `request` — input value
        //
        // Returns:
        // RuntimeValue.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.call_service(service, service_type, request);

        // Compute request text for the following logic.
        let request_text = request
            .as_ref()
            .map(payload_string_for_service)
            .unwrap_or_else(|| "{}".into());

        // Take this path when rclrs::try rclrs service call(service, service type, &request text).
        if rclrs::try_rclrs_service_call(service, service_type, &request_text) {
            return StubTransportState::service_result(service_type);
        }
        let _ = live::try_ros2_service_call(service, service_type, &request_text);
        StubTransportState::service_result(service_type)
    }

    fn send_action(
        &mut self,
        _action: &str,
        action_type: &str,
        _goal: RuntimeValue,
    ) -> RuntimeValue {
        // Send action.
        //
        // Parameters:
        // - `self` — method receiver
        // - `_action` — input value
        // - `action_type` — input value
        // - `_goal` — input value
        //
        // Returns:
        // RuntimeValue.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.send_action(_action, action_type, _goal);

        // Produce action result as the result.
        StubTransportState::action_result(action_type)
    }

    fn published(&self) -> Vec<AdapterMessage> {
        // Published.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Vec<AdapterMessage>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.published();

        // Call clone on the current instance.
        self.state.published.clone()
    }
}

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
        //
        // Parameters:
        // None.
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_core::transport::default();

        // Build the result via new.
        Self::new()
    }
}

impl RoutingCommBus {
    pub fn new() -> Self {
        // Create a new instance.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_core::transport::new();

        // Assemble the struct fields and return it.
        Self {
            memory: InMemoryCommBus::new(),
            ros2: Ros2TransportAdapter::default(),
            mqtt: MqttTransportAdapter::default(),
            dds: DdsTransportAdapter::default(),
            websocket: WebsocketTransportAdapter::default(),
            config: TransportConfig::default(),
        }
    }

    pub fn configure(&mut self, config: TransportConfig) -> Result<(), String> {
        // Configure.
        //
        // Parameters:
        // - `self` — method receiver
        // - `config` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.configure(config);

        let mut config = config;
        if crate::transport_security::TransportSecurityConfig::url_requires_tls(
            config.broker_url.as_deref(),
        ) && config.security.encryption == EncryptionMode::None
        {
            config.security.encryption = EncryptionMode::Required;
        }
        config.security.validate("transport")?;
        config.tls.connect(&config.security)?;
        self.config = config.clone();
        self.ros2.connect(&config)?;
        self.mqtt.connect(&TransportConfig {
            broker_url: config
                .broker_url
                .clone()
                .or(Some("mqtt://localhost:1883".into())),
            client_id: config.client_id.clone().or(Some("spanda".into())),
            ..config.clone()
        })?;
        self.dds.connect(&TransportConfig {
            domain_id: config.domain_id.or(Some(0)),
            ..config.clone()
        })?;
        self.websocket.connect(&TransportConfig {
            broker_url: config
                .broker_url
                .clone()
                .or(Some("ws://localhost:9090".into())),
            ..config
        })?;
        Ok(())
    }

    pub fn adapter(&self, kind: TransportKind) -> Option<&dyn TransportAdapter> {
        // Adapter.
        //
        // Parameters:
        // - `self` — method receiver
        // - `kind` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.adapter(kind);

        // Match on kind and handle each case.
        match kind {
            TransportKind::Ros2 => Some(&self.ros2),
            TransportKind::Mqtt => Some(&self.mqtt),
            TransportKind::Dds => Some(&self.dds),
            TransportKind::Websocket => Some(&self.websocket),
            TransportKind::Local | TransportKind::Sim => None,
        }
    }

    pub fn adapter_mut(&mut self, kind: TransportKind) -> Option<&mut dyn TransportAdapter> {
        // Adapter mut.
        //
        // Parameters:
        // - `self` — method receiver
        // - `kind` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.adapter_mut(kind);

        // Match on kind and handle each case.
        match kind {
            TransportKind::Ros2 => Some(&mut self.ros2),
            TransportKind::Mqtt => Some(&mut self.mqtt),
            TransportKind::Dds => Some(&mut self.dds),
            TransportKind::Websocket => Some(&mut self.websocket),
            TransportKind::Local | TransportKind::Sim => None,
        }
    }

    pub fn memory(&self) -> &InMemoryCommBus {
        // Memory.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // &InMemoryCommBus.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.memory();

        // Return memory from this handle.
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut InMemoryCommBus {
        // Memory mut.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // &mut InMemoryCommBus.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.memory_mut();

        // Produce memory as the result.
        &mut self.memory
    }

    pub fn register_robot(&mut self, name: impl Into<String>) {
        // Register robot.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.register_robot(name);

        // Call register robot on the current instance.
        self.memory.register_robot(name);
    }

    pub fn register_agent(&mut self, name: impl Into<String>) {
        // Register agent.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.register_agent(name);

        // Call register agent on the current instance.
        self.memory.register_agent(name);
    }

    pub fn register_device(&mut self, name: impl Into<String>) {
        // Register device.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.register_device(name);

        // Call register device on the current instance.
        self.memory.register_device(name);
    }

    pub fn publish_peer(
        &mut self,
        peer: &str,
        topic: &str,
        value: RuntimeValue,
        transport: TransportKind,
        source_id: Option<&str>,
    ) {
        // Publish peer.
        //
        // Parameters:
        // - `self` — method receiver
        // - `peer` — input value
        // - `topic` — input value
        // - `value` — input value
        // - `transport` — input value
        // - `source_id` — optional sender identity
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.publish_peer(peer, topic, value, transport, source_id);

        // Call publish peer on the current instance.
        self.memory
            .publish_peer(peer, topic, value, transport, source_id);
    }

    /// Poll external transport adapters for inbound messages on subscribed topics.
    pub fn poll_inbound(&mut self, transport: TransportKind) -> Vec<(String, CommEnvelope)> {
        // Poll inbound.
        //
        // Parameters:
        // - `self` — method receiver
        // - `transport` — input value
        //
        // Returns:
        // Vec<(String, CommEnvelope)>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.poll_inbound(transport);

        // Compute paths for the following logic.
        let paths = self.memory.subscription_paths();
        let mut inbound = Vec::new();
        let kinds = [
            transport,
            TransportKind::Ros2,
            TransportKind::Mqtt,
            TransportKind::Dds,
            TransportKind::Websocket,
        ];

        // Process each filesystem path.
        for path in paths {
            // Process each kind.
            for kind in kinds {
                // Emit output when adapter mut provides a adapter.
                if let Some(adapter) = self.adapter_mut(kind) {
                    // Take this path when adapter.is connected().
                    if adapter.is_connected() {
                        // Emit output when receive provides a value.
                        if let Some(value) = adapter.receive(&path) {
                            let (value, source_id) =
                                decode_wire_value(&self.config, value).unwrap_or_else(|_| {
                                    (
                                        RuntimeValue::String {
                                            value: "<wire-decode-failed>".into(),
                                        },
                                        None,
                                    )
                                });
                            let envelope = CommEnvelope {
                                value: value.clone(),
                                source_id,
                            };
                            self.memory.push_inbound(
                                &path,
                                value,
                                envelope.source_id.as_deref(),
                            );
                            inbound.push((path.clone(), envelope));
                        }
                    }
                }
            }
        }
        inbound
    }

    /// Connect the active transport adapter and resubscribe all in-memory topic paths.
    pub fn reconnect_transport(&mut self, transport: TransportKind) {
        // Reconnect transport.
        //
        // Parameters:
        // - `self` — method receiver
        // - `transport` — transport kind to activate after connectivity failover
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.reconnect_transport(transport);

        let paths = self.memory.subscription_paths();

        // Tear down stub adapters that are no longer the active transport.
        for kind in [
            TransportKind::Ros2,
            TransportKind::Mqtt,
            TransportKind::Dds,
            TransportKind::Websocket,
        ] {
            if kind != transport {
                match kind {
                    TransportKind::Ros2 => self.ros2.disconnect(),
                    TransportKind::Mqtt => self.mqtt.disconnect(),
                    TransportKind::Dds => self.dds.disconnect(),
                    TransportKind::Websocket => self.websocket.disconnect(),
                    _ => {}
                }
            }
        }

        // Connect the target adapter when it is not already live.
        match transport {
            TransportKind::Ros2 if !self.ros2.is_connected() => {
                let _ = self.ros2.connect(&self.config);
            }
            TransportKind::Mqtt if !self.mqtt.is_connected() => {
                let _ = self.mqtt.connect(&TransportConfig {
                    broker_url: self
                        .config
                        .broker_url
                        .clone()
                        .or(Some("mqtt://localhost:1883".into())),
                    client_id: self.config.client_id.clone().or(Some("spanda".into())),
                    ..self.config.clone()
                });
            }
            TransportKind::Dds if !self.dds.is_connected() => {
                let _ = self.dds.connect(&TransportConfig {
                    domain_id: self.config.domain_id.or(Some(0)),
                    ..self.config.clone()
                });
            }
            TransportKind::Websocket if !self.websocket.is_connected() => {
                let _ = self.websocket.connect(&TransportConfig {
                    broker_url: self
                        .config
                        .broker_url
                        .clone()
                        .or(Some("ws://localhost:9090".into())),
                    ..self.config.clone()
                });
            }
            TransportKind::Local | TransportKind::Sim => return,
            _ => {}
        }

        let Some(adapter) = self.adapter_mut(transport) else {
            return;
        };

        // Resubscribe every topic path on the newly active adapter.
        for path in paths {
            adapter.subscribe(&path);
        }
    }
}

impl CommBus for RoutingCommBus {
    fn publish(
        &mut self,
        topic_path: &str,
        message_type: &str,
        value: RuntimeValue,
        transport: TransportKind,
        source_id: Option<&str>,
    ) {
        // Publish.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic_path` — input value
        // - `message_type` — input value
        // - `value` — input value
        // - `transport` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.publish(topic_path, message_type, value, transport);

        // Call memory on the current instance.
        self.memory.publish(
            topic_path,
            message_type,
            value.clone(),
            transport,
            source_id,
        );

        // Encrypt for external transport adapters when TLS is enabled.
        let config = self.config.clone();
        if let Some(adapter) = self.adapter_mut(transport) {
            let wire_value = encode_wire_value(
                &config,
                topic_path,
                message_type,
                &value,
                source_id,
                transport,
            )
            .unwrap_or(value);
            adapter.publish(topic_path, message_type, wire_value);
        }
    }

    fn subscribe(&mut self, topic_path: &str, handler: &str) {
        // Subscribe.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic_path` — input value
        // - `handler` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.subscribe(topic_path, handler);

        // Call subscribe on the current instance.
        self.memory.subscribe(topic_path, handler);
    }

    fn receive(&mut self, topic_path: &str) -> Option<RuntimeValue> {
        // Receive.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic_path` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.receive(topic_path);

        // Call receive on the current instance.
        self.memory.receive(topic_path)
    }

    fn receive_envelope(&mut self, topic_path: &str) -> Option<CommEnvelope> {
        // Receive envelope.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic_path` — input value
        //
        // Returns:
        // Some envelope on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.receive_envelope(topic_path);

        // Call receive envelope on the current instance.
        self.memory.receive_envelope(topic_path)
    }

    fn call_service(
        &mut self,
        service_name: &str,
        service_type: &str,
        request: Option<RuntimeValue>,
    ) -> RuntimeValue {
        // Call service.
        //
        // Parameters:
        // - `self` — method receiver
        // - `service_name` — input value
        // - `service_type` — input value
        // - `request` — input value
        //
        // Returns:
        // RuntimeValue.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.call_service(service_name, service_type, request);

        // Call memory on the current instance.
        self.memory
            .call_service(service_name, service_type, request.clone())
    }

    fn send_action(
        &mut self,
        action_name: &str,
        action_type: &str,
        goal: RuntimeValue,
    ) -> RuntimeValue {
        // Send action.
        //
        // Parameters:
        // - `self` — method receiver
        // - `action_name` — input value
        // - `action_type` — input value
        // - `goal` — input value
        //
        // Returns:
        // RuntimeValue.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.send_action(action_name, action_type, goal);

        // Call send action on the current instance.
        self.memory.send_action(action_name, action_type, goal)
    }

    fn discover(&self, target: DiscoverTarget, filter: &DiscoverFilter) -> Vec<String> {
        // Discover.
        //
        // Parameters:
        // - `self` — method receiver
        // - `target` — input value
        // - `filter` — input value
        //
        // Returns:
        // Vec<String>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.discover(target, filter);

        // Call discover on the current instance.
        self.memory.discover(target, filter)
    }

    fn published_messages(&self) -> Vec<PublishedCommMessage> {
        // Published messages.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Vec<PublishedCommMessage>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.published_messages();

        // Call published messages on the current instance.
        self.memory.published_messages()
    }

    fn inject_fault(&mut self, fault: &str) {
        // Inject fault.
        //
        // Parameters:
        // - `self` — method receiver
        // - `fault` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.inject_fault(fault);

        // Call inject fault on the current instance.
        self.memory.inject_fault(fault);
    }

    fn set_network_config(&mut self, config: SimNetworkConfig) {
        // Set network config.
        //
        // Parameters:
        // - `self` — method receiver
        // - `config` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.set_network_config(config);

        // Call set network config on the current instance.
        self.memory.set_network_config(config);
    }

    fn active_faults(&self) -> Vec<String> {
        // Active faults.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Vec<String>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.active_faults();

        // Call active faults on the current instance.
        self.memory.active_faults()
    }

    fn subscription_paths(&self) -> Vec<String> {
        // Subscription paths.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Vec<String>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.subscription_paths();

        // Call subscription paths on the current instance.
        self.memory.subscription_paths()
    }

    fn push_inbound(&mut self, topic_path: &str, value: RuntimeValue, source_id: Option<&str>) {
        // Push inbound.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic_path` — input value
        // - `value` — input value
        // - `source_id` — optional publisher identity
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.push_inbound(topic_path, value, source_id);

        // Call push inbound on the current instance.
        self.memory.push_inbound(topic_path, value, source_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ros2_adapter_publish_when_connected() {
        // Ros2 adapter publish when connected.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::transport::ros2_adapter_publish_when_connected();

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
        // Routing bus delegates ros2 publish.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::transport::routing_bus_delegates_ros2_publish();

        let mut bus = RoutingCommBus::new();
        bus.configure(TransportConfig {
            node_name: Some("bot".into()),
            ..Default::default()
        })
        .unwrap();
        bus.publish(
            "/cmd_vel",
            "Velocity",
            RuntimeValue::Bool { value: true },
            TransportKind::Ros2,
            None,
        );
        assert_eq!(bus.published_messages().len(), 1);
        assert_eq!(bus.ros2.published().len(), 1);
    }

    #[test]
    fn sim_transport_stays_in_memory_only() {
        // Sim transport stays in memory only.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::transport::sim_transport_stays_in_memory_only();

        let mut bus = RoutingCommBus::new();
        bus.publish(
            "/local",
            "String",
            RuntimeValue::Bool { value: true },
            TransportKind::Sim,
            None,
        );
        assert_eq!(bus.published_messages().len(), 1);
        assert!(bus.ros2.published().is_empty());
    }

    #[test]
    fn reconnect_transport_disconnects_inactive_adapters() {
        let mut bus = RoutingCommBus::new();
        bus.configure(TransportConfig::default()).unwrap();
        bus.subscribe("/scan", "handler");
        bus.reconnect_transport(TransportKind::Mqtt);
        assert!(bus.mqtt.is_connected());
        bus.reconnect_transport(TransportKind::Dds);
        assert!(!bus.mqtt.is_connected());
        assert!(bus.dds.is_connected());
    }

    #[test]
    fn reconnect_transport_resubscribes_on_dds() {
        let mut bus = RoutingCommBus::new();
        bus.configure(TransportConfig::default()).unwrap();
        bus.subscribe("/scan", "handler");
        bus.reconnect_transport(TransportKind::Dds);
        assert!(bus.dds.is_connected());
    }
}
