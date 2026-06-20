//! Optional live transport hooks via the Python bridge subprocess.

use crate::bridge::protocol::call_subprocess_bridge;
use crate::bridge::python::{bridge_script_path, python_available};
use crate::runtime::RuntimeValue;
use std::path::PathBuf;
use std::process::Command;

fn python_cmd() -> Option<String> {
    for cmd in ["python3", "python"] {
        if Command::new(cmd)
            .arg("-c")
            .arg("import sys")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return Some(cmd.to_string());
        }
    }
    None
}

fn payload_string(value: &RuntimeValue) -> String {
    match value {
        RuntimeValue::String { value } => value.clone(),
        RuntimeValue::Number { value, .. } => value.to_string(),
        RuntimeValue::Bool { value } => value.to_string(),
        other => format!("{other:?}"),
    }
}

fn invoke_bridge(fn_name: &str, args: &[RuntimeValue]) -> bool {
    if !python_available() {
        return false;
    }
    let script = match bridge_script_path() {
        Some(path) => path,
        None => return false,
    };
    let python = match python_cmd() {
        Some(cmd) => cmd,
        None => return false,
    };
    let decl = crate::foundations::ExternFnDecl {
        name: fn_name.into(),
        library: Some("python".into()),
        bridge: crate::foundations::BridgeKind::Python,
        params: vec![],
        return_type: crate::ast::SpandaType::String,
        span: crate::ast::Span {
            start: crate::ast::SourceLocation {
                line: 0,
                column: 0,
                offset: 0,
            },
            end: crate::ast::SourceLocation {
                line: 0,
                column: 0,
                offset: 0,
            },
        },
    };
    call_subprocess_bridge(
        "Python",
        &PathBuf::from(python),
        &[script.to_str().unwrap()],
        &decl,
        args,
    )
    .is_ok()
}

pub fn ros2_live_enabled() -> bool {
    std::env::var("SPANDA_ROS2_LIVE").is_ok() || std::env::var("SPANDA_ROS2_RCLRS").is_ok()
}

pub fn ros2_native_enabled() -> bool {
    std::env::var("SPANDA_ROS2_NATIVE").is_ok()
}

pub fn mqtt_live_enabled() -> bool {
    std::env::var("SPANDA_MQTT_LIVE").is_ok()
}

pub fn try_ros2_native_publish(topic: &str, value: &RuntimeValue) -> bool {
    if !ros2_native_enabled() {
        return false;
    }
    let payload = payload_string(value);
    let message = format!(
        "{{data: \"{}\"}}",
        payload.replace('\\', "\\\\").replace('"', "\\\"")
    );
    Command::new("ros2")
        .args([
            "topic",
            "pub",
            "--once",
            topic,
            "std_msgs/msg/String",
            &message,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn try_ros2_native_subscribe(topic: &str) -> bool {
    if !ros2_native_enabled() {
        return false;
    }
    Command::new("ros2")
        .args(["topic", "echo", topic, "--once"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn try_ros2_native_service_call(service: &str, service_type: &str, request: &str) -> bool {
    if !ros2_native_enabled() {
        return false;
    }
    Command::new("ros2")
        .args(["service", "call", service, service_type, request])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn try_ros2_publish(topic: &str, value: &RuntimeValue) -> bool {
    if ros2_native_enabled() {
        return try_ros2_native_publish(topic, value);
    }
    try_ros2_bridge_publish(topic, value)
}

pub fn try_ros2_bridge_publish(topic: &str, value: &RuntimeValue) -> bool {
    if !ros2_live_enabled() {
        return false;
    }
    invoke_bridge(
        "ros2_publish",
        &[
            RuntimeValue::String {
                value: topic.to_string(),
            },
            RuntimeValue::String {
                value: payload_string(value),
            },
        ],
    )
}

pub fn try_ros2_subscribe(topic: &str) -> bool {
    if ros2_native_enabled() {
        return try_ros2_native_subscribe(topic);
    }
    try_ros2_bridge_subscribe(topic)
}

pub fn try_ros2_bridge_subscribe(topic: &str) -> bool {
    if !ros2_live_enabled() {
        return false;
    }
    invoke_bridge(
        "ros2_subscribe",
        &[RuntimeValue::String {
            value: topic.to_string(),
        }],
    )
}

pub fn try_ros2_service_call(service: &str, service_type: &str, request: &str) -> bool {
    if ros2_native_enabled() {
        return try_ros2_native_service_call(service, service_type, request);
    }
    try_ros2_bridge_service_call(service, service_type, request)
}

pub fn try_ros2_bridge_service_call(service: &str, service_type: &str, request: &str) -> bool {
    if !ros2_live_enabled() {
        return false;
    }
    invoke_bridge(
        "ros2_service_call",
        &[
            RuntimeValue::String {
                value: service.to_string(),
            },
            RuntimeValue::String {
                value: service_type.to_string(),
            },
            RuntimeValue::String {
                value: request.to_string(),
            },
        ],
    )
}

pub fn try_mqtt_publish(topic: &str, value: &RuntimeValue) -> bool {
    if !mqtt_live_enabled() {
        return false;
    }
    invoke_bridge(
        "mqtt_publish",
        &[
            RuntimeValue::String {
                value: topic.to_string(),
            },
            RuntimeValue::String {
                value: payload_string(value),
            },
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn live_flags_default_off() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::remove_var("SPANDA_ROS2_LIVE");
        std::env::remove_var("SPANDA_ROS2_NATIVE");
        assert!(!ros2_live_enabled());
        assert!(!ros2_native_enabled());
    }

    #[test]
    fn native_uses_ros2_cli_when_enabled() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::set_var("SPANDA_ROS2_NATIVE", "1");
        assert!(ros2_native_enabled());
        let _ = try_ros2_publish("/spanda/test", &RuntimeValue::String { value: "hi".into() });
        let _ = try_ros2_subscribe("/spanda/test");
        let _ = try_ros2_service_call("/spanda/srv", "std_srvs/srv/Trigger", "{}");
        std::env::remove_var("SPANDA_ROS2_NATIVE");
    }
}
