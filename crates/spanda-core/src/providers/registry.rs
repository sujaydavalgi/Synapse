//! Runtime registry for optional domain provider implementations.
//!
use super::traits::{
    ActuatorProvider, CloudProvider, ConnectivityProvider, CryptoProvider, FleetProvider,
    HalProvider, LedgerProvider, MaintenanceProvider, NavigationProvider, PositioningProvider,
    RosProvider, SensorProvider, SimulationProvider, SlamProvider, TransportProvider,
    VisionProvider,
};
use super::types::{ProviderCapabilitySet, ProviderId};
use crate::comm::TransportKind;
use std::collections::HashMap;

/// Holds installed provider implementations keyed by package and name.
#[derive(Default)]
pub struct ProviderRegistry {
    sensors: HashMap<String, Box<dyn SensorProvider>>,
    actuators: HashMap<String, Box<dyn ActuatorProvider>>,
    connectivity: HashMap<String, Box<dyn ConnectivityProvider>>,
    positioning: HashMap<String, Box<dyn PositioningProvider>>,
    transports: HashMap<String, Box<dyn TransportProvider>>,
    crypto: HashMap<String, Box<dyn CryptoProvider>>,
    navigation: HashMap<String, Box<dyn NavigationProvider>>,
    slam: HashMap<String, Box<dyn SlamProvider>>,
    vision: HashMap<String, Box<dyn VisionProvider>>,
    fleet: HashMap<String, Box<dyn FleetProvider>>,
    simulation: HashMap<String, Box<dyn SimulationProvider>>,
    maintenance: HashMap<String, Box<dyn MaintenanceProvider>>,
    ledger: HashMap<String, Box<dyn LedgerProvider>>,
    cloud: HashMap<String, Box<dyn CloudProvider>>,
    ros: HashMap<String, Box<dyn RosProvider>>,
    hal: HashMap<String, Box<dyn HalProvider>>,
    granted_capabilities: ProviderCapabilitySet,
    official_packages: Vec<String>,
}

fn registry_key(id: &ProviderId) -> String {
    format!("{}::{}", id.package, id.name)
}

/// Stable registry lookup key for a project-scoped transport provider.
pub fn transport_registry_key(package: &str) -> String {
    format!("{package}::project")
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn grant_capability(&mut self, cap: impl Into<String>) {
        self.granted_capabilities.insert(cap);
    }

    pub fn has_capability(&self, cap: &str) -> bool {
        self.granted_capabilities.contains(cap)
    }

    pub fn set_official_packages(&mut self, names: Vec<String>) {
        self.official_packages = names;
    }

    pub fn official_packages(&self) -> &[String] {
        &self.official_packages
    }

    pub fn has_official_package(&self, name: &str) -> bool {
        self.official_packages.iter().any(|pkg| pkg == name)
    }

    pub fn register_sensor(&mut self, provider: Box<dyn SensorProvider>) {
        let key = registry_key(&provider.metadata().id);
        self.sensors.insert(key, provider);
    }

    pub fn register_actuator(&mut self, provider: Box<dyn ActuatorProvider>) {
        let key = registry_key(&provider.metadata().id);
        self.actuators.insert(key, provider);
    }

    pub fn register_connectivity(&mut self, provider: Box<dyn ConnectivityProvider>) {
        let key = registry_key(&provider.metadata().id);
        self.connectivity.insert(key, provider);
    }

    pub fn register_positioning(&mut self, provider: Box<dyn PositioningProvider>) {
        let key = registry_key(&provider.metadata().id);
        self.positioning.insert(key, provider);
    }

    pub fn register_transport(&mut self, provider: Box<dyn TransportProvider>) {
        let key = registry_key(&provider.metadata().id);
        self.transports.insert(key, provider);
    }

    pub fn register_crypto(&mut self, provider: Box<dyn CryptoProvider>) {
        let key = registry_key(&provider.metadata().id);
        self.crypto.insert(key, provider);
    }

    pub fn register_navigation(&mut self, provider: Box<dyn NavigationProvider>) {
        let key = registry_key(&provider.metadata().id);
        self.navigation.insert(key, provider);
    }

    pub fn register_slam(&mut self, provider: Box<dyn SlamProvider>) {
        let key = registry_key(&provider.metadata().id);
        self.slam.insert(key, provider);
    }

    pub fn register_vision(&mut self, provider: Box<dyn VisionProvider>) {
        let key = registry_key(&provider.metadata().id);
        self.vision.insert(key, provider);
    }

    pub fn register_fleet(&mut self, provider: Box<dyn FleetProvider>) {
        let key = registry_key(&provider.metadata().id);
        self.fleet.insert(key, provider);
    }

    pub fn register_simulation(&mut self, provider: Box<dyn SimulationProvider>) {
        let key = registry_key(&provider.metadata().id);
        self.simulation.insert(key, provider);
    }

    pub fn register_maintenance(&mut self, provider: Box<dyn MaintenanceProvider>) {
        let key = registry_key(&provider.metadata().id);
        self.maintenance.insert(key, provider);
    }

    pub fn register_ledger(&mut self, provider: Box<dyn LedgerProvider>) {
        let key = registry_key(&provider.metadata().id);
        self.ledger.insert(key, provider);
    }

    pub fn register_cloud(&mut self, provider: Box<dyn CloudProvider>) {
        let key = registry_key(&provider.metadata().id);
        self.cloud.insert(key, provider);
    }

    pub fn register_ros(&mut self, provider: Box<dyn RosProvider>) {
        let key = registry_key(&provider.metadata().id);
        self.ros.insert(key, provider);
    }

    pub fn register_hal(&mut self, provider: Box<dyn HalProvider>) {
        let key = registry_key(&provider.metadata().id);
        self.hal.insert(key, provider);
    }

    pub fn list_transports(&self) -> Vec<ProviderId> {
        self.transports
            .values()
            .map(|p| p.metadata().id.clone())
            .collect()
    }

    pub fn list_positioning(&self) -> Vec<ProviderId> {
        self.positioning
            .values()
            .map(|p| p.metadata().id.clone())
            .collect()
    }

    pub fn list_fleet(&self) -> Vec<ProviderId> {
        self.fleet
            .values()
            .map(|p| p.metadata().id.clone())
            .collect()
    }

    pub fn transport_count(&self) -> usize {
        self.transports.len()
    }

    pub fn with_transport<F, R>(&mut self, key: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn TransportProvider) -> R,
    {
        self.transports.get_mut(key).map(|p| f(p.as_mut()))
    }

    pub fn with_transport_for_kind<F, R>(&mut self, kind: TransportKind, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn TransportProvider) -> R,
    {
        let key = self
            .transports
            .iter()
            .find(|(_, provider)| provider.kind() == kind)
            .map(|(key, _)| key.clone())?;
        self.with_transport(&key, f)
    }
}
