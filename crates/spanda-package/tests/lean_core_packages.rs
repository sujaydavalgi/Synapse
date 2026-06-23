//! Lean-core framework package registry tests.
//!
use spanda_package::adapter::{adapter_metadata_for_package, framework_packages};
use spanda_package::{
    installed_official_packages, is_official_package, load_official_packages_for_project,
};
use std::path::Path;

#[test]
fn official_packages_registered_in_framework_list() {
    let names: Vec<_> = framework_packages().iter().map(|p| p.name).collect();
    let registry_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/registry");
    let hosted: Vec<String> = std::fs::read_dir(&registry_root)
        .expect("packages/registry")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().join("spanda.toml").is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    for pkg in hosted {
        assert!(
            names.contains(&pkg.as_str()),
            "missing framework entry for {pkg}"
        );
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
fn load_official_packages_for_ros2_project() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/packages/ros2_adapter_package");
    let packages = load_official_packages_for_project(&root).expect("manifest");
    assert!(packages.contains(&"spanda-ros2".to_string()));
}

#[test]
fn adapter_metadata_for_gps_and_fleet() {
    assert!(adapter_metadata_for_package("spanda-gps").is_some());
    assert!(adapter_metadata_for_package("spanda-fleet").is_some());
    assert!(adapter_metadata_for_package("spanda-ota").is_some());
}
