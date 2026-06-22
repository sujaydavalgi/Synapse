//! Compatibility shim: ROS2 transport orchestration (delegates to `spanda-transport-ros2`).
//!
pub use spanda_transport_ros2::{
    init_node, native_sdk_available, rclrs_available, rclrs_enabled,
};
pub use spanda_transport_ros2::rclrs::{
    try_rclrs_publish, try_rclrs_service_call, try_rclrs_subscribe,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rclrs_off_by_default() {
        std::env::remove_var("SPANDA_ROS2_RCLRS");
        assert!(!rclrs_enabled());
    }
}
