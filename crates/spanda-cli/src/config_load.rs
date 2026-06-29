//! Shared configuration loading for CLI commands.
//!
use spanda_config::{
    config_flag_from_args, provider_packages_for_runtime, resolve_for_source_or_exit,
    ResolvedSystemConfig, SpandaManifest,
};
use spanda_driver::RunOptions;
use spanda_modules::load_project_modules;
use spanda_package::load_official_packages_for_source;
use std::path::Path;
use std::sync::Arc;

pub use spanda_config::{ensure_config_valid, verify_with_system_config};

/// Apply resolved configuration onto interpreter run options.
pub fn apply_system_config_to_run_options(
    cfg: Option<Arc<ResolvedSystemConfig>>,
    mut opts: RunOptions,
    source: &Path,
) -> RunOptions {
    opts.system_config = cfg.clone();
    if let Some(ref c) = cfg {
        let packages = provider_packages_for_runtime(c);
        if !packages.is_empty() {
            opts.official_packages = packages;
        }
        if let Ok(registry) = load_project_modules(&c.project_root) {
            opts.module_registry = Some(registry);
        }
    } else {
        opts.official_packages = load_official_packages_for_source(source);
        let root = SpandaManifest::find_project_root(source).unwrap_or_else(|| {
            if source.is_file() {
                source.parent().unwrap_or(source).to_path_buf()
            } else {
                source.to_path_buf()
            }
        });
        if let Ok(registry) = load_project_modules(&root) {
            opts.module_registry = Some(registry);
        }
    }
    if opts.runtime_hooks.is_none() {
        opts.runtime_hooks = Some(crate::runtime_hooks::default_runtime_hooks());
    }
    opts
}

/// Resolve config for a source file with optional explicit manifest path.
pub fn load_system_config(
    source: &Path,
    config_flag: Option<&Path>,
) -> Option<Arc<ResolvedSystemConfig>> {
    resolve_for_source_or_exit(source, config_flag, true)
}

/// Parse `--config` from args and resolve for the first non-flag file argument.
pub fn load_system_config_from_cli_args(args: &[String]) -> Option<Arc<ResolvedSystemConfig>> {
    let config_flag = config_flag_from_args(args);
    let source = args.iter().find(|a| !a.starts_with('-'))?;
    load_system_config(Path::new(source), config_flag.as_deref())
}
