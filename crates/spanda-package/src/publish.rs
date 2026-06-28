//! Package bundle creation and optional remote registry upload.

use crate::error::{PackageError, PackageResult};
use crate::integrity::write_checksum_sidecar;
use crate::manifest::{PackageManifest, MANIFEST_FILENAME};
use crate::project::collect_source_files;
use crate::registry_remote::{fetch_index_json, registry_base_url, RemoteRegistryEntry};
use crate::registry_sign::{registry_sign_key, sign_registry_tarball};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishReport {
    pub bundle_path: PathBuf,
    pub uploaded: bool,
    pub upload_url: Option<String>,
    pub sha256: Option<String>,
    pub signature: Option<crate::registry_sign::RegistryVersionSignature>,
}

/// Create a `.tar.gz` bundle containing manifest, lockfile, and source files.
pub fn bundle_package(root: &Path, manifest: &PackageManifest) -> PackageResult<PublishReport> {
    // Description:
    //     Bundle package.
    //
    // Inputs:
    //     roo: &Path
    //         Caller-supplied roo.
    //     anifes: &PackageManifest
    //         Caller-supplied anifes.
    //
    // Outputs:
    //     result: PackageResult<PublishReport>
    //         Return value from `bundle_package`.
    //
    // Example:
    //     let result = spanda_package::publish::bundle_package(roo, anifes);

    // Compute sources for the following logic.
    let sources = collect_source_files(root)?;

    // Skip further work when sources is empty.
    if sources.is_empty() {
        return Err(PackageError::Validation(
            "no source files to publish".into(),
        ));
    }
    let dist = root.join("dist");
    fs::create_dir_all(&dist).map_err(PackageError::from)?;
    let bundle_name = format!(
        "{}-{}.tar.gz",
        manifest.package.name, manifest.package.version
    );
    let bundle_path = dist.join(bundle_name);
    let mut paths = vec![root.join(MANIFEST_FILENAME)];
    let lock = root.join(crate::lockfile::LOCKFILE_FILENAME);

    // Act only when the target path already exists.
    if lock.exists() {
        paths.push(lock);
    }
    paths.extend(sources);
    create_tar_gz(&bundle_path, root, &paths)?;
    let sha256 = write_checksum_sidecar(&bundle_path).ok();
    let signature = sha256.as_deref().and_then(|digest| {
        registry_sign_key().map(|key| {
            sign_registry_tarball(
                &manifest.package.name,
                &manifest.package.version,
                digest,
                &key,
            )
        })
    });
    Ok(PublishReport {
        bundle_path,
        uploaded: false,
        upload_url: None,
        sha256,
        signature,
    })
}

/// Bundle the package and optionally upload to `SPANDA_REGISTRY_URL`.
pub fn publish_package(root: &Path, manifest: &PackageManifest) -> PackageResult<PublishReport> {
    // Description:
    //     Publish package.
    //
    // Inputs:
    //     roo: &Path
    //         Caller-supplied roo.
    //     anifes: &PackageManifest
    //         Caller-supplied anifes.
    //
    // Outputs:
    //     result: PackageResult<PublishReport>
    //         Return value from `publish_package`.
    //
    // Example:
    //     let result = spanda_package::publish::publish_package(roo, anifes);

    // Create mutable report for accumulating results.
    let mut report = bundle_package(root, manifest)?;

    if let Some(mirror_path) = mirror_bundle_to_local_registry(root, manifest, &report.bundle_path)?
    {
        eprintln!("Registry mirror: wrote {}", mirror_path.display());
    }

    // Emit output when registry base url provides a base.
    if let Some(base) = registry_base_url() {
        let url = format!(
            "{base}/packages/{}/{}",
            manifest.package.name, manifest.package.version
        );

        // Match on bundle path, &url) and handle each case.
        match upload_bundle(&report.bundle_path, &url) {
            Ok(()) => {
                report.uploaded = true;
                report.upload_url = Some(url);

                // Handle the error returned from update registry index.
                if let Err(err) = update_registry_index(
                    &base,
                    manifest,
                    report.sha256.as_deref(),
                    report.signature.as_ref(),
                ) {
                    eprintln!("Warning: registry index update failed: {err}");
                }
            }
            Err(err) => {
                eprintln!("Warning: registry upload failed: {err}");
                eprintln!("  Bundle written to {}", report.bundle_path.display());
            }
        }
    }
    Ok(report)
}

/// Copy a published bundle into the repo-local registry mirror when configured.
pub fn mirror_bundle_to_local_registry(
    root: &Path,
    manifest: &PackageManifest,
    bundle: &Path,
) -> PackageResult<Option<PathBuf>> {
    // Description:
    //     Mirror bundle to local registry.
    //
    // Inputs:
    //     roo: &Path
    //         Caller-supplied roo.
    //     anifes: &PackageManifest
    //         Caller-supplied anifes.
    //     bundle: &Path
    //         Caller-supplied bundle.
    //
    // Outputs:
    //     result: PackageResult<Option<PathBuf>>
    //         Return value from `mirror_bundle_to_local_registry`.
    //
    // Example:

    //     let result = spanda_package::publish::mirror_bundle_to_local_registry(roo, anifes, bundle);

    let mirror_root = std::env::var("SPANDA_REGISTRY_MIRROR")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            let candidate = root.join("registry/packages");
            candidate.is_dir().then_some(candidate)
        })
        .or_else(|| {
            root.ancestors().find_map(|ancestor| {
                let candidate = ancestor.join("registry/packages");
                candidate.is_dir().then_some(candidate)
            })
        });
    let Some(mirror_root) = mirror_root else {
        return Ok(None);
    };
    let dest_dir = mirror_root.join(&manifest.package.name);
    fs::create_dir_all(&dest_dir).map_err(PackageError::from)?;
    let dest = dest_dir.join(&manifest.package.version);
    fs::copy(bundle, &dest).map_err(PackageError::from)?;
    Ok(Some(dest))
}

fn create_tar_gz(output: &Path, root: &Path, files: &[PathBuf]) -> PackageResult<()> {
    // Description:
    //     Create tar gz.
    //
    // Inputs:
    //     outp: &Path
    //         Caller-supplied outp.
    //     roo: &Path
    //         Caller-supplied roo.
    //     files: &[PathBuf]
    //         Caller-supplied files.
    //
    // Outputs:
    //     result: PackageResult<()>
    //         Return value from `create_tar_gz`.
    //
    // Example:
    //     let result = spanda_package::publish::create_tar_gz(outp, roo, files);

    // Compute rel paths for the following logic.
    let rel_paths: Vec<String> = files
        .iter()
        .filter_map(|path| path.strip_prefix(root).ok())
        .map(|path| path.to_string_lossy().into_owned())
        .collect();
    let status = Command::new("tar")
        .arg("-czf")
        .arg(output)
        .arg("-C")
        .arg(root)
        .args(&rel_paths)
        .status()
        .map_err(PackageError::from)?;

    // Handle output when the subprocess succeeds.
    if status.success() {
        Ok(())
    } else {
        Err(PackageError::Validation(format!(
            "tar failed creating bundle (exit {status})"
        )))
    }
}

fn upload_bundle(bundle: &Path, url: &str) -> Result<(), String> {
    // Description:
    //     Upload bundle.
    //
    // Inputs:
    //     bundle: &Path
    //         Caller-supplied bundle.
    //     url: &str
    //         Caller-supplied url.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `upload_bundle`.
    //
    // Example:
    //     let result = spanda_package::publish::upload_bundle(bundle, rl);

    // Start building the generated output buffer.
    let output = Command::new("curl")
        .args([
            "-fsSL",
            "-X",
            "PUT",
            "-H",
            "Content-Type: application/gzip",
            "--data-binary",
            &format!("@{}", bundle.display()),
            url,
        ])
        .output()
        .map_err(|e| format!("curl not available: {e}"))?;

    // Handle output when the subprocess succeeds.
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "upload failed (exit {}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn update_registry_index(
    base: &str,
    manifest: &PackageManifest,
    sha256: Option<&str>,
    signature: Option<&crate::registry_sign::RegistryVersionSignature>,
) -> Result<(), String> {
    // Description:
    //     Update registry index.
    //
    // Inputs:
    //     base: &str
    //         Caller-supplied base.
    //     anifes: &PackageManifest
    //         Caller-supplied anifes.
    //     sha256: Option<&str>
    //         Caller-supplied sha256.
    //     signature: Option<&crate::registry_sign::RegistryVersionSignature>
    //         Caller-supplied signature.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `update_registry_index`.
    //
    // Example:
    //     let result = spanda_package::publish::update_registry_index(base, anifes, sha256, signature);

    // Compute index url for the following logic.
    let index_url = format!("{base}/index.json");
    let body = fetch_index_json(&index_url).unwrap_or_else(|_| "[]".to_string());
    let mut entries: Vec<RemoteRegistryEntry> = serde_json::from_str(&body).unwrap_or_default();
    let version = manifest.package.version.clone();
    let description = manifest
        .package
        .description
        .clone()
        .unwrap_or_else(|| manifest.package.name.clone());

    // Emit output when entries provides a existing.
    if let Some(existing) = entries
        .iter_mut()
        .find(|entry| entry.name == manifest.package.name)
    {
        // Check membership before continuing.
        if !existing.versions.contains(&version) {
            existing.versions.push(version.clone());
        }
        existing.description = description;
        if let Some(digest) = sha256 {
            existing
                .version_checksums
                .insert(version.clone(), digest.to_string());
        }
        if let Some(sig) = signature {
            existing
                .version_signatures
                .insert(version.clone(), sig.clone());
        }
    } else {
        let mut version_checksums = std::collections::BTreeMap::new();
        if let Some(digest) = sha256 {
            version_checksums.insert(version.clone(), digest.to_string());
        }
        let mut version_signatures = std::collections::BTreeMap::new();
        if let Some(sig) = signature {
            version_signatures.insert(version.clone(), sig.clone());
        }
        entries.push(RemoteRegistryEntry {
            name: manifest.package.name.clone(),
            description,
            versions: vec![version],
            category: "robotics".into(),
            license: manifest
                .package
                .license
                .clone()
                .unwrap_or_else(|| "Apache-2.0".into()),
            import_paths: vec![],
            version_checksums,
            version_signatures,
        });
    }
    let json = serde_json::to_string_pretty(&entries).map_err(|e| e.to_string())?;
    upload_json(&index_url, &json)
}

fn upload_json(url: &str, body: &str) -> Result<(), String> {
    // Description:
    //     Upload json.
    //
    // Inputs:
    //     url: &str
    //         Caller-supplied url.
    //     body: &str
    //         Caller-supplied body.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `upload_json`.
    //
    // Example:
    //     let result = spanda_package::publish::upload_json(rl, body);

    // Start building the generated output buffer.
    let output = Command::new("curl")
        .args([
            "-fsSL",
            "-X",
            "PUT",
            "-H",
            "Content-Type: application/json",
            "--data-binary",
            "@-",
            url,
        ])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;

            // Emit output when as mut provides a stdin.
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(body.as_bytes())?;
            }
            child.wait_with_output()
        })
        .map_err(|e| format!("curl not available: {e}"))?;

    // Handle output when the subprocess succeeds.
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "index upload failed (exit {}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::PackageSection;
    use std::collections::HashMap;

    #[test]
    fn bundles_manifest_and_sources() {
        // Description:
        //     Bundles manifest and sources.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_package::publish::bundles_manifest_and_sources();

        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        fs::create_dir_all(root.join("src")).expect("src");
        fs::write(
            root.join(MANIFEST_FILENAME),
            "[package]\nname = \"demo-pkg\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(root.join("src/main.sd"), "robot R { behavior run() {} }").unwrap();
        let manifest = PackageManifest {
            package: PackageSection {
                name: "demo-pkg".into(),
                version: "0.1.0".into(),
                description: None,
                license: None,
                authors: vec![],
            },
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            hardware: Default::default(),
            capabilities: Default::default(),
            requires_hardware: Default::default(),
            safety: Default::default(),
            adapter: Default::default(),
            categories: vec![],
            license_compat: vec![],
            entity_kinds: vec![],
        };
        let report = bundle_package(root, &manifest).expect("bundle");
        assert!(report.bundle_path.exists());
        assert!(report.sha256.is_some());
        assert!(!report.uploaded);
    }
}
