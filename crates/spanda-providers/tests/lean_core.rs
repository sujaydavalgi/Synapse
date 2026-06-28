//! Integration tests for lean-core provider contracts and registry.
//!
use spanda_providers::{
    bootstrap_default_providers, bootstrap_providers_for_packages, TransportAdapterProvider,
};
use spanda_runtime::classification::{
    module_classifications, official_package_names, ModuleOwnership,
};
use spanda_runtime::providers::TransportConfig;
use spanda_transport_ros2::Ros2TransportAdapter;

#[test]
fn official_package_list_is_non_empty() {
    // Description:
    //     Official package list is non empty.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::lean_core::official_package_list_is_non_empty();

    let names = official_package_names();
    assert!(names.contains(&"spanda-gps"));
    assert!(names.contains(&"spanda-ros2"));
    assert!(names.contains(&"spanda-mqtt"));
}

#[test]
fn module_classifications_include_core_and_shims() {
    // Description:
    //     Module classifications include core and shims.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::lean_core::module_classifications_include_core_and_shims();

    let table = module_classifications();
    assert!(table
        .iter()
        .any(|m| m.module == "providers" && m.ownership == ModuleOwnership::Core));
    assert!(table
        .iter()
        .any(|m| { m.module == "transport" && m.ownership == ModuleOwnership::Deprecated }));
}

#[test]
fn transport_adapter_provider_wraps_legacy_adapter() {
    // Description:
    //     Transport adapter provider wraps legacy adapter.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::lean_core::transport_adapter_provider_wraps_legacy_adapter();

    let mut registry = spanda_runtime::providers::ProviderRegistry::new();
    let adapter =
        TransportAdapterProvider::new("spanda-ros2", "project", Ros2TransportAdapter::default());
    registry.register_transport(Box::new(adapter));

    let ids = registry.list_transports();
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0].package, "spanda-ros2");

    let connected = registry
        .with_transport("spanda-ros2::project", |transport| {
            transport
                .connect(&TransportConfig::default())
                .expect("connect");
            transport.is_connected()
        })
        .expect("registered transport");
    assert!(connected);
}

#[test]
fn bootstrap_registers_default_transports() {
    // Description:
    //     Bootstrap registers default transports.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::lean_core::bootstrap_registers_default_transports();

    let registry = bootstrap_default_providers();
    assert_eq!(registry.transport_count(), 2);
    let ids = registry.list_transports();
    assert!(ids.iter().any(|id| id.package == "spanda-mqtt"));
    assert!(ids.iter().any(|id| id.package == "spanda-ros2"));
}

#[test]
fn bootstrap_providers_for_ros2_only() {
    // Description:
    //     Bootstrap providers for ros2 only.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::lean_core::bootstrap_providers_for_ros2_only();

    let registry = bootstrap_providers_for_packages(&["spanda-ros2"]);
    assert_eq!(registry.transport_count(), 1);
    assert!(registry.has_official_package("spanda-ros2"));
    assert!(!registry.has_official_package("spanda-mqtt"));
}

#[test]
fn bootstrap_registers_fleet_when_installed() {
    // Description:
    //     Bootstrap registers fleet when installed.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::lean_core::bootstrap_registers_fleet_when_installed();

    let registry = bootstrap_providers_for_packages(&["spanda-fleet"]);
    let ids = registry.list_fleet();
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0].package, "spanda-fleet");
}

#[test]
fn bootstrap_registers_positioning_when_gps_installed() {
    // Description:
    //     Bootstrap registers positioning when gps installed.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::lean_core::bootstrap_registers_positioning_when_gps_installed();

    let registry = bootstrap_providers_for_packages(&["spanda-gps"]);
    let ids = registry.list_positioning();
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0].package, "spanda-gps");
}

#[test]
fn bootstrap_registers_connectivity_when_wifi_installed() {
    // Description:
    //     Bootstrap registers connectivity when wifi installed.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::lean_core::bootstrap_registers_connectivity_when_wifi_installed();

    let registry = bootstrap_providers_for_packages(&["spanda-wifi"]);
    assert_eq!(registry.connectivity_count(), 1);
    assert!(registry.has_capability("connectivity.wifi"));
}

#[test]
fn dispatch_gps_read_when_package_installed() {
    // Description:
    //     Dispatch gps read when package installed.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::lean_core::dispatch_gps_read_when_package_installed();

    use spanda_providers::dispatch_official_package_call;
    let mut registry = bootstrap_providers_for_packages(&["spanda-gps"]);
    let value = dispatch_official_package_call(
        &mut registry,
        "positioning.gps",
        "read",
        &[],
        None,
        None,
        0.0,
    )
    .expect("dispatch");
    match value {
        spanda_runtime::value::RuntimeValue::Object { type_name, .. } => {
            assert_eq!(type_name, "GeoPoint");
        }
        other => panic!("expected GeoPoint, got {other:?}"),
    }
}

#[test]
fn dispatch_slam_localize_when_installed() {
    // Description:
    //     Dispatch slam localize when installed.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::lean_core::dispatch_slam_localize_when_installed();

    use spanda_providers::dispatch_official_package_call;
    let mut registry = bootstrap_providers_for_packages(&["spanda-slam"]);
    let value = dispatch_official_package_call(
        &mut registry,
        "navigation.slam",
        "localize",
        &[],
        None,
        None,
        0.0,
    )
    .expect("dispatch");
    match value {
        spanda_runtime::value::RuntimeValue::Object { type_name, .. } => {
            assert_eq!(type_name, "LocalizationEstimate");
        }
        other => panic!("expected LocalizationEstimate, got {other:?}"),
    }
}

#[test]
fn dispatch_skips_when_package_not_installed() {
    // Description:
    //     Dispatch skips when package not installed.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::lean_core::dispatch_skips_when_package_not_installed();

    use spanda_providers::dispatch_official_package_call;
    let mut registry = bootstrap_providers_for_packages(&[]);
    assert!(dispatch_official_package_call(
        &mut registry,
        "positioning.gps",
        "read",
        &[],
        None,
        None,
        0.0,
    )
    .is_none());
}

#[test]
fn ledger_package_append_dispatches() {
    // Description:
    //     Ledger package append dispatches.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::lean_core::ledger_package_append_dispatches();

    use spanda_providers::dispatch_official_package_call;
    let mut registry = bootstrap_providers_for_packages(&["spanda-ledger"]);
    let value = dispatch_official_package_call(
        &mut registry,
        "provenance.ledger",
        "append",
        &[spanda_runtime::value::RuntimeValue::String {
            value: "audit-record".into(),
        }],
        None,
        None,
        0.0,
    )
    .expect("ledger append dispatch");
    match value {
        spanda_runtime::value::RuntimeValue::Number { value, .. } => {
            assert!((value - 1.0).abs() < f64::EPSILON);
        }
        other => panic!("expected ok int, got {other:?}"),
    }
}

#[test]
fn iot_core_package_dispatches() {
    // Description:
    //     Iot core package dispatches.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::lean_core::iot_core_package_dispatches();

    use spanda_providers::dispatch_official_package_call;
    use spanda_providers::hub_stats;
    use spanda_runtime::value::RuntimeValue;

    let mut registry = bootstrap_providers_for_packages(&["spanda-iot-core"]);
    let value = dispatch_official_package_call(
        &mut registry,
        "iot.device",
        "register",
        &[
            RuntimeValue::String {
                value: "sensor-1".into(),
            },
            RuntimeValue::String {
                value: "mqtt".into(),
            },
        ],
        None,
        None,
        0.0,
    )
    .expect("iot device register dispatch");
    match value {
        RuntimeValue::Number { value, .. } => {
            assert!((value - 1.0).abs() < f64::EPSILON);
        }
        other => panic!("expected ok int, got {other:?}"),
    }
    let (devices, _) = hub_stats();
    assert_eq!(devices, 1);
}

#[test]
fn modbus_register_reads_seeded_value() {
    // Description:
    //     Modbus register reads seeded value.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::lean_core::modbus_register_reads_seeded_value();

    use spanda_providers::dispatch_official_package_call;
    use spanda_runtime::value::RuntimeValue;

    let mut registry = bootstrap_providers_for_packages(&["spanda-modbus"]);
    let value = dispatch_official_package_call(
        &mut registry,
        "iot.modbus",
        "read_register",
        &[RuntimeValue::Number {
            value: 40001.0,
            unit: spanda_ast::nodes::UnitKind::None,
        }],
        None,
        None,
        0.0,
    )
    .expect("modbus read dispatch");
    match value {
        RuntimeValue::Number { value, .. } => assert!((value - 42.0).abs() < f64::EPSILON),
        other => panic!("expected number, got {other:?}"),
    }
}

#[test]
fn fusion_package_dispatches_weights() {
    // Description:
    //     Fusion package dispatches weights.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::lean_core::fusion_package_dispatches_weights();

    use spanda_providers::dispatch_official_package_call;
    use spanda_runtime::value::RuntimeValue;

    let mut registry = bootstrap_providers_for_packages(&["spanda-fusion"]);
    let weight = dispatch_official_package_call(
        &mut registry,
        "assurance.fusion",
        "weight_for_sensor",
        &[RuntimeValue::String {
            value: "GPS".into(),
        }],
        None,
        None,
        0.0,
    )
    .expect("fusion weight dispatch");
    match weight {
        RuntimeValue::Number { value, .. } => assert!((value - 0.35).abs() < f64::EPSILON),
        other => panic!("expected number, got {other:?}"),
    }
    let confidence = dispatch_official_package_call(
        &mut registry,
        "assurance.fusion",
        "confidence_for_types",
        &[RuntimeValue::String {
            value: "GPS,Lidar".into(),
        }],
        None,
        None,
        0.0,
    )
    .expect("fusion confidence dispatch");
    match confidence {
        RuntimeValue::Number { value, .. } => assert!(value > 0.5),
        other => panic!("expected number, got {other:?}"),
    }
}

#[test]
fn hri_packages_dispatch_wearable_and_spatial() {
    use spanda_providers::dispatch_official_package_call;
    use spanda_runtime::value::RuntimeValue;

    let mut registry = bootstrap_providers_for_packages(&["spanda-smartwatch", "spanda-hololens"]);
    let telemetry = dispatch_official_package_call(
        &mut registry,
        "wearable.smartwatch",
        "read_telemetry",
        &[RuntimeValue::String {
            value: "watch-001".into(),
        }],
        None,
        None,
        0.0,
    )
    .expect("wearable telemetry dispatch");
    match telemetry {
        RuntimeValue::Object { type_name, .. } => assert_eq!(type_name, "WearableTelemetry"),
        other => panic!("expected object, got {other:?}"),
    }
    let session = dispatch_official_package_call(
        &mut registry,
        "spatial.hololens",
        "start_session",
        &[RuntimeValue::String {
            value: "hololens-001".into(),
        }],
        None,
        None,
        0.0,
    )
    .expect("spatial session dispatch");
    match session {
        RuntimeValue::Number { value, .. } => assert!((value - 1.0).abs() < f64::EPSILON),
        other => panic!("expected number, got {other:?}"),
    }
}

#[test]
fn h3_packages_dispatch_voice_and_overlay() {
    use spanda_providers::dispatch_official_package_call;
    use spanda_runtime::value::RuntimeValue;

    let mut registry = bootstrap_providers_for_packages(&["spanda-voice", "spanda-hololens"]);
    let events = dispatch_official_package_call(
        &mut registry,
        "hri.voice",
        "poll_events",
        &[],
        None,
        None,
        0.0,
    )
    .expect("voice poll dispatch");
    match events {
        RuntimeValue::Number { value, .. } => assert!((value - 1.0).abs() < f64::EPSILON),
        other => panic!("expected number, got {other:?}"),
    }
    let overlay = dispatch_official_package_call(
        &mut registry,
        "spatial.hololens",
        "subscribe_overlay",
        &[
            RuntimeValue::String {
                value: "annotation".into(),
            },
            RuntimeValue::String {
                value: "hololens-tech-001".into(),
            },
        ],
        None,
        None,
        0.0,
    )
    .expect("overlay dispatch");
    match overlay {
        RuntimeValue::Number { value, .. } => assert!((value - 1.0).abs() < f64::EPSILON),
        other => panic!("expected number, got {other:?}"),
    }
}
