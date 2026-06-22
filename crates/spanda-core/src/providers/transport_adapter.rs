//! Wrap legacy `spanda-transport` adapters as runtime `TransportProvider` implementations.
//!
use spanda_runtime::providers::{
    transport_types::{AdapterMessage as RuntimeAdapterMessage, TransportConfig as RuntimeTransportConfig},
    traits::TransportProvider,
    types::{ProviderId, ProviderMetadata, ProviderSafetyLevel},
};
use spanda_runtime::value::RuntimeValue;
use spanda_transport::{AdapterMessage, TransportAdapter, TransportConfig};

/// Map a full transport adapter config onto the lean provider contract.
pub fn adapter_config_to_runtime(config: &TransportConfig) -> RuntimeTransportConfig {
    RuntimeTransportConfig {
        broker_url: config.broker_url.clone(),
        node_name: config.node_name.clone(),
        namespace: config.namespace.clone(),
        domain_id: config.domain_id,
        client_id: config.client_id.clone(),
    }
}

fn runtime_config_to_adapter(config: &RuntimeTransportConfig) -> TransportConfig {
    // Map lean provider transport settings onto the full adapter configuration.
    TransportConfig {
        broker_url: config.broker_url.clone(),
        node_name: config.node_name.clone(),
        namespace: config.namespace.clone(),
        domain_id: config.domain_id,
        client_id: config.client_id.clone(),
        ..TransportConfig::default()
    }
}

fn adapter_message_to_runtime(message: AdapterMessage) -> RuntimeAdapterMessage {
    RuntimeAdapterMessage {
        topic: message.topic,
        message_type: message.message_type,
        value: message.value,
    }
}

/// Blanket adapter: wrap an existing `TransportAdapter` as a `TransportProvider`.
pub struct TransportAdapterProvider<T: TransportAdapter> {
    id: ProviderId,
    inner: T,
}

impl<T: TransportAdapter> TransportAdapterProvider<T> {
    pub fn new(package: impl Into<String>, name: impl Into<String>, inner: T) -> Self {
        Self {
            id: ProviderId::new(package, name),
            inner,
        }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T: TransportAdapter + Send + Sync> TransportProvider for TransportAdapterProvider<T> {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: self.id.clone(),
            description: format!("Transport adapter ({:?})", self.inner.kind()),
            safety_level: ProviderSafetyLevel::Development,
            capabilities_required: vec![],
            hardware_requirements: vec![],
        }
    }

    fn kind(&self) -> spanda_ast::comm_decl::TransportKind {
        self.inner.kind()
    }

    fn connect(&mut self, config: &RuntimeTransportConfig) -> Result<(), String> {
        self.inner.connect(&runtime_config_to_adapter(config))
    }

    fn disconnect(&mut self) {
        self.inner.disconnect();
    }

    fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    fn publish(&mut self, topic: &str, message_type: &str, value: RuntimeValue) {
        self.inner.publish(topic, message_type, value);
    }

    fn subscribe(&mut self, topic: &str) {
        self.inner.subscribe(topic);
    }

    fn receive(&mut self, topic: &str) -> Option<RuntimeValue> {
        self.inner.receive(topic)
    }

    fn call_service(
        &mut self,
        service: &str,
        service_type: &str,
        request: Option<RuntimeValue>,
    ) -> RuntimeValue {
        self.inner.call_service(service, service_type, request)
    }

    fn send_action(&mut self, action: &str, action_type: &str, goal: RuntimeValue) -> RuntimeValue {
        self.inner.send_action(action, action_type, goal)
    }

    fn published(&self) -> Vec<RuntimeAdapterMessage> {
        self.inner
            .published()
            .into_iter()
            .map(adapter_message_to_runtime)
            .collect()
    }
}
