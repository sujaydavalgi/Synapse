use crate::ai::resolve_ai_import;
use crate::ast::*;
use crate::error::{Diagnostic, SynapseError};
use crate::hal::hal_member_from_decl;
use crate::lib_registry::{all_library_sensor_types, resolve_import};
use crate::soc::{get_soc_profile, validate_hal_against_soc};
use std::collections::HashMap;

pub fn type_check(program: &Program) -> Result<(), SynapseError> {
    check(program)
}

pub fn check(program: &Program) -> Result<(), SynapseError> {
    let mut checker = TypeChecker::new();
    checker.check_program(program);
    if checker.errors.is_empty() {
        Ok(())
    } else {
        Err(SynapseError::TypeCheck {
            diagnostics: checker.errors,
        })
    }
}

pub fn units_compatible(a: UnitKind, b: UnitKind) -> bool {
    if a == b {
        return true;
    }
    if a == UnitKind::None || b == UnitKind::None {
        return true;
    }
    matches!(
        (a, b),
        (UnitKind::Deg, UnitKind::Rad) | (UnitKind::Rad, UnitKind::Deg)
    )
}

pub fn result_unit_for_binary(op: BinaryOp, left: &SynapseType, right: &SynapseType) -> Option<SynapseType> {
    match op {
        BinaryOp::And | BinaryOp::Or => {
            if matches!(left, SynapseType::Bool) && matches!(right, SynapseType::Bool) {
                Some(SynapseType::Bool)
            } else {
                None
            }
        }
        BinaryOp::Lt | BinaryOp::Lte | BinaryOp::Gt | BinaryOp::Gte | BinaryOp::Eq | BinaryOp::Neq => {
            if matches!(left, SynapseType::Number { .. }) && matches!(right, SynapseType::Number { .. }) {
                let SynapseType::Number { unit: lu, .. } = left else { unreachable!() };
                let SynapseType::Number { unit: ru, .. } = right else { unreachable!() };
                if units_compatible(*lu, *ru) {
                    return Some(SynapseType::Bool);
                }
            }
            if matches!(left, SynapseType::Bool) && matches!(right, SynapseType::Bool) {
                return Some(SynapseType::Bool);
            }
            if matches!(left, SynapseType::String) && matches!(right, SynapseType::String) {
                return Some(SynapseType::Bool);
            }
            None
        }
        BinaryOp::Add | BinaryOp::Sub => {
            if let (SynapseType::Number { unit: lu, .. }, SynapseType::Number { unit: ru, .. }) =
                (left, right)
            {
                if units_compatible(*lu, *ru) {
                    let unit = if *lu != UnitKind::None { *lu } else { *ru };
                    return Some(SynapseType::Number { unit });
                }
            }
            None
        }
        BinaryOp::Mul | BinaryOp::Div => {
            if matches!(left, SynapseType::Number { .. }) && matches!(right, SynapseType::Number { .. }) {
                Some(SynapseType::Number {
                    unit: UnitKind::None,
                })
            } else {
                None
            }
        }
    }
}

pub struct MethodSig {
    params: Vec<SynapseType>,
    named_params: HashMap<String, SynapseType>,
    returns: SynapseType,
}

#[derive(Clone)]
struct SymbolEntry {
    robo_type: SynapseType,
    kind: SymbolKind,
    sensor_type: Option<String>,
    actuator_type: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SymbolKind {
    Sensor,
    Actuator,
    Variable,
    Behavior,
    Topic,
    Service,
    Action,
    Robot,
    AiModel,
    Agent,
    Safety,
}

pub struct TypeChecker {
    pub errors: Vec<Diagnostic>,
    symbols: HashMap<String, SymbolEntry>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            symbols: HashMap::new(),
        }
    }

    pub fn check_program(&mut self, program: &Program) {
        let Program::Program { imports, robots, .. } = program;
        let mut imported = std::collections::HashSet::new();
        for imp in imports {
            let ImportDecl::ImportDecl { path, span } = imp;
            if resolve_import(path).is_none() && resolve_ai_import(path).is_none() {
                self.error(
                    format!("Unknown library '{path}'"),
                    span.start.line,
                    span.start.column,
                );
            } else {
                imported.insert(path.clone());
            }
        }

        for robot in robots {
            self.check_robot(robot, &imported);
        }
    }

    fn check_robot(&mut self, robot: &RobotDecl, imported: &std::collections::HashSet<String>) {
        let RobotDecl::RobotDecl {
            soc,
            hal,
            nodes,
            topics,
            services,
            actions,
            sensors,
            actuators,
            safety,
            ai_models,
            agents,
            behaviors,
            ..
        } = robot;

        self.symbols.clear();

        if let Some(soc_decl) = soc {
            let SocDecl::SocDecl { profile, span } = soc_decl;
            if get_soc_profile(profile).is_none() {
                self.error(
                    format!("Unknown SoC profile '{profile}'"),
                    span.start.line,
                    span.start.column,
                );
            }
        }

        if let (Some(hal_block), Some(soc_decl)) = (hal, soc) {
            let HalBlock::HalBlock { members, span, .. } = hal_block;
            let SocDecl::SocDecl { profile, .. } = soc_decl;
            {
                if let Some(profile) = get_soc_profile(profile) {
                    let hal_members: Vec<_> = members.iter().map(hal_member_from_decl).collect();
                    for err in validate_hal_against_soc(&profile, &hal_members) {
                        self.error(err.message, span.start.line, span.start.column);
                    }
                }
            }
        }

        let hal_bus_names: std::collections::HashSet<String> = hal
            .as_ref()
            .map(|h| match h {
                HalBlock::HalBlock { members, .. } => {
                    members.iter().map(|m| m.name().to_string()).collect()
                }
            })
            .unwrap_or_default();

        for node in nodes {
            let NodeDecl::NodeDecl { namespace, span, .. } = node;
            if namespace.is_none() {
                self.error(
                    "Node should specify namespace with 'on \"/namespace\"'".into(),
                    span.start.line,
                    span.start.column,
                );
            }
        }

        for topic in topics {
            self.check_topic(topic);
        }
        for service in services {
            self.check_service(service);
        }
        for action in actions {
            self.check_action(action);
        }
        for sensor in sensors {
            self.check_sensor(sensor, imported, &hal_bus_names);
        }
        for actuator in actuators {
            self.check_actuator(actuator);
        }

        if let Some(safety_block) = safety {
            let saved = self.symbols.clone();
            self.symbols.insert(
                "robot".into(),
                SymbolEntry {
                    robo_type: SynapseType::Named {
                        name: "Robot".into(),
                    },
                    kind: SymbolKind::Robot,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
            for rule in safety_block.rules() {
                self.check_safety_rule(rule);
            }
            for zone in safety_block.zones() {
                self.check_safety_zone(zone);
            }
            self.symbols = saved;
            self.symbols.insert(
                "safety".into(),
                SymbolEntry {
                    robo_type: SynapseType::Named {
                        name: "Safety".into(),
                    },
                    kind: SymbolKind::Safety,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
        }

        for model in ai_models {
            self.check_ai_model(model);
        }
        for agent in agents {
            self.check_agent(agent);
        }

        for behavior in behaviors {
            let BehaviorDecl::BehaviorDecl { name, body, .. } = behavior;
            self.symbols.insert(
                name.clone(),
                SymbolEntry {
                    robo_type: SynapseType::Void,
                    kind: SymbolKind::Behavior,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
            self.check_behavior(body);
        }
    }

    fn check_topic(&mut self, topic: &TopicDecl) {
        let TopicDecl::TopicDecl {
            name,
            message_type,
            span,
            ..
        } = topic;
            if message_type_for(message_type).is_none() {
                self.error(
                    format!("Unknown message type '{message_type}'"),
                    span.start.line,
                    span.start.column,
                );
            }
            self.symbols.insert(
                name.clone(),
                SymbolEntry {
                    robo_type: message_type_for(message_type).unwrap_or(SynapseType::Void),
                    kind: SymbolKind::Topic,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
    }

    fn check_service(&mut self, service: &ServiceDecl) {
        let ServiceDecl::ServiceDecl {
            name,
            service_type,
            span,
            ..
        } = service;
            if service_type_for(service_type).is_none() {
                self.error(
                    format!("Unknown service type '{service_type}'"),
                    span.start.line,
                    span.start.column,
                );
            }
            self.symbols.insert(
                name.clone(),
                SymbolEntry {
                    robo_type: service_type_for(service_type).unwrap_or(SynapseType::Void),
                    kind: SymbolKind::Service,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
    }

    fn check_action(&mut self, action: &ActionDecl) {
        let ActionDecl::ActionDecl {
            name,
            action_type,
            span,
            ..
        } = action;
            if action_type_for(action_type).is_none() {
                self.error(
                    format!("Unknown action type '{action_type}'"),
                    span.start.line,
                    span.start.column,
                );
            }
            self.symbols.insert(
                name.clone(),
                SymbolEntry {
                    robo_type: action_type_for(action_type).unwrap_or(SynapseType::Void),
                    kind: SymbolKind::Action,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
    }

    fn check_sensor(
        &mut self,
        sensor: &SensorDecl,
        imported: &std::collections::HashSet<String>,
        hal_bus_names: &std::collections::HashSet<String>,
    ) {
        let SensorDecl::SensorDecl {
            name,
            sensor_type,
            library,
            binding,
            span,
            ..
        } = sensor;
            if sensor_type_for(sensor_type).is_none() {
                self.error(
                    format!("Unknown sensor type '{sensor_type}'"),
                    span.start.line,
                    span.start.column,
                );
            }
            if let Some(lib) = library {
                if !imported.contains(lib) {
                    self.error(
                        format!("Library '{lib}' must be imported before use"),
                        span.start.line,
                        span.start.column,
                    );
                }
                if let Some(module) = resolve_import(lib) {
                    if !module.sensors.contains_key(sensor_type) {
                        self.error(
                            format!("Sensor type '{sensor_type}' not provided by library '{lib}'"),
                            span.start.line,
                            span.start.column,
                        );
                    }
                }
            }
            if let Some(SensorBinding::Hal { bus_name }) = binding {
                if !hal_bus_names.contains(bus_name) {
                    self.error(
                        format!("Unknown HAL bus '{bus_name}'"),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
            self.symbols.insert(
                name.clone(),
                SymbolEntry {
                    robo_type: sensor_type_for(sensor_type).unwrap_or(SynapseType::Named {
                        name: sensor_type.clone(),
                    }),
                    kind: SymbolKind::Sensor,
                    sensor_type: Some(sensor_type.clone()),
                    actuator_type: None,
                },
            );
    }

    fn check_actuator(&mut self, actuator: &ActuatorDecl) {
        let ActuatorDecl::ActuatorDecl {
            name,
            actuator_type,
            span,
            ..
        } = actuator;
            if actuator_type_for(actuator_type).is_none() {
                self.error(
                    format!("Unknown actuator type '{actuator_type}'"),
                    span.start.line,
                    span.start.column,
                );
            }
            self.symbols.insert(
                name.clone(),
                SymbolEntry {
                    robo_type: actuator_type_for(actuator_type).unwrap_or(SynapseType::Named {
                        name: actuator_type.clone(),
                    }),
                    kind: SymbolKind::Actuator,
                    sensor_type: None,
                    actuator_type: Some(actuator_type.clone()),
                },
            );
    }

    fn check_safety_rule(&mut self, rule: &SafetyRule) {
        match rule {
            SafetyRule::MaxSpeedRule { value, unit, span, .. } => {
                let t = self.check_expr(value);
                if !matches!(t, SynapseType::Number { .. }) || !units_compatible(t.unit(), *unit) {
                    self.error(
                        format!("Expected value with unit '{}' for max_speed", unit.as_str()),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
            SafetyRule::StopIfRule { condition, span } => {
                if !matches!(self.check_expr(condition), SynapseType::Bool) {
                    self.error(
                        "stop_if condition must be boolean".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
        }
    }

    fn check_safety_zone(&mut self, zone: &SafetyZoneDecl) {
        let SafetyZoneDecl::SafetyZoneDecl {
            x,
            y,
            radius,
            width,
            height,
            shape,
            span,
            ..
        } = zone;
            if !matches!(self.check_expr(x), SynapseType::Number { .. })
                || !matches!(self.check_expr(y), SynapseType::Number { .. })
            {
                self.error(
                    "Zone coordinates must be numeric".into(),
                    span.start.line,
                    span.start.column,
                );
            }
            if *shape == ZoneShape::Circle {
                if let Some(r) = radius {
                    if !matches!(self.check_expr(r), SynapseType::Number { .. }) {
                        self.error(
                            "Zone radius must be numeric".into(),
                            span.start.line,
                            span.start.column,
                        );
                    }
                }
            }
            if *shape == ZoneShape::Rect {
                if let Some(w) = width {
                    if !matches!(self.check_expr(w), SynapseType::Number { .. }) {
                        self.error(
                            "Zone size must be numeric".into(),
                            span.start.line,
                            span.start.column,
                        );
                    }
                }
                if let Some(h) = height {
                    if !matches!(self.check_expr(h), SynapseType::Number { .. }) {
                        self.error(
                            "Zone size must be numeric".into(),
                            span.start.line,
                            span.start.column,
                        );
                    }
                }
            }
    }

    fn check_ai_model(&mut self, model: &AiModelDecl) {
        let AiModelDecl::AiModelDecl {
            name,
            model_type,
            span,
            ..
        } = model;
            if ai_model_type_for(model_type).is_none() {
                self.error(
                    format!("Unknown AI model type '{model_type}'"),
                    span.start.line,
                    span.start.column,
                );
            }
            if self.symbols.contains_key(name) {
                self.error(
                    format!("Duplicate ai model name '{name}'"),
                    span.start.line,
                    span.start.column,
                );
            }
            self.symbols.insert(
                name.clone(),
                SymbolEntry {
                    robo_type: ai_model_type_for(model_type).unwrap_or(SynapseType::Void),
                    kind: SymbolKind::AiModel,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
    }

    fn check_agent(&mut self, agent: &AgentDecl) {
        let AgentDecl::AgentDecl {
            name,
            uses_ai,
            tools,
            plan_body,
            span,
            ..
        } = agent;
            if self.symbols.contains_key(name) {
                self.error(
                    format!("Duplicate agent name '{name}'"),
                    span.start.line,
                    span.start.column,
                );
            }
            for model_name in uses_ai {
                let sym = self.symbols.get(model_name);
                if sym.map(|s| s.kind) != Some(SymbolKind::AiModel) {
                    self.error(
                        format!("Agent '{name}' references unknown ai model '{model_name}'"),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
            for tool in tools {
                if !self.symbols.contains_key(tool) {
                    self.error(
                        format!("Agent '{name}' references unknown tool '{tool}'"),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
            self.symbols.insert(
                name.clone(),
                SymbolEntry {
                    robo_type: SynapseType::Named {
                        name: "Agent".into(),
                    },
                    kind: SymbolKind::Agent,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
            let saved = self.symbols.clone();
            for stmt in plan_body {
                self.check_stmt(stmt);
            }
            self.symbols = saved;
    }

    fn check_behavior(&mut self, body: &[Stmt]) {
        let parent = self.symbols.clone();
        self.symbols = parent.clone();
        self.symbols.insert(
            "robot".into(),
            SymbolEntry {
                robo_type: SynapseType::Named {
                    name: "Robot".into(),
                },
                kind: SymbolKind::Robot,
                sensor_type: None,
                actuator_type: None,
            },
        );
        for stmt in body {
            self.check_stmt(stmt);
        }
        self.symbols = parent;
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { name, init, span: _ } => {
                let t = self.check_expr(init);
                self.symbols.insert(
                    name.clone(),
                    SymbolEntry {
                        robo_type: t,
                        kind: SymbolKind::Variable,
                        sensor_type: None,
                        actuator_type: None,
                    },
                );
            }
            Stmt::IfStmt {
                condition,
                then_branch,
                else_branch,
                span,
            } => {
                if !matches!(self.check_expr(condition), SynapseType::Bool) {
                    self.error(
                        "if condition must be boolean".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
                for s in then_branch {
                    self.check_stmt(s);
                }
                if let Some(else_branch) = else_branch {
                    for s in else_branch {
                        self.check_stmt(s);
                    }
                }
            }
            Stmt::LoopStmt { body, .. } => {
                for s in body {
                    self.check_stmt(s);
                }
            }
            Stmt::PublishStmt {
                topic_name,
                value,
                span,
            } => {
                if let Some(topic) = self.symbols.get(topic_name).cloned() {
                    if topic.kind != SymbolKind::Topic {
                        self.error(
                            format!("Unknown topic '{topic_name}'"),
                            span.start.line,
                            span.start.column,
                        );
                    } else {
                        let val = self.check_expr(value);
                        self.assert_compatible(
                            &topic.robo_type,
                            &val,
                            span.start.line,
                            span.start.column,
                        );
                    }
                } else {
                    self.error(
                        format!("Unknown topic '{topic_name}'"),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
            Stmt::ServiceCallStmt { service_name, span } => {
                if self
                    .symbols
                    .get(service_name)
                    .map(|s| s.kind)
                    != Some(SymbolKind::Service)
                {
                    self.error(
                        format!("Unknown service '{service_name}'"),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
            Stmt::ActionSendStmt {
                action_name,
                goal,
                span,
            } => {
                if self
                    .symbols
                    .get(action_name)
                    .map(|s| s.kind)
                    != Some(SymbolKind::Action)
                {
                    self.error(
                        format!("Unknown action '{action_name}'"),
                        span.start.line,
                        span.start.column,
                    );
                } else {
                    let goal_t = self.check_expr(goal);
                    if !matches!(goal_t, SynapseType::Pose | SynapseType::Trajectory) {
                        self.error(
                            "Action goal must be pose or trajectory".into(),
                            span.start.line,
                            span.start.column,
                        );
                    }
                }
            }
            Stmt::EmergencyStopStmt { .. } | Stmt::ResetEmergencyStopStmt { .. } => {}
            Stmt::ExprStmt { expr, .. } => {
                self.check_expr(expr);
            }
            Stmt::ReturnStmt { value, .. } => {
                if let Some(v) = value {
                    self.check_expr(v);
                }
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> SynapseType {
        match expr {
            Expr::LiteralExpr { value, .. } => match value {
                LiteralValue::Bool(_) => SynapseType::Bool,
                LiteralValue::Number(_) => SynapseType::Number {
                    unit: UnitKind::None,
                },
                LiteralValue::String(_) => SynapseType::String,
                LiteralValue::Null => SynapseType::Void,
            },
            Expr::UnitLiteralExpr { value: _, unit, .. } => SynapseType::Number { unit: *unit },
            Expr::IdentExpr { name, span } => {
                if let Some(sym) = self.symbols.get(name) {
                    sym.robo_type.clone()
                } else {
                    self.error(
                        format!("Undefined identifier '{name}'"),
                        span.start.line,
                        span.start.column,
                    );
                    SynapseType::Void
                }
            }
            Expr::BinaryExpr { op, left, right, span } => {
                let l = self.check_expr(left);
                let r = self.check_expr(right);
                if let Some(result) = result_unit_for_binary(*op, &l, &r) {
                    result
                } else {
                    self.error(
                        format!("Invalid operation '{}' for types", op.as_str()),
                        span.start.line,
                        span.start.column,
                    );
                    SynapseType::Void
                }
            }
            Expr::UnaryExpr { op, operand, span } => {
                let t = self.check_expr(operand);
                match op {
                    UnaryOp::Not if !matches!(t, SynapseType::Bool) => {
                        self.error(
                            "Operand of 'not' must be boolean".into(),
                            span.start.line,
                            span.start.column,
                        );
                    }
                    UnaryOp::Neg if !matches!(t, SynapseType::Number { .. }) => {
                        self.error(
                            "Operand of '-' must be numeric".into(),
                            span.start.line,
                            span.start.column,
                        );
                    }
                    _ => {}
                }
                if *op == UnaryOp::Not {
                    SynapseType::Bool
                } else {
                    t
                }
            }
            Expr::MemberExpr { object, property, span } => self.check_member(object, property, span),
            Expr::CallExpr {
                callee,
                args,
                named_args,
                span,
            } => self.check_call(callee, args, named_args, span),
        }
    }

    fn check_member(&mut self, object: &Expr, property: &str, span: &Span) -> SynapseType {
        if let Expr::IdentExpr { name, .. } = object {
            if let Some(sym) = self.symbols.get(name) {
                if sym.sensor_type.as_deref() == Some("Lidar") && property == "nearest_distance" {
                    return SynapseType::Number { unit: UnitKind::M };
                }
            }
        }

        let obj_type = self.check_expr(object);
        match &obj_type {
            SynapseType::Scan if property == "nearest_distance" => {
                SynapseType::Number { unit: UnitKind::M }
            }
            SynapseType::Pose => pose_property(property).unwrap_or_else(|| {
                self.error(
                    format!("Unknown pose property '{property}'"),
                    span.start.line,
                    span.start.column,
                );
                SynapseType::Void
            }),
            SynapseType::Velocity => velocity_property(property).unwrap_or_else(|| {
                self.error(
                    format!("Unknown velocity property '{property}'"),
                    span.start.line,
                    span.start.column,
                );
                SynapseType::Void
            }),
            SynapseType::Named { name } => {
                if let Some(prop) = object_property(name, property) {
                    return prop;
                }
                if let Some(methods) = builtin_methods(name) {
                    if let Some(method) = methods.get(property) {
                        return method.returns.clone();
                    }
                }
                self.error(
                    format!("Unknown member '{property}'"),
                    span.start.line,
                    span.start.column,
                );
                SynapseType::Void
            }
            _ => {
                self.error(
                    format!("Unknown member '{property}'"),
                    span.start.line,
                    span.start.column,
                );
                SynapseType::Void
            }
        }
    }

    fn check_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        named_args: &[NamedArg],
        span: &Span,
    ) -> SynapseType {
        if let Expr::IdentExpr { name, .. } = callee {
            if let Some(sig) = builtin_functions().get(name.as_str()) {
                for arg in named_args {
                    if let Some(expected) = sig.named_params.get(&arg.name) {
                        let actual = self.check_expr(&arg.value);
                        self.assert_compatible(
                            expected,
                            &actual,
                            arg.span.start.line,
                            arg.span.start.column,
                        );
                    } else {
                        self.error(
                            format!("Unknown named argument '{}'", arg.name),
                            arg.span.start.line,
                            arg.span.start.column,
                        );
                    }
                }
                return sig.returns.clone();
            }
            self.error(
                format!("Unknown function '{name}'"),
                span.start.line,
                span.start.column,
            );
            return SynapseType::Void;
        }

        let Expr::MemberExpr { object, property, .. } = callee else {
            self.error("Invalid call target".into(), span.start.line, span.start.column);
            return SynapseType::Void;
        };
        let Expr::IdentExpr { name: target_name, .. } = object.as_ref() else {
            self.error("Invalid call target".into(), span.start.line, span.start.column);
            return SynapseType::Void;
        };

        let Some(sym) = self.symbols.get(target_name).cloned() else {
            self.error(
                format!("Undefined identifier '{target_name}'"),
                span.start.line,
                span.start.column,
            );
            return SynapseType::Void;
        };

        if sym.kind == SymbolKind::Robot {
            if let Some(method) = robot_methods().get(property.as_str()) {
                for (i, arg) in args.iter().enumerate() {
                    if let Some(expected) = method.params.get(i) {
                        let actual = self.check_expr(arg);
                        self.assert_compatible(
                            expected,
                            &actual,
                            span.start.line,
                            span.start.column,
                        );
                    }
                }
                return method.returns.clone();
            }
            self.error(
                format!("Unknown robot method '{property}'"),
                span.start.line,
                span.start.column,
            );
            return SynapseType::Void;
        }

        let type_name = match sym.kind {
            SymbolKind::Sensor => sym.sensor_type.clone().unwrap_or_default(),
            SymbolKind::Actuator => sym.actuator_type.clone().unwrap_or_default(),
            SymbolKind::Safety => "Safety".into(),
            SymbolKind::AiModel => {
                if let SynapseType::Named { name } = sym.robo_type {
                    name
                } else {
                    String::new()
                }
            }
            SymbolKind::Agent => "Agent".into(),
            _ => String::new(),
        };

        if type_name == "LLM" && property == "drive" {
            self.error(
                "AI models cannot control actuators directly — use reason(), safety.validate(), then actuator.execute()".into(),
                span.start.line,
                span.start.column,
            );
        }

        let Some(methods) = builtin_methods(&type_name) else {
            self.error(
                format!("Unknown method '{property}' on {type_name}"),
                span.start.line,
                span.start.column,
            );
            return SynapseType::Void;
        };
        let Some(method) = methods.get(property.as_str()) else {
            self.error(
                format!("Unknown method '{property}' on {type_name}"),
                span.start.line,
                span.start.column,
            );
            return SynapseType::Void;
        };

        for arg in named_args {
            if let Some(expected) = method.named_params.get(&arg.name) {
                let actual = self.check_expr(&arg.value);
                self.assert_compatible(
                    expected,
                    &actual,
                    arg.span.start.line,
                    arg.span.start.column,
                );
            }
        }

        for arg in args {
            let actual = self.check_expr(arg);
            if type_name == "Safety" && property == "validate" {
                self.assert_named_type(&actual, "ActionProposal", span.start.line, span.start.column);
            }
            if type_name == "DifferentialDrive" && property == "execute" {
                self.assert_named_type(&actual, "SafeAction", span.start.line, span.start.column);
            }
            if type_name == "VisionModel" && property == "detect" {
                self.assert_named_type(&actual, "CameraFrame", span.start.line, span.start.column);
            }
        }

        method.returns.clone()
    }

    fn types_compatible(&self, expected: &SynapseType, actual: &SynapseType) -> bool {
        if std::mem::discriminant(expected) == std::mem::discriminant(actual) {
            match (expected, actual) {
                (SynapseType::Number { unit: eu, .. }, SynapseType::Number { unit: au, .. }) => {
                    units_compatible(*eu, *au)
                }
                (SynapseType::Named { name: en }, SynapseType::Named { name: an }) => {
                    en == an || an.contains(en.as_str())
                }
                _ => true,
            }
        } else if let (SynapseType::Named { name }, SynapseType::Scan) = (expected, actual) {
            name.contains("Lidar")
        } else if let (SynapseType::Scan, SynapseType::Named { name }) = (expected, actual) {
            ["Detection", "CameraFrame", "Completion"].contains(&name.as_str())
        } else {
            false
        }
    }

    fn assert_named_type(&mut self, actual: &SynapseType, type_name: &str, line: u32, column: u32) {
        if let SynapseType::Named { name } = actual {
            if name == type_name {
                return;
            }
        }
        self.error(
            format!("Expected {type_name}, got {:?}", actual.kind_name()),
            line,
            column,
        );
    }

    fn assert_compatible(
        &mut self,
        expected: &SynapseType,
        actual: &SynapseType,
        line: u32,
        column: u32,
    ) {
        if matches!(expected, SynapseType::Void) && matches!(actual, SynapseType::Void) {
            return;
        }
        if !self.types_compatible(expected, actual) {
            if let (SynapseType::Number { unit: eu, .. }, SynapseType::Number { unit: au, .. }) =
                (expected, actual)
            {
                self.error(
                    format!("Unit mismatch: expected '{}', got '{}'", eu.as_str(), au.as_str()),
                    line,
                    column,
                );
            } else {
                self.error(
                    format!(
                        "Type mismatch: expected {}, got {}",
                        expected.kind_name(),
                        actual.kind_name()
                    ),
                    line,
                    column,
                );
            }
        }
    }

    fn error(&mut self, message: String, line: u32, column: u32) {
        self.errors.push(Diagnostic {
            message,
            line,
            column,
        });
    }
}

trait SynapseTypeExt {
    fn unit(&self) -> UnitKind;
    fn kind_name(&self) -> &'static str;
}

impl SynapseTypeExt for SynapseType {
    fn unit(&self) -> UnitKind {
        match self {
            SynapseType::Number { unit, .. } => *unit,
            _ => UnitKind::None,
        }
    }

    fn kind_name(&self) -> &'static str {
        match self {
            SynapseType::Void => "void",
            SynapseType::Bool => "bool",
            SynapseType::Number { .. } => "number",
            SynapseType::String => "string",
            SynapseType::Named { .. } => "named",
            SynapseType::Scan => "scan",
            SynapseType::Pose => "pose",
            SynapseType::Velocity => "velocity",
            SynapseType::Trajectory => "trajectory",
            SynapseType::Transform => "transform",
        }
    }
}

trait HalMemberDeclExt {
    fn name(&self) -> &str;
}

impl HalMemberDeclExt for HalMemberDecl {
    fn name(&self) -> &str {
        match self {
            HalMemberDecl::HalI2cDecl { name, .. }
            | HalMemberDecl::HalSpiDecl { name, .. }
            | HalMemberDecl::HalGpioDecl { name, .. }
            | HalMemberDecl::HalPwmDecl { name, .. }
            | HalMemberDecl::HalUartDecl { name, .. }
            | HalMemberDecl::HalAdcDecl { name, .. } => name,
        }
    }
}

trait SafetyBlockRules {
    fn rules(&self) -> &[SafetyRule];
    fn zones(&self) -> &[SafetyZoneDecl];
}

impl SafetyBlockRules for SafetyBlock {
    fn rules(&self) -> &[SafetyRule] {
        match self {
            SafetyBlock::SafetyBlock { rules, .. } => rules,
        }
    }

    fn zones(&self) -> &[SafetyZoneDecl] {
        match self {
            SafetyBlock::SafetyBlock { zones, .. } => zones,
        }
    }
}

pub struct FnSig {
    named_params: HashMap<String, SynapseType>,
    returns: SynapseType,
}

fn message_type_for(name: &str) -> Option<SynapseType> {
    match name {
        "Velocity" => Some(SynapseType::Velocity),
        "Pose" => Some(SynapseType::Pose),
        "Scan" => Some(SynapseType::Scan),
        "String" => Some(SynapseType::String),
        _ => None,
    }
}

fn service_type_for(name: &str) -> Option<SynapseType> {
    match name {
        "ResetCostmap" | "ClearCostmap" | "SetPose" => Some(SynapseType::Named {
            name: name.into(),
        }),
        _ => None,
    }
}

fn action_type_for(name: &str) -> Option<SynapseType> {
    match name {
        "NavigateTo" | "FollowPath" | "PickObject" => Some(SynapseType::Named {
            name: name.into(),
        }),
        _ => None,
    }
}

fn sensor_type_for(name: &str) -> Option<SynapseType> {
    let base = match name {
        "Lidar" | "IMU" | "GPS" | "Camera" | "AltitudeSensor" | "ForceTorque" => {
            Some(SynapseType::Named { name: name.into() })
        }
        _ => None,
    };
    if base.is_some() {
        return base;
    }
    if all_library_sensor_types().contains_key(name) {
        Some(SynapseType::Named { name: name.into() })
    } else {
        None
    }
}

fn actuator_type_for(name: &str) -> Option<SynapseType> {
    match name {
        "DifferentialDrive" | "RoboticArm" | "DroneRotors" | "Gripper" => Some(SynapseType::Named {
            name: name.into(),
        }),
        _ => None,
    }
}

fn ai_model_type_for(name: &str) -> Option<SynapseType> {
    match name {
        "LLM" | "VisionModel" | "EmbeddingModel" => Some(SynapseType::Named {
            name: name.into(),
        }),
        _ => None,
    }
}

fn pose_property(name: &str) -> Option<SynapseType> {
    match name {
        "x" | "y" | "z" => Some(SynapseType::Number { unit: UnitKind::M }),
        "theta" => Some(SynapseType::Number { unit: UnitKind::Rad }),
        _ => None,
    }
}

fn velocity_property(name: &str) -> Option<SynapseType> {
    match name {
        "linear" => Some(SynapseType::Number { unit: UnitKind::MPerS }),
        "angular" => Some(SynapseType::Number { unit: UnitKind::RadPerS }),
        _ => None,
    }
}

fn object_property(type_name: &str, property: &str) -> Option<SynapseType> {
    match (type_name, property) {
        ("IMUReading", "yaw" | "roll" | "pitch") => Some(SynapseType::Number { unit: UnitKind::Rad }),
        ("ForceTorqueReading", "force") => Some(SynapseType::Number { unit: UnitKind::None }),
        ("GPSReading", "lat" | "lon") => Some(SynapseType::Number { unit: UnitKind::None }),
        ("ActionProposal" | "SafeAction" | "NavigationPolicy", "linear") => {
            Some(SynapseType::Number { unit: UnitKind::MPerS })
        }
        ("ActionProposal" | "SafeAction" | "NavigationPolicy", "angular") => {
            Some(SynapseType::Number { unit: UnitKind::RadPerS })
        }
        ("Detection", "label") => Some(SynapseType::String),
        ("Detection", "confidence") => Some(SynapseType::Number { unit: UnitKind::None }),
        ("Detection", "nearest_distance") => Some(SynapseType::Number { unit: UnitKind::M }),
        ("Completion", "text") => Some(SynapseType::String),
        _ => None,
    }
}

fn builtin_functions() -> HashMap<&'static str, FnSig> {
    HashMap::from([
        (
            "pose",
            FnSig {
                named_params: HashMap::from([
                    ("x".into(), SynapseType::Number { unit: UnitKind::M }),
                    ("y".into(), SynapseType::Number { unit: UnitKind::M }),
                    ("theta".into(), SynapseType::Number { unit: UnitKind::Rad }),
                    ("z".into(), SynapseType::Number { unit: UnitKind::M }),
                ]),
                returns: SynapseType::Pose,
            },
        ),
        (
            "velocity",
            FnSig {
                named_params: HashMap::from([
                    ("linear".into(), SynapseType::Number { unit: UnitKind::MPerS }),
                    ("angular".into(), SynapseType::Number { unit: UnitKind::RadPerS }),
                ]),
                returns: SynapseType::Velocity,
            },
        ),
        (
            "trajectory",
            FnSig {
                named_params: HashMap::from([
                    ("from".into(), SynapseType::Pose),
                    ("to".into(), SynapseType::Pose),
                    ("steps".into(), SynapseType::Number { unit: UnitKind::None }),
                ]),
                returns: SynapseType::Trajectory,
            },
        ),
        (
            "transform",
            FnSig {
                named_params: HashMap::from([
                    ("from".into(), SynapseType::String),
                    ("to".into(), SynapseType::String),
                    ("pose".into(), SynapseType::Pose),
                ]),
                returns: SynapseType::Transform,
            },
        ),
    ])
}

fn robot_methods() -> HashMap<&'static str, MethodSig> {
    HashMap::from([
        (
            "pose",
            MethodSig {
                params: vec![],
                named_params: HashMap::new(),
                returns: SynapseType::Pose,
            },
        ),
        (
            "velocity",
            MethodSig {
                params: vec![],
                named_params: HashMap::new(),
                returns: SynapseType::Velocity,
            },
        ),
        (
            "in_zone",
            MethodSig {
                params: vec![SynapseType::String],
                named_params: HashMap::new(),
                returns: SynapseType::Bool,
            },
        ),
    ])
}

fn builtin_methods(type_name: &str) -> Option<HashMap<&'static str, MethodSig>> {
    let m = |params: Vec<SynapseType>, named: HashMap<&str, SynapseType>, returns: SynapseType| {
        MethodSig {
            params,
            named_params: named.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
            returns,
        }
    };

    match type_name {
        "Lidar" => Some(HashMap::from([
            (
                "read",
                m(vec![], HashMap::new(), SynapseType::Scan),
            ),
            (
                "nearest_distance",
                m(
                    vec![],
                    HashMap::new(),
                    SynapseType::Number { unit: UnitKind::M },
                ),
            ),
        ])),
        "IMU" => Some(HashMap::from([(
            "read",
            m(
                vec![],
                HashMap::new(),
                SynapseType::Named {
                    name: "IMUReading".into(),
                },
            ),
        )])),
        "AltitudeSensor" => Some(HashMap::from([(
            "read",
            m(
                vec![],
                HashMap::new(),
                SynapseType::Number { unit: UnitKind::M },
            ),
        )])),
        "ForceTorque" => Some(HashMap::from([(
            "read",
            m(
                vec![],
                HashMap::new(),
                SynapseType::Named {
                    name: "ForceTorqueReading".into(),
                },
            ),
        )])),
        "Scan" => Some(HashMap::from([(
            "nearest_distance",
            m(
                vec![],
                HashMap::new(),
                SynapseType::Number { unit: UnitKind::M },
            ),
        )])),
        "Camera" => Some(HashMap::from([
            (
                "read",
                m(
                    vec![],
                    HashMap::new(),
                    SynapseType::Named {
                        name: "CameraFrame".into(),
                    },
                ),
            ),
            (
                "frame",
                m(
                    vec![],
                    HashMap::new(),
                    SynapseType::Named {
                        name: "CameraFrame".into(),
                    },
                ),
            ),
            (
                "analyze",
                m(
                    vec![],
                    HashMap::new(),
                    SynapseType::Named {
                        name: "Detection".into(),
                    },
                ),
            ),
        ])),
        "DifferentialDrive" => Some(HashMap::from([
            (
                "drive",
                m(
                    vec![],
                    HashMap::from([
                        ("linear", SynapseType::Number { unit: UnitKind::MPerS }),
                        ("angular", SynapseType::Number { unit: UnitKind::RadPerS }),
                    ]),
                    SynapseType::Void,
                ),
            ),
            (
                "execute",
                m(
                    vec![SynapseType::Named {
                        name: "SafeAction".into(),
                    }],
                    HashMap::new(),
                    SynapseType::Void,
                ),
            ),
            (
                "follow",
                m(
                    vec![],
                    HashMap::from([("path", SynapseType::Trajectory)]),
                    SynapseType::Void,
                ),
            ),
            ("stop", m(vec![], HashMap::new(), SynapseType::Void)),
        ])),
        "RoboticArm" => Some(HashMap::from([
            (
                "move_to",
                m(
                    vec![],
                    HashMap::from([
                        ("x", SynapseType::Number { unit: UnitKind::M }),
                        ("y", SynapseType::Number { unit: UnitKind::M }),
                        ("z", SynapseType::Number { unit: UnitKind::M }),
                    ]),
                    SynapseType::Void,
                ),
            ),
            ("grip", m(vec![], HashMap::new(), SynapseType::Void)),
            ("release", m(vec![], HashMap::new(), SynapseType::Void)),
        ])),
        "DroneRotors" => Some(HashMap::from([
            (
                "set_thrust",
                m(
                    vec![],
                    HashMap::from([(
                        "thrust",
                        SynapseType::Number { unit: UnitKind::None },
                    )]),
                    SynapseType::Void,
                ),
            ),
            ("hover", m(vec![], HashMap::new(), SynapseType::Void)),
        ])),
        "Gripper" => Some(HashMap::from([
            ("close", m(vec![], HashMap::new(), SynapseType::Void)),
            ("open", m(vec![], HashMap::new(), SynapseType::Void)),
        ])),
        "LLM" => Some(HashMap::from([
            (
                "reason",
                m(
                    vec![],
                    HashMap::from([
                        ("prompt", SynapseType::String),
                        ("input", SynapseType::Scan),
                    ]),
                    SynapseType::Named {
                        name: "ActionProposal".into(),
                    },
                ),
            ),
            (
                "summarize",
                m(
                    vec![],
                    HashMap::from([("input", SynapseType::Scan)]),
                    SynapseType::Named {
                        name: "Completion".into(),
                    },
                ),
            ),
        ])),
        "VisionModel" => Some(HashMap::from([(
            "detect",
            m(
                vec![SynapseType::Named {
                    name: "CameraFrame".into(),
                }],
                HashMap::new(),
                SynapseType::Named {
                    name: "Detection".into(),
                },
            ),
        )])),
        "Agent" => Some(HashMap::from([(
            "plan",
            m(vec![], HashMap::new(), SynapseType::Void),
        )])),
        "Safety" => Some(HashMap::from([(
            "validate",
            m(
                vec![SynapseType::Named {
                    name: "ActionProposal".into(),
                }],
                HashMap::new(),
                SynapseType::Named {
                    name: "SafeAction".into(),
                },
            ),
        )])),
        other if all_library_sensor_types().contains_key(other) => {
            Some(library_sensor_methods(other))
        }
        _ => None,
    }
}

fn infer_read_return(type_name: &str) -> SynapseType {
    if type_name.contains("Lidar")
        || type_name.contains("Velodyne")
        || type_name.contains("Hokuyo")
        || type_name.contains("Ydlidar")
        || type_name.contains("RealSense")
    {
        return SynapseType::Scan;
    }
    if type_name.contains("BNO") || type_name.contains("LSM9") || type_name.contains("IMU") {
        return SynapseType::Named {
            name: "IMUReading".into(),
        };
    }
    if type_name.contains("BMP") || type_name.contains("VL53") || type_name.contains("UWMF") {
        return SynapseType::Number { unit: UnitKind::M };
    }
    SynapseType::Void
}

pub fn merge_library_methods(
    methods: &mut HashMap<String, HashMap<String, MethodSig>>,
) {
    for (type_name, info) in all_library_sensor_types() {
        methods.entry(type_name).or_insert_with(|| {
            let read_name = match info.robo_type {
                SynapseType::Named { ref name } => name.clone(),
                _ => String::new(),
            };
            HashMap::from([
                (
                    "read".to_string(),
                    MethodSig {
                        params: vec![],
                        named_params: HashMap::new(),
                        returns: infer_read_return(&read_name),
                    },
                ),
                (
                    "calibrate".to_string(),
                    MethodSig {
                        params: vec![],
                        named_params: HashMap::new(),
                        returns: SynapseType::Void,
                    },
                ),
            ])
        });
    }
}

fn library_sensor_methods(type_name: &str) -> HashMap<&'static str, MethodSig> {
    HashMap::from([
        (
            "read",
            MethodSig {
                params: vec![],
                named_params: HashMap::new(),
                returns: infer_read_return(type_name),
            },
        ),
        (
            "calibrate",
            MethodSig {
                params: vec![],
                named_params: HashMap::new(),
                returns: SynapseType::Void,
            },
        ),
    ])
}

pub fn get_library_for_sensor_type(sensor_type: &str) -> Option<String> {
    all_library_sensor_types()
        .get(sensor_type)
        .map(|info| info.library.clone())
}

#[allow(non_snake_case)]
pub fn MESSAGE_TYPES() -> HashMap<String, SynapseType> {
    HashMap::from([
        ("Velocity".into(), SynapseType::Velocity),
        ("Pose".into(), SynapseType::Pose),
        ("Scan".into(), SynapseType::Scan),
        ("String".into(), SynapseType::String),
    ])
}

#[allow(non_snake_case)]
pub fn SERVICE_TYPES() -> HashMap<String, SynapseType> {
    HashMap::from([
        ("ResetCostmap".into(), SynapseType::Named { name: "ResetCostmap".into() }),
        ("ClearCostmap".into(), SynapseType::Named { name: "ClearCostmap".into() }),
        ("SetPose".into(), SynapseType::Named { name: "SetPose".into() }),
    ])
}

#[allow(non_snake_case)]
pub fn ACTION_TYPES() -> HashMap<String, SynapseType> {
    HashMap::from([
        ("NavigateTo".into(), SynapseType::Named { name: "NavigateTo".into() }),
        ("FollowPath".into(), SynapseType::Named { name: "FollowPath".into() }),
        ("PickObject".into(), SynapseType::Named { name: "PickObject".into() }),
    ])
}

#[allow(non_snake_case)]
pub fn SENSOR_TYPES() -> HashMap<String, SynapseType> {
    let mut map = HashMap::from([
        ("Lidar".into(), SynapseType::Named { name: "Lidar".into() }),
        ("IMU".into(), SynapseType::Named { name: "IMU".into() }),
        ("GPS".into(), SynapseType::Named { name: "GPS".into() }),
        ("Camera".into(), SynapseType::Named { name: "Camera".into() }),
        ("AltitudeSensor".into(), SynapseType::Named { name: "AltitudeSensor".into() }),
        ("ForceTorque".into(), SynapseType::Named { name: "ForceTorque".into() }),
    ]);
    for (type_name, info) in all_library_sensor_types() {
        map.insert(type_name, info.robo_type);
    }
    map
}

#[allow(non_snake_case)]
pub fn ACTUATOR_TYPES() -> HashMap<String, SynapseType> {
    HashMap::from([
        ("DifferentialDrive".into(), SynapseType::Named { name: "DifferentialDrive".into() }),
        ("RoboticArm".into(), SynapseType::Named { name: "RoboticArm".into() }),
        ("DroneRotors".into(), SynapseType::Named { name: "DroneRotors".into() }),
        ("Gripper".into(), SynapseType::Named { name: "Gripper".into() }),
    ])
}

#[allow(non_snake_case)]
pub fn AI_MODEL_TYPES() -> HashMap<String, SynapseType> {
    HashMap::from([
        ("LLM".into(), SynapseType::Named { name: "LLM".into() }),
        ("VisionModel".into(), SynapseType::Named { name: "VisionModel".into() }),
        ("EmbeddingModel".into(), SynapseType::Named { name: "EmbeddingModel".into() }),
    ])
}

#[allow(non_snake_case)]
pub fn AI_VALUE_TYPES() -> HashMap<String, SynapseType> {
    HashMap::from([
        ("ActionProposal".into(), SynapseType::Named { name: "ActionProposal".into() }),
        ("SafeAction".into(), SynapseType::Named { name: "SafeAction".into() }),
        ("Completion".into(), SynapseType::Named { name: "Completion".into() }),
        ("Detection".into(), SynapseType::Named { name: "Detection".into() }),
        ("Classification".into(), SynapseType::Named { name: "Classification".into() }),
        ("Plan".into(), SynapseType::Named { name: "Plan".into() }),
        ("Agent".into(), SynapseType::Named { name: "Agent".into() }),
        ("CameraFrame".into(), SynapseType::Named { name: "CameraFrame".into() }),
        ("Memory".into(), SynapseType::Named { name: "Memory".into() }),
        ("Prompt".into(), SynapseType::String),
    ])
}

#[allow(non_snake_case)]
pub fn BUILTIN_FUNCTIONS() -> HashMap<&'static str, FnSig> {
    builtin_functions()
}

#[allow(non_snake_case)]
pub fn ROBOT_METHODS() -> HashMap<&'static str, MethodSig> {
    robot_methods()
}

#[allow(non_snake_case)]
pub fn BUILTIN_METHODS() -> HashMap<String, HashMap<String, MethodSig>> {
    let mut map: HashMap<String, HashMap<String, MethodSig>> = HashMap::new();
    for ty in [
        "Lidar", "Camera", "DifferentialDrive", "RoboticArm", "DroneRotors", "LLM", "VisionModel",
        "Agent", "Safety",
    ] {
        if let Some(methods) = builtin_methods(ty) {
            map.insert(
                ty.to_string(),
                methods
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect(),
            );
        }
    }
    merge_library_methods(&mut map);
    map
}

#[allow(non_snake_case)]
pub fn SCAN_PROPERTIES() -> HashMap<String, SynapseType> {
    HashMap::from([(
        "nearest_distance".into(),
        SynapseType::Number { unit: UnitKind::M },
    )])
}

#[allow(non_snake_case)]
pub fn OBJECT_PROPERTIES() -> HashMap<String, HashMap<String, SynapseType>> {
    HashMap::from([
        (
            "IMUReading".into(),
            HashMap::from([
                ("yaw".into(), SynapseType::Number { unit: UnitKind::Rad }),
                ("roll".into(), SynapseType::Number { unit: UnitKind::Rad }),
                ("pitch".into(), SynapseType::Number { unit: UnitKind::Rad }),
            ]),
        ),
        (
            "Detection".into(),
            HashMap::from([
                ("label".into(), SynapseType::String),
                ("confidence".into(), SynapseType::Number { unit: UnitKind::None }),
                ("nearest_distance".into(), SynapseType::Number { unit: UnitKind::M }),
            ]),
        ),
    ])
}

#[allow(non_snake_case)]
pub fn POSE_PROPERTIES() -> HashMap<String, SynapseType> {
    HashMap::from([
        ("x".into(), SynapseType::Number { unit: UnitKind::M }),
        ("y".into(), SynapseType::Number { unit: UnitKind::M }),
        ("theta".into(), SynapseType::Number { unit: UnitKind::Rad }),
        ("z".into(), SynapseType::Number { unit: UnitKind::M }),
    ])
}

#[allow(non_snake_case)]
pub fn VELOCITY_PROPERTIES() -> HashMap<String, SynapseType> {
    HashMap::from([
        ("linear".into(), SynapseType::Number { unit: UnitKind::MPerS }),
        ("angular".into(), SynapseType::Number { unit: UnitKind::RadPerS }),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;

    fn check_source(source: &str) -> Result<(), SynapseError> {
        let tokens = tokenize(source)?;
        let program = parse(tokens)?;
        type_check(&program)
    }

    #[test]
    fn accepts_valid_robot_program() {
        let source = r#"
            robot R {
              sensor lidar: Lidar;
              actuator wheels: DifferentialDrive;
              safety { max_speed = 1.5 m/s; }
              behavior go() {
                let d = lidar.read().nearest_distance;
                wheels.drive(linear: 0.5 m/s, angular: 0.0 rad/s);
              }
            }
        "#;
        assert!(check_source(source).is_ok());
    }

    #[test]
    fn rejects_unit_mismatch_in_drive_args() {
        let source = r#"
            robot R {
              actuator wheels: DifferentialDrive;
              behavior go() {
                wheels.drive(linear: 0.5 m, angular: 0.0 rad/s);
              }
            }
        "#;
        assert!(matches!(check_source(source), Err(SynapseError::TypeCheck { .. })));
    }

    #[test]
    fn rejects_unknown_sensor_type() {
        let source = r#"
            robot R {
              sensor cam: UnknownSensor;
            }
        "#;
        assert!(matches!(check_source(source), Err(SynapseError::TypeCheck { .. })));
    }

    #[test]
    fn rejects_unimported_library() {
        let source = r#"
            robot R {
              sensor imu: BoschBNO055 from bosch.bno055 on imu;
            }
        "#;
        assert!(matches!(check_source(source), Err(SynapseError::TypeCheck { .. })));
    }

    #[test]
    fn accepts_imported_library_sensor() {
        let source = r#"
            import bosch.bno055;
            robot R {
              soc ESP32;
              hal { i2c imu at 0x68; }
              sensor imu: BoschBNO055 from bosch.bno055 on imu;
            }
        "#;
        assert!(check_source(source).is_ok());
    }

    #[test]
    fn rejects_unsafe_ai_drive() {
        let source = r#"
            robot R {
              ai_model planner: LLM { provider: "mock"; model: "p"; }
              behavior demo() {
                planner.drive(linear: 0.5 m/s, angular: 0.0 rad/s);
              }
            }
        "#;
        let err = check_source(source).unwrap_err();
        assert!(err
            .diagnostics()
            .iter()
            .any(|d| d.message.contains("cannot control actuators")));
    }
}
