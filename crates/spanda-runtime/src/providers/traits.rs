//! Lean-core provider trait contracts for optional domain packages.
//!
use super::transport_types::{AdapterMessage, TransportConfig};
use super::types::{ProviderMetadata, ProviderResult};
use crate::hal_config::HalMemberConfig;
use crate::robot_state::RobotState;
use crate::value::{MotionCommand, RuntimeValue};
use spanda_ast::comm_decl::TransportKind;

/// Read sensor samples from hardware or simulation backends.
pub trait SensorProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn read(&mut self, sensor_name: &str, sensor_type: &str, topic: Option<&str>) -> RuntimeValue;
    fn list_sensors(&self) -> Vec<String>;
}

/// Execute actuator and motion commands.
pub trait ActuatorProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn execute_motion(&mut self, cmd: MotionCommand);
    fn emergency_stop(&mut self, active: bool);
    fn robot_state(&self) -> RobotState;
}

/// Wireless and network connectivity state.
pub trait ConnectivityProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn connect(&mut self, channel: &str) -> ProviderResult<()>;
    fn disconnect(&mut self, channel: &str);
    fn is_connected(&self, channel: &str) -> bool;
    fn signal_strength_dbm(&self, channel: &str) -> Option<f64>;
}

/// GPS/GNSS and geospatial positioning.
pub trait PositioningProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn read_fix(&mut self) -> RuntimeValue;
    fn accuracy_meters(&self) -> Option<f64>;
}

/// Pub/sub, service, and action transports (ROS2, MQTT, DDS, WebSocket).
pub trait TransportProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn kind(&self) -> TransportKind;
    fn connect(&mut self, config: &TransportConfig) -> Result<(), String>;
    fn disconnect(&mut self);
    fn is_connected(&self) -> bool;
    fn publish(&mut self, topic: &str, message_type: &str, value: RuntimeValue);
    fn subscribe(&mut self, topic: &str);
    fn receive(&mut self, topic: &str) -> Option<RuntimeValue>;
    fn call_service(
        &mut self,
        service: &str,
        service_type: &str,
        request: Option<RuntimeValue>,
    ) -> RuntimeValue;
    fn send_action(&mut self, action: &str, action_type: &str, goal: RuntimeValue) -> RuntimeValue;
    fn published(&self) -> Vec<AdapterMessage>;
}

/// Cryptographic operations delegated to `spanda-security` or package backends.
pub trait CryptoProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn hash(&self, algorithm: &str, payload: &[u8]) -> ProviderResult<Vec<u8>>;
    fn sign(&self, key_id: &str, payload: &[u8]) -> ProviderResult<Vec<u8>>;
    fn verify(&self, key_id: &str, payload: &[u8], signature: &[u8]) -> ProviderResult<bool>;
}

/// Path planning and navigation stacks (Nav2, custom planners).
pub trait NavigationProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn navigate_to(&mut self, goal: RuntimeValue) -> ProviderResult<RuntimeValue>;
    fn cancel_navigation(&mut self);
    fn navigation_status(&self) -> RuntimeValue;
}

/// SLAM and mapping backends.
pub trait SlamProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn localize(&mut self) -> ProviderResult<RuntimeValue>;
    fn update_map(&mut self, sensor_frame: RuntimeValue) -> ProviderResult<RuntimeValue>;
    fn export_map(&self) -> ProviderResult<RuntimeValue>;
}

/// Vision and perception models (OpenCV, YOLO, ONNX, etc.).
pub trait VisionProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn detect(&mut self, request: RuntimeValue) -> RuntimeValue;
    fn classify(&mut self, request: RuntimeValue) -> RuntimeValue;
    fn embed(&mut self, request: RuntimeValue) -> RuntimeValue;
}

/// Multi-robot fleet orchestration backends.
pub trait FleetProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn register_member(&mut self, member_id: &str, metadata: RuntimeValue) -> ProviderResult<()>;
    fn dispatch_task(
        &mut self,
        member_id: &str,
        task: RuntimeValue,
    ) -> ProviderResult<RuntimeValue>;
    fn member_status(&self, member_id: &str) -> Option<RuntimeValue>;
}

/// Physics and robot simulators (Gazebo, Webots, built-in sim).
pub trait SimulationProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn reset(&mut self);
    fn step(&mut self, dt_ms: f64);
    fn observe(&self) -> RobotState;
}

/// Predictive maintenance and health monitoring.
pub trait MaintenanceProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn record_metric(&mut self, component: &str, metric: RuntimeValue);
    fn health_report(&self, component: &str) -> RuntimeValue;
}

/// Audit ledger and provenance anchoring.
pub trait LedgerProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn append(&mut self, record: RuntimeValue) -> ProviderResult<String>;
    fn anchor(&mut self, digest: &[u8]) -> ProviderResult<String>;
}

/// Cloud upload, telemetry, and remote command channels.
pub trait CloudProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn upload(&mut self, path: &str, payload: RuntimeValue) -> ProviderResult<()>;
    fn invoke(&mut self, endpoint: &str, request: RuntimeValue) -> ProviderResult<RuntimeValue>;
}

/// ROS-specific bridge operations beyond generic transport.
pub trait RosProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn create_node(&mut self, name: &str, namespace: Option<&str>) -> ProviderResult<()>;
    fn spin_once(&mut self, timeout_ms: u64) -> ProviderResult<()>;
    fn declare_parameter(&mut self, name: &str, default: RuntimeValue) -> ProviderResult<()>;
}

/// HAL board access for I2C/SPI/GPIO/UART peripherals.
pub trait HalProvider: Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn configure(&mut self, members: &[HalMemberConfig]);
    fn read_gpio(&self, name: &str) -> bool;
    fn write_gpio(&mut self, name: &str, value: bool);
}
