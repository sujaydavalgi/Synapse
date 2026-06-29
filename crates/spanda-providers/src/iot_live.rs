//! Optional live Modbus TCP and OPC-UA bridge reads for IoT package dispatch.

use std::io::{Read, Write};
use std::process::{Command, Stdio};

/// Return true when live Modbus hardware reads are enabled.
pub fn live_modbus_enabled() -> bool {
    // Description:
    //     Live modbus enabled.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: bool
    //         Return value from `live_modbus_enabled`.
    //
    // Example:

    //     let result = spanda_providers::iot_live::live_modbus_enabled();

    std::env::var("SPANDA_LIVE_MODBUS")
        .ok()
        .as_deref()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

/// Return true when live OPC-UA bridge reads are enabled.
pub fn live_opcua_enabled() -> bool {
    // Description:
    //     Live opcua enabled.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: bool
    //         Return value from `live_opcua_enabled`.
    //
    // Example:

    //     let result = spanda_providers::iot_live::live_opcua_enabled();

    std::env::var("SPANDA_LIVE_OPCUA")
        .ok()
        .as_deref()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

/// Read a Modbus holding register from live TCP hardware when enabled.
pub fn read_modbus_register_live(address: u16) -> Option<f64> {
    // Description:
    //     Read modbus register live.
    //
    // Inputs:
    //     address: u16
    //         Caller-supplied address.
    //
    // Outputs:
    //     result: Option<f64>
    //         Return value from `read_modbus_register_live`.
    //
    // Example:
    //     let result = spanda_providers::iot_live::read_modbus_register_live(address);

    // Skip live path when the env gate is off.
    if !live_modbus_enabled() {
        return None;
    }

    #[cfg(feature = "live-iot")]
    {
        // Prefer native Modbus TCP when the live-iot feature is enabled.
        if let Ok(value) = read_modbus_tcp(address) {
            return Some(value);
        }
    }

    // Fall back to the Python bridge when pymodbus is installed.
    read_modbus_via_python_bridge(address)
}

/// Read an OPC-UA node via the Python bridge when enabled.
pub fn read_opcua_node_live(node: &str) -> Option<String> {
    // Description:
    //     Read opcua node live.
    //
    // Inputs:
    //     node: &str
    //         Caller-supplied node.
    //
    // Outputs:
    //     result: Option<String>
    //         Return value from `read_opcua_node_live`.
    //
    // Example:
    //     let result = spanda_providers::iot_live::read_opcua_node_live(node);

    // Skip live path when the env gate is off.
    if !live_opcua_enabled() {
        return None;
    }

    read_opcua_via_python_bridge(node)
}

fn live_iot_flag(name: &str) -> bool {
    // Description:
    //     Live iot flag.
    //
    // Inputs:
    //     name: &str
    //         Caller-supplied name.
    //
    // Outputs:
    //     result: bool
    //         Return value from `live_iot_flag`.
    //
    // Example:

    //     let result = spanda_providers::iot_live::live_iot_flag(name);

    std::env::var(name)
        .ok()
        .as_deref()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

/// Return true when live Zigbee bridge reads are enabled.
pub fn live_zigbee_enabled() -> bool {
    // Description:
    //     Live zigbee enabled.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: bool
    //         Return value from `live_zigbee_enabled`.
    //
    // Example:

    //     let result = spanda_providers::iot_live::live_zigbee_enabled();

    live_iot_flag("SPANDA_LIVE_ZIGBEE")
}

/// Return true when live LoRa bridge reads are enabled.
pub fn live_lora_enabled() -> bool {
    // Description:
    //     Live lora enabled.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: bool
    //         Return value from `live_lora_enabled`.
    //
    // Example:

    //     let result = spanda_providers::iot_live::live_lora_enabled();

    live_iot_flag("SPANDA_LIVE_LORA")
}

/// Return true when live Matter bridge reads are enabled.
pub fn live_matter_enabled() -> bool {
    // Description:
    //     Live matter enabled.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: bool
    //         Return value from `live_matter_enabled`.
    //
    // Example:

    //     let result = spanda_providers::iot_live::live_matter_enabled();

    live_iot_flag("SPANDA_LIVE_MATTER")
}

/// Return true when live CAN bus bridge reads are enabled.
pub fn live_canbus_enabled() -> bool {
    // Description:
    //     Live canbus enabled.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: bool
    //         Return value from `live_canbus_enabled`.
    //
    // Example:

    //     let result = spanda_providers::iot_live::live_canbus_enabled();

    live_iot_flag("SPANDA_LIVE_CANBUS")
}

/// Return true when live BACnet bridge reads are enabled.
pub fn live_bacnet_enabled() -> bool {
    live_iot_flag("SPANDA_LIVE_BACNET")
}

/// Return true when live KNX bridge reads are enabled.
pub fn live_knx_enabled() -> bool {
    live_iot_flag("SPANDA_LIVE_KNX")
}

/// Return true when live Thread bridge reads are enabled.
pub fn live_thread_enabled() -> bool {
    live_iot_flag("SPANDA_LIVE_THREAD")
}

/// Return true when live Z-Wave bridge reads are enabled.
pub fn live_zwave_enabled() -> bool {
    live_iot_flag("SPANDA_LIVE_ZWAVE")
}

/// Return true when live Home Assistant bridge reads are enabled.
pub fn live_home_assistant_enabled() -> bool {
    live_iot_flag("SPANDA_LIVE_HOME_ASSISTANT")
}

/// Return true when live automotive radar reads are enabled.
pub fn live_radar_enabled() -> bool {
    live_iot_flag("SPANDA_LIVE_RADAR")
}

/// Return true when live automotive LiDAR reads are enabled.
pub fn live_lidar_enabled() -> bool {
    live_iot_flag("SPANDA_LIVE_LIDAR")
}

/// Return true when live ultrasonic parking sensor reads are enabled.
pub fn live_ultrasonic_enabled() -> bool {
    live_iot_flag("SPANDA_LIVE_ULTRASONIC")
}

pub fn read_zigbee_attribute_live(device: &str, cluster: &str) -> Option<String> {
    // Description:
    //     Read zigbee attribute live.
    //
    // Inputs:
    //     device: &str
    //         Caller-supplied device.
    //     cluster: &str
    //         Caller-supplied cluster.
    //
    // Outputs:
    //     result: Option<String>
    //         Return value from `read_zigbee_attribute_live`.
    //
    // Example:

    //     let result = spanda_providers::iot_live::read_zigbee_attribute_live(device, cluster);

    if !live_zigbee_enabled() {
        return None;
    }
    read_string_via_python_bridge(
        "zigbee_read_attribute",
        vec![
            serde_json::Value::String(device.to_string()),
            serde_json::Value::String(cluster.to_string()),
        ],
    )
}

pub fn read_lora_payload_live(device_id: &str) -> Option<String> {
    // Description:
    //     Read lora payload live.
    //
    // Inputs:
    //     device_id: &str
    //         Caller-supplied device id.
    //
    // Outputs:
    //     result: Option<String>
    //         Return value from `read_lora_payload_live`.
    //
    // Example:

    //     let result = spanda_providers::iot_live::read_lora_payload_live(device_id);

    if !live_lora_enabled() {
        return None;
    }
    read_string_via_python_bridge(
        "lora_read_payload",
        vec![serde_json::Value::String(device_id.to_string())],
    )
}

pub fn read_matter_cluster_live(node: &str, cluster: &str) -> Option<f64> {
    // Description:
    //     Read matter cluster live.
    //
    // Inputs:
    //     node: &str
    //         Caller-supplied node.
    //     cluster: &str
    //         Caller-supplied cluster.
    //
    // Outputs:
    //     result: Option<f64>
    //         Return value from `read_matter_cluster_live`.
    //
    // Example:

    //     let result = spanda_providers::iot_live::read_matter_cluster_live(node, cluster);

    if !live_matter_enabled() {
        return None;
    }
    read_number_via_python_bridge(
        "matter_read_cluster",
        vec![
            serde_json::Value::String(node.to_string()),
            serde_json::Value::String(cluster.to_string()),
        ],
    )
}

pub fn read_bacnet_point_live(device: &str, object_id: &str) -> Option<String> {
    if !live_bacnet_enabled() {
        return None;
    }
    read_string_via_external_cmd_pair("SPANDA_BACNET_CMD", device, object_id).or_else(|| {
        read_string_via_python_bridge(
            "bacnet_read_point",
            vec![
                serde_json::Value::String(device.to_string()),
                serde_json::Value::String(object_id.to_string()),
            ],
        )
    })
}

pub fn read_knx_group_live(address: &str) -> Option<String> {
    if !live_knx_enabled() {
        return None;
    }
    read_string_via_external_cmd_single("SPANDA_KNX_CMD", address).or_else(|| {
        read_string_via_python_bridge(
            "knx_read_group",
            vec![serde_json::Value::String(address.to_string())],
        )
    })
}

pub fn read_thread_endpoint_live(device: &str) -> Option<String> {
    if !live_thread_enabled() {
        return None;
    }
    read_string_via_external_cmd_single("SPANDA_THREAD_CMD", device).or_else(|| {
        read_string_via_python_bridge(
            "thread_read_endpoint",
            vec![serde_json::Value::String(device.to_string())],
        )
    })
}

pub fn read_zwave_value_live(device: &str, command_class: &str) -> Option<String> {
    if !live_zwave_enabled() {
        return None;
    }
    read_string_via_external_cmd_pair("SPANDA_ZWAVE_CMD", device, command_class).or_else(|| {
        read_string_via_python_bridge(
            "zwave_read_value",
            vec![
                serde_json::Value::String(device.to_string()),
                serde_json::Value::String(command_class.to_string()),
            ],
        )
    })
}

pub fn read_home_assistant_state_live(entity_id: &str) -> Option<String> {
    if !live_home_assistant_enabled() {
        return None;
    }
    read_string_via_external_cmd_single("SPANDA_HOME_ASSISTANT_CMD", entity_id).or_else(|| {
        read_string_via_python_bridge(
            "home_assistant_get_state",
            vec![serde_json::Value::String(entity_id.to_string())],
        )
    })
}

pub fn read_canbus_frame_live(can_id: u32) -> Option<f64> {
    // Description:
    //     Read canbus frame live.
    //
    // Inputs:
    //     can_id: u32
    //         Caller-supplied can id.
    //
    // Outputs:
    //     result: Option<f64>
    //         Return value from `read_canbus_frame_live`.
    //
    // Example:

    //     let result = spanda_providers::iot_live::read_canbus_frame_live(can_id);

    if !live_canbus_enabled() {
        return None;
    }
    read_number_via_python_bridge(
        "canbus_read_frame",
        vec![serde_json::Value::Number(can_id.into())],
    )
}

/// Read radar range from `SPANDA_RADAR_CMD` or Python bridge when live mode is enabled.
pub fn read_radar_distance_live(sensor_id: &str) -> Option<f64> {
    if !live_radar_enabled() {
        return None;
    }
    read_distance_via_external_cmd("SPANDA_RADAR_CMD", sensor_id).or_else(|| {
        if bridge_script_path().is_some() {
            read_number_via_python_bridge(
                "radar_read_distance",
                vec![serde_json::Value::String(sensor_id.to_string())],
            )
        } else {
            None
        }
    })
}

/// Read LiDAR range from `SPANDA_LIDAR_CMD` or Python bridge when live mode is enabled.
pub fn read_lidar_distance_live(sensor_id: &str) -> Option<f64> {
    if !live_lidar_enabled() {
        return None;
    }
    read_distance_via_external_cmd("SPANDA_LIDAR_CMD", sensor_id).or_else(|| {
        if bridge_script_path().is_some() {
            read_number_via_python_bridge(
                "lidar_read_distance",
                vec![serde_json::Value::String(sensor_id.to_string())],
            )
        } else {
            None
        }
    })
}

/// Read ultrasonic range from `SPANDA_ULTRASONIC_CMD` or Python bridge when live mode is enabled.
pub fn read_ultrasonic_distance_live(sensor_id: &str) -> Option<f64> {
    if !live_ultrasonic_enabled() {
        return None;
    }
    read_distance_via_external_cmd("SPANDA_ULTRASONIC_CMD", sensor_id).or_else(|| {
        if bridge_script_path().is_some() {
            read_number_via_python_bridge(
                "ultrasonic_read_distance",
                vec![serde_json::Value::String(sensor_id.to_string())],
            )
        } else {
            None
        }
    })
}

fn read_distance_via_external_cmd(cmd_env: &str, sensor_id: &str) -> Option<f64> {
    let template = std::env::var(cmd_env)
        .ok()
        .filter(|value| !value.trim().is_empty())?;
    let command = template.replace("{sensor}", sensor_id);
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()?
        .trim()
        .parse()
        .ok()
}

fn read_string_via_external_cmd_single(cmd_env: &str, arg: &str) -> Option<String> {
    let template = std::env::var(cmd_env)
        .ok()
        .filter(|value| !value.trim().is_empty())?;
    let command = template.replace("{entity}", arg).replace("{device}", arg);
    read_command_stdout(&command)
}

fn read_string_via_external_cmd_pair(cmd_env: &str, device: &str, object_id: &str) -> Option<String> {
    let template = std::env::var(cmd_env)
        .ok()
        .filter(|value| !value.trim().is_empty())?;
    let command = template
        .replace("{device}", device)
        .replace("{object_id}", object_id)
        .replace("{address}", object_id);
    read_command_stdout(&command)
}

fn read_command_stdout(command: &str) -> Option<String> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()?
        .trim()
        .to_string();
    if line.is_empty() { None } else { Some(line) }
}

fn read_string_via_python_bridge(fn_name: &str, args: Vec<serde_json::Value>) -> Option<String> {
    // Description:
    //     Read string via python bridge.
    //
    // Inputs:
    //     fn_name: &str
    //         Caller-supplied fn name.
    //     args: Vec<serde_json::Value>
    //         Caller-supplied args.
    //
    // Outputs:
    //     result: Option<String>
    //         Return value from `read_string_via_python_bridge`.
    //
    // Example:

    //     let result = spanda_providers::iot_live::read_string_via_python_bridge(fn_name, args);

    match call_python_bridge(fn_name, args)?.get("result") {
        Some(serde_json::Value::String(text)) if !text.is_empty() => Some(text.clone()),
        _ => None,
    }
}

fn read_number_via_python_bridge(fn_name: &str, args: Vec<serde_json::Value>) -> Option<f64> {
    // Description:
    //     Read number via python bridge.
    //
    // Inputs:
    //     fn_name: &str
    //         Caller-supplied fn name.
    //     args: Vec<serde_json::Value>
    //         Caller-supplied args.
    //
    // Outputs:
    //     result: Option<f64>
    //         Return value from `read_number_via_python_bridge`.
    //
    // Example:

    //     let result = spanda_providers::iot_live::read_number_via_python_bridge(fn_name, args);

    match call_python_bridge(fn_name, args)?.get("result") {
        Some(serde_json::Value::Number(n)) => n.as_f64(),
        Some(serde_json::Value::String(text)) => text.parse().ok(),
        _ => None,
    }
}

#[cfg(feature = "live-iot")]
fn read_modbus_tcp(address: u16) -> Result<f64, String> {
    // Description:
    //     Read modbus tcp.
    //
    // Inputs:
    //     address: u16
    //         Caller-supplied address.
    //
    // Outputs:
    //     result: Result<f64, String>
    //         Return value from `read_modbus_tcp`.
    //
    // Example:

    //     let result = spanda_providers::iot_live::read_modbus_tcp(address);

    use modbus::{tcp, Client};

    let host = std::env::var("SPANDA_MODBUS_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = std::env::var("SPANDA_MODBUS_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(502);
    let unit = std::env::var("SPANDA_MODBUS_UNIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1u8);
    let zero_based = address.saturating_sub(40001);
    let endpoint = format!("{host}:{port}");
    let mut transport =
        tcp::Transport::new(&endpoint).map_err(|e| format!("modbus connect failed: {e}"))?;
    transport.set_uid(unit);
    let values = transport
        .read_holding_registers(zero_based, 1)
        .map_err(|e| format!("modbus read failed: {e}"))?;
    values
        .first()
        .copied()
        .map(f64::from)
        .ok_or_else(|| "modbus read returned no registers".into())
}

fn read_modbus_via_python_bridge(address: u16) -> Option<f64> {
    // Description:
    //     Read modbus via python bridge.
    //
    // Inputs:
    //     address: u16
    //         Caller-supplied address.
    //
    // Outputs:
    //     result: Option<f64>
    //         Return value from `read_modbus_via_python_bridge`.
    //
    // Example:

    //     let result = spanda_providers::iot_live::read_modbus_via_python_bridge(address);

    let host = std::env::var("SPANDA_MODBUS_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = std::env::var("SPANDA_MODBUS_PORT").unwrap_or_else(|_| "502".into());
    let response = call_python_bridge(
        "modbus_read_register",
        vec![
            serde_json::Value::String(host),
            serde_json::Value::String(port),
            serde_json::Value::Number(address.into()),
        ],
    )?;
    match response.get("result") {
        Some(serde_json::Value::Number(n)) => n.as_f64(),
        _ => None,
    }
}

fn read_opcua_via_python_bridge(node: &str) -> Option<String> {
    // Description:
    //     Read opcua via python bridge.
    //
    // Inputs:
    //     node: &str
    //         Caller-supplied node.
    //
    // Outputs:
    //     result: Option<String>
    //         Return value from `read_opcua_via_python_bridge`.
    //
    // Example:

    //     let result = spanda_providers::iot_live::read_opcua_via_python_bridge(node);

    let endpoint = std::env::var("SPANDA_OPCUA_ENDPOINT")
        .unwrap_or_else(|_| "opc.tcp://127.0.0.1:4840".into());
    let response = call_python_bridge(
        "opcua_read_node",
        vec![
            serde_json::Value::String(endpoint),
            serde_json::Value::String(node.to_string()),
        ],
    )?;
    match response.get("result") {
        Some(serde_json::Value::String(value)) => Some(value.clone()),
        Some(serde_json::Value::Number(n)) => n.as_f64().map(|v| v.to_string()),
        _ => None,
    }
}

fn call_python_bridge(fn_name: &str, args: Vec<serde_json::Value>) -> Option<serde_json::Value> {
    // Description:
    //     Call python bridge.
    //
    // Inputs:
    //     fn_name: &str
    //         Caller-supplied fn name.
    //     args: Vec<serde_json::Value>
    //         Caller-supplied args.
    //
    // Outputs:
    //     result: Option<serde_json::Value>
    //         Return value from `call_python_bridge`.
    //
    // Example:

    //     let result = spanda_providers::iot_live::call_python_bridge(fn_name, args);

    let script = bridge_script_path()?;
    let python = std::env::var("SPANDA_PYTHON").unwrap_or_else(|_| "python3".into());
    let request = serde_json::json!({ "fn": fn_name, "args": args });
    let mut child = Command::new(python)
        .arg(script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    {
        let stdin = child.stdin.as_mut()?;
        let payload = serde_json::to_string(&request).ok()?;
        stdin.write_all(payload.as_bytes()).ok()?;
    }
    let mut stdout = String::new();
    child.stdout.as_mut()?.read_to_string(&mut stdout).ok()?;
    let _ = child.wait();
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).ok()?;
    if parsed.get("ok") == Some(&serde_json::Value::Bool(true)) {
        Some(parsed)
    } else {
        None
    }
}

fn bridge_script_path() -> Option<String> {
    // Description:
    //     Bridge script path.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: Option<String>
    //         Return value from `bridge_script_path`.
    //
    // Example:

    //     let result = spanda_providers::iot_live::bridge_script_path();

    if let Ok(path) = std::env::var("SPANDA_PYTHON_BRIDGE") {
        if std::path::Path::new(&path).is_file() {
            return Some(path);
        }
    }
    let candidates = [
        "scripts/spanda_python_bridge.py".to_string(),
        format!(
            "{}/../../scripts/spanda_python_bridge.py",
            env!("CARGO_MANIFEST_DIR")
        ),
    ];
    for candidate in candidates {
        if std::path::Path::new(&candidate).is_file() {
            return Some(candidate);
        }
    }
    std::env::current_dir()
        .ok()
        .map(|cwd| cwd.join("scripts/spanda_python_bridge.py"))
        .filter(|p| p.is_file())
        .map(|p| p.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::radar_env_lock::RadarEnvLock;

    #[test]
    fn live_modbus_disabled_by_default() {
        // Description:
        //     Live modbus disabled by default.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_providers::iot_live::live_modbus_disabled_by_default();

        std::env::remove_var("SPANDA_LIVE_MODBUS");
        assert!(!live_modbus_enabled());
        assert!(read_modbus_register_live(40001).is_none());
    }

    #[test]
    fn live_radar_disabled_by_default() {
        let _lock = RadarEnvLock::acquire().expect("radar env lock");
        std::env::remove_var("SPANDA_LIVE_RADAR");
        assert!(!live_radar_enabled());
        assert!(read_radar_distance_live("front-radar").is_none());
    }

    #[test]
    fn live_radar_external_cmd_parses_stdout() {
        let _lock = RadarEnvLock::acquire().expect("radar env lock");
        std::env::remove_var("SPANDA_LIVE_RADAR");
        std::env::remove_var("SPANDA_RADAR_CMD");
        std::env::set_var("SPANDA_LIVE_RADAR", "1");
        std::env::set_var("SPANDA_RADAR_CMD", "echo 42.5");
        assert_eq!(read_radar_distance_live("front-radar"), Some(42.5));
        std::env::remove_var("SPANDA_LIVE_RADAR");
        std::env::remove_var("SPANDA_RADAR_CMD");
    }

    #[test]
    fn live_bacnet_external_cmd_parses_stdout() {
        std::env::remove_var("SPANDA_LIVE_BACNET");
        std::env::remove_var("SPANDA_BACNET_CMD");
        std::env::set_var("SPANDA_LIVE_BACNET", "1");
        std::env::set_var("SPANDA_BACNET_CMD", "echo live-bacnet:{device}:{object_id}");
        assert_eq!(
            read_bacnet_point_live("ahu-12", "present-value"),
            Some("live-bacnet:ahu-12:present-value".into())
        );
        std::env::remove_var("SPANDA_LIVE_BACNET");
        std::env::remove_var("SPANDA_BACNET_CMD");
    }
}
