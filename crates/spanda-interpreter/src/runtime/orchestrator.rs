//! runtime support for Spanda.
//!
use crate::platform_events::{emit_mission_completed, emit_mission_started};
use spanda_ai::{AgentRuntime, AiModel};
use spanda_ast::comm_decl::{QosDecl, TransportKind};
use spanda_ast::foundations::{CapabilityDecl, TaskPriority};
use spanda_ast::nodes::{Expr, Program, RobotDecl, Stmt};
use spanda_audit::{AuditRuntime, MockLedgerBackend};
use spanda_comm::CommBus;
use spanda_concurrency::ConcurrencyRuntime;
use spanda_connectivity_runtime::ConnectivityPolicyRuntime;
use spanda_debug::DebugController;
use spanda_error::SpandaError;
use spanda_ffi::FfiRegistry;
use spanda_hal::hal::{create_sim_hal, HalBackend, SimHalBackend};
use spanda_hal::HardwareMonitor;
use spanda_providers::{bootstrap_providers_for_packages, sync_comm_bus_for_official_packages};
use spanda_runtime::events::EventBus;
use spanda_runtime::reliability_runtime::{
    ModeRuntime, PipelineRuntime, RecoverHandlers, RetryRuntime, WatchdogRuntime,
};
use spanda_runtime::replay::MissionTrace;
use spanda_runtime::robot_state::{PoseState, RobotState, VelocityState};
use spanda_runtime::scheduler::SchedulerClock;
use spanda_runtime::state_machine::StateMachineRuntime;
use spanda_runtime::triggers::{
    ConditionTriggerState, TriggerRegistry, TriggerTimerSchedule, MAX_TRIGGERS_PER_TICK,
};
use spanda_runtime::twin::TwinRuntime;
use spanda_runtime::world_model::WorldModelRuntime;
use spanda_runtime_host::core_runtime_host;
use spanda_safety::{Pose2d, SafetyMonitor, SafetyZoneRuntime};
use spanda_security::SecurityContext;
use spanda_runtime::tamper_policy::{TamperPolicySpec, TamperSeverity};
use spanda_transport_routing::RoutingCommBus;
use spanda_typecheck::ModuleRegistry;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

type AgentTraitImplBody = (Vec<spanda_ast::foundations::TraitParamDecl>, Vec<Stmt>);
pub use spanda_runtime::environment::Environment;
pub use spanda_runtime::value::*;
pub use spanda_runtime::RuntimeError;
use spanda_runtime::RuntimeHost;

/// Convert extracted runtime errors into [`SpandaError`].
pub trait IntoSpandaError {
    fn into_spanda(self) -> SpandaError;
}

impl IntoSpandaError for RuntimeError {
    fn into_spanda(self) -> SpandaError {
        // Description:
        //     Into spanda.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     result: SpandaError
        //         Return value from `into_spanda`.
        //
        // Example:
        //     let result = instance.into_spanda();

        SpandaError::from(self)
    }
}

pub fn pose_from_state(state: &PoseState) -> RuntimeValue {
    // Description:

    //     Pose from state.

    //

    // Inputs:

    //     state: &PoseState

    //         Caller-supplied state.

    //

    // Outputs:

    //     result: RuntimeValue

    //         Return value from `pose_from_state`.

    //

    // Example:

    //     let result = spanda_interpreter::orchestrator::pose_from_state(state);

    runtime_pose(state.x, state.y, state.theta, state.z.unwrap_or(0.0))
}

pub fn velocity_from_state(state: &VelocityState) -> RuntimeValue {
    // Description:

    //     Velocity from state.

    //

    // Inputs:

    //     state: &VelocityState

    //         Caller-supplied state.

    //

    // Outputs:

    //     result: RuntimeValue

    //         Return value from `velocity_from_state`.

    //

    // Example:

    //     let result = spanda_interpreter::orchestrator::velocity_from_state(state);

    runtime_velocity(state.linear, state.angular)
}

pub trait RobotBackend {
    fn read_sensor(
        &mut self,
        sensor_name: &str,
        sensor_type: &str,
        topic: Option<&str>,
    ) -> RuntimeValue;
    fn execute_motion(&mut self, cmd: MotionCommand);
    fn tick(&mut self, dt_ms: f64);
    fn get_state(&self) -> RobotState;
    fn set_emergency_stop(&mut self, _active: bool) {
        // Description:
        //     Set emergency stop.
        //
        // Inputs:
        //     &mut self: value
        //         Caller-supplied &mut self.
        //     _active: bool
        //         Caller-supplied active.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_interpreter::orchestrator::set_emergency_stop(&mut self, _active);
    }
    fn publish_topic(&mut self, _topic_path: &str, _message_type: &str, _value: RuntimeValue) {
        // Description:
        //     Publish topic.
        //
        // Inputs:
        //     &mut self: value
        //         Caller-supplied &mut self.
        //     _topic_path: &str
        //         Caller-supplied topic path.
        //     _message_type: &str
        //         Caller-supplied message type.
        //     _value: RuntimeValue
        //         Caller-supplied value.
        //
        // Outputs:
        //     None.
        //
        // Example:
        //     let result = spanda_interpreter::orchestrator::publish_topic(&mut self, _topic_path, _message_type, _value);
    }
    fn call_service(&mut self, _service_name: &str, _service_type: &str) -> RuntimeValue {
        // Description:
        //     Call service.
        //
        // Inputs:
        //     &mut self: value
        //         Caller-supplied &mut self.
        //     _service_name: &str
        //         Caller-supplied service name.
        //     _service_type: &str
        //         Caller-supplied service type.
        //
        // Outputs:
        //     result: RuntimeValue
        //         Return value from `call_service`.
        //
        // Example:
        //     let result = spanda_interpreter::orchestrator::call_service(&mut self, _service_name, _service_type);
        RuntimeValue::Bool { value: true }
    }
    fn send_action(
        &mut self,
        _action_name: &str,
        _action_type: &str,
        _goal: RuntimeValue,
    ) -> RuntimeValue {
        // Description:
        //     Send action.
        //
        // Inputs:
        //     &mut self: value
        //         Caller-supplied &mut self.
        //     _action_name: &str
        //         Caller-supplied action name.
        //     _action_type: &str
        //         Caller-supplied action type.
        //     _goal: RuntimeValue
        //         Caller-supplied goal.
        //
        // Outputs:
        //     result: RuntimeValue
        //         Return value from `send_action`.
        //
        // Example:
        //     let result = spanda_interpreter::orchestrator::send_action(&mut self, _action_name, _action_type, _goal);
        RuntimeValue::Bool { value: true }
    }
    fn get_hal(&mut self) -> Option<&mut dyn HalBackend> {
        // Description:
        //     Get hal.
        //
        // Inputs:
        //     &mut self: value
        //         Caller-supplied &mut self.
        //
        // Outputs:
        //     result: Option<&mut dyn HalBackend>
        //         Return value from `get_hal`.
        //
        // Example:
        //     let result = spanda_interpreter::orchestrator::get_hal(&mut self);
        None
    }
    fn event_log(&self) -> Vec<String> {
        // Description:
        //     Event log.
        //
        // Inputs:
        //     &self: value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: Vec<String>
        //         Return value from `event_log`.
        //
        // Example:
        //     let result = spanda_interpreter::orchestrator::event_log(&self);
        Vec::new()
    }
}

type LogCallback = Rc<dyn Fn(String)>;
type MotionBlockedCallback = Rc<dyn Fn(String)>;

pub struct InterpreterOptions {
    pub max_loop_iterations: usize,
    pub on_motion_blocked: Option<MotionBlockedCallback>,
    pub on_log: Option<LogCallback>,
    pub module_registry: Option<ModuleRegistry>,
    pub debug: Option<DebugController>,
    pub ffi_registry: FfiRegistry,
    pub trace_scheduler: bool,
    pub trace_tasks: bool,
    pub trace_triggers: bool,
    pub trace_events: bool,
    pub trace_providers: bool,
    pub replay_trace: bool,
    pub record_trace: bool,
    pub trace_source: Option<String>,
    pub scheduler_clock: SchedulerClock,
    pub replay_deterministic: bool,

    /// Max trigger dispatches per scheduler tick (hardware-aware storm protection).
    pub max_triggers_per_tick: usize,

    /// Enforce strict secure communication at runtime.
    pub secure_mode: bool,

    /// Inject default security fault scenarios during simulation.
    pub inject_security_faults: bool,

    /// Activate named kill switch at simulation start.
    pub trigger_kill_switch: Option<String>,

    /// JSON-encoded SignedMessage for remote_signed kill switch activation.
    pub kill_switch_signature: Option<String>,

    /// Inject health fault scenarios during simulation.
    pub inject_health_faults: bool,

    /// Inbound comm payloads queued before each recovery approval poll (test/sim hook).
    pub inbound_comm_messages: Vec<(String, String)>,

    /// Optional domain provider registry; defaults to bootstrap shims when unset.
    pub provider_registry: Option<spanda_runtime::providers::ProviderRegistry>,

    /// Official package dependency names from the enclosing project manifest/lockfile.
    pub official_packages: Vec<String>,

    /// Optional domain host for adapter and connectivity hooks; defaults to core host.
    pub runtime_host: Option<&'static dyn RuntimeHost>,

    /// Enforce a named operational policy during simulation and live runs.
    pub enforce_policy: Option<String>,
}

impl Default for InterpreterOptions {
    fn default() -> Self {
        // Description:
        //     Provide the default value for this type.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     result: Self
        //         Return value from `default`.
        //
        // Example:
        //     let result = spanda_interpreter::orchestrator::default();
        Self {
            max_loop_iterations: 10,
            on_motion_blocked: None,
            on_log: None,
            module_registry: None,
            debug: None,
            ffi_registry: FfiRegistry::new(),
            trace_scheduler: false,
            trace_tasks: false,
            trace_triggers: false,
            trace_events: false,
            trace_providers: false,
            replay_trace: false,
            record_trace: false,
            trace_source: None,
            scheduler_clock: SchedulerClock::Sim,
            replay_deterministic: false,
            max_triggers_per_tick: MAX_TRIGGERS_PER_TICK,
            secure_mode: false,
            inject_security_faults: false,
            trigger_kill_switch: None,
            kill_switch_signature: None,
            inject_health_faults: false,
            inbound_comm_messages: Vec::new(),
            provider_registry: None,
            official_packages: Vec::new(),
            runtime_host: None,
            enforce_policy: None,
        }
    }
}

pub struct Interpreter<B: RobotBackend> {
    backend: B,
    options: InterpreterOptions,
    env: Environment,
    safety_monitor: Option<SafetyMonitor>,
    zones: Vec<SafetyZoneRuntime>,
    hal: SimHalBackend,
    ai_models: HashMap<String, AiModel>,
    agents: HashMap<String, AgentRuntime>,
    agent_capabilities: HashMap<String, Vec<CapabilityDecl>>,
    agent_capability_enforced: HashMap<String, bool>,
    current_agent: Option<String>,
    stop_if_conditions: Vec<Expr>,
    event_bus: EventBus,
    trigger_registry: TriggerRegistry,
    trigger_timers: Vec<TriggerTimerSchedule>,
    condition_trigger_state: ConditionTriggerState,
    declared_event_names: std::collections::HashSet<String>,
    declared_topic_names: std::collections::HashSet<String>,
    triggers_dispatched_this_tick: usize,
    twin: Option<TwinRuntime>,
    state_machines: HashMap<String, StateMachineRuntime>,
    enum_variants: HashMap<String, Vec<String>>,
    variant_owner: HashMap<String, String>,
    struct_defs: HashMap<String, Vec<(String, String)>>,
    agent_trait_impls: HashMap<String, HashMap<String, AgentTraitImplBody>>,
    verify_rules: Vec<Expr>,
    verify_warning_rules: Vec<Expr>,
    fusion_sensors: Vec<String>,
    world_model_fusion_hook: bool,
    hardware_monitor: HardwareMonitor,
    topic_path_to_name: HashMap<String, String>,
    topic_path_to_message_type: HashMap<String, String>,
    ai_confidence_low_active: bool,
    twin_faults_dispatched: std::collections::HashSet<String>,
    audit_runtime: Option<AuditRuntime>,
    mock_ledger: MockLedgerBackend,
    world_model: WorldModelRuntime,
    security: SecurityContext,
    comm_bus: RoutingCommBus,
    default_transport: TransportKind,
    module_functions: HashMap<String, spanda_ast::foundations::ModuleFnDecl>,
    imported_functions: HashMap<String, spanda_ast::foundations::ModuleFnDecl>,
    imported_function_modules: HashMap<String, String>,
    extern_functions: HashMap<String, spanda_ast::foundations::ExternFnDecl>,
    concurrency: ConcurrencyRuntime,
    telemetry: spanda_runtime::telemetry::RuntimeTelemetry,
    active_mode: String,
    active_robot_name: Option<String>,
    task_heartbeats: HashMap<String, f64>,
    sim_time_ms: f64,
    watchdogs: Vec<WatchdogRuntime>,
    pipelines: HashMap<String, PipelineRuntime>,
    retries: Vec<RetryRuntime>,
    recovers: RecoverHandlers,
    modes: HashMap<String, ModeRuntime>,
    topic_qos: HashMap<String, QosDecl>,
    topic_last_publish_ms: HashMap<String, f64>,
    topic_deadline_misses: HashMap<String, u64>,
    mission_trace: Option<MissionTrace>,
    geofences: Vec<spanda_connectivity::GeofenceRuntime>,
    geofence_active: std::collections::HashSet<String>,
    connectivity_policies: Vec<ConnectivityPolicyRuntime>,
    active_connectivity_link: String,
    connectivity_events_seen: std::collections::HashSet<String>,
    gps_available: bool,
    fleets: spanda_runtime::robotics::FleetRegistry,
    program_safety_zones: spanda_runtime::robotics::ProgramSafetyZoneRegistry,
    nav2_enabled: bool,
    slam_enabled: bool,
    provider_registry: Rc<RefCell<spanda_runtime::providers::ProviderRegistry>>,
    host: &'static dyn RuntimeHost,
    health_program: Option<Program>,
    last_health_overall: Option<String>,
    last_health_checks: HashMap<String, String>,
    fault_program: Option<Program>,
    seen_fault_keys: std::collections::HashSet<String>,
    applied_health_reactions: std::collections::HashSet<String>,
    applied_anomaly_handlers: std::collections::HashSet<String>,
    tamper_policies: Vec<TamperPolicySpec>,
    applied_tamper_branches: std::collections::HashSet<String>,
    learned_anomaly_triggers: std::collections::HashSet<String>,
    learned_anomaly_ema: std::collections::HashMap<String, f64>,
    kill_switch_defs: HashMap<String, spanda_ast::foundations::KillSwitchDecl>,
    program_swarms: Vec<spanda_ast::robotics_decl::SwarmDecl>,
    pending_recovery_approvals: std::collections::HashSet<String>,
    granted_recovery_approvals: std::collections::HashSet<String>,
    deferred_recovery_issues: Vec<String>,
    mission_approval_actions: std::collections::HashSet<String>,
    recovery_knowledge_path: std::path::PathBuf,
    recovery_speed_cap: Option<f64>,
    runtime_policy: Option<spanda_runtime::operational_policy::RuntimePolicyMonitor>,
}

impl<B: RobotBackend> Interpreter<B> {
    pub fn new(backend: B, mut options: InterpreterOptions) -> Self {
        // Description:
        //     Construct a new instance.
        //
        // Inputs:
        //     backend: B
        //         Caller-supplied backend.
        //     options: InterpreterOptions
        //         Caller-supplied options.
        //
        // Outputs:
        //     result: Self
        //         Return value from `new`.
        //
        // Example:
        //     let value = spanda_interpreter::orchestrator::new(backend, options);
        let provider_registry = Rc::new(RefCell::new(
            options.provider_registry.take().unwrap_or_else(|| {
                bootstrap_providers_for_packages(
                    &options
                        .official_packages
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>(),
                )
            }),
        ));
        let host = options.runtime_host.unwrap_or_else(|| core_runtime_host());
        let mut comm_bus = RoutingCommBus::new();
        comm_bus.attach_provider_registry(Rc::clone(&provider_registry));
        Self {
            backend,
            options,
            provider_registry,
            env: Environment::new(),
            safety_monitor: None,
            zones: Vec::new(),
            hal: create_sim_hal(),
            ai_models: HashMap::new(),
            agents: HashMap::new(),
            agent_capabilities: HashMap::new(),
            agent_capability_enforced: HashMap::new(),
            current_agent: None,
            stop_if_conditions: Vec::new(),
            event_bus: EventBus::new(),
            trigger_registry: TriggerRegistry::new(),
            trigger_timers: Vec::new(),
            condition_trigger_state: ConditionTriggerState::default(),
            declared_event_names: std::collections::HashSet::new(),
            declared_topic_names: std::collections::HashSet::new(),
            triggers_dispatched_this_tick: 0,
            twin: None,
            state_machines: HashMap::new(),
            enum_variants: HashMap::new(),
            variant_owner: HashMap::new(),
            struct_defs: HashMap::new(),
            agent_trait_impls: HashMap::new(),
            verify_rules: Vec::new(),
            verify_warning_rules: Vec::new(),
            fusion_sensors: Vec::new(),
            world_model_fusion_hook: false,
            hardware_monitor: HardwareMonitor::default(),
            topic_path_to_name: HashMap::new(),
            topic_path_to_message_type: HashMap::new(),
            ai_confidence_low_active: false,
            twin_faults_dispatched: std::collections::HashSet::new(),
            audit_runtime: None,
            mock_ledger: MockLedgerBackend::new(),
            world_model: WorldModelRuntime::new(),
            security: SecurityContext::new(),
            comm_bus,
            default_transport: TransportKind::Sim,
            module_functions: HashMap::new(),
            imported_functions: HashMap::new(),
            imported_function_modules: HashMap::new(),
            extern_functions: HashMap::new(),
            concurrency: ConcurrencyRuntime::new(),
            telemetry: spanda_runtime::telemetry::RuntimeTelemetry::default(),
            active_mode: "normal".into(),
            active_robot_name: None,
            task_heartbeats: HashMap::new(),
            sim_time_ms: 0.0,
            watchdogs: Vec::new(),
            pipelines: HashMap::new(),
            retries: Vec::new(),
            recovers: HashMap::new(),
            modes: HashMap::new(),
            topic_qos: HashMap::new(),
            topic_last_publish_ms: HashMap::new(),
            topic_deadline_misses: HashMap::new(),
            mission_trace: None,
            geofences: Vec::new(),
            geofence_active: std::collections::HashSet::new(),
            connectivity_policies: Vec::new(),
            active_connectivity_link: "wifi".into(),
            connectivity_events_seen: std::collections::HashSet::new(),
            gps_available: true,
            fleets: spanda_runtime::robotics::FleetRegistry::default(),
            program_safety_zones: spanda_runtime::robotics::ProgramSafetyZoneRegistry::default(),
            nav2_enabled: false,
            slam_enabled: false,
            host,
            health_program: None,
            last_health_overall: None,
            last_health_checks: HashMap::new(),
            fault_program: None,
            seen_fault_keys: std::collections::HashSet::new(),
            applied_health_reactions: std::collections::HashSet::new(),
            applied_anomaly_handlers: std::collections::HashSet::new(),
            tamper_policies: Vec::new(),
            applied_tamper_branches: std::collections::HashSet::new(),
            learned_anomaly_triggers: std::collections::HashSet::new(),
            learned_anomaly_ema: std::collections::HashMap::new(),
            kill_switch_defs: HashMap::new(),
            program_swarms: Vec::new(),
            pending_recovery_approvals: std::collections::HashSet::new(),
            granted_recovery_approvals: std::collections::HashSet::new(),
            deferred_recovery_issues: Vec::new(),
            mission_approval_actions: std::collections::HashSet::new(),
            recovery_knowledge_path: spanda_assurance::default_knowledge_store_path(),
            recovery_speed_cap: None,
            runtime_policy: None,
        }
    }

    pub fn runtime_host(&self) -> &dyn RuntimeHost {
        // Description:

        //     Runtime host.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //

        // Outputs:

        //     result: &dyn RuntimeHost

        //         Return value from `runtime_host`.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::runtime_host(&self);

        self.host
    }

    pub fn telemetry(&self) -> &spanda_runtime::telemetry::RuntimeTelemetry {
        // Description:

        //     Telemetry.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //

        // Outputs:

        //     result: &spanda_runtime::telemetry::RuntimeTelemetry

        //         Return value from `telemetry`.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::telemetry(&self);
        &self.telemetry
    }

    pub fn provider_registry(
        &self,
    ) -> std::cell::Ref<'_, spanda_runtime::providers::ProviderRegistry> {
        // Description:

        //     Provider registry.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //

        // Outputs:

        //     result: std::cell::Ref<'_, spanda_runtime::providers::ProviderRegistry>

        //         Return value from `provider_registry`.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::provider_registry(&self);

        self.provider_registry.borrow()
    }

    pub fn take_telemetry(&mut self) -> spanda_runtime::telemetry::RuntimeTelemetry {
        // Description:

        //     Take telemetry.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //

        // Outputs:

        //     result: spanda_runtime::telemetry::RuntimeTelemetry

        //         Return value from `take_telemetry`.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::take_telemetry(&mut self);
        std::mem::take(&mut self.telemetry)
    }

    pub fn sim_time_ms(&self) -> f64 {
        self.sim_time_ms
    }

    pub fn twin_replay_export(&self) -> Option<serde_json::Value> {
        // Description:

        //     Twin replay export.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //

        // Outputs:

        //     result: Option<serde_json::Value>

        //         Return value from `twin_replay_export`.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::twin_replay_export(&self);

        self.twin
            .as_ref()
            .filter(|twin| twin.replay_frame_count() > 0)
            .map(|twin| twin.export_replay_json())
    }

    pub fn take_mission_trace(&mut self) -> Option<MissionTrace> {
        // Description:

        //     Take mission trace.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //

        // Outputs:

        //     result: Option<MissionTrace>

        //         Return value from `take_mission_trace`.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::take_mission_trace(&mut self);
        self.mission_trace.take()
    }

    fn trace_scheduler_log(&self, message: impl Into<String>) {
        // Description:

        //     Trace scheduler log.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //     essage: impl Into<String>

        //         Caller-supplied essage.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::trace_scheduler_log(&self, essage);
        if self.options.trace_scheduler {
            self.log(format!("trace-scheduler: {}", message.into()));
        }
    }

    fn trace_task_log(&self, message: impl Into<String>) {
        // Description:

        //     Trace task log.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //     essage: impl Into<String>

        //         Caller-supplied essage.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::trace_task_log(&self, essage);
        if self.options.trace_tasks {
            self.log(format!("trace-task: {}", message.into()));
        }
    }

    fn trace_trigger_log(&self, message: impl Into<String>) {
        // Description:

        //     Trace trigger log.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //     essage: impl Into<String>

        //         Caller-supplied essage.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::trace_trigger_log(&self, essage);
        if self.options.trace_triggers {
            self.log(format!("trace-trigger: {}", message.into()));
        }
    }

    fn trace_event_log(&self, message: impl Into<String>) {
        // Description:

        //     Trace event log.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //     essage: impl Into<String>

        //         Caller-supplied essage.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::trace_event_log(&self, essage);
        if self.options.trace_events || self.options.trace_triggers {
            self.log(format!("trace-event: {}", message.into()));
        }
    }

    fn trace_replay_log(&self, message: impl Into<String>) {
        // Description:

        //     Trace replay log.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //     essage: impl Into<String>

        //         Caller-supplied essage.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::trace_replay_log(&self, essage);
        if self.options.replay_trace {
            self.log(format!("trace-replay: {}", message.into()));
        }
    }

    pub fn robot_backend(&self) -> &B {
        // Description:

        //     Robot backend.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //

        // Outputs:

        //     result: &B

        //         Return value from `robot_backend`.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::robot_backend(&self);
        &self.backend
    }

    pub fn env(&self) -> &Environment {
        // Description:

        //     Env.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //

        // Outputs:

        //     result: &Environment

        //         Return value from `env`.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::env(&self);
        &self.env
    }

    pub fn env_mut(&mut self) -> &mut Environment {
        // Description:

        //     Env mut.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //

        // Outputs:

        //     result: &mut Environment

        //         Return value from `env_mut`.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::env_mut(&mut self);
        &mut self.env
    }

    pub fn setup_robot_for_debug(&mut self, robot: &RobotDecl) -> Result<(), SpandaError> {
        // Description:

        //     Setup robot for debug.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     robo: &RobotDecl

        //         Caller-supplied robo.

        //

        // Outputs:

        //     result: Result<(), SpandaError>

        //         Return value from `setup_robot_for_debug`.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::setup_robot_for_debug(&mut self, robo);
        self.setup_robot(robot)
    }

    pub fn debug_execute_stmt(&mut self, stmt: &Stmt) -> Result<(), SpandaError> {
        // Description:

        //     Debug execute stmt.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     s: &Stmt

        //         Caller-supplied s.

        //

        // Outputs:

        //     result: Result<(), SpandaError>

        //         Return value from `debug_execute_stmt`.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::debug_execute_stmt(&mut self, s);
        self.execute_stmt(stmt)
    }

    pub fn resolve_sync_call(
        &self,
        stmt: &Stmt,
    ) -> Option<(
        String,
        spanda_ast::foundations::ModuleFnDecl,
        Vec<spanda_ast::nodes::Expr>,
    )> {
        // Description:

        //     Resolve sync call.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //     s: &Stmt

        //         Caller-supplied s.

        //

        // Outputs:

        //     result: Option<( String, spanda_ast::foundations::ModuleFnDecl, Vec<spanda_ast::nodes::Expr>, )>

        //         Return value from `resolve_sync_call`.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::resolve_sync_call(&self, s);
        use spanda_ast::nodes::{Expr, Stmt};
        let expr = match stmt {
            Stmt::VarDecl {
                init: Some(init), ..
            } => init,
            Stmt::ExprStmt { expr, .. } => expr,
            Stmt::ReturnStmt {
                value: Some(value), ..
            } => value,
            _ => return None,
        };
        let Expr::CallExpr { callee, args, .. } = expr else {
            return None;
        };
        let Expr::IdentExpr { name, .. } = callee.as_ref() else {
            return None;
        };
        let func = self
            .module_functions
            .get(name)
            .or_else(|| self.imported_functions.get(name))?
            .clone();

        // Skip synchronous handling for async functions.
        if func.is_async {
            return None;
        }
        Some((name.clone(), func, args.clone()))
    }

    pub fn bind_call_args(
        &mut self,
        func: &spanda_ast::foundations::ModuleFnDecl,
        args: &[spanda_ast::nodes::Expr],
    ) -> Result<Environment, SpandaError> {
        // Description:

        //     Bind call args.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     func: &spanda_ast::foundations::ModuleFnDecl

        //         Caller-supplied func.

        //     args: &[spanda_ast::nodes::Expr]

        //         Caller-supplied args.

        //

        // Outputs:

        //     result: Result<Environment, SpandaError>

        //         Return value from `bind_call_args`.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::bind_call_args(&mut self, func, args);
        let saved = self.env.clone_bindings();

        // Bind each formal parameter to its call argument.
        for (i, param) in func.params.iter().enumerate() {
            // Emit output when get provides a arg.
            if let Some(arg) = args.get(i) {
                let val = self.eval_expr(arg)?;
                self.env.define(param.name.clone(), val);
            }
        }
        Ok(saved)
    }

    pub fn restore_env(&mut self, env: Environment) {
        // Description:

        //     Restore env.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     env: Environment

        //         Caller-supplied env.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::restore_env(&mut self, env);
        self.env = env;
    }

    pub fn run(
        &mut self,
        program: &Program,
        entry_behavior: Option<&str>,
    ) -> Result<RobotState, SpandaError> {
        // Description:

        //     Run.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     progra: &Program

        //         Caller-supplied progra.

        //     entry_behavior: Option<&str>

        //         Caller-supplied entry behavior.

        //

        // Outputs:

        //     result: Result<RobotState, SpandaError>

        //         Return value from `run`.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::run(&mut self, progra, entry_behavior);
        let Program::Program {
            robots,
            geofences,
            fleets,
            program_safety_zones,
            certifications,
            connectivity_policies,
            simulate_compatibility,
            ..
        } = program;
        let mut sim_faults: Vec<String> = Vec::new();

        // Emit output when simulate compatibility provides a sim.
        if let Some(sim) = simulate_compatibility {
            use spanda_ast::foundations::SimulateCompatibilityDecl;
            let SimulateCompatibilityDecl::SimulateCompatibilityDecl { faults, .. } = sim;
            sim_faults = faults.iter().map(|f| f.fault_type.clone()).collect();
        }
        self.load_program_metadata(program);
        self.cache_health_program(program);
        self.cache_tamper_policies(program);
        self.cache_fault_program(program);
        self.cache_kill_switches(program);
        if let Some(ref policy_name) = self.options.enforce_policy.clone() {
            let monitor = spanda_runtime::operational_policy::build_runtime_policy_monitor(
                program,
                policy_name,
            )
                .map_err(|message| RuntimeError::new(message, 0).into_spanda())?;
            self.load_runtime_policy(monitor)?;
        }
        let Program::Program { swarms, .. } = program;
        self.program_swarms = swarms.clone();
        if !self
            .provider_registry
            .borrow()
            .official_packages()
            .is_empty()
        {
            sync_comm_bus_for_official_packages(
                &mut self.comm_bus,
                &mut self.provider_registry.borrow_mut(),
            );
            self.log(format!(
                "providers: {} official package(s) active",
                self.provider_registry.borrow().official_packages().len()
            ));
        }
        self.load_connectivity_metadata(geofences, connectivity_policies);
        self.load_robotics_platform_metadata(fleets, program_safety_zones, certifications);

        if self.options.secure_mode {
            self.security.enable_strict_permissions();
            self.log("security: strict secure mode enabled".into());
        }

        if self.options.inject_security_faults {
            for fault in ["InvalidSignature", "ExpiredCertificate", "ReplayAttack"] {
                self.comm_bus.inject_fault(fault);
                self.hardware_monitor.inject_fault(fault.to_string());
                self.security.inject_security_fault(fault);
                self.invoke_tamper_policies(fault, TamperSeverity::Critical);
            }
            self.log("security: injected default security fault scenarios".into());
        }

        let multi_robot = robots.len() > 1;

        if multi_robot {
            // Each robot gets a fresh runtime env; setup then execute before the next robot.
            for (index, robot) in robots.iter().enumerate() {
                self.setup_robot(robot)?;
                if index == 0 {
                    emit_mission_started(
                        self.audit_runtime.as_mut(),
                        program,
                        self.options.trace_source.as_deref(),
                    );
                    if self.options.inject_health_faults {
                        for fault in ["GPSDegraded", "CameraOffline", "RobotHealthCritical"] {
                            self.hardware_monitor.inject_fault(fault.to_string());
                            self.comm_bus.inject_fault(fault);
                        }
                        self.log("health: injected default health fault scenarios".into());
                        self.poll_runtime_health_changes();
                        self.poll_runtime_fault_changes();
                    }
                    if let Some(ks) = self.options.trigger_kill_switch.clone() {
                        self.activate_kill_switch(&ks)?;
                    }
                }
                self.execute_robot_entry(robot, entry_behavior, &sim_faults)?;
            }
        } else {
            // Register robot hardware, identity, and triggers before optional kill-switch activation.
            for robot in robots {
                self.setup_robot(robot)?;
            }

            emit_mission_started(
                self.audit_runtime.as_mut(),
                program,
                self.options.trace_source.as_deref(),
            );

            if self.options.inject_health_faults {
                for fault in ["GPSDegraded", "CameraOffline", "RobotHealthCritical"] {
                    self.hardware_monitor.inject_fault(fault.to_string());
                    self.comm_bus.inject_fault(fault);
                }
                self.log("health: injected default health fault scenarios".into());
                self.poll_runtime_health_changes();
                self.poll_runtime_fault_changes();
            }

            if let Some(ks) = self.options.trigger_kill_switch.clone() {
                self.activate_kill_switch(&ks)?;
            }

            // Handle each robot declared in the program.
            for robot in robots {
                self.execute_robot_entry(robot, entry_behavior, &sim_faults)?;
            }
        }
        self.process_spawn_queue()?;

        // Emit output when twin provides a twin.
        if let Some(twin) = &self.twin {
            let frames = twin.replay_frame_count();
            self.telemetry.record_replay_frames(frames as u64);

            // Record replay output when trace replay mode is active.
            if self.options.replay_trace && frames > 0 {
                self.trace_replay_log(format!("captured {frames} replay frame(s)"));
            }
        }
        // Record mission completion when audit runtime is active.
        emit_mission_completed(
            self.audit_runtime.as_mut(),
            program,
            self.options.trace_source.as_deref(),
            true,
        );
        Ok(self.backend.get_state())
    }

    pub(crate) fn audit_runtime_mut(&mut self) -> Option<&mut AuditRuntime> {
        self.audit_runtime.as_mut()
    }

    fn execute_robot_entry(
        &mut self,
        robot: &RobotDecl,
        entry_behavior: Option<&str>,
        sim_faults: &[String],
    ) -> Result<(), SpandaError> {
        // Description:

        //     Execute robot entry.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     robo: &RobotDecl

        //         Caller-supplied robo.

        //     entry_behavior: Option<&str>

        //         Caller-supplied entry behavior.

        //     sim_faults: &[String]

        //         Caller-supplied sim faults.

        //

        // Outputs:

        //     result: Result<(), SpandaError>

        //         Return value from `execute_robot_entry`.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::execute_robot_entry(&mut self, robo, entry_behavior, sim_faults);

        for fault in sim_faults {
            self.hardware_monitor.inject_fault(fault.clone());
            self.comm_bus.inject_fault(fault);
            if matches!(
                fault.as_str(),
                "InvalidSignature"
                    | "ExpiredCertificate"
                    | "ReplayAttack"
                    | "UnknownDevice"
                    | "ManInTheMiddle"
                    | "SecureHandshakeDropped"
            ) {
                self.security.inject_security_fault(fault);
            }
        }

        if !sim_faults.is_empty() {
            self.log(format!(
                "simulate_compatibility: {} fault(s) active",
                sim_faults.len()
            ));
            self.run_retry_policies()?;
        }
        let RobotDecl::RobotDecl {
            behaviors, tasks, ..
        } = robot;

        if behaviors.is_empty() && tasks.len() > 1 && entry_behavior.is_none() {
            self.execute_multiplexed_tasks(robot.all_task_schedules())?;
            return Ok(());
        }

        if behaviors.is_empty()
            && tasks.is_empty()
            && entry_behavior.is_none()
            && self.has_standalone_triggers()
        {
            self.execute_trigger_only_loop()?;
            return Ok(());
        }
        let behavior_name = entry_behavior
            .map(String::from)
            .or_else(|| robot.first_behavior_name());

        if let Some(name) = behavior_name {
            if let Some((body, requires, ensures, invariant)) = robot.behavior_with_contracts(&name)
            {
                self.execute_with_contracts(&body, &requires, &ensures, &invariant)?;
            } else if let Some((body, interval_ms, requires, ensures, invariant)) =
                robot.task_with_contracts(&name)
            {
                let schedule = robot
                    .all_task_schedules()
                    .into_iter()
                    .find(|schedule| schedule.name == name);
                let priority = schedule
                    .as_ref()
                    .map(|s| s.priority)
                    .unwrap_or(TaskPriority::Normal);
                let budget = schedule.as_ref().and_then(|s| s.budget.clone());
                self.execute_task_loop_with_contracts(
                    &name,
                    priority,
                    &body,
                    interval_ms,
                    &requires,
                    &ensures,
                    &invariant,
                    budget,
                )?;
            }
        }
        Ok(())
    }

    pub fn run_tests(&mut self, program: &Program) -> Result<(), SpandaError> {
        // Description:

        //     Run tests.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     progra: &Program

        //         Caller-supplied progra.

        //

        // Outputs:

        //     result: Result<(), SpandaError>

        //         Return value from `run_tests`.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::run_tests(&mut self, progra);
        let Program::Program { tests, .. } = program;
        self.load_program_metadata(program);

        // Run each test block in program order.
        for test in tests {
            self.log(format!("test {}", test.name));

            // Validate compile-fail expectations before executing runtime assertions.
            for stmt in &test.body {
                if let Stmt::ExpectCompileErrorStmt { body, span } = stmt {
                    self.verify_expect_compile_error(body, span.start.line)?;
                }
            }

            self.execute_test_block(&test.body)?;
            self.process_spawn_queue()?;
        }
        Ok(())
    }

    fn execute_test_block(&mut self, body: &[Stmt]) -> Result<(), SpandaError> {
        // Description:

        //     Execute test block.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     body: &[Stmt]

        //         Caller-supplied body.

        //

        // Outputs:

        //     result: Result<(), SpandaError>

        //         Return value from `execute_test_block`.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::execute_test_block(&mut self, body);

        for stmt in body {
            if matches!(stmt, Stmt::ExpectCompileErrorStmt { .. }) {
                continue;
            }
            self.execute_stmt(stmt)?;
        }
        Ok(())
    }

    fn verify_expect_compile_error(&self, body: &[Stmt], line: u32) -> Result<(), SpandaError> {
        // Description:

        //     Verify expect compile error.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //     body: &[Stmt]

        //         Caller-supplied body.

        //     line: u32

        //         Caller-supplied line.

        //

        // Outputs:

        //     result: Result<(), SpandaError>

        //         Return value from `verify_expect_compile_error`.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::verify_expect_compile_error(&self, body, line);

        use spanda_ast::foundations::{ModuleFnDecl, Visibility};
        use spanda_ast::nodes::{Program, SpandaType};
        use spanda_runtime_host::core_type_check_host;
        use spanda_typecheck::check_with_registry;

        let probe = Program::Program {
            module_name: None,
            imports: vec![],
            functions: vec![ModuleFnDecl {
                doc: None,
                name: "__compile_fail_probe".into(),
                visibility: Visibility::Private,
                type_params: vec![],
                params: vec![],
                return_type: SpandaType::Void,
                is_async: false,
                body: body.to_vec(),
                span: Default::default(),
            }],
            tests: vec![],
            extern_functions: vec![],
            structs: vec![],
            enums: vec![],
            traits: vec![],
            hardware_profiles: vec![],
            deployments: vec![],
            requires_hardware: None,
            requires_network: None,
            requires_connectivity: None,
            geofences: vec![],
            fleets: vec![],
            swarms: vec![],
            program_safety_zones: vec![],
            certifications: vec![],
            connectivity_policies: vec![],
            ble_services: vec![],
            simulate_compatibility: None,
            messages: vec![],
            validate_rules: vec![],
            kill_switches: vec![],
            health_checks: vec![],
            health_policies: vec![],
            requires_capabilities: vec![],
            knowledge_models: vec![],
            state_estimators: vec![],
            anomaly_detectors: vec![],
            anomaly_handlers: vec![],
            prognostics: vec![],
            mitigations: vec![],
            operating_modes: vec![],
            mission_plans: vec![],
            resilience_policies: vec![],
            recovery_policies: vec![],
            tamper_policies: vec![],
            continuity_policies: vec![],
            operational_policies: vec![],
            assurance_cases: vec![],
            runtime_fault_triggers: vec![],
            robots: vec![],
            span: Default::default(),
        };
        let registry = self.options.module_registry.clone().unwrap_or_default();
        if check_with_registry(&probe, &registry, core_type_check_host()).is_ok() {
            return Err(SpandaError::Runtime {
                message: "expect_compile_error block passed type check but should have failed"
                    .into(),
                line,
            });
        }
        self.log("expect_compile_error: compile failure confirmed".into());
        Ok(())
    }

    pub fn load_program_metadata(&mut self, program: &Program) {
        // Description:

        //     Load program metadata.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     progra: &Program

        //         Caller-supplied progra.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::load_program_metadata(&mut self, progra);
        use spanda_ast::foundations::{EnumDecl, ModuleFnDecl, StructDecl, TraitDecl, Visibility};
        let Program::Program {
            structs,
            enums,
            traits,
            functions,
            extern_functions,
            imports,
            ..
        } = program;
        self.module_functions.clear();
        self.imported_functions.clear();
        self.imported_function_modules.clear();
        self.extern_functions.clear();

        // Generate code for each module function.
        for func in functions {
            let ModuleFnDecl {
                name, visibility, ..
            } = func;

            // Keep entries that match the expected pattern.
            if matches!(visibility, Visibility::Export | Visibility::Public) {
                self.module_functions.insert(name.clone(), func.clone());
            }
        }

        // Declare each extern function in the generated output.
        for ext in extern_functions {
            self.extern_functions.insert(ext.name.clone(), ext.clone());
        }
        use spanda_ast::nodes::ImportDecl;

        // Emit output when module registry provides a registry.
        if let Some(registry) = &self.options.module_registry {
            // Process each import.
            for imp in imports {
                let ImportDecl::ImportDecl { path, .. } = imp;

                // Emit output when exports for provides a exports.
                if let Some(exports) = registry.exports_for(path) {
                    // Iterate over functions with destructured elements.
                    for (name, func) in &exports.functions {
                        self.imported_functions.insert(name.clone(), func.clone());
                        self.imported_function_modules
                            .insert(name.clone(), path.clone());
                    }
                }
            }
        }
        self.enum_variants.clear();
        self.variant_owner.clear();
        self.struct_defs.clear();

        // Process each enum.
        for enum_decl in enums {
            let EnumDecl::EnumDecl { name, variants, .. } = enum_decl;
            self.enum_variants.insert(
                name.clone(),
                variants.iter().map(|v| v.name.clone()).collect(),
            );

            // Handle each enum variant arm.
            for variant in variants {
                self.variant_owner
                    .insert(variant.name.clone(), name.clone());
            }
        }

        // Process each struct.
        for struct_decl in structs {
            let StructDecl::StructDecl { name, fields, .. } = struct_decl;
            self.struct_defs.insert(
                name.clone(),
                fields
                    .iter()
                    .map(|f| (f.name.clone(), f.type_name.clone()))
                    .collect(),
            );
        }
        let _ = traits;

        // Process each trait.
        for trait_decl in traits {
            let TraitDecl::TraitDecl { name, .. } = trait_decl;
            let _ = name;
        }
        self.nav2_enabled = {
            let paths: Vec<&str> = imports
                .iter()
                .map(|imp| {
                    let spanda_ast::nodes::ImportDecl::ImportDecl { path, .. } = imp;
                    path.as_str()
                })
                .collect();
            spanda_runtime::imports_enable_navigation(&paths, self.host)
        };
        self.slam_enabled = {
            let paths: Vec<&str> = imports
                .iter()
                .map(|imp| {
                    let spanda_ast::nodes::ImportDecl::ImportDecl { path, .. } = imp;
                    path.as_str()
                })
                .collect();
            spanda_runtime::imports_enable_slam(&paths, self.host)
        };
    }

    pub(super) fn check_safety_before_motion(&mut self) -> bool {
        // Description:

        //     Check safety before motion.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //

        // Outputs:

        //     result: bool

        //         Return value from `check_safety_before_motion`.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::check_safety_before_motion(&mut self);
        if let Some(reason) = self.check_runtime_policy_before_motion(0.0) {
            self.log(reason);
            return false;
        }
        let state = self.backend.get_state();

        // Take this path when self.evaluate stop if(&self.env.clone bindings()).
        if self.evaluate_stop_if(&self.env.clone_bindings()) {
            self.backend.set_emergency_stop(true);

            // Emit output when safety monitor provides a monitor.
            if let Some(monitor) = &mut self.safety_monitor {
                monitor.set_emergency_stop(true);
            }
            self.log("stop_if safety rule triggered".into());
            return false;
        }

        // Emit output when safety monitor provides a monitor.
        if let Some(monitor) = &mut self.safety_monitor {
            let pose2d = Pose2d {
                x: state.pose.x,
                y: state.pose.y,
            };
            let result = monitor.evaluate_before_motion(&self.env, &pose2d);

            // Take the branch when allowed is false.
            if !result.allowed {
                // Take this path when result.emergency stop.
                if result.emergency_stop {
                    self.backend.set_emergency_stop(true);
                }

                // Emit output when reason provides a reason.
                if let Some(reason) = result.reason {
                    self.log(reason);
                }
                return false;
            }
        }
        true
    }

    fn log(&self, message: String) {
        // Description:

        //     Log.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //     essage: String

        //         Caller-supplied essage.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_interpreter::orchestrator::log(&self, essage);
        if let Some(cb) = &self.options.on_log {
            cb(message);
        }
    }
}

fn pose_value_to_state(pose: &PoseValue) -> PoseState {
    // Description:

    //     Pose value to state.

    //

    // Inputs:

    //     pose: &PoseValue

    //         Caller-supplied pose.

    //

    // Outputs:

    //     result: PoseState

    //         Return value from `pose_value_to_state`.

    //

    // Example:

    //     let result = spanda_interpreter::orchestrator::pose_value_to_state(pose);
    PoseState {
        x: pose.x,
        y: pose.y,
        theta: pose.theta,
        z: Some(pose.z),
    }
}

#[path = "runtime_decl_extensions.rs"]
mod runtime_decl_extensions;
use runtime_decl_extensions::*;

#[path = "runtime_actuators.rs"]
mod runtime_actuators;
#[path = "runtime_assurance.rs"]
mod runtime_assurance;
#[path = "runtime_audit.rs"]
mod runtime_audit;
#[path = "runtime_builtins.rs"]
mod runtime_builtins;
#[path = "runtime_connectivity.rs"]
mod runtime_connectivity;
#[path = "runtime_continuity.rs"]
mod runtime_continuity;
#[path = "runtime_declarations.rs"]
mod runtime_declarations;
#[path = "runtime_eval.rs"]
mod runtime_eval;
#[path = "runtime_execute.rs"]
mod runtime_execute;
#[path = "runtime_faults.rs"]
mod runtime_faults;
#[path = "runtime_health.rs"]
mod runtime_health;
#[path = "runtime_helpers.rs"]
mod runtime_helpers;
#[path = "runtime_kill_switch.rs"]
mod runtime_kill_switch;
#[path = "runtime_navigation.rs"]
mod runtime_navigation;
#[path = "runtime_operational_policy.rs"]
mod runtime_operational_policy;
#[path = "runtime_program.rs"]
mod runtime_program;
#[path = "runtime_recovery.rs"]
mod runtime_recovery;
#[path = "runtime_reliability.rs"]
mod runtime_reliability;
#[path = "runtime_robot.rs"]
mod runtime_robot;
#[path = "runtime_robotics.rs"]
mod runtime_robotics;
#[path = "runtime_safety.rs"]
mod runtime_safety;
#[path = "runtime_scheduler.rs"]
mod runtime_scheduler;
#[path = "runtime_security.rs"]
mod runtime_security;
#[path = "runtime_sensors.rs"]
mod runtime_sensors;
#[path = "runtime_setup.rs"]
mod runtime_setup;
#[path = "runtime_spawn.rs"]
mod runtime_spawn;
#[path = "runtime_tamper.rs"]
mod runtime_tamper;
#[path = "runtime_triggers.rs"]
mod runtime_triggers;
#[path = "runtime_twin.rs"]
mod runtime_twin;
#[path = "runtime_world_model.rs"]
mod runtime_world_model;

pub use runtime_continuity::{
    continuity_context_from_request, execute_continuity_on_program, ContinuityExecutionSnapshot,
};
pub use runtime_recovery::{execute_recovery_on_program, RecoveryExecutionSnapshot};
