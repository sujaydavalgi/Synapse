//! Pluggable transport routing for ROS2, MQTT, DDS, and WebSocket.
//!
//! **Lean-core note:** `TransportAdapter` trait and wire protocol types live in
//! `spanda-transport`. Adapter implementations live in `spanda-transport-*`
//! crates; this module re-exports them and hosts `RoutingCommBus`.
//!
//! Adapters exchange versioned wire frames (`TransportWireFrame` v1) with optional
//! AES-256-GCM encryption negotiated via `TlsTransportSession`.

use crate::comm::{
    CommBus, CommEnvelope, DiscoverFilter, DiscoverTarget, InMemoryCommBus, PublishedCommMessage,
    SimNetworkConfig, TransportKind,
};
use crate::runtime::RuntimeValue;
use crate::transport_wire::{decode_wire_value, encode_wire_value};
use crate::providers::{ProviderRegistry, TransportProvider};
use spanda_security::policy::EncryptionMode;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub use spanda_transport::{
    payload_string_for_service, AdapterMessage, StubTransportState, TransportAdapter,
    TransportConfig,
};
pub use spanda_transport_dds::{DdsTransportAdapter, DdsTransportAdapterLive};
pub use spanda_transport_mqtt::MqttTransportAdapter;
pub use spanda_transport_ros2::Ros2TransportAdapter;
pub use spanda_transport_websocket::{WebsocketTransportAdapter, WebsocketTransportAdapterLive};

// ── Routing comm bus ──────────────────────────────────────────────────────────
/// Routes publish/subscribe/service/action calls to transport-specific adapters
/// while preserving in-memory semantics for simulation and discovery.
pub struct RoutingCommBus {
    memory: InMemoryCommBus,
    ros2: Ros2TransportAdapter,
    mqtt: MqttTransportAdapter,
    dds: DdsTransportAdapter,
    websocket: WebsocketTransportAdapter,
    config: TransportConfig,
    providers: Option<Rc<RefCell<ProviderRegistry>>>,
    registry_keys: HashMap<TransportKind, String>,
    registry_backed: HashSet<TransportKind>,
}

impl std::fmt::Debug for RoutingCommBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoutingCommBus")
            .field("memory", &self.memory)
            .field("config", &self.config)
            .field("registry_backed", &self.registry_backed)
            .finish_non_exhaustive()
    }
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
            providers: None,
            registry_keys: HashMap::new(),
            registry_backed: HashSet::new(),
        }
    }

    pub fn attach_provider_registry(&mut self, registry: Rc<RefCell<ProviderRegistry>>) {
        self.providers = Some(registry);
    }

    pub fn mark_registry_backed(&mut self, kind: TransportKind, key: String) {
        self.registry_backed.insert(kind);
        self.registry_keys.insert(kind, key);
    }

    pub fn clear_registry_backed(&mut self) {
        self.registry_backed.clear();
        self.registry_keys.clear();
    }

    pub fn is_registry_backed(&self, kind: TransportKind) -> bool {
        self.registry_backed.contains(&kind)
    }

    fn uses_registry_transport(&self, kind: TransportKind) -> bool {
        self.registry_backed.contains(&kind) && self.providers.is_some()
    }

    fn with_registry_transport<F, R>(&self, kind: TransportKind, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn TransportProvider) -> R,
    {
        if !self.uses_registry_transport(kind) {
            return None;
        }
        let key = self.registry_keys.get(&kind)?.clone();
        let providers = self.providers.as_ref()?;
        providers.borrow_mut().with_transport(&key, f)
    }

    fn publish_external(
        &mut self,
        kind: TransportKind,
        topic_path: &str,
        message_type: &str,
        value: RuntimeValue,
    ) {
        if self.uses_registry_transport(kind) {
            let _ = self.with_registry_transport(kind, |provider| {
                if provider.is_connected() {
                    provider.publish(topic_path, message_type, value);
                }
            });
            return;
        }
        if let Some(adapter) = self.adapter_mut(kind) {
            adapter.publish(topic_path, message_type, value);
        }
    }

    fn subscribe_external(&mut self, kind: TransportKind, topic_path: &str) {
        if let Some(()) = self.with_registry_transport(kind, |provider| {
            provider.subscribe(topic_path);
        }) {
            return;
        }
        if let Some(adapter) = self.adapter_mut(kind) {
            adapter.subscribe(topic_path);
        }
    }

    fn receive_external(&mut self, kind: TransportKind, topic_path: &str) -> Option<RuntimeValue> {
        if let Some(value) = self
            .with_registry_transport(kind, |provider| {
                if provider.is_connected() {
                    provider.receive(topic_path)
                } else {
                    None
                }
            })
            .flatten()
        {
            return Some(value);
        }
        if let Some(adapter) = self.adapter_mut(kind) {
            if adapter.is_connected() {
                return adapter.receive(topic_path);
            }
        }
        None
    }

    fn connect_external(&mut self, kind: TransportKind, config: &TransportConfig) {
        if let Some(()) = self.with_registry_transport(kind, |provider| {
            let _ = provider.connect(config);
        }) {
            return;
        }
        let _ = match kind {
            TransportKind::Ros2 => self.ros2.connect(config),
            TransportKind::Mqtt => self.mqtt.connect(config),
            TransportKind::Dds => self.dds.connect(config),
            TransportKind::Websocket => self.websocket.connect(config),
            TransportKind::Local | TransportKind::Sim => Ok(()),
        };
    }

    fn disconnect_external(&mut self, kind: TransportKind) {
        if let Some(()) = self.with_registry_transport(kind, |provider| {
            provider.disconnect();
        }) {
            return;
        }
        match kind {
            TransportKind::Ros2 => self.ros2.disconnect(),
            TransportKind::Mqtt => self.mqtt.disconnect(),
            TransportKind::Dds => self.dds.disconnect(),
            TransportKind::Websocket => self.websocket.disconnect(),
            TransportKind::Local | TransportKind::Sim => {}
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
        config
            .tls
            .connect(&config.security, config.broker_url.as_deref())?;
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
        if self.uses_registry_transport(kind) {
            return None;
        }
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
        if self.uses_registry_transport(kind) {
            return None;
        }
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

    fn is_external_connected(&self, kind: TransportKind) -> bool {
        if let Some(connected) = self.with_registry_transport(kind, |provider| provider.is_connected())
        {
            return connected;
        }
        self.adapter(kind)
            .map(|adapter| adapter.is_connected())
            .unwrap_or(false)
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
                if !self.is_external_connected(kind) {
                    continue;
                }

                // Emit output when receive provides a value.
                if let Some(value) = self.receive_external(kind, &path) {
                    let (value, source_id) = decode_wire_value(&self.config, value)
                        .unwrap_or_else(|_| {
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
                    self.memory
                        .push_inbound(&path, value, envelope.source_id.as_deref());
                    inbound.push((path.clone(), envelope));
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
        let config = self.config.clone();

        // Tear down transports that are no longer the active kind.
        for kind in [
            TransportKind::Ros2,
            TransportKind::Mqtt,
            TransportKind::Dds,
            TransportKind::Websocket,
        ] {
            if kind != transport {
                self.disconnect_external(kind);
            }
        }

        // Connect the target transport when it is not already live.
        match transport {
            TransportKind::Ros2 if !self.is_external_connected(TransportKind::Ros2) => {
                self.connect_external(TransportKind::Ros2, &config);
            }
            TransportKind::Mqtt if !self.is_external_connected(TransportKind::Mqtt) => {
                self.connect_external(
                    TransportKind::Mqtt,
                    &TransportConfig {
                        broker_url: config
                            .broker_url
                            .clone()
                            .or(Some("mqtt://localhost:1883".into())),
                        client_id: config.client_id.clone().or(Some("spanda".into())),
                        ..config.clone()
                    },
                );
            }
            TransportKind::Dds if !self.is_external_connected(TransportKind::Dds) => {
                self.connect_external(
                    TransportKind::Dds,
                    &TransportConfig {
                        domain_id: config.domain_id.or(Some(0)),
                        ..config.clone()
                    },
                );
            }
            TransportKind::Websocket if !self.is_external_connected(TransportKind::Websocket) => {
                self.connect_external(
                    TransportKind::Websocket,
                    &TransportConfig {
                        broker_url: config
                            .broker_url
                            .clone()
                            .or(Some("ws://localhost:9090".into())),
                        ..config
                    },
                );
            }
            TransportKind::Local | TransportKind::Sim => return,
            _ => {}
        }

        // Resubscribe every topic path on the newly active transport.
        for path in paths {
            self.subscribe_external(transport, &path);
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
        if matches!(transport, TransportKind::Local | TransportKind::Sim) {
            return;
        }
        let wire_value = encode_wire_value(
            &config,
            topic_path,
            message_type,
            &value,
            source_id,
            transport,
        )
        .unwrap_or(value);
        self.publish_external(transport, topic_path, message_type, wire_value);
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
