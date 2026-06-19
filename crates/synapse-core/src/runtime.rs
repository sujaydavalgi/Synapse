use crate::ai::{
    create_agent_runtime, create_ai_model, execute_agent_plan, is_action_proposal, is_safe_action,
    mock_analyze_frame, mock_camera_frame, proposal_from_value, safe_action_from_proposal,
    AgentRuntime, AiModel, MemoryStore, PlanExecutor,
};
use crate::ast::{
    ActuatorDecl, ActionDecl, AgentDecl, BehaviorDecl, BinaryOp, Expr, LiteralValue,
    Program, RobotDecl, SafetyRule, SafetyZoneDecl, SensorBinding, SensorDecl, ServiceDecl, Stmt,
    TopicDecl, UnaryOp, UnitKind, ZoneShape,
};
use crate::error::{PoseState, RobotState, SynapseError, VelocityState};
use crate::hal::{create_sim_hal, hal_member_from_decl, HalBackend, SimHalBackend};
use crate::lib_registry::{get_sensor_driver, read_with_driver, DriverContext, SimState};
use crate::safety::{
    create_safety_config_from_robot, interpolate_poses, Pose2d, SafetyMonitor, SafetyZoneRuntime,
    SafetyZoneShape, ValidateActionResult,
};
use crate::soc::get_soc_profile;
#[cfg(test)]
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

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
    Number { value: f64, unit: UnitKind },
    Bool { value: bool },
    String { value: String },
    Void,
    Scan { nearest_distance: f64 },
    Pose {
        x: f64,
        y: f64,
        theta: f64,
        z: f64,
    },
    Velocity { linear: f64, angular: f64 },
    Trajectory { waypoints: Vec<PoseValue> },
    Transform {
        from_frame: String,
        to_frame: String,
        pose: PoseValue,
    },
    Object {
        type_name: String,
        fields: HashMap<String, RuntimeValue>,
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
    Agent { name: String },
    SafetyCtx,
    AiModel {
        name: String,
        model_type: String,
        provider: String,
    },
    ActionProposal {
        linear: f64,
        angular: f64,
        source: String,
    },
    SafeAction { linear: f64, angular: f64 },
    Completion { text: String, model: Option<String> },
    Embedding { dimensions: usize, vector: Vec<f64> },
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
    Stop { actuator: String },
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
    Grip { actuator: String },
    Release { actuator: String },
    Open { actuator: String },
    SetThrust { thrust: f64, actuator: String },
    Hover { actuator: String },
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

    pub fn into_synapse(self) -> SynapseError {
        SynapseError::Runtime {
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
}

impl Default for InterpreterOptions {
    fn default() -> Self {
        Self {
            max_loop_iterations: 10,
            on_motion_blocked: None,
            on_log: None,
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
    stop_if_conditions: Vec<Expr>,
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
            stop_if_conditions: Vec::new(),
        }
    }

    pub fn robot_backend(&self) -> &B {
        &self.backend
    }

    pub fn run(
        &mut self,
        program: &Program,
        entry_behavior: Option<&str>,
    ) -> Result<RobotState, SynapseError> {
        let Program::Program { robots, .. } = program;
        for robot in robots {
            self.setup_robot(robot)?;
            let behavior_name = entry_behavior
                .map(String::from)
                .or_else(|| robot.first_behavior_name());
            if let Some(name) = behavior_name {
                if let Some(body) = robot.behavior_body(&name) {
                    self.execute_block(&body)?;
                }
            }
        }
        Ok(self.backend.get_state())
    }

    fn setup_robot(&mut self, robot: &RobotDecl) -> Result<(), SynapseError> {
        let RobotDecl::RobotDecl {
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
            ..
        } = robot;

        self.env = Environment::new();
        self.zones.clear();
        self.stop_if_conditions.clear();

        if let Some(soc_decl) = soc {
            let profile_name = soc_decl.profile();
            if let Some(profile) = get_soc_profile(profile_name) {
                self.log(format!(
                    "SoC: {} ({})",
                    profile.name, profile.architecture
                ));
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
            self.env
                .define(name.clone(), model.to_runtime_value());
            self.log(format!(
                "AI model '{}': {} ({}/{})",
                name, model.model_type, model.config.provider, model.config.model
            ));
            self.ai_models.insert(name, model);
        }

        for agent_decl in agents {
            self.setup_agent(agent_decl);
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

    fn eval_safety_zone(&mut self, zone: &SafetyZoneDecl) -> Result<SafetyZoneRuntime, SynapseError> {
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
            ..
        } = topic;
        self.env.define(
            name.clone(),
            RuntimeValue::Topic {
                name: name.clone(),
                message_type: message_type.clone(),
                topic_path: topic_path.clone(),
            },
        );
    }

    fn define_service(&mut self, service: &ServiceDecl) {
        let ServiceDecl::ServiceDecl {
            name,
            service_type,
            ..
        } = service;
        self.env.define(
            name.clone(),
            RuntimeValue::Service {
                name: name.clone(),
                service_type: service_type.clone(),
            },
        );
    }

    fn define_action(&mut self, action: &ActionDecl) {
        let ActionDecl::ActionDecl {
            name,
            action_type,
            ..
        } = action;
        self.env.define(
            name.clone(),
            RuntimeValue::Action {
                name: name.clone(),
                action_type: action_type.clone(),
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
            ..
        } = agent_decl;
        let memory = memory_kind.map(|k| MemoryStore::new(k.into(), None));
        let agent = create_agent_runtime(agent_decl.clone(), memory);
        self.agents.insert(name.clone(), agent);
        self.env
            .define(name.clone(), RuntimeValue::Agent { name: name.clone() });
        self.log(format!("Agent '{name}': {goal}"));
    }

    fn execute_block(&mut self, stmts: &[Stmt]) -> Result<(), SynapseError> {
        for stmt in stmts {
            self.execute_stmt(stmt)?;
        }
        Ok(())
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> Result<(), SynapseError> {
        match stmt {
            Stmt::VarDecl { name, init, .. } => {
                let value = self.eval_expr(init)?;
                self.env.define(name.clone(), value);
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
            Stmt::LoopStmt { interval_ms, body, .. } => {
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
                let topic = self.env.get(topic_name).cloned();
                let val = self.eval_expr(value)?;
                if let Some(RuntimeValue::Topic {
                    topic_path,
                    message_type,
                    ..
                }) = topic
                {
                    self.backend
                        .publish_topic(&topic_path, &message_type, val);
                    self.log(format!("publish {topic_path}"));
                }
                let _ = span;
            }
            Stmt::ServiceCallStmt { service_name, .. } => {
                if let Some(RuntimeValue::Service {
                    name,
                    service_type,
                }) = self.env.get(service_name).cloned()
                {
                    self.backend.call_service(&name, &service_type);
                    self.log(format!("call {name}()"));
                }
            }
            Stmt::ActionSendStmt {
                action_name,
                goal,
                ..
            } => {
                if let Some(RuntimeValue::Action {
                    name,
                    action_type,
                }) = self.env.get(action_name).cloned()
                {
                    let goal_val = self.eval_expr(goal)?;
                    self.backend.send_action(&name, &action_type, goal_val);
                    self.log(format!("send_goal {name}"));
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
            Stmt::ExprStmt { expr, .. } => {
                self.eval_expr(expr)?;
            }
            Stmt::ReturnStmt { .. } => {}
        }
        Ok(())
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<RuntimeValue, SynapseError> {
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
            Expr::IdentExpr { name, span } => self.env.get(name).cloned().ok_or_else(|| {
                RuntimeError::new(format!("Undefined variable '{name}'"), span.start.line).into_synapse()
            }),
            Expr::BinaryExpr { op, left, right, span } => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;
                self.eval_binary(*op, left_val, right_val, span.start.line)
            }
            Expr::UnaryExpr { op, operand, span: _ } => {
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
            Expr::MemberExpr { object, property, .. } => {
                let obj = self.eval_expr(object)?;
                self.eval_member(&obj, property)
            }
            Expr::CallExpr { callee, args, named_args, span } => {
                self.eval_call(callee, args, named_args, span.start.line)
            }
        }
    }

    fn eval_member(&mut self, obj: &RuntimeValue, property: &str) -> Result<RuntimeValue, SynapseError> {
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
            RuntimeValue::ActionProposal { linear, angular, .. }
            | RuntimeValue::SafeAction { linear, angular } => match property {
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
            RuntimeValue::Completion { text, .. } if property == "text" => {
                Ok(RuntimeValue::String { value: text.clone() })
            }
            RuntimeValue::Object { fields, .. } => Ok(fields.get(property).cloned().unwrap_or(RuntimeValue::Void)),
            _ => Ok(RuntimeValue::Void),
        }
    }

    fn eval_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        named_args: &[crate::ast::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SynapseError> {
        if let Expr::IdentExpr { name, .. } = callee {
            return self.eval_builtin_function(name, args, named_args);
        }

        let Expr::MemberExpr { object, property, .. } = callee else {
            return Ok(RuntimeValue::Void);
        };
        let Expr::IdentExpr { name: target_name, .. } = object.as_ref() else {
            return Ok(RuntimeValue::Void);
        };

        let target = self
            .env
            .get(target_name)
            .cloned()
            .ok_or_else(|| RuntimeError::new(format!("Undefined '{target_name}'"), line).into_synapse())?;

        if matches!(target, RuntimeValue::Robot) || target_name == "robot" {
            return self.eval_robot_method(property, args, named_args);
        }

        if let RuntimeValue::Sensor { sensor_type, .. } = &target {
            if property == "read" {
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

        if let RuntimeValue::Agent { name } = &target {
            if property == "plan" {
                let agent = self.agents.get(name).ok_or_else(|| {
                    RuntimeError::new(format!("Unknown agent '{name}'"), line).into_synapse()
                })?;
                let agent = agent.clone();
                struct PlanRunner<'a, B: RobotBackend> {
                    interp: &'a mut Interpreter<B>,
                }
                impl<B: RobotBackend> PlanExecutor for PlanRunner<'_, B> {
                    fn execute_block(&mut self, stmts: &[Stmt]) {
                        let _ = self.interp.execute_block(stmts);
                    }
                }
                let mut runner = PlanRunner { interp: self };
                execute_agent_plan(&agent, &mut runner);
                self.log(format!("agent {name}.plan()"));
                return Ok(RuntimeValue::Void);
            }
        }

        if matches!(target, RuntimeValue::SafetyCtx) && property == "validate" {
            return self.eval_safety_validate(args, named_args, line);
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
            return self.execute_actuator_method(&name, &actuator_type, property, args, named_args, line);
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
    ) -> Result<RuntimeValue, SynapseError> {
        match method {
            "reason" => {
                let prompt = get_string(&self.get_named_arg_value(named_args, "prompt")?, "");
                let input = self.get_named_arg_value(named_args, "input")?;
                let input = if matches!(input, RuntimeValue::Void) {
                    None
                } else {
                    Some(input)
                };
                let result = self
                    .ai_models
                    .get(target_name)
                    .ok_or_else(|| {
                        RuntimeError::new(format!("Unknown AI model '{target_name}'"), line)
                            .into_synapse()
                    })?
                    .reason(&prompt, input)
                    .map_err(|message| SynapseError::Runtime { message, line })?;
                self.log(format!("ai {target_name}.reason() -> ActionProposal"));
                Ok(result)
            }
            "summarize" => {
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
                            .into_synapse()
                    })?
                    .summarize(input)
                    .map_err(|message| SynapseError::Runtime { message, line })
            }
            "detect" => {
                let frame = if let Some(first) = args.first() {
                    self.eval_expr(first)?
                } else {
                    self.get_named_arg_value(named_args, "frame")?
                };
                self.ai_models
                    .get(target_name)
                    .ok_or_else(|| {
                        RuntimeError::new(format!("Unknown AI model '{target_name}'"), line)
                            .into_synapse()
                    })?
                    .detect(frame)
                    .map_err(|message| SynapseError::Runtime { message, line })
            }
            "drive" => Err(RuntimeError::new(
                "Unsafe AI action: LLM cannot drive actuators directly — use safety.validate() then wheels.execute()",
                line,
            )
            .into_synapse()),
            _ => Ok(RuntimeValue::Void),
        }
    }

    fn read_sensor_value(&mut self, target: &RuntimeValue) -> Result<RuntimeValue, SynapseError> {
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
        Ok(self.backend.read_sensor(name, sensor_type, topic.as_deref()))
    }

    fn eval_builtin_function(
        &mut self,
        name: &str,
        _args: &[Expr],
        named_args: &[crate::ast::NamedArg],
    ) -> Result<RuntimeValue, SynapseError> {
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
            _ => Ok(RuntimeValue::Void),
        }
    }

    fn eval_safety_validate(
        &mut self,
        args: &[Expr],
        named_args: &[crate::ast::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SynapseError> {
        let arg = if let Some(first) = args.first() {
            self.eval_expr(first)?
        } else {
            self.get_named_arg_value(named_args, "proposal")?
        };
        let proposal = proposal_from_value(&arg).ok_or_else(|| {
            RuntimeError::new("safety.validate() expects ActionProposal", line).into_synapse()
        })?;
        let state = self.backend.get_state();
        let pose2d = Pose2d {
            x: state.pose.x,
            y: state.pose.y,
        };
        let monitor = self.safety_monitor.as_ref().ok_or_else(|| {
            RuntimeError::new("Safety monitor not configured", line).into_synapse()
        })?;
        let result = monitor.validate_action_proposal(
            proposal.linear,
            proposal.angular,
            &self.env,
            &pose2d,
        );
        match result {
            ValidateActionResult::Ok(motion) => {
                self.log("safety.validate() approved ActionProposal".into());
                Ok(safe_action_from_proposal(motion.linear, motion.angular))
            }
            ValidateActionResult::Err { reason } => {
                Err(RuntimeError::new(reason, line).into_synapse())
            }
        }
    }

    fn eval_robot_method(
        &mut self,
        method: &str,
        args: &[Expr],
        _named_args: &[crate::ast::NamedArg],
    ) -> Result<RuntimeValue, SynapseError> {
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
            _ => Ok(RuntimeValue::Void),
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
    ) -> Result<RuntimeValue, SynapseError> {
        let motion_methods = [
            "drive", "move_to", "set_thrust", "grip", "release", "open", "hover", "follow",
        ];
        if motion_methods.contains(&method) || method == "stop" {
            if !self.check_safety_before_motion() {
                if let Some(cb) = &self.options.on_motion_blocked {
                    cb("Safety rule triggered — motion blocked".into());
                }
                self.backend.execute_motion(MotionCommand::Stop {
                    actuator: name.to_string(),
                });
                return Ok(RuntimeValue::Void);
            }
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
                self.backend
                    .execute_motion(MotionCommand::Grip { actuator: name.to_string() });
            }
            "release" => {
                self.backend.execute_motion(MotionCommand::Release {
                    actuator: name.to_string(),
                });
            }
            "open" => {
                self.backend
                    .execute_motion(MotionCommand::Open { actuator: name.to_string() });
            }
            "set_thrust" => {
                let thrust = get_number(&self.get_named_arg_value(named_args, "thrust")?, 0.0);
                self.backend.execute_motion(MotionCommand::SetThrust {
                    thrust,
                    actuator: name.to_string(),
                });
            }
            "hover" => {
                self.backend
                    .execute_motion(MotionCommand::Hover { actuator: name.to_string() });
            }
            "execute" => {
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
                        .into_synapse());
                    }
                    return Err(RuntimeError::new(
                        "Actuator execute() requires SafeAction from safety.validate()",
                        line,
                    )
                    .into_synapse());
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
    ) -> Result<RuntimeValue, SynapseError> {
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
    ) -> Result<RuntimeValue, SynapseError> {
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
                    && matches!(left, RuntimeValue::Bool { .. })
                    && matches!(right, RuntimeValue::Bool { .. })
                {
                    let RuntimeValue::Bool { value: l, .. } = left else { unreachable!() };
                    let RuntimeValue::Bool { value: r, .. } = right else { unreachable!() };
                    return Ok(RuntimeValue::Bool {
                        value: if op == BinaryOp::Eq { l == r } else { l != r },
                    });
                }
                if let (
                    RuntimeValue::Number {
                        value: l,
                        unit: lu,
                    },
                    RuntimeValue::Number {
                        value: r,
                        unit: _ru,
                    },
                ) = (left, right)
                {
                    return Ok(match op {
                        BinaryOp::Add => RuntimeValue::Number {
                            value: l + r,
                            unit: lu,
                        },
                        BinaryOp::Sub => RuntimeValue::Number {
                            value: l - r,
                            unit: lu,
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
    runtime_pose(
        state.x,
        state.y,
        state.theta,
        state.z.unwrap_or(0.0),
    )
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
trait RobotDeclExt {
    fn first_behavior_name(&self) -> Option<String>;
    fn behavior_body(&self, name: &str) -> Option<Vec<Stmt>>;
}

impl RobotDeclExt for RobotDecl {
    fn first_behavior_name(&self) -> Option<String> {
        let RobotDecl::RobotDecl { behaviors, .. } = self;
        behaviors.first().map(|b| match b {
            BehaviorDecl::BehaviorDecl { name, .. } => name.clone(),
        })
    }

    fn behavior_body(&self, name: &str) -> Option<Vec<Stmt>> {
        let RobotDecl::RobotDecl { behaviors, .. } = self;
        behaviors.iter().find_map(|b| match b {
            BehaviorDecl::BehaviorDecl { name: n, body, .. } if n == name => Some(body.clone()),
            _ => None,
        })
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
    use crate::simulator::{create_default_simulator, SimulatorConfig, Obstacle};

    fn compile_and_run(source: &str, max_iters: usize) -> Result<RobotState, SynapseError> {
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
