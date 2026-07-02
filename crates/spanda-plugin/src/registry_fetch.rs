//! Fetch and extract plugin packages from registry tarballs.

use crate::error::{PluginError, PluginResult};
use crate::registry::{lookup_plugin_entry, PluginRegistryEntry};
use spanda_package::registry_fetch::{fetch_registry_tarball, fetch_url_to_file};
use spanda_package::registry_remote::registry_base_url;
use spanda_package::tar_extract::extract_tarball_safe;
use std::fs;
use std::path::{Path, PathBuf};

pub const AUDIT_LOG_FILENAME: &str = "audit.jsonl";

/// Resolve a plugin tarball URL from the Spanda registry base.
pub fn plugin_registry_tarball_url(name: &str, version: &str) -> Option<String> {
    registry_base_url().map(|base| format!("{base}/plugins/{name}/{version}.tar.gz"))
}

/// Local example plugin directory for bundled registry names.
pub fn local_example_plugin_dir(name: &str) -> Option<PathBuf> {
    let slug = name.strip_prefix("spanda-plugin-").unwrap_or(name);
    for root in std::env::current_dir().ok().into_iter().flat_map(|cwd| {
        cwd.ancestors()
            .map(Path::to_path_buf)
            .collect::<Vec<_>>()
    }) {
        let direct = root.join("examples/plugins").join(slug);
        if direct.is_dir() {
            return Some(direct);
        }
        let underscored = root.join("examples/plugins").join(slug.replace('-', "_"));
        if underscored.is_dir() {
            return Some(underscored);
        }
    }
    None
}

/// Fetch plugin sources into a temporary directory for install.
pub fn fetch_plugin_sources(name: &str, version: &str, project_root: &Path) -> PluginResult<PathBuf> {
    if let Some(example) = local_example_plugin_dir(name) {
        return Ok(example);
    }

    if let Some(url) = plugin_registry_tarball_url(name, version) {
        if let Ok(path) = fetch_plugin_tarball(name, version, &url, project_root) {
            return Ok(path);
        }
    }

    if let Some(pkg) = try_fetch_as_package(name, version, project_root) {
        return Ok(pkg);
    }

    Err(PluginError::Registry(format!(
        "could not fetch plugin '{name}@{version}'; use --path for local install"
    )))
}

fn try_fetch_as_package(name: &str, version: &str, project_root: &Path) -> Option<PathBuf> {
    let entry = lookup_plugin_entry(name)?;
    let cache_dir = project_root.join(".spanda/plugins/.cache");
    let archive = fetch_registry_tarball(
        project_root,
        name,
        version,
        &cache_dir,
        entry.version_sha256(version).as_deref(),
        entry.version_signature(version),
    )
    .ok()?;
    let extract_dir = cache_dir.join(format!("{name}-{version}-pkg"));
    if extract_dir.exists() {
        let _ = fs::remove_dir_all(&extract_dir);
    }
    fs::create_dir_all(&extract_dir).ok()?;
    extract_tarball_safe(&archive, &extract_dir).ok()?;
    Some(extract_dir)
}

fn fetch_plugin_tarball(
    name: &str,
    version: &str,
    url: &str,
    project_root: &Path,
) -> PluginResult<PathBuf> {
    let cache_dir = project_root.join(".spanda/plugins/.cache");
    fs::create_dir_all(&cache_dir)?;
    let archive = cache_dir.join(format!("{name}-{version}.tar.gz"));
    fetch_url_to_file(url, &archive)
        .map_err(|e| PluginError::Registry(format!("download failed: {e}")))?;
    let entry = lookup_plugin_entry(name);
    if let (Some(entry), Some(digest)) = (entry.as_ref(), entry.as_ref().and_then(|e| e.version_sha256(version))) {
        verify_plugin_digest(name, version, &archive, digest, entry)?;
    }
    let extract_dir = cache_dir.join(format!("{name}-{version}"));
    if extract_dir.exists() {
        fs::remove_dir_all(&extract_dir)?;
    }
    fs::create_dir_all(&extract_dir)?;
    extract_tarball_safe(&archive, &extract_dir)?;
    Ok(extract_dir)
}

fn verify_plugin_digest(
    name: &str,
    version: &str,
    archive: &Path,
    expected: &str,
    entry: &PluginRegistryEntry,
) -> PluginResult<()> {
    let bytes = fs::read(archive)?;
    let digest = format!("{:x}", sha2::Sha256::digest(bytes));
    if digest != expected {
        return Err(PluginError::Registry(format!(
            "checksum mismatch for {name}@{version}"
        )));
    }
    if let Some(sig) = entry.version_signature(version) {
        if !crate::registry::verify_plugin_registry_signature(name, version, &digest, sig) {
            return Err(PluginError::Security(format!(
                "signature verification failed for {name}@{version}"
            )));
        }
    }
    Ok(())
}

use sha2::Digest;
