use crate::ai::{
    create_agent_runtime, create_ai_model, execute_agent_plan, is_action_proposal, is_safe_action,
    mock_analyze_frame, mock_camera_frame, proposal_from_value, safe_action_from_proposal,
    AgentRuntime, AiModel, MemoryStore, PlanExecutor,
};
use crate::ast::{
    ActionDecl, ActuatorDecl, AgentDecl, BehaviorDecl, BinaryOp, Expr, LiteralValue, Program,
    RobotDecl, SafetyRule, SafetyZoneDecl, SensorBinding, SensorDecl, ServiceDecl, SpandaType,
    Stmt, TopicDecl, UnaryOp, UnitKind, ZoneShape,
};
use crate::audit::{
    sha256 as audit_sha256, sign as audit_sign, verify_signature, AuditRuntime, DeviceIdentity,
    MockLedgerBackend,
};
use crate::comm::{CommBus, DiscoverFilter, TransportKind};
use crate::error::{PoseState, RobotState, SpandaError, VelocityState};
use crate::events::EventBus;
use crate::foundations::{CapabilityDecl, StateMachineDecl, TaskDecl, TwinDecl};
use crate::hal::{create_sim_hal, hal_member_from_decl, HalBackend, SimHalBackend};
use crate::lib_registry::{get_sensor_driver, read_with_driver, DriverContext, SimState};
use crate::safety::{
    create_safety_config_from_robot, interpolate_poses, Pose2d, SafetyMonitor, SafetyZoneRuntime,
    SafetyZoneShape, ValidateActionResult,
};
use crate::security::{
    RobotIdentity, SecretHandle, SecretSource, SecurePolicy, SecurityContext, TrustLevel,
};
use crate::soc::get_soc_profile;
use crate::state_machine::StateMachineRuntime;
use crate::transport::{RoutingCommBus, TransportConfig};
use crate::twin::TwinRuntime;
use crate::units::align_for_binary;
#[cfg(test)]
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

type AgentTraitImplBody = (Vec<crate::foundations::TraitParamDecl>, Vec<Stmt>);
type BehaviorContracts = (Vec<Stmt>, Option<Expr>, Option<Expr>, Option<Expr>);
type TaskContracts = (Vec<Stmt>, f64, Option<Expr>, Option<Expr>, Option<Expr>);

#[derive(Debug, Clone, PartialEq)]
pub struct PoseValue {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
    pub z: f64,
}

impl Default for PoseValue {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            theta: 0.0,
            z: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Number {
        value: f64,
        unit: UnitKind,
    },
    Bool {
        value: bool,
    },
    String {
        value: String,
    },
    Void,
    Scan {
        nearest_distance: f64,
    },
    Pose {
        x: f64,
        y: f64,
        theta: f64,
        z: f64,
    },
    Velocity {
        linear: f64,
        angular: f64,
    },
    Trajectory {
        waypoints: Vec<PoseValue>,
    },
    Transform {
        from_frame: String,
        to_frame: String,
        pose: PoseValue,
    },
    Object {
        type_name: String,
        fields: HashMap<String, RuntimeValue>,
    },
    Enum {
        enum_name: String,
        variant: String,
        payloads: Vec<RuntimeValue>,
    },
    Sensor {
        name: String,
        sensor_type: String,
        library: Option<String>,
        hal_binding: Option<String>,
        topic: Option<String>,
    },
    Actuator {
        name: String,
        actuator_type: String,
    },
    Topic {
        name: String,
        message_type: String,
        topic_path: String,
    },
    Service {
        name: String,
        service_type: String,
    },
    Action {
        name: String,
        action_type: String,
    },
    Robot,
    Agent {
        name: String,
    },
    TraitObject {
        trait_name: String,
        agent: String,
    },
    Twin {
        name: String,
    },
    SafetyCtx,
    AuditCtx,
    LedgerCtx,
    Identity {
        id: String,
        public_key: String,
    },
    Secret {
        name: String,
    },
    AiModel {
        name: String,
        model_type: String,
        provider: String,
    },
    ActionProposal {
        linear: f64,
        angular: f64,
        source: String,
        trace: Vec<String>,
    },
    SafeAction {
        linear: f64,
        angular: f64,
    },
    Goal {
        text: String,
    },
    SensorFusion {
        sensors: Vec<String>,
    },
    Completion {
        text: String,
        model: Option<String>,
    },
    Embedding {
        dimensions: usize,
        vector: Vec<f64>,
    },
    Result {
        ok: bool,
        value: Box<RuntimeValue>,
    },
    Option {
        present: bool,
        value: Option<Box<RuntimeValue>>,
    },
    Bytes {
        data: Vec<u8>,
    },
    Null,
    Future {
        func_name: String,
        args: Vec<RuntimeValue>,
        resolved: Option<Box<RuntimeValue>>,
    },
    Channel {
        id: u64,
    },
}

impl RuntimeValue {
    pub fn number(value: f64, unit: UnitKind) -> Self {
        RuntimeValue::Number { value, unit }
    }

    pub fn string(value: impl Into<String>) -> Self {
        RuntimeValue::String {
            value: value.into(),
        }
    }

    pub fn object(type_name: impl Into<String>, fields: HashMap<String, RuntimeValue>) -> Self {
        RuntimeValue::Object {
            type_name: type_name.into(),
            fields,
        }
    }

    pub fn scan(nearest_distance: f64) -> Self {
        RuntimeValue::Scan { nearest_distance }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            RuntimeValue::Number { value, .. } => Some(*value),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            RuntimeValue::String { value } => Some(value),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MotionCommand {
    Drive {
        linear: f64,
        angular: f64,
        actuator: String,
    },
    Stop {
        actuator: String,
    },
    MoveTo {
        x: f64,
        y: f64,
        z: f64,
        actuator: String,
    },
    Follow {
        waypoints: Vec<PoseValue>,
        actuator: String,
    },
    Grip {
        actuator: String,
    },
    Release {
        actuator: String,
    },
    Open {
        actuator: String,
    },
    SetThrust {
        thrust: f64,
        actuator: String,
    },
    Hover {
        actuator: String,
    },
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
    fn set_emergency_stop(&mut self, _active: bool) {}
    fn publish_topic(&mut self, _topic_path: &str, _message_type: &str, _value: RuntimeValue) {}
    fn call_service(&mut self, _service_name: &str, _service_type: &str) -> RuntimeValue {
        RuntimeValue::Bool { value: true }
    }
    fn send_action(
        &mut self,
        _action_name: &str,
        _action_type: &str,
        _goal: RuntimeValue,
    ) -> RuntimeValue {
        RuntimeValue::Bool { value: true }
    }
    fn get_hal(&mut self) -> Option<&mut dyn HalBackend> {
        None
    }
    fn event_log(&self) -> Vec<String> {
        Vec::new()
    }
}

pub fn format_runtime_value(value: &RuntimeValue) -> String {
    match value {
        RuntimeValue::Number { value, unit } => {
            if *unit == UnitKind::None {
                value.to_string()
            } else {
                format!("{value} {}", unit.as_str())
            }
        }
        RuntimeValue::Bool { value } => value.to_string(),
        RuntimeValue::String { value } => value.clone(),
        RuntimeValue::Void => "void".into(),
        RuntimeValue::Enum {
            variant,
            payloads,
            ..
        } => {
            if payloads.is_empty() {
                variant.clone()
            } else {
                format!(
                    "{variant}({})",
                    payloads
                        .iter()
                        .map(format_runtime_value)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        RuntimeValue::TraitObject { trait_name, agent } => {
            format!("dyn {trait_name}@{agent}")
        }
        RuntimeValue::Agent { name } => format!("agent {name}"),
        RuntimeValue::Object { type_name, fields } => format!(
            "{type_name} {{ {} }}",
            fields
                .iter()
                .map(|(k, v)| format!("{k}: {}", format_runtime_value(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        other => format!("{other:?}"),
    }
}

#[derive(Debug, Clone)]
pub struct Environment {
    bindings: HashMap<String, RuntimeValue>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: impl Into<String>, value: RuntimeValue) {
        self.bindings.insert(name.into(), value);
    }

    pub fn get(&self, name: &str) -> Option<&RuntimeValue> {
        self.bindings.get(name)
    }

    pub fn set(&mut self, name: impl Into<String>, value: RuntimeValue) {
        self.bindings.insert(name.into(), value);
    }

    pub fn clone_bindings(&self) -> Self {
        Self {
            bindings: self.bindings.clone(),
        }
    }

    pub fn snapshot_display(&self) -> std::collections::HashMap<String, String> {
        self.bindings
            .iter()
            .map(|(name, value)| (name.clone(), format_runtime_value(value)))
            .collect()
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RuntimeError {
    pub message: String,
    pub line: u32,
}

impl RuntimeError {
    pub fn new(message: impl Into<String>, line: u32) -> Self {
        Self {
            message: message.into(),
            line,
        }
    }

    pub fn into_spanda(self) -> SpandaError {
        SpandaError::Runtime {
            message: self.message,
            line: self.line,
        }
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
}

impl Default for InterpreterOptions {
    fn default() -> Self {
        Self {
            max_loop_iterations: 10,
            on_motion_blocked: None,
            on_log: None,
            module_registry: None,
            debug: None,
            ffi_registry: crate::ffi::FfiRegistry::new(),
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
    twin: Option<TwinRuntime>,
    state_machines: HashMap<String, StateMachineRuntime>,
    enum_variants: HashMap<String, Vec<String>>,
    variant_owner: HashMap<String, String>,
    struct_defs: HashMap<String, Vec<(String, String)>>,
    agent_trait_impls: HashMap<String, HashMap<String, AgentTraitImplBody>>,
    verify_rules: Vec<Expr>,
    fusion_sensors: Vec<String>,
    audit_runtime: Option<AuditRuntime>,
    mock_ledger: MockLedgerBackend,
    security: SecurityContext,
    comm_bus: RoutingCommBus,
    default_transport: TransportKind,
    module_functions: HashMap<String, crate::foundations::ModuleFnDecl>,
    imported_functions: HashMap<String, crate::foundations::ModuleFnDecl>,
    extern_functions: HashMap<String, crate::foundations::ExternFnDecl>,
    concurrency: crate::concurrency::ConcurrencyRuntime,
}

impl<B: RobotBackend> Interpreter<B> {
    pub fn new(backend: B, options: InterpreterOptions) -> Self {
        Self {
            backend,
            options,
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
            twin: None,
            state_machines: HashMap::new(),
            enum_variants: HashMap::new(),
            variant_owner: HashMap::new(),
            struct_defs: HashMap::new(),
            agent_trait_impls: HashMap::new(),
            verify_rules: Vec::new(),
            fusion_sensors: Vec::new(),
            audit_runtime: None,
            mock_ledger: MockLedgerBackend::new(),
            security: SecurityContext::new(),
            comm_bus: RoutingCommBus::new(),
            default_transport: TransportKind::Sim,
            module_functions: HashMap::new(),
            imported_functions: HashMap::new(),
            extern_functions: HashMap::new(),
            concurrency: crate::concurrency::ConcurrencyRuntime::new(),
        }
    }

    pub fn robot_backend(&self) -> &B {
        &self.backend
    }

    pub fn run(
        &mut self,
        program: &Program,
        entry_behavior: Option<&str>,
    ) -> Result<RobotState, SpandaError> {
        let Program::Program { robots, .. } = program;
        self.load_program_metadata(program);
        for robot in robots {
            self.setup_robot(robot)?;
            let RobotDecl::RobotDecl {
                behaviors, tasks, ..
            } = robot;
            if behaviors.is_empty() && tasks.len() > 1 && entry_behavior.is_none() {
                self.execute_multiplexed_tasks(robot.all_task_schedules())?;
                continue;
            }
            let behavior_name = entry_behavior
                .map(String::from)
                .or_else(|| robot.first_behavior_name());
            if let Some(name) = behavior_name {
                if let Some((body, requires, ensures, invariant)) =
                    robot.behavior_with_contracts(&name)
                {
                    self.execute_with_contracts(&body, &requires, &ensures, &invariant)?;
                } else if let Some((body, interval_ms, requires, ensures, invariant)) =
                    robot.task_with_contracts(&name)
                {
                    self.execute_task_loop_with_contracts(
                        &body,
                        interval_ms,
                        &requires,
                        &ensures,
                        &invariant,
                    )?;
                }
            }
        }
        self.process_spawn_queue()?;
        Ok(self.backend.get_state())
    }

    pub fn run_tests(&mut self, program: &Program) -> Result<(), SpandaError> {
        let Program::Program { tests, .. } = program;
        self.load_program_metadata(program);
        for test in tests {
            self.log(format!("test {}", test.name));
            self.execute_block(&test.body)?;
            self.process_spawn_queue()?;
        }
        Ok(())
    }

    fn load_program_metadata(&mut self, program: &Program) {
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
        for func in functions {
            let ModuleFnDecl {
                name, visibility, ..
            } = func;
            if matches!(visibility, Visibility::Export | Visibility::Public) {
                self.module_functions.insert(name.clone(), func.clone());
            }
        }
        for ext in extern_functions {
            self.extern_functions.insert(ext.name.clone(), ext.clone());
        }
        use crate::ast::ImportDecl;
        if let Some(registry) = &self.options.module_registry {
            for imp in imports {
                let ImportDecl::ImportDecl { path, .. } = imp;
                if let Some(exports) = registry.exports_for(path) {
                    for (name, func) in &exports.functions {
                        self.imported_functions.insert(name.clone(), func.clone());
                    }
                }
            }
        }
        self.enum_variants.clear();
        self.variant_owner.clear();
        self.struct_defs.clear();
        for enum_decl in enums {
            let EnumDecl::EnumDecl { name, variants, .. } = enum_decl;
            self.enum_variants.insert(
                name.clone(),
                variants.iter().map(|v| v.name.clone()).collect(),
            );
            for variant in variants {
                self.variant_owner
                    .insert(variant.name.clone(), name.clone());
            }
        }
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
        for trait_decl in traits {
            let TraitDecl::TraitDecl { name, .. } = trait_decl;
            let _ = name;
        }
    }

    fn setup_robot(&mut self, robot: &RobotDecl) -> Result<(), SpandaError> {
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
            ..
        } = robot;

        self.env = Environment::new();
        self.comm_bus = RoutingCommBus::new();
        self.zones.clear();
        self.stop_if_conditions.clear();
        self.event_bus = EventBus::new();
        self.twin = None;
        self.state_machines.clear();
        self.agent_capabilities.clear();
        self.agent_trait_impls.clear();
        self.verify_rules.clear();
        self.fusion_sensors.clear();
        self.audit_runtime = None;
        self.mock_ledger = MockLedgerBackend::new();
        self.security = SecurityContext::new();
        self.current_agent = None;

        if let Some(soc_decl) = soc {
            let profile_name = soc_decl.profile();
            if let Some(profile) = get_soc_profile(profile_name) {
                self.log(format!("SoC: {} ({})", profile.name, profile.architecture));
            } else {
                self.log(format!("SoC: {profile_name} (unknown)"));
            }
        }

        if let Some(hal_backend) = self.backend.get_hal() {
            // Use simulator HAL when available
            let _ = hal_backend;
        }
        if let Some(hal_block) = hal {
            let members: Vec<_> = hal_block
                .members()
                .iter()
                .map(hal_member_from_decl)
                .collect();
            self.hal.configure(&members);
            self.log(format!("HAL configured: {} bus(es)/pin(s)", members.len()));
        }

        for bus in buses {
            let crate::comm::BusDecl::BusDecl { transport, .. } = bus;
            self.default_transport = *transport;
            self.comm_bus.configure(TransportConfig {
                node_name: Some(robot_name.clone()),
                ..Default::default()
            });
            self.log(format!("bus transport: {}", transport.as_str()));
        }
        for peer in peer_robots {
            let crate::comm::PeerRobotDecl::PeerRobotDecl { name, .. } = peer;
            self.comm_bus.register_robot(name);
        }
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

        for topic in topics {
            self.define_topic(topic);
        }
        for service in services {
            self.define_service(service);
        }
        for action in actions {
            self.define_action(action);
        }
        for sensor in sensors {
            self.define_sensor(sensor);
        }
        for actuator in actuators {
            self.define_actuator(actuator);
        }

        self.ai_models.clear();
        self.agents.clear();
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

        for agent_decl in agents {
            self.setup_agent(agent_decl);
        }
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
            for method in methods {
                agent_methods.insert(
                    method.name.clone(),
                    (method.params.clone(), method.body.clone()),
                );
            }
        }

        for event in events {
            let crate::foundations::EventDecl::EventDecl { name, .. } = event;
            self.log(format!("event declared: {name}"));
        }
        for handler in event_handlers {
            let crate::foundations::EventHandlerDecl::EventHandlerDecl {
                event_name, body, ..
            } = handler;
            self.event_bus.register(event_name.clone(), body.clone());
            self.log(format!("handler registered for {event_name}"));
        }

        if let Some(twin_decl) = twin {
            let TwinDecl::TwinDecl {
                name,
                mirrors,
                replay,
                ..
            } = twin_decl;
            let mut runtime = TwinRuntime::new(name.clone(), mirrors.clone(), *replay);
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

        if let Some(verify_decl) = verify {
            let crate::foundations::VerifyDecl::VerifyDecl { rules, .. } = verify_decl;
            self.verify_rules = rules.clone();
            self.log(format!("verify: {} rule(s) registered", rules.len()));
        }

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

        if let Some(trust_decl) = trust {
            let crate::foundations::TrustDecl::TrustDecl { level, .. } = trust_decl;
            if let Ok(t) = level.parse::<TrustLevel>() {
                self.security.trust = t;
                self.log(format!("trust: level set to {}", t.as_str()));
            }
        }

        for secret_decl in secrets {
            let crate::foundations::SecretDecl::SecretDecl { name, source, .. } = secret_decl;
            let src = match source {
                crate::foundations::SecretSourceDecl::Env { var } => {
                    SecretSource::Env { var: var.clone() }
                }
                crate::foundations::SecretSourceDecl::Literal { value } => SecretSource::Literal {
                    value: value.clone(),
                },
            };
            self.security.secrets.register(SecretHandle {
                name: name.clone(),
                source: src,
            });
            self.env
                .define(name.clone(), RuntimeValue::Secret { name: name.clone() });
            self.log(format!("secret '{name}': registered"));
        }

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
            if let Some(rt) = self.audit_runtime.as_mut() {
                rt.identity = Some(DeviceIdentity::new(id.clone(), public_key));
            }
            self.security.set_identity(robot_id);
            self.security.grant_if_not_strict("identity.sign");
            self.security.grant_if_not_strict("identity.verify");
            self.log(format!("identity: device '{id}' registered"));
        }

        if let Some(audit_decl) = audit {
            let crate::foundations::AuditDecl::AuditDecl { name, records, .. } = audit_decl;
            let watched: Vec<String> = records.iter().map(|e| Self::expr_path_string(e)).collect();
            let mut rt = AuditRuntime::new(name.clone(), watched.clone());
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

        if let Some(provenance_decl) = provenance {
            let crate::foundations::ProvenanceDecl::ProvenanceDecl { name, .. } = provenance_decl;
            self.log(format!("provenance {name}: sha256 signing enabled"));
        }

        for signed in signed_records {
            let crate::foundations::SignedRecordDecl::SignedRecordDecl {
                event_name,
                signed_by,
                ..
            } = signed;
            if let Some(rt) = self.audit_runtime.as_mut() {
                let _ = rt.record_event(event_name, &format!("signed_by={signed_by}"));
            }
            self.log(format!(
                "signed record stream: {event_name} (signed_by {signed_by})"
            ));
        }

        self.env.define("mock_ledger", RuntimeValue::LedgerCtx);
        self.security.grant_if_not_strict("ledger.anchor");

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

        if safety.is_some() {
            self.env.define("safety", RuntimeValue::SafetyCtx);
        }
        self.env.define("robot", RuntimeValue::Robot);

        let mut max_speed = f64::INFINITY;
        if let Some(safety_block) = safety {
            for rule in safety_block.rules() {
                match rule {
                    SafetyRule::MaxSpeedRule { value, .. } => {
                        let val = self.eval_expr(value)?;
                        if let RuntimeValue::Number { value, .. } = val {
                            max_speed = value;
                        }
                    }
                    SafetyRule::StopIfRule { condition, .. } => {
                        self.stop_if_conditions.push(condition.clone());
                    }
                }
            }
            for zone in safety_block.zones() {
                let evaluated = self.eval_safety_zone(zone)?;
                self.zones.push(evaluated);
            }
        }

        self.safety_monitor = Some(SafetyMonitor::new(create_safety_config_from_robot(
            max_speed,
            vec![],
            self.zones.clone(),
        )));

        Ok(())
    }

    fn evaluate_stop_if(&mut self, env: &Environment) -> bool {
        for condition in &self.stop_if_conditions.clone() {
            let saved = self.env.clone_bindings();
            self.env = env.clone_bindings();
            let result = self.eval_expr(condition);
            self.env = saved;
            if let Ok(RuntimeValue::Bool { value: true, .. }) = result {
                return true;
            }
        }
        false
    }

    fn check_safety_before_motion(&mut self) -> bool {
        let state = self.backend.get_state();

        if self.evaluate_stop_if(&self.env.clone_bindings()) {
            self.backend.set_emergency_stop(true);
            if let Some(monitor) = &mut self.safety_monitor {
                monitor.set_emergency_stop(true);
            }
            self.log("stop_if safety rule triggered".into());
            return false;
        }

        if let Some(monitor) = &mut self.safety_monitor {
            let pose2d = Pose2d {
                x: state.pose.x,
                y: state.pose.y,
            };
            let result = monitor.evaluate_before_motion(&self.env, &pose2d);
            if !result.allowed {
                if result.emergency_stop {
                    self.backend.set_emergency_stop(true);
                }
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

        if *shape == ZoneShape::Circle {
            if let Some(r) = radius {
                runtime.radius = Some(get_number(&self.eval_expr(r)?, 0.0));
            }
        }
        if *shape == ZoneShape::Rect {
            if let Some(w) = width {
                runtime.width = Some(get_number(&self.eval_expr(w)?, 0.0));
            }
            if let Some(h) = height {
                runtime.height = Some(get_number(&self.eval_expr(h)?, 0.0));
            }
        }
        Ok(runtime)
    }

    fn define_topic(&mut self, topic: &TopicDecl) {
        let TopicDecl::TopicDecl {
            name,
            message_type,
            topic: topic_path,
            transport,
            secure,
            ..
        } = topic;
        let path = topic_path.clone().unwrap_or_else(|| format!("/{name}"));
        let transport = transport.unwrap_or(self.default_transport);
        if let Some(block) = secure {
            self.security
                .register_secure_endpoint(&path, Self::secure_policy_from_block(block));
        }
        self.comm_bus.subscribe(&path, name);
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
        let ServiceDecl::ServiceDecl {
            name,
            service_type,
            request_type,
            response_type,
            secure,
            ..
        } = service;
        let endpoint = format!("/service/{name}");
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
        let ActionDecl::ActionDecl {
            name,
            action_type,
            result_type,
            secure,
            ..
        } = action;
        let endpoint = format!("/action/{name}");
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
    }

    fn define_actuator(&mut self, actuator: &ActuatorDecl) {
        let ActuatorDecl::ActuatorDecl {
            name,
            actuator_type,
            ..
        } = actuator;
        self.env.define(
            name.clone(),
            RuntimeValue::Actuator {
                name: name.clone(),
                actuator_type: actuator_type.clone(),
            },
        );
    }

    fn setup_agent(&mut self, agent_decl: &AgentDecl) {
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
        self.env
            .define(name.clone(), RuntimeValue::Agent { name: name.clone() });
        self.log(format!("Agent '{name}': {goal}"));
    }

    fn eval_contract(&mut self, expr: &Expr) -> Result<bool, SpandaError> {
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
        if let Some(req) = requires {
            if !self.eval_contract(req)? {
                return Err(RuntimeError::new("requires contract failed", 0).into_spanda());
            }
        }
        self.execute_block(body)?;
        if let Some(ens) = ensures {
            if !self.eval_contract(ens)? {
                return Err(RuntimeError::new("ensures contract failed", 0).into_spanda());
            }
        }
        if let Some(inv) = invariant {
            if !self.eval_contract(inv)? {
                return Err(RuntimeError::new("invariant contract failed", 0).into_spanda());
            }
        }
        self.run_verify_rules()?;
        Ok(())
    }

    fn run_verify_rules(&mut self) -> Result<(), SpandaError> {
        let rules = self.verify_rules.clone();
        if rules.is_empty() {
            return Ok(());
        }
        for (index, rule) in rules.iter().enumerate() {
            match self.eval_expr(rule)? {
                RuntimeValue::Bool { value: true, .. } => {}
                RuntimeValue::Bool { value: false, .. } => {
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

    fn execute_task_loop_with_contracts(
        &mut self,
        body: &[Stmt],
        interval_ms: f64,
        requires: &Option<Expr>,
        ensures: &Option<Expr>,
        invariant: &Option<Expr>,
    ) -> Result<(), SpandaError> {
        for _ in 0..self.options.max_loop_iterations {
            self.backend.tick(interval_ms);
            if !self.execute_task_iteration(body, requires, ensures, invariant, None)? {
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
        if let Some(req) = requires {
            if !self.eval_contract(req)? {
                let label = task_name
                    .map(|name| format!("task '{name}'"))
                    .unwrap_or_else(|| "task".into());
                self.log(format!(
                    "{label} requires contract failed — skipping iteration"
                ));
                return Ok(true);
            }
        }
        self.execute_block(body)?;
        if let Some(ens) = ensures {
            if !self.eval_contract(ens)? {
                return Err(RuntimeError::new("task ensures contract failed", 0).into_spanda());
            }
        }
        if let Some(inv) = invariant {
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
        self.log(format!(
            "scheduler: multiplexing {} task(s) with base tick {}ms",
            schedules.len(),
            base_tick
        ));
        for _ in 0..self.options.max_loop_iterations {
            self.backend.tick(base_tick);
            sim_time += base_tick;
            for schedule in &mut schedules {
                if schedule.next_due_ms <= sim_time {
                    self.log(format!("task '{}': tick", schedule.name));
                    if !self.execute_task_iteration(
                        &schedule.body,
                        &schedule.requires,
                        &schedule.ensures,
                        &schedule.invariant,
                        Some(&schedule.name),
                    )? {
                        return Ok(());
                    }
                    schedule.next_due_ms = sim_time + schedule.interval_ms;
                }
            }
            self.run_verify_rules()?;
            self.update_twin_snapshot();
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

    fn update_twin_snapshot(&mut self) {
        self.refresh_twin_shadow_from_backend();
        let Some(twin) = &mut self.twin else {
            return;
        };
        twin.commit_frame();
        let twin_name = twin.name.clone();
        let field_count = twin.shadow.len();
        let replay_frames = twin.replay_frame_count();
        if field_count > 0 || twin.telemetry_sync {
            self.log(format!(
                "twin {twin_name} mirrored {field_count} field(s), replay frames={replay_frames}"
            ));
        }
    }

    fn refresh_twin_shadow_from_backend(&mut self) {
        let Some(twin) = &mut self.twin else {
            return;
        };
        let state = self.backend.get_state();
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

    fn dispatch_event(&mut self, event_name: &str) -> Result<(), SpandaError> {
        if let Some(body) = self.event_bus.handler_body(event_name).map(|b| b.to_vec()) {
            self.log(format!("emit {event_name}"));
            self.execute_block(&body)?;
        } else {
            self.log(format!("emit {event_name} (no handler)"));
        }
        Ok(())
    }

    fn execute_enter(&mut self, state_name: &str, line: u32) -> Result<(), SpandaError> {
        let mut logs = Vec::new();
        let mut transitioned = false;
        for (sm_name, sm) in &mut self.state_machines {
            if let Some(previous) = sm.try_enter(state_name) {
                logs.push(format!(
                    "state_machine {sm_name}: {previous} -> {state_name}"
                ));
                transitioned = true;
            }
        }
        for msg in logs {
            self.log(msg);
        }
        if !transitioned {
            return Err(RuntimeError::new(
                format!("No valid transition to state '{state_name}'"),
                line,
            )
            .into_spanda());
        }
        Ok(())
    }

    fn check_agent_capability(
        &self,
        agent: &str,
        action: &str,
        target: Option<&str>,
        line: u32,
    ) -> Result<(), SpandaError> {
        let caps = self
            .agent_capabilities
            .get(agent)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        if caps.is_empty() {
            return Ok(());
        }
        let allowed = caps
            .iter()
            .any(|c| c.action == action && (target.is_none() || c.target.as_deref() == target));
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

    fn secure_policy_from_block(block: &crate::foundations::SecureBlockDecl) -> SecurePolicy {
        SecurePolicy {
            signed: block.signed,
            min_trust: block
                .min_trust
                .as_ref()
                .and_then(|s| s.parse::<TrustLevel>().ok()),
            requires: block.requires.clone(),
        }
    }

    fn resolve_signing_key(&self, key: &str) -> Result<String, SpandaError> {
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
        RuntimeError::new(err.to_string(), line).into_spanda()
    }

    fn execute_block(&mut self, stmts: &[Stmt]) -> Result<(), SpandaError> {
        for stmt in stmts {
            self.execute_stmt(stmt)?;
        }
        Ok(())
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> Result<(), SpandaError> {
        if let Some(debug) = &self.options.debug {
            let line = crate::debug::stmt_line(stmt);
            if debug.should_pause(line) {
                let variables = self.env.snapshot_display();
                debug.record_pause(line, "breakpoint", variables);
                return Err(SpandaError::DebugPause {
                    line,
                    reason: "breakpoint".into(),
                });
            }
        }
        match stmt {
            Stmt::VarDecl {
                name,
                type_annotation,
                init,
                ..
            } => {
                if let Some(expr) = init {
                    let value = if matches!(
                        type_annotation,
                        Some(SpandaType::TraitObject { .. })
                    ) {
                        if let Expr::IdentExpr { name: agent, .. } = expr {
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
                if matches!(cond, RuntimeValue::Bool { value: true, .. }) {
                    self.execute_block(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute_block(else_branch)?;
                }
            }
            Stmt::LoopStmt {
                interval_ms, body, ..
            } => {
                for _ in 0..self.options.max_loop_iterations {
                    self.backend.tick(*interval_ms);
                    self.execute_block(body)?;
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
                if let Some(agent) = self.current_agent.clone() {
                    self.check_agent_capability(&agent, "publish", Some(topic_name), line)?;
                }
                let topic = self.env.get(topic_name).cloned();
                let val = self.eval_expr(value)?;
                if let Some(RuntimeValue::Topic {
                    topic_path,
                    message_type,
                    ..
                }) = topic
                {
                    let payload = Self::runtime_value_payload(&val);
                    if let Err(e) = self.security.sign_outbound(&topic_path, &payload) {
                        return Err(self.security_error(e, line));
                    }
                    self.comm_bus.publish(
                        &topic_path,
                        &message_type,
                        val.clone(),
                        self.default_transport,
                    );
                    self.backend.publish_topic(&topic_path, &message_type, val);
                    if let Some(rt) = self.audit_runtime.as_mut() {
                        let _ = self.security.audit_event(
                            rt,
                            "publish",
                            &format!("topic={topic_path}"),
                        );
                    }
                    self.log(format!("publish {topic_path}"));
                }
            }
            Stmt::ServiceCallStmt {
                service_name, span, ..
            } => {
                let line = span.start.line;
                if let Some(agent) = self.current_agent.clone() {
                    self.check_agent_capability(&agent, "call", Some(service_name), line)?;
                }
                if let Some(RuntimeValue::Service { name, service_type }) =
                    self.env.get(service_name).cloned()
                {
                    let endpoint = format!("/service/{name}");
                    if let Err(e) = self.security.verify_inbound(&endpoint, None) {
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
                if let Some(agent) = self.current_agent.clone() {
                    self.check_agent_capability(&agent, "execute", Some(action_name), line)?;
                }
                if let Some(RuntimeValue::Action { name, action_type }) =
                    self.env.get(action_name).cloned()
                {
                    let endpoint = format!("/action/{name}");
                    let goal_val = self.eval_expr(goal)?;
                    let payload = Self::runtime_value_payload(&goal_val);
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
                if let Err(e) = self.security.verify_inbound(&path, None) {
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
                if let Some(agent) = self.current_agent.clone() {
                    self.check_agent_capability(&agent, "execute", Some(action_name), line)?;
                }
                if let Some(RuntimeValue::Action { name, action_type }) =
                    self.env.get(action_name).cloned()
                {
                    let endpoint = format!("/action/{name}");
                    let goal_val = self.eval_expr(goal)?;
                    let payload = Self::runtime_value_payload(&goal_val);
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
                ..
            } => {
                if let Some(RuntimeValue::Topic { topic_path, .. }) = self.env.get(topic_name) {
                    let path = topic_path.clone();
                    if let Some(val) = self.comm_bus.receive(&path) {
                        self.env.define(var_name.clone(), val);
                        self.log(format!("receive {topic_name} to {var_name}"));
                    }
                }
            }
            Stmt::EmergencyStopStmt { .. } => {
                if let Some(monitor) = &mut self.safety_monitor {
                    monitor.set_emergency_stop(true);
                }
                self.backend.set_emergency_stop(true);
                self.backend.execute_motion(MotionCommand::Stop {
                    actuator: "all".into(),
                });
                self.log("EMERGENCY STOP triggered".into());
            }
            Stmt::ResetEmergencyStopStmt { .. } => {
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
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.eval_expr(arg)?);
                }
                let name = match callee {
                    Expr::IdentExpr { name, .. } => name.clone(),
                    _ => {
                        return Err(RuntimeError::new(
                            "spawn requires function name",
                            span.start.line,
                        )
                        .into_spanda())
                    }
                };
                self.concurrency.queue_spawn(name, arg_values);
            }
            Stmt::SelectStmt { arms, span } => {
                'select: for arm in arms {
                    let channel_val = self.eval_expr(&arm.channel)?;
                    if let Some(msg) = self.concurrency.try_recv(&channel_val, span.start.line)? {
                        self.env.define("_msg", msg);
                        self.execute_block(&arm.body)?;
                        break 'select;
                    }
                }
            }
            Stmt::ReturnStmt { .. } => {}
        }
        Ok(())
    }

    fn execute_block_with_return(
        &mut self,
        stmts: &[Stmt],
    ) -> Result<Option<RuntimeValue>, SpandaError> {
        for stmt in stmts {
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
                if matches!(cond, RuntimeValue::Bool { value: true, .. }) {
                    if let Some(v) = self.execute_block_with_return(then_branch)? {
                        return Ok(Some(v));
                    }
                } else if let Some(else_branch) = else_branch {
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
        let saved = self.env.clone_bindings();
        for (i, param) in func.params.iter().enumerate() {
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
                for (i, param) in func.params.iter().enumerate() {
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

    fn process_spawn_queue(&mut self) -> Result<(), SpandaError> {
        let jobs = self.concurrency.drain_spawn_queue();
        for job in jobs {
            if let Some(func) = self
                .module_functions
                .get(&job.func_name)
                .or_else(|| self.imported_functions.get(&job.func_name))
                .cloned()
            {
                let saved = self.env.clone_bindings();
                for (i, param) in func.params.iter().enumerate() {
                    if let Some(val) = job.args.get(i) {
                        self.env.define(param.name.clone(), val.clone());
                    }
                }
                self.execute_block(&func.body)?;
                self.env = saved;
            }
        }
        Ok(())
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<RuntimeValue, SpandaError> {
        match expr {
            Expr::LiteralExpr { value, .. } => Ok(match value {
                LiteralValue::Bool(b) => RuntimeValue::Bool { value: *b },
                LiteralValue::Number(n) => RuntimeValue::Number {
                    value: *n,
                    unit: UnitKind::None,
                },
                LiteralValue::String(s) => RuntimeValue::String { value: s.clone() },
                LiteralValue::Null => RuntimeValue::Void,
            }),
            Expr::UnitLiteralExpr { value, unit, .. } => Ok(RuntimeValue::Number {
                value: *value,
                unit: *unit,
            }),
            Expr::IdentExpr { name, span } => {
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
                match op {
                    UnaryOp::Not => Ok(RuntimeValue::Bool {
                        value: matches!(operand_val, RuntimeValue::Bool { value, .. } if !value),
                    }),
                    UnaryOp::Neg => {
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
                if let Expr::IdentExpr { name, .. } = object.as_ref() {
                    if let Some(variants) = self.enum_variants.get(name) {
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
                for arm in arms {
                    if arm.variant == variant {
                        if !arm.bindings.is_empty() {
                            if let RuntimeValue::Enum { payloads, .. } = &value {
                                for (binding, payload) in arm.bindings.iter().zip(payloads.iter()) {
                                    self.env.set(binding.clone(), payload.clone());
                                }
                            }
                        }
                        for stmt in &arm.body {
                            self.execute_stmt(stmt)?;
                        }
                        for binding in &arm.bindings {
                            self.env.bindings.remove(binding);
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
        let mut values = HashMap::new();
        for field in fields {
            values.insert(field.name.clone(), self.eval_expr(&field.value)?);
        }
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

    fn eval_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        named_args: &[crate::ast::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
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

    fn eval_ai_method(
        &mut self,
        target_name: &str,
        method: &str,
        args: &[Expr],
        named_args: &[crate::ast::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        match method {
            "reason" => {
                if let Some(agent) = self.current_agent.as_deref() {
                    self.check_agent_capability(agent, "propose_motion", None, line)?;
                }
                let prompt = get_string(&self.get_named_arg_value(named_args, "prompt")?, "");
                let input = self.get_named_arg_value(named_args, "input")?;
                let input = if matches!(input, RuntimeValue::Void) {
                    None
                } else {
                    Some(input)
                };
                let goal_text = self.resolve_reason_goal(named_args, line)?;
                let goal_text = self.enrich_reason_goal(goal_text);
                let result = self
                    .ai_models
                    .get(target_name)
                    .ok_or_else(|| {
                        RuntimeError::new(format!("Unknown AI model '{target_name}'"), line)
                            .into_spanda()
                    })?
                    .reason(&prompt, input, goal_text.as_deref())
                    .map_err(|message| SpandaError::Runtime { message, line })?;
                self.log(format!("ai {target_name}.reason() -> ActionProposal"));
                Ok(result)
            }
            "summarize" => {
                if let Some(agent) = self.current_agent.as_deref() {
                    self.check_agent_capability(agent, "summarize", None, line)?;
                }
                let input = self.get_named_arg_value(named_args, "input")?;
                let input = if matches!(input, RuntimeValue::Void) {
                    None
                } else {
                    Some(input)
                };
                self.ai_models
                    .get(target_name)
                    .ok_or_else(|| {
                        RuntimeError::new(format!("Unknown AI model '{target_name}'"), line)
                            .into_spanda()
                    })?
                    .summarize(input)
                    .map_err(|message| SpandaError::Runtime { message, line })
            }
            "detect" => {
                if let Some(agent) = self.current_agent.as_deref() {
                    self.check_agent_capability(agent, "detect", None, line)?;
                }
                let frame = if let Some(first) = args.first() {
                    self.eval_expr(first)?
                } else {
                    self.get_named_arg_value(named_args, "frame")?
                };
                self.ai_models
                    .get(target_name)
                    .ok_or_else(|| {
                        RuntimeError::new(format!("Unknown AI model '{target_name}'"), line)
                            .into_spanda()
                    })?
                    .detect(frame)
                    .map_err(|message| SpandaError::Runtime { message, line })
            }
            "drive" => Err(RuntimeError::new(
                "Unsafe AI action: LLM cannot drive actuators directly — use safety.validate() then wheels.execute()",
                line,
            )
            .into_spanda()),
            _ => Ok(RuntimeValue::Void),
        }
    }

    fn read_sensor_value(&mut self, target: &RuntimeValue) -> Result<RuntimeValue, SpandaError> {
        let RuntimeValue::Sensor {
            name,
            sensor_type,
            library,
            hal_binding,
            topic,
        } = target
        else {
            return Ok(RuntimeValue::Void);
        };

        let state = self.backend.get_state();
        if let Some(lib) = library {
            if let Some(driver) = get_sensor_driver(lib, sensor_type) {
                let ctx = DriverContext {
                    hal: Some(&self.hal),
                    hal_binding: hal_binding.as_deref(),
                    topic: topic.as_deref(),
                    sim_state: Some(SimState {
                        pose: state.pose.clone(),
                    }),
                };
                return Ok(read_with_driver(&driver, &ctx));
            }
        }
        Ok(self
            .backend
            .read_sensor(name, sensor_type, topic.as_deref()))
    }

    fn read_fused_observation(&mut self) -> Result<RuntimeValue, SpandaError> {
        let sensors = self.fusion_sensors.clone();
        let mut fields = HashMap::new();
        for sensor_name in &sensors {
            let sensor_val = self.env.get(sensor_name).cloned().ok_or_else(|| {
                RuntimeError::new(format!("Unknown observe sensor '{sensor_name}'"), 0)
                    .into_spanda()
            })?;
            let reading = self.read_sensor_value(&sensor_val)?;
            fields.insert(sensor_name.clone(), reading);
        }
        let state = self.backend.get_state();
        fields.insert("pose".into(), pose_from_state(&state.pose));
        fields.insert(
            "count".into(),
            RuntimeValue::Number {
                value: sensors.len() as f64,
                unit: UnitKind::None,
            },
        );
        Ok(RuntimeValue::Object {
            type_name: "FusedObservation".into(),
            fields,
        })
    }

    fn eval_builtin_function(
        &mut self,
        name: &str,
        args: &[Expr],
        named_args: &[crate::ast::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        match name {
            "pose" => Ok(runtime_pose(
                get_number(&self.get_named_arg_value(named_args, "x")?, 0.0),
                get_number(&self.get_named_arg_value(named_args, "y")?, 0.0),
                get_number(&self.get_named_arg_value(named_args, "theta")?, 0.0),
                get_number(&self.get_named_arg_value(named_args, "z")?, 0.0),
            )),
            "velocity" => Ok(runtime_velocity(
                get_number(&self.get_named_arg_value(named_args, "linear")?, 0.0),
                get_number(&self.get_named_arg_value(named_args, "angular")?, 0.0),
            )),
            "trajectory" => {
                let from_val = self.get_named_arg_value(named_args, "from")?;
                let to_val = self.get_named_arg_value(named_args, "to")?;
                let steps = get_number(&self.get_named_arg_value(named_args, "steps")?, 5.0);
                let from = get_pose_fields(&from_val).unwrap_or_default();
                let to = get_pose_fields(&to_val).unwrap_or_default();
                let waypoints: Vec<PoseValue> = interpolate_poses(
                    &pose_value_to_state(&from),
                    &pose_value_to_state(&to),
                    steps,
                )
                .into_iter()
                .map(|p| PoseValue {
                    x: p.x,
                    y: p.y,
                    theta: p.theta,
                    z: p.z,
                })
                .collect();
                Ok(runtime_trajectory(waypoints))
            }
            "transform" => {
                let from_frame = get_string(&self.get_named_arg_value(named_args, "from")?, "base");
                let to_frame = get_string(&self.get_named_arg_value(named_args, "to")?, "map");
                let pose = get_pose_fields(&self.get_named_arg_value(named_args, "pose")?)
                    .unwrap_or_default();
                Ok(RuntimeValue::Transform {
                    from_frame,
                    to_frame,
                    pose,
                })
            }
            "goal" => {
                let text = if let Some(arg) = args.first() {
                    match self.eval_expr(arg)? {
                        RuntimeValue::String { value } => value,
                        RuntimeValue::Goal { text } => text,
                        _ => String::new(),
                    }
                } else {
                    get_string(&self.get_named_arg_value(named_args, "text")?, "")
                };
                Ok(RuntimeValue::Goal { text })
            }
            "recall" => {
                let key = if let Some(arg) = args.first() {
                    match self.eval_expr(arg)? {
                        RuntimeValue::String { value } => value,
                        _ => String::new(),
                    }
                } else {
                    get_string(&self.get_named_arg_value(named_args, "key")?, "")
                };
                let agent_name = self.current_agent.clone().ok_or_else(|| {
                    RuntimeError::new(
                        "recall requires active agent context (run inside agent plan)",
                        line,
                    )
                    .into_spanda()
                })?;
                let agent = self.agents.get(&agent_name).ok_or_else(|| {
                    RuntimeError::new(format!("Unknown agent '{agent_name}'"), line).into_spanda()
                })?;
                let memory = agent.memory.as_ref().ok_or_else(|| {
                    RuntimeError::new(
                        "Agent has no memory — declare memory short_term or long_term on the agent",
                        line,
                    )
                    .into_spanda()
                })?;
                Ok(memory.recall(&key).cloned().unwrap_or(RuntimeValue::Void))
            }
            "sha256" => {
                let data = if let Some(arg) = args.first() {
                    get_string(&self.eval_expr(arg)?, "")
                } else {
                    get_string(&self.get_named_arg_value(named_args, "data")?, "")
                };
                let hash = audit_sha256(&data);
                Ok(RuntimeValue::Object {
                    type_name: "Hash".into(),
                    fields: HashMap::from([("hex".into(), RuntimeValue::String { value: hash.0 })]),
                })
            }
            "sign" => {
                self.security
                    .require_operation("identity.sign")
                    .map_err(|e| self.security_error(e, line))?;
                let data = if let Some(arg) = args.first() {
                    get_string(&self.eval_expr(arg)?, "")
                } else {
                    get_string(&self.get_named_arg_value(named_args, "data")?, "")
                };
                let key_raw = if args.len() > 1 {
                    get_string(&self.eval_expr(&args[1])?, "")
                } else {
                    get_string(&self.get_named_arg_value(named_args, "key")?, "")
                };
                let key = self.resolve_signing_key(&key_raw)?;
                Ok(RuntimeValue::Object {
                    type_name: "Signature".into(),
                    fields: HashMap::from([(
                        "value".into(),
                        RuntimeValue::String {
                            value: audit_sign(&data, &key),
                        },
                    )]),
                })
            }
            "verify_signature" => {
                self.security
                    .require_operation("identity.verify")
                    .map_err(|e| self.security_error(e, line))?;
                let data = if let Some(arg) = args.first() {
                    get_string(&self.eval_expr(arg)?, "")
                } else {
                    get_string(&self.get_named_arg_value(named_args, "data")?, "")
                };
                let signature = if args.len() > 1 {
                    get_string(&self.eval_expr(&args[1])?, "")
                } else {
                    get_string(&self.get_named_arg_value(named_args, "signature")?, "")
                };
                let key_raw = if args.len() > 2 {
                    get_string(&self.eval_expr(&args[2])?, "")
                } else {
                    get_string(&self.get_named_arg_value(named_args, "key")?, "")
                };
                let key = self.resolve_signing_key(&key_raw)?;
                Ok(RuntimeValue::Bool {
                    value: verify_signature(&data, &signature, &key),
                })
            }
            "Ok" => {
                let val = if let Some(arg) = args.first() {
                    self.eval_expr(arg)?
                } else {
                    RuntimeValue::Void
                };
                Ok(RuntimeValue::Result {
                    ok: true,
                    value: Box::new(val),
                })
            }
            "Err" => {
                let val = if let Some(arg) = args.first() {
                    self.eval_expr(arg)?
                } else {
                    RuntimeValue::Object {
                        type_name: "Error".into(),
                        fields: HashMap::new(),
                    }
                };
                Ok(RuntimeValue::Result {
                    ok: false,
                    value: Box::new(val),
                })
            }
            "Some" => {
                let val = if let Some(arg) = args.first() {
                    self.eval_expr(arg)?
                } else {
                    RuntimeValue::Void
                };
                Ok(RuntimeValue::Option {
                    present: true,
                    value: Some(Box::new(val)),
                })
            }
            "None" => Ok(RuntimeValue::Option {
                present: false,
                value: None,
            }),
            "channel" => Ok(self.concurrency.create_channel()),
            "send" => {
                let channel = args
                    .first()
                    .map(|a| self.eval_expr(a))
                    .transpose()?
                    .ok_or_else(|| {
                        RuntimeError::new("send requires channel", line).into_spanda()
                    })?;
                let value = if args.len() > 1 {
                    self.eval_expr(&args[1])?
                } else {
                    self.get_named_arg_value(named_args, "value")?
                };
                self.concurrency.send(&channel, value, line)?;
                Ok(RuntimeValue::Void)
            }
            "recv" => {
                let channel = args
                    .first()
                    .map(|a| self.eval_expr(a))
                    .transpose()?
                    .ok_or_else(|| {
                        RuntimeError::new("recv requires channel", line).into_spanda()
                    })?;
                match self.concurrency.try_recv(&channel, line)? {
                    Some(value) => Ok(value),
                    None => Ok(RuntimeValue::Void),
                }
            }
            "serialize" => {
                let value = args
                    .first()
                    .map(|a| self.eval_expr(a))
                    .transpose()?
                    .unwrap_or(RuntimeValue::Void);
                let format = if args.len() > 1 {
                    get_string(&self.eval_expr(&args[1])?, "json")
                } else {
                    get_string(&self.get_named_arg_value(named_args, "format")?, "json")
                };
                crate::serialize::serialize_value(&value, &format)
            }
            "deserialize" => {
                let data = args
                    .first()
                    .map(|a| self.eval_expr(a))
                    .transpose()?
                    .ok_or_else(|| {
                        RuntimeError::new("deserialize requires data", line).into_spanda()
                    })?;
                let format = if args.len() > 1 {
                    get_string(&self.eval_expr(&args[1])?, "json")
                } else {
                    get_string(&self.get_named_arg_value(named_args, "format")?, "json")
                };
                crate::serialize::deserialize_value(&data, &format)
            }
            "assert" => {
                let condition = args
                    .first()
                    .map(|a| self.eval_expr(a))
                    .transpose()?
                    .ok_or_else(|| {
                        RuntimeError::new("assert requires a boolean condition", line).into_spanda()
                    })?;
                match condition {
                    RuntimeValue::Bool { value: true, .. } => Ok(RuntimeValue::Void),
                    RuntimeValue::Bool { value: false, .. } => {
                        Err(RuntimeError::new("Assertion failed", line).into_spanda())
                    }
                    _ => Err(
                        RuntimeError::new("assert requires a boolean condition", line)
                            .into_spanda(),
                    ),
                }
            }
            _ => Ok(RuntimeValue::Void),
        }
    }

    fn goal_text_from_value(value: &RuntimeValue) -> Option<String> {
        match value {
            RuntimeValue::Goal { text } => Some(text.clone()),
            RuntimeValue::String { value } => Some(value.clone()),
            _ => None,
        }
    }

    fn resolve_reason_goal(
        &mut self,
        named_args: &[crate::ast::NamedArg],
        line: u32,
    ) -> Result<Option<String>, SpandaError> {
        if let Ok(value) = self.get_named_arg_value(named_args, "goal") {
            if !matches!(value, RuntimeValue::Void) {
                return Ok(Self::goal_text_from_value(&value));
            }
        }
        if let Some(agent_name) = self.current_agent.as_deref() {
            if let Some(agent) = self.agents.get(agent_name) {
                let text = match &agent.decl {
                    AgentDecl::AgentDecl { goal, .. } => goal.clone(),
                };
                if !text.is_empty() {
                    return Ok(Some(text));
                }
            }
        }
        let _ = line;
        Ok(None)
    }

    fn enrich_reason_goal(&self, goal: Option<String>) -> Option<String> {
        let mut parts = Vec::new();
        if let Some(g) = goal.filter(|s| !s.is_empty()) {
            parts.push(g);
        }
        if let Some(agent_name) = self.current_agent.as_deref() {
            if let Some(summary) = self
                .agents
                .get(agent_name)
                .and_then(|a| a.memory.as_ref())
                .and_then(MemoryStore::summary_for_prompt)
            {
                parts.push(summary);
            }
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("\n"))
        }
    }

    fn expr_path_string(expr: &Expr) -> String {
        match expr {
            Expr::IdentExpr { name, .. } => name.clone(),
            Expr::MemberExpr {
                object, property, ..
            } => {
                format!("{}.{}", Self::expr_path_string(object), property)
            }
            _ => String::new(),
        }
    }

    fn runtime_value_payload(value: &RuntimeValue) -> String {
        match value {
            RuntimeValue::String { value } => value.clone(),
            RuntimeValue::Number { value, .. } => value.to_string(),
            RuntimeValue::Bool { value } => value.to_string(),
            RuntimeValue::Pose { x, y, theta, z } => {
                format!(r#"{{"x":{x},"y":{y},"theta":{theta},"z":{z}}}"#)
            }
            RuntimeValue::SafeAction { linear, angular } => {
                format!(r#"{{"linear":{linear},"angular":{angular}}}"#)
            }
            RuntimeValue::ActionProposal {
                linear,
                angular,
                source,
                ..
            } => format!(r#"{{"linear":{linear},"angular":{angular},"source":"{source}"}}"#),
            _ => format!("{value:?}"),
        }
    }

    fn eval_audit_method(
        &mut self,
        method: &str,
        args: &[Expr],
        _named_args: &[crate::ast::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        match method {
            "record" => {
                self.security
                    .require_operation("audit.record")
                    .map_err(|e| self.security_error(e, line))?;
                let event_type = if let Some(arg) = args.first() {
                    get_string(&self.eval_expr(arg)?, "event")
                } else {
                    "event".into()
                };
                let payload = if args.len() > 1 {
                    Self::runtime_value_payload(&self.eval_expr(&args[1])?)
                } else {
                    String::new()
                };
                let rt = self.audit_runtime.as_mut().ok_or_else(|| {
                    RuntimeError::new(
                        "Audit not configured — declare an audit block on the robot",
                        line,
                    )
                    .into_spanda()
                })?;
                let id = rt.record_event(&event_type, &payload).map_err(|e| {
                    RuntimeError::new(format!("audit.record failed: {e}"), line).into_spanda()
                })?;
                let _ = self.security.audit_event(rt, "audit.record", &event_type);
                self.log(format!("audit.record({event_type}) -> {}", id.0));
                Ok(RuntimeValue::Object {
                    type_name: "RecordId".into(),
                    fields: HashMap::from([("id".into(), RuntimeValue::String { value: id.0 })]),
                })
            }
            "export" => {
                self.security
                    .require_operation("audit.read")
                    .map_err(|e| self.security_error(e, line))?;
                let rt = self.audit_runtime.as_mut().ok_or_else(|| {
                    RuntimeError::new(
                        "Audit not configured — declare an audit block on the robot",
                        line,
                    )
                    .into_spanda()
                })?;
                let json = rt.export_json().map_err(|e| {
                    RuntimeError::new(format!("audit.export failed: {e}"), line).into_spanda()
                })?;
                Ok(RuntimeValue::String { value: json })
            }
            "count" => {
                let count = self
                    .audit_runtime
                    .as_ref()
                    .map(|rt| rt.record_count())
                    .unwrap_or(0);
                Ok(RuntimeValue::Number {
                    value: count as f64,
                    unit: UnitKind::None,
                })
            }
            "root_hash" => {
                let hash = self
                    .audit_runtime
                    .as_ref()
                    .and_then(|rt| rt.root_hash())
                    .map(|h| h.0)
                    .unwrap_or_default();
                Ok(RuntimeValue::Object {
                    type_name: "Hash".into(),
                    fields: HashMap::from([("hex".into(), RuntimeValue::String { value: hash })]),
                })
            }
            "create_provenance" => {
                self.security
                    .require_operation("identity.sign")
                    .map_err(|e| self.security_error(e, line))?;
                let name = if let Some(arg) = args.first() {
                    get_string(&self.eval_expr(arg)?, "provenance")
                } else {
                    "provenance".into()
                };
                let record_id = if args.len() > 1 {
                    match self.eval_expr(&args[1])? {
                        RuntimeValue::Object { fields, .. } => fields
                            .get("id")
                            .and_then(|v| match v {
                                RuntimeValue::String { value } => {
                                    Some(crate::audit::RecordId(value.clone()))
                                }
                                _ => None,
                            })
                            .unwrap_or_else(|| crate::audit::RecordId("audit-1".into())),
                        _ => crate::audit::RecordId("audit-1".into()),
                    }
                } else {
                    crate::audit::RecordId("audit-1".into())
                };
                let rt = self.audit_runtime.as_ref().ok_or_else(|| {
                    RuntimeError::new(
                        "Audit not configured — declare an audit block on the robot",
                        line,
                    )
                    .into_spanda()
                })?;
                let prov = rt.create_provenance(&name, &record_id).map_err(|e| {
                    RuntimeError::new(format!("audit.create_provenance failed: {e}"), line)
                        .into_spanda()
                })?;
                Ok(RuntimeValue::Object {
                    type_name: "ProvenanceRecord".into(),
                    fields: HashMap::from([
                        (
                            "name".into(),
                            RuntimeValue::String {
                                value: prov.name.clone(),
                            },
                        ),
                        (
                            "record_id".into(),
                            RuntimeValue::Object {
                                type_name: "RecordId".into(),
                                fields: HashMap::from([(
                                    "id".into(),
                                    RuntimeValue::String {
                                        value: prov.record_id.0.clone(),
                                    },
                                )]),
                            },
                        ),
                        (
                            "signed_by".into(),
                            RuntimeValue::String {
                                value: prov.signed_by.clone(),
                            },
                        ),
                    ]),
                })
            }
            _ => Ok(RuntimeValue::Void),
        }
    }

    fn eval_ledger_method(
        &mut self,
        method: &str,
        args: &[Expr],
        _named_args: &[crate::ast::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        use crate::audit::LedgerBackend;
        match method {
            "anchor" => {
                self.security
                    .require_operation("ledger.anchor")
                    .map_err(|e| self.security_error(e, line))?;
                let hash_hex = if let Some(arg) = args.first() {
                    match self.eval_expr(arg)? {
                        RuntimeValue::Object { fields, .. } => fields
                            .get("hex")
                            .and_then(|v| match v {
                                RuntimeValue::String { value } => Some(value.clone()),
                                _ => None,
                            })
                            .unwrap_or_default(),
                        RuntimeValue::String { value } => value,
                        _ => String::new(),
                    }
                } else {
                    String::new()
                };
                let hash = crate::audit::Hash(hash_hex);
                let tx = self.mock_ledger.anchor_hash(&hash).map_err(|e| {
                    RuntimeError::new(format!("mock_ledger.anchor failed: {e}"), line).into_spanda()
                })?;
                self.log(format!("mock_ledger.anchor -> {}", tx.0));
                Ok(RuntimeValue::Object {
                    type_name: "TransactionId".into(),
                    fields: HashMap::from([("id".into(), RuntimeValue::String { value: tx.0 })]),
                })
            }
            "verify" => {
                let hash_hex = if let Some(arg) = args.first() {
                    match self.eval_expr(arg)? {
                        RuntimeValue::Object { fields, .. } => fields
                            .get("hex")
                            .and_then(|v| match v {
                                RuntimeValue::String { value } => Some(value.clone()),
                                _ => None,
                            })
                            .unwrap_or_default(),
                        RuntimeValue::String { value } => value,
                        _ => String::new(),
                    }
                } else {
                    String::new()
                };
                let hash = crate::audit::Hash(hash_hex);
                let ok = self.mock_ledger.verify_anchor(&hash).map_err(|e| {
                    RuntimeError::new(format!("mock_ledger.verify failed: {e}"), line).into_spanda()
                })?;
                Ok(RuntimeValue::Bool { value: ok })
            }
            _ => Ok(RuntimeValue::Void),
        }
    }

    fn eval_safety_validate(
        &mut self,
        args: &[Expr],
        named_args: &[crate::ast::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        let arg = if let Some(first) = args.first() {
            self.eval_expr(first)?
        } else {
            self.get_named_arg_value(named_args, "proposal")?
        };
        let proposal = proposal_from_value(&arg).ok_or_else(|| {
            RuntimeError::new("safety.validate() expects ActionProposal", line).into_spanda()
        })?;
        let state = self.backend.get_state();
        let pose2d = Pose2d {
            x: state.pose.x,
            y: state.pose.y,
        };
        let monitor = self.safety_monitor.as_ref().ok_or_else(|| {
            RuntimeError::new("Safety monitor not configured", line).into_spanda()
        })?;
        let result =
            monitor.validate_action_proposal(proposal.linear, proposal.angular, &self.env, &pose2d);
        match result {
            ValidateActionResult::Ok(motion) => {
                self.log("safety.validate() approved ActionProposal".into());
                Ok(safe_action_from_proposal(motion.linear, motion.angular))
            }
            ValidateActionResult::Err { reason } => {
                Err(RuntimeError::new(reason, line).into_spanda())
            }
        }
    }

    fn eval_robot_method(
        &mut self,
        method: &str,
        args: &[Expr],
        _named_args: &[crate::ast::NamedArg],
    ) -> Result<RuntimeValue, SpandaError> {
        let state = self.backend.get_state();
        match method {
            "pose" => Ok(pose_from_state(&state.pose)),
            "velocity" => Ok(velocity_from_state(&state.velocity)),
            "in_zone" => {
                let zone_name = args
                    .first()
                    .map(|e| self.eval_expr(e))
                    .transpose()?
                    .map(|v| get_string(&v, ""))
                    .unwrap_or_default();
                let pose2d = Pose2d {
                    x: state.pose.x,
                    y: state.pose.y,
                };
                let in_zone = self
                    .safety_monitor
                    .as_ref()
                    .map(|m| m.is_in_zone(&zone_name, &pose2d))
                    .unwrap_or(false);
                Ok(RuntimeValue::Bool { value: in_zone })
            }
            "identity" => self
                .env
                .get("identity")
                .cloned()
                .ok_or_else(|| SpandaError::Runtime {
                    message: "robot has no identity — declare an identity block".into(),
                    line: 0,
                }),
            _ => Ok(RuntimeValue::Void),
        }
    }

    fn eval_twin_method(
        &mut self,
        method: &str,
        args: &[Expr],
        named_args: &[crate::ast::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        if self.twin.is_none() {
            return Err(RuntimeError::new("No digital twin configured", line).into_spanda());
        }

        self.refresh_twin_shadow_from_backend();

        match method {
            "frame_count" => {
                let count = self.twin.as_ref().unwrap().replay_frame_count();
                Ok(RuntimeValue::Number {
                    value: count as f64,
                    unit: UnitKind::None,
                })
            }
            "mirror" => {
                let field = self.twin_field_name(args, named_args, line)?;
                self.twin
                    .as_ref()
                    .unwrap()
                    .shadow_field(&field)
                    .cloned()
                    .ok_or_else(|| {
                        RuntimeError::new(
                            format!("Twin has no mirrored shadow field '{field}'"),
                            line,
                        )
                        .into_spanda()
                    })
            }
            "replay" => {
                if !self.twin.as_ref().unwrap().replay {
                    return Err(RuntimeError::new(
                        "Twin replay is disabled — set replay true in twin block",
                        line,
                    )
                    .into_spanda());
                }
                let index =
                    get_number(&self.get_named_arg_value(named_args, "index")?, 0.0) as usize;
                let field = self.twin_field_name(args, named_args, line)?;
                self.twin
                    .as_ref()
                    .unwrap()
                    .replay_field(index, &field)
                    .cloned()
                    .ok_or_else(|| {
                        RuntimeError::new(
                            format!("Twin replay frame {index} has no field '{field}'"),
                            line,
                        )
                        .into_spanda()
                    })
            }
            method => {
                if self
                    .twin
                    .as_ref()
                    .unwrap()
                    .mirrors
                    .iter()
                    .any(|m| m == method)
                {
                    self.twin
                        .as_ref()
                        .unwrap()
                        .shadow_field(method)
                        .cloned()
                        .ok_or_else(|| {
                            RuntimeError::new(
                                format!("Twin shadow field '{method}' not yet mirrored"),
                                line,
                            )
                            .into_spanda()
                        })
                } else {
                    Ok(RuntimeValue::Void)
                }
            }
        }
    }

    fn twin_field_name(
        &mut self,
        args: &[Expr],
        named_args: &[crate::ast::NamedArg],
        line: u32,
    ) -> Result<String, SpandaError> {
        for arg in named_args {
            if arg.name == "field" {
                return self.twin_field_from_expr(&arg.value, line);
            }
        }
        if let Some(arg) = args.first() {
            return self.twin_field_from_expr(arg, line);
        }
        Err(RuntimeError::new("Expected 'field' argument for twin method", line).into_spanda())
    }

    fn twin_field_from_expr(&mut self, expr: &Expr, _line: u32) -> Result<String, SpandaError> {
        match expr {
            Expr::LiteralExpr {
                value: LiteralValue::String(s),
                ..
            } => Ok(s.clone()),
            Expr::IdentExpr { name, .. } => Ok(name.clone()),
            _ => Ok(get_string(&self.eval_expr(expr)?, "")),
        }
    }

    fn execute_actuator_method(
        &mut self,
        name: &str,
        _actuator_type: &str,
        method: &str,
        args: &[Expr],
        named_args: &[crate::ast::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        let motion_methods = [
            "drive",
            "move_to",
            "set_thrust",
            "grip",
            "release",
            "open",
            "hover",
            "follow",
        ];
        if (motion_methods.contains(&method) || method == "stop")
            && !self.check_safety_before_motion()
        {
            if let Some(cb) = &self.options.on_motion_blocked {
                cb("Safety rule triggered — motion blocked".into());
            }
            self.backend.execute_motion(MotionCommand::Stop {
                actuator: name.to_string(),
            });
            return Ok(RuntimeValue::Void);
        }

        match method {
            "stop" => {
                self.backend.execute_motion(MotionCommand::Stop {
                    actuator: name.to_string(),
                });
            }
            "drive" => {
                let linear = get_number(&self.get_named_arg_value(named_args, "linear")?, 0.0);
                let angular = get_number(&self.get_named_arg_value(named_args, "angular")?, 0.0);
                let max_speed = self
                    .safety_monitor
                    .as_ref()
                    .map(|m| m.clamp_speed(linear))
                    .unwrap_or(linear);
                self.backend.execute_motion(MotionCommand::Drive {
                    linear: max_speed,
                    angular,
                    actuator: name.to_string(),
                });
            }
            "follow" => {
                let path_val = self.get_named_arg_value(named_args, "path")?;
                let waypoints = get_trajectory_waypoints(&path_val).unwrap_or_default();
                self.backend.execute_motion(MotionCommand::Follow {
                    waypoints,
                    actuator: name.to_string(),
                });
            }
            "move_to" => {
                let x = get_number(&self.get_named_arg_value(named_args, "x")?, 0.0);
                let y = get_number(&self.get_named_arg_value(named_args, "y")?, 0.0);
                let z = get_number(&self.get_named_arg_value(named_args, "z")?, 0.0);
                self.backend.execute_motion(MotionCommand::MoveTo {
                    x,
                    y,
                    z,
                    actuator: name.to_string(),
                });
            }
            "grip" => {
                self.backend.execute_motion(MotionCommand::Grip {
                    actuator: name.to_string(),
                });
            }
            "release" => {
                self.backend.execute_motion(MotionCommand::Release {
                    actuator: name.to_string(),
                });
            }
            "open" => {
                self.backend.execute_motion(MotionCommand::Open {
                    actuator: name.to_string(),
                });
            }
            "set_thrust" => {
                let thrust = get_number(&self.get_named_arg_value(named_args, "thrust")?, 0.0);
                self.backend.execute_motion(MotionCommand::SetThrust {
                    thrust,
                    actuator: name.to_string(),
                });
            }
            "hover" => {
                self.backend.execute_motion(MotionCommand::Hover {
                    actuator: name.to_string(),
                });
            }
            "execute" => {
                if let Some(agent) = self.current_agent.as_deref() {
                    self.check_agent_capability(agent, "propose_motion", None, line)?;
                }
                let action_val = if let Some(first) = args.first() {
                    self.eval_expr(first)?
                } else {
                    self.get_named_arg_value(named_args, "action")?
                };
                if !is_safe_action(&action_val) {
                    if is_action_proposal(&action_val) {
                        return Err(RuntimeError::new(
                            "Unsafe AI action: ActionProposal cannot execute actuators — call safety.validate() first",
                            line,
                        )
                        .into_spanda());
                    }
                    return Err(RuntimeError::new(
                        "Actuator execute() requires SafeAction from safety.validate()",
                        line,
                    )
                    .into_spanda());
                }
                if !self.check_safety_before_motion() {
                    if let Some(cb) = &self.options.on_motion_blocked {
                        cb("Safety rule triggered — motion blocked".into());
                    }
                    self.backend.execute_motion(MotionCommand::Stop {
                        actuator: name.to_string(),
                    });
                    return Ok(RuntimeValue::Void);
                }
                if let RuntimeValue::SafeAction { linear, angular } = action_val {
                    self.backend.execute_motion(MotionCommand::Drive {
                        linear,
                        angular,
                        actuator: name.to_string(),
                    });
                }
            }
            _ => {}
        }

        self.log(format!("{name}.{method}()"));
        Ok(RuntimeValue::Void)
    }

    fn get_named_arg_value(
        &mut self,
        named_args: &[crate::ast::NamedArg],
        name: &str,
    ) -> Result<RuntimeValue, SpandaError> {
        for arg in named_args {
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
                if matches!(op, BinaryOp::Eq | BinaryOp::Neq)
                    && matches!(left, RuntimeValue::Enum { .. })
                    && matches!(right, RuntimeValue::Enum { .. })
                {
                    let RuntimeValue::Enum {
                        enum_name: e1,
                        variant: v1,
                        payloads: p1,
                    } = left
                    else {
                        unreachable!()
                    };
                    let RuntimeValue::Enum {
                        enum_name: e2,
                        variant: v2,
                        payloads: p2,
                    } = right
                    else {
                        unreachable!()
                    };
                    let equal = e1 == e2 && v1 == v2 && p1 == p2;
                    return Ok(RuntimeValue::Bool {
                        value: if op == BinaryOp::Eq { equal } else { !equal },
                    });
                }
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
        if let Some(cb) = &self.options.on_log {
            cb(message);
        }
    }
}

pub fn runtime_pose(x: f64, y: f64, theta: f64, z: f64) -> RuntimeValue {
    RuntimeValue::Pose { x, y, theta, z }
}

pub fn runtime_velocity(linear: f64, angular: f64) -> RuntimeValue {
    RuntimeValue::Velocity { linear, angular }
}

pub fn runtime_trajectory(waypoints: Vec<PoseValue>) -> RuntimeValue {
    RuntimeValue::Trajectory { waypoints }
}

pub fn pose_from_state(state: &PoseState) -> RuntimeValue {
    runtime_pose(state.x, state.y, state.theta, state.z.unwrap_or(0.0))
}

pub fn velocity_from_state(state: &VelocityState) -> RuntimeValue {
    runtime_velocity(state.linear, state.angular)
}

pub fn get_pose_fields(val: &RuntimeValue) -> Option<PoseValue> {
    match val {
        RuntimeValue::Pose { x, y, theta, z } => Some(PoseValue {
            x: *x,
            y: *y,
            theta: *theta,
            z: *z,
        }),
        _ => None,
    }
}

pub fn get_velocity_fields(val: &RuntimeValue) -> Option<(f64, f64)> {
    match val {
        RuntimeValue::Velocity { linear, angular } => Some((*linear, *angular)),
        _ => None,
    }
}

pub fn get_trajectory_waypoints(val: &RuntimeValue) -> Option<Vec<PoseValue>> {
    match val {
        RuntimeValue::Trajectory { waypoints } => Some(waypoints.clone()),
        _ => None,
    }
}

pub fn get_number(val: &RuntimeValue, default: f64) -> f64 {
    val.as_number().unwrap_or(default)
}

pub fn get_string(val: &RuntimeValue, default: &str) -> String {
    val.as_string().unwrap_or(default).to_string()
}

fn pose_value_to_state(pose: &PoseValue) -> PoseState {
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
    interval_ms: f64,
    next_due_ms: f64,
    body: Vec<Stmt>,
    requires: Option<Expr>,
    ensures: Option<Expr>,
    invariant: Option<Expr>,
}

trait RobotDeclExt {
    fn first_behavior_name(&self) -> Option<String>;
    fn behavior_with_contracts(&self, name: &str) -> Option<BehaviorContracts>;
    fn task_with_contracts(&self, name: &str) -> Option<TaskContracts>;
    fn all_task_schedules(&self) -> Vec<TaskSchedule>;
}

impl RobotDeclExt for RobotDecl {
    fn first_behavior_name(&self) -> Option<String> {
        let RobotDecl::RobotDecl {
            behaviors, tasks, ..
        } = self;
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
        let RobotDecl::RobotDecl { tasks, .. } = self;
        tasks.iter().find_map(|t| match t {
            TaskDecl::TaskDecl {
                name: n,
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
        let RobotDecl::RobotDecl { tasks, .. } = self;
        tasks
            .iter()
            .map(|t| match t {
                TaskDecl::TaskDecl {
                    name,
                    interval_ms,
                    requires,
                    ensures,
                    invariant,
                    body,
                    ..
                } => TaskSchedule {
                    name: name.clone(),
                    interval_ms: *interval_ms,
                    next_due_ms: 0.0,
                    body: body.clone(),
                    requires: requires.clone(),
                    ensures: ensures.clone(),
                    invariant: invariant.clone(),
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
        match self {
            crate::ast::SafetyBlock::SafetyBlock { rules, .. } => rules,
        }
    }

    fn zones(&self) -> &[SafetyZoneDecl] {
        match self {
            crate::ast::SafetyBlock::SafetyBlock { zones, .. } => zones,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulator::{create_default_simulator, Obstacle, SimulatorConfig};

    fn compile_and_run(source: &str, max_iters: usize) -> Result<RobotState, SpandaError> {
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
