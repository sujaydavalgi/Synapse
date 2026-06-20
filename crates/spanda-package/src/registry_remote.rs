//! Optional remote registry mirror via `SPANDA_REGISTRY_URL`.
//!
//! Fetches `index.json` from the configured base URL (curl when available).
//! Entries merge with `LOCAL_REGISTRY` for search and dependency resolution.

use crate::category::PackageCategory;
use crate::registry::RegistryEntry;
use crate::safety::SafetyLevel;
use serde::{Deserialize, Serialize};
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
    std::env::var("SPANDA_REGISTRY_URL")
        .ok()
        .map(|url| url.trim_end_matches('/').to_string())
        .filter(|url| !url.is_empty())
}

pub fn fetch_index_json(url: &str) -> Result<String, String> {
    if let Ok(output) = std::process::Command::new("curl")
        .args(["-fsSL", url])
        .output()
    {
        if output.status.success() {
            return String::from_utf8(output.stdout)
                .map_err(|e| format!("registry response is not UTF-8: {e}"));
        }
    }
    Err(format!("failed to fetch registry index from {url}"))
}

pub fn load_remote_registry() -> Vec<RemoteRegistryEntry> {
    REMOTE_CACHE
        .get_or_init(|| {
            let Some(base) = registry_base_url() else {
                return Vec::new();
            };
            let url = format!("{base}/index.json");
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
    load_remote_registry()
        .into_iter()
        .find(|entry| entry.name == name)
}

pub fn search_remote_registry(query: &str) -> Vec<RemoteRegistryEntry> {
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
    name.parse().unwrap_or(PackageCategory::Robotics)
}

pub fn remote_safety_level(name: &str) -> SafetyLevel {
    match name {
        "spanda-ros2" | "spanda-opencv" | "spanda-yolo" | "spanda-mqtt" => {
            SafetyLevel::SimulationOnly
        }
        "spanda-python-bridge" | "spanda-cpp-bridge" => SafetyLevel::HardwareSafe,
        _ => SafetyLevel::Experimental,
    }
}

pub fn remote_as_static_view(entry: &RemoteRegistryEntry) -> RegistryEntryView<'_> {
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
        RegistryEntryView {
            name: entry.name,
            description: entry.description,
            versions: entry.versions.iter().copied().collect(),
            category: entry.category,
            license: entry.license,
            import_paths: entry.import_paths.iter().copied().collect(),
            safety_level: entry.safety_level(),
        }
    }
}

pub fn lookup_registry_entry(name: &str) -> Option<RegistryEntryLookup> {
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
        match self {
            RegistryEntryLookup::Local(entry) => entry.name,
            RegistryEntryLookup::Remote(entry) => &entry.name,
        }
    }

    pub fn versions(&self) -> Vec<String> {
        match self {
            RegistryEntryLookup::Local(entry) => entry.versions.iter().map(|v| (*v).to_string()).collect(),
            RegistryEntryLookup::Remote(entry) => entry.versions.clone(),
        }
    }

    pub fn registry_label(&self) -> &'static str {
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
        std::env::remove_var("SPANDA_REGISTRY_URL");
        assert!(load_remote_registry().is_empty());
    }
}
