//! Lean-core provider contracts and registry for optional domain packages.
//!
pub mod registry;
pub mod traits;
pub mod transport_types;
pub mod types;

pub use registry::{transport_registry_key, ProviderRegistry};
pub use traits::{
    ActuatorProvider, CloudProvider, ConnectivityProvider, CryptoProvider, FleetProvider,
    HalProvider, LedgerProvider, MaintenanceProvider, NavigationProvider, PositioningProvider,
    RosProvider, SensorProvider, SimulationProvider, SlamProvider, TransportProvider,
    VisionProvider,
};
pub use transport_types::{AdapterMessage, TransportConfig};
pub use types::{
    ProviderCapability, ProviderCapabilitySet, ProviderError, ProviderId, ProviderMetadata,
    ProviderResult, ProviderSafetyLevel,
};
