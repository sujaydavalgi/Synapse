//! Lean transport configuration types for provider contracts.
//!
//! Full wire-security settings remain in `spanda-transport`; core shims convert
//! when wrapping `TransportAdapter` implementations as `TransportProvider`.
use crate::value::RuntimeValue;

/// Connection settings passed to `TransportProvider::connect`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransportConfig {
    pub broker_url: Option<String>,
    pub node_name: Option<String>,
    pub namespace: Option<String>,
    pub domain_id: Option<u32>,
    pub client_id: Option<String>,
}

/// One published message recorded by a transport provider.
#[derive(Debug, Clone, PartialEq)]
pub struct AdapterMessage {
    pub topic: String,
    pub message_type: String,
    pub value: RuntimeValue,
}
