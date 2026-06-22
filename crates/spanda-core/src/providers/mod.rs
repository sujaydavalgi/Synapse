//! Lean-core provider contracts and registry — compatibility shims over `spanda-runtime`.
//!
//! Spanda Core wires optional domain packages through `bootstrap` and `package_stubs`.
//! Trait definitions, registry, and shared types live in `spanda-runtime` for the
//! Phase 4 lean-core split.
//!
pub mod bootstrap;
pub mod classification;
pub mod package_stubs;
pub mod transport_adapter;

pub use bootstrap::{
    bootstrap_default_providers, bootstrap_providers_for_packages, official_package_for_transport,
    sync_comm_bus_for_official_packages,
};
pub use classification::{
    module_classifications, official_package_names, ModuleClassification, ModuleOwnership,
};
pub use spanda_runtime::providers::{
    transport_registry_key, ActuatorProvider, AdapterMessage, CloudProvider,
    ConnectivityProvider, CryptoProvider, FleetProvider, HalProvider, LedgerProvider,
    MaintenanceProvider, NavigationProvider, PositioningProvider, ProviderCapability,
    ProviderCapabilitySet, ProviderError, ProviderId, ProviderMetadata, ProviderRegistry,
    ProviderResult, ProviderSafetyLevel, RosProvider, SensorProvider, SimulationProvider,
    SlamProvider, TransportConfig, TransportProvider, VisionProvider,
};
pub use transport_adapter::{adapter_config_to_runtime, TransportAdapterProvider};

/// Re-export legacy AI provider surface for vision-capable packages.
pub use crate::ai::{AiProvider, CompletionRequest, DetectionRequest, EmbedRequest};
