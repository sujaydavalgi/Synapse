//! Bootstrap default provider registrations from core compatibility shims.
//!
use super::package_stubs::{
    CloudPackageStub, ConnectivityPackageStub, FleetPackageStub, GpsPositioningStub,
    LedgerPackageStub, MaintenancePackageStub, NavNavigationStub, SimulationPackageStub,
    SlamPackageStub, VisionPackageStub,
};
use super::transport_adapter::TransportAdapterProvider;
use spanda_comm::TransportKind;
use spanda_runtime::providers::{transport_registry_key, ProviderRegistry, TransportConfig};
use spanda_transport::TransportAdapter;
use spanda_transport_dds::DdsTransportAdapterLive;
use spanda_transport_mqtt::MqttTransportAdapter;
use spanda_transport_ros2::Ros2TransportAdapter;
use spanda_transport_routing::RoutingCommBus;
use spanda_transport_websocket::WebsocketTransportAdapterLive;

fn register_transport_stub(
    registry: &mut ProviderRegistry,
    package: &str,
    adapter: impl TransportAdapter + Send + Sync + 'static,
) {
    registry.register_transport(Box::new(TransportAdapterProvider::new(
        package, "project", adapter,
    )));
}

/// Register built-in transport shims so legacy programs work without installed packages.
pub fn bootstrap_default_providers() -> ProviderRegistry {
    bootstrap_providers_for_packages(&[])
}

/// Build a provider registry from installed official package names.
pub fn bootstrap_providers_for_packages(package_names: &[&str]) -> ProviderRegistry {
    // Build a provider registry from installed official package names.
    //
    // Parameters:
    // - `package_names` — dependency keys from `spanda.toml` / `spanda.lock`
    //
    // Returns:
    // Registry with default shims plus project-scoped transports for official packages.
    //
    // Options:
    // None.
    //
    // Example:

    // let registry = bootstrap_providers_for_packages(&["spanda-ros2"]);

    let mut registry = ProviderRegistry::new();
    registry.set_official_packages(
        package_names
            .iter()
            .map(|name| (*name).to_string())
            .collect(),
    );
    registry.grant_capability("mqtt.publish");
    registry.grant_capability("mqtt.subscribe");
    registry.grant_capability("comm.ros2.publish");
    registry.grant_capability("comm.ros2.subscribe");

    let names: std::collections::HashSet<&str> = package_names.iter().copied().collect();
    let include_all = names.is_empty();

    if include_all || names.contains("spanda-mqtt") {
        register_transport_stub(
            &mut registry,
            "spanda-mqtt",
            MqttTransportAdapter::default(),
        );
    }
    if include_all || names.contains("spanda-ros2") {
        register_transport_stub(
            &mut registry,
            "spanda-ros2",
            Ros2TransportAdapter::default(),
        );
    }
    if names.contains("spanda-dds") {
        registry.grant_capability("dds.publish");
        registry.grant_capability("dds.subscribe");
        register_transport_stub(
            &mut registry,
            "spanda-dds",
            DdsTransportAdapterLive::default(),
        );
    }
    if names.contains("spanda-ble") || names.contains("spanda-wifi") {
        registry.grant_capability("connectivity.wifi");
        registry.grant_capability("connectivity.ble");
        register_transport_stub(
            &mut registry,
            "spanda-ble",
            WebsocketTransportAdapterLive::default(),
        );
    }
    if names.contains("spanda-wifi") {
        registry.register_connectivity(Box::new(ConnectivityPackageStub::wifi()));
    }
    if names.contains("spanda-ble") {
        registry.register_connectivity(Box::new(ConnectivityPackageStub::ble()));
    }
    if names.contains("spanda-gps") {
        registry.grant_capability("positioning.read");
        registry.register_positioning(Box::new(GpsPositioningStub));
    }
    if names.contains("spanda-nav") || names.contains("spanda-nav2") {
        registry.grant_capability("navigation.plan");
        registry.register_navigation(Box::new(NavNavigationStub));
    }
    if names.contains("spanda-slam") {
        registry.grant_capability("slam.localize");
        registry.grant_capability("slam.map");
        registry.register_slam(Box::new(SlamPackageStub));
    }
    if names.contains("spanda-fleet") {
        registry.grant_capability("fleet.orchestrate");
        registry.register_fleet(Box::new(FleetPackageStub));
    }
    if names.contains("spanda-ota") {
        registry.grant_capability("deploy.rollout");
    }
    if names.contains("spanda-ledger") {
        registry.grant_capability("audit.append");
        registry.register_ledger(Box::new(LedgerPackageStub::default()));
    }
    if names.contains("spanda-cloud") {
        registry.grant_capability("cloud.invoke");
        registry.register_cloud(Box::new(CloudPackageStub));
    }
    if names.contains("spanda-maintenance") {
        registry.grant_capability("maintenance.health");
        registry.register_maintenance(Box::new(MaintenancePackageStub));
    }
    if names.contains("spanda-opencv") {
        registry.grant_capability("vision.detect");
        registry.register_vision(Box::new(VisionPackageStub::opencv()));
    }
    if names.contains("spanda-yolo") {
        registry.grant_capability("vision.detect");
        registry.register_vision(Box::new(VisionPackageStub::yolo()));
    }
    if names.contains("spanda-gazebo") {
        registry.grant_capability("simulation.step");
        registry.register_simulation(Box::new(SimulationPackageStub::gazebo()));
    }
    if names.contains("spanda-webots") {
        registry.grant_capability("simulation.step");
        registry.register_simulation(Box::new(SimulationPackageStub::webots()));
    }
    if names.contains("spanda-moveit") {
        registry.grant_capability("manipulation.plan");
    }
    if names.contains("spanda-cellular") {
        registry.grant_capability("connectivity.cellular");
        registry.register_connectivity(Box::new(ConnectivityPackageStub::cellular()));
    }
    if names.contains("spanda-openai") {
        registry.grant_capability("ai.invoke");
    }
    if names.contains("spanda-iot-core") {
        registry.grant_capability("iot.device");
        registry.grant_capability("iot.telemetry");
        registry.grant_capability("iot.command");
        registry.grant_capability("iot.shadow");
    }
    if names.contains("spanda-modbus") {
        registry.grant_capability("iot.modbus");
    }
    if names.contains("spanda-opcua") {
        registry.grant_capability("iot.opcua");
    }
    if names.contains("spanda-zigbee")
        || names.contains("spanda-lora")
        || names.contains("spanda-matter")
        || names.contains("spanda-canbus")
    {
        registry.grant_capability("iot.device");
    }

    registry
}

/// Map a transport kind to the official package that backs it when installed.
pub fn official_package_for_transport(kind: TransportKind) -> Option<&'static str> {
    match kind {
        TransportKind::Ros2 => Some("spanda-ros2"),
        TransportKind::Mqtt => Some("spanda-mqtt"),
        TransportKind::Dds => Some("spanda-dds"),
        TransportKind::Websocket => Some("spanda-ble"),
        TransportKind::Local | TransportKind::Sim => None,
    }
}

fn connect_registry_transport(
    comm_bus: &mut RoutingCommBus,
    registry: &mut ProviderRegistry,
    kind: TransportKind,
    package: &str,
    config: &TransportConfig,
) {
    let key = transport_registry_key(package);
    if registry
        .with_transport(&key, |provider| provider.connect(config))
        .is_some()
    {
        comm_bus.mark_registry_backed(kind, key);
    }
}

/// Connect comm-bus transports through installed official package providers.
pub fn sync_comm_bus_for_official_packages(
    comm_bus: &mut RoutingCommBus,
    registry: &mut ProviderRegistry,
) {
    comm_bus.clear_registry_backed();
    let base = TransportConfig::default();
    let packages: Vec<String> = registry.official_packages().to_vec();
    for name in packages {
        match name.as_str() {
            "spanda-ros2" => {
                connect_registry_transport(comm_bus, registry, TransportKind::Ros2, &name, &base);
            }
            "spanda-mqtt" => {
                connect_registry_transport(
                    comm_bus,
                    registry,
                    TransportKind::Mqtt,
                    &name,
                    &TransportConfig {
                        broker_url: Some("mqtt://localhost:1883".into()),
                        client_id: Some("spanda".into()),
                        ..base.clone()
                    },
                );
            }
            "spanda-dds" => {
                connect_registry_transport(
                    comm_bus,
                    registry,
                    TransportKind::Dds,
                    &name,
                    &TransportConfig {
                        domain_id: Some(0),
                        ..base.clone()
                    },
                );
            }
            "spanda-ble" | "spanda-wifi" => {
                connect_registry_transport(
                    comm_bus,
                    registry,
                    TransportKind::Websocket,
                    "spanda-ble",
                    &TransportConfig {
                        broker_url: Some("ws://localhost:9090".into()),
                        ..base.clone()
                    },
                );
            }
            _ => {}
        }
    }
}
