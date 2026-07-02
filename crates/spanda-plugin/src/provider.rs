//! Provider-type plugin discovery and registry metadata.

use crate::error::PluginResult;
use crate::manifest::PluginManifest;
use crate::runtime::{PluginManager, PluginState};
use crate::types::PluginType;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderPluginRecord {
    pub name: String,
    pub version: String,
    pub install_path: String,
}

pub fn list_enabled_provider_plugins(project_root: &Path) -> PluginResult<Vec<ProviderPluginRecord>> {
    let manager = PluginManager::open(project_root, env!("CARGO_PKG_VERSION"))?;
    Ok(manager
        .store()
        .list()
        .into_iter()
        .filter(|p| {
            p.state == PluginState::Enabled && p.plugin_type == PluginType::Provider.as_str()
        })
        .map(|p| ProviderPluginRecord {
            name: p.name.clone(),
            version: p.version.clone(),
            install_path: p.install_path.display().to_string(),
        })
        .collect())
}

pub fn provider_plugin_manifest(project_root: &Path, name: &str) -> PluginResult<PluginManifest> {
    let manager = PluginManager::open(project_root, env!("CARGO_PKG_VERSION"))?;
    let record = manager.store().get(name).ok_or_else(|| {
        crate::error::PluginError::Runtime(format!("plugin not installed: {name}"))
    })?;
    PluginManifest::load_from_dir(&record.install_path)
}
