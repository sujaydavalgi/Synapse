//! Bootstrap default provider registrations from core compatibility shims.
//!
use super::package_stubs::{
    CloudPackageStub, ConnectivityPackageStub, FleetPackageStub, GpsPositioningStub,
    HriInputPackageStub, LedgerPackageStub, MaintenancePackageStub, NavNavigationStub,
    OverlayPackageStub, SimulationPackageStub, SlamPackageStub, SpatialSessionPackageStub,
    VisionPackageStub, WearablePackageStub,
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
    // Description:
    //     Bootstrap default providers.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: ProviderRegistry
    //         Return value from `bootstrap_default_providers`.
    //
    // Example:

    //     let result = spanda_providers::bootstrap::bootstrap_default_providers();

    bootstrap_providers_for_packages(&[])
}

/// Build a provider registry from installed official package names.
pub fn bootstrap_providers_for_packages(package_names: &[&str]) -> ProviderRegistry {
    // Description:
    //     Bootstrap providers for packages.
    //
    // Inputs:
    //     package_names: &[&str]
    //         Caller-supplied package names.
    //
    // Outputs:
    //     result: ProviderRegistry
    //         Return value from `bootstrap_providers_for_packages`.
    //
    // Example:
    //     let result = spanda_providers::bootstrap::bootstrap_providers_for_packages(package_names);

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
        crate::iot_hub::seed_modbus_demo_register(40001, 42.0);
    }
    if names.contains("spanda-opcua") {
        registry.grant_capability("iot.opcua");
        crate::iot_hub::seed_protocol_demos();
    }
    if names.contains("spanda-zigbee") {
        registry.grant_capability("iot.zigbee");
        crate::iot_hub::seed_protocol_demos();
    }
    if names.contains("spanda-lora") {
        registry.grant_capability("iot.lora");
        crate::iot_hub::seed_protocol_demos();
    }
    if names.contains("spanda-matter") {
        registry.grant_capability("iot.matter");
        crate::iot_hub::seed_protocol_demos();
    }
    if names.contains("spanda-canbus") {
        registry.grant_capability("iot.canbus");
        crate::iot_hub::seed_protocol_demos();
    }
    if names.contains("spanda-radar") {
        registry.grant_capability("sensors.radar.read");
    }
    if names.contains("spanda-lidar") {
        registry.grant_capability("sensors.lidar.read");
    }
    if names.contains("spanda-ultrasonic") {
        registry.grant_capability("sensors.ultrasonic.read");
    }
    if names.contains("spanda-automotive-ethernet") {
        registry.grant_capability("automotive.ethernet.connect");
    }
    if names.contains("spanda-lin") {
        registry.grant_capability("automotive.lin.read");
    }
    if names.contains("spanda-uds") {
        registry.grant_capability("automotive.uds.diagnose");
    }
    if names.contains("spanda-v2x") {
        registry.grant_capability("automotive.v2x.receive");
    }
    if names.contains("spanda-radar")
        || names.contains("spanda-lidar")
        || names.contains("spanda-ultrasonic")
    {
        crate::automotive_hub::seed_automotive_demos();
    }
    if include_all || names.contains("spanda-anomaly") {
        registry.grant_capability("assurance.anomaly.scan");
    }
    if include_all || names.contains("spanda-fusion") {
        registry.grant_capability("assurance.fusion.weight");
    }
    for package in [
        "spanda-smartwatch",
        "spanda-industrial-wearables",
        "spanda-bodycam",
    ] {
        if names.contains(package) {
            registry.grant_capability("wearable.telemetry");
            registry.register_wearable_telemetry(Box::new(WearablePackageStub::new(package)));
        }
    }
    for package in [
        "spanda-hololens",
        "spanda-arkit",
        "spanda-arcore",
        "spanda-vision-pro",
        "spanda-magic-leap",
        "spanda-openxr",
    ] {
        if names.contains(package) {
            registry.grant_capability("spatial.session");
            registry.register_spatial_session(Box::new(SpatialSessionPackageStub::new(package)));
        }
    }
    if names.contains("spanda-hololens") {
        registry.grant_capability("hri.overlay");
        registry.register_overlay(Box::new(OverlayPackageStub::new("spanda-hololens")));
    }
    for package in ["spanda-voice", "spanda-gesture", "spanda-eye-tracking"] {
        if names.contains(package) {
            registry.grant_capability("hri.input");
            registry.register_hri_input(Box::new(HriInputPackageStub::new(package)));
        }
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
    // Description:
    //     Official package for transport.
    //
    // Inputs:
    //     kind: TransportKind
    //         Caller-supplied kind.
    //
    // Outputs:
    //     result: Option<&'static str>
    //         Return value from `official_package_for_transport`.
    //
    // Example:

    //     let result = spanda_providers::bootstrap::official_package_for_transport(kind);

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
    // Description:
    //     Connect registry transport.
    //
    // Inputs:
    //     comm_bus: &mut RoutingCommBus
    //         Caller-supplied comm bus.
    //     registry: &mut ProviderRegistry
    //         Caller-supplied registry.
    //     kind: TransportKind
    //         Caller-supplied kind.
    //     package: &str
    //         Caller-supplied package.
    //     config: &TransportConfig
    //         Caller-supplied config.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::bootstrap::connect_registry_transport(comm_bus, registry, kind, package, config);

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
    // Description:
    //     Sync comm bus for official packages.
    //
    // Inputs:
    //     comm_bus: &mut RoutingCommBus
    //         Caller-supplied comm bus.
    //     registry: &mut ProviderRegistry
    //         Caller-supplied registry.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::bootstrap::sync_comm_bus_for_official_packages(comm_bus, registry);

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
