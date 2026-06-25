//! Integration helpers for wiring `ResolvedSystemConfig` into runtime and verification.
//!
use crate::manifest::SpandaManifest;
use crate::resolved::ResolvedSystemConfig;
use crate::resolver::ConfigResolver;
use crate::validation::ValidationSeverity;
use spanda_ast::nodes::Program;
use spanda_hardware::{verify_program_compatibility, CompatibilityReport, VerifyOptions};
use spanda_package::{
    official_packages_from_lockfile, official_packages_from_manifest, PackageManifest,
    LOCKFILE_FILENAME, MANIFEST_FILENAME,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Resolve system configuration from a source file and optional `--config` path.
pub fn resolve_for_source(
    source: &Path,
    config_flag: Option<&Path>,
    validate: bool,
) -> crate::error::ConfigResult<Option<ResolvedSystemConfig>> {
    // Resolve cascading config for a `.sd` file or project directory.
    //
    // Parameters:
    // - `source` — program file or directory path
    // - `config_flag` — optional explicit `spanda.toml` path from `--config`
    // - `validate` — run device-tree validation during resolution
    //
    // Returns:
    // Resolved config when a manifest exists, otherwise `Ok(None)`.
    //
    // Options:
    // None.
    //
    // Example:
    // let cfg = resolve_for_source(path, None, true)?;

    let project_root = if let Some(flag) = config_flag {
        flag.parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        SpandaManifest::find_project_root(source).unwrap_or_else(|| {
            if source.is_dir() {
                source.to_path_buf()
            } else {
                source.parent().unwrap_or(source).to_path_buf()
            }
        })
    };
    let manifest_path = project_root.join(MANIFEST_FILENAME);
    if !manifest_path.exists() {
        return Ok(None);
    }
    let resolver = ConfigResolver::new().with_validation(validate);
    resolver
        .resolve(&project_root, &SpandaManifest::load(&manifest_path)?)
        .map(Some)
}

/// Resolve config or exit with a CLI error message.
pub fn resolve_for_source_or_exit(
    source: &Path,
    config_flag: Option<&Path>,
    validate: bool,
) -> Option<Arc<ResolvedSystemConfig>> {
    match resolve_for_source(source, config_flag, validate) {
        Ok(Some(cfg)) => Some(Arc::new(cfg)),
        Ok(None) => None,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

/// Official provider packages derived from resolved project state (lockfile preferred).
pub fn official_packages_from_resolved(cfg: &ResolvedSystemConfig) -> Vec<String> {
    let lock_path = cfg.project_root.join(LOCKFILE_FILENAME);
    if lock_path.exists() {
        if let Ok(lock) = spanda_package::Lockfile::load(&lock_path) {
            return official_packages_from_lockfile(&lock);
        }
    }
    if let Ok(manifest) = PackageManifest::load_from_dir(&cfg.project_root) {
        return official_packages_from_manifest(&manifest);
    }
    cfg.providers.clone()
}

/// Default hardware verify target from fleet device tree.
pub fn default_verify_target(cfg: &ResolvedSystemConfig) -> Option<String> {
    cfg.device_tree
        .fleet
        .as_ref()
        .and_then(|f| f.robots.first())
        .and_then(|r| r.hardware_profile.clone())
}

/// Project-relative recovery knowledge store path.
pub fn recovery_knowledge_path(cfg: &ResolvedSystemConfig) -> PathBuf {
    if let Some(section) = cfg.recovery_config() {
        if let Some(path) = section.get("knowledge_store").and_then(|v| v.as_str()) {
            return cfg.project_root.join(path);
        }
    }
    cfg.project_root
        .join(".spanda")
        .join("recovery_knowledge.json")
}

/// Verify program compatibility enriched with resolved configuration checks.
pub fn verify_with_system_config(
    program: &Program,
    cfg: Option<&ResolvedSystemConfig>,
    mut options: VerifyOptions,
) -> CompatibilityReport {
    if options.target.is_none() {
        if let Some(c) = cfg {
            options.target = default_verify_target(c);
        }
    }
    let mut report = verify_program_compatibility(program, &options);
    if let Some(c) = cfg {
        for finding in &c.validation.findings {
            report.items.push(spanda_hardware::CompatItem {
                category: finding.code.clone(),
                message: finding.message.clone(),
                severity: match finding.severity {
                    ValidationSeverity::Error => spanda_hardware::CompatSeverity::Error,
                    ValidationSeverity::Warning => spanda_hardware::CompatSeverity::Warning,
                    ValidationSeverity::Info => spanda_hardware::CompatSeverity::Pass,
                },
                line: 0,
                column: 0,
            });
            if finding.severity == ValidationSeverity::Error {
                report.compatible = false;
            }
        }
    }
    report
}

/// Parse optional `--config` path from CLI argument slice.
pub fn config_flag_from_args(args: &[String]) -> Option<PathBuf> {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--config" {
            return args.get(i + 1).map(PathBuf::from);
        }
    }
    None
}

/// Fail fast when config validation did not pass.
pub fn ensure_config_valid(cfg: Option<&ResolvedSystemConfig>) {
    if let Some(c) = cfg {
        if !c.validation.passed {
            eprintln!(
                "Configuration validation failed ({} errors); run `spanda config validate`",
                c.validation.error_count()
            );
            std::process::exit(1);
        }
    }
}

/// Robot ids declared in the resolved fleet config.
pub fn configured_robot_ids(cfg: &ResolvedSystemConfig) -> Vec<String> {
    cfg.device_tree
        .fleet
        .as_ref()
        .map(|f| f.robots.iter().map(|r| r.id.clone()).collect())
        .unwrap_or_default()
}
