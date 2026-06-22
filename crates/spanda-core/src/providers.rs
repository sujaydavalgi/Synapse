//! Lean-core provider contracts and registry — compatibility facade over workspace crates.
//!
pub use spanda_providers::{
    adapter_config_to_runtime, bootstrap_default_providers, bootstrap_providers_for_packages,
    official_package_for_transport, sync_comm_bus_for_official_packages, TransportAdapterProvider,
};
pub use spanda_providers::package_stubs::*;
pub use spanda_runtime::classification::{
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
pub use spanda_ai::{AiProvider, CompletionRequest, DetectionRequest, EmbedRequest};
