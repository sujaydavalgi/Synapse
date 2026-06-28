//! Static trust scoring for registry and vendored packages.
//!
use crate::lockfile::{Lockfile, LOCKFILE_FILENAME};
use crate::manifest::PackageManifest;
use crate::official::{
    dependency_provenance, is_official_package, locked_dependency_provenance,
    provenance_wires_official_providers, OfficialProvenance,
};
use crate::registry_remote::lookup_registry_entry;
use crate::registry_sign::{registry_trust_key, verify_registry_signature};
use crate::safety::SafetyLevel;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// One scored trust factor for a package.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustFactor {
    pub name: String,
    pub score: u32,
    pub max_score: u32,
    pub passed: bool,
    pub detail: String,
}

/// Trust evaluation report for a single package.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustScoreReport {
    pub package: String,
    pub version: String,
    pub score: u32,
    pub max_score: u32,
    pub passed: bool,
    pub tier: String,
    pub factors: Vec<TrustFactor>,
    pub recommendations: Vec<String>,
}

/// Evaluate trust for a registry or vendored package.
pub fn evaluate_package_trust(
    name: &str,
    version: Option<&str>,
    project_root: Option<&Path>,
) -> TrustScoreReport {
    // Score package integrity signals from registry metadata and local vendor tree.
    //
    // Parameters:
    // - `name` — package name
    // - `version` — optional version (latest registry version when omitted)
    // - `project_root` — optional project root for vendored manifest inspection
    //
    // Returns:
    // Trust score report with factor breakdown.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = evaluate_package_trust("spanda-mqtt", Some("0.1.0"), None);

    let mut factors = Vec::new();
    let mut recommendations = Vec::new();
    let lookup = lookup_registry_entry(name);
    let version = version
        .map(str::to_string)
        .or_else(|| {
            lookup
                .as_ref()
                .and_then(|entry| entry.versions().last().cloned())
        })
        .unwrap_or_else(|| "unknown".into());

    if lookup.is_some() {
        factors.push(factor(
            "registry_listed",
            20,
            20,
            true,
            "listed in Spanda registry",
        ));
    } else {
        factors.push(factor(
            "registry_listed",
            0,
            20,
            false,
            "package not found in registry",
        ));
        recommendations.push("Publish to registry or vendor from a trusted source".into());
    }

    if is_official_package(name) {
        let (official_score, official_passed, official_detail) = match project_dependency_provenance(
            name,
            project_root,
        ) {
            Some(OfficialProvenance::UnofficialOverride) => {
                recommendations.push(
                        "Use a registry version or path to packages/registry/<name> for official providers"
                            .into(),
                    );
                (
                    0,
                    false,
                    "official package name overridden by path/git source",
                )
            }
            Some(prov) if provenance_wires_official_providers(prov) => (
                15,
                true,
                "official Spanda framework package with registry provenance",
            ),
            Some(_) => (0, false, "official package without registry provenance"),
            None => (15, true, "official Spanda framework package"),
        };
        factors.push(factor(
            "official_framework",
            official_score,
            15,
            official_passed,
            official_detail,
        ));
    } else {
        factors.push(factor(
            "official_framework",
            0,
            15,
            false,
            "not an official framework package",
        ));
    }

    let license_ok = lookup.as_ref().is_some_and(|entry| match entry {
        crate::registry_remote::RegistryEntryLookup::Local(e) => {
            matches!(e.license, "Apache-2.0" | "MIT" | "BSD-3-Clause")
        }
        crate::registry_remote::RegistryEntryLookup::Remote(e) => {
            matches!(e.license.as_str(), "Apache-2.0" | "MIT" | "BSD-3-Clause")
        }
    });
    factors.push(factor(
        "license",
        if license_ok { 10 } else { 0 },
        10,
        license_ok,
        if license_ok {
            "permissive license"
        } else {
            "unknown or restrictive license"
        },
    ));

    let maintained = lookup
        .as_ref()
        .is_some_and(|entry| !entry.versions().is_empty());
    factors.push(factor(
        "maintained",
        if maintained { 10 } else { 0 },
        10,
        maintained,
        if maintained {
            format!(
                "{} published version(s)",
                lookup.as_ref().map(|e| e.versions().len()).unwrap_or(0)
            )
        } else {
            "no published versions".into()
        },
    ));

    let checksum = lookup
        .as_ref()
        .and_then(|entry| entry.version_sha256(&version));
    let has_checksum = checksum.is_some();
    factors.push(factor(
        "checksum",
        if has_checksum { 15 } else { 0 },
        15,
        has_checksum,
        if has_checksum {
            "SHA-256 checksum published".to_string()
        } else {
            "no published checksum".to_string()
        },
    ));
    if !has_checksum {
        recommendations.push("Publish checksum sidecar in registry index".into());
    }

    let signature = lookup
        .as_ref()
        .and_then(|entry| entry.version_signature(&version));
    let signed = signature
        .as_ref()
        .zip(checksum.as_deref())
        .is_some_and(|(sig, digest)| {
            let trust_key = registry_trust_key().unwrap_or_else(|| sig.public_key.clone());
            verify_registry_signature(name, &version, digest, sig, &trust_key)
        });
    factors.push(factor(
        "signed",
        if signed { 20 } else { 0 },
        20,
        signed,
        if signed {
            "Ed25519 registry signature verified".to_string()
        } else if signature.is_some() {
            "signature present but verification failed".to_string()
        } else {
            "not signed".to_string()
        },
    ));
    if !signed {
        recommendations.push("Sign registry tarball with registry trust key".into());
    }

    let safety_ok = vendored_safety_level(name, project_root)
        .is_some_and(|level| matches!(level, SafetyLevel::HardwareSafe | SafetyLevel::Certified));
    factors.push(factor(
        "safety_metadata",
        if safety_ok { 10 } else { 5 },
        10,
        safety_ok,
        vendored_safety_level(name, project_root)
            .map(|l| format!("safety level {}", l.as_str()))
            .unwrap_or_else(|| "no vendored safety metadata".into()),
    ));

    let score: u32 = factors.iter().map(|f| f.score).sum();
    let max_score: u32 = factors.iter().map(|f| f.max_score).sum();
    let passed = score >= 60;
    let tier = if score >= 85 {
        "trusted"
    } else if score >= 60 {
        "acceptable"
    } else {
        "low"
    }
    .to_string();
    TrustScoreReport {
        package: name.to_string(),
        version,
        score,
        max_score,
        passed,
        tier,
        factors,
        recommendations,
    }
}

fn factor(
    name: &str,
    score: u32,
    max_score: u32,
    passed: bool,
    detail: impl Into<String>,
) -> TrustFactor {
    TrustFactor {
        name: name.to_string(),
        score,
        max_score,
        passed,
        detail: detail.into(),
    }
}

fn vendored_safety_level(name: &str, project_root: Option<&Path>) -> Option<SafetyLevel> {
    let root = project_root?;
    let manifest = root.join(".spanda/packages").join(name).join("spanda.toml");
    if !manifest.exists() {
        return None;
    }
    let text = std::fs::read_to_string(&manifest).ok()?;
    let parsed = crate::manifest::PackageManifest::parse_str(&text).ok()?;
    Some(parsed.safety.level)
}

fn project_dependency_provenance(
    name: &str,
    project_root: Option<&Path>,
) -> Option<OfficialProvenance> {
    let root = project_root?;
    let lock_path = root.join(LOCKFILE_FILENAME);
    if lock_path.is_file() {
        if let Ok(lock) = Lockfile::load(&lock_path) {
            if let Some(dep) = lock.dependencies.get(name) {
                return Some(locked_dependency_provenance(name, dep, root));
            }
        }
    }
    if let Ok(manifest) = PackageManifest::load_from_dir(root) {
        if let Some(spec) = manifest.dependencies.get(name) {
            return Some(dependency_provenance(name, spec, root));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::registry_sign::{
        registry_trust_key, sign_registry_tarball, verify_registry_signature,
    };

    #[test]
    fn signed_factor_verifies_with_embedded_public_key() {
        let _guard = crate::testing::env_lock();
        unsafe {
            std::env::remove_var("SPANDA_REGISTRY_TRUST_KEY");
        }
        let digest = "abc123digest";
        let signed =
            sign_registry_tarball("demo-pkg", "0.1.0", digest, "registry-test-signing-key");
        let trust_key = registry_trust_key().unwrap_or_else(|| signed.public_key.clone());
        assert!(verify_registry_signature(
            "demo-pkg", "0.1.0", digest, &signed, &trust_key
        ));
    }
}
