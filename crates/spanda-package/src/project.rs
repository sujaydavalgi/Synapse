//! project support for Spanda.
//!
use crate::error::{PackageError, PackageResult};
use crate::manifest::{PackageManifest, MANIFEST_FILENAME};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_MAIN: &str = r#"module main;

import std.robotics;
import std.sensors;

robot WarehouseBot {
    sensor front_lidar: LidarScan;
    actuator drive: MotionCommand;

    behavior navigate {

    // TODO: implement navigation
    }
}
"#;

const DEFAULT_README: &str = r#"# Spanda Package

Created with `spanda init`.

## Commands

- `spanda check` — type-check sources
- `spanda build` — compile the project
- `spanda test` — run tests
- `spanda install` — resolve dependencies and write spanda.lock
"#;

/// Initialize a new Spanda package in `dir`.
pub fn init_package(
    dir: &Path,
    name: Option<&str>,
    description: Option<&str>,
) -> PackageResult<PathBuf> {
    // Description:
    //     Init package.
    //
    // Inputs:
    //     dir: &Path
    //         Caller-supplied dir.
    //     name: Option<&str>
    //         Caller-supplied name.
    //     description: Option<&str>
    //         Caller-supplied description.
    //
    // Outputs:
    //     result: PackageResult<PathBuf>
    //         Return value from `init_package`.
    //
    // Example:
    //     let result = spanda_package::project::init_package(dir, name, description);

    // Compute pkg name for the following logic.
    let pkg_name = name
        .map(str::to_string)
        .or_else(|| dir.file_name().and_then(|n| n.to_str()).map(str::to_string))
        .unwrap_or_else(|| "my_robot".into());
    fs::create_dir_all(dir).map_err(PackageError::from)?;
    fs::create_dir_all(dir.join("src")).map_err(PackageError::from)?;
    fs::create_dir_all(dir.join("tests")).map_err(PackageError::from)?;
    let manifest = PackageManifest {
        package: crate::manifest::PackageSection {
            name: pkg_name.clone(),
            version: "0.1.0".into(),
            description: description.map(str::to_string),
            license: Some("Apache-2.0".into()),
            authors: vec![],
        },
        dependencies: Default::default(),
        dev_dependencies: Default::default(),
        hardware: Default::default(),
        capabilities: Default::default(),
        requires_hardware: Default::default(),
        safety: Default::default(),
        adapter: Default::default(),
        categories: vec![],
        license_compat: vec![],
        entity_kinds: vec![],
    };
    manifest.save(&dir.join(MANIFEST_FILENAME))?;
    fs::write(dir.join("src/main.sd"), DEFAULT_MAIN).map_err(PackageError::from)?;
    fs::write(dir.join("README.md"), DEFAULT_README).map_err(PackageError::from)?;
    Ok(dir.to_path_buf())
}

/// Collect `.sd` source files from a project (src/ and tests/).
pub fn collect_source_files(project_root: &Path) -> PackageResult<Vec<PathBuf>> {
    // Description:
    //     Collect source files.
    //
    // Inputs:
    //     project_roo: &Path
    //         Caller-supplied project roo.
    //
    // Outputs:
    //     result: PackageResult<Vec<PathBuf>>
    //         Return value from `collect_source_files`.
    //
    // Example:
    //     let result = spanda_package::project::collect_source_files(project_roo);

    // Create mutable files for accumulating results.
    let mut files = Vec::new();

    // Iterate over ["src", "tests"].
    for sub in ["src", "tests"] {
        let dir = project_root.join(sub);

        // Treat the path as a directory and scan its contents.
        if dir.is_dir() {
            collect_sd_files(&dir, &mut files)?;
        }
    }

    // Skip further work when files is empty.
    if files.is_empty() {
        // Handle the success value from read dir.
        if let Ok(entries) = fs::read_dir(project_root) {
            // Process each registry entry.
            for entry in entries.flatten() {
                let path = entry.path();

                // Take the branch when is some and equals "sd").
                if path.extension().is_some_and(|e| e == "sd") {
                    files.push(path);
                }
            }
        }
    }
    Ok(files)
}

fn collect_sd_files(dir: &Path, out: &mut Vec<PathBuf>) -> PackageResult<()> {
    // Description:
    //     Collect sd files.
    //
    // Inputs:
    //     dir: &Path
    //         Caller-supplied dir.
    //     o: &mut Vec<PathBuf>
    //         Caller-supplied o.
    //
    // Outputs:
    //     result: PackageResult<()>
    //         Return value from `collect_sd_files`.
    //
    // Example:
    //     let result = spanda_package::project::collect_sd_files(dir, o);

    // Process each registry entry.
    for entry in fs::read_dir(dir).map_err(PackageError::from)? {
        let entry = entry.map_err(PackageError::from)?;
        let path = entry.path();

        // Treat the path as a directory and scan its contents.
        if path.is_dir() {
            collect_sd_files(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "sd") {
            out.push(path);
        }
    }
    Ok(())
}

/// Add a dependency to the manifest and save.
pub fn add_dependency(
    project_root: &Path,
    name: &str,
    spec: crate::dependency::DependencySpec,
) -> PackageResult<()> {
    // Description:
    //     Add dependency.
    //
    // Inputs:
    //     project_roo: &Path
    //         Caller-supplied project roo.
    //     name: &str
    //         Caller-supplied name.
    //     spec: crate::dependency::DependencySpec
    //         Caller-supplied spec.
    //
    // Outputs:
    //     result: PackageResult<()>
    //         Return value from `add_dependency`.
    //
    // Example:
    //     let result = spanda_package::project::add_dependency(project_roo, name, spec);

    // Compute manifest path for the following logic.
    let manifest_path = project_root.join(MANIFEST_FILENAME);
    let mut manifest = PackageManifest::load(&manifest_path)?;
    manifest.dependencies.insert(name.to_string(), spec);
    manifest.save(&manifest_path)
}

/// Remove a dependency from the manifest and save.
pub fn remove_dependency(project_root: &Path, name: &str) -> PackageResult<bool> {
    // Description:
    //     Remove dependency.
    //
    // Inputs:
    //     project_roo: &Path
    //         Caller-supplied project roo.
    //     name: &str
    //         Caller-supplied name.
    //
    // Outputs:
    //     result: PackageResult<bool>
    //         Return value from `remove_dependency`.
    //
    // Example:
    //     let result = spanda_package::project::remove_dependency(project_roo, name);

    // Compute manifest path for the following logic.
    let manifest_path = project_root.join(MANIFEST_FILENAME);
    let mut manifest = PackageManifest::load(&manifest_path)?;
    let removed = manifest.dependencies.remove(name).is_some();

    // Take this path when removed.
    if removed {
        manifest.save(&manifest_path)?;
    }
    Ok(removed)
}
