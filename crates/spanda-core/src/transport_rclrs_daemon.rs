//! Compatibility shim: ROS2 rclpy daemon moved to `spanda-transport-ros2`.
//!
use crate::runtime::RuntimeValue;

pub use spanda_transport_ros2::daemon::{
    daemon_service_call as daemon_service_call_str,
    daemon_subscribe as daemon_subscribe_str,
};

fn payload_string(value: &RuntimeValue) -> String {
    match value {
        RuntimeValue::String { value } => value.clone(),
        RuntimeValue::Number { value, .. } => value.to_string(),
        RuntimeValue::Bool { value } => value.to_string(),
        other => format!("{other:?}"),
    }
}

pub fn daemon_publish(topic: &str, value: &RuntimeValue) -> bool {
    spanda_transport_ros2::daemon_publish(topic, &payload_string(value))
}

pub fn daemon_subscribe(topic: &str) -> bool {
    daemon_subscribe_str(topic)
}

pub fn daemon_service_call(service: &str, service_type: &str, request: &str) -> bool {
    daemon_service_call_str(service, service_type, request)
}
