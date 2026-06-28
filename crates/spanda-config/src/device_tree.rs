//! Physical device hierarchy parsed from fleet/devices TOML.
//!
use crate::human_entities::{
    ControlCenterEntity, HazardZoneEntity, HumanEntity, SpatialDeviceEntity, WearableEntity,
};
use serde::{Deserialize, Serialize};

/// Fleet-level device tree root.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DeviceTree {
    #[serde(default)]
    pub fleet: Option<FleetNode>,
}

/// Fleet node containing robots and human-collaboration entities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetNode {
    pub id: String,
    #[serde(default)]
    pub robots: Vec<RobotNode>,
    #[serde(default)]
    pub humans: Vec<HumanEntity>,
    #[serde(default)]
    pub wearables: Vec<WearableEntity>,
    #[serde(default)]
    pub ar_devices: Vec<SpatialDeviceEntity>,
    #[serde(default)]
    pub vr_devices: Vec<SpatialDeviceEntity>,
    #[serde(default)]
    pub drones: Vec<SpatialDeviceEntity>,
    #[serde(default)]
    pub iot_devices: Vec<SpatialDeviceEntity>,
    #[serde(default)]
    pub control_center: Vec<ControlCenterEntity>,
    #[serde(default)]
    pub hazard_zones: Vec<HazardZoneEntity>,
}

/// Robot with optional onboard compute and attached devices.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RobotNode {
    pub id: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub hardware_profile: Option<String>,
    #[serde(default)]
    pub entity_kind: Option<String>,
    #[serde(default)]
    pub compliance_profile: Option<String>,
    #[serde(default)]
    pub compute: Option<ComputeNode>,
}

/// Onboard compute unit hosting buses, ports, and devices.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComputeNode {
    pub id: String,
    #[serde(rename = "type")]
    pub compute_type: String,
    #[serde(default)]
    pub serial: Option<String>,
    #[serde(default)]
    pub devices: Vec<DeviceNode>,
}

/// Leaf device attached to compute (sensor, actuator, accessory, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceNode {
    pub id: String,
    #[serde(rename = "type")]
    pub device_type: String,
    #[serde(default)]
    pub logical_name: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub port: Option<String>,
    #[serde(default)]
    pub network_port: Option<u16>,
    #[serde(default)]
    pub bus: Option<String>,
    #[serde(default)]
    pub mount: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub firmware: Option<String>,
    #[serde(default, alias = "firmware_version")]
    pub firmware_version: Option<String>,
    #[serde(default)]
    pub hardware_revision: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub trusted: Option<bool>,
    #[serde(default)]
    pub identity: Option<String>,
    #[serde(default)]
    pub security_identity: Option<String>,
    #[serde(default)]
    pub certificate_fingerprint: Option<String>,
    #[serde(default)]
    pub trust_level: Option<String>,
    #[serde(default)]
    pub safety_critical: Option<bool>,
    #[serde(default)]
    pub entity_kind: Option<String>,
    #[serde(default)]
    pub compliance_profile: Option<String>,
    #[serde(default)]
    pub serial: Option<String>,
    #[serde(default, alias = "mac")]
    pub mac_address: Option<String>,
    #[serde(default, alias = "ip")]
    pub ip_address: Option<String>,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub dns_name: Option<String>,
    #[serde(default)]
    pub mdns_name: Option<String>,
    #[serde(default, alias = "endpoint")]
    pub endpoint_url: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub can_id: Option<String>,
    #[serde(default)]
    pub usb_path: Option<String>,
    #[serde(default)]
    pub pci_path: Option<String>,
    #[serde(default)]
    pub bluetooth_address: Option<String>,
    #[serde(default)]
    pub ble_uuid: Option<String>,
    #[serde(default)]
    pub cellular_imei: Option<String>,
    #[serde(default)]
    pub sim_iccid: Option<String>,
    #[serde(default)]
    pub gps_device_id: Option<String>,
    #[serde(default)]
    pub redundant_group: Option<String>,
    #[serde(default)]
    pub failover_priority: Option<u32>,
    #[serde(default)]
    pub health_status: Option<String>,
    #[serde(default)]
    pub last_heartbeat_ms: Option<f64>,
    #[serde(default)]
    pub calibration_status: Option<String>,
    #[serde(default)]
    pub calibration_expiry_ms: Option<f64>,
    #[serde(default)]
    pub last_firmware_update_ms: Option<f64>,
    #[serde(default)]
    pub last_self_test_ms: Option<f64>,
    #[serde(default)]
    pub min_firmware_version: Option<String>,
    #[serde(default)]
    pub lifecycle_state: Option<String>,
    #[serde(default)]
    pub assigned_robot: Option<String>,
}

impl DeviceTree {
    pub fn from_toml_value(value: &toml::Value) -> Self {
        // Deserialize a device tree from a merged TOML value.
        //
        // Parameters:
        // - `value` — merged configuration containing `[fleet]` tables
        //
        // Returns:
        // Parsed device tree (empty when `[fleet]` is absent).
        //
        // Options:
        // None.
        //
        // Example:
        // let tree = DeviceTree::from_toml_value(&resolved.raw);

        value
            .clone()
            .try_into()
            .unwrap_or_else(|_| DeviceTree::default())
    }

    pub fn robot(&self, robot_id: &str) -> Option<&RobotNode> {
        // Look up a robot by id in the fleet tree.
        //
        // Parameters:
        // - `robot_id` — robot identifier
        //
        // Returns:
        // Robot node reference when found.
        //
        // Options:
        // None.
        //
        // Example:
        // let robot = tree.robot("rover-001");

        self.fleet
            .as_ref()?
            .robots
            .iter()
            .find(|r| r.id == robot_id)
    }

    pub fn all_devices(&self) -> Vec<(&RobotNode, &ComputeNode, &DeviceNode)> {
        // Flatten the hierarchy into (robot, compute, device) tuples.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Every device with its parent robot and compute context.
        //
        // Options:
        // None.
        //
        // Example:
        // for (robot, compute, device) in tree.all_devices() { ... }

        let mut out = Vec::new();
        let Some(ref fleet) = self.fleet else {
            return out;
        };
        for robot in &fleet.robots {
            if let Some(ref compute) = robot.compute {
                for device in &compute.devices {
                    out.push((robot, compute, device));
                }
            }
        }
        out
    }

    pub fn hierarchy_lines(&self) -> Vec<String> {
        // Render an indented hierarchy report for CLI output.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Human-readable tree lines.
        //
        // Options:
        // None.
        //
        // Example:
        // for line in tree.hierarchy_lines() { println!("{line}"); }

        let mut lines = Vec::new();
        let Some(ref fleet) = self.fleet else {
            lines.push("(no fleet configured)".into());
            return lines;
        };
        lines.push(format!("fleet: {}", fleet.id));
        for robot in &fleet.robots {
            lines.push(format!(
                "  robot: {} ({})",
                robot.id,
                robot.model.as_deref().unwrap_or("?")
            ));
            if let Some(ref compute) = robot.compute {
                lines.push(format!(
                    "    compute: {} [{}]",
                    compute.id, compute.compute_type
                ));
                for device in &compute.devices {
                    let attach = device
                        .port
                        .as_deref()
                        .or(device.bus.as_deref())
                        .unwrap_or("-");
                    lines.push(format!(
                        "      device: {} ({}) @ {attach}",
                        device.id, device.device_type
                    ));
                }
            }
        }
        for human in &fleet.humans {
            lines.push(format!("  human: {} ({})", human.id, human.role));
        }
        for wearable in &fleet.wearables {
            lines.push(format!(
                "  wearable: {} ({})",
                wearable.id, wearable.device_type
            ));
        }
        for ar in &fleet.ar_devices {
            lines.push(format!("  ar_device: {} ({})", ar.id, ar.device_type));
        }
        for vr in &fleet.vr_devices {
            lines.push(format!("  vr_device: {} ({})", vr.id, vr.device_type));
        }
        for drone in &fleet.drones {
            lines.push(format!("  drone: {} ({})", drone.id, drone.device_type));
        }
        for iot in &fleet.iot_devices {
            lines.push(format!("  iot_device: {} ({})", iot.id, iot.device_type));
        }
        for cc in &fleet.control_center {
            lines.push(format!("  control_center: {}", cc.id));
        }
        lines
    }
}
