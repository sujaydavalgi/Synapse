//! adapter support for Spanda.
//!
use serde::{Deserialize, Serialize};

/// Driver / adapter package model — what a community package provides and requires.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AdapterMetadata {
    /// Symbols this package exports (e.g. `LidarAdapter`, `Topic<LidarScan>`).
    #[serde(default)]
    pub provides: Vec<String>,

    /// Capabilities this adapter needs from the runtime.
    #[serde(default)]
    pub requires: Vec<String>,
}

/// Framework packages planned for the ecosystem (registry stub metadata).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FrameworkPackage {
    pub name: &'static str,
    pub description: &'static str,
    pub category: super::category::PackageCategory,
    pub import_paths: &'static [&'static str],
}

pub fn framework_packages() -> &'static [FrameworkPackage] {
    // Framework packages.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // &'static [FrameworkPackage].
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::adapter::framework_packages();

    // Return the static list of known values.
    &[
        FrameworkPackage {
            name: "spanda-ros2",
            description: "ROS 2 integration framework",
            category: super::category::PackageCategory::Ros2,
            import_paths: &["robotics.ros2"],
        },
        FrameworkPackage {
            name: "spanda-mqtt",
            description: "MQTT pub/sub transport",
            category: super::category::PackageCategory::Mqtt,
            import_paths: &["communication.mqtt"],
        },
        FrameworkPackage {
            name: "spanda-opencv",
            description: "OpenCV vision bindings",
            category: super::category::PackageCategory::Vision,
            import_paths: &["vision.opencv"],
        },
        FrameworkPackage {
            name: "spanda-yolo",
            description: "YOLO object detection",
            category: super::category::PackageCategory::Vision,
            import_paths: &["vision.yolo"],
        },
        FrameworkPackage {
            name: "spanda-slam",
            description: "SLAM algorithms",
            category: super::category::PackageCategory::Navigation,
            import_paths: &["navigation.slam"],
        },
        FrameworkPackage {
            name: "spanda-nav",
            description: "Path planning and navigation",
            category: super::category::PackageCategory::Navigation,
            import_paths: &["navigation.path_planning"],
        },
        FrameworkPackage {
            name: "spanda-manipulation",
            description: "Arm manipulation and grasping",
            category: super::category::PackageCategory::Manipulation,
            import_paths: &["manipulation.grasp"],
        },
        FrameworkPackage {
            name: "spanda-hri",
            description: "Human-robot interaction",
            category: super::category::PackageCategory::Hri,
            import_paths: &["hri.dialogue"],
        },
        FrameworkPackage {
            name: "spanda-digital-twin",
            description: "Digital twin synchronization",
            category: super::category::PackageCategory::DigitalTwin,
            import_paths: &["twin.sync"],
        },
        FrameworkPackage {
            name: "spanda-sim-gazebo",
            description: "Gazebo simulation backend",
            category: super::category::PackageCategory::Simulation,
            import_paths: &["sim.gazebo"],
        },
        FrameworkPackage {
            name: "spanda-sim-webots",
            description: "Webots simulation backend",
            category: super::category::PackageCategory::Simulation,
            import_paths: &["sim.webots"],
        },
    ]
}

pub fn framework_import_paths() -> Vec<&'static str> {
    // Framework import paths.
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
    // let result = spanda_package::adapter::framework_import_paths();

    // Produce framework packages as the result.
    framework_packages()
        .iter()
        .flat_map(|p| p.import_paths.iter().copied())
        .collect()
}
