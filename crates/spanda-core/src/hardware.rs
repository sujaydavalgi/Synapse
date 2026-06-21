//! Hardware profiles and compile-time deployment compatibility verification.

use crate::ast::{
    AiModelDecl, BehaviorDecl, ConfigValue, Program, RobotDecl, SensorDecl, Stmt, TopicDecl,
};
use crate::comm::{default_message_size, estimate_topic_bandwidth_mbps, TopicRole};
use crate::connectivity_positioning::{
    validate_connectivity_policy, validate_geofence, verify_requires_connectivity,
};
use crate::foundations::{
    DeployDecl, HardwareDecl, MissionDecl, RequiresConnectivityDecl, RequiresHardwareDecl,
    RequiresNetworkDecl, ResourceBudgetDecl, SimulateCompatibilityDecl, TaskDecl, TraitDecl,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompatSeverity {
    Pass,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompatItem {
    pub category: String,
    pub message: String,
    pub severity: CompatSeverity,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatrixCell {
    pub robot: String,
    pub target: String,
    pub compatible: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompatibilityMatrix {
    pub cells: Vec<MatrixCell>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompatibilityReport {
    pub compatible: bool,
    pub target: Option<String>,
    pub items: Vec<CompatItem>,
    pub matrix: Option<CompatibilityMatrix>,
}

impl CompatibilityReport {
    pub fn errors(&self) -> impl Iterator<Item = &CompatItem> {
        // Errors.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // impl Iterator<Item = &CompatItem>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.errors();

        // Call items on the current instance.
        self.items
            .iter()
            .filter(|i| i.severity == CompatSeverity::Error)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VerifyOptions {
    pub target: Option<String>,
    pub all_targets: bool,
    pub simulate: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HardwareProfile {
    pub name: String,
    pub cpu: Option<String>,
    pub memory_mb: Option<f64>,
    pub storage_mb: Option<f64>,
    pub gpu_tops: Option<f64>,
    pub gpu_required: bool,
    pub sensors: Vec<String>,
    pub actuators: Vec<String>,
    pub connectivity: Vec<String>,
    pub battery_wh: Option<f64>,
    pub network_bandwidth_mbps: Option<f64>,
    pub network_latency_ms: Option<f64>,
    pub packet_loss_pct: Option<f64>,
    pub min_control_period_ms: f64,
    pub power_draw_w: f64,
}

const ESTIMATED_TASK_COST_MS: f64 = 5.0;

fn builtin_profiles() -> HashMap<String, HardwareProfile> {
    // Builtin profiles.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<String, HardwareProfile>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::builtin_profiles();

    // Produce from as the result.
    HashMap::from([
        profile(
            "RoverV1",
            "CortexA78",
            4096.0,
            32_768.0,
            None,
            false,
            vec!["Camera", "Lidar", "IMU"],
            vec!["DifferentialDrive"],
            vec!["WiFi", "Ethernet"],
            Some(100.0),
            Some(50.0),
            Some(20.0),
            10.0,
            15.0,
        ),
        profile(
            "RoverV2",
            "CortexA78",
            8192.0,
            65_536.0,
            Some(1.0),
            false,
            vec!["Camera", "Lidar", "IMU", "GPS"],
            vec!["DifferentialDrive", "RoboticArm"],
            vec!["WiFi6", "Bluetooth5", "LTE", "GPS"],
            Some(150.0),
            Some(100.0),
            Some(15.0),
            8.0,
            20.0,
        ),
        profile(
            "JetsonOrin",
            "CortexA78AE",
            8192.0,
            131_072.0,
            Some(275.0),
            true,
            vec!["Camera", "Lidar", "IMU"],
            vec!["DifferentialDrive"],
            vec!["Ethernet", "WiFi6", "FiveG"],
            None,
            Some(1000.0),
            Some(5.0),
            5.0,
            25.0,
        ),
        profile(
            "RaspberryPi5",
            "CortexA76",
            8192.0,
            65_536.0,
            None,
            false,
            vec!["Camera", "IMU"],
            vec!["DifferentialDrive"],
            vec!["WiFi", "Bluetooth", "Ethernet"],
            None,
            Some(100.0),
            Some(30.0),
            15.0,
            8.0,
        ),
        profile(
            "ESP32",
            "Xtensa",
            4.0,
            4.0,
            None,
            false,
            vec!["IMU"],
            vec!["DifferentialDrive"],
            vec!["WiFi", "BLE"],
            Some(5.0),
            Some(10.0),
            Some(100.0),
            50.0,
            2.0,
        ),
    ])
}

#[allow(clippy::too_many_arguments)]
fn profile(
    name: &str,
    cpu: &str,
    memory_mb: f64,
    storage_mb: f64,
    gpu_tops: Option<f64>,
    gpu_required: bool,
    sensors: Vec<&str>,
    actuators: Vec<&str>,
    connectivity: Vec<&str>,
    battery_wh: Option<f64>,
    network_bandwidth_mbps: Option<f64>,
    network_latency_ms: Option<f64>,
    min_control_period_ms: f64,
    power_draw_w: f64,
) -> (String, HardwareProfile) {
    // Profile.
    //
    // Parameters:
    // - `name` — input value
    // - `cpu` — input value
    // - `memory_mb` — input value
    // - `storage_mb` — input value
    // - `gpu_tops` — input value
    // - `gpu_required` — input value
    // - `sensors` — input value
    // - `actuators` — input value
    // - `battery_wh` — input value
    // - `network_bandwidth_mbps` — input value
    // - `network_latency_ms` — input value
    // - `min_control_period_ms` — input value
    // - `power_draw_w` — input value
    //
    // Returns:
    // (String, HardwareProfile).
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::profile(name, cpu, memory_mb, storage_mb, gpu_tops, gpu_required, sensors, actuators, battery_wh, network_bandwidth_mbps, network_latency_ms, min_control_period_ms, power_draw_w);

    // Produce value as the result.
    (
        name.into(),
        HardwareProfile {
            name: name.into(),
            cpu: Some(cpu.into()),
            memory_mb: Some(memory_mb),
            storage_mb: Some(storage_mb),
            gpu_tops,
            gpu_required,
            sensors: sensors.into_iter().map(str::to_string).collect(),
            actuators: actuators.into_iter().map(str::to_string).collect(),
            connectivity: connectivity.into_iter().map(str::to_string).collect(),
            battery_wh,
            network_bandwidth_mbps,
            network_latency_ms,
            packet_loss_pct: None,
            min_control_period_ms,
            power_draw_w,
        },
    )
}

pub fn list_hardware_profiles() -> Vec<String> {
    // List hardware profiles.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Vec<String>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::list_hardware_profiles();

    // Create mutable names for accumulating results.
    let mut names: Vec<_> = builtin_profiles().into_keys().collect();
    names.sort();
    names
}

pub fn hardware_profile_from_decl(decl: &HardwareDecl) -> HardwareProfile {
    // Hardware profile from decl.
    //
    // Parameters:
    // - `decl` — input value
    //
    // Returns:
    // HardwareProfile.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::hardware_profile_from_decl(decl);

    // Compute HardwareDecl for the following logic.
    let HardwareDecl::HardwareDecl {
        name,
        cpu,
        memory_mb,
        storage_mb,
        gpu_tops,
        gpu_required,
        sensors,
        actuators,
        connectivity,
        battery_wh,
        network_bandwidth_mbps,
        network_latency_ms,
        min_control_period_ms,
        power_draw_w,
        ..
    } = decl;
    HardwareProfile {
        name: name.clone(),
        cpu: cpu.clone(),
        memory_mb: *memory_mb,
        storage_mb: *storage_mb,
        gpu_tops: *gpu_tops,
        gpu_required: *gpu_required,
        sensors: sensors.clone(),
        actuators: actuators.clone(),
        connectivity: connectivity.clone(),
        battery_wh: *battery_wh,
        network_bandwidth_mbps: *network_bandwidth_mbps,
        network_latency_ms: *network_latency_ms,
        packet_loss_pct: None,
        min_control_period_ms: min_control_period_ms.unwrap_or(20.0),
        power_draw_w: power_draw_w.unwrap_or(10.0),
    }
}

pub fn build_profile_registry(program: &Program) -> HashMap<String, HardwareProfile> {
    // Build profile registry.
    //
    // Parameters:
    // - `program` — input value
    //
    // Returns:
    // HashMap<String, HardwareProfile>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::build_profile_registry(program);

    // Create mutable registry for accumulating results.
    let mut registry = builtin_profiles();
    let Program::Program {
        hardware_profiles, ..
    } = program;

    // Process each hardware profile.
    for decl in hardware_profiles {
        let profile = hardware_profile_from_decl(decl);
        registry.insert(profile.name.clone(), profile);
    }
    registry
}

fn pass(category: &str, message: impl Into<String>, line: u32, column: u32) -> CompatItem {
    // Pass.
    //
    // Parameters:
    // - `category` — input value
    // - `message` — input value
    // - `line` — input value
    // - `column` — input value
    //
    // Returns:
    // CompatItem.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::pass(category, message, line, column);

    // Produce CompatItem as the result.
    CompatItem {
        category: category.into(),
        message: message.into(),
        severity: CompatSeverity::Pass,
        line,
        column,
    }
}

fn warn(category: &str, message: impl Into<String>, line: u32, column: u32) -> CompatItem {
    // Warn.
    //
    // Parameters:
    // - `category` — input value
    // - `message` — input value
    // - `line` — input value
    // - `column` — input value
    //
    // Returns:
    // CompatItem.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::warn(category, message, line, column);

    // Produce CompatItem as the result.
    CompatItem {
        category: category.into(),
        message: message.into(),
        severity: CompatSeverity::Warning,
        line,
        column,
    }
}

fn error(category: &str, message: impl Into<String>, line: u32, column: u32) -> CompatItem {
    // Error.
    //
    // Parameters:
    // - `category` — input value
    // - `message` — input value
    // - `line` — input value
    // - `column` — input value
    //
    // Returns:
    // CompatItem.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::error(category, message, line, column);

    // Produce CompatItem as the result.
    CompatItem {
        category: category.into(),
        message: message.into(),
        severity: CompatSeverity::Error,
        line,
        column,
    }
}

fn sensor_adapter(sensor_type: &str) -> Option<&'static str> {
    // Sensor adapter.
    //
    // Parameters:
    // - `sensor_type` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::sensor_adapter(sensor_type);

    // Match on sensor type and handle each case.
    match sensor_type {
        "Camera" => Some("CameraAdapter"),
        "Lidar" => Some("LidarAdapter"),
        "IMU" => Some("ImuAdapter"),
        "GPS" => Some("GpsAdapter"),
        "GNSS" => Some("GpsAdapter"),
        _ => None,
    }
}

fn actuator_adapter(actuator_type: &str) -> Option<&'static str> {
    // Actuator adapter.
    //
    // Parameters:
    // - `actuator_type` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::actuator_adapter(actuator_type);

    // Match on actuator type and handle each case.
    match actuator_type {
        "DifferentialDrive" => Some("MotorAdapter"),
        "RoboticArm" => Some("ArmAdapter"),
        "DroneRotors" => Some("RotorAdapter"),
        "Gripper" => Some("GripperAdapter"),
        _ => None,
    }
}

fn apply_fault(mut profile: HardwareProfile, fault_type: &str) -> HardwareProfile {
    // Apply fault.
    //
    // Parameters:
    // - `mut profile` — input value
    // - `fault_type` — input value
    //
    // Returns:
    // HardwareProfile.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::apply_fault(mut profile, fault_type);

    // Match on fault type and handle each case.
    match fault_type {
        "CameraFailure" => {
            profile.sensors.retain(|s| s != "Camera");
        }
        "LidarFailure" => {
            profile.sensors.retain(|s| s != "Lidar");
        }
        "BatteryDegradation" => {
            // Emit output when battery wh provides a b.
            if let Some(b) = profile.battery_wh {
                profile.battery_wh = Some(b * 0.5);
            }
        }
        "NetworkOutage" => {
            profile.network_bandwidth_mbps = Some(0.0);
            profile.network_latency_ms = Some(10_000.0);
        }
        "ImuFailure" => {
            profile.sensors.retain(|s| s != "IMU");
        }
        "GpsFailure" | "GPSLost" => {
            profile.sensors.retain(|s| s != "GPS" && s != "GNSS");
            profile.connectivity.retain(|c| c != "GPS" && c != "GNSS");
        }
        "GpsDrift" | "GpsSpoofing" => {}
        "WeakWifi" => {
            profile.network_bandwidth_mbps = Some(1.0);
            profile.network_latency_ms = Some(500.0);
        }
        "LteOutage" => {
            profile.network_bandwidth_mbps = Some(0.0);
            profile.network_latency_ms = Some(10_000.0);
            profile
                .connectivity
                .retain(|c| !matches!(c.as_str(), "LTE" | "FourG" | "4G" | "FiveG" | "5G"));
        }
        "SatelliteOutage" => {
            profile.network_bandwidth_mbps = Some(0.0);
            profile.network_latency_ms = Some(10_000.0);
            profile.connectivity.retain(|c| c != "Satellite");
        }
        "NetworkLatencySpike" | "LatencySpike" => {
            profile.network_latency_ms = Some(2000.0);
        }
        "FiveGHandoff" => {
            profile.network_latency_ms = Some(150.0);
        }
        "BluetoothDisconnect" => {
            profile
                .connectivity
                .retain(|c| !matches!(c.as_str(), "Bluetooth" | "Bluetooth5" | "BLE"));
        }
        "PacketLoss" => {
            profile.packet_loss_pct = Some(10.0);
        }
        _ => {}
    }
    profile
}

fn collect_loop_intervals(stmts: &[Stmt]) -> Vec<f64> {
    // Collect loop intervals.
    //
    // Parameters:
    // - `stmts` — input value
    //
    // Returns:
    // Vec<f64>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::collect_loop_intervals(stmts);

    // Create mutable intervals for accumulating results.
    let mut intervals = Vec::new();

    // Execute each statement in sequence.
    for stmt in stmts {
        // Match on stmt and handle each case.
        match stmt {
            Stmt::LoopStmt {
                interval_ms, body, ..
            } => {
                intervals.push(*interval_ms);
                intervals.extend(collect_loop_intervals(body));
            }
            Stmt::IfStmt {
                then_branch,
                else_branch,
                ..
            } => {
                intervals.extend(collect_loop_intervals(then_branch));

                // Emit output when else branch provides a el.
                if let Some(el) = else_branch {
                    intervals.extend(collect_loop_intervals(el));
                }
            }
            _ => {}
        }
    }
    intervals
}

fn ai_config_number(config: &[(String, ConfigValue)], key: &str) -> Option<f64> {
    // Iterate over config.
    config.iter().find_map(|e| {
        // Take the branch when 0 equals key.
        if e.0 == key {
            // Match on 1 and handle each case.
            match &e.1 {
                ConfigValue::Number(n) => Some(*n),
                _ => None,
            }
        } else {
            None
        }
    })
}

fn ai_config_bool(config: &[(String, ConfigValue)], key: &str) -> bool {
    // Iterate over config.
    config.iter().any(|(k, v)| {
        // Take the branch when k differs from key.
        if k != key {
            return false;
        }

        // Match on v and handle each case.
        match v {
            ConfigValue::Bool(true) => true,
            ConfigValue::Number(n) => *n > 0.0,
            _ => false,
        }
    })
}

fn verify_requires_hardware(
    req: &RequiresHardwareDecl,
    profile: &HardwareProfile,
) -> Vec<CompatItem> {
    // Verify requires hardware.
    //
    // Parameters:
    // - `req` — input value
    // - `profile` — input value
    //
    // Returns:
    // Vec<CompatItem>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::verify_requires_hardware(req, profile);

    // Compute RequiresHardwareDecl for the following logic.
    let RequiresHardwareDecl::RequiresHardwareDecl {
        memory_mb_min,
        storage_mb_min,
        gpu_tops_min,
        gpu_required,
        sensors,
        actuators,
        span,
    } = req;
    let mut items = Vec::new();
    let line = span.start.line;
    let column = span.start.column;

    // Emit output when memory mb min provides a min mem.
    if let Some(min_mem) = memory_mb_min {
        // Match on memory mb and handle each case.
        match profile.memory_mb {
            Some(mem) if mem >= *min_mem => {
                items.push(pass(
                    "memory",
                    format!("Memory {mem} MB meets requirement >= {min_mem} MB"),
                    line,
                    column,
                ));
            }
            Some(mem) => {
                items.push(error(
                    "memory",
                    format!("Memory requirement {min_mem} MB exceeds target {mem} MB"),
                    line,
                    column,
                ));
            }
            None => items.push(warn(
                "memory",
                "Target memory unknown — cannot verify memory requirement",
                line,
                column,
            )),
        }
    }

    // Emit output when storage mb min provides a min storage.
    if let Some(min_storage) = storage_mb_min {
        // Match on storage mb and handle each case.
        match profile.storage_mb {
            Some(storage) if storage >= *min_storage => {
                items.push(pass(
                    "storage",
                    format!("Storage {storage} MB meets requirement >= {min_storage} MB"),
                    line,
                    column,
                ));
            }
            Some(storage) => {
                items.push(error(
                    "storage",
                    format!("Storage requirement {min_storage} MB exceeds target {storage} MB"),
                    line,
                    column,
                ));
            }
            None => items.push(warn(
                "storage",
                "Target storage unknown — cannot verify storage requirement",
                line,
                column,
            )),
        }
    }

    // Take this path when *gpu required && !profile.gpu required && profile.gpu tops.is none().
    if *gpu_required && !profile.gpu_required && profile.gpu_tops.is_none() {
        items.push(error(
            "gpu",
            format!(
                "GPU required but hardware profile '{}' has no GPU",
                profile.name
            ),
            line,
            column,
        ));
    }

    // Emit output when gpu tops min provides a min tops.
    if let Some(min_tops) = gpu_tops_min {
        // Match on gpu tops and handle each case.
        match profile.gpu_tops {
            Some(tops) if tops >= *min_tops => {
                items.push(pass(
                    "gpu",
                    format!("GPU {tops} TOPS meets requirement >= {min_tops} TOPS"),
                    line,
                    column,
                ));
            }
            Some(tops) => {
                items.push(error(
                    "gpu",
                    format!("GPU requirement {min_tops} TOPS exceeds target {tops} TOPS"),
                    line,
                    column,
                ));
            }
            None => items.push(error(
                "gpu",
                format!("GPU requirement {min_tops} TOPS but target has no GPU"),
                line,
                column,
            )),
        }
    }
    let sensor_set: HashSet<_> = profile.sensors.iter().collect();

    // Process each sensor.
    for sensor_type in sensors {
        // Check membership before continuing.
        if sensor_set.contains(sensor_type) {
            items.push(pass(
                "sensors",
                format!(
                    "Required sensor '{sensor_type}' available on {}",
                    profile.name
                ),
                line,
                column,
            ));
        } else {
            items.push(error(
                "sensors",
                format!(
                    "Required sensor '{sensor_type}' not on '{}' [{}]",
                    profile.name,
                    profile.sensors.join(", ")
                ),
                line,
                column,
            ));
        }
    }
    let actuator_set: HashSet<_> = profile.actuators.iter().collect();

    // Process each actuator.
    for actuator_type in actuators {
        // Check membership before continuing.
        if actuator_set.contains(actuator_type) {
            items.push(pass(
                "actuators",
                format!(
                    "Required actuator '{actuator_type}' available on {}",
                    profile.name
                ),
                line,
                column,
            ));
        } else {
            items.push(error(
                "actuators",
                format!(
                    "Required actuator '{actuator_type}' not on '{}' [{}]",
                    profile.name,
                    profile.actuators.join(", ")
                ),
                line,
                column,
            ));
        }
    }
    items
}

fn verify_requires_network(
    req: &RequiresNetworkDecl,
    profile: &HardwareProfile,
) -> Vec<CompatItem> {
    // Verify requires network.
    //
    // Parameters:
    // - `req` — input value
    // - `profile` — input value
    //
    // Returns:
    // Vec<CompatItem>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::verify_requires_network(req, profile);

    // Compute RequiresNetworkDecl for the following logic.
    let RequiresNetworkDecl::RequiresNetworkDecl {
        bandwidth_mbps_min,
        latency_ms_max,
        span,
    } = req;
    let mut items = Vec::new();
    let line = span.start.line;
    let column = span.start.column;

    // Emit output when bandwidth mbps min provides a min bw.
    if let Some(min_bw) = bandwidth_mbps_min {
        // Match on network bandwidth mbps and handle each case.
        match profile.network_bandwidth_mbps {
            Some(bw) if bw >= *min_bw => {
                items.push(pass(
                    "network",
                    format!("Bandwidth {bw} Mbps meets requirement >= {min_bw} Mbps"),
                    line,
                    column,
                ));
            }
            Some(bw) => {
                items.push(error(
                    "network",
                    format!("Bandwidth requirement {min_bw} Mbps exceeds target {bw} Mbps"),
                    line,
                    column,
                ));
            }
            None => items.push(warn(
                "network",
                "Target bandwidth unknown — cannot verify bandwidth requirement",
                line,
                column,
            )),
        }
    }

    // Emit output when latency ms max provides a max lat.
    if let Some(max_lat) = latency_ms_max {
        // Match on network latency ms and handle each case.
        match profile.network_latency_ms {
            Some(lat) if lat <= *max_lat => {
                items.push(pass(
                    "network",
                    format!("Latency {lat} ms meets requirement <= {max_lat} ms"),
                    line,
                    column,
                ));
            }
            Some(lat) => {
                items.push(error(
                    "network",
                    format!("Latency requirement {max_lat} ms violated by target {lat} ms"),
                    line,
                    column,
                ));
            }
            None => items.push(warn(
                "network",
                "Target latency unknown — cannot verify latency requirement",
                line,
                column,
            )),
        }
    }
    items
}

fn verify_resource_budget(
    budget: &ResourceBudgetDecl,
    profile: &HardwareProfile,
    task_interval_ms: f64,
) -> Vec<CompatItem> {
    // Verify resource budget.
    //
    // Parameters:
    // - `budget` — input value
    // - `profile` — input value
    // - `task_interval_ms` — input value
    //
    // Returns:
    // Vec<CompatItem>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::verify_resource_budget(budget, profile, task_interval_ms);

    // Compute ResourceBudgetDecl for the following logic.
    let ResourceBudgetDecl::ResourceBudgetDecl {
        battery_pct_max,
        memory_mb_max,
        cpu_pct_max,
        gpu_pct_max,
        network_mbps_max,
        storage_mb_max,
        span,
    } = budget;
    let mut items = Vec::new();
    let line = span.start.line;
    let column = span.start.column;

    // Emit output when memory mb max provides a mem max.
    if let Some(mem_max) = memory_mb_max {
        // Match on memory mb and handle each case.
        match profile.memory_mb {
            Some(total) if *mem_max <= total => {
                items.push(pass(
                    "memory",
                    format!("Task memory budget {mem_max} MB within target {total} MB"),
                    line,
                    column,
                ));
            }
            Some(total) => {
                items.push(error(
                    "memory",
                    format!("Task memory budget {mem_max} MB exceeds target {total} MB"),
                    line,
                    column,
                ));
            }
            None => items.push(warn(
                "memory",
                "Cannot verify task memory budget — target memory unknown",
                line,
                column,
            )),
        }
    }

    // Emit output when cpu pct max provides a cpu max.
    if let Some(cpu_max) = cpu_pct_max {
        let duty = (ESTIMATED_TASK_COST_MS / task_interval_ms.max(1.0)) * 100.0;

        // Take this path when duty <= *cpu max.
        if duty <= *cpu_max {
            items.push(pass(
                "cpu",
                format!("Estimated task CPU {duty:.1}% within budget {cpu_max}%"),
                line,
                column,
            ));
        } else {
            items.push(error(
                "cpu",
                format!("Estimated task CPU {duty:.1}% exceeds budget {cpu_max}%"),
                line,
                column,
            ));
        }
    }

    // Emit output when gpu pct max provides a gpu max.
    if let Some(gpu_max) = gpu_pct_max {
        let duty = (ESTIMATED_TASK_COST_MS / task_interval_ms.max(1.0)) * 100.0;
        if duty <= *gpu_max {
            items.push(pass(
                "gpu",
                format!("Estimated task GPU {duty:.1}% within budget {gpu_max}%"),
                line,
                column,
            ));
        } else {
            items.push(error(
                "gpu",
                format!("Estimated task GPU {duty:.1}% exceeds budget {gpu_max}%"),
                line,
                column,
            ));
        }
    }

    // Emit output when battery pct max provides a bat max.
    if let Some(bat_max) = battery_pct_max {
        let duty_pct = (task_interval_ms / 1000.0) * (*bat_max / 100.0);
        items.push(pass(
            "power",
            format!("Task battery duty cycle ~{duty_pct:.2}% within budget {bat_max}%"),
            line,
            column,
        ));
    }

    // Emit output when network mbps max provides a net max.
    if let Some(net_max) = network_mbps_max {
        // Match on network bandwidth mbps and handle each case.
        match profile.network_bandwidth_mbps {
            Some(bw) if *net_max <= bw => {
                items.push(pass(
                    "network",
                    format!("Task network budget {net_max} Mbps within target {bw} Mbps"),
                    line,
                    column,
                ));
            }
            Some(bw) => {
                items.push(error(
                    "network",
                    format!("Task network budget {net_max} Mbps exceeds target {bw} Mbps"),
                    line,
                    column,
                ));
            }
            None => items.push(warn(
                "network",
                "Cannot verify task network budget — target bandwidth unknown",
                line,
                column,
            )),
        }
    }

    // Emit output when storage mb max provides a stor max.
    if let Some(stor_max) = storage_mb_max {
        // Match on storage mb and handle each case.
        match profile.storage_mb {
            Some(total) if *stor_max <= total => {
                items.push(pass(
                    "storage",
                    format!("Task storage budget {stor_max} MB within target {total} MB"),
                    line,
                    column,
                ));
            }
            Some(total) => {
                items.push(error(
                    "storage",
                    format!("Task storage budget {stor_max} MB exceeds target {total} MB"),
                    line,
                    column,
                ));
            }
            None => items.push(warn(
                "storage",
                "Cannot verify task storage budget — target storage unknown",
                line,
                column,
            )),
        }
    }
    items
}

fn verify_timing(robot: &RobotDecl, profile: &HardwareProfile) -> Vec<CompatItem> {
    // Verify timing.
    //
    // Parameters:
    // - `robot` — input value
    // - `profile` — input value
    //
    // Returns:
    // Vec<CompatItem>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::verify_timing(robot, profile);

    // Compute RobotDecl for the following logic.
    let RobotDecl::RobotDecl {
        tasks,
        behaviors,
        span,
        ..
    } = robot;
    let mut items = Vec::new();
    let line = span.start.line;
    let column = span.start.column;
    let min_period = profile.min_control_period_ms;
    let mut total_cpu_pct = 0.0;

    // Process each task.
    for task in tasks {
        let TaskDecl::TaskDecl {
            name,
            interval_ms,
            body: _,
            span: task_span,
            ..
        } = task;

        // Take this path when *interval ms < min period.
        if *interval_ms < min_period {
            items.push(error(
                "timing",
                format!(
                    "Task '{name}' period {interval_ms}ms below hardware minimum {min_period}ms on '{}'",
                    profile.name
                ),
                task_span.start.line,
                task_span.start.column,
            ));
        } else {
            items.push(pass(
                "timing",
                format!(
                    "Task '{name}' period {interval_ms}ms schedulable on '{}' (min {min_period}ms)",
                    profile.name
                ),
                task_span.start.line,
                task_span.start.column,
            ));
        }
        total_cpu_pct += (ESTIMATED_TASK_COST_MS / interval_ms.max(1.0)) * 100.0;
    }

    // Process each behavior.
    for behavior in behaviors {
        let BehaviorDecl::BehaviorDecl {
            name, body, span, ..
        } = behavior;

        // Process each collect loop interval.
        for interval in collect_loop_intervals(body) {
            // Take this path when interval < min period.
            if interval < min_period {
                items.push(error(
                    "timing",
                    format!(
                        "Behavior '{name}' loop {interval}ms below hardware minimum {min_period}ms on '{}'",
                        profile.name
                    ),
                    span.start.line,
                    span.start.column,
                ));
            } else {
                items.push(pass(
                    "timing",
                    format!(
                        "Behavior '{name}' loop {interval}ms schedulable on '{}'",
                        profile.name
                    ),
                    span.start.line,
                    span.start.column,
                ));
            }
            total_cpu_pct += (ESTIMATED_TASK_COST_MS / interval.max(1.0)) * 100.0;
        }
    }

    // Take this path when total cpu pct > 100.0.
    if total_cpu_pct > 100.0 {
        items.push(error(
            "timing",
            format!(
                "Aggregate CPU load {total_cpu_pct:.1}% exceeds 100% on '{}'",
                profile.name
            ),
            line,
            column,
        ));
    } else if total_cpu_pct > 80.0 {
        items.push(warn(
            "timing",
            format!(
                "Aggregate CPU load {total_cpu_pct:.1}% leaves little scheduling margin on '{}'",
                profile.name
            ),
            line,
            column,
        ));
    } else if total_cpu_pct > 0.0 {
        items.push(pass(
            "timing",
            format!(
                "Aggregate CPU load {total_cpu_pct:.1}% within scheduling budget on '{}'",
                profile.name
            ),
            line,
            column,
        ));
    }
    items
}

fn verify_ai_models(robot: &RobotDecl, profile: &HardwareProfile) -> Vec<CompatItem> {
    // Verify ai models.
    //
    // Parameters:
    // - `robot` — input value
    // - `profile` — input value
    //
    // Returns:
    // Vec<CompatItem>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::verify_ai_models(robot, profile);

    // Compute RobotDecl for the following logic.
    let RobotDecl::RobotDecl { ai_models, .. } = robot;
    let mut items = Vec::new();

    // Process each ai model.
    for model in ai_models {
        let AiModelDecl::AiModelDecl {
            name, config, span, ..
        } = model;
        let config_pairs: Vec<_> = config
            .iter()
            .map(|e| (e.key.clone(), e.value.clone()))
            .collect();

        // Emit output when ai config number provides a mem req.
        if let Some(mem_req) = ai_config_number(&config_pairs, "memory_required") {
            // Match on memory mb and handle each case.
            match profile.memory_mb {
                Some(mem) if mem >= mem_req => {
                    items.push(pass(
                        "ai",
                        format!("AI model '{name}' memory {mem_req} MB fits in target {mem} MB"),
                        span.start.line,
                        span.start.column,
                    ));
                }
                Some(mem) => {
                    items.push(error(
                        "ai",
                        format!("AI model '{name}' requires {mem_req} MB but target has {mem} MB"),
                        span.start.line,
                        span.start.column,
                    ));
                }
                None => items.push(warn(
                    "ai",
                    format!("AI model '{name}' memory requirement cannot be verified"),
                    span.start.line,
                    span.start.column,
                )),
            }
        }

        // Take this path when ai config bool(&config pairs, "gpu required").
        if ai_config_bool(&config_pairs, "gpu_required") {
            // Proceed only when is some is available.
            if profile.gpu_required || profile.gpu_tops.is_some() {
                items.push(pass(
                    "ai",
                    format!(
                        "AI model '{name}' GPU requirement satisfied on {}",
                        profile.name
                    ),
                    span.start.line,
                    span.start.column,
                ));
            } else {
                items.push(error(
                    "ai",
                    format!(
                        "AI model '{name}' requires GPU but '{}' has no GPU",
                        profile.name
                    ),
                    span.start.line,
                    span.start.column,
                ));
            }
        }
    }
    items
}

fn verify_battery_mission(mission: &MissionDecl, profile: &HardwareProfile) -> Vec<CompatItem> {
    // Verify battery mission.
    //
    // Parameters:
    // - `mission` — input value
    // - `profile` — input value
    //
    // Returns:
    // Vec<CompatItem>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::verify_battery_mission(mission, profile);

    // Compute MissionDecl for the following logic.
    let MissionDecl::MissionDecl {
        duration_hours,
        span,
    } = mission;
    let mut items = Vec::new();
    let line = span.start.line;
    let column = span.start.column;
    let Some(battery_wh) = profile.battery_wh else {
        return vec![warn(
            "power",
            "Mission duration declared but target battery capacity unknown",
            line,
            column,
        )];
    };
    let energy_wh = profile.power_draw_w * duration_hours;
    let margin = battery_wh - energy_wh;

    // Take this path when energy wh > battery wh.
    if energy_wh > battery_wh {
        items.push(error(
            "power",
            format!(
                "Mission requires {:.1} Wh but battery supports {:.1} Wh ({:.0} min)",
                energy_wh,
                battery_wh,
                battery_wh / profile.power_draw_w * 60.0
            ),
            line,
            column,
        ));
    } else if margin < battery_wh * 0.2 {
        items.push(warn(
            "power",
            format!(
                "Battery margin low: mission {:.1} Wh vs capacity {:.1} Wh",
                energy_wh, battery_wh
            ),
            line,
            column,
        ));
    } else {
        items.push(pass(
            "power",
            format!(
                "Mission energy {:.1} Wh within battery capacity {:.1} Wh",
                energy_wh, battery_wh
            ),
            line,
            column,
        ));
    }
    items
}

fn verify_adapters(
    robot: &RobotDecl,
    profile: &HardwareProfile,
    program_traits: &HashSet<String>,
) -> Vec<CompatItem> {
    // Verify adapters.
    //
    // Parameters:
    // - `robot` — input value
    // - `profile` — input value
    // - `program_traits` — input value
    //
    // Returns:
    // Vec<CompatItem>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::verify_adapters(robot, profile, program_traits);

    // Compute RobotDecl for the following logic.
    let RobotDecl::RobotDecl {
        sensors, actuators, ..
    } = robot;
    let mut items = Vec::new();

    // Iterate over the collection.
    for SensorDecl::SensorDecl {
        name,
        sensor_type,
        span,
        ..
    } in sensors
    {
        // Emit output when sensor adapter provides a adapter.
        if let Some(adapter) = sensor_adapter(sensor_type) {
            let hw_ok = profile.sensors.iter().any(|s| s == sensor_type);

            // Take this path when hw ok.
            if hw_ok {
                let msg = if program_traits.contains(adapter) {
                    format!(
                        "Adapter trait '{adapter}' maps sensor '{name}' ({sensor_type}) to {}",
                        profile.name
                    )
                } else {
                    format!(
                        "Builtin adapter '{adapter}' maps sensor '{name}' ({sensor_type}) to {}",
                        profile.name
                    )
                };
                items.push(pass("adapter", msg, span.start.line, span.start.column));
            }
        }
    }

    // Process each actuator.
    for actuator in actuators {
        let crate::ast::ActuatorDecl::ActuatorDecl {
            name,
            actuator_type,
            span,
            ..
        } = actuator;

        // Emit output when actuator adapter provides a adapter.
        if let Some(adapter) = actuator_adapter(actuator_type) {
            let hw_ok = profile.actuators.iter().any(|a| a == actuator_type);

            // Take this path when hw ok.
            if hw_ok {
                let msg = if program_traits.contains(adapter) {
                    format!(
                        "Adapter trait '{adapter}' maps actuator '{name}' ({actuator_type}) to {}",
                        profile.name
                    )
                } else {
                    format!(
                        "Builtin adapter '{adapter}' maps actuator '{name}' ({actuator_type}) to {}",
                        profile.name
                    )
                };
                items.push(pass("adapter", msg, span.start.line, span.start.column));
            }
        }
    }
    items
}

fn verify_topic_bandwidth(topics: &[TopicDecl], profile: &HardwareProfile) -> Vec<CompatItem> {
    // Verify topic bandwidth.
    //
    // Parameters:
    // - `topics` — input value
    // - `profile` — input value
    //
    // Returns:
    // Vec<CompatItem>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::verify_topic_bandwidth(topics, profile);

    // Create mutable total mbps for accumulating results.
    let mut total_mbps = 0.0;
    let mut items = Vec::new();

    // Process each topic.
    for topic in topics {
        let TopicDecl::TopicDecl {
            name,
            message_type,
            role,
            qos,
            span,
            ..
        } = topic;

        // Keep entries that match the expected pattern.
        if matches!(role, TopicRole::Subscribe) {
            continue;
        }
        let Some(qos) = qos else { continue };
        let Some(rate_hz) = qos.rate_hz else { continue };
        let msg_size = default_message_size(message_type);
        let mbps = estimate_topic_bandwidth_mbps(rate_hz, msg_size);
        total_mbps += mbps;
        items.push(pass(
            "network",
            format!("Topic '{name}' ({message_type}) at {rate_hz} Hz ≈ {mbps:.2} Mbps",),
            span.start.line,
            span.start.column,
        ));
    }

    // Take this path when total mbps <= 0.0.
    if total_mbps <= 0.0 {
        return items;
    }

    // Match on network bandwidth mbps and handle each case.
    match profile.network_bandwidth_mbps {
        Some(bw) if total_mbps <= bw => {
            items.push(pass(
                "network",
                format!("Estimated topic bandwidth {total_mbps:.2} Mbps within target {bw} Mbps",),
                1,
                1,
            ));
        }
        Some(bw) => {
            items.push(error(
                "network",
                format!("Estimated topic bandwidth {total_mbps:.2} Mbps exceeds target {bw} Mbps",),
                1,
                1,
            ));
        }
        None => {
            items.push(warn(
                "network",
                format!(
                    "Estimated topic bandwidth {total_mbps:.2} Mbps — target bandwidth unknown",
                ),
                1,
                1,
            ));
        }
    }
    items
}

#[allow(clippy::too_many_arguments)]
fn verify_robot_against_profile(
    robot: &RobotDecl,
    profile: &HardwareProfile,
    program_traits: &HashSet<String>,
    program_requires_hw: Option<&RequiresHardwareDecl>,
    program_requires_net: Option<&RequiresNetworkDecl>,
    program_requires_conn: Option<&RequiresConnectivityDecl>,
    span_line: u32,
    span_column: u32,
) -> Vec<CompatItem> {
    // Verify robot against profile.
    //
    // Parameters:
    // - `robot` — input value
    // - `profile` — input value
    // - `program_traits` — input value
    // - `program_requires_hw` — input value
    // - `program_requires_net` — input value
    // - `span_line` — input value
    // - `span_column` — input value
    //
    // Returns:
    // Vec<CompatItem>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::verify_robot_against_profile(robot, profile, program_traits, program_requires_hw, program_requires_net, span_line, span_column);

    // Compute RobotDecl for the following logic.
    let RobotDecl::RobotDecl {
        name: robot_name,
        topics,
        sensors,
        actuators,
        observe,
        requires_hardware,
        requires_network,
        requires_connectivity,
        mission,
        tasks,
        ..
    } = robot;
    let mut items = Vec::new();
    let sensor_set: HashSet<_> = profile.sensors.iter().collect();
    let actuator_set: HashSet<_> = profile.actuators.iter().collect();

    // Iterate over the collection.
    for SensorDecl::SensorDecl {
        name,
        sensor_type,
        span,
        ..
    } in sensors
    {
        // Check membership before continuing.
        if sensor_set.contains(sensor_type) {
            items.push(pass(
                "sensors",
                format!(
                    "Sensor '{name}' ({sensor_type}) available on {}",
                    profile.name
                ),
                span.start.line,
                span.start.column,
            ));
        } else {
            items.push(error(
                "sensors",
                format!(
                    "Sensor '{name}' requires type '{sensor_type}' but hardware profile '{}' provides [{}]",
                    profile.name,
                    profile.sensors.join(", ")
                ),
                span.start.line,
                span.start.column,
            ));
        }
    }

    // Emit output when observe provides a observe decl.
    if let Some(observe_decl) = observe {
        let crate::foundations::ObserveDecl::ObserveDecl {
            sensors: observe_sensors,
            span,
            ..
        } = observe_decl;

        // Process each observe sensor.
        for sensor_name in observe_sensors {
            let declared = sensors.iter().find(|s| match s {
                SensorDecl::SensorDecl { name, .. } => name == sensor_name,
            });

            // Take this path when let Some(SensorDecl::SensorDecl { sensor type, .. }) = declared.
            if let Some(SensorDecl::SensorDecl { sensor_type, .. }) = declared {
                // Check membership before continuing.
                if !sensor_set.contains(sensor_type) {
                    items.push(error(
                        "sensors",
                        format!(
                            "observe fuses sensor '{sensor_name}' ({sensor_type}) not supported on '{}'",
                            profile.name
                        ),
                        span.start.line,
                        span.start.column,
                    ));
                }
            }
        }
    }

    // Process each actuator.
    for actuator in actuators {
        let crate::ast::ActuatorDecl::ActuatorDecl {
            name,
            actuator_type,
            span,
            ..
        } = actuator;

        // Check membership before continuing.
        if actuator_set.contains(actuator_type) {
            items.push(pass(
                "actuators",
                format!(
                    "Actuator '{name}' ({actuator_type}) available on {}",
                    profile.name
                ),
                span.start.line,
                span.start.column,
            ));
        } else {
            items.push(error(
                "actuators",
                format!(
                    "Actuator '{name}' requires type '{actuator_type}' but hardware profile '{}' provides [{}]",
                    profile.name,
                    profile.actuators.join(", ")
                ),
                span.start.line,
                span.start.column,
            ));
        }
    }

    // Skip further work when sensors is empty.
    if sensors.is_empty() && actuators.is_empty() {
        items.push(pass(
            "robot",
            format!("Robot '{robot_name}' has no sensor/actuator requirements"),
            span_line,
            span_column,
        ));
    }

    // Emit output when or provides a req.
    if let Some(req) = requires_hardware.as_ref().or(program_requires_hw) {
        items.extend(verify_requires_hardware(req, profile));
    }

    // Emit output when or provides a req.
    if let Some(req) = requires_network.as_ref().or(program_requires_net) {
        items.extend(verify_requires_network(req, profile));
    }

    if let Some(req) = requires_connectivity.as_ref().or(program_requires_conn) {
        items.extend(verify_requires_connectivity(req, profile));
    }

    // Process each task.
    for task in tasks {
        let TaskDecl::TaskDecl {
            budget,
            interval_ms,
            ..
        } = task;

        // Emit output when budget provides a b.
        if let Some(b) = budget {
            items.extend(verify_resource_budget(b, profile, *interval_ms));
        }
    }

    // Emit output when mission provides a m.
    if let Some(m) = mission {
        items.extend(verify_battery_mission(m, profile));
    }
    items.extend(verify_timing(robot, profile));
    items.extend(verify_ai_models(robot, profile));
    items.extend(verify_adapters(robot, profile, program_traits));
    items.extend(verify_topic_bandwidth(topics, profile));
    items
}

fn trait_names(program: &Program) -> HashSet<String> {
    // Trait names.
    //
    // Parameters:
    // - `program` — input value
    //
    // Returns:
    // HashSet<String>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::trait_names(program);

    // Destructure the program into its top-level sections.
    let Program::Program { traits, .. } = program;
    traits
        .iter()
        .map(|t| match t {
            TraitDecl::TraitDecl { name, .. } => name.clone(),
        })
        .collect()
}

fn resolve_targets(
    program: &Program,
    options: &VerifyOptions,
    registry: &HashMap<String, HardwareProfile>,
) -> Vec<(String, String, u32, u32)> {
    // Resolve targets.
    //
    // Parameters:
    // - `program` — input value
    // - `options` — input value
    // - `registry` — input value
    //
    // Returns:
    // Vec<(String, String, u32, u32)>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::resolve_targets(program, options, registry);

    // Destructure the program into its top-level sections.
    let Program::Program {
        robots,
        deployments,
        ..
    } = program;

    // Take this path when options.all targets.
    if options.all_targets {
        let profile_names: Vec<_> = registry.keys().cloned().collect();
        let mut pairs = Vec::new();

        // Handle each robot declared in the program.
        for robot in robots {
            let RobotDecl::RobotDecl { name, span, .. } = robot;

            // Process each profile name.
            for target in &profile_names {
                pairs.push((
                    name.clone(),
                    target.clone(),
                    span.start.line,
                    span.start.column,
                ));
            }
        }
        return pairs;
    }

    // Emit output when target provides a target.
    if let Some(target) = &options.target {
        return robots
            .iter()
            .map(|r| {
                let RobotDecl::RobotDecl { name, span, .. } = r;
                (
                    name.clone(),
                    target.clone(),
                    span.start.line,
                    span.start.column,
                )
            })
            .collect();
    }
    let mut pairs = Vec::new();

    // Process each deployment.
    for deploy in deployments {
        let DeployDecl::DeployDecl {
            robot_name,
            targets,
            span,
        } = deploy;

        // Process each target.
        for t in targets {
            pairs.push((
                robot_name.clone(),
                t.clone(),
                span.start.line,
                span.start.column,
            ));
        }
    }
    pairs
}

pub fn verify_program_compatibility(
    program: &Program,
    options: &VerifyOptions,
) -> CompatibilityReport {
    // Verify program compatibility.
    //
    // Parameters:
    // - `program` — input value
    // - `options` — input value
    //
    // Returns:
    // CompatibilityReport.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::verify_program_compatibility(program, options);

    // Compute registry for the following logic.
    let registry = build_profile_registry(program);
    let Program::Program {
        robots,
        requires_hardware,
        requires_network,
        requires_connectivity,
        geofences,
        connectivity_policies,
        simulate_compatibility,
        ..
    } = program;
    let mut items = Vec::new();

    for geofence in geofences {
        items.extend(validate_geofence(geofence));
    }
    for policy in connectivity_policies {
        items.extend(validate_connectivity_policy(policy));
    }

    let program_traits = trait_names(program);
    let targets_to_check = resolve_targets(program, options, &registry);
    let run_simulation = options.simulate || simulate_compatibility.is_some();

    // Skip further work when targets to check is empty.
    if targets_to_check.is_empty() && options.target.is_none() && !options.all_targets {
        return CompatibilityReport {
            compatible: true,
            target: None,
            items: vec![pass(
                "deploy",
                "No deployment targets declared — hardware compatibility not required",
                1,
                1,
            )],
            matrix: None,
        };
    }

    // Skip further work when all targets && robots is empty.
    if options.all_targets && robots.is_empty() {
        items.push(error(
            "deploy",
            "No robots in program for --all-targets matrix",
            1,
            1,
        ));
    } else if let Some(target) = &options.target {
        // Skip further work when robots is empty.
        if robots.is_empty() {
            items.push(error(
                "deploy",
                format!("No robots in program to verify against target '{target}'"),
                1,
                1,
            ));
        }
    }
    let primary_target = targets_to_check.first().map(|(_, t, _, _)| t.clone());
    let mut matrix_cells = Vec::new();

    // Iterate over targets to check with destructured elements.
    for (robot_name, target_name, line, column) in &targets_to_check {
        let Some(mut profile) = registry.get(target_name).cloned() else {
            items.push(error(
                "deploy",
                format!("Unknown hardware profile '{target_name}'"),
                *line,
                *column,
            ));
            matrix_cells.push(MatrixCell {
                robot: robot_name.clone(),
                target: target_name.clone(),
                compatible: false,
            });
            continue;
        };

        // Take this path when run simulation.
        if run_simulation {
            // Take this path when let Some(SimulateCompatibilityDecl::SimulateCompatibilityDecl { faults.
            if let Some(SimulateCompatibilityDecl::SimulateCompatibilityDecl { faults, span }) =
                simulate_compatibility
            {
                // Inject each configured hardware fault.
                for fault in faults {
                    profile = apply_fault(profile, &fault.fault_type);
                    items.push(pass(
                        "simulate",
                        format!(
                            "Applied fault '{}' against '{}'",
                            fault.fault_type, target_name
                        ),
                        span.start.line,
                        span.start.column,
                    ));
                }
            }
        }
        let Some(robot) = robots.iter().find(|r| r.name() == *robot_name) else {
            items.push(error(
                "deploy",
                format!("deploy references unknown robot '{robot_name}'"),
                *line,
                *column,
            ));
            matrix_cells.push(MatrixCell {
                robot: robot_name.clone(),
                target: target_name.clone(),
                compatible: false,
            });
            continue;
        };
        items.push(pass(
            "deploy",
            format!("Verifying robot '{robot_name}' against hardware '{target_name}'"),
            *line,
            *column,
        ));
        let pair_items = verify_robot_against_profile(
            robot,
            &profile,
            &program_traits,
            requires_hardware.as_ref(),
            requires_network.as_ref(),
            requires_connectivity.as_ref(),
            *line,
            *column,
        );
        let pair_ok = !pair_items
            .iter()
            .any(|i| i.severity == CompatSeverity::Error);
        items.extend(pair_items);
        matrix_cells.push(MatrixCell {
            robot: robot_name.clone(),
            target: target_name.clone(),
            compatible: pair_ok,
        });
    }
    let matrix = if options.all_targets {
        Some(CompatibilityMatrix {
            cells: matrix_cells,
        })
    } else {
        None
    };
    let compatible = !items.iter().any(|i| i.severity == CompatSeverity::Error);
    CompatibilityReport {
        compatible,
        target: primary_target,
        items,
        matrix,
    }
}

/// Backward-compatible entry point.
pub fn verify_program_compatibility_legacy(
    program: &Program,
    cli_target: Option<&str>,
) -> CompatibilityReport {
    // Verify program compatibility legacy.
    //
    // Parameters:
    // - `program` — input value
    // - `cli_target` — input value
    //
    // Returns:
    // CompatibilityReport.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::hardware::verify_program_compatibility_legacy(program, cli_target);

    // Produce verify program compatibility as the result.
    verify_program_compatibility(
        program,
        &VerifyOptions {
            target: cli_target.map(str::to_string),
            all_targets: false,
            simulate: false,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;

    #[test]
    fn rover_missing_lidar_fails_on_esp32() {
        // Rover missing lidar fails on esp32.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::hardware::rover_missing_lidar_fails_on_esp32();

        let source = r#"
hardware Tiny {
  sensors [ IMU ];
  actuators [ DifferentialDrive ];
}

robot Rover {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}

deploy Rover to Tiny;
"#;
        let program = parse(tokenize(source).unwrap()).unwrap();
        let report = verify_program_compatibility(&program, &VerifyOptions::default());
        assert!(!report.compatible);
    }

    #[test]
    fn timing_violation_detected() {
        // Timing violation detected.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::hardware::timing_violation_detected();

        let source = r#"
robot Rover {
  sensor imu: IMU;
  actuator wheels: DifferentialDrive;
  task control_loop every 5ms {
    wheels.stop();
  }
}
"#;
        let program = parse(tokenize(source).unwrap()).unwrap();
        let report = verify_program_compatibility(
            &program,
            &VerifyOptions {
                target: Some("ESP32".into()),
                ..Default::default()
            },
        );
        assert!(!report.compatible);
        assert!(report.items.iter().any(|i| i.category == "timing"));
    }

    #[test]
    fn battery_mission_exceeds_capacity() {
        // Battery mission exceeds capacity.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::hardware::battery_mission_exceeds_capacity();

        let source = r#"
robot Rover {
  sensor imu: IMU;
  actuator wheels: DifferentialDrive;
  mission { duration: 3 h; }
  behavior run() { wheels.stop(); }
}
"#;
        let program = parse(tokenize(source).unwrap()).unwrap();
        let report = verify_program_compatibility(
            &program,
            &VerifyOptions {
                target: Some("ESP32".into()),
                ..Default::default()
            },
        );
        assert!(!report.compatible);
        assert!(report.items.iter().any(|i| i.category == "power"));
    }

    #[test]
    fn fault_injection_removes_camera() {
        // Fault injection removes camera.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::hardware::fault_injection_removes_camera();

        let source = r#"
robot Rover {
  sensor camera: Camera on "/camera";
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}

simulate_compatibility {
  fault CameraFailure;
}

deploy Rover to RoverV1;
"#;
        let program = parse(tokenize(source).unwrap()).unwrap();
        let report = verify_program_compatibility(
            &program,
            &VerifyOptions {
                simulate: true,
                ..Default::default()
            },
        );
        assert!(!report.compatible);
    }
}
