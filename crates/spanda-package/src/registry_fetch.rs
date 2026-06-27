//! Download and extract registry package tarballs.
//!
//! Resolution order: local `dist/` tarballs, in-repo hosted `registry/packages/`,
//! then `.spanda/registry/` cache and remote `SPANDA_REGISTRY_URL`.

use crate::integrity::{
    checksum_sidecar_path, read_checksum_sidecar, registry_require_checksum, verify_sha256,
    write_checksum_sidecar,
};
use crate::registry_remote::registry_base_url;
use crate::registry_sign::{
    registry_require_signature, registry_trust_key, verify_registry_signature,
};
use crate::tar_extract::extract_tarball_safe;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn registry_tarball_url(name: &str, version: &str) -> Option<String> {
    // Description:
    //     Registry tarball url.
    //
    // Inputs:
    //     name: &str
    //         Caller-supplied name.
    //     version: &str
    //         Caller-supplied version.
    //
    // Outputs:
    //     result: Option<String>
    //         Return value from `registry_tarball_url`.
    //
    // Example:
    //     let result = spanda_package::registry_fetch::registry_tarball_url(name, version);

    // Transform registry base url and continue the chain.
    registry_base_url().map(|base| format!("{base}/packages/{name}/{version}"))
}

pub fn registry_cache_dir(project_root: &Path) -> PathBuf {
    // Description:
    //     Registry cache dir.
    //
    // Inputs:
    //     project_roo: &Path
    //         Caller-supplied project roo.
    //
    // Outputs:
    //     result: PathBuf
    //         Return value from `registry_cache_dir`.
    //
    // Example:
    //     let result = spanda_package::registry_fetch::registry_cache_dir(project_roo);

    // Produce spanda/registry") as the result.
    project_root.join(".spanda/registry")
}

pub fn global_registry_cache_dir() -> Option<PathBuf> {
    // Description:
    //     Global registry cache dir.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: Option<PathBuf>
    //         Return value from `global_registry_cache_dir`.
    //
    // Example:
    //     let result = spanda_package::registry_fetch::global_registry_cache_dir();

    // Produce var as the result.
    std::env::var("SPANDA_REGISTRY_CACHE")
        .ok()
        .map(PathBuf::from)
        .or_else(|| dirs_home().map(|home| home.join(".spanda/registry")))
}

fn dirs_home() -> Option<PathBuf> {
    // Description:
    //     Dirs home.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: Option<PathBuf>
    //         Return value from `dirs_home`.
    //
    // Example:
    //     let result = spanda_package::registry_fetch::dirs_home();

    // Produce var as the result.
    std::env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| std::env::var("USERPROFILE").ok().map(PathBuf::from))
}

pub fn local_tarball_candidates(project_root: &Path, name: &str, version: &str) -> Vec<PathBuf> {
    // Description:
    //     List local tarball paths in preferred resolution order.
    //
    // Inputs:
    //     project_roo: &Path
    //         Caller-supplied project roo.
    //     name: &str
    //         Caller-supplied name.
    //     version: &str
    //         Caller-supplied version.
    //
    // Outputs:
    //     result: Vec<PathBuf>
    //         Ordered tarball paths to probe.
    //
    // Example:
    //     let result = spanda_package::registry_fetch::local_tarball_candidates(project_roo, name, version);

    let mut candidates = vec![
        project_root
            .join("dist")
            .join(format!("{name}-{version}.tar.gz")),
    ];

    // Prefer the in-repo hosted registry when vendoring from a monorepo checkout.
    let mut dir = project_root.to_path_buf();
    loop {
        candidates.push(
            dir.join("registry")
                .join("packages")
                .join(name)
                .join(version),
        );
        if !dir.pop() {
            break;
        }
    }

    // Handle the success value from var.
    if let Ok(local) = std::env::var("SPANDA_REGISTRY_LOCAL") {
        let trimmed = local.trim();

        // Skip further work when !trimmed is empty.
        if !trimmed.is_empty() {
            candidates.push(PathBuf::from(trimmed).join(format!("{name}-{version}.tar.gz")));
        }
    }

    candidates.push(
        registry_cache_dir(project_root).join(format!("{name}-{version}.tar.gz")),
    );

    // Emit output when global registry cache dir provides a global.
    if let Some(global) = global_registry_cache_dir() {
        candidates.push(global.join(format!("{name}-{version}.tar.gz")));
    }
    candidates
}

pub fn resolve_local_tarball(project_root: &Path, name: &str, version: &str) -> Option<PathBuf> {
    // Description:
    //     Resolve local tarball.
    //
    // Inputs:
    //     project_roo: &Path
    //         Caller-supplied project roo.
    //     name: &str
    //         Caller-supplied name.
    //     version: &str
    //         Caller-supplied version.
    //
    // Outputs:
    //     result: Option<PathBuf>
    //         Return value from `resolve_local_tarball`.
    //
    // Example:
    //     let result = spanda_package::registry_fetch::resolve_local_tarball(project_roo, name, version);

    local_tarball_candidates(project_root, name, version)
        .into_iter()
        .find(|path| path.is_file())
}

fn is_registry_cache_tarball(path: &Path, project_root: &Path) -> bool {
    // Description:
    //     Return true when `path` lives under a registry tarball cache directory.
    //
    // Inputs:
    //     path: &Path
    //         Candidate tarball path.
    //     project_roo: &Path
    //         Project root for project-local cache detection.
    //
    // Outputs:
    //     result: bool
    //         Whether stale cache entries may be discarded.
    //
    // Example:
    //     let result = spanda_package::registry_fetch::is_registry_cache_tarball(path, project_roo);

    path.starts_with(registry_cache_dir(project_root))
        || global_registry_cache_dir()
            .is_some_and(|global| path.starts_with(global))
}

fn try_local_tarball(
    project_root: &Path,
    name: &str,
    version: &str,
    dest: &Path,
    expected_sha256: Option<&str>,
    expected_signature: Option<&crate::registry_sign::RegistryVersionSignature>,
) -> Result<Option<PathBuf>, String> {
    // Description:
    //     Extract the first verified local tarball candidate into `dest`.
    //
    // Inputs:
    //     project_roo: &Path
    //         Caller-supplied project roo.
    //     name: &str
    //         Caller-supplied name.
    //     version: &str
    //         Caller-supplied version.
    //     des: &Path
    //         Vendor destination directory.
    //     expected_sha256: Option<&str>
    //         Expected digest from the registry index.
    //     expected_signature: Option<&crate::registry_sign::RegistryVersionSignature>
    //         Expected signature from the registry index.
    //
    // Outputs:
    //     result: Result<Option<PathBuf>, String>
    //         Vendor path when a local tarball succeeds, otherwise `None`.
    //
    // Example:
    //     let result = spanda_package::registry_fetch::try_local_tarball(project_roo, name, version, des, expected_sha256, expected_signature);

    for local in local_tarball_candidates(project_root, name, version) {
        // Skip missing candidates and continue probing the next path.
        if !local.is_file() {
            continue;
        }

        let sidecar_digest = read_checksum_sidecar(&local);
        let digest = expected_sha256.or(sidecar_digest.as_deref());

        // Drop stale cache entries and continue when checksum verification fails.
        match verify_tarball_digest(&local, digest, name, version, expected_signature) {
            Ok(()) => {
                extract_tarball_safe(&local, dest)?;
                return Ok(Some(dest.to_path_buf()));
            }
            Err(err) if is_registry_cache_tarball(&local, project_root) => {
                let _ = fs::remove_file(&local);
                let _ = fs::remove_file(checksum_sidecar_path(&local));
                let _ = err;
            }
            Err(err) => return Err(err),
        }
    }
    Ok(None)
}

pub fn cache_registry_tarball(
    project_root: &Path,
    name: &str,
    version: &str,
    tarball: &Path,
) -> Result<PathBuf, String> {
    // Description:
    //     Cache registry tarball.
    //
    // Inputs:
    //     project_roo: &Path
    //         Caller-supplied project roo.
    //     name: &str
    //         Caller-supplied name.
    //     version: &str
    //         Caller-supplied version.
    //     arball: &Path
    //         Caller-supplied arball.
    //
    // Outputs:
    //     result: Result<PathBuf, String>
    //         Return value from `cache_registry_tarball`.
    //
    // Example:
    //     let result = spanda_package::registry_fetch::cache_registry_tarball(project_roo, name, version, arball);

    // Compute cache dir for the following logic.
    let cache_dir = registry_cache_dir(project_root);
    fs::create_dir_all(&cache_dir).map_err(|e| format!("create registry cache: {e}"))?;
    let dest = cache_dir.join(format!("{name}-{version}.tar.gz"));
    fs::copy(tarball, &dest).map_err(|e| format!("cache tarball: {e}"))?;

    // Emit output when global registry cache dir provides a global.
    if let Some(global) = global_registry_cache_dir() {
        // Take the branch when global differs from cache dir.
        if global != cache_dir {
            let _ = fs::create_dir_all(&global);
            let global_dest = global.join(format!("{name}-{version}.tar.gz"));
            let _ = fs::copy(tarball, &global_dest);
        }
    }
    Ok(dest)
}

pub fn fetch_registry_tarball(
    project_root: &Path,
    name: &str,
    version: &str,
    dest: &Path,
    expected_sha256: Option<&str>,
    expected_signature: Option<&crate::registry_sign::RegistryVersionSignature>,
) -> Result<PathBuf, String> {
    // Description:
    //     Fetch registry tarball.
    //
    // Inputs:
    //     project_roo: &Path
    //         Caller-supplied project roo.
    //     name: &str
    //         Caller-supplied name.
    //     version: &str
    //         Caller-supplied version.
    //     des: &Path
    //         Caller-supplied des.
    //     expected_sha256: Option<&str>
    //         Caller-supplied expected sha256.
    //     expected_signature: Option<&crate::registry_sign::RegistryVersionSignature>
    //         Caller-supplied expected signature.
    //
    // Outputs:
    //     result: Result<PathBuf, String>
    //         Return value from `fetch_registry_tarball`.
    //
    // Example:
    //     let result = spanda_package::registry_fetch::fetch_registry_tarball(project_roo, name, version, des, expected_sha256, expected_signature);

    // Produce map err as the result.
    fs::create_dir_all(dest).map_err(|e| format!("create vendor dir: {e}"))?;

    // Emit output when a verified local tarball is available.
    if let Some(path) = try_local_tarball(
        project_root,
        name,
        version,
        dest,
        expected_sha256,
        expected_signature,
    )? {
        return Ok(path);
    }
    if registry_require_checksum() && expected_sha256.is_none() {
        return Err(format!(
            "checksum required for '{name}@{version}' — set SPANDA_REGISTRY_REQUIRE_CHECKSUM=0 to allow unsigned remote installs"
        ));
    }
    if registry_require_signature() && expected_signature.is_none() {
        return Err(format!(
            "signature required for '{name}@{version}' — set SPANDA_REGISTRY_REQUIRE_SIGNATURE=0 to allow unsigned remote installs"
        ));
    }
    let url = registry_tarball_url(name, version).ok_or_else(|| {
        format!(
            "no tarball for '{name}@{version}' — run spanda publish (dist/) or set SPANDA_REGISTRY_URL"
        )
    })?;
    let tarball = dest.join(format!("{name}-{version}.tar.gz"));
    fetch_url_to_file(&url, &tarball)?;
    let digest = expected_sha256
        .map(str::to_string)
        .or_else(|| read_checksum_sidecar(&tarball))
        .or_else(|| crate::integrity::sha256_file(&tarball).ok());
    verify_tarball_digest(
        &tarball,
        digest.as_deref(),
        name,
        version,
        expected_signature,
    )?;
    let _ = cache_registry_tarball(project_root, name, version, &tarball);
    let _ = write_checksum_sidecar(&tarball);
    extract_tarball_safe(&tarball, dest)?;
    let _ = fs::remove_file(&tarball);
    Ok(dest.to_path_buf())
}

fn verify_tarball_digest(
    tarball: &Path,
    expected: Option<&str>,
    name: &str,
    version: &str,
    expected_signature: Option<&crate::registry_sign::RegistryVersionSignature>,
) -> Result<(), String> {
    // Description:
    //     Verify tarball digest.
    //
    // Inputs:
    //     arball: &Path
    //         Caller-supplied arball.
    //     expected: Option<&str>
    //         Caller-supplied expected.
    //     name: &str
    //         Caller-supplied name.
    //     version: &str
    //         Caller-supplied version.
    //     expected_signature: Option<&crate::registry_sign::RegistryVersionSignature>
    //         Caller-supplied expected signature.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `verify_tarball_digest`.
    //
    // Example:

    //     let result = spanda_package::registry_fetch::verify_tarball_digest(arball, expected, name, version, expected_signature);

    let digest = match expected {
        Some(digest) => {
            verify_sha256(tarball, digest)?;
            digest.to_string()
        }
        None if registry_require_checksum() => {
            return Err(format!("missing checksum for {}", tarball.display()));
        }
        None => crate::integrity::sha256_file(tarball)?,
    };

    if let Some(signature) = expected_signature {
        let trust_key = registry_trust_key().unwrap_or_else(|| signature.public_key.clone());
        if !verify_registry_signature(name, version, &digest, signature, &trust_key) {
            return Err(format!("invalid registry signature for {name}@{version}"));
        }
    } else if registry_require_signature() {
        return Err(format!("missing registry signature for {name}@{version}"));
    }
    Ok(())
}

pub fn fetch_url_to_file(url: &str, output: &Path) -> Result<(), String> {
    // Description:
    //     Fetch url to file.
    //
    // Inputs:
    //     url: &str
    //         Caller-supplied url.
    //     outp: &Path
    //         Caller-supplied outp.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `fetch_url_to_file`.
    //
    // Example:
    //     let result = spanda_package::registry_fetch::fetch_url_to_file(rl, outp);
    // use path when file url path is present.

    // Emit output when file url path provides a path.
    if let Some(path) = file_url_path(url) {
        fs::copy(&path, output).map_err(|e| format!("copy {path:?} to vendor: {e}"))?;
        return Ok(());
    }
    let status = Command::new("curl")
        .args(["-fsSL", url, "-o"])
        .arg(output)
        .status()
        .map_err(|e| format!("curl failed for {url}: {e}"))?;

    // Handle output when the subprocess succeeds.
    if status.success() {
        Ok(())
    } else {
        Err(format!("curl exited with {status} for {url}"))
    }
}

pub fn file_url_path(url: &str) -> Option<PathBuf> {
    // Description:
    //     File url path.
    //
    // Inputs:
    //     url: &str
    //         Caller-supplied url.
    //
    // Outputs:
    //     result: Option<PathBuf>
    //         Return value from `file_url_path`.
    //
    // Example:
    //     let result = spanda_package::registry_fetch::file_url_path(rl);

    // Resolve the filesystem path for the next step.
    let path = url.strip_prefix("file://")?;

    // Skip further work when path is empty.
    if path.is_empty() {
        return None;
    }
    Some(PathBuf::from(path))
}

pub fn extract_tarball(tarball: &Path, dest: &Path) -> Result<(), String> {
    // Description:
    //     Extract tarball.
    //
    // Inputs:
    //     arball: &Path
    //         Caller-supplied arball.
    //     des: &Path
    //         Caller-supplied des.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `extract_tarball`.
    //
    // Example:

    //     let result = spanda_package::registry_fetch::extract_tarball(arball, des);

    extract_tarball_safe(tarball, dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tarball_url_requires_registry_env() {
        // Description:
        //     Tarball url requires registry env.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_package::registry_fetch::tarball_url_requires_registry_env();

        let _guard = crate::testing::env_lock();
        // Tarball url requires registry env.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_package::registry_fetch::tarball_url_requires_registry_env();

        std::env::remove_var("SPANDA_REGISTRY_URL");
        assert!(registry_tarball_url("demo", "0.1.0").is_some());
        std::env::set_var("SPANDA_REGISTRY_URL", "");
        assert!(registry_tarball_url("demo", "0.1.0").is_none());
    }

    #[test]
    fn tarball_url_uses_base_path() {
        // Description:
        //     Tarball url uses base path.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_package::registry_fetch::tarball_url_uses_base_path();

        let _guard = crate::testing::env_lock();
        // Tarball url uses base path.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_package::registry_fetch::tarball_url_uses_base_path();

        std::env::set_var("SPANDA_REGISTRY_URL", "https://registry.example.com");
        assert_eq!(
            registry_tarball_url("spanda-mqtt", "1.0.0"),
            Some("https://registry.example.com/packages/spanda-mqtt/1.0.0".into())
        );
        std::env::remove_var("SPANDA_REGISTRY_URL");
    }

    #[test]
    fn file_url_path_parses_local_paths() {
        // Description:
        //     File url path parses local paths.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_package::registry_fetch::file_url_path_parses_local_paths();

        assert_eq!(
            file_url_path("file:///tmp/registry/index.json"),
            Some(PathBuf::from("/tmp/registry/index.json"))
        );
    }

    #[test]
    fn resolve_local_tarball_finds_dist_bundle() {
        // Description:
        //     Resolve local tarball finds dist bundle.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_package::registry_fetch::resolve_local_tarball_finds_dist_bundle();

        let root = std::env::temp_dir().join(format!("spanda-fetch-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("dist")).unwrap();
        let bundle = root.join("dist/demo-0.1.0.tar.gz");
        fs::write(&bundle, b"not a real tar").unwrap();
        assert_eq!(resolve_local_tarball(&root, "demo", "0.1.0"), Some(bundle));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn local_tarball_candidates_prefers_hosted_registry_before_cache() {
        let root =
            std::env::temp_dir().join(format!("spanda-hosted-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".spanda/registry")).unwrap();
        fs::create_dir_all(root.join("registry/packages/demo")).unwrap();
        let stale = root.join(".spanda/registry/demo-0.1.0.tar.gz");
        let hosted = root.join("registry/packages/demo/0.1.0");
        fs::write(&stale, b"stale").unwrap();
        fs::write(&hosted, b"fresh").unwrap();
        assert_eq!(resolve_local_tarball(&root, "demo", "0.1.0"), Some(hosted));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn cache_registry_tarball_writes_project_cache() {
        // Description:
        //     Cache registry tarball writes project cache.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_package::registry_fetch::cache_registry_tarball_writes_project_cache();

        let root = std::env::temp_dir().join(format!("spanda-cache-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let src = root.join("src.tar.gz");
        fs::write(&src, b"payload").unwrap();
        let cached = cache_registry_tarball(&root, "demo", "0.2.0", &src).unwrap();
        assert!(cached.is_file());
        assert_eq!(resolve_local_tarball(&root, "demo", "0.2.0"), Some(cached));
        let _ = fs::remove_dir_all(&root);
    }
}
