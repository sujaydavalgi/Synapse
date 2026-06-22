//! Spanda runtime kernel primitives extracted for the Phase 4 lean-core split.
//!
pub mod classification;
pub mod environment;
pub mod error;
pub mod events;
pub mod hal_config;
pub mod host;
pub mod provider_types;
pub mod providers;
pub mod reliability_runtime;
pub mod replay;
pub mod robot_state;
pub mod robotics;
pub mod scheduler;
pub mod serialize;
pub mod state_machine;
pub mod telemetry;
pub mod triggers;
pub mod twin;
pub mod value;
pub mod world_model;

pub use classification::{
    module_classifications, official_package_names, ModuleClassification, ModuleOwnership,
};
pub use environment::Environment;
pub use error::RuntimeError;
pub use events::EventBus;
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
pub use reliability_runtime::{
    recover_handlers_from_decls, ModeRuntime, PipelineRuntime, RecoverHandlers, RetryRuntime,
    WatchdogRuntime,
};
pub use replay::{
    parse_replay_offset, playback_frames, verify_traces, MissionTrace, PlaybackReport,
    ReplayStateSnapshot, ReplayStateTarget, TraceFrame, TraceVerification,
};
pub use robot_state::{PoseState, RobotState, VelocityState};
pub use robotics::{
    FleetRegistry, MissionRuntime, MissionState, ProgramSafetyZoneRegistry,
};
pub use scheduler::{
    advance_wall_tick, elapsed_ms, sleep_until, SchedulerClock,
};
pub use serialize::{deserialize_value, serialize_value};
pub use state_machine::StateMachineRuntime;
pub use telemetry::{
    ExecutionMetrics, PipelineMetrics, RuntimeTelemetry, SchedulerMetrics, TaskMetrics,
    TopicMetrics, TriggerMetrics, WatchdogMetrics,
};
pub use triggers::{
    priority_rank, trigger_display_name, ConditionTriggerState, RegisteredTrigger,
    SystemTriggerCategory, TriggerRegistry, TriggerTimerSchedule, MAX_TRIGGERS_PER_TICK,
};
pub use twin::TwinRuntime;
pub use world_model::WorldModelRuntime;
pub use value::{
    format_runtime_value, get_number, get_pose_fields, get_string, get_trajectory_waypoints,
    get_velocity_fields, runtime_pose, runtime_trajectory, runtime_velocity, MotionCommand,
    PoseValue, RuntimeValue,
};
