//! Bundled offline registry slice shipped inside the spanda CLI crate.

use std::env;
use std::path::{Path, PathBuf};

/// Return the bundled registry directory when `index.json` is present.
pub fn bundled_registry_dir() -> Option<PathBuf> {
    // Locate the offline registry slice shipped with the spanda CLI crate.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Path to `bundled-registry` when present.
    //
    // Options:
    // None.
    //
    // Example:
    // let dir = bundled_registry::bundled_registry_dir();

    let bundled = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bundled-registry");
    if bundled.join("index.json").is_file() {
        Some(bundled)
    } else {
        None
    }
}

/// Return a `file://` URL for the bundled registry when available.
pub fn bundled_registry_url() -> Option<String> {
    bundled_registry_dir().map(|dir| format!("file://{}", dir.display()))
}

/// Set `SPANDA_REGISTRY_URL` to the bundled registry when unset.
pub fn ensure_bundled_registry_env() {
    // Default registry resolution to bundled trust and spoofing packages.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // None (may set process environment).
    //
    // Options:
    // Respects a non-empty existing `SPANDA_REGISTRY_URL`.
    //
    // Example:
    // bundled_registry::ensure_bundled_registry_env();

    if env::var("SPANDA_REGISTRY_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .is_some()
    {
        return;
    }
    if let Some(url) = bundled_registry_url() {
        env::set_var("SPANDA_REGISTRY_URL", url);
    }
}

/// Return true when `path` lives under the bundled registry directory.
#[allow(dead_code)]
pub fn is_bundled_registry_path(path: &Path) -> bool {
    bundled_registry_dir()
        .map(|dir| path.starts_with(dir))
        .unwrap_or(false)
}
