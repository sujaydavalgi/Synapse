//! Vendor resolved dependencies into `.spanda/packages/` for install/build.

use crate::dependency::LockedSource;
use crate::error::{PackageError, PackageResult};
use crate::lockfile::Lockfile;
use crate::registry::registry_package_dir;
use crate::registry_fetch::fetch_registry_tarball;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct VendorReport {
    pub vendored: Vec<String>,
    pub skipped: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn vendor_dependencies(
    project_root: &Path,
    lockfile: &Lockfile,
) -> PackageResult<VendorReport> {
    // Vendor dependencies.
    //
    // Parameters:
    // - `project_root` — input value
    // - `lockfile` — input value
    //
    // Returns:
    // PackageResult<VendorReport>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::vendor::vendor_dependencies(project_root, lockfile);

    // Compute vendor root for the following logic.
    let vendor_root = project_root.join(".spanda/packages");
    fs::create_dir_all(&vendor_root).map_err(PackageError::Io)?;
    let mut report = VendorReport::default();

    // Iterate over dependencies with destructured elements.
    for (name, dep) in &lockfile.dependencies {
        // Match on source and handle each case.
        match &dep.source {
            LockedSource::Local { path } => {
                report
                    .skipped
                    .push(format!("{name} (local path {})", path.display()));
            }
            LockedSource::Registry { .. } => {
                // Match on version, &vendor root)? and handle each case.
                match vendor_registry_package(project_root, name, &dep.version, &vendor_root)? {
                    Some(path) => report.vendored.push(format!("{name} → {}", path.display())),
                    None => report.warnings.push(format!(
                        "registry package '{name}' has no local source tree — lockfile only"
                    )),
                }
            }
            LockedSource::Git {
                url,
                branch,
                tag,
                rev,
            } => {
                let dest = vendor_root.join(name);

                // Act only when the target path already exists.
                if dest.exists() {
                    fs::remove_dir_all(&dest).map_err(PackageError::Io)?;
                }
                vendor_git(
                    url,
                    branch.as_deref(),
                    tag.as_deref(),
                    rev.as_deref(),
                    &dest,
                )?;
                report.vendored.push(format!("{name} → {}", dest.display()));
            }
        }
    }
    Ok(report)
}

fn vendor_registry_package(
    project_root: &Path,
    name: &str,
    version: &str,
    vendor_root: &Path,
) -> PackageResult<Option<PathBuf>> {
    // Vendor registry package.
    //
    // Parameters:
    // - `project_root` — input value
    // - `name` — input value
    // - `version` — input value
    // - `vendor_root` — input value
    //
    // Returns:
    // PackageResult<Option<PathBuf>>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::vendor::vendor_registry_package(project_root, name, version, vendor_root);

    // Compute dest for the following logic.
    let dest = vendor_root.join(name);

    // Act only when the target path already exists.
    if dest.exists() {
        fs::remove_dir_all(&dest).map_err(PackageError::Io)?;
    }

    // Emit output when registry package dir provides a src.
    if let Some(src) = registry_package_dir(name) {
        copy_dir_recursive(&src, &dest)?;
        let _ = project_root;
        return Ok(Some(dest));
    }

    // Match on fetch registry tarball and handle each case.
    match fetch_registry_tarball(project_root, name, version, &dest) {
        Ok(path) => Ok(Some(path)),
        Err(err) => {
            let _ = err;
            Ok(None)
        }
    }
}

fn vendor_git(
    url: &str,
    branch: Option<&str>,
    tag: Option<&str>,
    rev: Option<&str>,
    dest: &Path,
) -> PackageResult<()> {
    // Vendor git.
    //
    // Parameters:
    // - `url` — input value
    // - `branch` — input value
    // - `tag` — input value
    // - `rev` — input value
    // - `dest` — input value
    //
    // Returns:
    // PackageResult<()>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::vendor::vendor_git(url, branch, tag, rev, dest);

    // Create mutable args for accumulating results.
    let mut args = vec!["clone", "--depth", "1"];

    // Emit output when tag provides a t.
    if let Some(t) = tag {
        args.push("--branch");
        args.push(t);
    } else if let Some(b) = branch {
        args.push("--branch");
        args.push(b);
    }
    args.push(url);
    args.push(
        dest.to_str()
            .ok_or_else(|| PackageError::Dependency("invalid vendor destination path".into()))?,
    );
    let status = Command::new("git")
        .args(&args)
        .status()
        .map_err(|e| PackageError::Dependency(format!("git clone failed for {url}: {e}")))?;

    // Handle output when the subprocess succeeds.
    if !status.success() {
        return Err(PackageError::Dependency(format!(
            "git clone exited with {status} for {url}"
        )));
    }

    // Emit output when rev provides a revision.
    if let Some(revision) = rev {
        let checkout = Command::new("git")
            .args(["checkout", revision])
            .current_dir(dest)
            .status()
            .map_err(|e| PackageError::Dependency(format!("git checkout failed: {e}")))?;

        // Handle output when the subprocess succeeds.
        if !checkout.success() {
            return Err(PackageError::Dependency(format!(
                "git checkout {revision} failed"
            )));
        }
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> PackageResult<()> {
    // Copy dir recursive.
    //
    // Parameters:
    // - `src` — input value
    // - `dest` — input value
    //
    // Returns:
    // PackageResult<()>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::vendor::copy_dir_recursive(src, dest);

    // Produce Io)? as the result.
    fs::create_dir_all(dest).map_err(PackageError::Io)?;

    // Process each registry entry.
    for entry in fs::read_dir(src).map_err(PackageError::Io)? {
        let entry = entry.map_err(PackageError::Io)?;
        let file_type = entry.file_type().map_err(PackageError::Io)?;
        let target = dest.join(entry.file_name());

        // Treat the path as a directory and scan its contents.
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target).map_err(PackageError::Io)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependency::{LockedDependency, LockedSource};
    use crate::lockfile::Lockfile;
    use crate::manifest::{PackageManifest, PackageSection};
    use std::collections::BTreeMap;

    #[test]
    fn vendors_local_registry_package_when_present() {
        // Vendors local registry package when present.
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
        // let result = spanda_package::vendor::vendors_local_registry_package_when_present();

        let root = std::env::temp_dir().join(format!("spanda-vendor-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let mut deps = BTreeMap::new();
        deps.insert(
            "spanda-mqtt".into(),
            LockedDependency {
                name: "spanda-mqtt".into(),
                version: "0.1.0".into(),
                source: LockedSource::Registry {
                    registry: "local".into(),
                },
                checksum: None,
            },
        );
        let manifest = PackageManifest {
            package: PackageSection {
                name: "demo".into(),
                version: "0.1.0".into(),
                description: None,
                license: None,
                authors: vec![],
            },
            dependencies: std::collections::HashMap::new(),
            dev_dependencies: std::collections::HashMap::new(),
            hardware: Default::default(),
            capabilities: Default::default(),
            requires_hardware: Default::default(),
            safety: Default::default(),
            adapter: Default::default(),
            categories: vec![],
            license_compat: vec![],
        };
        let lockfile = Lockfile::new(&manifest, deps);
        let report = vendor_dependencies(&root, &lockfile).unwrap();
        if registry_package_dir("spanda-mqtt").is_some() {
            assert!(!report.vendored.is_empty());
        }
        let _ = std::fs::remove_dir_all(&root);
    }
}
