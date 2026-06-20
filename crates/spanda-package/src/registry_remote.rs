//! Optional remote registry mirror via `SPANDA_REGISTRY_URL`.
//!
//! Fetches `index.json` from the configured base URL (curl when available).
//! Entries merge with `LOCAL_REGISTRY` for search and dependency resolution.

use crate::category::PackageCategory;
use crate::registry::RegistryEntry;
use crate::safety::SafetyLevel;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RemoteRegistryEntry {
    pub name: String,
    pub description: String,
    pub versions: Vec<String>,
    pub category: String,
    pub license: String,
    #[serde(default)]
    pub import_paths: Vec<String>,
}

static REMOTE_CACHE: OnceLock<Vec<RemoteRegistryEntry>> = OnceLock::new();

pub fn registry_base_url() -> Option<String> {
    // Registry base url.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::registry_remote::registry_base_url();

    // Produce var as the result.
    std::env::var("SPANDA_REGISTRY_URL")
        .ok()
        .map(|url| url.trim_end_matches('/').to_string())
        .filter(|url| !url.is_empty())
}

pub fn fetch_index_json(url: &str) -> Result<String, String> {
    // Fetch index json.
    //
    // Parameters:
    // - `url` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::registry_remote::fetch_index_json(url);

    // use path when file url path is present.

    // Emit output when file url path provides a path.
    if let Some(path) = super::registry_fetch::file_url_path(url) {
        return fs::read_to_string(&path)
            .map_err(|e| format!("failed to read registry index at {}: {e}", path.display()));
    }

    // Handle the success value from new.
    if let Ok(output) = std::process::Command::new("curl")
        .args(["-fsSL", url])
        .output()
    {
        // Handle output when the subprocess succeeds.
        if output.status.success() {
            return String::from_utf8(output.stdout)
                .map_err(|e| format!("registry response is not UTF-8: {e}"));
        }
    }
    Err(format!("failed to fetch registry index from {url}"))
}

pub fn load_remote_registry() -> Vec<RemoteRegistryEntry> {
    // Load remote registry.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Vec<RemoteRegistryEntry>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::registry_remote::load_remote_registry();

    // Produce REMOTE CACHE as the result.
    REMOTE_CACHE
        .get_or_init(|| {
            let Some(base) = registry_base_url() else {
                return Vec::new();
            };
            let url = format!("{base}/index.json");

            // Match on fetch index json and handle each case.
            match fetch_index_json(&url) {
                Ok(body) => serde_json::from_str(&body).unwrap_or_else(|e| {
                    eprintln!("Warning: invalid remote registry JSON at {url}: {e}");
                    Vec::new()
                }),
                Err(err) => {
                    eprintln!("Warning: {err}");
                    Vec::new()
                }
            }
        })
        .clone()
}

pub fn find_remote_entry(name: &str) -> Option<RemoteRegistryEntry> {
    // Find remote entry.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::registry_remote::find_remote_entry(name);

    // Produce load remote registry as the result.
    load_remote_registry()
        .into_iter()
        .find(|entry| entry.name == name)
}

pub fn search_remote_registry(query: &str) -> Vec<RemoteRegistryEntry> {
    // Search remote registry.
    //
    // Parameters:
    // - `query` — input value
    //
    // Returns:
    // Vec<RemoteRegistryEntry>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::registry_remote::search_remote_registry(query);

    // Compute q for the following logic.
    let q = query.to_lowercase();
    load_remote_registry()
        .into_iter()
        .filter(|entry| {
            entry.name.to_lowercase().contains(&q)
                || entry.description.to_lowercase().contains(&q)
                || entry.category.to_lowercase().contains(&q)
        })
        .collect()
}

pub fn remote_category(name: &str) -> PackageCategory {
    // Remote category.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // PackageCategory.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::registry_remote::remote_category(name);

    // Produce Robotics) as the result.
    name.parse().unwrap_or(PackageCategory::Robotics)
}

pub fn remote_safety_level(name: &str) -> SafetyLevel {
    // Remote safety level.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // SafetyLevel.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::registry_remote::remote_safety_level(name);

    // Match on name and handle each case.
    match name {
        "spanda-ros2" | "spanda-opencv" | "spanda-yolo" | "spanda-mqtt" => {
            SafetyLevel::SimulationOnly
        }
        "spanda-python-bridge" | "spanda-cpp-bridge" => SafetyLevel::HardwareSafe,
        _ => SafetyLevel::Experimental,
    }
}

pub fn remote_as_static_view(entry: &RemoteRegistryEntry) -> RegistryEntryView<'_> {
    // Remote as static view.
    //
    // Parameters:
    // - `entry` — input value
    //
    // Returns:
    // RegistryEntryView<'_>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::registry_remote::remote_as_static_view(entry);

    // Produce RegistryEntryView as the result.
    RegistryEntryView {
        name: entry.name.as_str(),
        description: entry.description.as_str(),
        versions: entry.versions.iter().map(String::as_str).collect(),
        category: remote_category(&entry.category),
        license: entry.license.as_str(),
        import_paths: entry.import_paths.iter().map(String::as_str).collect(),
        safety_level: remote_safety_level(&entry.name),
    }
}

/// Unified view for local static entries and remote owned entries.
#[derive(Debug, Clone)]
pub struct RegistryEntryView<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub versions: Vec<&'a str>,
    pub category: PackageCategory,
    pub license: &'a str,
    pub import_paths: Vec<&'a str>,
    pub safety_level: SafetyLevel,
}

impl<'a> From<&'static RegistryEntry> for RegistryEntryView<'a> {
    fn from(entry: &'static RegistryEntry) -> Self {
        // From.
        //
        // Parameters:
        // - `entry` — input value
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_package::registry_remote::from(entry);

        RegistryEntryView {
            name: entry.name,
            description: entry.description,
            versions: entry.versions.to_vec(),
            category: entry.category,
            license: entry.license,
            import_paths: entry.import_paths.to_vec(),
            safety_level: entry.safety_level(),
        }
    }
}

pub fn lookup_registry_entry(name: &str) -> Option<RegistryEntryLookup> {
    // Lookup registry entry.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::registry_remote::lookup_registry_entry(name);

    // use entry when find registry entry is present.

    // Emit output when find registry entry provides a entry.
    if let Some(entry) = super::registry::find_registry_entry(name) {
        return Some(RegistryEntryLookup::Local(entry));
    }
    find_remote_entry(name).map(RegistryEntryLookup::Remote)
}

#[derive(Debug, Clone)]
pub enum RegistryEntryLookup {
    Local(&'static RegistryEntry),
    Remote(RemoteRegistryEntry),
}

impl RegistryEntryLookup {
    pub fn name(&self) -> &str {
        // Name.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Text result.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.name();

        // Dispatch based on the enum variant or current state.
        match self {
            RegistryEntryLookup::Local(entry) => entry.name,
            RegistryEntryLookup::Remote(entry) => &entry.name,
        }
    }

    pub fn versions(&self) -> Vec<String> {
        // Versions.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Vec<String>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.versions();

        // Dispatch based on the enum variant or current state.
        match self {
            RegistryEntryLookup::Local(entry) => {
                entry.versions.iter().map(|v| (*v).to_string()).collect()
            }
            RegistryEntryLookup::Remote(entry) => entry.versions.clone(),
        }
    }

    pub fn registry_label(&self) -> &'static str {
        // Registry label.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Text result.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.registry_label();

        // Dispatch based on the enum variant or current state.
        match self {
            RegistryEntryLookup::Local(_) => "local",
            RegistryEntryLookup::Remote(_) => "remote",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_disabled_without_env() {
        let _guard = crate::testing::env_lock();
        // Remote disabled without env.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_package::registry_remote::remote_disabled_without_env();

        std::env::remove_var("SPANDA_REGISTRY_URL");
        assert!(load_remote_registry().is_empty());
    }
}
