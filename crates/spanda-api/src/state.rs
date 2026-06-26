//! Shared mutable state for the Control Center API server.
//!
use crate::correlation::TraceLog;
use spanda_audit::AuditRuntime;
use spanda_config::{DeviceRegistry, ResolvedSystemConfig};
use spanda_ops::{AlertDispatcher, AlertStore};
use spanda_security::{ApiKeyStore, ManagedSecretVault, RateLimiter};
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
    pub rate_limiter: RateLimiter,
    pub mutation_audit: AuditRuntime,
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
            rate_limiter: RateLimiter::from_env(),
            mutation_audit: AuditRuntime::new("control-center", vec![]),
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

    pub fn project_root(&self) -> Option<PathBuf> {
        let path = self.config_path.as_ref()?;
        Some(if path.is_dir() {
            path.clone()
        } else {
            path.parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."))
        })
    }

    /// Persist one device record to disk and reload resolved configuration.
    pub fn persist_device(
        &mut self,
        device_id: &str,
    ) -> Result<spanda_config::DevicePersistResult, String> {
        let root = self
            .project_root()
            .ok_or_else(|| "no config path".to_string())?;
        let resolved = self
            .resolved
            .as_ref()
            .ok_or_else(|| "no resolved configuration".to_string())?;
        let device = resolved
            .device_registry
            .get(device_id)
            .ok_or_else(|| format!("device '{device_id}' not found"))?
            .clone();
        let result = spanda_config::persist_device_record(&root, &resolved.manifest, &device)
            .map_err(|e| e.to_string())?;
        self.reload_config()?;
        Ok(result)
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
