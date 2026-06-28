//! Known robot capabilities and their minimum hardware/package requirements.

use serde::{Deserialize, Serialize};

/// Severity when a required capability is missing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VerificationSeverity {
    Error,
    Warning,
    Info,
}

/// Minimum requirement for a single capability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityRequirement {
    pub any_of_sensors: Vec<String>,
    pub any_of_actuators: Vec<String>,
    pub any_of_connectivity: Vec<String>,
    pub required_packages: Vec<String>,
    pub required_providers: Vec<String>,
    pub required_safety_rules: Vec<String>,
    pub required_security: Vec<String>,
    pub severity: VerificationSeverity,
}

/// Full definition of a known capability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityDefinition {
    pub name: String,
    pub description: String,
    pub minimum: CapabilityRequirement,
    pub optional_sensors: Vec<String>,
    pub optional_packages: Vec<String>,
}

/// Package contribution to the capability registry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageCapabilityContribution {
    pub package: String,
    pub capabilities: Vec<String>,
}

/// Built-in capability registry entries.
pub fn capability_registry() -> Vec<CapabilityDefinition> {
    // Description:
    //     Capability registry.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: Vec<CapabilityDefinition>
    //         Return value from `capability_registry`.
    //
    // Example:

    //     let result = spanda_capability::registry::capability_registry();

    vec![
        def(
            "autonomous_navigation",
            "Plan and execute paths without human intervention",
            req(
                &["Lidar", "Camera", "DepthCamera"],
                &["DifferentialDrive", "AckermannDrive"],
                &[],
                &["spanda-nav"],
                &["NavigationProvider"],
                &["max_speed", "stop_if"],
                &[],
                VerificationSeverity::Error,
            ),
            &["Radar"],
            &["spanda-slam"],
        ),
        def(
            "gps_navigation",
            "Navigate using GPS/GNSS positioning",
            req(
                &["GPS", "GNSS"],
                &["DifferentialDrive"],
                &["WiFi", "LTE", "FiveG"],
                &["spanda-gps"],
                &["PositioningProvider"],
                &[],
                &[],
                VerificationSeverity::Error,
            ),
            &[],
            &[],
        ),
        def(
            "obstacle_avoidance",
            "Detect and avoid obstacles during motion",
            req(
                &["Lidar", "DepthCamera", "Radar"],
                &["DifferentialDrive"],
                &[],
                &["spanda-nav"],
                &["NavigationProvider"],
                &["stop_if"],
                &[],
                VerificationSeverity::Error,
            ),
            &["Camera"],
            &["spanda-vision"],
        ),
        def(
            "remote_control",
            "Accept signed remote commands over network",
            req(
                &[],
                &["DifferentialDrive"],
                &["WiFi", "LTE", "FiveG", "Bluetooth"],
                &[],
                &[],
                &[],
                &["signed_commands"],
                VerificationSeverity::Error,
            ),
            &[],
            &["spanda-mqtt"],
        ),
        def(
            "telemetry_streaming",
            "Stream robot state and sensor data remotely",
            req(
                &[],
                &[],
                &["WiFi", "LTE", "FiveG", "MQTT"],
                &["spanda-mqtt"],
                &["TransportProvider"],
                &[],
                &[],
                VerificationSeverity::Warning,
            ),
            &["GPS", "Camera"],
            &["spanda-cloud"],
        ),
        def(
            "emergency_stop",
            "Immediate actuator halt via hardware or kill switch",
            req(
                &[],
                &["DifferentialDrive"],
                &[],
                &[],
                &[],
                &["emergency_stop", "kill_switch"],
                &[],
                VerificationSeverity::Error,
            ),
            &[],
            &[],
        ),
        def(
            "local_ai_inference",
            "Run AI models on onboard compute",
            req(
                &["Camera"],
                &[],
                &[],
                &[],
                &[],
                &["ai.validate"],
                &[],
                VerificationSeverity::Warning,
            ),
            &["Lidar"],
            &[],
        ),
        def(
            "vision_processing",
            "Capture and process camera frames",
            req(
                &["Camera"],
                &[],
                &[],
                &["spanda-opencv", "spanda-yolo"],
                &["VisionProvider"],
                &[],
                &[],
                VerificationSeverity::Warning,
            ),
            &[],
            &[],
        ),
        def(
            "manipulation",
            "Plan and execute arm/gripper motions",
            req(
                &["Camera"],
                &["Arm", "Gripper"],
                &[],
                &["spanda-moveit"],
                &[],
                &["max_force"],
                &[],
                VerificationSeverity::Error,
            ),
            &["ForceTorque"],
            &[],
        ),
        def(
            "fleet_coordination",
            "Coordinate multiple robots in a fleet",
            req(
                &[],
                &[],
                &["WiFi", "LTE"],
                &["spanda-fleet"],
                &["FleetProvider"],
                &[],
                &[],
                VerificationSeverity::Warning,
            ),
            &[],
            &[],
        ),
        def(
            "ota_update",
            "Over-the-air firmware and package updates",
            req(
                &[],
                &[],
                &["WiFi", "LTE"],
                &["spanda-ota"],
                &[],
                &[],
                &["signed_commands"],
                VerificationSeverity::Warning,
            ),
            &[],
            &[],
        ),
        def(
            "secure_communication",
            "Encrypted and authenticated messaging",
            req(
                &[],
                &[],
                &["WiFi", "LTE"],
                &[],
                &["CryptoProvider"],
                &[],
                &["signed_commands", "encrypted_transport"],
                VerificationSeverity::Error,
            ),
            &[],
            &[],
        ),
    ]
    .into_iter()
    .chain(operator_capability_defs())
    .collect()
}

/// Package contributions to the registry.
pub fn package_contributions() -> Vec<PackageCapabilityContribution> {
    // Description:
    //     Package contributions.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: Vec<PackageCapabilityContribution>
    //         Return value from `package_contributions`.
    //
    // Example:

    //     let result = spanda_capability::registry::package_contributions();

    vec![
        contrib(
            "spanda-nav",
            &[
                "autonomous_navigation",
                "path_planning",
                "obstacle_avoidance",
            ],
        ),
        contrib("spanda-gps", &["gps_navigation", "geofencing"]),
        contrib("spanda-mqtt", &["telemetry_streaming", "remote_command"]),
        contrib("spanda-fleet", &["fleet_coordination"]),
        contrib("spanda-ota", &["ota_update"]),
        contrib("spanda-opencv", &["vision_processing"]),
        contrib("spanda-yolo", &["vision_processing", "local_ai_inference"]),
        contrib("spanda-moveit", &["manipulation"]),
        contrib("spanda-cloud", &["telemetry_streaming"]),
        contrib("spanda-slam", &["autonomous_navigation"]),
        contrib(
            "spanda-mission-continuity",
            &["mission_continuity", "human_takeover"],
        ),
        contrib(
            "spanda-smartwatch",
            &["heart_rate", "battery_level", "connectivity_status"],
        ),
        contrib(
            "spanda-industrial-wearables",
            &["fall_detection", "proximity_alert", "gas_detection"],
        ),
        contrib("spanda-bodycam", &["video_stream", "gps_location"]),
        contrib(
            "spanda-hololens",
            &[
                "spatial_anchors",
                "hand_tracking",
                "annotation",
                "robot_overlay",
                "mission_overlay",
            ],
        ),
        contrib(
            "spanda-arkit",
            &["spatial_anchors", "plane_detection", "robot_overlay"],
        ),
        contrib("spanda-arcore", &["spatial_anchors", "plane_detection"]),
        contrib(
            "spanda-vision-pro",
            &["spatial_anchors", "hand_tracking", "robot_overlay"],
        ),
        contrib("spanda-magic-leap", &["spatial_anchors", "robot_overlay"]),
        contrib(
            "spanda-openxr",
            &["vr_training", "mission_replay", "digital_twin_view"],
        ),
        contrib("spanda-voice", &["voice_command"]),
        contrib("spanda-gesture", &["gesture_recognition", "hand_tracking"]),
        contrib("spanda-eye-tracking", &["eye_tracking", "gaze_target"]),
    ]
}

fn operator_capability_defs() -> Vec<CapabilityDefinition> {
    vec![
        operator_def(
            "operate_robot",
            "Human operator authorized to control robots",
        ),
        operator_def("approve_mission", "Supervisor approval for mission start"),
        operator_def(
            "approve_recovery",
            "Supervisor approval for recovery execution",
        ),
        operator_def("emergency_override", "Emergency safety override authority"),
        operator_def("drone_pilot", "Licensed drone pilot operator"),
        operator_def("medical_responder", "Medical responder certification"),
        operator_def("hazmat_certified", "Hazmat zone entry certification"),
        operator_def("remote_expert", "Remote expert assist authority"),
        operator_def(
            "maintenance_technician",
            "Maintenance technician certification",
        ),
        operator_def("forklift_operator", "Forklift operator certification"),
        operator_def(
            "search_rescue_operator",
            "Search and rescue operator certification",
        ),
    ]
}

fn operator_def(name: &str, description: &str) -> CapabilityDefinition {
    def(
        name,
        description,
        req(
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            VerificationSeverity::Info,
        ),
        &[],
        &[],
    )
}

pub fn is_operator_capability(name: &str) -> bool {
    operator_capability_names().contains(&name)
}

fn operator_capability_names() -> &'static [&'static str] {
    &[
        "operate_robot",
        "approve_mission",
        "approve_recovery",
        "emergency_override",
        "drone_pilot",
        "medical_responder",
        "hazmat_certified",
        "remote_expert",
        "maintenance_technician",
        "forklift_operator",
        "search_rescue_operator",
    ]
}

/// Look up a capability by name.
pub fn lookup_capability(name: &str) -> Option<CapabilityDefinition> {
    // Description:
    //     Lookup capability.
    //
    // Inputs:
    //     name: &str
    //         Caller-supplied name.
    //
    // Outputs:
    //     result: Option<CapabilityDefinition>
    //         Return value from `lookup_capability`.
    //
    // Example:

    //     let result = spanda_capability::registry::lookup_capability(name);

    capability_registry().into_iter().find(|c| c.name == name)
}

/// Map sensor/actuator types to hardware-level capabilities.
pub fn sensor_capabilities(sensor_type: &str) -> Vec<&'static str> {
    // Description:
    //     Sensor capabilities.
    //
    // Inputs:
    //     sensor_type: &str
    //         Caller-supplied sensor type.
    //
    // Outputs:
    //     result: Vec<&'static str>
    //         Return value from `sensor_capabilities`.
    //
    // Example:

    //     let result = spanda_capability::registry::sensor_capabilities(sensor_type);

    match sensor_type {
        "GPS" | "GNSS" => vec!["read_location", "read_altitude", "read_heading"],
        "Camera" => vec!["capture_image", "stream_video", "detect_motion"],
        "Lidar" => vec!["scan_range", "obstacle_detection"],
        "DepthCamera" => vec!["depth_map", "obstacle_detection"],
        "Radar" => vec!["range_detection", "obstacle_detection"],
        "IMU" => vec!["read_orientation", "read_acceleration"],
        _ => vec!["read"],
    }
}

/// Map actuator types to hardware-level capabilities.
pub fn actuator_capabilities(actuator_type: &str) -> Vec<&'static str> {
    // Description:
    //     Actuator capabilities.
    //
    // Inputs:
    //     actuator_type: &str
    //         Caller-supplied actuator type.
    //
    // Outputs:
    //     result: Vec<&'static str>
    //         Return value from `actuator_capabilities`.
    //
    // Example:

    //     let result = spanda_capability::registry::actuator_capabilities(actuator_type);

    match actuator_type {
        "DifferentialDrive" | "AckermannDrive" => {
            vec!["move_forward", "rotate", "stop", "emergency_stop"]
        }
        "Arm" => vec!["move_joint", "stop", "emergency_stop"],
        "Gripper" => vec!["open", "close", "stop"],
        _ => vec!["execute", "stop"],
    }
}

fn def(
    name: &str,
    description: &str,
    minimum: CapabilityRequirement,
    optional_sensors: &[&str],
    optional_packages: &[&str],
) -> CapabilityDefinition {
    // Description:
    //     Def.
    //
    // Inputs:
    //     name: &str
    //         Caller-supplied name.
    //     description: &str
    //         Caller-supplied description.
    //     ini: CapabilityRequirement
    //         Caller-supplied ini.
    //     optional_sensors: &[&str]
    //         Caller-supplied optional sensors.
    //     optional_packages: &[&str]
    //         Caller-supplied optional packages.
    //
    // Outputs:
    //     result: CapabilityDefinition
    //         Return value from `def`.
    //
    // Example:

    //     let result = spanda_capability::registry::def(name, description, ini, optional_sensors, optional_packages);

    CapabilityDefinition {
        name: name.into(),
        description: description.into(),
        minimum,
        optional_sensors: optional_sensors.iter().map(|s| (*s).into()).collect(),
        optional_packages: optional_packages.iter().map(|s| (*s).into()).collect(),
    }
}

#[allow(clippy::too_many_arguments)]
fn req(
    sensors: &[&str],
    actuators: &[&str],
    connectivity: &[&str],
    packages: &[&str],
    providers: &[&str],
    safety: &[&str],
    security: &[&str],
    severity: VerificationSeverity,
) -> CapabilityRequirement {
    // Description:
    //     Req.
    //
    // Inputs:
    //     sensors: &[&str]
    //         Caller-supplied sensors.
    //     actuators: &[&str]
    //         Caller-supplied actuators.
    //     connectivity: &[&str]
    //         Caller-supplied connectivity.
    //     packages: &[&str]
    //         Caller-supplied packages.
    //     providers: &[&str]
    //         Caller-supplied providers.
    //     safety: &[&str]
    //         Caller-supplied safety.
    //     security: &[&str]
    //         Caller-supplied security.
    //     severity: VerificationSeverity
    //         Caller-supplied severity.
    //
    // Outputs:
    //     result: CapabilityRequirement
    //         Return value from `req`.
    //
    // Example:

    //     let result = spanda_capability::registry::req(sensors, actuators, connectivity, packages, providers, safety, security, severity);

    CapabilityRequirement {
        any_of_sensors: sensors.iter().map(|s| (*s).into()).collect(),
        any_of_actuators: actuators.iter().map(|s| (*s).into()).collect(),
        any_of_connectivity: connectivity.iter().map(|s| (*s).into()).collect(),
        required_packages: packages.iter().map(|s| (*s).into()).collect(),
        required_providers: providers.iter().map(|s| (*s).into()).collect(),
        required_safety_rules: safety.iter().map(|s| (*s).into()).collect(),
        required_security: security.iter().map(|s| (*s).into()).collect(),
        severity,
    }
}

fn contrib(package: &str, capabilities: &[&str]) -> PackageCapabilityContribution {
    // Description:
    //     Contrib.
    //
    // Inputs:
    //     package: &str
    //         Caller-supplied package.
    //     capabilities: &[&str]
    //         Caller-supplied capabilities.
    //
    // Outputs:
    //     result: PackageCapabilityContribution
    //         Return value from `contrib`.
    //
    // Example:

    //     let result = spanda_capability::registry::contrib(package, capabilities);

    PackageCapabilityContribution {
        package: package.into(),
        capabilities: capabilities.iter().map(|c| (*c).into()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_contains_core_capabilities() {
        // Description:
        //     Registry contains core capabilities.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_capability::registry::registry_contains_core_capabilities();

        assert!(lookup_capability("gps_navigation").is_some());
        assert!(lookup_capability("emergency_stop").is_some());
        assert!(lookup_capability("nonexistent").is_none());
    }

    #[test]
    fn sensor_capability_mapping() {
        // Description:
        //     Sensor capability mapping.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_capability::registry::sensor_capability_mapping();

        let caps = sensor_capabilities("GPS");
        assert!(caps.contains(&"read_location"));
    }
}
