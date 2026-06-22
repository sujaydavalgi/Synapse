//! In-process ROS2 transport orchestration (native rclrs, daemon, Python bridge).
//!
use spanda_runtime::RuntimeValue;

use crate::daemon::{daemon_publish, daemon_service_call, daemon_subscribe};
use crate::live_bridge::{
    try_ros2_bridge_publish, try_ros2_bridge_service_call, try_ros2_bridge_subscribe,
};
use crate::native;

fn payload_string(value: &RuntimeValue) -> String {
    // Serialize a runtime value into a plain string for ROS2 wire payloads.
    match value {
        RuntimeValue::String { value } => value.clone(),
        RuntimeValue::Number { value, .. } => value.to_string(),
        RuntimeValue::Bool { value } => value.to_string(),
        other => format!("{other:?}"),
    }
}

/// Publish on ROS2 when `SPANDA_ROS2_RCLRS` is set, trying native, daemon, then bridge.
pub fn try_rclrs_publish(topic: &str, value: &RuntimeValue) -> bool {
    // Try rclrs publish.
    //
    // Parameters:
    // - `topic` — ROS2 topic path
    // - `value` — runtime payload
    //
    // Returns:
    // `true` when a backend handled the publish.
    //
    // Options:
    // None.
    //
    // Example:
    // let ok = try_rclrs_publish("/scan", &value);

    // Skip when in-process ROS2 transport is disabled.
    if !crate::rclrs_enabled() {
        return false;
    }

    // Prefer native rclrs, then daemon, then Python bridge.
    let payload = payload_string(value);
    if native::publish(topic, &payload) {
        return true;
    }
    if daemon_publish(topic, &payload) {
        return true;
    }
    try_ros2_bridge_publish(topic, &payload)
}

/// Subscribe on ROS2 when `SPANDA_ROS2_RCLRS` is set.
pub fn try_rclrs_subscribe(topic: &str) -> bool {
    // Try rclrs subscribe.
    //
    // Parameters:
    // - `topic` — ROS2 topic path
    //
    // Returns:
    // `true` when a backend handled the subscribe.
    //
    // Options:
    // None.
    //
    // Example:
    // let ok = try_rclrs_subscribe("/scan");

    // Skip when in-process ROS2 transport is disabled.
    if !crate::rclrs_enabled() {
        return false;
    }

    // Prefer native rclrs, then daemon, then Python bridge.
    if native::subscribe(topic) {
        return true;
    }
    if daemon_subscribe(topic) {
        return true;
    }
    try_ros2_bridge_subscribe(topic)
}

/// Call a ROS2 service when `SPANDA_ROS2_RCLRS` is set.
pub fn try_rclrs_service_call(service: &str, service_type: &str, request: &str) -> bool {
    // Try rclrs service call.
    //
    // Parameters:
    // - `service` — service name
    // - `service_type` — ROS2 service type
    // - `request` — serialized request payload
    //
    // Returns:
    // `true` when a backend handled the call.
    //
    // Options:
    // None.
    //
    // Example:
    // let ok = try_rclrs_service_call("/trigger", "std_srvs/srv/Trigger", "{}");

    // Skip when in-process ROS2 transport is disabled.
    if !crate::rclrs_enabled() {
        return false;
    }

    // Prefer native rclrs, then daemon, then Python bridge.
    if native::service_call(service, service_type, request) {
        return true;
    }
    if daemon_service_call(service, service_type, request) {
        return true;
    }
    try_ros2_bridge_service_call(service, service_type, request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rclrs_off_by_default() {
        std::env::remove_var("SPANDA_ROS2_RCLRS");
        assert!(!crate::rclrs_enabled());
    }
}
