//! Plugin error types and result alias.

use thiserror::Error;

/// Errors raised while parsing manifests, loading plugins, or enforcing security.
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("manifest error: {0}")]
    Manifest(String),

    #[error("compatibility error: {0}")]
    Compatibility(String),

    #[error("capability error: {0}")]
    Capability(String),

    #[error("security error: {0}")]
    Security(String),

    #[error("registry error: {0}")]
    Registry(String),

    #[error("loader error: {0}")]
    Loader(String),

    #[error("runtime error: {0}")]
    Runtime(String),

    #[error("hook error: {0}")]
    Hook(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("semver error: {0}")]
    Semver(#[from] semver::Error),
}

impl From<String> for PluginError {
    fn from(value: String) -> Self {
        Self::Registry(value)
    }
}

/// Convenience alias for plugin operations that may fail with [`PluginError`].
pub type PluginResult<T> = Result<T, PluginError>;
