//! Builtin hardware profile catalog for package validation and verify.
//!
//! `HardwareProfile` is now defined in `spanda-connectivity::hardware_types`; re-exported here.

use std::collections::HashMap;

pub use spanda_connectivity::HardwareProfile;

pub fn builtin_profiles() -> HashMap<String, HardwareProfile> {
    // Description:
    //     Builtin profiles.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: HashMap<String, HardwareProfile>
    //         Return value from `builtin_profiles`.
    //
    // Example:
    //     let result = spanda_hardware::profiles::builtin_profiles();

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
    // Description:
    //     Profile.
    //
    // Inputs:
    //     name: &str
    //         Caller-supplied name.
    //     cp: &str
    //         Caller-supplied cp.
    //     emory_mb: f64
    //         Caller-supplied emory mb.
    //     storage_mb: f64
    //         Caller-supplied storage mb.
    //     gpu_tops: Option<f64>
    //         Caller-supplied gpu tops.
    //     gpu_required: bool
    //         Caller-supplied gpu required.
    //     sensors: Vec<&str>
    //         Caller-supplied sensors.
    //     actuators: Vec<&str>
    //         Caller-supplied actuators.
    //     connectivity: Vec<&str>
    //         Caller-supplied connectivity.
    //     battery_wh: Option<f64>
    //         Caller-supplied battery wh.
    //     network_bandwidth_mbps: Option<f64>
    //         Caller-supplied network bandwidth mbps.
    //     network_latency_ms: Option<f64>
    //         Caller-supplied network latency ms.
    //     in_control_period_ms: f64
    //         Caller-supplied in control period ms.
    //     power_draw_w: f64
    //         Caller-supplied power draw w.
    //
    // Outputs:
    //     result: (String, HardwareProfile)
    //         Return value from `profile`.
    //
    // Example:
    //     let result = spanda_hardware::profiles::profile(name, cp, emory_mb, storage_mb, gpu_tops, gpu_required, sensors, actuators, connectivity, battery_wh, network_bandwidth_mbps, network_latency_ms, in_control_period_ms, power_draw_w);

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
    // Description:
    //     List hardware profiles.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: Vec<String>
    //         Return value from `list_hardware_profiles`.
    //
    // Example:
    //     let result = spanda_hardware::profiles::list_hardware_profiles();

    // Create mutable names for accumulating results.
    let mut names: Vec<_> = builtin_profiles().into_keys().collect();
    names.sort();
    names
}
