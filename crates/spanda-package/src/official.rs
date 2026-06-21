//! Resolve installed official lean-core packages from project manifests.
//!
use crate::adapter::framework_packages;
use std::collections::HashSet;

/// Return dependency names that match known official framework packages.
pub fn installed_official_packages<'a>(
    dependency_names: impl IntoIterator<Item = &'a str>,
) -> Vec<&'static str> {
    // Collect installed official package names from a dependency list.
    //
    // Parameters:
    // - `dependency_names` — keys from `spanda.toml` `[dependencies]`
    //
    // Returns:
    // Sorted list of official package names present in the manifest.
    //
    // Options:
    // None.
    //
    // Example:
    // let names = installed_official_packages(["spanda-ros2", "my-local-lib"]);

    let official: HashSet<&str> = framework_packages().iter().map(|p| p.name).collect();
    let mut found: Vec<&str> = dependency_names
        .into_iter()
        .filter_map(|name| official.get(name).copied())
        .collect();
    found.sort_unstable();
    found.dedup();
    found
}

/// Whether a package name is a registered official framework package.
pub fn is_official_package(name: &str) -> bool {
    // Check if a package name is in the official framework catalog.
    //
    // Parameters:
    // - `name` — candidate package name
    //
    // Returns:
    // True when the name appears in `framework_packages()`.
    //
    // Options:
    // None.
    //
    // Example:
    // assert!(is_official_package("spanda-gps"));

    framework_packages().iter().any(|p| p.name == name)
}
