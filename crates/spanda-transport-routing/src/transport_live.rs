//! RuntimeValue live transport hooks for ROS2 and MQTT backends.
//!
use spanda_runtime::value::RuntimeValue;

pub use spanda_transport_mqtt::{mqtt_live_enabled, try_mqtt_publish as try_mqtt_publish_str};
pub use spanda_transport_ros2::live_bridge::{
    ros2_live_enabled, ros2_native_enabled, try_ros2_bridge_publish,
    try_ros2_bridge_service_call, try_ros2_bridge_subscribe, try_ros2_native_publish,
    try_ros2_native_service_call, try_ros2_native_subscribe, try_ros2_publish as try_ros2_publish_str,
    try_ros2_service_call as try_ros2_service_call_str,
    try_ros2_subscribe as try_ros2_subscribe_str,
};

fn payload_string(value: &RuntimeValue) -> String {
    match value {
        RuntimeValue::String { value } => value.clone(),
        RuntimeValue::Number { value, .. } => value.to_string(),
        RuntimeValue::Bool { value } => value.to_string(),
        other => format!("{other:?}"),
    }
}

pub fn try_ros2_publish(topic: &str, value: &RuntimeValue) -> bool {
    try_ros2_publish_str(topic, &payload_string(value))
}

pub fn try_ros2_subscribe(topic: &str) -> bool {
    try_ros2_subscribe_str(topic)
}

pub fn try_ros2_service_call(service: &str, service_type: &str, request: &str) -> bool {
    try_ros2_service_call_str(service, service_type, request)
}

pub fn try_mqtt_publish(topic: &str, value: &RuntimeValue) -> bool {
    try_mqtt_publish_str(topic, &payload_string(value))
}
