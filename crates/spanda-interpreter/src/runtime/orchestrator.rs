//! runtime support for Spanda.
//!
use spanda_ai::{AgentRuntime, AiModel};
use spanda_ast::comm_decl::{QosDecl, TransportKind};
use spanda_ast::foundations::{CapabilityDecl, TaskDecl, TaskPriority, TriggerKind};
use spanda_ast::nodes::{BehaviorDecl, Expr, Program, RobotDecl, SafetyRule, SafetyZoneDecl, Stmt};
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
use spanda_transport_routing::RoutingCommBus;
use spanda_typecheck::ModuleRegistry;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

type AgentTraitImplBody = (Vec<spanda_ast::foundations::TraitParamDecl>, Vec<Stmt>);
type BehaviorContracts = (Vec<Stmt>, Option<Expr>, Option<Expr>, Option<Expr>);
type TaskContracts = (Vec<Stmt>, f64, Option<Expr>, Option<Expr>, Option<Expr>);
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

    /// Inject health fault scenarios during simulation.
    pub inject_health_faults: bool,

    /// Optional domain provider registry; defaults to bootstrap shims when unset.
    pub provider_registry: Option<spanda_runtime::providers::ProviderRegistry>,

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
            inject_health_faults: false,
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
        }
    }

    pub fn runtime_host(&self) -> &dyn RuntimeHost {
        self.host
    }

    pub fn telemetry(&self) -> &spanda_runtime::telemetry::RuntimeTelemetry {
        // Telemetry.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // &spanda_runtime::telemetry::RuntimeTelemetry.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.telemetry();

        // Return telemetry from this handle.
        &self.telemetry
    }

    pub fn provider_registry(
        &self,
    ) -> std::cell::Ref<'_, spanda_runtime::providers::ProviderRegistry> {
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

    pub fn take_telemetry(&mut self) -> spanda_runtime::telemetry::RuntimeTelemetry {
        // Take telemetry.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // spanda_runtime::telemetry::RuntimeTelemetry.
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
        spanda_ast::foundations::ModuleFnDecl,
        Vec<spanda_ast::nodes::Expr>,
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
            use spanda_ast::foundations::SimulateCompatibilityDecl;
            let SimulateCompatibilityDecl::SimulateCompatibilityDecl { faults, .. } = sim;
            sim_faults = faults.iter().map(|f| f.fault_type.clone()).collect();
        }
        self.load_program_metadata(program);
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
            }
            self.log("security: injected default security fault scenarios".into());
        }

        if self.options.inject_health_faults {
            for fault in ["GPSDegraded", "CameraOffline", "RobotHealthCritical"] {
                self.hardware_monitor.inject_fault(fault.to_string());
                self.comm_bus.inject_fault(fault);
            }
            self.log("health: injected default health fault scenarios".into());
        }

        if let Some(ks) = &self.options.trigger_kill_switch {
            self.backend.set_emergency_stop(true);
            self.log(format!("kill_switch: activated {ks}"));
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
        for stmt in body {
            if matches!(stmt, Stmt::ExpectCompileErrorStmt { .. }) {
                continue;
            }
            self.execute_stmt(stmt)?;
        }
        Ok(())
    }

    fn verify_expect_compile_error(&self, body: &[Stmt], line: u32) -> Result<(), SpandaError> {
        use spanda_ast::foundations::{ModuleFnDecl, Visibility};
        use spanda_ast::nodes::{Program, SpandaType};
        use spanda_runtime_host::core_type_check_host;
        use spanda_typecheck::check_with_registry;

        let probe = Program::Program {
            module_name: None,
            imports: vec![],
            functions: vec![ModuleFnDecl {
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
            robots: vec![],
            span: Default::default(),
        };
        let registry = self
            .options
            .module_registry
            .clone()
            .unwrap_or_default();
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
    budget: Option<spanda_ast::foundations::ResourceBudgetDecl>,
}

const RUNTIME_TASK_COST_MS: f64 = 5.0;

fn task_budget_violation_kind(
    budget: &spanda_ast::foundations::ResourceBudgetDecl,
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
    let spanda_ast::foundations::ResourceBudgetDecl::ResourceBudgetDecl {
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

impl SocDeclExt for spanda_ast::nodes::SocDecl {
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
            spanda_ast::nodes::SocDecl::SocDecl { profile, .. } => profile,
        }
    }
}

trait HalBlockExt {
    fn members(&self) -> &[spanda_ast::nodes::HalMemberDecl];
}

impl HalBlockExt for spanda_ast::nodes::HalBlock {
    fn members(&self) -> &[spanda_ast::nodes::HalMemberDecl] {
        // Members.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // &[spanda_ast::nodes::HalMemberDecl].
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.members();

        // Dispatch based on the enum variant or current state.
        match self {
            spanda_ast::nodes::HalBlock::HalBlock { members, .. } => members,
        }
    }
}

trait SafetyBlockExt {
    fn rules(&self) -> &[SafetyRule];
    fn zones(&self) -> &[SafetyZoneDecl];
}

impl SafetyBlockExt for spanda_ast::nodes::SafetyBlock {
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
            spanda_ast::nodes::SafetyBlock::SafetyBlock { rules, .. } => rules,
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
            spanda_ast::nodes::SafetyBlock::SafetyBlock { zones, .. } => zones,
        }
    }
}

#[path = "runtime_actuators.rs"]
mod runtime_actuators;
#[path = "runtime_audit.rs"]
mod runtime_audit;
#[path = "runtime_builtins.rs"]
mod runtime_builtins;
#[path = "runtime_connectivity.rs"]
mod runtime_connectivity;
#[path = "runtime_declarations.rs"]
mod runtime_declarations;
#[path = "runtime_eval.rs"]
mod runtime_eval;
#[path = "runtime_execute.rs"]
mod runtime_execute;
#[path = "runtime_helpers.rs"]
mod runtime_helpers;
#[path = "runtime_navigation.rs"]
mod runtime_navigation;
#[path = "runtime_program.rs"]
mod runtime_program;
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
#[path = "runtime_triggers.rs"]
mod runtime_triggers;
#[path = "runtime_twin.rs"]
mod runtime_twin;
#[path = "runtime_world_model.rs"]
mod runtime_world_model;
