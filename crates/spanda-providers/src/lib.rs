//! Official package provider bootstrap and transport adapter wiring.
//!
pub mod anomaly_onnx;
pub mod automotive_hub;
pub mod bootstrap;
pub mod hri_backends;
pub mod iot_hub;
pub mod iot_live;
pub mod package_dispatch;
pub mod package_stubs;
pub mod transport_adapter;

pub use automotive_hub::{
    read_lidar_distance, read_radar_distance, read_ultrasonic_distance, seed_automotive_demos,
};
pub use bootstrap::{
    bootstrap_default_providers, bootstrap_providers_for_packages, official_package_for_transport,
    sync_comm_bus_for_official_packages,
};
pub use iot_hub::{hub_stats, register_device, seed_modbus_demo_register};
pub use package_dispatch::{
    dispatch_official_package_call, official_package_for_module, ProviderDispatchContext,
};
pub use transport_adapter::{adapter_config_to_runtime, TransportAdapterProvider};
