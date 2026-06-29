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
    // Description:
    //     Framework packages.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: &'static [FrameworkPackage]
    //         Return value from `framework_packages`.
    //
    // Example:
    //     let result = spanda_package::adapter::framework_packages();

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
            name: "spanda-nav2",
            description: "Nav2 stack adapter for ROS 2 navigation",
            category: super::category::PackageCategory::Navigation,
            import_paths: &["navigation.nav2"],
        },
        FrameworkPackage {
            name: "spanda-cartographer",
            description: "Cartographer SLAM adapter",
            category: super::category::PackageCategory::Navigation,
            import_paths: &["navigation.cartographer"],
        },
        FrameworkPackage {
            name: "spanda-rtabmap",
            description: "RTAB-Map SLAM adapter",
            category: super::category::PackageCategory::Navigation,
            import_paths: &["navigation.rtabmap"],
        },
        FrameworkPackage {
            name: "spanda-detectron",
            description: "Detectron2 object detection",
            category: super::category::PackageCategory::Vision,
            import_paths: &["vision.detectron"],
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
        FrameworkPackage {
            name: "spanda-ble",
            description: "Bluetooth Low Energy sensor/actuator bridge",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["connectivity.ble"],
        },
        FrameworkPackage {
            name: "spanda-gps",
            description: "GPS/GNSS receiver adapters",
            category: super::category::PackageCategory::Sensors,
            import_paths: &["positioning.gps"],
        },
        FrameworkPackage {
            name: "spanda-lte",
            description: "LTE/cellular connectivity adapters (alias for spanda-cellular)",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["connectivity.lte"],
        },
        FrameworkPackage {
            name: "spanda-cellular",
            description: "LTE/4G/5G cellular connectivity adapters",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["connectivity.cellular"],
        },
        FrameworkPackage {
            name: "spanda-wifi",
            description: "Wi-Fi connectivity adapters",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["connectivity.wifi"],
        },
        FrameworkPackage {
            name: "spanda-dds",
            description: "DDS pub/sub transport",
            category: super::category::PackageCategory::Mqtt,
            import_paths: &["communication.dds"],
        },
        FrameworkPackage {
            name: "spanda-moveit",
            description: "MoveIt motion planning",
            category: super::category::PackageCategory::Manipulation,
            import_paths: &["manipulation.moveit"],
        },
        FrameworkPackage {
            name: "spanda-gazebo",
            description: "Gazebo simulation backend (alias for spanda-sim-gazebo)",
            category: super::category::PackageCategory::Simulation,
            import_paths: &["sim.gazebo"],
        },
        FrameworkPackage {
            name: "spanda-webots",
            description: "Webots simulation backend (alias for spanda-sim-webots)",
            category: super::category::PackageCategory::Simulation,
            import_paths: &["sim.webots"],
        },
        FrameworkPackage {
            name: "spanda-fleet",
            description: "Multi-robot fleet orchestration",
            category: super::category::PackageCategory::Robotics,
            import_paths: &["robotics.fleet"],
        },
        FrameworkPackage {
            name: "spanda-ota",
            description: "Over-the-air deploy and rollout",
            category: super::category::PackageCategory::SupplyChain,
            import_paths: &["deploy.ota"],
        },
        FrameworkPackage {
            name: "spanda-maintenance",
            description: "Predictive maintenance and health monitoring",
            category: super::category::PackageCategory::Robotics,
            import_paths: &["maintenance.health"],
        },
        FrameworkPackage {
            name: "spanda-ledger",
            description: "Audit ledger and provenance anchoring",
            category: super::category::PackageCategory::Ledger,
            import_paths: &["provenance.ledger"],
        },
        FrameworkPackage {
            name: "spanda-cloud",
            description: "Cloud telemetry and remote command channels",
            category: super::category::PackageCategory::Robotics,
            import_paths: &["cloud.remote"],
        },
        FrameworkPackage {
            name: "spanda-openai",
            description: "OpenAI LLM provider via Python bridge",
            category: super::category::PackageCategory::Ai,
            import_paths: &["ai.openai"],
        },
        FrameworkPackage {
            name: "spanda-anthropic",
            description: "Anthropic Claude live AI provider",
            category: super::category::PackageCategory::Ai,
            import_paths: &["ai.anthropic"],
        },
        FrameworkPackage {
            name: "spanda-onnx",
            description: "ONNX local inference provider",
            category: super::category::PackageCategory::Ai,
            import_paths: &["ai.onnx"],
        },
        FrameworkPackage {
            name: "spanda-iot-core",
            description: "IoT device, telemetry, command, and shadow contracts",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["iot.device", "iot.telemetry", "iot.command", "iot.shadow"],
        },
        FrameworkPackage {
            name: "spanda-opcua",
            description: "OPC-UA industrial protocol adapter",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["iot.opcua"],
        },
        FrameworkPackage {
            name: "spanda-modbus",
            description: "Modbus IoT protocol adapter",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["iot.modbus"],
        },
        FrameworkPackage {
            name: "spanda-zigbee",
            description: "Zigbee mesh protocol adapter",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["iot.zigbee"],
        },
        FrameworkPackage {
            name: "spanda-lora",
            description: "LoRa long-range protocol adapter",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["iot.lora"],
        },
        FrameworkPackage {
            name: "spanda-matter",
            description: "Matter smart-home protocol adapter",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["iot.matter"],
        },
        FrameworkPackage {
            name: "spanda-canbus",
            description: "CAN bus vehicle/industrial protocol adapter",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["iot.canbus"],
        },
        FrameworkPackage {
            name: "spanda-radar",
            description: "Automotive radar sensor adapters",
            category: super::category::PackageCategory::Sensors,
            import_paths: &["sensors.radar"],
        },
        FrameworkPackage {
            name: "spanda-lidar",
            description: "Automotive LiDAR sensor adapters",
            category: super::category::PackageCategory::Sensors,
            import_paths: &["sensors.lidar"],
        },
        FrameworkPackage {
            name: "spanda-ultrasonic",
            description: "Ultrasonic parking sensor adapters",
            category: super::category::PackageCategory::Sensors,
            import_paths: &["sensors.ultrasonic"],
        },
        FrameworkPackage {
            name: "spanda-automotive-ethernet",
            description: "Automotive Ethernet SOME/IP and DoIP adapters",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["automotive.ethernet"],
        },
        FrameworkPackage {
            name: "spanda-lin",
            description: "LIN bus vehicle protocol adapter",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["automotive.lin"],
        },
        FrameworkPackage {
            name: "spanda-uds",
            description: "UDS/ISO-TP vehicle diagnostics adapter",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["automotive.uds"],
        },
        FrameworkPackage {
            name: "spanda-v2x",
            description: "V2X DSRC and C-V2X communication adapter",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["automotive.v2x"],
        },
        FrameworkPackage {
            name: "spanda-assurance",
            description: "Assurance evidence and safety case scaffolds",
            category: super::category::PackageCategory::Robotics,
            import_paths: &["assurance.evidence"],
        },
        FrameworkPackage {
            name: "spanda-knowledge-model",
            description: "System knowledge models and dependency graphs",
            category: super::category::PackageCategory::Robotics,
            import_paths: &["assurance.knowledge"],
        },
        FrameworkPackage {
            name: "spanda-anomaly",
            description: "Anomaly detection backends",
            category: super::category::PackageCategory::Robotics,
            import_paths: &["assurance.anomaly"],
        },
        FrameworkPackage {
            name: "spanda-diagnosis",
            description: "Fault diagnosis and root cause analysis",
            category: super::category::PackageCategory::Robotics,
            import_paths: &["assurance.diagnosis"],
        },
        FrameworkPackage {
            name: "spanda-discovery-mdns",
            description: "mDNS/DNS-SD device discovery transport",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["discovery.mdns"],
        },
        FrameworkPackage {
            name: "spanda-discovery-ble",
            description: "Bluetooth LE device discovery transport for Spanda Control Center",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["discovery.ble"],
        },
        FrameworkPackage {
            name: "spanda-discovery-cellular",
            description: "LTE/5G modem discovery transport for Spanda Control Center",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["discovery.cellular"],
        },
        FrameworkPackage {
            name: "spanda-discovery-serial",
            description: "Serial port discovery transport for Spanda Control Center",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["discovery.serial"],
        },
        FrameworkPackage {
            name: "spanda-discovery-tls",
            description: "Production TLS certificate policy for Spanda discovery transports",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["discovery.tls"],
        },
        FrameworkPackage {
            name: "spanda-discovery-usb",
            description: "USB device discovery transport for Spanda Control Center",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["discovery.usb"],
        },
        FrameworkPackage {
            name: "spanda-discovery-wifi",
            description: "WiFi subnet discovery transport for Spanda Control Center",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["discovery.wifi"],
        },
        FrameworkPackage {
            name: "spanda-alert-escalation",
            description:
                "On-call rotation and alert escalation policy templates for Spanda Control Center",
            category: super::category::PackageCategory::Robotics,
            import_paths: &["alerting.escalation"],
        },
        FrameworkPackage {
            name: "spanda-alert-pagerduty",
            description: "PagerDuty Events API v2 alert channel for Spanda Control Center",
            category: super::category::PackageCategory::Robotics,
            import_paths: &["alerting.pagerduty"],
        },
        FrameworkPackage {
            name: "spanda-alert-slack",
            description: "Slack incoming-webhook alert channel for Spanda Control Center",
            category: super::category::PackageCategory::Robotics,
            import_paths: &["alerting.slack"],
        },
        FrameworkPackage {
            name: "spanda-alert-teams",
            description: "Microsoft Teams incoming-webhook alert channel for Spanda Control Center",
            category: super::category::PackageCategory::Robotics,
            import_paths: &["alerting.teams"],
        },
        FrameworkPackage {
            name: "spanda-audit-siem",
            description: "SIEM export adapter for Spanda Control Center mutation audit trail",
            category: super::category::PackageCategory::Robotics,
            import_paths: &["audit.siem"],
        },
        FrameworkPackage {
            name: "spanda-grafana-dashboards",
            description:
                "Grafana dashboard templates for Spanda Control Center SRE and OTA metrics",
            category: super::category::PackageCategory::Robotics,
            import_paths: &["observability.grafana"],
        },
        FrameworkPackage {
            name: "spanda-otel-collector",
            description:
                "OpenTelemetry collector integration for Spanda Control Center distributed traces",
            category: super::category::PackageCategory::Robotics,
            import_paths: &["observability.otel"],
        },
        FrameworkPackage {
            name: "spanda-security-audit",
            description: "Third-party security audit checklist for Spanda Control Center",
            category: super::category::PackageCategory::Robotics,
            import_paths: &["security.audit"],
        },
        FrameworkPackage {
            name: "spanda-prognostics",
            description: "Prognostics and remaining useful life",
            category: super::category::PackageCategory::Robotics,
            import_paths: &["assurance.prognostics"],
        },
        FrameworkPackage {
            name: "spanda-mission-planning",
            description: "Mission planning and execution assurance",
            category: super::category::PackageCategory::Robotics,
            import_paths: &["assurance.mission"],
        },
        FrameworkPackage {
            name: "spanda-mission-continuity",
            description: "Mission continuity, takeover, and succession assurance",
            category: super::category::PackageCategory::Robotics,
            import_paths: &["assurance.continuity"],
        },
        FrameworkPackage {
            name: "spanda-resilience",
            description: "Resilience and recovery policies",
            category: super::category::PackageCategory::Robotics,
            import_paths: &["assurance.resilience"],
        },
        FrameworkPackage {
            name: "spanda-fusion",
            description: "Weighted sensor fusion backends",
            category: super::category::PackageCategory::Robotics,
            import_paths: &["assurance.fusion"],
        },
        FrameworkPackage {
            name: "spanda-smartwatch",
            description: "Smart watch wearable telemetry bridge",
            category: super::category::PackageCategory::Hri,
            import_paths: &["wearable.smartwatch"],
        },
        FrameworkPackage {
            name: "spanda-industrial-wearables",
            description: "Industrial wearable safety telemetry bridge",
            category: super::category::PackageCategory::Hri,
            import_paths: &["wearable.industrial"],
        },
        FrameworkPackage {
            name: "spanda-bodycam",
            description: "Body-worn camera telemetry bridge",
            category: super::category::PackageCategory::Hri,
            import_paths: &["wearable.bodycam"],
        },
        FrameworkPackage {
            name: "spanda-hololens",
            description: "Microsoft HoloLens spatial session provider",
            category: super::category::PackageCategory::Hri,
            import_paths: &["spatial.hololens"],
        },
        FrameworkPackage {
            name: "spanda-arkit",
            description: "Apple ARKit spatial session provider",
            category: super::category::PackageCategory::Hri,
            import_paths: &["spatial.arkit"],
        },
        FrameworkPackage {
            name: "spanda-arcore",
            description: "Google ARCore spatial session provider",
            category: super::category::PackageCategory::Hri,
            import_paths: &["spatial.arcore"],
        },
        FrameworkPackage {
            name: "spanda-vision-pro",
            description: "Apple Vision Pro spatial session provider",
            category: super::category::PackageCategory::Hri,
            import_paths: &["spatial.vision_pro"],
        },
        FrameworkPackage {
            name: "spanda-magic-leap",
            description: "Magic Leap spatial session provider",
            category: super::category::PackageCategory::Hri,
            import_paths: &["spatial.magic_leap"],
        },
        FrameworkPackage {
            name: "spanda-openxr",
            description: "OpenXR VR/XR session provider",
            category: super::category::PackageCategory::Hri,
            import_paths: &["spatial.openxr"],
        },
        FrameworkPackage {
            name: "spanda-voice",
            description: "Voice command HRI input provider",
            category: super::category::PackageCategory::Hri,
            import_paths: &["hri.voice"],
        },
        FrameworkPackage {
            name: "spanda-gesture",
            description: "Gesture recognition HRI input provider",
            category: super::category::PackageCategory::Hri,
            import_paths: &["hri.gesture"],
        },
        FrameworkPackage {
            name: "spanda-eye-tracking",
            description: "Eye tracking and gaze HRI input provider",
            category: super::category::PackageCategory::Hri,
            import_paths: &["hri.eye"],
        },
        FrameworkPackage {
            name: "spanda-trust-jetson",
            description: "Jetson secure boot and attestation contract",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["trust.jetson"],
        },
        FrameworkPackage {
            name: "spanda-trust-pi",
            description: "Raspberry Pi secure boot and attestation contract",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["trust.pi"],
        },
        FrameworkPackage {
            name: "spanda-thread",
            description: "Thread mesh networking for smart devices",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["iot.thread"],
        },
        FrameworkPackage {
            name: "spanda-zwave",
            description: "Z-Wave home automation protocol",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["iot.zwave"],
        },
        FrameworkPackage {
            name: "spanda-bacnet",
            description: "BACnet building automation protocol",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["iot.bacnet"],
        },
        FrameworkPackage {
            name: "spanda-knx",
            description: "KNX building control bus",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["iot.knx"],
        },
        FrameworkPackage {
            name: "spanda-home-assistant",
            description: "Home Assistant bridge for consumer IoT",
            category: super::category::PackageCategory::Mqtt,
            import_paths: &["bridge.home_assistant"],
        },
        FrameworkPackage {
            name: "spanda-energy",
            description: "Solar, battery, and demand-response energy systems",
            category: super::category::PackageCategory::Hardware,
            import_paths: &["energy.solar"],
        },
        FrameworkPackage {
            name: "spanda-building",
            description: "Facility zones, gateways, and readiness orchestration",
            category: super::category::PackageCategory::DigitalTwin,
            import_paths: &["building.entity"],
        },
        FrameworkPackage {
            name: "spanda-smart-locks",
            description: "Smart lock and access control adapters",
            category: super::category::PackageCategory::Safety,
            import_paths: &["access.lock"],
        },
        FrameworkPackage {
            name: "spanda-environment",
            description: "Air quality and environmental sensing",
            category: super::category::PackageCategory::Sensors,
            import_paths: &["environment.aq"],
        },
    ]
}

pub fn framework_import_paths() -> Vec<&'static str> {
    // Description:
    //     Framework import paths.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: Vec<&'static str>
    //         Return value from `framework_import_paths`.
    //
    // Example:
    //     let result = spanda_package::adapter::framework_import_paths();

    // Produce framework packages as the result.
    framework_packages()
        .iter()
        .flat_map(|p| p.import_paths.iter().copied())
        .collect()
}

/// Nav2 adapter package metadata for registry stubs and verify.
pub fn nav2_adapter_metadata() -> AdapterMetadata {
    // Description:
    //     Nav2 adapter metadata.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: AdapterMetadata
    //         Return value from `nav2_adapter_metadata`.
    //
    // Example:

    //     let result = spanda_package::adapter::nav2_adapter_metadata();

    AdapterMetadata {
        provides: vec![
            "Nav2Adapter".into(),
            "NavigationGoal".into(),
            "CostMap".into(),
            "navigate".into(),
        ],
        requires: vec![
            "topic.publish".into(),
            "ros2.bridge".into(),
            "actuator.drive".into(),
        ],
    }
}

/// Cartographer SLAM adapter metadata for registry stubs and verify.
pub fn cartographer_adapter_metadata() -> AdapterMetadata {
    // Description:
    //     Cartographer adapter metadata.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: AdapterMetadata
    //         Return value from `cartographer_adapter_metadata`.
    //
    // Example:

    //     let result = spanda_package::adapter::cartographer_adapter_metadata();

    AdapterMetadata {
        provides: vec![
            "CartographerSlam".into(),
            "OccupancyGrid".into(),
            "PoseGraph".into(),
            "slam.localize".into(),
            "slam.map".into(),
        ],
        requires: vec![
            "topic.publish".into(),
            "sensor.read".into(),
            "lidar.read".into(),
        ],
    }
}

/// RTAB-Map SLAM adapter metadata for registry stubs and verify.
pub fn rtabmap_adapter_metadata() -> AdapterMetadata {
    // Description:
    //     Rtabmap adapter metadata.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: AdapterMetadata
    //         Return value from `rtabmap_adapter_metadata`.
    //
    // Example:

    //     let result = spanda_package::adapter::rtabmap_adapter_metadata();

    AdapterMetadata {
        provides: vec![
            "RtabmapSlam".into(),
            "LoopClosure".into(),
            "VisualOdometry".into(),
            "slam.localize".into(),
            "slam.map".into(),
        ],
        requires: vec![
            "topic.publish".into(),
            "sensor.read".into(),
            "camera.read".into(),
        ],
    }
}

/// Generic SLAM adapter metadata for `navigation.slam` imports.
pub fn slam_adapter_metadata() -> AdapterMetadata {
    // Description:
    //     Slam adapter metadata.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: AdapterMetadata
    //         Return value from `slam_adapter_metadata`.
    //
    // Example:

    //     let result = spanda_package::adapter::slam_adapter_metadata();

    AdapterMetadata {
        provides: vec![
            "SlamAdapter".into(),
            "slam.localize".into(),
            "slam.map".into(),
        ],
        requires: vec!["topic.publish".into(), "sensor.read".into()],
    }
}

/// Resolve expected adapter metadata for a framework import path.
pub fn adapter_metadata_for_import(import_path: &str) -> Option<AdapterMetadata> {
    // Description:
    //     Adapter metadata for import.
    //
    // Inputs:
    //     import_path: &str
    //         Caller-supplied import path.
    //
    // Outputs:
    //     result: Option<AdapterMetadata>
    //         Return value from `adapter_metadata_for_import`.
    //
    // Example:

    //     let result = spanda_package::adapter::adapter_metadata_for_import(import_path);

    match import_path {
        "navigation.nav2" => Some(nav2_adapter_metadata()),
        "navigation.cartographer" => Some(cartographer_adapter_metadata()),
        "navigation.rtabmap" => Some(rtabmap_adapter_metadata()),
        "navigation.slam" => Some(slam_adapter_metadata()),
        _ => None,
    }
}

/// Resolve expected adapter metadata for a registry package name.
pub fn adapter_metadata_for_package(package_name: &str) -> Option<AdapterMetadata> {
    // Description:
    //     Adapter metadata for package.
    //
    // Inputs:
    //     package_name: &str
    //         Caller-supplied package name.
    //
    // Outputs:
    //     result: Option<AdapterMetadata>
    //         Return value from `adapter_metadata_for_package`.
    //
    // Example:

    //     let result = spanda_package::adapter::adapter_metadata_for_package(package_name);

    match package_name {
        "spanda-nav2" | "spanda-nav" => Some(nav2_adapter_metadata()),
        "spanda-cartographer" => Some(cartographer_adapter_metadata()),
        "spanda-rtabmap" => Some(rtabmap_adapter_metadata()),
        "spanda-slam" => Some(slam_adapter_metadata()),
        "spanda-gps" => Some(AdapterMetadata {
            provides: vec!["GpsFix".into(), "positioning.gps.read".into()],
            requires: vec!["sensor.read".into()],
        }),
        "spanda-mqtt" | "spanda-dds" => Some(AdapterMetadata {
            provides: vec!["TransportAdapter".into(), "topic.publish".into()],
            requires: vec!["comm.encrypt".into()],
        }),
        "spanda-ros2" => Some(AdapterMetadata {
            provides: vec!["RosProvider".into(), "comm.ros2.publish".into()],
            requires: vec!["ros2.bridge".into()],
        }),
        "spanda-fleet" => Some(AdapterMetadata {
            provides: vec!["FleetProvider".into(), "fleet.orchestrate".into()],
            requires: vec!["comm.publish".into(), "deploy.agent".into()],
        }),
        "spanda-ota" => Some(AdapterMetadata {
            provides: vec!["deploy.rollout".into(), "deploy.rollback".into()],
            requires: vec!["deploy.sign".into(), "deploy.verify".into()],
        }),
        "spanda-ledger" => Some(AdapterMetadata {
            provides: vec!["LedgerProvider".into(), "audit.append".into()],
            requires: vec!["crypto.sign".into()],
        }),
        _ => None,
    }
}
