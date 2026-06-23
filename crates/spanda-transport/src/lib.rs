//! Transport adapter traits, security policy, TLS session primitives, and wire frames.
//!
pub mod adapter;
pub mod security;
#[cfg(feature = "tls")]
pub mod tls;
pub mod wire;

pub use adapter::{
    payload_string_for_service, AdapterMessage, StubTransportState, TransportAdapter,
    TransportConfig,
};
pub use security::{
    effective_transport_policy, TlsTransportSession, TlsTransportStub, TransportSecurityConfig,
};
#[cfg(feature = "tls")]
pub use tls::{
    build_client_config, parse_tls_endpoint, perform_mtls_handshake, MtlsHandshakeResult,
    TlsEndpoint,
};
pub use wire::{decode_wire_value, encode_wire_value, TransportWireFrame};
