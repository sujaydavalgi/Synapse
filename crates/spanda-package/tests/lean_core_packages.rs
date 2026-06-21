//! Lean-core framework package registry tests.
//!
use spanda_package::adapter::{adapter_metadata_for_package, framework_packages};
use spanda_package::{installed_official_packages, is_official_package};

#[test]
fn official_packages_registered_in_framework_list() {
    let names: Vec<_> = framework_packages().iter().map(|p| p.name).collect();
    for pkg in [
        "spanda-gps",
        "spanda-wifi",
        "spanda-ble",
        "spanda-cellular",
        "spanda-dds",
        "spanda-fleet",
        "spanda-ota",
        "spanda-ledger",
        "spanda-cloud",
    ] {
        assert!(names.contains(&pkg), "missing framework entry for {pkg}");
    }
}

#[test]
fn installed_official_packages_filters_dependencies() {
    let found = installed_official_packages(["spanda-ros2", "my-local-lib", "spanda-gps"]);
    assert_eq!(found, vec!["spanda-gps", "spanda-ros2"]);
    assert!(is_official_package("spanda-mqtt"));
    assert!(!is_official_package("my-local-lib"));
}

#[test]
fn adapter_metadata_for_gps_and_fleet() {
    assert!(adapter_metadata_for_package("spanda-gps").is_some());
    assert!(adapter_metadata_for_package("spanda-fleet").is_some());
    assert!(adapter_metadata_for_package("spanda-ota").is_some());
}
