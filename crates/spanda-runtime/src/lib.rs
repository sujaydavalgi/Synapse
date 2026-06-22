//! Spanda runtime kernel primitives extracted for the Phase 4 lean-core split.
//!
pub mod classification;
pub mod environment;
pub mod error;
pub mod hal_config;
pub mod host;
pub mod provider_types;
pub mod providers;
pub mod robot_state;
pub mod robotics;
pub mod scheduler;
pub mod value;

pub use classification::{
    module_classifications, official_package_names, ModuleClassification, ModuleOwnership,
};
pub use environment::Environment;
pub use error::RuntimeError;
pub use host::{imports_enable_navigation, imports_enable_slam, RuntimeHost};
pub use hal_config::HalMemberConfig;
pub use provider_types::{
    ProviderCapability, ProviderCapabilitySet, ProviderError, ProviderId, ProviderMetadata,
    ProviderResult, ProviderSafetyLevel,
};
pub use providers::{
    transport_registry_key, ActuatorProvider, AdapterMessage, CloudProvider, ConnectivityProvider,
    CryptoProvider, FleetProvider, HalProvider, LedgerProvider, MaintenanceProvider,
    NavigationProvider, PositioningProvider, ProviderRegistry, RosProvider, SensorProvider,
    SimulationProvider, SlamProvider, TransportConfig, TransportProvider, VisionProvider,
};
pub use robot_state::{PoseState, RobotState, VelocityState};
pub use robotics::{
    FleetRegistry, MissionRuntime, MissionState, ProgramSafetyZoneRegistry,
};
pub use scheduler::{
    advance_wall_tick, elapsed_ms, sleep_until, SchedulerClock,
};
pub use value::{
    format_runtime_value, get_number, get_pose_fields, get_string, get_trajectory_waypoints,
    get_velocity_fields, runtime_pose, runtime_trajectory, runtime_velocity, MotionCommand,
    PoseValue, RuntimeValue,
};
