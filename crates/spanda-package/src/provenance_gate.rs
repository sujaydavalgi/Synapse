//! Deploy-time provenance checks for production gates.
//!
use crate::dependency::LockedSource;
use crate::lockfile::{Lockfile, LOCKFILE_FILENAME};
use crate::manifest::{PackageManifest, MANIFEST_FILENAME};
use crate::official::{
    unofficial_official_overrides_from_lockfile, unofficial_official_overrides_from_manifest,
};
use crate::registry_remote::{find_remote_entry, RemoteRegistryEntry};
use crate::registry_sign::{
    registry_require_signature, registry_trust_key, verify_registry_signature,
};
use std::fs;
use std::path::{Path, PathBuf};

/// Outcome of project-level provenance checks used by deploy gates.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProjectProvenanceGateReport {
    pub official_overrides: Vec<String>,
    pub registry_signature_failures: Vec<String>,
    pub registry_signature_env_enabled: bool,
}

impl ProjectProvenanceGateReport {
    pub fn passed_official_provenance(&self) -> bool {
        self.official_overrides.is_empty()
    }

    pub fn passed_registry_signatures(&self) -> bool {
        self.registry_signature_env_enabled && self.registry_signature_failures.is_empty()
    }
}

/// Evaluate official-name squatting and registry signature posture for a project.
pub fn evaluate_project_provenance_gate(project_root: &Path) -> ProjectProvenanceGateReport {
    // Run deploy-time provenance checks for a project directory.
    //
    // Parameters:
    // - `project_root` — directory containing `spanda.toml` / `spanda.lock`
    //
    // Returns:
    // Provenance gate report with override and signature failures.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = evaluate_project_provenance_gate(project_root);

    let lock_path = project_root.join(LOCKFILE_FILENAME);
    let official_overrides = if lock_path.is_file() {
        Lockfile::load(&lock_path)
            .map(|lock| unofficial_official_overrides_from_lockfile(&lock, project_root))
            .unwrap_or_default()
    } else if project_root.join(MANIFEST_FILENAME).is_file() {
        PackageManifest::load_from_dir(project_root)
            .map(|manifest| unofficial_official_overrides_from_manifest(&manifest, project_root))
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let registry_signature_env_enabled = registry_require_signature();
    let mut registry_signature_failures = Vec::new();
    if !registry_signature_env_enabled {
        registry_signature_failures.push(
            "SPANDA_REGISTRY_REQUIRE_SIGNATURE is not enabled (set to 1 for production)".into(),
        );
    }
    if lock_path.is_file() {
        if let Ok(lockfile) = Lockfile::load(&lock_path) {
            registry_signature_failures.extend(registry_lockfile_signature_failures(
                project_root,
                &lockfile,
            ));
        } else {
            registry_signature_failures
                .push("failed to parse spanda.lock for signature audit".into());
        }
    } else {
        registry_signature_failures
            .push("spanda.lock required for registry signature audit".into());
    }

    ProjectProvenanceGateReport {
        official_overrides,
        registry_signature_failures,
        registry_signature_env_enabled,
    }
}

/// Verify registry signatures for lockfile registry dependencies.
pub fn registry_lockfile_signature_failures(
    project_root: &Path,
    lockfile: &Lockfile,
) -> Vec<String> {
    // Return human-readable failures for unsigned or invalid registry dependencies.
    //
    // Parameters:
    // - `project_root` — project directory (used to locate hosted `registry/index.json`)
    // - `lockfile` — resolved dependency lockfile
    //
    // Returns:
    // Failure messages (empty when all registry deps verify).
    //
    // Options:
    // None.
    //
    // Example:
    // let failures = registry_lockfile_signature_failures(project_root, &lockfile);

    let mut failures = Vec::new();
    for (name, dep) in &lockfile.dependencies {
        if !matches!(dep.source, LockedSource::Registry { .. }) {
            continue;
        }
        let version = dep.version.as_str();
        let Some((digest, signature)) = registry_version_artifacts(name, version, project_root)
        else {
            failures.push(format!(
                "{name}@{version}: missing registry checksum/signature metadata"
            ));
            continue;
        };
        if let Some(locked_checksum) = dep.checksum.as_deref() {
            if locked_checksum != digest {
                failures.push(format!(
                    "{name}@{version}: lockfile checksum does not match registry index"
                ));
                continue;
            }
        }
        let trust_key = registry_trust_key().unwrap_or_else(|| signature.public_key.clone());
        if !verify_registry_signature(name, version, &digest, &signature, &trust_key) {
            failures.push(format!("{name}@{version}: invalid registry signature"));
        }
    }
    failures
}

fn registry_version_artifacts(
    name: &str,
    version: &str,
    project_root: &Path,
) -> Option<(String, crate::registry_sign::RegistryVersionSignature)> {
    // Resolve checksum and signature for one registry package version.

    if let Some(entry) =
        find_hosted_registry_entry(name, project_root).or_else(|| find_remote_entry(name))
    {
        let digest = entry.version_checksums.get(version)?.clone();
        let signature = entry.version_signatures.get(version)?.clone();
        return Some((digest, signature));
    }
    None
}

fn find_hosted_registry_entry(name: &str, project_root: &Path) -> Option<RemoteRegistryEntry> {
    // Look up a package entry from a local hosted `registry/index.json` when present.

    load_hosted_registry_index(project_root)?
        .into_iter()
        .find(|entry| entry.name == name)
}

fn load_hosted_registry_index(project_root: &Path) -> Option<Vec<RemoteRegistryEntry>> {
    // Load `registry/index.json` from well-known monorepo-relative paths.

    for path in hosted_registry_index_candidates(project_root) {
        if !path.is_file() {
            continue;
        }
        let body = fs::read_to_string(&path).ok()?;
        return serde_json::from_str(&body).ok();
    }
    None
}

fn hosted_registry_index_candidates(project_root: &Path) -> Vec<PathBuf> {
    // Collect candidate paths to a hosted registry index file.

    let mut paths = vec![
        project_root.join("registry/index.json"),
        project_root.join("../registry/index.json"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../registry/index.json"),
        PathBuf::from("registry/index.json"),
    ];
    if let Ok(root) = project_root.canonicalize() {
        paths.push(root.join("registry/index.json"));
        if let Some(parent) = root.parent() {
            paths.push(parent.join("registry/index.json"));
        }
    }
    paths.sort_unstable();
    paths.dedup();
    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependency::{LockedDependency, LockedSource};
    use crate::lockfile::LockPackageInfo;
    use tempfile::tempdir;

    #[test]
    fn path_override_fails_official_provenance_gate() {
        let root = tempdir().unwrap();
        let evil = root.path().join("evil-mqtt");
        fs::create_dir_all(&evil).unwrap();
        fs::write(
            root.path().join(MANIFEST_FILENAME),
            format!(
                r#"
[package]
name = "demo"
version = "0.1.0"

[dependencies]
spanda-mqtt = {{ path = "{}" }}
"#,
                evil.display()
            ),
        )
        .unwrap();
        let report = evaluate_project_provenance_gate(root.path());
        assert!(!report.passed_official_provenance());
        assert!(report
            .official_overrides
            .contains(&"spanda-mqtt".to_string()));
    }

    #[test]
    fn production_signature_gate_requires_env_and_lockfile() {
        let root = tempdir().unwrap();
        fs::write(
            root.path().join(MANIFEST_FILENAME),
            r#"
[package]
name = "demo"
version = "0.1.0"
"#,
        )
        .unwrap();
        let report = evaluate_project_provenance_gate(root.path());
        assert!(!report.passed_registry_signatures());
        assert!(!report.registry_signature_env_enabled);
    }

    #[test]
    fn hosted_registry_lockfile_signatures_verify_for_spanda_mqtt() {
        let index_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../registry/index.json");
        if !index_path.is_file() {
            return;
        }
        let root = tempdir().unwrap();
        let lockfile = Lockfile {
            version: 1,
            package: LockPackageInfo {
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
        let failures = registry_lockfile_signature_failures(root.path(), &lockfile);
        assert!(
            failures.is_empty(),
            "expected hosted registry signatures to verify: {failures:?}"
        );
    }
}
