//! registry support for Spanda.
//!
use crate::category::PackageCategory;
use crate::safety::SafetyLevel;
use serde::Serialize;

/// Stub registry entry for local resolution (no public registry yet).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RegistryEntry {
    pub name: &'static str,
    pub description: &'static str,
    pub versions: &'static [&'static str],
    pub category: PackageCategory,
    pub license: &'static str,
    pub import_paths: &'static [&'static str],
}

/// Local stub registry — framework packages available for dependency resolution.
pub static LOCAL_REGISTRY: &[RegistryEntry] = &[
    RegistryEntry {
        name: "spanda-ros2",
        description: "ROS 2 integration framework",
        versions: &["0.1.0", "0.2.0"],
        category: PackageCategory::Ros2,
        license: "Apache-2.0",
        import_paths: &["robotics.ros2"],
    },
    RegistryEntry {
        name: "spanda-vision",
        description: "Computer vision utilities",
        versions: &["0.1.0"],
        category: PackageCategory::Vision,
        license: "Apache-2.0",
        import_paths: &["vision.core"],
    },
    RegistryEntry {
        name: "spanda-navigation",
        description: "Path planning and navigation",
        versions: &["0.1.0"],
        category: PackageCategory::Navigation,
        license: "Apache-2.0",
        import_paths: &["navigation.path_planning"],
    },
    RegistryEntry {
        name: "spanda-mqtt",
        description: "MQTT pub/sub transport",
        versions: &["0.1.0"],
        category: PackageCategory::Mqtt,
        license: "Apache-2.0",
        import_paths: &["communication.mqtt"],
    },
    RegistryEntry {
        name: "spanda-lidar-rplidar",
        description: "RPLidar driver adapter",
        versions: &["0.1.0"],
        category: PackageCategory::Sensors,
        license: "MIT",
        import_paths: &["sensors.lidar.rplidar"],
    },
    RegistryEntry {
        name: "spanda-openai",
        description: "OpenAI provider adapter",
        versions: &["0.1.0"],
        category: PackageCategory::Ai,
        license: "Apache-2.0",
        import_paths: &["ai.openai"],
    },
    RegistryEntry {
        name: "spanda-opencv",
        description: "OpenCV vision adapter",
        versions: &["0.1.0"],
        category: PackageCategory::Vision,
        license: "Apache-2.0",
        import_paths: &["vision.opencv"],
    },
    RegistryEntry {
        name: "spanda-yolo",
        description: "YOLO object detection adapter",
        versions: &["0.1.0"],
        category: PackageCategory::Vision,
        license: "Apache-2.0",
        import_paths: &["vision.yolo"],
    },
    RegistryEntry {
        name: "spanda-slam",
        description: "SLAM localization framework",
        versions: &["0.1.0"],
        category: PackageCategory::Navigation,
        license: "Apache-2.0",
        import_paths: &["navigation.slam"],
    },
    RegistryEntry {
        name: "spanda-nav",
        description: "Navigation stack (alias for spanda-navigation)",
        versions: &["0.1.0"],
        category: PackageCategory::Navigation,
        license: "Apache-2.0",
        import_paths: &["navigation.path_planning"],
    },
    RegistryEntry {
        name: "spanda-ledger",
        description: "Mock ledger backend for audit hash anchoring",
        versions: &["0.1.0"],
        category: PackageCategory::Ledger,
        license: "Apache-2.0",
        import_paths: &["ledger.mock"],
    },
    RegistryEntry {
        name: "spanda-did",
        description: "Decentralized identity for robots and devices",
        versions: &["0.1.0"],
        category: PackageCategory::Identity,
        license: "Apache-2.0",
        import_paths: &["identity.core"],
    },
    RegistryEntry {
        name: "spanda-provenance",
        description: "Mission provenance and tamper-evident records",
        versions: &["0.1.0"],
        category: PackageCategory::Provenance,
        license: "Apache-2.0",
        import_paths: &["provenance.core"],
    },
    RegistryEntry {
        name: "spanda-supply-chain",
        description: "Supply-chain traceability for hardware components",
        versions: &["0.1.0"],
        category: PackageCategory::SupplyChain,
        license: "Apache-2.0",
        import_paths: &["supply_chain.trace"],
    },
    RegistryEntry {
        name: "spanda-python-bridge",
        description: "Python ecosystem orchestration bridge (PyTorch, OpenCV, NumPy)",
        versions: &["0.1.0"],
        category: PackageCategory::Ai,
        license: "Apache-2.0",
        import_paths: &["python.torch", "python.opencv"],
    },
    RegistryEntry {
        name: "spanda-cpp-bridge",
        description: "C++ ecosystem orchestration bridge (ROS2, PCL, CUDA)",
        versions: &["0.1.0"],
        category: PackageCategory::Ros2,
        license: "Apache-2.0",
        import_paths: &["cpp.ros2", "cpp.pcl"],
    },
];

pub fn search_registry(query: &str) -> Vec<&'static RegistryEntry> {
    // Search registry.
    //
    // Parameters:
    // - `query` — input value
    //
    // Returns:
    // Vec<&'static RegistryEntry>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::registry::search_registry(query);

    // Compute q for the following logic.
    let q = query.to_lowercase();
    LOCAL_REGISTRY
        .iter()
        .filter(|e| {
            e.name.contains(&q)
                || e.description.to_lowercase().contains(&q)
                || e.category.as_str().contains(&q)
        })
        .collect()
}

pub fn search_registry_merged(query: &str) -> Vec<String> {
    // Search registry merged.
    //
    // Parameters:
    // - `query` — input value
    //
    // Returns:
    // Vec<String>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::registry::search_registry_merged(query);

    // Create mutable names for accumulating results.
    let mut names: Vec<String> = search_registry(query)
        .into_iter()
        .map(|entry| entry.name.to_string())
        .collect();

    // Iterate over search remote registry.
    for remote in super::registry_remote::search_remote_registry(query) {
        // Take the branch when any equals name).
        if !names.iter().any(|name| name == &remote.name) {
            names.push(remote.name);
        }
    }
    names.sort_unstable();
    names
}

pub fn find_registry_entry(name: &str) -> Option<&'static RegistryEntry> {
    // Find registry entry.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::registry::find_registry_entry(name);

    // Iterate over LOCAL REGISTRY.
    LOCAL_REGISTRY.iter().find(|e| e.name == name)
}

/// Local source tree for a registry package (when shipped in-repo).
pub fn registry_package_dir(name: &str) -> Option<std::path::PathBuf> {
    // Registry package dir.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::registry::registry_package_dir(name);

    // Produce find registry entry as the result.
    find_registry_entry(name)?;
    let candidates = [
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/registry")
            .join(name),
        std::path::PathBuf::from("packages/registry").join(name),
    ];
    candidates.into_iter().find(|p| p.is_dir())
}

impl RegistryEntry {
    /// Default safety level for registry packages (until per-entry metadata is stored remotely).
    pub fn safety_level(&self) -> SafetyLevel {
        // Safety level.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // SafetyLevel.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.safety_level();

        // Match on name and handle each case.
        match self.name {
            "spanda-ros2" | "spanda-opencv" | "spanda-yolo" | "spanda-mqtt" => {
                SafetyLevel::SimulationOnly
            }
            "spanda-python-bridge" | "spanda-cpp-bridge" => SafetyLevel::HardwareSafe,
            "spanda-openai" => SafetyLevel::Experimental,
            _ => SafetyLevel::Experimental,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistryInfo {
    pub name: String,
    pub description: String,
    pub versions: Vec<String>,
    pub category: String,
    pub license: String,
    pub import_paths: Vec<String>,
    pub safety_level: String,
}

pub fn registry_info(name: &str) -> Option<RegistryInfo> {
    // Registry info.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::registry::registry_info(name);

    // Produce find registry entry as the result.
    find_registry_entry(name)
        .map(|e| RegistryInfo {
            name: e.name.to_string(),
            description: e.description.to_string(),
            versions: e.versions.iter().map(|v| v.to_string()).collect(),
            category: e.category.as_str().to_string(),
            license: e.license.to_string(),
            import_paths: e.import_paths.iter().map(|p| p.to_string()).collect(),
            safety_level: e.safety_level().as_str().to_string(),
        })
        .or_else(|| {
            super::registry_remote::find_remote_entry(name).map(|entry| {
                let safety_level = super::registry_remote::remote_safety_level(&entry.name)
                    .as_str()
                    .to_string();
                RegistryInfo {
                    name: entry.name,
                    description: entry.description,
                    versions: entry.versions,
                    category: entry.category,
                    license: entry.license,
                    import_paths: entry.import_paths,
                    safety_level,
                }
            })
        })
}

pub fn all_import_paths() -> Vec<&'static str> {
    // All import paths.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Vec<&'static str>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::registry::all_import_paths();

    let mut paths: Vec<&'static str> = LOCAL_REGISTRY
        .iter()
        .flat_map(|e| e.import_paths.iter().copied())
        .collect();
    paths.extend(
        super::adapter::framework_packages()
            .iter()
            .flat_map(|p| p.import_paths.iter().copied()),
    );
    paths.sort_unstable();
    paths.dedup();
    paths
}
