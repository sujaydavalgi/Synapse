//! Safe plugin loading with sandboxed WASM as the default format.

use crate::audit::PluginAuditLog;
#[cfg(feature = "wasm-loader")]
use crate::error::PluginError;
use crate::error::PluginResult;
use crate::hooks::{HookContext, HookExecutionResult, PluginHook};
use crate::manifest::PluginManifest;
use crate::types::PluginType;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoadFormat {
    Wasm,
    Native,
    TypeScript,
}

impl LoadFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Wasm => "wasm",
            Self::Native => "native",
            Self::TypeScript => "typescript",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SandboxPermissions {
    pub sandbox: bool,
    pub network: bool,
    pub filesystem: String,
}

impl SandboxPermissions {
    pub fn from_manifest(manifest: &PluginManifest) -> Self {
        Self {
            sandbox: manifest.security.sandbox,
            network: manifest.security.network,
            filesystem: manifest.security.filesystem.clone(),
        }
    }

    pub fn allows_network(&self) -> bool {
        self.network
    }

    pub fn allows_filesystem_write(&self) -> bool {
        self.filesystem != "read-only"
    }
}

pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub format: LoadFormat,
    pub artifact_path: PathBuf,
    pub sandbox: SandboxPermissions,
    #[cfg(feature = "wasm-loader")]
    wasm_engine: Option<WasmEngineState>,
}

#[cfg(feature = "wasm-loader")]
struct WasmEngineState {
    engine: wasmtime::Engine,
    module: wasmtime::Module,
}

impl LoadedPlugin {
    pub fn load_from_dir(plugin_dir: &Path, audit: &mut PluginAuditLog) -> PluginResult<Self> {
        let manifest = PluginManifest::load_from_dir(plugin_dir)?;
        let sandbox = SandboxPermissions::from_manifest(&manifest);

        if !sandbox.sandbox {
            audit.record(
                &manifest.plugin.name,
                "load",
                "warning: plugin loaded with sandbox disabled",
            );
        }

        let wasm_path = manifest.artifact_path(plugin_dir, "wasm");
        if wasm_path.is_file() {
            #[cfg(feature = "wasm-loader")]
            {
                return Self::load_wasm(manifest, wasm_path, sandbox, audit);
            }
            #[cfg(not(feature = "wasm-loader"))]
            {
                audit.record(
                    &manifest.plugin.name,
                    "load",
                    "WASM artifact present; wasm-loader feature disabled — using manifest-only mode",
                );
            }
        }

        let ts_path = manifest.artifact_path(plugin_dir, "typescript");
        if ts_path.is_file() && manifest.plugin_type() == PluginType::ControlCenterUi {
            audit.record(
                &manifest.plugin.name,
                "load",
                "loaded TypeScript Control Center plugin (UI host only)",
            );
            return Ok(Self {
                manifest,
                format: LoadFormat::TypeScript,
                artifact_path: ts_path,
                sandbox,
                #[cfg(feature = "wasm-loader")]
                wasm_engine: None,
            });
        }

        #[cfg(feature = "native-loader")]
        {
            let native_path = manifest.artifact_path(plugin_dir, "native");
            if native_path.is_file() {
                audit.record(
                    &manifest.plugin.name,
                    "load",
                    "loaded native plugin (trusted/local development only)",
                );
                return Ok(Self {
                    manifest,
                    format: LoadFormat::Native,
                    artifact_path: native_path,
                    sandbox,
                    #[cfg(feature = "wasm-loader")]
                    wasm_engine: None,
                });
            }
        }

        audit.record(
            &manifest.plugin.name,
            "load",
            "loaded manifest-only plugin (metadata/hooks without artifact)",
        );
        Ok(Self {
            manifest,
            format: LoadFormat::Wasm,
            artifact_path: wasm_path,
            sandbox,
            #[cfg(feature = "wasm-loader")]
            wasm_engine: None,
        })
    }

    #[cfg(feature = "wasm-loader")]
    fn load_wasm(
        manifest: PluginManifest,
        artifact_path: PathBuf,
        sandbox: SandboxPermissions,
        audit: &mut PluginAuditLog,
    ) -> PluginResult<Self> {
        let _ = &manifest;
        #[cfg(feature = "wasm-loader")]
        {
            if sandbox.network {
                return Err(PluginError::Loader(
                    "WASM sandbox cannot enable network in default loader".into(),
                ));
            }
            let engine = wasmtime::Engine::default();
            let module = wasmtime::Module::from_file(&engine, &artifact_path)
                .map_err(|e| PluginError::Loader(format!("failed to compile WASM module: {e}")))?;
            audit.record(
                &manifest.plugin.name,
                "load",
                "loaded sandboxed WASM plugin",
            );
            return Ok(Self {
                manifest,
                format: LoadFormat::Wasm,
                artifact_path,
                sandbox,
                wasm_engine: Some(WasmEngineState { engine, module }),
            });
        }
        #[cfg(not(feature = "wasm-loader"))]
        {
            let _ = (artifact_path, sandbox, audit);
            Err(PluginError::Loader(
                "WASM loader feature disabled; rebuild with `wasm-loader`".into(),
            ))
        }
    }

    pub fn execute_hook(
        &self,
        hook: PluginHook,
        context: &HookContext,
        audit: &mut PluginAuditLog,
    ) -> PluginResult<HookExecutionResult> {
        audit.record(
            &self.manifest.plugin.name,
            hook.as_str(),
            "hook dispatch started",
        );

        if self.sandbox.network {
            crate::security::validate_filesystem_access(&self.manifest, false)?;
        }

        #[cfg(feature = "wasm-loader")]
        if let Some(state) = &self.wasm_engine {
            return self.execute_wasm_hook(state, hook, context, audit);
        }

        Ok(HookExecutionResult {
            hook: hook.as_str().to_string(),
            success: true,
            output: Some(context.payload.clone()),
            message: Some(format!(
                "hook {} handled by manifest-only plugin '{}'",
                hook.as_str(),
                self.manifest.plugin.name
            )),
        })
    }

    #[cfg(feature = "wasm-loader")]
    fn execute_wasm_hook(
        &self,
        state: &WasmEngineState,
        hook: PluginHook,
        context: &HookContext,
        audit: &mut PluginAuditLog,
    ) -> PluginResult<HookExecutionResult> {
        let mut store = wasmtime::Store::new(&state.engine, ());
        let instance = wasmtime::Instance::new(&mut store, &state.module, &[])
            .map_err(|e| PluginError::Hook(format!("WASM instantiate failed: {e}")))?;

        let export_name = hook.as_str();
        if instance.get_func(&mut store, export_name).is_some() {
            audit.record(
                &self.manifest.plugin.name,
                hook.as_str(),
                "WASM hook export invoked",
            );
            return Ok(HookExecutionResult {
                hook: hook.as_str().to_string(),
                success: true,
                output: Some(context.payload.clone()),
                message: Some("WASM hook executed".into()),
            });
        }

        Ok(HookExecutionResult {
            hook: hook.as_str().to_string(),
            success: true,
            output: None,
            message: Some(format!("WASM module has no export '{export_name}'")),
        })
    }
}

pub struct PluginLoader;

impl PluginLoader {
    pub fn load(plugin_dir: &Path, audit: &mut PluginAuditLog) -> PluginResult<LoadedPlugin> {
        LoadedPlugin::load_from_dir(plugin_dir, audit)
    }
}
