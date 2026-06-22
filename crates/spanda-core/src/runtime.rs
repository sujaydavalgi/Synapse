//! runtime support for Spanda.
//!
use crate::ai::{
    create_agent_runtime, create_ai_model, execute_agent_plan, is_action_proposal, is_safe_action,
    mock_analyze_frame, mock_camera_frame, AgentRuntime, AiModel, MemoryStore, PlanExecutor,
};
use crate::ast::{
    ActionDecl, ActuatorDecl, AgentDecl, BehaviorDecl, BinaryOp, Expr, LiteralValue, Program,
    RobotDecl, SafetyRule, SafetyZoneDecl, SensorBinding, SensorDecl, ServiceDecl, SpandaType,
    Stmt, TopicDecl, UnaryOp, UnitKind, ZoneShape,
};
use crate::audit::{AuditRuntime, DeviceIdentity, MockLedgerBackend};
use crate::comm::{CommBus, DiscoverFilter, QosDecl, TransportKind};
use crate::error::{PoseState, RobotState, SpandaError, VelocityState};
use crate::events::EventBus;
use crate::foundations::{
    CapabilityDecl, StateMachineDecl, TaskDecl, TaskPriority, TriggerHandlerDecl, TriggerKind,
    TwinDecl,
};
use crate::hal::{create_sim_hal, hal_member_from_decl, HalBackend, SimHalBackend};
use crate::hardware_monitor::HardwareMonitor;
use crate::reliability_runtime::{
    recover_handlers_from_decls, ModeRuntime, PipelineRuntime, RecoverHandlers, RetryRuntime,
    WatchdogRuntime,
};
use crate::replay::MissionTrace;
use crate::safety::{
    create_safety_config_from_robot, Pose2d, SafetyMonitor, SafetyZoneRuntime,
    SafetyZoneShape,
};
use crate::scheduler::{self, SchedulerClock};
use crate::security::{
    RobotIdentity, SecretHandle, SecretSource, SecurePolicy, SecurityContext, TrustLevel,
};
use crate::soc::get_soc_profile;
use crate::state_machine::StateMachineRuntime;
use crate::transport::{RoutingCommBus, TransportConfig};
use std::cell::RefCell;
use std::rc::Rc;
use crate::triggers::{
    priority_rank, trigger_display_name, ConditionTriggerState, SystemTriggerCategory,
    TriggerRegistry, TriggerTimerSchedule, MAX_TRIGGERS_PER_TICK,
};
use crate::twin::TwinRuntime;
use crate::units::align_for_binary;
use std::collections::HashMap;

type AgentTraitImplBody = (Vec<crate::foundations::TraitParamDecl>, Vec<Stmt>);
type BehaviorContracts = (Vec<Stmt>, Option<Expr>, Option<Expr>, Option<Expr>);
type TaskContracts = (Vec<Stmt>, f64, Option<Expr>, Option<Expr>, Option<Expr>);
pub use spanda_runtime::value::*;
pub use spanda_runtime::environment::Environment;
pub use spanda_runtime::RuntimeError;
use spanda_runtime::RuntimeHost;

/// Convert extracted runtime errors into [`SpandaError`].
pub trait IntoSpandaError {
    fn into_spanda(self) -> SpandaError;
}

impl IntoSpandaError for RuntimeError {
    fn into_spanda(self) -> SpandaError {
        SpandaError::from(self)
    }
}

pub fn pose_from_state(state: &PoseState) -> RuntimeValue {
    // Build a pose runtime value from HAL pose state.
    //
    // Parameters:
    // - `state` — robot pose snapshot from the HAL layer
    //
    // Returns:
    // Pose `RuntimeValue` with optional Z defaulting to zero.
    //
    // Options:
    // None.
    //
    // Example:
    // let pose = pose_from_state(&hal_state.pose);

    runtime_pose(state.x, state.y, state.theta, state.z.unwrap_or(0.0))
}

pub fn velocity_from_state(state: &VelocityState) -> RuntimeValue {
    // Build a velocity runtime value from HAL velocity state.
    //
    // Parameters:
    // - `state` — robot velocity snapshot from the HAL layer
    //
    // Returns:
    // Velocity `RuntimeValue`.
    //
    // Options:
    // None.
    //
    // Example:
    // let vel = velocity_from_state(&hal_state.velocity);

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
        // Set emergency stop.
        //
        // Parameters:
        // - `self` — method receiver
        // - `_active` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.set_emergency_stop(_active);
    }
    fn publish_topic(&mut self, _topic_path: &str, _message_type: &str, _value: RuntimeValue) {
        // Publish topic.
        //
        // Parameters:
        // - `self` — method receiver
        // - `_topic_path` — input value
        // - `_message_type` — input value
        // - `_value` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.publish_topic(_topic_path, _message_type, _value);
    }
    fn call_service(&mut self, _service_name: &str, _service_type: &str) -> RuntimeValue {
        // Call service.
        //
        // Parameters:
        // - `self` — method receiver
        // - `_service_name` — input value
        // - `_service_type` — input value
        //
        // Returns:
        // RuntimeValue.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.call_service(_service_name, _service_type);

        // Build a Bool runtime value.
        RuntimeValue::Bool { value: true }
    }
    fn send_action(
        &mut self,
        _action_name: &str,
        _action_type: &str,
        _goal: RuntimeValue,
    ) -> RuntimeValue {
        // Send action.
        //
        // Parameters:
        // - `self` — method receiver
        // - `_action_name` — input value
        // - `_action_type` — input value
        // - `_goal` — input value
        //
        // Returns:
        // RuntimeValue.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.send_action(_action_name, _action_type, _goal);

        // Build a Bool runtime value.
        RuntimeValue::Bool { value: true }
    }
    fn get_hal(&mut self) -> Option<&mut dyn HalBackend> {
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.get_hal();

        // Return no value for this path.
        None
    }
    fn event_log(&self) -> Vec<String> {
        // Event log.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Vec<String>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.event_log();

        // Return an empty list.
        Vec::new()
    }
}




type LogCallback = Rc<dyn Fn(String)>;
type MotionBlockedCallback = Rc<dyn Fn(String)>;

pub struct InterpreterOptions {
    pub max_loop_iterations: usize,
    pub on_motion_blocked: Option<MotionBlockedCallback>,
    pub on_log: Option<LogCallback>,
    pub module_registry: Option<crate::modules::ModuleRegistry>,
    pub debug: Option<crate::debug::DebugController>,
    pub ffi_registry: crate::ffi::FfiRegistry,
    pub trace_scheduler: bool,
    pub trace_tasks: bool,
    pub trace_triggers: bool,
    pub trace_events: bool,
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

    /// Optional domain provider registry; defaults to bootstrap shims when unset.
    pub provider_registry: Option<crate::providers::ProviderRegistry>,

    /// Official package dependency names from the enclosing project manifest/lockfile.
    pub official_packages: Vec<String>,

    /// Optional domain host for adapter and connectivity hooks; defaults to core host.
    pub runtime_host: Option<&'static dyn RuntimeHost>,
}

impl Default for InterpreterOptions {
    fn default() -> Self {
        //
        // Parameters:
        // None.
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_core::runtime::default();

        // Assemble the struct fields and return it.
        Self {
            max_loop_iterations: 10,
            on_motion_blocked: None,
            on_log: None,
            module_registry: None,
            debug: None,
            ffi_registry: crate::ffi::FfiRegistry::new(),
            trace_scheduler: false,
            trace_tasks: false,
            trace_triggers: false,
            trace_events: false,
            replay_trace: false,
            record_trace: false,
            trace_source: None,
            scheduler_clock: SchedulerClock::Sim,
            replay_deterministic: false,
            max_triggers_per_tick: MAX_TRIGGERS_PER_TICK,
            secure_mode: false,
            inject_security_faults: false,
            provider_registry: None,
            official_packages: Vec::new(),
            runtime_host: None,
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
    hardware_monitor: HardwareMonitor,
    topic_path_to_name: HashMap<String, String>,
    topic_path_to_message_type: HashMap<String, String>,
    ai_confidence_low_active: bool,
    twin_faults_dispatched: std::collections::HashSet<String>,
    audit_runtime: Option<AuditRuntime>,
    mock_ledger: MockLedgerBackend,
    security: SecurityContext,
    comm_bus: RoutingCommBus,
    default_transport: TransportKind,
    module_functions: HashMap<String, crate::foundations::ModuleFnDecl>,
    imported_functions: HashMap<String, crate::foundations::ModuleFnDecl>,
    extern_functions: HashMap<String, crate::foundations::ExternFnDecl>,
    concurrency: crate::concurrency::ConcurrencyRuntime,
    telemetry: crate::telemetry::RuntimeTelemetry,
    active_mode: String,
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
    geofences: Vec<crate::connectivity_positioning::GeofenceRuntime>,
    geofence_active: std::collections::HashSet<String>,
    connectivity_policies: Vec<crate::connectivity_positioning::ConnectivityPolicyRuntime>,
    active_connectivity_link: String,
    connectivity_events_seen: std::collections::HashSet<String>,
    gps_available: bool,
    fleets: crate::robotics_platform::FleetRegistry,
    program_safety_zones: crate::robotics_platform::ProgramSafetyZoneRegistry,
    nav2_enabled: bool,
    slam_enabled: bool,
    provider_registry: Rc<RefCell<crate::providers::ProviderRegistry>>,
    host: &'static dyn RuntimeHost,
}

impl<B: RobotBackend> Interpreter<B> {
    pub fn new(backend: B, mut options: InterpreterOptions) -> Self {
        // Create a new instance.
        //
        // Parameters:
        // - `backend` — input value
        // - `options` — input value
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_core::runtime::new(backend, options);

        // Assemble the struct fields and return it.
        let provider_registry = Rc::new(RefCell::new(
            options
                .provider_registry
                .take()
                .unwrap_or_else(|| {
                    crate::providers::bootstrap_providers_for_packages(
                        &options
                            .official_packages
                            .iter()
                            .map(String::as_str)
                            .collect::<Vec<_>>(),
                    )
                }),
        ));
        let host = options
            .runtime_host
            .unwrap_or_else(|| crate::runtime_host::core_runtime_host());
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
            hardware_monitor: HardwareMonitor::default(),
            topic_path_to_name: HashMap::new(),
            topic_path_to_message_type: HashMap::new(),
            ai_confidence_low_active: false,
            twin_faults_dispatched: std::collections::HashSet::new(),
            audit_runtime: None,
            mock_ledger: MockLedgerBackend::new(),
            security: SecurityContext::new(),
            comm_bus,
            default_transport: TransportKind::Sim,
            module_functions: HashMap::new(),
            imported_functions: HashMap::new(),
            extern_functions: HashMap::new(),
            concurrency: crate::concurrency::ConcurrencyRuntime::new(),
            telemetry: crate::telemetry::RuntimeTelemetry::default(),
            active_mode: "normal".into(),
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
            fleets: crate::robotics_platform::FleetRegistry::default(),
            program_safety_zones: crate::robotics_platform::ProgramSafetyZoneRegistry::default(),
            nav2_enabled: false,
            slam_enabled: false,
            host,
        }
    }

    pub fn runtime_host(&self) -> &dyn RuntimeHost {
        self.host
    }

    pub fn telemetry(&self) -> &crate::telemetry::RuntimeTelemetry {
        // Telemetry.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // &crate::telemetry::RuntimeTelemetry.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.telemetry();

        // Return telemetry from this handle.
        &self.telemetry
    }

    pub fn provider_registry(&self) -> std::cell::Ref<'_, crate::providers::ProviderRegistry> {
        // Return the domain provider registry active for this interpreter session.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Reference to the installed provider registry (includes bootstrap shims).
        //
        // Options:
        // None.
        //
        // Example:
        // let count = interp.provider_registry().transport_count();

        self.provider_registry.borrow()
    }

    pub fn take_telemetry(&mut self) -> crate::telemetry::RuntimeTelemetry {
        // Take telemetry.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // crate::telemetry::RuntimeTelemetry.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.take_telemetry();

        // Move out the stored value and leave a default behind.
        std::mem::take(&mut self.telemetry)
    }

    pub fn twin_replay_export(&self) -> Option<serde_json::Value> {
        // Export digital twin replay frames as JSON when replay is enabled.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Replay JSON or none when no twin replay buffer exists.
        //
        // Options:
        // None.
        //
        // Example:
        // let export = interp.twin_replay_export();

        self.twin
            .as_ref()
            .filter(|twin| twin.replay_frame_count() > 0)
            .map(|twin| twin.export_replay_json())
    }

    pub fn take_mission_trace(&mut self) -> Option<MissionTrace> {
        // Take the recorded mission trace, if any.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Recorded trace or none when recording was disabled.
        //
        // Options:
        // None.
        //
        // Example:
        // let trace = interp.take_mission_trace();

        // Move out the stored trace container.
        self.mission_trace.take()
    }

    fn trace_scheduler_log(&self, message: impl Into<String>) {
        // Trace scheduler log.
        //
        // Parameters:
        // - `self` — method receiver
        // - `message` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.trace_scheduler_log(message);

        // Log scheduler decisions when scheduler tracing is enabled.
        if self.options.trace_scheduler {
            self.log(format!("trace-scheduler: {}", message.into()));
        }
    }

    fn trace_task_log(&self, message: impl Into<String>) {
        // Trace task log.
        //
        // Parameters:
        // - `self` — method receiver
        // - `message` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.trace_task_log(message);

        // Log task lifecycle events when task tracing is enabled.
        if self.options.trace_tasks {
            self.log(format!("trace-task: {}", message.into()));
        }
    }

    fn trace_trigger_log(&self, message: impl Into<String>) {
        // Trace trigger log.
        //
        // Parameters:
        // - `self` — method receiver
        // - `message` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.trace_trigger_log(message);

        // Log trigger evaluation when trigger tracing is enabled.
        if self.options.trace_triggers {
            self.log(format!("trace-trigger: {}", message.into()));
        }
    }

    fn trace_event_log(&self, message: impl Into<String>) {
        // Trace event log.
        //
        // Parameters:
        // - `self` — method receiver
        // - `message` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.trace_event_log(message);

        // Log trigger evaluation when trigger tracing is enabled.
        if self.options.trace_events || self.options.trace_triggers {
            self.log(format!("trace-event: {}", message.into()));
        }
    }

    fn trace_replay_log(&self, message: impl Into<String>) {
        // Trace replay log.
        //
        // Parameters:
        // - `self` — method receiver
        // - `message` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.trace_replay_log(message);

        // Record replay output when trace replay mode is active.
        if self.options.replay_trace {
            self.log(format!("trace-replay: {}", message.into()));
        }
    }

    pub fn robot_backend(&self) -> &B {
        // Robot backend.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // &B.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.robot_backend();

        // Return backend from this handle.
        &self.backend
    }

    pub fn env(&self) -> &Environment {
        // Env.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // &Environment.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.env();

        // Return env from this handle.
        &self.env
    }

    pub fn env_mut(&mut self) -> &mut Environment {
        // Env mut.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // &mut Environment.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.env_mut();

        // Produce env as the result.
        &mut self.env
    }

    pub fn setup_robot_for_debug(&mut self, robot: &RobotDecl) -> Result<(), SpandaError> {
        // Setup robot for debug.
        //
        // Parameters:
        // - `self` — method receiver
        // - `robot` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.setup_robot_for_debug(robot);

        // Call setup robot on the current instance.
        self.setup_robot(robot)
    }

    pub fn debug_execute_stmt(&mut self, stmt: &Stmt) -> Result<(), SpandaError> {
        // Debug execute stmt.
        //
        // Parameters:
        // - `self` — method receiver
        // - `stmt` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.debug_execute_stmt(stmt);

        // Call execute stmt on the current instance.
        self.execute_stmt(stmt)
    }

    pub fn resolve_sync_call(
        &self,
        stmt: &Stmt,
    ) -> Option<(
        String,
        crate::foundations::ModuleFnDecl,
        Vec<crate::ast::Expr>,
    )> {
        // Resolve sync call.
        //
        // Parameters:
        // - `self` — method receiver
        // - `stmt` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.resolve_sync_call(stmt);

        // Import the items needed by the logic below.
        use crate::ast::{Expr, Stmt};
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
        func: &crate::foundations::ModuleFnDecl,
        args: &[crate::ast::Expr],
    ) -> Result<Environment, SpandaError> {
        // Bind call args.
        //
        // Parameters:
        // - `self` — method receiver
        // - `func` — input value
        // - `args` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.bind_call_args(func, args);

        // Save current variable bindings before the call.
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
        // Restore env.
        //
        // Parameters:
        // - `self` — method receiver
        // - `env` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.restore_env(env);

        // Call env = env; on the current instance.
        self.env = env;
    }

    pub fn run(
        &mut self,
        program: &Program,
        entry_behavior: Option<&str>,
    ) -> Result<RobotState, SpandaError> {
        // Run the operation.
        //
        // Parameters:
        // - `self` — method receiver
        // - `program` — input value
        // - `entry_behavior` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.run(program, entry_behavior);

        // Destructure the program into its top-level sections.
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
            use crate::foundations::SimulateCompatibilityDecl;
            let SimulateCompatibilityDecl::SimulateCompatibilityDecl { faults, .. } = sim;
            sim_faults = faults.iter().map(|f| f.fault_type.clone()).collect();
        }
        self.load_program_metadata(program);
        if !self.provider_registry.borrow().official_packages().is_empty() {
            crate::providers::sync_comm_bus_for_official_packages(
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
            }
            self.log("security: injected default security fault scenarios".into());
        }

        // Handle each robot declared in the program.
        for robot in robots {
            self.setup_robot(robot)?;

            // Inject each configured hardware fault.
            for fault in &sim_faults {
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

            // Skip further work when !sim faults is empty.
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

            // Skip further work when behaviors is empty.
            if behaviors.is_empty() && tasks.len() > 1 && entry_behavior.is_none() {
                self.execute_multiplexed_tasks(robot.all_task_schedules())?;
                continue;
            }

            // Skip further work when behaviors is empty.
            if behaviors.is_empty()
                && tasks.is_empty()
                && entry_behavior.is_none()
                && self.has_standalone_triggers()
            {
                self.execute_trigger_only_loop()?;
                continue;
            }
            let behavior_name = entry_behavior
                .map(String::from)
                .or_else(|| robot.first_behavior_name());

            // Emit output when behavior name provides a name.
            if let Some(name) = behavior_name {
                // Take this path when let Some((body, requires, ensures, invariant)) =.
                if let Some((body, requires, ensures, invariant)) =
                    robot.behavior_with_contracts(&name)
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
        Ok(self.backend.get_state())
    }

    pub fn run_tests(&mut self, program: &Program) -> Result<(), SpandaError> {
        // Run tests.
        //
        // Parameters:
        // - `self` — method receiver
        // - `program` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.run_tests(program);

        // Extract test blocks from the parsed program.
        let Program::Program { tests, .. } = program;
        self.load_program_metadata(program);

        // Run each test block in program order.
        for test in tests {
            self.log(format!("test {}", test.name));
            self.execute_block(&test.body)?;
            self.process_spawn_queue()?;
        }
        Ok(())
    }

    pub fn load_program_metadata(&mut self, program: &Program) {
        // Load program metadata.
        //
        // Parameters:
        // - `self` — method receiver
        // - `program` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.load_program_metadata(program);

        // Import the items needed by the logic below.
        use crate::foundations::{EnumDecl, ModuleFnDecl, StructDecl, TraitDecl, Visibility};
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
        use crate::ast::ImportDecl;

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
                    let crate::ast::ImportDecl::ImportDecl { path, .. } = imp;
                    path.as_str()
                })
                .collect();
            spanda_runtime::imports_enable_navigation(&paths, self.host)
        };
        self.slam_enabled = {
            let paths: Vec<&str> = imports
                .iter()
                .map(|imp| {
                    let crate::ast::ImportDecl::ImportDecl { path, .. } = imp;
                    path.as_str()
                })
                .collect();
            spanda_runtime::imports_enable_slam(&paths, self.host)
        };
    }

    fn load_robotics_platform_metadata(
        &mut self,
        fleets: &[crate::robotics_platform::FleetDecl],
        program_safety_zones: &[crate::robotics_platform::ProgramSafetyZoneDecl],
        certifications: &[crate::robotics_platform::CertifyDecl],
    ) {
        // Load fleet groupings, safety zone policies, and certification metadata.
        use crate::robotics_platform::{CertifyDecl, FleetDecl, ProgramSafetyZoneDecl};
        self.fleets = crate::robotics_platform::FleetRegistry::default();
        self.program_safety_zones = crate::robotics_platform::ProgramSafetyZoneRegistry::default();

        // Register each declared fleet and expose it through the fleet runtime object.
        for fleet in fleets {
            let FleetDecl::FleetDecl { name, members, .. } = fleet;
            self.fleets.register(name, members.clone());
            self.log(format!("fleet '{name}': {} member(s)", members.len()));
        }
        self.env.define(
            "fleet",
            RuntimeValue::FleetControl {
                registry: self.fleets.clone(),
            },
        );

        // Register program-level safety zone speed caps.
        for zone in program_safety_zones {
            let ProgramSafetyZoneDecl::ProgramSafetyZoneDecl {
                name,
                max_speed_mps,
                ..
            } = zone;
            if let Some(max_speed) = max_speed_mps {
                self.program_safety_zones.register(name, *max_speed);
                self.log(format!(
                    "safety_zone '{name}': max_speed {:.2} m/s",
                    max_speed
                ));
            }
        }

        // Log declared certification standards (metadata only).
        for cert in certifications {
            let CertifyDecl::CertifyDecl {
                standard, level, ..
            } = cert;
            let level_suffix = level
                .as_deref()
                .map(|l| format!(" level {l}"))
                .unwrap_or_default();
            self.log(format!(
                "certify {}{level_suffix}: metadata recorded (verify-only)",
                standard.as_str()
            ));
        }
    }

    fn setup_robot(&mut self, robot: &RobotDecl) -> Result<(), SpandaError> {
        // Setup robot.
        //
        // Parameters:
        // - `self` — method receiver
        // - `robot` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.setup_robot(robot);

        // Compute RobotDecl for the following logic.
        let RobotDecl::RobotDecl {
            name: robot_name,
            soc,
            hal,
            topics,
            services,
            actions,
            sensors,
            actuators,
            safety,
            ai_models,
            agents,
            state_machines,
            events,
            event_handlers,
            trigger_handlers,
            twin,
            verify,
            observe,
            identity,
            audit,
            provenance,
            signed_records,
            secrets,
            trust,
            permissions,
            trait_impls,
            buses,
            peer_robots,
            devices,
            twin_sync,
            agent_channels,
            secure_comm,
            trust_boundaries,
            mission,
            ..
        } = robot;
        self.env = Environment::new();
        if self.fleets.names().next().is_some() {
            self.env.define(
                "fleet",
                RuntimeValue::FleetControl {
                    registry: self.fleets.clone(),
                },
            );
        }
        let registry = Rc::clone(&self.provider_registry);
        self.comm_bus = RoutingCommBus::new();
        self.comm_bus.attach_provider_registry(registry);
        self.zones.clear();
        self.stop_if_conditions.clear();
        self.event_bus = EventBus::new();
        self.trigger_registry = TriggerRegistry::new();
        self.trigger_timers.clear();
        self.condition_trigger_state = ConditionTriggerState::default();
        self.declared_event_names.clear();
        self.declared_topic_names.clear();
        self.triggers_dispatched_this_tick = 0;
        self.twin = None;
        self.topic_qos.clear();
        self.topic_last_publish_ms.clear();
        self.topic_deadline_misses.clear();
        self.state_machines.clear();
        self.agent_capabilities.clear();
        self.agent_trait_impls.clear();
        self.verify_rules.clear();
        self.verify_warning_rules.clear();
        self.fusion_sensors.clear();
        self.hardware_monitor = HardwareMonitor::default();
        self.topic_path_to_name.clear();
        self.topic_path_to_message_type.clear();
        self.ai_confidence_low_active = false;
        self.twin_faults_dispatched.clear();
        self.audit_runtime = None;
        self.mock_ledger = MockLedgerBackend::new();
        self.security = SecurityContext::new();
        self.geofences.clear();
        self.geofence_active.clear();
        self.connectivity_events_seen.clear();
        self.gps_available = true;
        if let Some(soc_decl) = soc {
            let profile_name = soc_decl.profile();

            // Emit output when get soc profile provides a profile.
            if let Some(profile) = get_soc_profile(profile_name) {
                self.log(format!("SoC: {} ({})", profile.name, profile.architecture));
            } else {
                self.log(format!("SoC: {profile_name} (unknown)"));
            }
        }

        // Emit output when get hal provides a hal backend.
        if let Some(hal_backend) = self.backend.get_hal() {
            let _ = hal_backend;
        }

        // Emit output when hal provides a hal block.
        if let Some(hal_block) = hal {
            let members: Vec<_> = hal_block
                .members()
                .iter()
                .map(hal_member_from_decl)
                .collect();
            self.hal.configure(&members);
            self.log(format!("HAL configured: {} bus(es)/pin(s)", members.len()));
        }

        // Register declared trust boundaries for runtime enforcement.
        for tb in trust_boundaries {
            let crate::foundations::TrustBoundaryDecl::TrustBoundaryDecl { name, .. } = tb;
            if let Ok(kind) = name.parse::<spanda_security::TrustBoundaryKind>() {
                self.security.trust_boundaries.declare(kind);
            }
        }

        // Process each bus.
        for bus in buses {
            let crate::comm::BusDecl::BusDecl {
                transport,
                encryption,
                authentication,
                integrity,
                broker_url,
                ..
            } = bus;
            self.default_transport = *transport;
            let mut bus_security =
                crate::transport_security::TransportSecurityConfig::from_bus_fields(
                    encryption.as_deref(),
                    authentication.as_deref(),
                    integrity.as_deref(),
                )
                .unwrap_or_default();
            if let Some(sc) = secure_comm {
                let crate::foundations::SecureCommPolicyDecl::SecureCommPolicyDecl {
                    encryption,
                    authentication,
                    integrity,
                    ..
                } = sc;
                let robot_policy = spanda_security::SecureCommPolicy {
                    encryption: encryption
                        .as_deref()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or_default(),
                    authentication: authentication
                        .as_deref()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or_default(),
                    integrity: integrity
                        .as_deref()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or_default(),
                };
                bus_security = crate::transport_security::effective_transport_policy(
                    &robot_policy,
                    &bus_security,
                );
            }
            for secret_decl in secrets {
                let crate::foundations::SecretDecl::SecretDecl { name, source, .. } = secret_decl;
                if name.contains("cert") {
                    if let crate::foundations::SecretSourceDecl::File { path } = source {
                        bus_security.cert_path = Some(path.clone());
                    }
                }
                if name.contains("key") {
                    bus_security.key_secret = Some(name.clone());
                    if let crate::foundations::SecretSourceDecl::File { path } = source {
                        bus_security.key_path = Some(path.clone());
                    }
                }
            }
            if let Err(e) = bus_security.validate(transport.as_str()) {
                return Err(RuntimeError::new(format!("bus security: {e}"), 1).into_spanda());
            }
            self.security.configure_wire_session(
                bus_security.cert_path.clone(),
                bus_security.key_secret.clone(),
            );
            let transport_boundary =
                spanda_security::boundary_for_transport_name(transport.as_str());
            self.security.set_transport_context(
                transport_boundary,
                bus_security.encryption,
                bus_security.authentication,
                bus_security.integrity,
            );
            let resolved_broker =
                crate::transport_security::TransportSecurityConfig::resolve_broker_url(
                    broker_url.as_deref(),
                );
            self.comm_bus
                .configure(TransportConfig {
                    node_name: Some(robot_name.clone()),
                    security: bus_security.clone(),
                    broker_url: resolved_broker,
                    ..Default::default()
                })
                .map_err(|e| {
                    RuntimeError::new(format!("transport configure: {e}"), 1).into_spanda()
                })?;
            self.log(format!(
                "bus transport: {} (encryption: {:?})",
                transport.as_str(),
                bus_security.encryption
            ));
        }

        // Process each peer robot.
        for peer in peer_robots {
            let crate::comm::PeerRobotDecl::PeerRobotDecl { name, .. } = peer;
            self.comm_bus.register_robot(name);
        }

        // Process each device.
        for device in devices {
            let crate::comm::DeviceDecl::DeviceDecl { name, .. } = device;
            self.comm_bus.register_device(name);
            self.env.define(
                name.clone(),
                RuntimeValue::Object {
                    type_name: "Device".into(),
                    fields: HashMap::new(),
                },
            );
        }

        // Process each topic.
        for topic in topics {
            let TopicDecl::TopicDecl { name, .. } = topic;
            self.declared_topic_names.insert(name.clone());
            self.define_topic(topic);
        }

        // Process each service.
        for service in services {
            self.define_service(service);
        }

        // Process each action.
        for action in actions {
            self.define_action(action);
        }

        // Process each sensor.
        for sensor in sensors {
            self.define_sensor(sensor);
        }

        // Process each actuator.
        for actuator in actuators {
            self.define_actuator(actuator);
        }
        self.ai_models.clear();
        self.agents.clear();

        // Process each ai model.
        for model_decl in ai_models {
            let model = create_ai_model(model_decl);
            let name = model.name.clone();
            self.env.define(name.clone(), model.to_runtime_value());
            self.log(format!(
                "AI model '{}': {} ({}/{})",
                name, model.model_type, model.config.provider, model.config.model
            ));
            self.ai_models.insert(name, model);
        }

        // Process each agent.
        for agent_decl in agents {
            self.setup_agent(agent_decl);
        }

        // Process each agent channel.
        for channel in agent_channels {
            let crate::comm::AgentChannelDecl::AgentChannelDecl {
                from_agent,
                to_agent,
                message_type,
                ..
            } = channel;
            self.concurrency
                .register_agent_route(from_agent, to_agent, message_type);
            self.log(format!(
                "agent channel: {from_agent} -> {to_agent}{}",
                // Skip further work when message type is empty.
                if message_type.is_empty() {
                    String::new()
                } else {
                    format!(" ({message_type})")
                }
            ));
        }

        // Process each trait impl.
        for trait_impl in trait_impls {
            use crate::foundations::TraitImplDecl;
            let TraitImplDecl::TraitImplDecl {
                agent_name,
                methods,
                ..
            } = trait_impl;
            let agent_methods = self
                .agent_trait_impls
                .entry(agent_name.clone())
                .or_default();

            // Process each method.
            for method in methods {
                agent_methods.insert(
                    method.name.clone(),
                    (method.params.clone(), method.body.clone()),
                );
            }
        }

        // Process each event.
        for event in events {
            let crate::foundations::EventDecl::EventDecl { name, .. } = event;
            self.declared_event_names.insert(name.clone());
            self.log(format!("event declared: {name}"));
        }

        // Invoke each registered handler.
        for handler in event_handlers {
            let crate::foundations::EventHandlerDecl::EventHandlerDecl {
                event_name, body, ..
            } = handler;
            self.event_bus.register(event_name.clone(), body.clone());
            self.trigger_registry
                .register_legacy_event(event_name.clone(), body.clone());
            self.log(format!("handler registered for {event_name}"));
        }

        // Evaluate each trigger definition.
        for trigger in trigger_handlers {
            self.register_trigger_decl(trigger, None);
        }

        // Process each agent.
        for agent in agents {
            let AgentDecl::AgentDecl {
                name: agent_name,
                trigger_handlers: agent_triggers,
                ..
            } = agent;

            // Evaluate each trigger definition.
            for trigger in agent_triggers {
                self.register_trigger_decl(trigger, Some(agent_name.clone()));
            }
        }
        self.trigger_timers = self
            .trigger_registry
            .timer_handlers()
            .iter()
            .filter_map(|h| TriggerTimerSchedule::from_handler(h))
            .collect();

        // Emit output when twin provides a twin decl.
        if let Some(twin_decl) = twin {
            let TwinDecl::TwinDecl {
                name,
                mirrors,
                replay,
                ..
            } = twin_decl;
            let mut runtime = TwinRuntime::new(name.clone(), mirrors.clone(), *replay);

            // Emit output when twin sync provides a sync.
            if let Some(sync) = twin_sync {
                let crate::comm::TwinSyncDecl::TwinSyncDecl {
                    telemetry,
                    replay: sync_replay,
                    faults,
                    events,
                    ..
                } = sync;
                runtime = runtime.with_sync(*telemetry, *sync_replay, *faults, *events);
            }
            self.twin = Some(runtime);
            self.env
                .define(name.clone(), RuntimeValue::Twin { name: name.clone() });
            self.log(format!(
                "twin {name}: mirrors [{}], replay={replay}",
                mirrors.join(", ")
            ));
        } else if let Some(sync) = twin_sync {
            let crate::comm::TwinSyncDecl::TwinSyncDecl {
                telemetry,
                replay,
                faults,
                events,
                ..
            } = sync;
            let name = format!("{robot_name}Twin");
            let runtime = TwinRuntime::new(name.clone(), Vec::new(), *replay)
                .with_sync(*telemetry, *replay, *faults, *events);
            self.twin = Some(runtime);
            self.env
                .define(name.clone(), RuntimeValue::Twin { name: name.clone() });
            self.log(format!(
                "twin sync for {robot_name}: telemetry={telemetry}, replay={replay}, faults={faults}, events={events}"
            ));
        }

        // Emit output when verify provides a verify decl.
        if let Some(verify_decl) = verify {
            let crate::foundations::VerifyDecl::VerifyDecl {
                rules, warnings, ..
            } = verify_decl;
            self.verify_rules = rules.clone();
            self.verify_warning_rules = warnings.clone();
            self.log(format!(
                "verify: {} rule(s), {} warning(s) registered",
                rules.len(),
                warnings.len()
            ));
        }

        // Emit output when observe provides a observe decl.
        if let Some(observe_decl) = observe {
            let crate::foundations::ObserveDecl::ObserveDecl { sensors, .. } = observe_decl;
            self.fusion_sensors = sensors.clone();
            self.env.define(
                "fusion",
                RuntimeValue::SensorFusion {
                    sensors: sensors.clone(),
                },
            );
            self.log(format!(
                "observe: fusing {} sensor(s) [{}]",
                sensors.len(),
                sensors.join(", ")
            ));
        }

        // Initialize mission controller and navigation helpers when declared.
        if let Some(mission_decl) = mission {
            use crate::foundations::MissionDecl;
            let MissionDecl::MissionDecl {
                name,
                duration_hours,
                steps,
                ..
            } = mission_decl;
            let runtime = crate::robotics_platform::MissionRuntime::new(
                name.clone(),
                steps.clone(),
                *duration_hours,
            );
            self.env
                .define("mission", RuntimeValue::MissionControl { runtime });
            self.env
                .define("navigation", RuntimeValue::NavigationControl { goal: None });
            let label = name.as_deref().unwrap_or("mission");
            self.log(format!(
                "mission '{label}': {} step(s), duration={:?} h",
                steps.len(),
                duration_hours
            ));
        }

        if self.slam_enabled {
            self.env.define("slam", RuntimeValue::SlamControl);
            self.log("slam: adapter enabled (stub localize/map hooks)".into());
        }

        // Emit output when permissions provides a perm decl.
        if let Some(perm_decl) = permissions {
            let crate::foundations::PermissionsDecl::PermissionsDecl { capabilities, .. } =
                perm_decl;
            self.security.enable_strict_permissions();
            self.security.capabilities.grant_all(capabilities);
            self.log(format!(
                "permissions: strict mode, granted {} capability(ies)",
                self.security.capabilities.granted().count()
            ));
        }

        // Emit output when trust provides a trust decl.
        if let Some(trust_decl) = trust {
            let crate::foundations::TrustDecl::TrustDecl { level, .. } = trust_decl;

            // Handle the success value from <TrustLevel>.
            if let Ok(t) = level.parse::<TrustLevel>() {
                self.security.trust = t;
                self.log(format!("trust: level set to {}", t.as_str()));
            }
        }

        // Process each secret.
        for secret_decl in secrets {
            let crate::foundations::SecretDecl::SecretDecl { name, source, .. } = secret_decl;
            let src = match source {
                crate::foundations::SecretSourceDecl::Env { var } => {
                    SecretSource::Env { var: var.clone() }
                }
                crate::foundations::SecretSourceDecl::File { path } => {
                    SecretSource::File { path: path.clone() }
                }
                crate::foundations::SecretSourceDecl::Literal { value } => SecretSource::Literal {
                    value: value.clone(),
                },
            };
            self.security.grant_if_not_strict("secret.read");
            self.security.secrets.register(SecretHandle {
                name: name.clone(),
                source: src,
            });
            self.env
                .define(name.clone(), RuntimeValue::Secret { name: name.clone() });
            self.log(format!("secret '{name}': registered"));
        }

        // Emit output when identity provides a identity decl.
        if let Some(identity_decl) = identity {
            let crate::foundations::IdentityDecl::IdentityDecl { fields, .. } = identity_decl;
            let id = fields
                .iter()
                .find(|(k, _)| k == "id")
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "unknown".into());
            let public_key = fields
                .iter()
                .find(|(k, _)| k == "public_key")
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            let robot_id =
                RobotIdentity::new(id.clone(), public_key.clone()).with_trust(self.security.trust);
            self.env.define(
                String::from("identity"),
                RuntimeValue::Identity {
                    id: id.clone(),
                    public_key: public_key.clone(),
                },
            );

            // Emit output when as mut provides a rt.
            if let Some(rt) = self.audit_runtime.as_mut() {
                rt.identity = Some(DeviceIdentity::new(id.clone(), public_key));
            }
            self.security.set_identity(robot_id);
            self.security.grant_if_not_strict("identity.sign");
            self.security.grant_if_not_strict("identity.verify");
            self.log(format!("identity: device '{id}' registered"));
        }

        // Emit output when audit provides a audit decl.
        if let Some(audit_decl) = audit {
            let crate::foundations::AuditDecl::AuditDecl { name, records, .. } = audit_decl;
            let watched: Vec<String> = records.iter().map(|e| Self::expr_path_string(e)).collect();
            let mut rt = AuditRuntime::new(name.clone(), watched.clone());

            // Emit output when identity provides a identity decl.
            if let Some(identity_decl) = identity {
                let crate::foundations::IdentityDecl::IdentityDecl { fields, .. } = identity_decl;
                let id = fields
                    .iter()
                    .find(|(k, _)| k == "id")
                    .map(|(_, v)| v.clone())
                    .unwrap_or_else(|| "unknown".into());
                let public_key = fields
                    .iter()
                    .find(|(k, _)| k == "public_key")
                    .map(|(_, v)| v.clone())
                    .unwrap_or_default();
                rt = rt.with_identity(DeviceIdentity::new(id, public_key));
            }

            // Emit output when provenance provides a provenance decl.
            if let Some(provenance_decl) = provenance {
                let crate::foundations::ProvenanceDecl::ProvenanceDecl {
                    hash_algo,
                    signed_by,
                    ..
                } = provenance_decl;
                rt = rt.with_provenance(hash_algo.clone(), signed_by.clone());
            }
            self.env.define("audit", RuntimeValue::AuditCtx);
            self.audit_runtime = Some(rt);
            self.security.grant_if_not_strict("audit.write");
            self.security.grant_if_not_strict("audit.read");
            self.log(format!(
                "audit {name}: recording {} field(s) [{}]",
                watched.len(),
                watched.join(", ")
            ));
        }

        // Emit output when provenance provides a provenance decl.
        if let Some(provenance_decl) = provenance {
            let crate::foundations::ProvenanceDecl::ProvenanceDecl { name, .. } = provenance_decl;
            self.log(format!("provenance {name}: sha256 signing enabled"));
        }

        // Process each signed record.
        for signed in signed_records {
            let crate::foundations::SignedRecordDecl::SignedRecordDecl {
                event_name,
                signed_by,
                ..
            } = signed;

            // Emit output when as mut provides a rt.
            if let Some(rt) = self.audit_runtime.as_mut() {
                let _ = rt.record_event(event_name, &format!("signed_by={signed_by}"));
            }
            self.log(format!(
                "signed record stream: {event_name} (signed_by {signed_by})"
            ));
        }
        self.env.define("mock_ledger", RuntimeValue::LedgerCtx);
        self.security.grant_if_not_strict("ledger.anchor");

        // Process each state machine.
        for sm in state_machines {
            let StateMachineDecl::StateMachineDecl {
                name,
                states,
                transitions,
                ..
            } = sm;
            let pairs: Vec<(String, String)> = transitions
                .iter()
                .map(|t| (t.from.clone(), t.to.clone()))
                .collect();
            let runtime = StateMachineRuntime::new(name.clone(), states.clone(), pairs);
            self.log(format!(
                "state_machine {name}: initial state {}",
                runtime.current
            ));
            self.state_machines.insert(name.clone(), runtime);
        }

        // Proceed only when is some is available.
        if safety.is_some() {
            self.env.define("safety", RuntimeValue::SafetyCtx);
        }
        self.env.define("robot", RuntimeValue::Robot);
        let mut max_speed = f64::INFINITY;

        // Emit output when safety provides a safety block.
        if let Some(safety_block) = safety {
            // Process each rule.
            for rule in safety_block.rules() {
                // Match on rule and handle each case.
                match rule {
                    SafetyRule::MaxSpeedRule { value, .. } => {
                        let val = self.eval_expr(value)?;

                        // Take this path when let RuntimeValue::Number { value, .. } = val.
                        if let RuntimeValue::Number { value, .. } = val {
                            max_speed = value;
                        }
                    }
                    SafetyRule::StopIfRule { condition, .. } => {
                        self.stop_if_conditions.push(condition.clone());
                    }
                }
            }

            // Process each zone.
            for zone in safety_block.zones() {
                let evaluated = self.eval_safety_zone(zone)?;
                self.zones.push(evaluated);
            }
        }
        self.safety_monitor = Some(SafetyMonitor::new(create_safety_config_from_robot(
            max_speed,
            vec![],
            self.zones.clone(),
            self.program_safety_zones.speed_caps().clone(),
        )));
        self.load_reliability_config(robot)?;
        Ok(())
    }

    fn load_reliability_config(&mut self, robot: &RobotDecl) -> Result<(), SpandaError> {
        // Load watchdog, pipeline, retry, and recovery runtime state from a robot block.
        //
        // Parameters:
        // - `self` — method receiver
        // - `robot` — parsed robot declaration
        //
        // Returns:
        // Ok when configuration is loaded.
        //
        // Options:
        // None.
        //
        // Example:
        // interp.load_reliability_config(robot)?;

        // Reset reliability runtime containers for this robot.
        self.watchdogs.clear();
        self.pipelines.clear();
        self.retries.clear();
        self.recovers.clear();
        self.modes.clear();
        self.task_heartbeats.clear();
        self.active_mode = "normal".into();

        // Start mission trace recording when enabled in interpreter options.
        if self.options.record_trace {
            let source = self
                .options
                .trace_source
                .clone()
                .unwrap_or_else(|| "program.sd".into());
            self.mission_trace = Some(MissionTrace::new(source));
        } else {
            self.mission_trace = None;
        }

        // Copy parsed reliability declarations into runtime form.
        let RobotDecl::RobotDecl {
            pipelines,
            watchdogs,
            retries,
            recovers,
            modes,
            ..
        } = robot;
        for pipeline in pipelines {
            let runtime = PipelineRuntime::from_decl(pipeline);
            self.pipelines.insert(runtime.name.clone(), runtime);
        }
        for watchdog in watchdogs {
            self.watchdogs.push(WatchdogRuntime::from_decl(watchdog));
        }
        for retry in retries {
            self.retries.push(RetryRuntime::from_decl(retry));
        }
        self.recovers = recover_handlers_from_decls(recovers);
        for mode in modes {
            let runtime = ModeRuntime::from_decl(mode);
            self.modes.insert(runtime.name.clone(), runtime);
        }

        // Enter the default normal mode when declared.
        if self.modes.contains_key("normal") {
            self.enter_mode("normal")?;
        }
        Ok(())
    }

    fn enter_mode(&mut self, mode: &str) -> Result<(), SpandaError> {
        // Switch the active operating mode and run its configuration body.
        //
        // Parameters:
        // - `self` — method receiver
        // - `mode` — mode name without `_mode` suffix
        //
        // Returns:
        // Ok when the mode body completes.
        //
        // Options:
        // None.
        //
        // Example:
        // interp.enter_mode("degraded")?;

        // Update active mode and execute the declared body when present.
        self.active_mode = mode.to_string();
        if let Some(body) = self.modes.get(mode).map(|m| m.body.clone()) {
            self.execute_block(&body)?;
        } else {
            self.log(format!("mode: entered '{mode}' (no body declared)"));
            return Ok(());
        }
        self.record_mission_event("mode_enter", serde_json::json!({ "mode": mode }));
        self.log(format!("mode: entered '{mode}'"));
        Ok(())
    }

    fn check_topic_qos_deadlines(&mut self) {
        // Detect topic publish deadline misses against declared QoS.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // interp.check_topic_qos_deadlines();

        // Compare elapsed sim time since the last publish for each topic.
        let snapshots: Vec<(String, f64, f64)> = self
            .topic_qos
            .iter()
            .filter_map(|(path, qos)| {
                let deadline_ms = qos.deadline_ms?;
                let last = self.topic_last_publish_ms.get(path).copied().unwrap_or(0.0);
                if last <= 0.0 {
                    return None;
                }
                let elapsed = self.sim_time_ms - last;
                if elapsed <= deadline_ms {
                    return None;
                }
                Some((path.clone(), elapsed, deadline_ms))
            })
            .collect();
        for (path, elapsed, deadline_ms) in snapshots {
            let misses = self.topic_deadline_misses.entry(path.clone()).or_insert(0);
            if *misses == 0 || elapsed > deadline_ms * (*misses as f64 + 1.0) {
                *misses += 1;
                self.telemetry
                    .record_topic_deadline_miss(&path, elapsed, deadline_ms);
                self.record_mission_event(
                    "topic_deadline_miss",
                    serde_json::json!({
                        "topic": path,
                        "elapsed_ms": elapsed,
                        "deadline_ms": deadline_ms,
                    }),
                );
                self.log(format!(
                    "topic '{path}': deadline miss ({elapsed:.1}ms > {deadline_ms:.1}ms)"
                ));
            }
        }
    }

    fn capture_replay_state(&self) -> crate::replay::ReplayStateSnapshot {
        // Capture the current robot snapshot for mission trace playback.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Pose, velocity, safety, and mode snapshot.
        //
        // Options:
        // None.
        //
        // Example:
        // let snapshot = interp.capture_replay_state();

        let state = self.backend.get_state();
        crate::replay::ReplayStateSnapshot {
            pose: state.pose,
            velocity: state.velocity,
            emergency_stop: state.emergency_stop,
            active_mode: Some(self.active_mode.clone()),
        }
    }

    fn record_mission_event(&mut self, event: impl Into<String>, payload: serde_json::Value) {
        // Append one frame to the mission trace when recording is enabled.
        //
        // Parameters:
        // - `self` — method receiver
        // - `event` — event label
        // - `payload` — structured payload
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // interp.record_mission_event("task_tick", json!({"task":"sense"}));

        // Skip when trace recording is disabled.
        if self.mission_trace.is_some() {
            let state = self.capture_replay_state();
            let sim_time = self.sim_time_ms;
            if let Some(trace) = self.mission_trace.as_mut() {
                trace.record_with_state(sim_time, event, payload, Some(state));
            }
        }
    }

    fn uses_wall_scheduler(&self) -> bool {
        // Report whether the scheduler should sleep on wall-clock deadlines.
        self.options.scheduler_clock == SchedulerClock::Wall && !self.options.replay_deterministic
    }

    fn touch_task_heartbeat(&mut self, task_name: &str) {
        // Record the latest heartbeat time for watchdog evaluation.
        //
        // Parameters:
        // - `self` — method receiver
        // - `task_name` — watched task name
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // interp.touch_task_heartbeat("SafetyMonitor");

        // Store the current simulation time as the task heartbeat.
        self.task_heartbeats
            .insert(task_name.to_string(), self.sim_time_ms);
    }

    fn check_watchdogs(&mut self) -> Result<(), SpandaError> {
        // Evaluate watchdog timeouts against task heartbeats at the current sim time.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Ok when watchdog bodies finish, or an execution error.
        //
        // Options:
        // None.
        //
        // Example:
        // interp.check_watchdogs()?;

        // Evaluate each declared watchdog handler.
        for index in 0..self.watchdogs.len() {
            let reference_ms = if let Some(target) = &self.watchdogs[index].target {
                *self.task_heartbeats.get(target).unwrap_or(&0.0)
            } else {
                0.0
            };
            let elapsed = self.sim_time_ms - reference_ms;
            let timeout_ms = self.watchdogs[index].timeout_ms;
            let should_fire = elapsed >= timeout_ms
                && self.watchdogs[index]
                    .last_fired_at_ms
                    .map(|last| self.sim_time_ms - last >= timeout_ms)
                    .unwrap_or(true);
            if !should_fire {
                continue;
            }
            let name = self.watchdogs[index].name.clone();
            let body = self.watchdogs[index].body.clone();
            self.watchdogs[index].last_fired_at_ms = Some(self.sim_time_ms);
            self.telemetry
                .record_watchdog_timeout(&name, self.sim_time_ms);
            self.record_mission_event(
                "watchdog_timeout",
                serde_json::json!({ "watchdog": name, "elapsed_ms": elapsed }),
            );
            self.log(format!(
                "watchdog '{name}': timeout after {elapsed:.1}ms (limit {timeout_ms:.1}ms)"
            ));
            self.execute_block(&body)?;
        }
        Ok(())
    }

    fn execute_navigate_stmt(
        &mut self,
        goal: &Expr,
        linear: Option<&Expr>,
        angular: Option<&Expr>,
        line: u32,
    ) -> Result<(), SpandaError> {
        // Execute navigate { goal: ... } sugar over navigation.goal/navigate.
        let goal_text = match self.eval_expr(goal)? {
            RuntimeValue::String { value } => value,
            RuntimeValue::Number { value, .. } => value.to_string(),
            _ => {
                return Err(RuntimeError::new(
                    "navigate.goal requires a text or numeric expression",
                    line,
                )
                .into_spanda());
            }
        };

        // Require a mission-scoped navigation controller in the active robot env.
        match self.env.get("navigation") {
            Some(RuntimeValue::NavigationControl { goal: _ }) => {
                self.env.set(
                    "navigation",
                    RuntimeValue::NavigationControl {
                        goal: Some(goal_text.clone()),
                    },
                );
            }
            _ => {
                return Err(RuntimeError::new(
                    "navigate statement requires a robot with a declared mission",
                    line,
                )
                .into_spanda());
            }
        }

        let linear_mps = if let Some(expr) = linear {
            match self.eval_expr(expr)? {
                RuntimeValue::Number { value, .. } => value,
                _ => 0.2,
            }
        } else {
            0.2
        };
        let angular_rad = if let Some(expr) = angular {
            match self.eval_expr(expr)? {
                RuntimeValue::Number { value, .. } => value,
                _ => 0.0,
            }
        } else {
            0.0
        };

        self.log(format!("navigation: executing goal '{goal_text}'"));
        if self.nav2_enabled || self.topic_path_to_message_type.contains_key("/cmd_vel") {
            if let Some(message_type) = self.topic_path_to_message_type.get("/cmd_vel").cloned() {
                let velocity = runtime_velocity(linear_mps, angular_rad);
                self.backend
                    .publish_topic("/cmd_vel", &message_type, velocity);
                let prefix = if self.nav2_enabled {
                    "Nav2Adapter"
                } else {
                    "Nav2 bridge"
                };
                self.log(format!(
                    "navigation: {prefix} publish /cmd_vel goal='{goal_text}'"
                ));
            }
        }
        Ok(())
    }

    fn execute_pipeline(&mut self, name: &str) -> Result<(), SpandaError> {
        // Execute a named pipeline and record latency-budget telemetry.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — pipeline name
        //
        // Returns:
        // Ok when the pipeline body completes.
        //
        // Options:
        // None.
        //
        // Example:
        // interp.execute_pipeline("obstacle_avoidance")?;

        // Resolve the pipeline body and budget from runtime state.
        let Some(pipeline) = self.pipelines.get(name).cloned() else {
            return Err(RuntimeError::new(format!("unknown pipeline '{name}'"), 0).into_spanda());
        };
        let started = std::time::Instant::now();
        self.execute_block(&pipeline.body)?;
        let duration_ms = started.elapsed().as_secs_f64() * 1000.0;
        let duration_ms = duration_ms.max(RUNTIME_TASK_COST_MS);
        let slow = duration_ms > pipeline.budget_ms;
        self.telemetry
            .record_pipeline_execution(name, pipeline.budget_ms, duration_ms, slow);
        if slow {
            self.log(format!(
                "pipeline '{name}': budget {:.1}ms exceeded ({duration_ms:.2}ms)",
                pipeline.budget_ms
            ));
        } else {
            self.log(format!(
                "pipeline '{name}': completed in {duration_ms:.2}ms (budget {:.1}ms)",
                pipeline.budget_ms
            ));
        }
        self.record_mission_event(
            "pipeline_run",
            serde_json::json!({
                "pipeline": name,
                "duration_ms": duration_ms,
                "budget_ms": pipeline.budget_ms,
            }),
        );
        Ok(())
    }

    fn run_retry_policies(&mut self) -> Result<(), SpandaError> {
        // Run robot-level retry policies when injected hardware faults are active.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Ok when retry and fallback blocks finish.
        //
        // Options:
        // None.
        //
        // Example:
        // interp.run_retry_policies()?;

        // Skip when no retry policies or faults are present.
        if self.retries.is_empty() || !self.hardware_monitor.has_injected_faults() {
            return Ok(());
        }

        // Execute each retry policy until success or fallback.
        for index in 0..self.retries.len() {
            if self.retries[index].exhausted {
                continue;
            }
            while self.retries[index].attempt < self.retries[index].attempts {
                self.retries[index].attempt += 1;
                let attempt = self.retries[index].attempt;
                let attempts = self.retries[index].attempts;
                let backoff_ms = self.retries[index].backoff_ms;
                self.log(format!(
                    "retry: attempt {attempt}/{attempts} (backoff {backoff_ms:.0}ms)"
                ));
                self.record_mission_event(
                    "retry_attempt",
                    serde_json::json!({
                        "attempt": attempt,
                        "max_attempts": attempts,
                    }),
                );
                let body = self.retries[index].body.clone();
                self.execute_block(&body)?;
                if !self.hardware_monitor.has_injected_faults() {
                    self.retries[index].attempt = 0;
                    break;
                }
            }
            if self.retries[index].attempt >= self.retries[index].attempts
                && !self.retries[index].exhausted
            {
                self.retries[index].exhausted = true;
                self.log("retry: exhausted attempts — running fallback".into());
                self.record_mission_event("retry_fallback", serde_json::json!({}));
                let fallback = self.retries[index].fallback.clone();
                self.execute_block(&fallback)?;
            }
        }
        Ok(())
    }

    fn try_invoke_recovery(&mut self, err: &SpandaError) -> Result<bool, SpandaError> {
        // Attempt a declared recovery handler for a runtime error.
        //
        // Parameters:
        // - `self` — method receiver
        // - `err` — runtime error to match
        //
        // Returns:
        // true when a recovery handler ran successfully.
        //
        // Options:
        // None.
        //
        // Example:
        // if interp.try_invoke_recovery(&err)? { ... }

        // Only runtime errors participate in recovery dispatch.
        let SpandaError::Runtime { message, .. } = err else {
            return Ok(false);
        };

        // Match recovery handlers by declared error name or substring.
        for (error_name, body) in self.recovers.clone() {
            if message.contains(&error_name)
                || (error_name == "RuntimeError" && matches!(err, SpandaError::Runtime { .. }))
            {
                self.log(format!("recover: invoking handler for '{error_name}'"));
                self.record_mission_event(
                    "recover",
                    serde_json::json!({ "error": error_name, "message": message }),
                );
                self.execute_block(&body)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn invoke_recovery_for_event(&mut self, event: &str) -> Result<(), SpandaError> {
        // Run a recovery handler keyed by hardware event name.
        //
        // Parameters:
        // - `self` — method receiver
        // - `event` — hardware event label
        //
        // Returns:
        // Ok when a handler completes or none matched.
        //
        // Options:
        // None.
        //
        // Example:
        // interp.invoke_recovery_for_event("LidarFailure")?;

        // Prefer an exact event match, then generic sensor failure handlers.
        let handler_key = if self.recovers.contains_key(event) {
            Some(event.to_string())
        } else if event.ends_with("Failure") && self.recovers.contains_key("SensorFailure") {
            Some("SensorFailure".into())
        } else {
            None
        };
        if let Some(key) = handler_key {
            if let Some(body) = self.recovers.get(&key).cloned() {
                self.log(format!("recover: hardware event '{event}' -> '{key}'"));
                self.record_mission_event(
                    "recover_hardware",
                    serde_json::json!({ "event": event, "handler": key }),
                );
                self.execute_block(&body)?;
            }
        }
        Ok(())
    }

    fn evaluate_stop_if(&mut self, env: &Environment) -> bool {
        // Evaluate stop if.
        //
        // Parameters:
        // - `self` — method receiver
        // - `env` — input value
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.evaluate_stop_if(env);

        // Iterate over clone.
        for condition in &self.stop_if_conditions.clone() {
            let saved = self.env.clone_bindings();
            self.env = env.clone_bindings();
            let result = self.eval_expr(condition);
            self.env = saved;

            // Take this path when let Ok(RuntimeValue::Bool { value: true, .. }) = result.
            if let Ok(RuntimeValue::Bool { value: true, .. }) = result {
                return true;
            }
        }
        false
    }

    fn check_safety_before_motion(&mut self) -> bool {
        // Check safety before motion.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_safety_before_motion();

        // Compute state for the following logic.
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

    fn eval_safety_zone(
        &mut self,
        zone: &SafetyZoneDecl,
    ) -> Result<SafetyZoneRuntime, SpandaError> {
        // Eval safety zone.
        //
        // Parameters:
        // - `self` — method receiver
        // - `zone` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.eval_safety_zone(zone);

        // Compute SafetyZoneDecl for the following logic.
        let SafetyZoneDecl::SafetyZoneDecl {
            name,
            shape,
            x,
            y,
            radius,
            width,
            height,
            ..
        } = zone;
        let mut runtime = SafetyZoneRuntime {
            name: name.clone(),
            shape: match shape {
                ZoneShape::Circle => SafetyZoneShape::Circle,
                ZoneShape::Rect => SafetyZoneShape::Rect,
            },
            x: get_number(&self.eval_expr(x)?, 0.0),
            y: get_number(&self.eval_expr(y)?, 0.0),
            radius: None,
            width: None,
            height: None,
        };

        // Take the branch when *shape equals Circle.
        if *shape == ZoneShape::Circle {
            // Emit output when radius provides a r.
            if let Some(r) = radius {
                runtime.radius = Some(get_number(&self.eval_expr(r)?, 0.0));
            }
        }

        // Take the branch when *shape equals Rect.
        if *shape == ZoneShape::Rect {
            // Emit output when width provides a w.
            if let Some(w) = width {
                runtime.width = Some(get_number(&self.eval_expr(w)?, 0.0));
            }

            // Emit output when height provides a h.
            if let Some(h) = height {
                runtime.height = Some(get_number(&self.eval_expr(h)?, 0.0));
            }
        }
        Ok(runtime)
    }

    fn define_topic(&mut self, topic: &TopicDecl) {
        // Define topic.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.define_topic(topic);

        // Compute TopicDecl for the following logic.
        let TopicDecl::TopicDecl {
            name,
            message_type,
            topic: topic_path,
            transport,
            secure,
            qos,
            ..
        } = topic;
        let path = topic_path.clone().unwrap_or_else(|| format!("/{name}"));
        let transport = transport.unwrap_or(self.default_transport);

        // Emit output when secure provides a block.
        if let Some(block) = secure {
            self.security
                .register_secure_endpoint(&path, Self::secure_policy_from_block(block));
        }
        self.comm_bus.subscribe(&path, name);
        self.topic_path_to_name.insert(path.clone(), name.clone());
        self.topic_path_to_message_type
            .insert(path.clone(), message_type.clone());
        if let Some(qos_decl) = qos {
            self.topic_qos.insert(path.clone(), qos_decl.clone());
        }
        self.env.define(
            name.clone(),
            RuntimeValue::Topic {
                name: name.clone(),
                message_type: message_type.clone(),
                topic_path: path,
            },
        );
        let _ = transport;
    }

    fn define_service(&mut self, service: &ServiceDecl) {
        // Define service.
        //
        // Parameters:
        // - `self` — method receiver
        // - `service` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.define_service(service);

        // Compute ServiceDecl for the following logic.
        let ServiceDecl::ServiceDecl {
            name,
            service_type,
            request_type,
            response_type,
            secure,
            ..
        } = service;
        let endpoint = format!("/service/{name}");

        // Emit output when secure provides a block.
        if let Some(block) = secure {
            self.security
                .register_secure_endpoint(&endpoint, Self::secure_policy_from_block(block));
        }
        let st = service_type
            .clone()
            .or_else(|| response_type.clone())
            .unwrap_or_else(|| name.clone());
        self.env.define(
            name.clone(),
            RuntimeValue::Service {
                name: name.clone(),
                service_type: st,
            },
        );
        let _ = request_type;
    }

    fn define_action(&mut self, action: &ActionDecl) {
        // Define action.
        //
        // Parameters:
        // - `self` — method receiver
        // - `action` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.define_action(action);

        // Compute ActionDecl for the following logic.
        let ActionDecl::ActionDecl {
            name,
            action_type,
            result_type,
            secure,
            ..
        } = action;
        let endpoint = format!("/action/{name}");

        // Emit output when secure provides a block.
        if let Some(block) = secure {
            self.security
                .register_secure_endpoint(&endpoint, Self::secure_policy_from_block(block));
        }
        let at = action_type
            .clone()
            .or_else(|| result_type.clone())
            .unwrap_or_else(|| name.clone());
        self.env.define(
            name.clone(),
            RuntimeValue::Action {
                name: name.clone(),
                action_type: at,
            },
        );
    }

    fn define_sensor(&mut self, sensor: &SensorDecl) {
        // Define sensor.
        //
        // Parameters:
        // - `self` — method receiver
        // - `sensor` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.define_sensor(sensor);

        // Compute SensorDecl for the following logic.
        let SensorDecl::SensorDecl {
            name,
            sensor_type,
            library,
            binding,
            ..
        } = sensor;
        let (topic, hal_binding) = match binding {
            Some(SensorBinding::Topic { path }) => (Some(path.clone()), None),
            Some(SensorBinding::Hal { bus_name }) => (None, Some(bus_name.clone())),
            None => (None, None),
        };
        self.env.define(
            name.clone(),
            RuntimeValue::Sensor {
                name: name.clone(),
                sensor_type: sensor_type.clone(),
                library: library.clone(),
                hal_binding,
                topic,
            },
        );
        self.hardware_monitor
            .register_sensor(name.clone(), sensor_type.clone());
    }

    fn define_actuator(&mut self, actuator: &ActuatorDecl) {
        // Define actuator.
        //
        // Parameters:
        // - `self` — method receiver
        // - `actuator` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.define_actuator(actuator);

        // Compute ActuatorDecl for the following logic.
        let ActuatorDecl::ActuatorDecl {
            name,
            actuator_type,
            ..
        } = actuator;
        self.hardware_monitor
            .register_actuator(name.clone(), actuator_type.clone());
        self.env.define(
            name.clone(),
            RuntimeValue::Actuator {
                name: name.clone(),
                actuator_type: actuator_type.clone(),
            },
        );
    }

    fn setup_agent(&mut self, agent_decl: &AgentDecl) {
        // Setup agent.
        //
        // Parameters:
        // - `self` — method receiver
        // - `agent_decl` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.setup_agent(agent_decl);

        // Compute AgentDecl for the following logic.
        let AgentDecl::AgentDecl {
            name,
            goal,
            memory_kind,
            capabilities,
            ..
        } = agent_decl;
        let memory = memory_kind.map(|k| MemoryStore::new(k.into(), None));
        let agent = create_agent_runtime(agent_decl.clone(), memory);
        self.agents.insert(name.clone(), agent);
        self.agent_capabilities
            .insert(name.clone(), capabilities.clone());
        self.comm_bus.register_agent(name);
        self.env
            .define(name.clone(), RuntimeValue::Agent { name: name.clone() });
        self.log(format!("Agent '{name}': {goal}"));
    }

    fn run_scheduled_task(&mut self, schedule: &TaskSchedule) -> Result<bool, SpandaError> {
        // Run scheduled task.
        //
        // Parameters:
        // - `self` — method receiver
        // - `schedule` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.run_scheduled_task(schedule);

        // use budget when budget is present.

        // Emit output when budget provides a budget.
        if let Some(budget) = &schedule.budget {
            // Emit output when name) provides a metrics.
            if let Some(metrics) = self.telemetry.tasks.get(&schedule.name) {
                // Take this path when metrics.max duration ms > 0.0.
                if metrics.max_duration_ms > 0.0 {
                    // Emit output when task budget violation kind provides a kind.
                    if let Some(kind) = task_budget_violation_kind(
                        budget,
                        metrics.max_duration_ms,
                        schedule.interval_ms,
                    ) {
                        self.telemetry.record_budget_violation(
                            &schedule.name,
                            schedule.priority,
                            schedule.interval_ms,
                        );
                        self.telemetry.record_task_skip(
                            &schedule.name,
                            schedule.priority,
                            schedule.interval_ms,
                        );
                        self.log(format!(
                            "task '{}': {kind} budget exceeded — skipping tick",
                            schedule.name
                        ));
                        self.trace_task_log(format!("{} skipped ({kind} budget)", schedule.name));
                        return Ok(true);
                    }
                }
            }
        }
        let started = std::time::Instant::now();
        let continue_running = match self.execute_task_iteration(
            &schedule.body,
            &schedule.requires,
            &schedule.ensures,
            &schedule.invariant,
            Some(&schedule.name),
        ) {
            Ok(value) => value,
            Err(err) => {
                if self.try_invoke_recovery(&err)? {
                    true
                } else {
                    return Err(err);
                }
            }
        };
        self.touch_task_heartbeat(&schedule.name);
        let measured_ms = started.elapsed().as_secs_f64() * 1000.0;
        let duration_ms = measured_ms.max(RUNTIME_TASK_COST_MS);
        self.telemetry.record_task_duration(
            &schedule.name,
            schedule.priority,
            schedule.interval_ms,
            duration_ms,
        );

        // Emit output when budget provides a budget.
        if let Some(budget) = &schedule.budget {
            // Take this path when let Some(kind) =.
            if let Some(kind) =
                task_budget_violation_kind(budget, duration_ms, schedule.interval_ms)
            {
                self.telemetry.record_budget_violation(
                    &schedule.name,
                    schedule.priority,
                    schedule.interval_ms,
                );
                self.log(format!(
                    "task '{}': {kind} budget exceeded ({duration_ms:.2}ms)",
                    schedule.name
                ));
                self.trace_task_log(format!(
                    "{} budget violation {kind} duration={duration_ms:.2}ms",
                    schedule.name
                ));
            }
        }
        Ok(continue_running)
    }

    fn eval_contract(&mut self, expr: &Expr) -> Result<bool, SpandaError> {
        // Eval contract.
        //
        // Parameters:
        // - `self` — method receiver
        // - `expr` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.eval_contract(expr);

        // Match on eval expr and handle each case.
        match self.eval_expr(expr)? {
            RuntimeValue::Bool { value, .. } => Ok(value),
            _ => Err(RuntimeError::new("Contract expression must be boolean", 0).into_spanda()),
        }
    }

    fn execute_with_contracts(
        &mut self,
        body: &[Stmt],
        requires: &Option<Expr>,
        ensures: &Option<Expr>,
        invariant: &Option<Expr>,
    ) -> Result<(), SpandaError> {
        // Execute with contracts.
        //
        // Parameters:
        // - `self` — method receiver
        // - `body` — input value
        // - `requires` — input value
        // - `ensures` — input value
        // - `invariant` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_with_contracts(body, requires, ensures, invariant);

        // use req when requires is present.

        // Emit output when requires provides a req.
        if let Some(req) = requires {
            // Take the branch when eval contract is false.
            if !self.eval_contract(req)? {
                return Err(RuntimeError::new("requires contract failed", 0).into_spanda());
            }
        }
        self.execute_block(body)?;

        // Emit output when ensures provides a ens.
        if let Some(ens) = ensures {
            // Take the branch when eval contract is false.
            if !self.eval_contract(ens)? {
                return Err(RuntimeError::new("ensures contract failed", 0).into_spanda());
            }
        }

        // Emit output when invariant provides a inv.
        if let Some(inv) = invariant {
            // Take the branch when eval contract is false.
            if !self.eval_contract(inv)? {
                return Err(RuntimeError::new("invariant contract failed", 0).into_spanda());
            }
        }
        self.run_verify_rules()?;
        self.run_verify_warnings()?;
        Ok(())
    }

    fn run_verify_warnings(&mut self) -> Result<(), SpandaError> {
        // Run verify warnings.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.run_verify_warnings();

        // Compute warnings for the following logic.
        let warnings = self.verify_warning_rules.clone();

        // Skip further work when warnings is empty.
        if warnings.is_empty() {
            return Ok(());
        }

        // Iterate over enumerate with destructured elements.
        for (index, rule) in warnings.iter().enumerate() {
            // Match on eval expr and handle each case.
            match self.eval_expr(rule)? {
                RuntimeValue::Bool { value: false, .. } => {
                    let _ = self
                        .dispatch_system_trigger(SystemTriggerCategory::Verification, "Warning");
                    self.log(format!("verify warning {} triggered", index + 1));
                }
                RuntimeValue::Bool { value: true, .. } => {}
                _ => {
                    return Err(RuntimeError::new(
                        format!("verify warning {} must be boolean", index + 1),
                        0,
                    )
                    .into_spanda());
                }
            }
        }
        Ok(())
    }

    fn run_verify_rules(&mut self) -> Result<(), SpandaError> {
        // Run verify rules.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.run_verify_rules();

        // Compute rules for the following logic.
        let rules = self.verify_rules.clone();

        // Skip further work when rules is empty.
        if rules.is_empty() {
            return Ok(());
        }

        // Iterate over enumerate with destructured elements.
        for (index, rule) in rules.iter().enumerate() {
            // Match on eval expr and handle each case.
            match self.eval_expr(rule)? {
                RuntimeValue::Bool { value: true, .. } => {}
                RuntimeValue::Bool { value: false, .. } => {
                    let _ =
                        self.dispatch_system_trigger(SystemTriggerCategory::Verification, "Failed");
                    return Err(
                        RuntimeError::new(format!("verify rule {} failed", index + 1), 0)
                            .into_spanda(),
                    );
                }
                _ => {
                    return Err(RuntimeError::new(
                        format!("verify rule {} must be boolean", index + 1),
                        0,
                    )
                    .into_spanda());
                }
            }
        }
        self.log(format!("verify: all {} rule(s) passed", rules.len()));
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn execute_task_loop_with_contracts(
        &mut self,
        task_name: &str,
        priority: TaskPriority,
        body: &[Stmt],
        interval_ms: f64,
        requires: &Option<Expr>,
        ensures: &Option<Expr>,
        invariant: &Option<Expr>,
        budget: Option<crate::foundations::ResourceBudgetDecl>,
    ) -> Result<(), SpandaError> {
        // Execute task loop with contracts.
        //
        // Parameters:
        // - `self` — method receiver
        // - `task_name` — input value
        // - `priority` — input value
        // - `body` — input value
        // - `interval_ms` — input value
        // - `requires` — input value
        // - `ensures` — input value
        // - `invariant` — input value
        // - `budget` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_task_loop_with_contracts(task_name, priority, body, interval_ms, requires, ensures, invariant, budget);

        // Call record scheduler start on the current instance.
        self.telemetry.record_scheduler_start(1, interval_ms);
        self.trace_scheduler_log(format!(
            "single-task {task_name} interval={interval_ms}ms priority={}",
            priority_label(priority)
        ));
        let schedule = TaskSchedule {
            name: task_name.to_string(),
            priority,
            interval_ms,
            deadline_ms: None,
            jitter_ms_max: None,
            isolated: false,
            next_due_ms: 0.0,
            last_start_ms: None,
            body: body.to_vec(),
            requires: requires.clone(),
            ensures: ensures.clone(),
            invariant: invariant.clone(),
            budget,
        };

        let wall_mode = self.uses_wall_scheduler();
        let wall_start = std::time::Instant::now();
        let mut next_deadline = wall_start;

        // Process each max loop iteration.
        for iteration in 0..self.options.max_loop_iterations {
            let sim_time = if wall_mode {
                let deadline = scheduler::advance_wall_tick(&mut next_deadline, interval_ms);
                scheduler::sleep_until(deadline);
                scheduler::elapsed_ms(wall_start, std::time::Instant::now())
            } else {
                (iteration as f64 + 1.0) * interval_ms
            };
            self.backend.tick(interval_ms);
            self.sim_time_ms = sim_time;
            self.triggers_dispatched_this_tick = 0;
            self.telemetry.record_scheduler_tick();
            self.telemetry
                .record_task_tick(task_name, priority, interval_ms);
            self.trace_task_log(format!(
                "{task_name} tick={} priority={} interval={interval_ms}ms",
                iteration + 1,
                priority_label(priority)
            ));
            self.run_timer_triggers(sim_time)?;
            self.run_condition_triggers()?;
            self.run_trigger_maintenance()?;

            // Take the branch when run scheduled task is false.
            let continue_running = self.run_scheduled_task(&schedule)?;
            self.check_watchdogs()?;
            self.check_topic_qos_deadlines();
            self.record_mission_event(
                "scheduler_tick",
                serde_json::json!({ "sim_time_ms": sim_time, "task": task_name }),
            );
            if !continue_running {
                self.telemetry.record_emergency_stop();
                break;
            }
            self.run_verify_rules()?;
            self.update_twin_snapshot();
        }
        Ok(())
    }

    fn execute_task_iteration(
        &mut self,
        body: &[Stmt],
        requires: &Option<Expr>,
        ensures: &Option<Expr>,
        invariant: &Option<Expr>,
        task_name: Option<&str>,
    ) -> Result<bool, SpandaError> {
        // Execute task iteration.
        //
        // Parameters:
        // - `self` — method receiver
        // - `body` — input value
        // - `requires` — input value
        // - `ensures` — input value
        // - `invariant` — input value
        // - `task_name` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_task_iteration(body, requires, ensures, invariant, task_name);

        // use req when requires is present.

        // Emit output when requires provides a req.
        if let Some(req) = requires {
            // Take the branch when eval contract is false.
            if !self.eval_contract(req)? {
                let label = task_name
                    .map(|name| format!("task '{name}'"))
                    .unwrap_or_else(|| "task".into());

                // Emit output when task name provides a name.
                if let Some(name) = task_name {
                    self.telemetry
                        .record_task_skip(name, TaskPriority::Normal, 0.0);
                    self.trace_task_log(format!("{name} skipped (requires failed)"));
                }
                self.log(format!(
                    "{label} requires contract failed — skipping iteration"
                ));
                return Ok(true);
            }
        }
        self.execute_block(body).or_else(|err| {
            if self.try_invoke_recovery(&err)? {
                Ok(())
            } else {
                Err(err)
            }
        })?;

        // Emit output when ensures provides a ens.
        if let Some(ens) = ensures {
            // Take the branch when eval contract is false.
            if !self.eval_contract(ens)? {
                return Err(RuntimeError::new("task ensures contract failed", 0).into_spanda());
            }
        }

        // Emit output when invariant provides a inv.
        if let Some(inv) = invariant {
            // Take the branch when eval contract is false.
            if !self.eval_contract(inv)? {
                return Err(RuntimeError::new("task invariant contract failed", 0).into_spanda());
            }
        }
        let stop = self
            .safety_monitor
            .as_ref()
            .map(|m| m.is_emergency_stop())
            .unwrap_or(false);
        Ok(!stop)
    }

    fn execute_multiplexed_tasks(&mut self, tasks: Vec<TaskSchedule>) -> Result<(), SpandaError> {
        // Execute multiplexed tasks.
        //
        // Parameters:
        // - `self` — method receiver
        // - `tasks` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_multiplexed_tasks(tasks);

        // skip further work when tasks is empty.
        if tasks.is_empty() {
            return Ok(());
        }
        let base_tick = tasks
            .iter()
            .map(|task| task.interval_ms)
            .fold(f64::INFINITY, f64::min)
            .max(1.0);
        let mut schedules = tasks;
        let mut sim_time = 0.0;
        self.telemetry
            .record_scheduler_start(schedules.len() as u64, base_tick);
        self.log(format!(
            "scheduler: multiplexing {} task(s) with base tick {}ms",
            schedules.len(),
            base_tick
        ));
        self.trace_scheduler_log(format!(
            "start tasks={} base_tick={base_tick}ms",
            schedules.len()
        ));
        let wall_mode = self.uses_wall_scheduler();
        let wall_start = std::time::Instant::now();
        let mut next_deadline = wall_start;

        // Process each max loop iteration.
        for iteration in 0..self.options.max_loop_iterations {
            let dt_ms = if wall_mode {
                let deadline = scheduler::advance_wall_tick(&mut next_deadline, base_tick);
                scheduler::sleep_until(deadline);
                sim_time = scheduler::elapsed_ms(wall_start, std::time::Instant::now());
                base_tick
            } else {
                sim_time += base_tick;
                base_tick
            };
            self.backend.tick(dt_ms);
            self.sim_time_ms = sim_time;
            self.triggers_dispatched_this_tick = 0;
            self.telemetry.record_scheduler_tick();
            self.trace_scheduler_log(format!("tick={} sim_time={sim_time}ms", iteration + 1));
            self.run_timer_triggers(sim_time)?;
            self.run_condition_triggers()?;
            self.run_trigger_maintenance()?;
            schedules.sort_by_key(|schedule| schedule.priority_rank());

            // Process each schedule.
            for schedule in &mut schedules {
                // Take this path when schedule.next due ms <= sim time.
                if schedule.next_due_ms <= sim_time {
                    // Record release jitter against the declared bound before running the task.
                    if let Some(max_jitter) = schedule.jitter_ms_max {
                        let lateness = (sim_time - schedule.next_due_ms).max(0.0);
                        self.telemetry.record_task_jitter(
                            &schedule.name,
                            schedule.priority,
                            schedule.interval_ms,
                            lateness,
                            max_jitter,
                        );
                    }
                    // Take this path when sim time > schedule.next due ms + declared deadline slack.
                    let deadline = schedule.deadline_ms.unwrap_or(schedule.interval_ms);
                    if sim_time > schedule.next_due_ms + deadline {
                        self.telemetry.record_missed_deadline(
                            &schedule.name,
                            schedule.priority,
                            schedule.interval_ms,
                        );
                        self.trace_task_log(format!(
                            "{} missed deadline at sim_time={sim_time}ms",
                            schedule.name
                        ));
                    }
                    self.telemetry.record_task_tick(
                        &schedule.name,
                        schedule.priority,
                        schedule.interval_ms,
                    );
                    self.log(format!("task '{}': tick", schedule.name));
                    self.trace_task_log(format!(
                        "{} tick priority={} interval={}ms",
                        schedule.name,
                        priority_label(schedule.priority),
                        schedule.interval_ms
                    ));
                    schedule.last_start_ms = Some(sim_time);

                    // Take the branch when run scheduled task is false.
                    if !self.run_scheduled_task(schedule)? {
                        self.telemetry.record_emergency_stop();
                        return Ok(());
                    }
                    schedule.next_due_ms = sim_time + schedule.interval_ms;
                }
            }
            self.check_watchdogs()?;
            self.check_topic_qos_deadlines();
            self.record_mission_event(
                "scheduler_tick",
                serde_json::json!({ "sim_time_ms": sim_time, "iteration": iteration + 1 }),
            );
            self.run_verify_rules()?;
            self.update_twin_snapshot();

            // Take this path when self.
            if self
                .safety_monitor
                .as_ref()
                .map(|m| m.is_emergency_stop())
                .unwrap_or(false)
            {
                self.telemetry.record_emergency_stop();
                break;
            }
        }
        Ok(())
    }

    fn execute_trigger_only_loop(&mut self) -> Result<(), SpandaError> {
        // Execute trigger only loop.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_trigger_only_loop();

        // Compute base tick for the following logic.
        let base_tick = self
            .trigger_timers
            .iter()
            .map(|t| t.interval_ms)
            .fold(f64::INFINITY, f64::min)
            .max(1.0);
        let mut sim_time = 0.0;
        self.log(format!(
            "scheduler: trigger-only loop with base tick {base_tick}ms"
        ));
        self.trace_scheduler_log(format!("trigger-only base_tick={base_tick}ms"));
        let wall_mode = self.uses_wall_scheduler();
        let wall_start = std::time::Instant::now();
        let mut next_deadline = wall_start;

        // Process each max loop iteration.
        for iteration in 0..self.options.max_loop_iterations {
            let dt_ms = if wall_mode {
                let deadline = scheduler::advance_wall_tick(&mut next_deadline, base_tick);
                scheduler::sleep_until(deadline);
                sim_time = scheduler::elapsed_ms(wall_start, std::time::Instant::now());
                base_tick
            } else {
                sim_time += base_tick;
                base_tick
            };
            self.backend.tick(dt_ms);
            self.sim_time_ms = sim_time;
            self.triggers_dispatched_this_tick = 0;
            self.telemetry.record_scheduler_tick();
            self.run_timer_triggers(sim_time)?;
            self.run_condition_triggers()?;
            self.run_trigger_maintenance()?;
            self.run_verify_rules()?;
            self.run_verify_warnings()?;
            self.update_twin_snapshot();

            // Take this path when self.
            if self
                .safety_monitor
                .as_ref()
                .map(|m| m.is_emergency_stop())
                .unwrap_or(false)
            {
                break;
            }
            let _ = iteration;
        }
        Ok(())
    }

    fn update_twin_snapshot(&mut self) {
        // Update twin snapshot.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.update_twin_snapshot();

        // Compute divergence threshold for the following logic.
        let divergence_threshold = 0.15;

        // Emit output when twin provides a twin.
        if let Some(twin) = &self.twin {
            let state = self.backend.get_state();
            let live = TwinRuntime::live_mirrored_fields(
                (
                    state.pose.x,
                    state.pose.y,
                    state.pose.theta,
                    state.pose.z.unwrap_or(0.0),
                ),
                (state.velocity.linear, state.velocity.angular),
                &twin.mirrors,
            );

            // Take this path when twin.detect divergence(&live, divergence threshold).
            if twin.detect_divergence(&live, divergence_threshold) {
                let _ =
                    self.dispatch_system_trigger(SystemTriggerCategory::Twin, "DivergenceDetected");
            }
        }
        self.refresh_twin_shadow_from_backend();
        let Some(twin) = &mut self.twin else {
            return;
        };
        twin.commit_frame();
        let twin_name = twin.name.clone();
        let field_count = twin.shadow.len();
        let replay_frames = twin.replay_frame_count();

        // Take this path when field count > 0 || twin.telemetry sync.
        if field_count > 0 || twin.telemetry_sync {
            self.log(format!(
                "twin {twin_name} mirrored {field_count} field(s), replay frames={replay_frames}"
            ));
        }
    }

    fn refresh_twin_shadow_from_backend(&mut self) {
        // Refresh twin shadow from backend.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.refresh_twin_shadow_from_backend();

        // Compute Some for the following logic.
        let Some(twin) = &mut self.twin else {
            return;
        };
        let state = self.backend.get_state();

        // Take the branch when any equals "pose").
        if twin.mirrors.iter().any(|m| m == "pose") {
            twin.snapshot(
                "pose",
                RuntimeValue::Pose {
                    x: state.pose.x,
                    y: state.pose.y,
                    theta: state.pose.theta,
                    z: state.pose.z.unwrap_or(0.0),
                },
            );
        }

        // Take the branch when any equals "velocity").
        if twin.mirrors.iter().any(|m| m == "velocity") {
            twin.snapshot(
                "velocity",
                RuntimeValue::Velocity {
                    linear: state.velocity.linear,
                    angular: state.velocity.angular,
                },
            );
        }
    }

    fn has_standalone_triggers(&self) -> bool {
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.has_standalone_triggers();

        // skip further work when trigger timers is empty.
        if !self.trigger_timers.is_empty() {
            return true;
        }
        self.trigger_registry
            .condition_handlers()
            .iter()
            .any(|h| matches!(h.kind, TriggerKind::Condition { level: true, .. }))
    }

    fn register_trigger_decl(&mut self, trigger: &TriggerHandlerDecl, agent: Option<String>) {
        // Register trigger decl.
        //
        // Parameters:
        // - `self` — method receiver
        // - `trigger` — input value
        // - `agent` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.register_trigger_decl(trigger, agent);

        // Compute TriggerHandlerDecl for the following logic.
        let TriggerHandlerDecl::TriggerHandlerDecl {
            trigger_kind,
            priority,
            body,
            span,
        } = trigger;
        let final_kind = if let TriggerKind::Event { name } = trigger_kind {
            // Check membership before continuing.
            if self.declared_topic_names.contains(name) && !self.declared_event_names.contains(name)
            {
                TriggerKind::Message {
                    topic: name.clone(),
                }
            } else {
                (*trigger_kind).clone()
            }
        } else {
            (*trigger_kind).clone()
        };
        let decl = TriggerHandlerDecl::TriggerHandlerDecl {
            trigger_kind: final_kind.clone(),
            priority: *priority,
            body: body.clone(),
            span: *span,
        };
        let name = trigger_display_name(&final_kind, agent.as_deref());
        self.trigger_registry.register(&decl, agent);
        self.log(format!(
            "trigger registered: {name} priority={}",
            priority_label(*priority)
        ));
    }

    fn can_dispatch_trigger(&mut self) -> bool {
        // Can dispatch trigger.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.can_dispatch_trigger();

        // Call max triggers per tick on the current instance.
        self.triggers_dispatched_this_tick < self.options.max_triggers_per_tick
    }

    fn execute_trigger_handlers(&mut self, handler_ids: Vec<usize>) -> Result<(), SpandaError> {
        // Execute trigger handlers.
        //
        // Parameters:
        // - `self` — method receiver
        // - `handler_ids` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_trigger_handlers(handler_ids);

        // Create mutable ids for accumulating results.
        let mut ids = handler_ids;
        ids.sort_by_key(|id| {
            self.trigger_registry
                .get(*id)
                .map(|h| priority_rank(h.priority))
                .unwrap_or(u8::MAX)
        });

        // Iterate over ids.
        for id in ids {
            // Take the branch when execute trigger body by id is false.
            if !self.execute_trigger_body_by_id(id)? {
                break;
            }

            // Take this path when self.
            if self
                .safety_monitor
                .as_ref()
                .map(|m| m.is_emergency_stop())
                .unwrap_or(false)
            {
                break;
            }
        }
        Ok(())
    }

    fn execute_trigger_body_by_id(&mut self, handler_id: usize) -> Result<bool, SpandaError> {
        // Execute trigger body by id.
        //
        // Parameters:
        // - `self` — method receiver
        // - `handler_id` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_trigger_body_by_id(handler_id);

        // Bind a local value for the next steps.
        let (name, kind, priority, body, agent) = {
            let handler = self
                .trigger_registry
                .get(handler_id)
                .ok_or_else(|| RuntimeError::new("unknown trigger handler", 0).into_spanda())?;
            (
                handler.name.clone(),
                handler.kind.clone(),
                handler.priority,
                handler.body.clone(),
                handler.agent.clone(),
            )
        };

        // Take the branch when can dispatch trigger is false.
        if !self.can_dispatch_trigger() {
            self.trace_trigger_log(format!("{name} suppressed (trigger storm limit)"));
            return Ok(false);
        }
        self.triggers_dispatched_this_tick += 1;
        let start = std::time::Instant::now();
        let saved_agent = self.current_agent.clone();

        // Emit output when agent provides a agent.
        if let Some(agent) = &agent {
            self.current_agent = Some(agent.clone());
        }
        let category = trigger_category_label(&kind);
        self.trace_trigger_log(format!(
            "dispatch {name} priority={} category={category}",
            priority_label(priority)
        ));

        // Keep entries that match the expected pattern.
        if matches!(kind, TriggerKind::Event { .. }) {
            self.trace_event_log(format!("dispatch {name}"));
        }
        let result = self.execute_block(&body);
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        let failed = result.is_err();
        self.telemetry
            .record_trigger_execution(&name, category, priority, duration_ms, failed);
        self.current_agent = saved_agent;
        result?;
        Ok(true)
    }

    fn dispatch_event(&mut self, event_name: &str) -> Result<(), SpandaError> {
        //
        // Parameters:
        // - `self` — method receiver
        // - `event_name` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.dispatch_event(event_name);

        // Compute ids for the following logic.
        let ids: Vec<usize> = self
            .trigger_registry
            .handlers_for_event(event_name)
            .iter()
            .map(|h| h.id)
            .collect();

        // Skip further work when !ids is empty.
        if !ids.is_empty() {
            self.trace_event_log(format!("emit {event_name}"));
            self.log(format!("emit {event_name}"));
            return self.execute_trigger_handlers(ids);
        }

        // Emit output when to vec provides a body.
        if let Some(body) = self.event_bus.handler_body(event_name).map(|b| b.to_vec()) {
            self.trace_event_log(format!("emit {event_name} (legacy)"));
            self.log(format!("emit {event_name}"));
            self.execute_block(&body)?;
        } else {
            self.log(format!("emit {event_name} (no handler)"));
        }
        Ok(())
    }

    fn execute_enter(&mut self, state_name: &str, line: u32) -> Result<(), SpandaError> {
        // Execute enter.
        //
        // Parameters:
        // - `self` — method receiver
        // - `state_name` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_enter(state_name, line);

        // Create mutable logs for accumulating results.
        let mut logs = Vec::new();
        let mut transitioned = false;
        let mut previous_states = Vec::new();

        // Iterate over state machines with destructured elements.
        for (sm_name, sm) in &mut self.state_machines {
            // Emit output when try enter provides a previous.
            if let Some(previous) = sm.try_enter(state_name) {
                logs.push(format!(
                    "state_machine {sm_name}: {previous} -> {state_name}"
                ));
                previous_states.push(previous);
                transitioned = true;
            }
        }

        // Process each log.
        for msg in logs {
            self.log(msg);
        }

        // Take the branch when transitioned is false.
        if !transitioned {
            return Err(RuntimeError::new(
                format!("No valid transition to state '{state_name}'"),
                line,
            )
            .into_spanda());
        }

        // Process each previous state.
        for previous in previous_states {
            let ids: Vec<usize> = self
                .trigger_registry
                .handlers_for_state_exited(&previous)
                .iter()
                .map(|h| h.id)
                .collect();
            self.execute_trigger_handlers(ids)?;
        }
        let ids: Vec<usize> = self
            .trigger_registry
            .handlers_for_state_entered(state_name)
            .iter()
            .map(|h| h.id)
            .collect();
        self.execute_trigger_handlers(ids)?;
        Ok(())
    }

    fn check_agent_capability(
        &self,
        agent: &str,
        action: &str,
        target: Option<&str>,
        line: u32,
    ) -> Result<(), SpandaError> {
        // Check agent capability.
        //
        // Parameters:
        // - `self` — method receiver
        // - `agent` — input value
        // - `action` — input value
        // - `target` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_agent_capability(agent, action, target, line);

        // Compute caps for the following logic.
        let caps = self
            .agent_capabilities
            .get(agent)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        // Skip further work when caps is empty.
        if caps.is_empty() {
            return Ok(());
        }
        let allowed = caps
            .iter()
            .any(|c| c.action == action && (target.is_none() || c.target.as_deref() == target));

        // Take the branch when allowed is false.
        if !allowed {
            return Err(RuntimeError::new(
                format!(
                    "Agent '{agent}' lacks capability {action}{}",
                    target.map(|t| format!("({t})")).unwrap_or_default()
                ),
                line,
            )
            .into_spanda());
        }
        Ok(())
    }

    fn publish_source_id(&self) -> String {
        if let Some(agent) = &self.current_agent {
            return agent.clone();
        }
        self.security
            .identity
            .as_ref()
            .map(|id| id.id().to_string())
            .unwrap_or_else(|| "robot".into())
    }

    fn secure_policy_from_block(block: &crate::foundations::SecureBlockDecl) -> SecurePolicy {
        // Secure policy from block.
        //
        // Parameters:
        // - `block` — input value
        //
        // Returns:
        // SecurePolicy.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::runtime::secure_policy_from_block(block);

        // Produce SecurePolicy as the result.
        SecurePolicy {
            signed: block.signed,
            min_trust: block
                .min_trust
                .as_ref()
                .and_then(|s| s.parse::<TrustLevel>().ok()),
            requires: block.requires.clone(),
            encryption: block
                .encryption
                .as_deref()
                .and_then(|s| s.parse().ok())
                .unwrap_or_default(),
            authentication: block
                .authentication
                .as_deref()
                .and_then(|s| s.parse().ok())
                .unwrap_or_default(),
            integrity: block
                .integrity
                .as_deref()
                .and_then(|s| s.parse().ok())
                .unwrap_or_default(),
            trusted_sources: block.trusted_sources.clone(),
            reject_untrusted: block.reject_untrusted,
        }
    }

    fn resolve_signing_key(&self, key: &str) -> Result<String, SpandaError> {
        // Resolve signing key.
        //
        // Parameters:
        // - `self` — method receiver
        // - `key` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.resolve_signing_key(key);

        // proceed only when is ok is available.
        if self.security.secrets.get(key).is_ok() {
            self.security
                .secrets
                .resolve(key)
                .map_err(|e| RuntimeError::new(e.to_string(), 0).into_spanda())
        } else {
            Ok(key.to_string())
        }
    }

    fn security_error(&self, err: crate::security::SecurityError, line: u32) -> SpandaError {
        // Security error.
        //
        // Parameters:
        // - `self` — method receiver
        // - `err` — input value
        // - `line` — input value
        //
        // Returns:
        // SpandaError.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.security_error(err, line);

        // Produce into spanda as the result.
        RuntimeError::new(err.to_string(), line).into_spanda()
    }

    fn execute_block(&mut self, stmts: &[Stmt]) -> Result<(), SpandaError> {
        // Execute block.
        //
        // Parameters:
        // - `self` — method receiver
        // - `stmts` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_block(stmts);

        // Execute each statement in sequence.
        for stmt in stmts {
            self.execute_stmt(stmt)?;
        }
        Ok(())
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> Result<(), SpandaError> {
        // Execute stmt.
        //
        // Parameters:
        // - `self` — method receiver
        // - `stmt` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_stmt(stmt);

        // use debug when debug is present.

        // Emit output when debug provides a debug.
        if let Some(debug) = &self.options.debug {
            let line = crate::debug::stmt_line(stmt);

            // Take this path when debug.should pause(line).
            if debug.should_pause(line) {
                let variables = self.env.snapshot_display();
                debug.record_pause(line, "breakpoint", variables);
                return Err(SpandaError::DebugPause {
                    line,
                    reason: "breakpoint".into(),
                });
            }
        }

        // Match on stmt and handle each case.
        match stmt {
            Stmt::VarDecl {
                name,
                type_annotation,
                init,
                ..
            } => {
                // Emit output when init provides a expr.
                if let Some(expr) = init {
                    let value = if matches!(type_annotation, Some(SpandaType::TraitObject { .. })) {
                        // Take this path when let Expr::IdentExpr { name: agent, .. } = expr.
                        if let Expr::IdentExpr { name: agent, .. } = expr {
                            // Take this path when let Some(SpandaType::TraitObject { trait name }) = type annotation.
                            if let Some(SpandaType::TraitObject { trait_name }) = type_annotation {
                                RuntimeValue::TraitObject {
                                    trait_name: trait_name.clone(),
                                    agent: agent.clone(),
                                }
                            } else {
                                self.eval_expr(expr)?
                            }
                        } else {
                            self.eval_expr(expr)?
                        }
                    } else {
                        self.eval_expr(expr)?
                    };
                    self.env.define(name.clone(), value);
                } else {
                    self.env.define(name.clone(), RuntimeValue::Void);
                }
            }
            Stmt::IfStmt {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let cond = self.eval_expr(condition)?;

                // Keep entries that match the expected pattern.
                if matches!(cond, RuntimeValue::Bool { value: true, .. }) {
                    self.execute_block(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute_block(else_branch)?;
                }
            }
            Stmt::LoopStmt {
                interval_ms, body, ..
            } => {
                // Process each max loop iteration.
                for _ in 0..self.options.max_loop_iterations {
                    self.backend.tick(*interval_ms);
                    self.execute_block(body)?;

                    // Take this path when self.
                    if self
                        .safety_monitor
                        .as_ref()
                        .map(|m| m.is_emergency_stop())
                        .unwrap_or(false)
                    {
                        break;
                    }
                }
            }
            Stmt::PublishStmt {
                topic_name,
                value,
                span,
            } => {
                let line = span.start.line;

                // Emit output when clone provides a agent.
                if let Some(agent) = self.current_agent.clone() {
                    self.check_agent_capability(&agent, "publish", Some(topic_name), line)?;
                }
                let topic = self.env.get(topic_name).cloned();
                let val = self.eval_expr(value)?;

                // Take this path when let Some(RuntimeValue::Topic.
                if let Some(RuntimeValue::Topic {
                    topic_path,
                    message_type,
                    ..
                }) = topic
                {
                    let payload = Self::runtime_value_payload(&val);
                    let source_id = self.publish_source_id();

                    if let Err(e) = self.security.prepare_publish(
                        &topic_path,
                        &payload,
                        &source_id,
                        &message_type,
                    ) {
                        if let Some(rt) = self.audit_runtime.as_mut() {
                            let _ = self.security.audit_security_event(
                                rt,
                                "publish_denied",
                                &format!("topic={topic_path} source={source_id} reason={e}"),
                            );
                        }
                        return Err(self.security_error(e, line));
                    }
                    if let Some(rt) = self.audit_runtime.as_mut() {
                        let _ = self.security.audit_security_event(
                            rt,
                            "encryption_enabled",
                            &format!("topic={topic_path}"),
                        );
                    }
                    self.comm_bus.publish(
                        &topic_path,
                        &message_type,
                        val.clone(),
                        self.default_transport,
                        Some(&source_id),
                    );
                    self.backend.publish_topic(&topic_path, &message_type, val);
                    self.topic_last_publish_ms
                        .insert(topic_path.clone(), self.sim_time_ms);
                    self.record_mission_event(
                        "topic_publish",
                        serde_json::json!({ "topic": topic_path }),
                    );

                    // Emit output when as mut provides a rt.
                    if let Some(rt) = self.audit_runtime.as_mut() {
                        let _ = self.security.audit_event(
                            rt,
                            "publish",
                            &format!("topic={topic_path}"),
                        );
                    }
                    self.log(format!("publish {topic_path}"));
                    let _ = self.dispatch_message_triggers(topic_name, &topic_path);
                }
            }
            Stmt::ServiceCallStmt {
                service_name, span, ..
            } => {
                let line = span.start.line;

                // Emit output when clone provides a agent.
                if let Some(agent) = self.current_agent.clone() {
                    self.check_agent_capability(&agent, "call", Some(service_name), line)?;
                }

                // Take this path when let Some(RuntimeValue::Service { name, service type }) =.
                if let Some(RuntimeValue::Service { name, service_type }) =
                    self.env.get(service_name).cloned()
                {
                    let endpoint = format!("/service/{name}");

                    // Handle the error returned from verify inbound.
                    if let Err(e) = self.security.verify_inbound(&endpoint, None, None) {
                        return Err(self.security_error(e, line));
                    }
                    self.backend.call_service(&name, &service_type);
                    self.log(format!("call {name}()"));
                }
            }
            Stmt::ActionSendStmt {
                action_name,
                goal,
                span,
                ..
            } => {
                let line = span.start.line;

                // Emit output when clone provides a agent.
                if let Some(agent) = self.current_agent.clone() {
                    self.check_agent_capability(&agent, "execute", Some(action_name), line)?;
                }

                // Take this path when let Some(RuntimeValue::Action { name, action type }) =.
                if let Some(RuntimeValue::Action { name, action_type }) =
                    self.env.get(action_name).cloned()
                {
                    let endpoint = format!("/action/{name}");
                    let goal_val = self.eval_expr(goal)?;
                    let payload = Self::runtime_value_payload(&goal_val);

                    // Handle the error returned from sign outbound.
                    if let Err(e) = self.security.sign_outbound(&endpoint, &payload) {
                        return Err(self.security_error(e, line));
                    }
                    self.comm_bus
                        .send_action(&name, &action_type, goal_val.clone());
                    self.backend.send_action(&name, &action_type, goal_val);
                    self.log(format!("send_goal {name}"));
                }
            }
            Stmt::SubscribeStmt { target, span, .. } => {
                let line = span.start.line;

                // Emit output when clone provides a agent.
                if let Some(agent) = self.current_agent.clone() {
                    self.check_agent_capability(&agent, "subscribe", Some(target), line)?;
                }
                let path = if target.contains('.') {
                    format!("/{}", target.replace('.', "/"))
                } else if let Some(RuntimeValue::Topic { topic_path, .. }) = self.env.get(target) {
                    topic_path.clone()
                } else {
                    format!("/{target}")
                };

                // Handle the error returned from verify inbound.
                if let Err(e) = self.security.authorize_subscribe(&path) {
                    return Err(self.security_error(e, line));
                }
                self.comm_bus.subscribe(&path, target);
                self.log(format!("subscribe {target}"));
            }
            Stmt::ExecuteStmt {
                action_name,
                goal,
                span,
                ..
            } => {
                let line = span.start.line;

                // Emit output when clone provides a agent.
                if let Some(agent) = self.current_agent.clone() {
                    self.check_agent_capability(&agent, "execute", Some(action_name), line)?;
                }

                // Take this path when let Some(RuntimeValue::Action { name, action type }) =.
                if let Some(RuntimeValue::Action { name, action_type }) =
                    self.env.get(action_name).cloned()
                {
                    let endpoint = format!("/action/{name}");
                    let goal_val = self.eval_expr(goal)?;
                    let payload = Self::runtime_value_payload(&goal_val);

                    // Handle the error returned from sign outbound.
                    if let Err(e) = self.security.sign_outbound(&endpoint, &payload) {
                        return Err(self.security_error(e, line));
                    }
                    self.comm_bus
                        .send_action(&name, &action_type, goal_val.clone());
                    self.backend.send_action(&name, &action_type, goal_val);
                    self.log(format!("execute {name}"));
                }
            }
            Stmt::DiscoverStmt {
                target,
                filter,
                span,
                ..
            } => {
                let line = span.start.line;

                // Emit output when clone provides a agent.
                if let Some(agent) = self.current_agent.clone() {
                    self.check_agent_capability(&agent, "discover", None, line)?;
                }
                let results = self.comm_bus.discover(
                    *target,
                    filter
                        .as_ref()
                        .unwrap_or(&DiscoverFilter { capability: None }),
                );
                self.log(format!("discover {:?}: {:?}", target, results));
                let _ = line;
            }
            Stmt::ReceiveStmt {
                topic_name,
                var_name,
                span,
                ..
            } => {
                let line = span.start.line;
                let path = if topic_name.contains('.') {
                    format!("/{}", topic_name.replace('.', "/"))
                } else if let Some(RuntimeValue::Topic { topic_path, .. }) =
                    self.env.get(topic_name)
                {
                    topic_path.clone()
                } else {
                    format!("/{topic_name}")
                };

                // Emit output when receive provides a val.
                if let Some(envelope) = self.comm_bus.receive_envelope(&path) {
                    let payload = Self::runtime_value_payload(&envelope.value);
                    if let Err(e) = self.security.verify_inbound_message(
                        &path,
                        &payload,
                        envelope.source_id.as_deref(),
                        None,
                        self.topic_path_to_message_type
                            .get(&path)
                            .map(String::as_str)
                            .unwrap_or("Unknown"),
                    ) {
                        return Err(self.security_error(e, line));
                    }
                    self.env.define(var_name.clone(), envelope.value);
                    self.log(format!("receive {topic_name} to {var_name}"));
                }
            }
            Stmt::EmergencyStopStmt { .. } => {
                // Emit output when safety monitor provides a monitor.
                if let Some(monitor) = &mut self.safety_monitor {
                    monitor.set_emergency_stop(true);
                }
                self.backend.set_emergency_stop(true);
                self.backend.execute_motion(MotionCommand::Stop {
                    actuator: "all".into(),
                });
                self.log("EMERGENCY STOP triggered".into());
                let _ =
                    self.dispatch_system_trigger(SystemTriggerCategory::Safety, "EmergencyStop");
            }
            Stmt::ResetEmergencyStopStmt { .. } => {
                // Emit output when safety monitor provides a monitor.
                if let Some(monitor) = &mut self.safety_monitor {
                    monitor.reset();
                }
                self.backend.set_emergency_stop(false);
                self.log("Emergency stop reset".into());
            }
            Stmt::EmitStmt { event_name, .. } => {
                self.dispatch_event(event_name)?;
            }
            Stmt::EnterStmt { state_name, span } => {
                self.execute_enter(state_name, span.start.line)?;
            }
            Stmt::RememberStmt { key, value, span } => {
                let stored = self.eval_expr(value)?;
                let agent_name = self.current_agent.clone().ok_or_else(|| {
                    RuntimeError::new(
                        "remember requires active agent context (run inside agent plan)",
                        span.start.line,
                    )
                    .into_spanda()
                })?;
                let agent = self.agents.get_mut(&agent_name).ok_or_else(|| {
                    RuntimeError::new(format!("Unknown agent '{agent_name}'"), span.start.line)
                        .into_spanda()
                })?;
                let memory = agent.memory.as_mut().ok_or_else(|| {
                    RuntimeError::new(
                        "Agent has no memory — declare memory short_term or long_term on the agent",
                        span.start.line,
                    )
                    .into_spanda()
                })?;
                memory.remember(key.clone(), stored);
                self.log(format!("remember '{key}'"));
            }
            Stmt::ExprStmt { expr, .. } => {
                self.eval_expr(expr)?;
            }
            Stmt::SpawnStmt { callee, args, span } => {
                let (name, arg_values) = self.eval_spawn_target(callee, args, span.start.line)?;
                self.telemetry.record_fire_and_forget_spawn();
                self.trace_task_log(format!("spawn fire-and-forget {name}"));
                self.concurrency.queue_fire_and_forget(name, arg_values);
            }
            Stmt::SelectStmt { arms, span } => {
                'select: for arm in arms {
                    let channel_val = self.eval_expr(&arm.channel)?;

                    // Emit output when line)? provides a msg.
                    if let Some(msg) = self.concurrency.try_recv(&channel_val, span.start.line)? {
                        self.env.define("_msg", msg);
                        self.execute_block(&arm.body)?;
                        break 'select;
                    }
                }
            }
            Stmt::ParallelStmt { body, span } => {
                self.telemetry.record_parallel_block();
                self.trace_task_log(format!("parallel block {} branch(es)", body.len()));
                let saved = self.env.clone_bindings();
                let mut pending_handles: Vec<(Option<String>, u64)> = Vec::new();
                let mut results = HashMap::new();

                self.log(format!(
                    "parallel: executing {} branch(es) cooperatively",
                    body.len()
                ));

                for stmt in body {
                    self.env = saved.clone_bindings();
                    match stmt {
                        Stmt::VarDecl {
                            name,
                            init: Some(init),
                            ..
                        } => {
                            let val = self.eval_expr(init)?;
                            if let RuntimeValue::TaskHandle { id } = val {
                                pending_handles.push((Some(name.clone()), id));
                            } else {
                                results.insert(name.clone(), val);
                            }
                        }
                        Stmt::ExprStmt { expr, .. } => {
                            let val = self.eval_expr(expr)?;
                            if let RuntimeValue::TaskHandle { id } = val {
                                pending_handles.push((None, id));
                            }
                        }
                        Stmt::SpawnStmt { callee, args, .. } => {
                            let (func_name, arg_values) =
                                self.eval_spawn_target(callee, args, span.start.line)?;
                            let handle = self.concurrency.create_task_handle(func_name, arg_values);
                            if let RuntimeValue::TaskHandle { id } = handle {
                                pending_handles.push((None, id));
                            }
                        }
                        _ => self.execute_stmt(stmt)?,
                    }
                }

                self.env = saved;

                for (name, id) in pending_handles {
                    let result = self.resolve_task_handle(id, span.start.line)?;
                    if let Some(binding) = name {
                        results.insert(binding, result);
                    }
                }

                if !results.is_empty() {
                    let count = results.len();
                    self.env.define(
                        "_parallel",
                        RuntimeValue::object("ParallelResults", results),
                    );
                    self.log(format!("parallel: aggregated {count} result(s)"));
                }
            }
            Stmt::ReturnStmt { .. } => {}
            Stmt::EnterModeStmt { mode, .. } => {
                self.enter_mode(mode)?;
            }
            Stmt::UseFallbackStmt { resource, .. } => {
                self.log(format!("fault: using fallback resource '{resource}'"));
            }
            Stmt::StopAllActuatorsStmt { .. } => {
                if let Some(monitor) = &mut self.safety_monitor {
                    monitor.set_emergency_stop(true);
                }
                self.backend.set_emergency_stop(true);
                self.backend.execute_motion(MotionCommand::Stop {
                    actuator: "all".into(),
                });
                self.log("safety: stop_all_actuators invoked".into());
            }
            Stmt::RunPipelineStmt { name, .. } => {
                self.execute_pipeline(name)?;
            }
            Stmt::NavigateStmt {
                goal,
                linear,
                angular,
                span,
            } => {
                self.execute_navigate_stmt(goal, linear.as_deref(), angular.as_deref(), span.start.line)?;
            }
        }
        Ok(())
    }

    fn execute_block_with_return(
        &mut self,
        stmts: &[Stmt],
    ) -> Result<Option<RuntimeValue>, SpandaError> {
        // Execute block with return.
        //
        // Parameters:
        // - `self` — method receiver
        // - `stmts` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_block_with_return(stmts);

        // Execute each statement in sequence.
        for stmt in stmts {
            // Emit output when execute stmt with return provides a val.
            if let Some(val) = self.execute_stmt_with_return(stmt)? {
                return Ok(Some(val));
            }
        }
        Ok(None)
    }

    fn execute_stmt_with_return(
        &mut self,
        stmt: &Stmt,
    ) -> Result<Option<RuntimeValue>, SpandaError> {
        // Execute stmt with return.
        //
        // Parameters:
        // - `self` — method receiver
        // - `stmt` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_stmt_with_return(stmt);

        // Match on stmt and handle each case.
        match stmt {
            Stmt::ReturnStmt { value, .. } => {
                let val = if let Some(expr) = value {
                    self.eval_expr(expr)?
                } else {
                    RuntimeValue::Void
                };
                Ok(Some(val))
            }
            Stmt::IfStmt {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let cond = self.eval_expr(condition)?;

                // Keep entries that match the expected pattern.
                if matches!(cond, RuntimeValue::Bool { value: true, .. }) {
                    // Emit output when execute block with return provides a v.
                    if let Some(v) = self.execute_block_with_return(then_branch)? {
                        return Ok(Some(v));
                    }
                } else if let Some(else_branch) = else_branch {
                    // Emit output when execute block with return provides a v.
                    if let Some(v) = self.execute_block_with_return(else_branch)? {
                        return Ok(Some(v));
                    }
                }
                Ok(None)
            }
            _ => {
                self.execute_stmt(stmt)?;
                Ok(None)
            }
        }
    }

    fn call_module_function(
        &mut self,
        func: &crate::foundations::ModuleFnDecl,
        args: &[Expr],
        _line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Call module function.
        //
        // Parameters:
        // - `self` — method receiver
        // - `func` — input value
        // - `args` — input value
        // - `_line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.call_module_function(func, args, _line);

        // Save current variable bindings before the call.
        let saved = self.env.clone_bindings();

        // Bind each formal parameter to its call argument.
        for (i, param) in func.params.iter().enumerate() {
            // Emit output when get provides a arg.
            if let Some(arg) = args.get(i) {
                let val = self.eval_expr(arg)?;
                self.env.define(param.name.clone(), val);
            }
        }
        let result = self
            .execute_block_with_return(&func.body)?
            .unwrap_or(RuntimeValue::Void);
        self.env = saved;
        Ok(result)
    }

    fn resolve_future(
        &mut self,
        future: RuntimeValue,
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Resolve future.
        //
        // Parameters:
        // - `self` — method receiver
        // - `future` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.resolve_future(future, line);

        // Match on future and handle each case.
        match future {
            RuntimeValue::Future {
                resolved: Some(value),
                ..
            } => Ok(*value),
            RuntimeValue::Future {
                func_name,
                args,
                resolved: None,
                ..
            } => {
                let func = self
                    .module_functions
                    .get(&func_name)
                    .or_else(|| self.imported_functions.get(&func_name))
                    .cloned()
                    .ok_or_else(|| {
                        RuntimeError::new(format!("Unknown async function '{func_name}'"), line)
                            .into_spanda()
                    })?;
                let saved = self.env.clone_bindings();

                // Bind each formal parameter to its call argument.
                for (i, param) in func.params.iter().enumerate() {
                    // Emit output when get provides a val.
                    if let Some(val) = args.get(i) {
                        self.env.define(param.name.clone(), val.clone());
                    }
                }
                let result = self
                    .execute_block_with_return(&func.body)?
                    .unwrap_or(RuntimeValue::Void);
                self.env = saved;
                Ok(result)
            }
            other => Ok(other),
        }
    }

    fn eval_spawn_target(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        line: u32,
    ) -> Result<(String, Vec<RuntimeValue>), SpandaError> {
        // Eval spawn target.
        //
        // Parameters:
        // - `self` — method receiver
        // - `callee` — input value
        // - `args` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.eval_spawn_target(callee, args, line);

        // Create mutable arg values for accumulating results.
        let mut arg_values = Vec::new();

        // Apply each command-line argument.
        for arg in args {
            arg_values.push(self.eval_expr(arg)?);
        }
        let name = match callee {
            Expr::IdentExpr { name, .. } => name.clone(),
            _ => return Err(RuntimeError::new("spawn requires function name", line).into_spanda()),
        };
        Ok((name, arg_values))
    }

    fn execute_spawn_job(
        &mut self,
        func_name: &str,
        args: &[RuntimeValue],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Execute spawn job.
        //
        // Parameters:
        // - `self` — method receiver
        // - `func_name` — input value
        // - `args` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_spawn_job(func_name, args, line);

        // Compute func for the following logic.
        let func = self
            .module_functions
            .get(func_name)
            .or_else(|| self.imported_functions.get(func_name))
            .cloned()
            .ok_or_else(|| {
                RuntimeError::new(format!("Unknown spawn target '{func_name}'"), line).into_spanda()
            })?;
        let saved = self.env.clone_bindings();

        // Bind each formal parameter to its call argument.
        for (i, param) in func.params.iter().enumerate() {
            // Emit output when get provides a val.
            if let Some(val) = args.get(i) {
                self.env.define(param.name.clone(), val.clone());
            }
        }
        let result = self
            .execute_block_with_return(&func.body)?
            .unwrap_or(RuntimeValue::Void);
        self.env = saved;
        Ok(result)
    }

    fn resolve_task_handle(&mut self, id: u64, line: u32) -> Result<RuntimeValue, SpandaError> {
        // Resolve task handle.
        //
        // Parameters:
        // - `self` — method receiver
        // - `id` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.resolve_task_handle(id, line);

        // use result when clone is present.

        // Emit output when clone provides a result.
        if let Some(result) = self.concurrency.handle(id).and_then(|h| h.result.clone()) {
            return Ok(result);
        }
        let result = self.execute_spawn_handle(id, line)?;
        self.telemetry.record_join();
        self.trace_task_log(format!("join handle {id} -> completed"));
        Ok(result)
    }

    fn execute_spawn_handle(&mut self, id: u64, line: u32) -> Result<RuntimeValue, SpandaError> {
        // Execute spawn handle.
        //
        // Parameters:
        // - `self` — method receiver
        // - `id` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_spawn_handle(id, line);

        // use result when clone is present.

        // Emit output when clone provides a result.
        if let Some(result) = self.concurrency.handle(id).and_then(|h| h.result.clone()) {
            return Ok(result);
        }
        let (func_name, args) = {
            let handle = self.concurrency.handle(id).ok_or_else(|| {
                RuntimeError::new(format!("Unknown task handle id {id}"), line).into_spanda()
            })?;
            (handle.func_name.clone(), handle.args.clone())
        };
        let result = self.execute_spawn_job(&func_name, &args, line)?;
        self.concurrency.set_handle_result(id, result.clone());
        Ok(result)
    }

    fn process_spawn_queue(&mut self) -> Result<(), SpandaError> {
        // Process spawn queue.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.process_spawn_queue();

        // Compute ids for the following logic.
        let ids = self.concurrency.drain_fire_and_forget_queue();

        // Iterate over ids.
        for id in ids {
            self.execute_spawn_handle(id, 0)?;
        }
        Ok(())
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<RuntimeValue, SpandaError> {
        // Eval expr.
        //
        // Parameters:
        // - `self` — method receiver
        // - `expr` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.eval_expr(expr);

        // Match on expr and handle each case.
        match expr {
            Expr::LiteralExpr { value, .. } => Ok(match value {
                LiteralValue::Bool(b) => RuntimeValue::Bool { value: *b },
                LiteralValue::Number(n) => RuntimeValue::Number {
                    value: *n,
                    unit: UnitKind::None,
                },
                LiteralValue::String(s) => RuntimeValue::String { value: s.clone() },
                LiteralValue::Null => RuntimeValue::Void,
                LiteralValue::Regex(pattern) => RuntimeValue::Regex {
                    pattern: pattern.clone(),
                },
            }),
            Expr::UnitLiteralExpr { value, unit, .. } => Ok(RuntimeValue::Number {
                value: *value,
                unit: *unit,
            }),
            Expr::IdentExpr { name, span } => {
                // Emit output when get provides a enum name.
                if let Some(enum_name) = self.variant_owner.get(name) {
                    return Ok(RuntimeValue::Enum {
                        enum_name: enum_name.clone(),
                        variant: name.clone(),
                        payloads: Vec::new(),
                    });
                }
                self.env.get(name).cloned().ok_or_else(|| {
                    RuntimeError::new(format!("Undefined variable '{name}'"), span.start.line)
                        .into_spanda()
                })
            }
            Expr::BinaryExpr {
                op,
                left,
                right,
                span,
            } => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;
                self.eval_binary(*op, left_val, right_val, span.start.line)
            }
            Expr::UnaryExpr {
                op,
                operand,
                span: _,
            } => {
                let operand_val = self.eval_expr(operand)?;

                // Match on op and handle each case.
                match op {
                    UnaryOp::Not => Ok(RuntimeValue::Bool {
                        value: matches!(operand_val, RuntimeValue::Bool { value, .. } if !value),
                    }),
                    UnaryOp::Neg => {
                        // Take this path when let RuntimeValue::Number { value, unit } = operand val.
                        if let RuntimeValue::Number { value, unit } = operand_val {
                            Ok(RuntimeValue::Number {
                                value: -value,
                                unit,
                            })
                        } else {
                            Ok(RuntimeValue::Void)
                        }
                    }
                }
            }
            Expr::MemberExpr {
                object,
                property,
                span: _,
            } => {
                // Take this path when let Expr::IdentExpr { name, .. } = object.as ref().
                if let Expr::IdentExpr { name, .. } = object.as_ref() {
                    // Emit output when get provides a variants.
                    if let Some(variants) = self.enum_variants.get(name) {
                        // Take the branch when any equals property).
                        if variants.iter().any(|v| v == property) {
                            return Ok(RuntimeValue::Enum {
                                enum_name: name.clone(),
                                variant: property.clone(),
                                payloads: Vec::new(),
                            });
                        }
                    }
                }
                let obj = self.eval_expr(object)?;
                self.eval_member(&obj, property)
            }
            Expr::CallExpr {
                callee,
                args,
                named_args,
                span,
            } => self.eval_call(callee, args, named_args, span.start.line),
            Expr::AwaitExpr { operand, span } => {
                let value = self.eval_expr(operand)?;
                self.resolve_future(value, span.start.line)
            }
            Expr::SpawnExpr { callee, args, span } => {
                let (name, arg_values) = self.eval_spawn_target(callee, args, span.start.line)?;
                self.telemetry.record_spawn();
                self.trace_task_log(format!("spawn handle {name}"));
                Ok(self.concurrency.create_task_handle(name, arg_values))
            }
            Expr::MatchExpr {
                scrutinee, arms, ..
            } => {
                let value = self.eval_expr(scrutinee)?;
                let variant = match &value {
                    RuntimeValue::Enum { variant, .. } => variant.clone(),
                    RuntimeValue::Result { ok: true, .. } => "Ok".into(),
                    RuntimeValue::Result { ok: false, .. } => "Err".into(),
                    RuntimeValue::Option { present: true, .. } => "Some".into(),
                    RuntimeValue::Option { present: false, .. } => "None".into(),
                    RuntimeValue::String { value } => value.clone(),
                    RuntimeValue::Object { type_name, .. } => type_name.clone(),
                    _ => String::new(),
                };

                // Process each arm.
                for arm in arms {
                    // Take the branch when variant equals variant.
                    if arm.variant == variant {
                        // Skip further work when bindings is empty.
                        if !arm.bindings.is_empty() {
                            // Take this path when let RuntimeValue::Enum { payloads, .. } = &value.
                            if let RuntimeValue::Enum { payloads, .. } = &value {
                                // Iterate over iter with destructured elements.
                                for (binding, payload) in arm.bindings.iter().zip(payloads.iter()) {
                                    self.env.set(binding.clone(), payload.clone());
                                }
                            }
                        }

                        // Execute each statement in sequence.
                        for stmt in &arm.body {
                            self.execute_stmt(stmt)?;
                        }

                        // Process each binding.
                        for binding in &arm.bindings {
                            self.env.remove(binding);
                        }
                        break;
                    }
                }
                Ok(RuntimeValue::Void)
            }
            Expr::StructLiteralExpr {
                type_name,
                fields,
                span,
            } => self.eval_struct_literal(type_name, fields, span.start.line),
            Expr::ServiceCallExpr { service_name, .. } => {
                // Take this path when let Some(RuntimeValue::Service { name, service type }) =.
                if let Some(RuntimeValue::Service { name, service_type }) =
                    self.env.get(service_name).cloned()
                {
                    let result = self.comm_bus.call_service(&name, &service_type, None);
                    self.backend.call_service(&name, &service_type);
                    self.log(format!("call {name}()"));
                    Ok(result)
                } else {
                    Ok(RuntimeValue::Void)
                }
            }
            Expr::ExecuteExpr {
                action_name, goal, ..
            } => {
                // Take this path when let Some(RuntimeValue::Action { name, action type }) =.
                if let Some(RuntimeValue::Action { name, action_type }) =
                    self.env.get(action_name).cloned()
                {
                    let goal_val = self.eval_expr(goal)?;
                    let result = self
                        .comm_bus
                        .send_action(&name, &action_type, goal_val.clone());
                    self.backend.send_action(&name, &action_type, goal_val);
                    self.log(format!("execute {name}"));
                    Ok(result)
                } else {
                    Ok(RuntimeValue::Void)
                }
            }
            Expr::DiscoverExpr { target, filter, .. } => {
                let results = self.comm_bus.discover(
                    *target,
                    filter
                        .as_ref()
                        .unwrap_or(&DiscoverFilter { capability: None }),
                );
                Ok(RuntimeValue::Object {
                    type_name: "DiscoveryResult".into(),
                    fields: HashMap::from([(
                        "count".into(),
                        RuntimeValue::Number {
                            value: results.len() as f64,
                            unit: UnitKind::None,
                        },
                    )]),
                })
            }
        }
    }

    fn eval_struct_literal(
        &mut self,
        type_name: &str,
        fields: &[crate::ast::StructFieldInit],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Eval struct literal.
        //
        // Parameters:
        // - `self` — method receiver
        // - `type_name` — input value
        // - `fields` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.eval_struct_literal(type_name, fields, line);

        // Create mutable values for accumulating results.
        let mut values = HashMap::new();

        // Check each struct field.
        for field in fields {
            values.insert(field.name.clone(), self.eval_expr(&field.value)?);
        }

        // Take the branch when type name equals "Pose".
        if type_name == "Pose" {
            let x = values
                .get("x")
                .and_then(|v| v.as_number())
                .ok_or_else(|| RuntimeError::new("Pose.x must be numeric", line).into_spanda())?;
            let y = values
                .get("y")
                .and_then(|v| v.as_number())
                .ok_or_else(|| RuntimeError::new("Pose.y must be numeric", line).into_spanda())?;
            let heading = values
                .get("heading")
                .or_else(|| values.get("theta"))
                .and_then(|v| v.as_number())
                .unwrap_or(0.0);
            let z = values.get("z").and_then(|v| v.as_number()).unwrap_or(0.0);
            return Ok(RuntimeValue::Pose {
                x,
                y,
                theta: heading,
                z,
            });
        }
        Ok(RuntimeValue::Object {
            type_name: type_name.to_string(),
            fields: values,
        })
    }

    fn eval_member(
        &mut self,
        obj: &RuntimeValue,
        property: &str,
    ) -> Result<RuntimeValue, SpandaError> {
        // Eval member.
        //
        // Parameters:
        // - `self` — method receiver
        // - `obj` — input value
        // - `property` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.eval_member(obj, property);

        // Match on obj and handle each case.
        match obj {
            RuntimeValue::Scan { nearest_distance } if property == "nearest_distance" => {
                Ok(RuntimeValue::Number {
                    value: *nearest_distance,
                    unit: UnitKind::M,
                })
            }
            RuntimeValue::Pose { x, y, theta, z } => match property {
                "x" => Ok(RuntimeValue::Number {
                    value: *x,
                    unit: UnitKind::M,
                }),
                "y" => Ok(RuntimeValue::Number {
                    value: *y,
                    unit: UnitKind::M,
                }),
                "theta" => Ok(RuntimeValue::Number {
                    value: *theta,
                    unit: UnitKind::Rad,
                }),
                "z" => Ok(RuntimeValue::Number {
                    value: *z,
                    unit: UnitKind::M,
                }),
                _ => Ok(RuntimeValue::Void),
            },
            RuntimeValue::Velocity { linear, angular } => match property {
                "linear" => Ok(RuntimeValue::Number {
                    value: *linear,
                    unit: UnitKind::MPerS,
                }),
                "angular" => Ok(RuntimeValue::Number {
                    value: *angular,
                    unit: UnitKind::RadPerS,
                }),
                _ => Ok(RuntimeValue::Void),
            },
            RuntimeValue::Sensor { .. } if property == "nearest_distance" => {
                // Take this path when let RuntimeValue::Scan { nearest distance } = self.read sensor value(o.
                if let RuntimeValue::Scan { nearest_distance } = self.read_sensor_value(obj)? {
                    Ok(RuntimeValue::Number {
                        value: nearest_distance,
                        unit: UnitKind::M,
                    })
                } else {
                    Ok(RuntimeValue::Void)
                }
            }
            RuntimeValue::ActionProposal {
                linear,
                angular,
                source,
                trace,
            } => match property {
                "linear" => Ok(RuntimeValue::Number {
                    value: *linear,
                    unit: UnitKind::MPerS,
                }),
                "angular" => Ok(RuntimeValue::Number {
                    value: *angular,
                    unit: UnitKind::RadPerS,
                }),
                "trace" => {
                    let mut fields = HashMap::new();
                    fields.insert("source".to_string(), RuntimeValue::string(source.clone()));
                    fields.insert("steps".to_string(), RuntimeValue::string(trace.join("\n")));
                    fields.insert(
                        "step_count".to_string(),
                        RuntimeValue::Number {
                            value: trace.len() as f64,
                            unit: UnitKind::None,
                        },
                    );
                    Ok(RuntimeValue::object("ReasoningTrace", fields))
                }
                _ => Ok(RuntimeValue::Void),
            },
            RuntimeValue::SafeAction { linear, angular } => match property {
                "linear" => Ok(RuntimeValue::Number {
                    value: *linear,
                    unit: UnitKind::MPerS,
                }),
                "angular" => Ok(RuntimeValue::Number {
                    value: *angular,
                    unit: UnitKind::RadPerS,
                }),
                _ => Ok(RuntimeValue::Void),
            },
            RuntimeValue::Goal { text } if property == "text" => {
                Ok(RuntimeValue::string(text.clone()))
            }
            RuntimeValue::Agent { name } if property == "goal" => {
                let text = self
                    .agents
                    .get(name)
                    .map(|agent| match &agent.decl {
                        AgentDecl::AgentDecl { goal, .. } => goal.clone(),
                    })
                    .unwrap_or_default();
                Ok(RuntimeValue::Goal { text })
            }
            RuntimeValue::Completion { text, .. } if property == "text" => {
                Ok(RuntimeValue::String {
                    value: text.clone(),
                })
            }
            RuntimeValue::Object { fields, .. } => {
                Ok(fields.get(property).cloned().unwrap_or(RuntimeValue::Void))
            }
            _ => Ok(RuntimeValue::Void),
        }
    }

    fn eval_string_regex_method(
        &mut self,
        method: &str,
        text: &str,
        args: &[Expr],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Evaluate string regex helper methods: matches, find, replace, split, capture.
        let pattern_val = args.first().ok_or_else(|| {
            RuntimeError::new("Regex method requires pattern argument", line).into_spanda()
        })?;
        let pattern = match self.eval_expr(pattern_val)? {
            RuntimeValue::Regex { pattern } => pattern,
            _ => {
                return Err(
                    RuntimeError::new("Regex method requires Regex pattern argument", line)
                        .into_spanda(),
                )
            }
        };
        match method {
            "matches" => Ok(RuntimeValue::Bool {
                value: crate::regex_lang::regex_matches(&pattern, text)?,
            }),
            "find" => Ok(match crate::regex_lang::regex_find(&pattern, text)? {
                Some(found) => RuntimeValue::String { value: found },
                None => RuntimeValue::Null,
            }),
            "replace" => {
                let replacement = args.get(1).ok_or_else(|| {
                    RuntimeError::new("replace requires replacement argument", line).into_spanda()
                })?;
                let replacement = match self.eval_expr(replacement)? {
                    RuntimeValue::String { value } => value,
                    _ => {
                        return Err(
                            RuntimeError::new("replace replacement must be string", line)
                                .into_spanda(),
                        )
                    }
                };
                Ok(RuntimeValue::String {
                    value: crate::regex_lang::regex_replace(&pattern, text, &replacement)?,
                })
            }
            "split" => {
                let parts = crate::regex_lang::regex_split(&pattern, text)?;
                Ok(RuntimeValue::Object {
                    type_name: "StringList".into(),
                    fields: parts
                        .into_iter()
                        .enumerate()
                        .map(|(i, part)| (i.to_string(), RuntimeValue::String { value: part }))
                        .collect(),
                })
            }
            "capture" => Ok(match crate::regex_lang::regex_capture(&pattern, text)? {
                Some(result) => RuntimeValue::Capture { result },
                None => RuntimeValue::Null,
            }),
            _ => Ok(RuntimeValue::Void),
        }
    }

    fn eval_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        named_args: &[crate::ast::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Eval call.
        //
        // Parameters:
        // - `self` — method receiver
        // - `callee` — input value
        // - `args` — input value
        // - `named_args` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.eval_call(callee, args, named_args, line);

        if let Expr::IdentExpr { name, .. } = callee {
            if let Some(ext) = self.extern_functions.get(name).cloned() {
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.eval_expr(arg)?);
                }
                return self.options.ffi_registry.call(&ext, &arg_values);
            }
            if let Some(func) = self
                .module_functions
                .get(name)
                .or_else(|| self.imported_functions.get(name))
                .cloned()
            {
                if func.is_async {
                    let mut arg_values = Vec::new();
                    for arg in args {
                        arg_values.push(self.eval_expr(arg)?);
                    }
                    return Ok(RuntimeValue::Future {
                        func_name: func.name.clone(),
                        args: arg_values,
                        resolved: None,
                    });
                }
                return self.call_module_function(&func, args, line);
            }
            if let Some(enum_name) = self.variant_owner.get(name).cloned() {
                let mut payloads = Vec::new();
                for arg in args {
                    payloads.push(self.eval_expr(arg)?);
                }
                return Ok(RuntimeValue::Enum {
                    enum_name,
                    variant: name.clone(),
                    payloads,
                });
            }
            return self.eval_builtin_function(name, args, named_args, line);
        }

        let Expr::MemberExpr {
            object, property, ..
        } = callee
        else {
            return Ok(RuntimeValue::Void);
        };

        // Handle string regex methods on arbitrary receiver expressions.
        if let Ok(RuntimeValue::String { value: text }) = self.eval_expr(object) {
            return self.eval_string_regex_method(property, &text, args, line);
        }

        let Expr::IdentExpr {
            name: target_name, ..
        } = object.as_ref()
        else {
            return Ok(RuntimeValue::Void);
        };

        let target = self.env.get(target_name).cloned().ok_or_else(|| {
            RuntimeError::new(format!("Undefined '{target_name}'"), line).into_spanda()
        })?;

        if matches!(target, RuntimeValue::Robot) || target_name == "robot" {
            return self.eval_robot_method(property, args, named_args);
        }

        if matches!(target, RuntimeValue::Twin { .. }) {
            return self.eval_twin_method(property, args, named_args, line);
        }

        if matches!(target, RuntimeValue::SensorFusion { .. }) && property == "read" {
            return self.read_fused_observation();
        }

        if let RuntimeValue::MissionControl { mut runtime } = target {
            let result = self.eval_mission_method(&mut runtime, property, line)?;
            self.env.define(
                target_name.clone(),
                RuntimeValue::MissionControl { runtime },
            );
            return Ok(result);
        }

        if let RuntimeValue::NavigationControl { mut goal } = target {
            let result =
                self.eval_navigation_method(&mut goal, property, args, named_args, line)?;
            self.env.define(
                target_name.clone(),
                RuntimeValue::NavigationControl { goal },
            );
            return Ok(result);
        }

        if matches!(target, RuntimeValue::SlamControl) {
            return self.eval_slam_method(property, line);
        }

        if let RuntimeValue::FleetControl { registry } = target {
            return self.eval_fleet_method(&registry, property, args, line);
        }

        if let RuntimeValue::Sensor { sensor_type, .. } = &target {
            if property == "read" {
                if let Some(agent) = self.current_agent.as_deref() {
                    self.check_agent_capability(agent, "read", Some(target_name), line)?;
                }
                return self.read_sensor_value(&target);
            }
            if sensor_type == "Camera" {
                if property == "frame" {
                    return Ok(mock_camera_frame());
                }
                if property == "analyze" {
                    let frame = mock_camera_frame();
                    return Ok(mock_analyze_frame(Some(&frame), target_name));
                }
            }
        }

        let target = match target {
            RuntimeValue::TraitObject { agent, .. } => RuntimeValue::Agent { name: agent },
            other => other,
        };

        if let RuntimeValue::Agent { name } = &target {
            if let Some((params, body)) = self
                .agent_trait_impls
                .get(name)
                .and_then(|methods| methods.get(property))
                .cloned()
            {
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.eval_expr(arg)?);
                }
                let saved = self.env.clone();
                for (param, value) in params.iter().zip(arg_values) {
                    self.env.define(param.name.clone(), value);
                }
                self.current_agent = Some(name.clone());
                self.execute_block(&body)?;
                self.current_agent = None;
                self.env = saved;
                self.log(format!("agent {name}.{property}()"));
                return Ok(RuntimeValue::Void);
            }
            if property == "plan" {
                self.check_agent_capability(name, "plan", None, line)?;
                let agent = self.agents.get(name).ok_or_else(|| {
                    RuntimeError::new(format!("Unknown agent '{name}'"), line).into_spanda()
                })?;
                let agent = agent.clone();
                struct PlanRunner<'a, B: RobotBackend> {
                    interp: &'a mut Interpreter<B>,
                    agent_name: String,
                }
                impl<B: RobotBackend> PlanExecutor for PlanRunner<'_, B> {
                    fn execute_block(&mut self, stmts: &[Stmt]) -> Result<(), SpandaError> {
                        // Execute block.
                        //
                        // Parameters:
                        // - `self` — method receiver
                        // - `stmts` — input value
                        //
                        // Returns:
                        // Success value on completion, or an error.
                        //
                        // Options:
                        // None.
                        //
                        // Example:
                        // let result = instance.execute_block(stmts);

                        // Call current agent = Some on the current instance.
                        self.interp.current_agent = Some(self.agent_name.clone());
                        let result = self.interp.execute_block(stmts);
                        self.interp.current_agent = None;
                        result
                    }
                }
                let mut runner = PlanRunner {
                    interp: self,
                    agent_name: name.clone(),
                };
                execute_agent_plan(&agent, &mut runner)?;
                let _ = self.dispatch_system_trigger(SystemTriggerCategory::Ai, "GoalCompleted");
                self.log(format!("agent {name}.plan()"));
                return Ok(RuntimeValue::Void);
            }
        }

        if matches!(target, RuntimeValue::SafetyCtx) && property == "validate" {
            return self.eval_safety_validate(args, named_args, line);
        }

        if matches!(target, RuntimeValue::AuditCtx) {
            return self.eval_audit_method(property, args, named_args, line);
        }

        if matches!(target, RuntimeValue::LedgerCtx) {
            return self.eval_ledger_method(property, args, named_args, line);
        }

        if self.ai_models.contains_key(target_name)
            || matches!(target, RuntimeValue::AiModel { .. })
        {
            return self.eval_ai_method(target_name, property, args, named_args, line);
        }

        if let RuntimeValue::Actuator {
            name,
            actuator_type,
        } = target
        {
            return self.execute_actuator_method(
                &name,
                &actuator_type,
                property,
                args,
                named_args,
                line,
            );
        }

        Ok(RuntimeValue::Void)
    }

    fn get_named_arg_value(
        &mut self,
        named_args: &[crate::ast::NamedArg],
        name: &str,
    ) -> Result<RuntimeValue, SpandaError> {
        //
        // Parameters:
        // - `self` — method receiver
        // - `named_args` — input value
        // - `name` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.get_named_arg_value(named_args, name);

        // Apply each command-line argument.
        for arg in named_args {
            // Take the branch when name equals name.
            if arg.name == name {
                return self.eval_expr(&arg.value);
            }
        }
        Ok(RuntimeValue::Void)
    }

    fn eval_binary(
        &self,
        op: BinaryOp,
        left: RuntimeValue,
        right: RuntimeValue,
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Eval binary.
        //
        // Parameters:
        // - `self` — method receiver
        // - `op` — input value
        // - `left` — input value
        // - `right` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.eval_binary(op, left, right, line);

        // Match on op and handle each case.
        match op {
            BinaryOp::And => Ok(RuntimeValue::Bool {
                value: matches!(left, RuntimeValue::Bool { value: true, .. })
                    && matches!(right, RuntimeValue::Bool { value: true, .. }),
            }),
            BinaryOp::Or => Ok(RuntimeValue::Bool {
                value: matches!(left, RuntimeValue::Bool { value: true, .. })
                    || matches!(right, RuntimeValue::Bool { value: true, .. }),
            }),
            _ => {
                // Keep entries that match the expected pattern.
                if matches!(op, BinaryOp::Eq | BinaryOp::Neq)
                    && matches!(left, RuntimeValue::Enum { .. })
                    && matches!(right, RuntimeValue::Enum { .. })
                {
                    let RuntimeValue::Enum {
                        enum_name: e1,
                        variant: v1,
                        payloads: p1,
                    } = left
                    // Handle any remaining cases.
                    else {
                        unreachable!()
                    };
                    let RuntimeValue::Enum {
                        enum_name: e2,
                        variant: v2,
                        payloads: p2,
                    } = right
                    // Handle any remaining cases.
                    else {
                        unreachable!()
                    };
                    let equal = e1 == e2 && v1 == v2 && p1 == p2;
                    return Ok(RuntimeValue::Bool {
                        value: if op == BinaryOp::Eq { equal } else { !equal },
                    });
                }

                // Keep entries that match the expected pattern.
                if matches!(op, BinaryOp::Eq | BinaryOp::Neq)
                    && matches!(left, RuntimeValue::Bool { .. })
                    && matches!(right, RuntimeValue::Bool { .. })
                {
                    let RuntimeValue::Bool { value: l, .. } = left else {
                        unreachable!()
                    };
                    let RuntimeValue::Bool { value: r, .. } = right else {
                        unreachable!()
                    };
                    return Ok(RuntimeValue::Bool {
                        value: if op == BinaryOp::Eq { l == r } else { l != r },
                    });
                }

                // Take this path when let (.
                if let (
                    RuntimeValue::Number { value: l, unit: lu },
                    RuntimeValue::Number { value: r, unit: ru },
                ) = (left, right)
                {
                    let (l, r, result_unit) = align_for_binary(l, lu, r, ru).unwrap_or((l, r, lu));
                    return Ok(match op {
                        BinaryOp::Add => RuntimeValue::Number {
                            value: l + r,
                            unit: result_unit,
                        },
                        BinaryOp::Sub => RuntimeValue::Number {
                            value: l - r,
                            unit: result_unit,
                        },
                        BinaryOp::Mul => RuntimeValue::Number {
                            value: l * r,
                            unit: UnitKind::None,
                        },
                        BinaryOp::Div => RuntimeValue::Number {
                            value: l / r,
                            unit: UnitKind::None,
                        },
                        BinaryOp::Lt => RuntimeValue::Bool { value: l < r },
                        BinaryOp::Lte => RuntimeValue::Bool { value: l <= r },
                        BinaryOp::Gt => RuntimeValue::Bool { value: l > r },
                        BinaryOp::Gte => RuntimeValue::Bool { value: l >= r },
                        BinaryOp::Eq => RuntimeValue::Bool { value: l == r },
                        BinaryOp::Neq => RuntimeValue::Bool { value: l != r },
                        _ => RuntimeValue::Void,
                    });
                }
                let _ = line;
                Ok(RuntimeValue::Void)
            }
        }
    }

    fn log(&self, message: String) {
        // Log.
        //
        // Parameters:
        // - `self` — method receiver
        // - `message` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.log(message);

        // use cb when on log is present.

        // Emit output when on log provides a cb.
        if let Some(cb) = &self.options.on_log {
            cb(message);
        }
    }
}


fn pose_value_to_state(pose: &PoseValue) -> PoseState {
    // Pose value to state.
    //
    // Parameters:
    // - `pose` — input value
    //
    // Returns:
    // PoseState.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::runtime::pose_value_to_state(pose);

    // Produce PoseState as the result.
    PoseState {
        x: pose.x,
        y: pose.y,
        theta: pose.theta,
        z: Some(pose.z),
    }
}

// AST accessor extensions
struct TaskSchedule {
    name: String,
    priority: TaskPriority,
    interval_ms: f64,
    deadline_ms: Option<f64>,
    jitter_ms_max: Option<f64>,
    isolated: bool,
    next_due_ms: f64,
    last_start_ms: Option<f64>,
    body: Vec<Stmt>,
    requires: Option<Expr>,
    ensures: Option<Expr>,
    invariant: Option<Expr>,
    budget: Option<crate::foundations::ResourceBudgetDecl>,
}

const RUNTIME_TASK_COST_MS: f64 = 5.0;

fn task_budget_violation_kind(
    budget: &crate::foundations::ResourceBudgetDecl,
    duration_ms: f64,
    interval_ms: f64,
) -> Option<&'static str> {
    // Task budget violation kind.
    //
    // Parameters:
    // - `budget` — input value
    // - `duration_ms` — input value
    // - `interval_ms` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::runtime::task_budget_violation_kind(budget, duration_ms, interval_ms);

    // Compute crate for the following logic.
    let crate::foundations::ResourceBudgetDecl::ResourceBudgetDecl {
        cpu_pct_max,
        battery_pct_max,
        ..
    } = budget;
    let interval = interval_ms.max(1.0);
    let duty_pct = (duration_ms / interval) * 100.0;

    // Emit output when cpu pct max provides a cpu max.
    if let Some(cpu_max) = cpu_pct_max {
        // Take this path when duty pct > *cpu max.
        if duty_pct > *cpu_max {
            return Some("cpu");
        }
    }

    // Emit output when battery pct max provides a bat max.
    if let Some(bat_max) = battery_pct_max {
        // Take this path when duty pct > *bat max.
        if duty_pct > *bat_max {
            return Some("battery");
        }
    }
    None
}

impl TaskSchedule {
    fn priority_rank(&self) -> u8 {
        // Priority rank.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // u8.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.priority_rank();

        // Isolated safety tasks preempt other work at the same priority tier.
        let isolation_rank = if self.isolated { 0 } else { 1 };
        let priority_rank = match self.priority {
            TaskPriority::Critical => 0,
            TaskPriority::High => 1,
            TaskPriority::Normal => 2,
            TaskPriority::Low => 3,
        };
        isolation_rank * 10 + priority_rank
    }
}

fn priority_label(priority: TaskPriority) -> &'static str {
    // Priority label.
    //
    // Parameters:
    // - `priority` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::runtime::priority_label(priority);

    // Match on priority and handle each case.
    match priority {
        TaskPriority::Critical => "critical",
        TaskPriority::High => "high",
        TaskPriority::Normal => "normal",
        TaskPriority::Low => "low",
    }
}

pub(super) fn trigger_category_label(kind: &TriggerKind) -> &'static str {
    // Trigger category label.
    //
    // Parameters:
    // - `kind` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::runtime::trigger_category_label(kind);

    // Match on kind and handle each case.
    match kind {
        TriggerKind::Event { .. } => "event",
        TriggerKind::Message { .. } => "message",
        TriggerKind::Timer { .. } => "timer",
        TriggerKind::Condition { .. } => "condition",
        TriggerKind::StateEntered { .. } => "state_entered",
        TriggerKind::StateExited { .. } => "state_exited",
        TriggerKind::Safety { .. } => "safety",
        TriggerKind::Hardware { .. } => "hardware",
        TriggerKind::Ai { .. } => "ai",
        TriggerKind::Verification { .. } => "verification",
        TriggerKind::Twin { .. } => "twin",
        TriggerKind::LogMatch { .. } => "log_match",
        TriggerKind::MessageMatch { .. } => "message_match",
        TriggerKind::Connectivity { .. } => "connectivity",
        TriggerKind::Geofence { .. } => "geofence",
        TriggerKind::SensorEvent { .. } => "sensor_event",
    }
}

trait RobotDeclExt {
    fn first_behavior_name(&self) -> Option<String>;
    fn behavior_with_contracts(&self, name: &str) -> Option<BehaviorContracts>;
    fn task_with_contracts(&self, name: &str) -> Option<TaskContracts>;
    fn all_task_schedules(&self) -> Vec<TaskSchedule>;
}

impl RobotDeclExt for RobotDecl {
    fn first_behavior_name(&self) -> Option<String> {
        // First behavior name.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.first_behavior_name();

        // Compute RobotDecl for the following logic.
        let RobotDecl::RobotDecl {
            behaviors, tasks, ..
        } = self;

        // Emit output when first provides a b.
        if let Some(b) = behaviors.first() {
            return match b {
                BehaviorDecl::BehaviorDecl { name, .. } => Some(name.clone()),
            };
        }
        tasks.first().map(|t| match t {
            TaskDecl::TaskDecl { name, .. } => name.clone(),
        })
    }

    fn behavior_with_contracts(&self, name: &str) -> Option<BehaviorContracts> {
        // Behavior with contracts.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.behavior_with_contracts(name);

        // Compute RobotDecl for the following logic.
        let RobotDecl::RobotDecl { behaviors, .. } = self;
        behaviors.iter().find_map(|b| match b {
            BehaviorDecl::BehaviorDecl {
                name: n,
                requires,
                ensures,
                invariant,
                body,
                ..
            } if n == name => Some((
                body.clone(),
                requires.clone(),
                ensures.clone(),
                invariant.clone(),
            )),
            _ => None,
        })
    }

    fn task_with_contracts(&self, name: &str) -> Option<TaskContracts> {
        // Task with contracts.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.task_with_contracts(name);

        // Compute RobotDecl for the following logic.
        let RobotDecl::RobotDecl { tasks, .. } = self;
        tasks.iter().find_map(|t| match t {
            TaskDecl::TaskDecl {
                name: n,
                priority: _priority,
                interval_ms,
                requires,
                ensures,
                invariant,
                body,
                ..
            } if n == name => Some((
                body.clone(),
                *interval_ms,
                requires.clone(),
                ensures.clone(),
                invariant.clone(),
            )),
            _ => None,
        })
    }

    fn all_task_schedules(&self) -> Vec<TaskSchedule> {
        // All task schedules.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Vec<TaskSchedule>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.all_task_schedules();

        // Compute RobotDecl for the following logic.
        let RobotDecl::RobotDecl { tasks, .. } = self;
        tasks
            .iter()
            .map(|t| match t {
                TaskDecl::TaskDecl {
                    name,
                    priority,
                    interval_ms,
                    deadline_ms,
                    jitter_ms_max,
                    isolated,
                    requires,
                    ensures,
                    invariant,
                    budget,
                    body,
                    ..
                } => TaskSchedule {
                    name: name.clone(),
                    priority: *priority,
                    interval_ms: *interval_ms,
                    deadline_ms: *deadline_ms,
                    jitter_ms_max: *jitter_ms_max,
                    isolated: *isolated,
                    next_due_ms: 0.0,
                    last_start_ms: None,
                    body: body.clone(),
                    requires: requires.clone(),
                    ensures: ensures.clone(),
                    invariant: invariant.clone(),
                    budget: budget.clone(),
                },
            })
            .collect()
    }
}

trait SocDeclExt {
    fn profile(&self) -> &str;
}

impl SocDeclExt for crate::ast::SocDecl {
    fn profile(&self) -> &str {
        // Profile.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Text result.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.profile();

        // Dispatch based on the enum variant or current state.
        match self {
            crate::ast::SocDecl::SocDecl { profile, .. } => profile,
        }
    }
}

trait HalBlockExt {
    fn members(&self) -> &[crate::ast::HalMemberDecl];
}

impl HalBlockExt for crate::ast::HalBlock {
    fn members(&self) -> &[crate::ast::HalMemberDecl] {
        // Members.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // &[crate::ast::HalMemberDecl].
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.members();

        // Dispatch based on the enum variant or current state.
        match self {
            crate::ast::HalBlock::HalBlock { members, .. } => members,
        }
    }
}

trait SafetyBlockExt {
    fn rules(&self) -> &[SafetyRule];
    fn zones(&self) -> &[SafetyZoneDecl];
}

impl SafetyBlockExt for crate::ast::SafetyBlock {
    fn rules(&self) -> &[SafetyRule] {
        // Rules.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // &[SafetyRule].
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.rules();

        // Dispatch based on the enum variant or current state.
        match self {
            crate::ast::SafetyBlock::SafetyBlock { rules, .. } => rules,
        }
    }

    fn zones(&self) -> &[SafetyZoneDecl] {
        // Zones.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // &[SafetyZoneDecl].
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.zones();

        // Dispatch based on the enum variant or current state.
        match self {
            crate::ast::SafetyBlock::SafetyBlock { zones, .. } => zones,
        }
    }
}

#[path = "runtime_connectivity.rs"]
mod runtime_connectivity;
#[path = "runtime_navigation.rs"]
mod runtime_navigation;
#[path = "runtime_robot.rs"]
mod runtime_robot;
#[path = "runtime_triggers.rs"]
mod runtime_triggers;
#[path = "runtime_sensors.rs"]
mod runtime_sensors;
#[path = "runtime_robotics.rs"]
mod runtime_robotics;
#[path = "runtime_twin.rs"]
mod runtime_twin;
#[path = "runtime_helpers.rs"]
mod runtime_helpers;
#[path = "runtime_builtins.rs"]
mod runtime_builtins;
#[path = "runtime_audit.rs"]
mod runtime_audit;
#[path = "runtime_actuators.rs"]
mod runtime_actuators;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulator::{create_default_simulator, Obstacle, SimulatorConfig};

    fn compile_and_run(source: &str, max_iters: usize) -> Result<RobotState, SpandaError> {
        // Compile and run.
        //
        // Parameters:
        // - `source` — input value
        // - `max_iters` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::runtime::compile_and_run(source, max_iters);

        // Tokenize the source before parsing.
        let tokens = crate::lexer::tokenize(source)?;
        let program = crate::parser::parse(tokens)?;
        let sim = create_default_simulator(SimulatorConfig {
            obstacles: vec![Obstacle {
                x: 100.0,
                y: 100.0,
                radius: 0.1,
            }],
            ..Default::default()
        });
        let mut interp = Interpreter::new(
            sim,
            InterpreterOptions {
                max_loop_iterations: max_iters,
                ..Default::default()
            },
        );
        interp.run(&program, None)
    }

    #[test]
    fn executes_let_bindings_and_if_else() {
        // Executes let bindings and if else.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::runtime::executes_let_bindings_and_if_else();

        let source = r#"
      robot R {
        sensor lidar: Lidar;
        actuator wheels: DifferentialDrive;
        behavior test() {
          let scan = lidar.read();
          if scan.nearest_distance < 0.5 m {
            wheels.stop();
          } else {
            wheels.drive(linear: 0.5 m/s, angular: 0.0 rad/s);
          }
        }
      }
    "#;
        let state = compile_and_run(source, 1).unwrap();
        assert!(state.velocity.linear > 0.0);
    }

    #[test]
    fn runs_deterministic_loop() {
        // Runs deterministic loop.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::runtime::runs_deterministic_loop();

        let source = r#"
      robot R {
        actuator wheels: DifferentialDrive;
        behavior tick() {
          loop every 100ms {
            wheels.drive(linear: 0.1 m/s, angular: 0.0 rad/s);
          }
        }
      }
    "#;
        let state = compile_and_run(source, 5).unwrap();
        assert!(state.pose.x > 0.0);
    }

    #[test]
    fn stops_on_close_obstacle() {
        // Stops on close obstacle.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::runtime::stops_on_close_obstacle();

        let source = r#"
      robot R {
        sensor lidar: Lidar;
        actuator wheels: DifferentialDrive;
        behavior avoid() {
          loop every 50ms {
            let scan = lidar.read();
            if scan.nearest_distance < 0.5 m {
              wheels.stop();
            } else {
              wheels.drive(linear: 0.8 m/s, angular: 0.0 rad/s);
            }
          }
        }
      }
    "#;
        let tokens = crate::lexer::tokenize(source).unwrap();
        let program = crate::parser::parse(tokens).unwrap();
        let sim = create_default_simulator(SimulatorConfig {
            obstacles: vec![Obstacle {
                x: 0.3,
                y: 0.0,
                radius: 0.1,
            }],
            ..Default::default()
        });
        let mut interp = Interpreter::new(
            sim,
            InterpreterOptions {
                max_loop_iterations: 3,
                ..Default::default()
            },
        );
        let state = interp.run(&program, None).unwrap();
        assert_eq!(state.velocity.linear, 0.0);
    }

    #[test]
    fn enforces_safety_in_interpreter() {
        // Enforces safety in interpreter.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::runtime::enforces_safety_in_interpreter();

        let source = r#"
      robot R {
        sensor lidar: Lidar;
        actuator wheels: DifferentialDrive;
        safety {
          stop_if lidar.read().nearest_distance < 1.0 m;
        }
        behavior go() {
          wheels.drive(linear: 0.5 m/s, angular: 0.0 rad/s);
        }
      }
    "#;
        let tokens = crate::lexer::tokenize(source).unwrap();
        let program = crate::parser::parse(tokens).unwrap();
        let sim = create_default_simulator(SimulatorConfig {
            obstacles: vec![Obstacle {
                x: 0.5,
                y: 0.0,
                radius: 0.1,
            }],
            ..Default::default()
        });
        let blocked = Rc::new(RefCell::new(Vec::new()));
        let blocked_cb = blocked.clone();
        let mut interp = Interpreter::new(
            sim,
            InterpreterOptions {
                max_loop_iterations: 1,
                on_motion_blocked: Some(Rc::new(move |reason| {
                    blocked_cb.borrow_mut().push(reason);
                })),
                ..Default::default()
            },
        );
        let state = interp.run(&program, None).unwrap();
        assert!(!blocked.borrow().is_empty());
        assert!(state.emergency_stop);
    }
}
