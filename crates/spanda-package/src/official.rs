//! Resolve installed official lean-core packages from project manifests.
//!
use crate::adapter::framework_packages;
use crate::dependency::{DependencySpec, LockedDependency, LockedSource};
use crate::error::{PackageError, PackageResult};
use crate::lockfile::{Lockfile, LOCKFILE_FILENAME};
use crate::manifest::{PackageManifest, MANIFEST_FILENAME};
use crate::registry::registry_package_dir;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// How an official package name was resolved for provider bootstrap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfficialProvenance {
    /// Name is not in the official framework catalog.
    NotOfficial,
    /// Resolved from registry (`version` constraint or lockfile `registry` source).
    Registry,
    /// Local path points at the canonical in-repo `packages/registry/<name>` tree.
    CanonicalLocalRegistry,
    /// Official name reused via path/git override outside the registry tree.
    UnofficialOverride,
}

/// Return dependency names that match known official framework packages (catalog only).
pub fn installed_official_packages<'a>(
    dependency_names: impl IntoIterator<Item = &'a str>,
) -> Vec<&'static str> {
    // Collect catalog official names from a dependency key list (ignores source provenance).
    //
    // Parameters:
    // - `dependency_names` — keys from `spanda.toml` `[dependencies]`
    //
    // Returns:
    // Sorted list of official package names present in the dependency keys.
    //
    // Options:
    // None.
    //
    // Example:
    // let names = installed_official_packages(["spanda-ros2", "my-local-lib"]);

    let official: HashSet<&str> = framework_packages().iter().map(|p| p.name).collect();
    let mut found: Vec<&str> = dependency_names
        .into_iter()
        .filter_map(|name| official.get(name).copied())
        .collect();
    found.sort_unstable();
    found.dedup();
    found
}

/// Whether a package name is a registered official framework package.
pub fn is_official_package(name: &str) -> bool {
    // Return true when the name appears in the static official framework catalog.
    //
    // Parameters:
    // - `name` — package name
    //
    // Returns:
    // Catalog membership (does not imply registry provenance).
    //
    // Options:
    // None.
    //
    // Example:
    // assert!(is_official_package("spanda-mqtt"));

    framework_packages().iter().any(|p| p.name == name)
}

/// Classify provenance for one manifest dependency.
pub fn dependency_provenance(
    name: &str,
    spec: &DependencySpec,
    project_root: &Path,
) -> OfficialProvenance {
    // Classify whether an official dependency name is registry-provenanced.
    //
    // Parameters:
    // - `name` — dependency key from `spanda.toml`
    // - `spec` — parsed dependency specification
    // - `project_root` — project directory for relative path resolution
    //
    // Returns:
    // Provenance tier used for provider bootstrap and validation.
    //
    // Options:
    // None.
    //
    // Example:
    // let kind = dependency_provenance("spanda-mqtt", &spec, project_root);

    if !is_official_package(name) {
        return OfficialProvenance::NotOfficial;
    }
    match spec.source_kind() {
        crate::dependency::DependencySourceKind::Registry => OfficialProvenance::Registry,
        crate::dependency::DependencySourceKind::Git => OfficialProvenance::UnofficialOverride,
        crate::dependency::DependencySourceKind::Local => {
            if let Some(path) = spec.local_path(project_root) {
                if local_path_is_canonical_registry(name, &path) {
                    OfficialProvenance::CanonicalLocalRegistry
                } else {
                    OfficialProvenance::UnofficialOverride
                }
            } else {
                OfficialProvenance::UnofficialOverride
            }
        }
    }
}

/// Classify provenance for one lockfile dependency.
pub fn locked_dependency_provenance(
    name: &str,
    dep: &LockedDependency,
    project_root: &Path,
) -> OfficialProvenance {
    // Classify provenance for a resolved lockfile entry.
    //
    // Parameters:
    // - `name` — dependency key
    // - `dep` — locked dependency record
    // - `project_root` — project directory for relative path resolution
    //
    // Returns:
    // Provenance tier used for provider bootstrap.
    //
    // Options:
    // None.
    //
    // Example:
    // let kind = locked_dependency_provenance("spanda-mqtt", &dep, project_root);

    if !is_official_package(name) {
        return OfficialProvenance::NotOfficial;
    }
    match &dep.source {
        LockedSource::Registry { .. } => OfficialProvenance::Registry,
        LockedSource::Git { .. } => OfficialProvenance::UnofficialOverride,
        LockedSource::Local { path } => {
            let resolved = resolve_local_dependency_path(project_root, path);
            if local_path_is_canonical_registry(name, &resolved) {
                OfficialProvenance::CanonicalLocalRegistry
            } else {
                OfficialProvenance::UnofficialOverride
            }
        }
    }
}

/// Whether provenance qualifies for built-in official provider bootstrap.
pub fn provenance_wires_official_providers(provenance: OfficialProvenance) -> bool {
    // Return true when provenance should wire built-in official providers.
    //
    // Parameters:
    // - `provenance` — classified dependency provenance
    //
    // Returns:
    // `true` for registry and canonical monorepo registry trees.
    //
    // Options:
    // None.
    //
    // Example:
    // assert!(provenance_wires_official_providers(OfficialProvenance::Registry));

    matches!(
        provenance,
        OfficialProvenance::Registry | OfficialProvenance::CanonicalLocalRegistry
    )
}

/// Official names with registry or canonical local provenance from a manifest.
pub fn official_packages_from_manifest(
    manifest: &PackageManifest,
    project_root: &Path,
) -> Vec<String> {
    // Resolve official packages from manifest dependencies with provenance checks.
    //
    // Parameters:
    // - `manifest` — parsed project manifest
    // - `project_root` — project directory for path resolution
    //
    // Returns:
    // Official package names eligible for provider bootstrap.
    //
    // Options:
    // None.
    //
    // Example:
    // let names = official_packages_from_manifest(&manifest, project_root);

    let mut found: Vec<String> = manifest
        .dependencies
        .iter()
        .filter_map(|(name, spec)| {
            let provenance = dependency_provenance(name, spec, project_root);
            provenance_wires_official_providers(provenance).then(|| name.clone())
        })
        .collect();
    found.sort_unstable();
    found.dedup();
    found
}

/// Official names with registry or canonical local provenance from a lockfile.
pub fn official_packages_from_lockfile(lockfile: &Lockfile, project_root: &Path) -> Vec<String> {
    // Resolve official packages from lockfile entries with provenance checks.
    //
    // Parameters:
    // - `lockfile` — resolved dependency lockfile
    // - `project_root` — project directory for path resolution
    //
    // Returns:
    // Official package names eligible for provider bootstrap.
    //
    // Options:
    // None.
    //
    // Example:
    // let names = official_packages_from_lockfile(&lockfile, project_root);

    let mut found: Vec<String> = lockfile
        .dependencies
        .iter()
        .filter_map(|(name, dep)| {
            let provenance = locked_dependency_provenance(name, dep, project_root);
            provenance_wires_official_providers(provenance).then(|| name.clone())
        })
        .collect();
    found.sort_unstable();
    found.dedup();
    found
}

/// Official-name dependencies that fail registry provenance checks.
pub fn unofficial_official_overrides_from_manifest(
    manifest: &PackageManifest,
    project_root: &Path,
) -> Vec<String> {
    // List official catalog names overridden by path/git sources.
    //
    // Parameters:
    // - `manifest` — parsed project manifest
    // - `project_root` — project directory for path resolution
    //
    // Returns:
    // Dependency keys that squat an official name without registry provenance.
    //
    // Options:
    // None.
    //
    // Example:
    // let squat = unofficial_official_overrides_from_manifest(&manifest, project_root);

    manifest
        .dependencies
        .iter()
        .filter_map(|(name, spec)| {
            matches!(
                dependency_provenance(name, spec, project_root),
                OfficialProvenance::UnofficialOverride
            )
            .then(|| name.clone())
        })
        .collect()
}

/// Official-name lockfile entries that fail registry provenance checks.
pub fn unofficial_official_overrides_from_lockfile(
    lockfile: &Lockfile,
    project_root: &Path,
) -> Vec<String> {
    // List lockfile official-name overrides without registry provenance.
    //
    // Parameters:
    // - `lockfile` — resolved dependency lockfile
    // - `project_root` — project directory for path resolution
    //
    // Returns:
    // Dependency keys that squat an official name without registry provenance.
    //
    // Options:
    // None.
    //
    // Example:
    // let squat = unofficial_official_overrides_from_lockfile(&lockfile, project_root);

    lockfile
        .dependencies
        .iter()
        .filter_map(|(name, dep)| {
            matches!(
                locked_dependency_provenance(name, dep, project_root),
                OfficialProvenance::UnofficialOverride
            )
            .then(|| name.clone())
        })
        .collect()
}

/// Load official package names for a project directory (prefers lockfile over manifest).
pub fn load_official_packages_for_project(root: &Path) -> PackageResult<Vec<String>> {
    // Load provenanced official package names for a project root.
    //
    // Parameters:
    // - `root` — project directory containing `spanda.toml` / `spanda.lock`
    //
    // Returns:
    // Official package names eligible for provider bootstrap.
    //
    // Options:
    // None.
    //
    // Example:
    // let names = load_official_packages_for_project(project_root)?;

    let lock_path = root.join(LOCKFILE_FILENAME);
    if lock_path.is_file() {
        let lockfile = Lockfile::load(&lock_path)?;
        return Ok(official_packages_from_lockfile(&lockfile, root));
    }
    let manifest_path = root.join(MANIFEST_FILENAME);
    if manifest_path.is_file() {
        let manifest = PackageManifest::load_from_dir(root)?;
        return Ok(official_packages_from_manifest(&manifest, root));
    }
    Err(PackageError::Manifest(format!(
        "no {MANIFEST_FILENAME} or {LOCKFILE_FILENAME} in {}",
        root.display()
    )))
}

/// Resolve official packages for a source file by walking up to the project root.
pub fn load_official_packages_for_source(source: &Path) -> Vec<String> {
    // Resolve provenanced official packages for a `.sd` file or directory.
    //
    // Parameters:
    // - `source` — program file or directory path
    //
    // Returns:
    // Official package names eligible for provider bootstrap (empty when unresolved).
    //
    // Options:
    // None.
    //
    // Example:
    // let names = load_official_packages_for_source(path);

    let start = if source.is_dir() {
        source.to_path_buf()
    } else {
        source
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    };
    let Some(root) = crate::manifest::find_project_root(&start) else {
        return Vec::new();
    };
    load_official_packages_for_project(&root).unwrap_or_default()
}

fn resolve_local_dependency_path(project_root: &Path, path: &Path) -> PathBuf {
    // Normalize a lockfile local path relative to the project root.

    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_root.join(path)
    }
}

fn local_path_is_canonical_registry(name: &str, resolved: &Path) -> bool {
    // Return true when a local path resolves to the canonical registry scaffold.

    let Some(expected) = registry_package_dir(name) else {
        return false;
    };
    match (resolved.canonicalize(), expected.canonicalize()) {
        (Ok(actual), Ok(canon)) => actual == canon,
        _ => resolved == expected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependency::{DependencyDetail, DependencySpec, LockedDependency, LockedSource};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn registry_version_dependency_is_provenanced() {
        let root = tempdir().unwrap();
        let spec = DependencySpec::Version("0.1.0".into());
        assert_eq!(
            dependency_provenance("spanda-mqtt", &spec, root.path()),
            OfficialProvenance::Registry
        );
    }

    #[test]
    fn path_override_of_official_name_is_not_provenanced() {
        let root = tempdir().unwrap();
        let evil = root.path().join("evil-mqtt");
        fs::create_dir_all(&evil).unwrap();
        let spec = DependencySpec::Detail(DependencyDetail {
            version: None,
            path: Some(evil),
            git: None,
            branch: None,
            tag: None,
            rev: None,
        });
        assert_eq!(
            dependency_provenance("spanda-mqtt", &spec, root.path()),
            OfficialProvenance::UnofficialOverride
        );
    }

    #[test]
    fn lockfile_registry_source_wires_official_providers() {
        let root = tempdir().unwrap();
        let lockfile = Lockfile {
            version: 1,
            package: crate::lockfile::LockPackageInfo {
                name: "demo".into(),
                version: "0.1.0".into(),
            },
            dependencies: [(
                "spanda-mqtt".into(),
                LockedDependency {
                    name: "spanda-mqtt".into(),
                    version: "0.1.0".into(),
                    source: LockedSource::Registry {
                        registry: "spanda".into(),
                    },
                    checksum: None,
                },
            )]
            .into(),
        };
        let names = official_packages_from_lockfile(&lockfile, root.path());
        assert_eq!(names, vec!["spanda-mqtt".to_string()]);
    }

    #[test]
    fn lockfile_path_override_does_not_wire_official_providers() {
        let root = tempdir().unwrap();
        let evil = root.path().join("evil-mqtt");
        fs::create_dir_all(&evil).unwrap();
        let lockfile = Lockfile {
            version: 1,
            package: crate::lockfile::LockPackageInfo {
                name: "demo".into(),
                version: "0.1.0".into(),
            },
            dependencies: [(
                "spanda-mqtt".into(),
                LockedDependency {
                    name: "spanda-mqtt".into(),
                    version: "0.1.0".into(),
                    source: LockedSource::Local { path: evil },
                    checksum: None,
                },
            )]
            .into(),
        };
        let names = official_packages_from_lockfile(&lockfile, root.path());
        assert!(names.is_empty());
        assert_eq!(
            unofficial_official_overrides_from_lockfile(&lockfile, root.path()),
            vec!["spanda-mqtt".to_string()]
        );
    }
}
