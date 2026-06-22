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
    let names = official_package_names();
    assert!(names.contains(&"spanda-gps"));
    assert!(names.contains(&"spanda-ros2"));
    assert!(names.contains(&"spanda-mqtt"));
}

#[test]
fn module_classifications_include_core_and_shims() {
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
    let registry = bootstrap_default_providers();
    assert_eq!(registry.transport_count(), 2);
    let ids = registry.list_transports();
    assert!(ids.iter().any(|id| id.package == "spanda-mqtt"));
    assert!(ids.iter().any(|id| id.package == "spanda-ros2"));
}

#[test]
fn bootstrap_providers_for_ros2_only() {
    let registry = bootstrap_providers_for_packages(&["spanda-ros2"]);
    assert_eq!(registry.transport_count(), 1);
    assert!(registry.has_official_package("spanda-ros2"));
    assert!(!registry.has_official_package("spanda-mqtt"));
}

#[test]
fn bootstrap_registers_fleet_when_installed() {
    let registry = bootstrap_providers_for_packages(&["spanda-fleet"]);
    let ids = registry.list_fleet();
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0].package, "spanda-fleet");
}

#[test]
fn bootstrap_registers_positioning_when_gps_installed() {
    let registry = bootstrap_providers_for_packages(&["spanda-gps"]);
    let ids = registry.list_positioning();
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0].package, "spanda-gps");
}

#[test]
fn bootstrap_registers_connectivity_when_wifi_installed() {
    let registry = bootstrap_providers_for_packages(&["spanda-wifi"]);
    assert_eq!(registry.connectivity_count(), 1);
    assert!(registry.has_capability("connectivity.wifi"));
}

#[test]
fn dispatch_gps_read_when_package_installed() {
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
