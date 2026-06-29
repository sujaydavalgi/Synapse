//! Spanda runtime kernel primitives extracted for the Phase 4 lean-core split.
//!
pub mod assurance_runtime;
pub mod readiness_runtime;
pub mod fleet_tamper_runtime;
pub mod fleet_telemetry_runtime;
pub mod classification;
pub mod fault_primitives;
pub mod fault_runtime;
pub mod fault_types;
pub mod health_primitives;
pub mod health_types;
pub mod provider_runtime;
pub mod security_primitives;
pub mod security_runtime;
pub mod security_types;
pub mod device_telemetry_sink;
pub mod telemetry_sink;
pub mod wire_crypto;
pub mod continuity_primitives;
pub mod continuity_types;
pub mod environment;
pub mod error;
pub mod events;
pub mod fusion;
pub mod hal_config;
pub mod hooks;
pub mod host;
pub mod operational_policy;
pub mod provider_types;
pub mod providers;
pub mod reliability_runtime;
pub mod recovery_primitives;
pub mod recovery_types;
pub mod replay;
pub mod robot_state;
pub mod robotics;
pub mod scheduler;
pub mod serialize;
pub mod state_machine;
pub mod telemetry;
pub mod tamper_policy;
pub mod triggers;
pub mod twin;
pub mod value;
pub mod world_model;

pub use assurance_runtime::{
    default_assurance_runtime, platform_assurance_runtime, set_platform_assurance_runtime,
    AssuranceRuntime, BuiltinAssuranceRuntime, SharedAssuranceRuntime,
};
pub use readiness_runtime::{
    readiness_runtime, set_readiness_runtime, NoopReadinessRuntime, ReadinessRuntime,
};
pub use fleet_tamper_runtime::{
    fleet_tamper_runtime, set_fleet_tamper_runtime, FleetTamperRuntime, NoopFleetTamperRuntime,
};
pub use fleet_telemetry_runtime::{
    fleet_telemetry_runtime, set_fleet_telemetry_runtime, FleetTelemetryRuntime,
    NoopFleetTelemetryRuntime,
};
pub use fault_primitives::{
    empty_fault_scan_report, faults_from_hardware_signals, record_fault_in_trace,
};
pub use fault_runtime::{
    default_fault_runtime, BuiltinFaultRuntime, FaultRuntime, SharedFaultRuntime,
};
pub use fault_types::{
    FaultEvidence, FaultScanOptions, FaultScanReport, FaultTimeline, RuntimeFault,
    RuntimeFaultKind, RuntimeHealth, RuntimeHealthStatus,
};
pub use health_primitives::{
    apply_fleet_health_checks, evaluate_health_checks, evaluate_runtime_health,
};
pub use health_types::{
    HealthCheckResult, HealthReport, HealthStatus, HealthTraceRow,
};
pub use provider_runtime::{
    default_provider_runtime, BuiltinProviderRuntime, ProviderDispatchContext, ProviderRuntime,
    SharedProviderRuntime,
};
pub use security_primitives::{
    boundary_for_transport_name, bus_security_from_fields, effective_bus_security,
    resolve_broker_url, validate_bus_security,
};
pub use security_runtime::{
    default_security_runtime, default_security_runtime_factory, BuiltinSecurityRuntime,
    SecurityRuntime, SecurityRuntimeFactory,
};
pub use security_types::{
    AuthenticationMode, BusTransportSecurity, CommTransportSetup, EncryptionMode,
    IntegrityMode, RobotIdentity, SecretHandle, SecretSource, SecureCommPolicy, SecurePolicy,
    TrustBoundaryKind, TrustLevel,
};
pub use telemetry_sink::{
    default_telemetry_sink, NoopTelemetrySink, SharedTelemetrySink, TelemetrySink,
};
pub use device_telemetry_sink::{
    device_telemetry_sink, set_device_telemetry_sink, DeviceTelemetrySink,
    NoopDeviceTelemetrySink,
};
pub use wire_crypto::WireCryptoSession;
pub use classification::{
    module_classifications, official_package_names, ModuleClassification, ModuleOwnership,
};
pub use continuity_primitives::{
    default_checkpoint_store_path, extract_continuity_policies, issue_to_continuity_trigger,
    load_checkpoint, load_checkpoint_store, parse_trigger, program_has_continuity_for_trigger,
    record_checkpoint, save_checkpoint_store,
};
pub use continuity_types::{
    ContinuationDecision, ContinuityCheckpointStore, ContinuityContext, ContinuityEvidence,
    ContinuityPolicySpec, ContinuityTrigger, MissionCheckpoint, MissionExecutionState,
    MissionStateSnapshot, MissionStateTransfer, SuccessionScope, TakeoverMode, TakeoverReport,
};
pub use recovery_primitives::{
    classify_failure, default_knowledge_store_path, extract_recovery_policies,
    issue_to_recovery_issue, load_recovery_knowledge_store, merge_recovery_knowledge,
    program_has_recovery_for_issue, record_recovery_outcome, save_recovery_knowledge_store,
};
pub use recovery_types::{
    FailureClassification, FleetRecoveryPlan, OperationalMode, PlannedRecoveryAction,
    RecoveryAssuranceMetrics, RecoveryAuditRecord, RecoveryContext, RecoveryEvidence,
    RecoveryKnowledgeBase, RecoveryKnowledgeEntry, RecoveryLevel, RecoveryPlan, RecoveryPolicySpec,
    RecoveryReadiness, RecoveryReport, RecoveryResult, RecoveryStatus, RecoveryStrategy,
    RecoveryTraceChain, SafeRecoveryAction, SelfCorrectionAction, ValidationGateResult,
};
pub use environment::Environment;
pub use error::RuntimeError;
pub use events::EventBus;
pub use fusion::{
    parse_fusion_input, preview_fusion_inputs, sensor_type_index, weight_for_sensor_type,
    weighted_confidence, FusionPreview,
};
pub use hal_config::HalMemberConfig;
pub use hooks::{NoopRuntimeHooks, RuntimeHooks, SharedRuntimeHooks};
pub use host::{imports_enable_navigation, imports_enable_slam, RuntimeHost};
pub use operational_policy::{
    build_runtime_policy_monitor, check_runtime_policy_motion, RuntimePolicyMonitor,
    RuntimePolicyViolation,
};
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
pub use robotics::{FleetRegistry, MissionRuntime, MissionState, ProgramSafetyZoneRegistry};
pub use scheduler::{advance_wall_tick, elapsed_ms, sleep_until, SchedulerClock};
pub use serialize::{deserialize_value, serialize_value};
pub use state_machine::StateMachineRuntime;
pub use tamper_policy::{
    actions_for_tamper_event, extract_tamper_policies, tamper_policy_coverage, TamperPolicySpec,
    TamperSeverity,
};
pub use telemetry::{
    ExecutionMetrics, PipelineMetrics, RuntimeTelemetry, SchedulerMetrics, TaskMetrics,
    TopicMetrics, TriggerMetrics, WatchdogMetrics,
};
pub use triggers::{
    priority_rank, trigger_display_name, ConditionTriggerState, RegisteredTrigger,
    SystemTriggerCategory, TriggerRegistry, TriggerTimerSchedule, MAX_TRIGGERS_PER_TICK,
};
pub use twin::TwinRuntime;
pub use value::{
    format_runtime_value, get_number, get_pose_fields, get_string, get_trajectory_waypoints,
    get_velocity_fields, runtime_pose, runtime_trajectory, runtime_velocity, MotionCommand,
    PoseValue, RuntimeValue,
};
pub use world_model::WorldModelRuntime;
