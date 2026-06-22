//! ROS 2 transport backend extracted from Spanda core for lean-core package architecture.
//!
//! Provides native rclrs dynamic loading, an rclpy daemon bridge, and optional live
//! `ros2` CLI / Python bridge fallbacks. Spanda core retains thin `RuntimeValue`
//! compatibility shims that delegate here.
//!
pub mod adapter;
pub mod daemon;
pub mod live_bridge;
mod python_bridge;
pub mod rclrs;

#[cfg_attr(target_arch = "wasm32", path = "native_stub.rs")]
#[cfg_attr(not(target_arch = "wasm32"), path = "native.rs")]
mod native_loader;

pub mod native {
    pub use super::native_loader::*;
}

pub use adapter::Ros2TransportAdapter;
pub use daemon::{
    daemon_publish, daemon_script_path, daemon_service_call, daemon_subscribe, python_available,
};

/// Whether in-process ROS2 transport is enabled (`SPANDA_ROS2_RCLRS` env var).
pub fn rclrs_enabled() -> bool {
    std::env::var("SPANDA_ROS2_RCLRS").is_ok()
}

/// Alias for `rclrs_enabled`.
pub fn rclrs_available() -> bool {
    rclrs_enabled()
}

pub fn native_sdk_available() -> bool {
    native::sdk_available()
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

pub fn try_native_publish(topic: &str, payload: &str) -> bool {
    native::publish(topic, payload)
}

pub fn try_native_subscribe(topic: &str) -> bool {
    native::subscribe(topic)
}

pub fn try_native_service_call(service: &str, service_type: &str, request: &str) -> bool {
    native::service_call(service, service_type, request)
}
