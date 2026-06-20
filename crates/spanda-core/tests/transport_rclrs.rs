use spanda_core::runtime::RuntimeValue;
use spanda_core::transport_rclrs;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn native_sdk_reports_unavailable_without_ros_by_default() {
    assert!(!transport_rclrs::native_sdk_available());
}

#[test]
fn rclrs_transport_chain_respects_env_flag() {
    let _lock = ENV_LOCK.lock().unwrap();
    std::env::remove_var("SPANDA_ROS2_RCLRS");
    assert!(!transport_rclrs::rclrs_enabled());

    std::env::set_var("SPANDA_ROS2_RCLRS", "1");
    assert!(transport_rclrs::rclrs_enabled());
    assert!(!transport_rclrs::native_sdk_available());

    let value = RuntimeValue::String {
        value: "hello".into(),
    };
    let _ = transport_rclrs::try_rclrs_publish("/spanda/test", &value);

    std::env::remove_var("SPANDA_ROS2_RCLRS");
}
