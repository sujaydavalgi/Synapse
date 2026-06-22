//! Optional live ROS2 hooks via `ros2` CLI and the Python bridge subprocess.
//!
use std::process::{Command, Stdio};

use crate::python_bridge;

pub fn ros2_live_enabled() -> bool {
    std::env::var("SPANDA_ROS2_LIVE").is_ok() || std::env::var("SPANDA_ROS2_RCLRS").is_ok()
}

pub fn ros2_native_enabled() -> bool {
    std::env::var("SPANDA_ROS2_NATIVE").is_ok()
}

pub fn try_ros2_native_publish(topic: &str, payload: &str) -> bool {
    if !ros2_native_enabled() {
        return false;
    }
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
        .stdout(Stdio::null())
        .stderr(Stdio::null())
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
        .stdout(Stdio::null())
        .stderr(Stdio::null())
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
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn try_ros2_bridge_publish(topic: &str, payload: &str) -> bool {
    if !ros2_live_enabled() {
        return false;
    }
    python_bridge::invoke_python_bridge(
        "ros2_publish",
        &[topic.to_string(), payload.to_string()],
    )
}

pub fn try_ros2_bridge_subscribe(topic: &str) -> bool {
    if !ros2_live_enabled() {
        return false;
    }
    python_bridge::invoke_python_bridge("ros2_subscribe", &[topic.to_string()])
}

pub fn try_ros2_bridge_service_call(service: &str, service_type: &str, request: &str) -> bool {
    if !ros2_live_enabled() {
        return false;
    }
    python_bridge::invoke_python_bridge(
        "ros2_service_call",
        &[
            service.to_string(),
            service_type.to_string(),
            request.to_string(),
        ],
    )
}

pub fn try_ros2_publish(topic: &str, payload: &str) -> bool {
    if ros2_native_enabled() {
        return try_ros2_native_publish(topic, payload);
    }
    try_ros2_bridge_publish(topic, payload)
}

pub fn try_ros2_subscribe(topic: &str) -> bool {
    if ros2_native_enabled() {
        return try_ros2_native_subscribe(topic);
    }
    try_ros2_bridge_subscribe(topic)
}

pub fn try_ros2_service_call(service: &str, service_type: &str, request: &str) -> bool {
    if ros2_native_enabled() {
        return try_ros2_native_service_call(service, service_type, request);
    }
    try_ros2_bridge_service_call(service, service_type, request)
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
        let _ = try_ros2_publish("/spanda/test", "hi");
        let _ = try_ros2_subscribe("/spanda/test");
        let _ = try_ros2_service_call("/spanda/srv", "std_srvs/srv/Trigger", "{}");
        std::env::remove_var("SPANDA_ROS2_NATIVE");
    }
}
