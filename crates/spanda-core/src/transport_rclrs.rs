//! In-process ROS2 via dynamically loaded native rclrs, rclpy daemon, or Python bridge.
//!
//! Priority when `SPANDA_ROS2_RCLRS=1`:
//! 1. Native `rclrs` shared library (`libspanda_ros2_rclrs_native`, built with sourced ROS 2)
//! 2. Persistent rclpy daemon subprocess
//! 3. Per-call Python bridge fallback

use crate::runtime::RuntimeValue;
use crate::transport_live::{
    try_ros2_bridge_publish, try_ros2_bridge_service_call, try_ros2_bridge_subscribe,
};
use crate::transport_rclrs_daemon::{daemon_publish, daemon_service_call, daemon_subscribe};
use crate::transport_rclrs_native as native;

pub fn rclrs_enabled() -> bool {
    std::env::var("SPANDA_ROS2_RCLRS").is_ok()
}

pub fn rclrs_available() -> bool {
    rclrs_enabled()
}

fn payload_string(value: &RuntimeValue) -> String {
    match value {
        RuntimeValue::String { value } => value.clone(),
        RuntimeValue::Number { value, .. } => value.to_string(),
        RuntimeValue::Bool { value } => value.to_string(),
        other => format!("{other:?}"),
    }
}

pub fn try_rclrs_publish(topic: &str, value: &RuntimeValue) -> bool {
    if !rclrs_enabled() {
        return false;
    }
    if native::publish(topic, &payload_string(value)) {
        return true;
    }
    if daemon_publish(topic, value) {
        return true;
    }
    try_ros2_bridge_publish(topic, value)
}

pub fn try_rclrs_subscribe(topic: &str) -> bool {
    if !rclrs_enabled() {
        return false;
    }
    if native::subscribe(topic) {
        return true;
    }
    if daemon_subscribe(topic) {
        return true;
    }
    try_ros2_bridge_subscribe(topic)
}

pub fn try_rclrs_service_call(service: &str, service_type: &str, request: &str) -> bool {
    if !rclrs_enabled() {
        return false;
    }
    if native::service_call(service, service_type, request) {
        return true;
    }
    if daemon_service_call(service, service_type, request) {
        return true;
    }
    try_ros2_bridge_service_call(service, service_type, request)
}

pub fn init_node(name: &str) -> Result<(), String> {
    if native::sdk_available() {
        return native::init_node(name);
    }
    if daemon_subscribe("/spanda/rclrs/init") {
        let _ = name;
        Ok(())
    } else {
        Err(
            "ROS2 rclrs SDK unavailable — build libspanda_ros2_rclrs_native and source ROS 2"
                .into(),
        )
    }
}

pub fn native_sdk_available() -> bool {
    native::sdk_available()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rclrs_off_by_default() {
        std::env::remove_var("SPANDA_ROS2_RCLRS");
        assert!(!rclrs_enabled());
    }
}
