use crate::dependency::{
    parse_version, DependencySourceKind, DependencySpec, LockedDependency, LockedSource,
};
use crate::error::{PackageError, PackageResult};
use crate::manifest::{PackageManifest, MANIFEST_FILENAME};
use crate::registry_remote::{lookup_registry_entry, RegistryEntryLookup};
use semver::Version;
use std::collections::BTreeMap;
use std::path::Path;

/// Options for dependency resolution.
#[derive(Debug, Clone, Default)]
pub struct ResolveOptions {
    pub offline: bool,
}

/// Result of resolving all dependencies for a project.
#[derive(Debug, Clone)]
pub struct ResolveResult {
    pub lockfile_deps: BTreeMap<String, LockedDependency>,
    pub warnings: Vec<String>,
}

pub fn resolve_dependencies(
    project_root: &Path,
    manifest: &PackageManifest,
    options: &ResolveOptions,
) -> PackageResult<ResolveResult> {
    let mut lockfile_deps = BTreeMap::new();
    let mut warnings = Vec::new();

    for (name, spec) in manifest.all_dependencies() {
        let locked = resolve_one(project_root, name, spec, options)?;
        lockfile_deps.insert(name.to_string(), locked);
    }

    if options.offline {
        warnings.push("resolved in offline mode — registry packages use cached versions".into());
    }

    Ok(ResolveResult {
        lockfile_deps,
        warnings,
    })
}

fn resolve_one(
    project_root: &Path,
    name: &str,
    spec: &DependencySpec,
    _options: &ResolveOptions,
) -> PackageResult<LockedDependency> {
    match spec.source_kind() {
        DependencySourceKind::Local => {
            let path = spec.local_path(project_root).ok_or_else(|| {
                PackageError::Dependency(format!("local path missing for {name}"))
            })?;
            let dep_manifest = load_dep_manifest(&path)?;
            Ok(LockedDependency {
                name: name.to_string(),
                version: dep_manifest.package.version.clone(),
                source: LockedSource::Local { path },
                checksum: None,
            })
        }
        DependencySourceKind::Git => {
            let detail = match spec {
                DependencySpec::Detail(d) => d,
                _ => {
                    return Err(PackageError::Dependency(format!(
                        "git dependency '{name}' requires inline table"
                    )));
                }
            };
            let url = detail
                .git
                .clone()
                .ok_or_else(|| PackageError::Dependency(format!("git URL missing for {name}")))?;
            let version = detail
                .tag
                .clone()
                .or_else(|| detail.rev.clone())
                .unwrap_or_else(|| "0.0.0-git".into());
            Ok(LockedDependency {
                name: name.to_string(),
                version,
                source: LockedSource::Git {
                    url,
                    branch: detail.branch.clone(),
                    tag: detail.tag.clone(),
                    rev: detail.rev.clone(),
                },
                checksum: None,
            })
        }
        DependencySourceKind::Registry => {
            let lookup = lookup_registry_entry(name).ok_or_else(|| {
                PackageError::Dependency(format!(
                    "registry package '{name}' not found — add to local registry, set SPANDA_REGISTRY_URL, or use path/git"
                ))
            })?;
            let version_req = spec.parse_version_req()?.ok_or_else(|| {
                PackageError::Dependency(format!("version constraint required for {name}"))
            })?;
            let resolved = select_registry_version(&lookup, &version_req)?;
            Ok(LockedDependency {
                name: name.to_string(),
                version: resolved.to_string(),
                source: LockedSource::Registry {
                    registry: lookup.registry_label().into(),
                },
                checksum: None,
            })
        }
    }
}

fn load_dep_manifest(path: &Path) -> PackageResult<PackageManifest> {
    let manifest_path = path.join(MANIFEST_FILENAME);
    if !manifest_path.is_file() {
        return Err(PackageError::Dependency(format!(
            "dependency at {} has no spanda.toml",
            path.display()
        )));
    }
    PackageManifest::load(&manifest_path)
}

fn select_registry_version(
    entry: &RegistryEntryLookup,
    req: &semver::VersionReq,
) -> PackageResult<Version> {
    let mut candidates: Vec<Version> = entry
        .versions()
        .iter()
        .filter_map(|v| parse_version(v).ok())
        .filter(|v| req.matches(v))
        .collect();
    candidates.sort();
    candidates.pop().ok_or_else(|| {
        PackageError::Dependency(format!(
            "no version of '{}' satisfies constraint {}",
            entry.name(),
            req
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependency::DependencyDetail;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn write_manifest(dir: &Path, name: &str, version: &str) {
        let content = format!(
            r#"
[package]
name = "{name}"
version = "{version}"
"#
        );
        fs::write(dir.join(MANIFEST_FILENAME), content).unwrap();
    }

    #[test]
    fn resolves_local_dependency() {
        let root = TempDir::new().unwrap();
        let lib = root.path().join("lib");
        fs::create_dir(&lib).unwrap();
        write_manifest(&lib, "my_lib", "0.2.0");
        write_manifest(root.path(), "app", "0.1.0");

        let mut manifest = PackageManifest::load_from_dir(root.path()).unwrap();
        manifest.dependencies.insert(
            "my_lib".into(),
            DependencySpec::Detail(DependencyDetail {
                version: None,
                path: Some(PathBuf::from("lib")),
                git: None,
                branch: None,
                tag: None,
                rev: None,
            }),
        );

        let result =
            resolve_dependencies(root.path(), &manifest, &ResolveOptions::default()).unwrap();
        let locked = result.lockfile_deps.get("my_lib").unwrap();
        assert_eq!(locked.version, "0.2.0");
        assert!(matches!(locked.source, LockedSource::Local { .. }));
    }

    #[test]
    fn resolves_registry_dependency() {
        let root = TempDir::new().unwrap();
        write_manifest(root.path(), "app", "0.1.0");

        let mut manifest = PackageManifest::load_from_dir(root.path()).unwrap();
        manifest.dependencies.insert(
            "spanda-ros2".into(),
            DependencySpec::Version("0.1.0".into()),
        );

        let result =
            resolve_dependencies(root.path(), &manifest, &ResolveOptions::default()).unwrap();
        let locked = result.lockfile_deps.get("spanda-ros2").unwrap();
        assert_eq!(locked.version, "0.1.0");
    }

    #[test]
    fn parses_git_dependency() {
        let root = TempDir::new().unwrap();
        write_manifest(root.path(), "app", "0.1.0");

        let mut manifest = PackageManifest::load_from_dir(root.path()).unwrap();
        manifest.dependencies.insert(
            "spanda-nav".into(),
            DependencySpec::Detail(DependencyDetail {
                version: None,
                path: None,
                git: Some("https://github.com/spanda/spanda-nav".into()),
                branch: Some("main".into()),
                tag: None,
                rev: None,
            }),
        );

        let result =
            resolve_dependencies(root.path(), &manifest, &ResolveOptions::default()).unwrap();
        let locked = result.lockfile_deps.get("spanda-nav").unwrap();
        assert!(matches!(locked.source, LockedSource::Git { .. }));
    }
}
