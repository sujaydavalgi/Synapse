//! Versioned, capability-controlled plugin system for Spanda.
//!
//! Plugins complement packages and providers: they extend Control Center UI,
//! CLI commands, readiness/assurance/diagnosis hooks, and platform integrations
//! without modifying core crates.

pub mod api;
#[cfg(feature = "api-host")]
pub mod api_host;
pub mod audit;
pub mod bridge;
pub mod capability;
pub mod compatibility;
pub mod error;
pub mod hooks;
pub mod loader;
pub mod manifest;
#[cfg(feature = "api-host")]
pub mod provider;
pub mod registry;
pub mod registry_fetch;
pub mod runtime;
pub mod security;
pub mod types;

pub use api::{PluginApiContext, PluginApiSurface};
#[cfg(feature = "api-host")]
pub use api_host::{PluginApiHost, PluginApiHostContext};
pub use audit::{PluginAuditEntry, PluginAuditLog};
pub use bridge::{hook_for_platform_event, hook_for_report_request};
pub use capability::{CapabilitySet, PluginCapability, KNOWN_PLUGIN_CAPABILITIES};
pub use compatibility::{validate_api_version, validate_spanda_version, CompatibilityReport};
pub use error::{PluginError, PluginResult};
pub use hooks::{HookContext, HookExecutionResult, PluginHook};
pub use loader::{LoadFormat, PluginLoader, SandboxPermissions};
pub use manifest::{PluginManifest, MANIFEST_FILENAME};
pub use registry::{
    lookup_plugin_entry, plugin_registry_index, search_plugins, PluginRegistryEntry,
    PluginRegistryIndex, PluginTrustTier,
};
pub use registry_fetch::{fetch_plugin_sources, local_example_plugin_dir, AUDIT_LOG_FILENAME};
pub use runtime::{InstalledPlugin, PluginManager, PluginState, PluginStore};
pub use security::{dangerous_capabilities, validate_install_security, SecurityValidationReport};
pub use types::PluginType;
