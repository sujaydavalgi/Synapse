//! Shared mutable state for the Control Center API server.
//!
use crate::correlation::TraceLog;
use spanda_config::{DeviceRegistry, ResolvedSystemConfig};
use spanda_ops::{AlertDispatcher, AlertStore};
use spanda_security::{ApiKeyStore, ManagedSecretVault};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Application state backing REST handlers.
#[derive(Debug)]
pub struct ControlCenterState {
    pub config_path: Option<PathBuf>,
    pub program_path: Option<PathBuf>,
    pub resolved: Option<ResolvedSystemConfig>,
    pub api_keys: ApiKeyStore,
    pub secret_vault: ManagedSecretVault,
    pub alert_dispatcher: AlertDispatcher,
    pub alert_store: AlertStore,
    pub trace_log: TraceLog,
}

impl ControlCenterState {
    pub fn new() -> Self {
        Self {
            config_path: None,
            program_path: None,
            resolved: None,
            api_keys: ApiKeyStore::from_env(),
            secret_vault: ManagedSecretVault::new(),
            alert_dispatcher: AlertDispatcher::from_env(),
            alert_store: AlertStore::new(500),
            trace_log: TraceLog::new(1000),
        }
    }

    pub fn with_config_path(mut self, path: PathBuf) -> Self {
        self.config_path = Some(path);
        self
    }

    pub fn device_registry(&self) -> DeviceRegistry {
        self.resolved
            .as_ref()
            .map(|r| r.device_registry.clone())
            .unwrap_or_default()
    }

    pub fn reload_config(&mut self) -> Result<(), String> {
        let Some(path) = self.config_path.as_ref() else {
            return Ok(());
        };
        let dir = if path.is_dir() {
            path.clone()
        } else {
            path.parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."))
        };
        let resolver = spanda_config::ConfigResolver::new();
        let resolved = resolver.resolve_from_dir(&dir).map_err(|e| e.to_string())?;
        self.resolved = Some(resolved);
        Ok(())
    }
}

impl Default for ControlCenterState {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedState = Arc<Mutex<ControlCenterState>>;

pub fn shared_state() -> SharedState {
    Arc::new(Mutex::new(ControlCenterState::new()))
}
