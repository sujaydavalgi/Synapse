//! Root `spanda.toml` manifest and config file reference types.
//!
use crate::error::{ConfigError, ConfigResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const MANIFEST_FILENAME: &str = "spanda.toml";

/// Root Spanda system manifest parsed from `spanda.toml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SpandaManifest {
    #[serde(default)]
    pub project: Option<ProjectSection>,
    #[serde(default)]
    pub config: ConfigReferences,
    #[serde(default)]
    pub extends: ExtendsSection,
    #[serde(default)]
    pub merge: HashMap<String, MergeStrategyHint>,
}

/// Project metadata for autonomous system configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectSection {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub language: Option<String>,
}

/// Paths to domain-specific configuration fragments referenced by the root manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ConfigReferences {
    #[serde(default)]
    pub hardware: Option<String>,
    #[serde(default)]
    pub devices: Option<String>,
    #[serde(default)]
    pub facilities: Option<String>,
    #[serde(default)]
    pub network_devices: Option<String>,
    #[serde(default)]
    pub providers: Option<String>,
    #[serde(default)]
    pub fleet: Option<String>,
    #[serde(default)]
    pub security: Option<String>,
    #[serde(default)]
    pub health: Option<String>,
    #[serde(default)]
    pub readiness: Option<String>,
    #[serde(default)]
    pub assurance: Option<String>,
    #[serde(default)]
    pub recovery: Option<String>,
    #[serde(default)]
    pub mission: Option<String>,
}

/// Cascading configuration layer references.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ExtendsSection {
    #[serde(default)]
    pub base: Option<String>,
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(default)]
    pub deployment: Option<String>,
    #[serde(default)]
    pub robot: Option<String>,
}

/// Per-section array merge strategy hint from TOML `[merge]` table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MergeStrategyHint {
    #[default]
    Replace,
    Append,
    MergeById,
}

impl SpandaManifest {
    pub fn parse_str(content: &str) -> ConfigResult<Self> {
        // Parse raw TOML text into a Spanda manifest struct.
        //
        // Parameters:
        // - `content` — raw `spanda.toml` source
        //
        // Returns:
        // Parsed manifest, or a parse error.
        //
        // Options:
        // None.
        //
        // Example:
        // let manifest = SpandaManifest::parse_str(toml_text)?;

        toml::from_str(content).map_err(|source| ConfigError::TomlParse {
            path: PathBuf::from(MANIFEST_FILENAME),
            source,
        })
    }

    pub fn load(path: &Path) -> ConfigResult<Self> {
        // Load and parse a manifest from disk.
        //
        // Parameters:
        // - `path` — absolute or relative path to `spanda.toml`
        //
        // Returns:
        // Parsed manifest, or I/O / parse error.
        //
        // Options:
        // None.
        //
        // Example:
        // let manifest = SpandaManifest::load(project_root.join("spanda.toml"))?;

        let content = std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let mut manifest = Self::parse_str(&content)?;
        if manifest.project.is_none() {
            if let Ok(package) = spanda_package::PackageManifest::parse_str(&content) {
                manifest.project = Some(ProjectSection {
                    name: package.package.name,
                    version: package.package.version,
                    language: None,
                });
            }
        }
        Ok(manifest)
    }

    pub fn load_from_dir(dir: &Path) -> ConfigResult<Self> {
        // Load `spanda.toml` from a project directory.
        //
        // Parameters:
        // - `dir` — project root directory
        //
        // Returns:
        // Parsed manifest, or not-found / parse error.
        //
        // Options:
        // None.
        //
        // Example:
        // let manifest = SpandaManifest::load_from_dir(&project_root)?;

        let path = dir.join(MANIFEST_FILENAME);
        if !path.exists() {
            return Err(ConfigError::ManifestNotFound { path });
        }
        Self::load(&path)
    }

    pub fn find_project_root(start: &Path) -> Option<PathBuf> {
        // Walk parent directories until a `spanda.toml` is found.
        //
        // Parameters:
        // - `start` — file or directory to begin the search from
        //
        // Returns:
        // Project root path when a manifest exists, otherwise `None`.
        //
        // Options:
        // None.
        //
        // Example:
        // let root = SpandaManifest::find_project_root(&cwd)?;

        spanda_package::find_project_root(start)
    }

    pub fn layer_paths(&self) -> Vec<(&'static str, &str)> {
        // Collect non-empty `[extends]` layer paths in merge order.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Ordered `(layer_name, relative_path)` pairs.
        //
        // Options:
        // None.
        //
        // Example:
        // let layers = manifest.layer_paths();

        let mut layers = Vec::new();
        if let Some(ref p) = self.extends.base {
            layers.push(("base", p.as_str()));
        }
        if let Some(ref p) = self.extends.environment {
            layers.push(("environment", p.as_str()));
        }
        if let Some(ref p) = self.extends.deployment {
            layers.push(("deployment", p.as_str()));
        }
        if let Some(ref p) = self.extends.robot {
            layers.push(("robot", p.as_str()));
        }
        layers
    }

    pub fn fragment_paths(&self) -> Vec<(&'static str, &str)> {
        // Collect non-empty `[config]` fragment paths.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // `(fragment_name, relative_path)` pairs for referenced domain files.
        //
        // Options:
        // None.
        //
        // Example:
        // let fragments = manifest.fragment_paths();

        let refs = &self.config;
        let mut out = Vec::new();
        macro_rules! push_opt {
            ($name:expr, $field:expr) => {
                if let Some(ref p) = $field {
                    out.push(($name, p.as_str()));
                }
            };
        }
        push_opt!("hardware", refs.hardware);
        push_opt!("devices", refs.devices);
        push_opt!("facilities", refs.facilities);
        push_opt!("network_devices", refs.network_devices);
        push_opt!("providers", refs.providers);
        push_opt!("fleet", refs.fleet);
        push_opt!("security", refs.security);
        push_opt!("health", refs.health);
        push_opt!("readiness", refs.readiness);
        push_opt!("assurance", refs.assurance);
        push_opt!("recovery", refs.recovery);
        push_opt!("mission", refs.mission);
        out
    }
}
