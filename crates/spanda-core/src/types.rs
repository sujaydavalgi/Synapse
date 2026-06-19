use crate::ai::resolve_ai_import;
use crate::ast::*;
use crate::error::{Diagnostic, SpandaError};
use crate::foundations::{
    resolve_module_import, resolve_type_alias, CapabilityDecl, EnumDecl, EventDecl,
    EventHandlerDecl, MatchArm, StateMachineDecl, StructDecl, TaskDecl, TraitDecl, TraitImplDecl,
    TwinDecl,
};
use crate::hal::hal_member_from_decl;
use crate::lib_registry::{all_library_sensor_types, resolve_import};
use crate::soc::{get_soc_profile, validate_hal_against_soc};
use crate::stdlib::resolve_std_import;
use crate::type_system::{binary_physical_op_allowed, is_action_proposal_type, resolve_type_name};
use std::collections::HashMap;

pub fn type_check(program: &Program) -> Result<(), SpandaError> {
    check(program)
}

pub fn check(program: &Program) -> Result<(), SpandaError> {
    let mut checker = TypeChecker::new();
    checker.check_program(program);
    if checker.errors.is_empty() {
        Ok(())
    } else {
        Err(SpandaError::TypeCheck {
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

pub fn result_unit_for_binary(
    op: BinaryOp,
    left: &SpandaType,
    right: &SpandaType,
) -> Option<SpandaType> {
    match op {
        BinaryOp::And | BinaryOp::Or => {
            if matches!(left, SpandaType::Bool) && matches!(right, SpandaType::Bool) {
                Some(SpandaType::Bool)
            } else {
                None
            }
        }
        BinaryOp::Lt
        | BinaryOp::Lte
        | BinaryOp::Gt
        | BinaryOp::Gte
        | BinaryOp::Eq
        | BinaryOp::Neq => {
            if matches!(left, SpandaType::Number { .. })
                && matches!(right, SpandaType::Number { .. })
            {
                let SpandaType::Number { unit: lu, .. } = left else {
                    unreachable!()
                };
                let SpandaType::Number { unit: ru, .. } = right else {
                    unreachable!()
                };
                if units_compatible(*lu, *ru) {
                    return Some(SpandaType::Bool);
                }
            }
            if matches!(left, SpandaType::Bool) && matches!(right, SpandaType::Bool) {
                return Some(SpandaType::Bool);
            }
            if matches!(left, SpandaType::String) && matches!(right, SpandaType::String) {
                return Some(SpandaType::Bool);
            }
            None
        }
        BinaryOp::Add | BinaryOp::Sub => {
            if let (SpandaType::Number { unit: lu, .. }, SpandaType::Number { unit: ru, .. }) =
                (left, right)
            {
                if units_compatible(*lu, *ru) {
                    let unit = if *lu != UnitKind::None { *lu } else { *ru };
                    return Some(SpandaType::Number { unit });
                }
            }
            None
        }
        BinaryOp::Mul | BinaryOp::Div => {
            if matches!(left, SpandaType::Number { .. })
                && matches!(right, SpandaType::Number { .. })
            {
                Some(SpandaType::Number {
                    unit: UnitKind::None,
                })
            } else {
                None
            }
        }
    }
}

pub struct MethodSig {
    params: Vec<SpandaType>,
    named_params: HashMap<String, SpandaType>,
    returns: SpandaType,
}

#[derive(Clone)]
struct SymbolEntry {
    robo_type: SpandaType,
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

type TraitMethodSig = (Vec<(String, String)>, String);

pub struct TypeChecker {
    pub errors: Vec<Diagnostic>,
    symbols: HashMap<String, SymbolEntry>,
    enum_variants: HashMap<String, Vec<String>>,
    variant_owner: HashMap<String, String>,
    struct_defs: HashMap<String, Vec<(String, String)>>,
    trait_defs: HashMap<String, HashMap<String, TraitMethodSig>>,
    agent_trait_methods: HashMap<String, HashMap<String, SpandaType>>,
    state_machine_states: std::collections::HashSet<String>,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            symbols: HashMap::new(),
            enum_variants: HashMap::new(),
            variant_owner: HashMap::new(),
            struct_defs: HashMap::new(),
            trait_defs: HashMap::new(),
            agent_trait_methods: HashMap::new(),
            state_machine_states: std::collections::HashSet::new(),
        }
    }

    pub fn check_program(&mut self, program: &Program) {
        let Program::Program {
            imports,
            structs,
            enums,
            traits,
            robots,
            ..
        } = program;
        let mut imported = std::collections::HashSet::new();
        for imp in imports {
            let ImportDecl::ImportDecl { path, span } = imp;
            if resolve_import(path).is_none()
                && resolve_ai_import(path).is_none()
                && !resolve_module_import(path)
                && !resolve_std_import(path)
            {
                self.error(
                    format!("Unknown import '{path}'"),
                    span.start.line,
                    span.start.column,
                );
            } else {
                imported.insert(path.clone());
            }
        }

        for struct_decl in structs {
            self.check_struct(struct_decl);
        }
        for enum_decl in enums {
            self.check_enum(enum_decl);
        }
        for trait_decl in traits {
            self.check_trait(trait_decl);
        }

        for robot in robots {
            self.check_robot(robot, &imported);
        }
    }

    fn check_struct(&mut self, decl: &StructDecl) {
        let StructDecl::StructDecl { name, fields, span } = decl;
        for field in fields {
            if resolve_type_alias(&field.type_name).is_none()
                && !matches!(
                    field.type_name.as_str(),
                    "Pose" | "Velocity" | "Scan" | "String" | "Bool" | "Path"
                )
            {
                self.error(
                    format!("Unknown field type '{}'", field.type_name),
                    field.span.start.line,
                    field.span.start.column,
                );
            }
        }
        self.symbols.insert(
            name.clone(),
            SymbolEntry {
                robo_type: SpandaType::Named { name: name.clone() },
                kind: SymbolKind::Variable,
                sensor_type: None,
                actuator_type: None,
            },
        );
        self.struct_defs.insert(
            name.clone(),
            fields
                .iter()
                .map(|f| (f.name.clone(), f.type_name.clone()))
                .collect(),
        );
        let _ = span;
    }

    fn check_enum(&mut self, decl: &EnumDecl) {
        let EnumDecl::EnumDecl {
            name,
            variants,
            span,
        } = decl;
        if variants.is_empty() {
            self.error(
                format!("Enum '{name}' must declare at least one variant"),
                span.start.line,
                span.start.column,
            );
        }
        self.symbols.insert(
            name.clone(),
            SymbolEntry {
                robo_type: SpandaType::Named { name: name.clone() },
                kind: SymbolKind::Variable,
                sensor_type: None,
                actuator_type: None,
            },
        );
        self.enum_variants.insert(name.clone(), variants.clone());
        for variant in variants {
            if let Some(existing) = self.variant_owner.insert(variant.clone(), name.clone()) {
                self.error(
                    format!("Enum variant '{variant}' already declared in enum '{existing}'"),
                    span.start.line,
                    span.start.column,
                );
            }
        }
    }

    fn check_trait(&mut self, decl: &TraitDecl) {
        let TraitDecl::TraitDecl {
            name,
            methods,
            span,
        } = decl;
        if methods.is_empty() {
            self.error(
                format!("Trait '{name}' must declare at least one method"),
                span.start.line,
                span.start.column,
            );
        }
        let mut method_map = HashMap::new();
        for method in methods {
            method_map.insert(
                method.name.clone(),
                (
                    method
                        .params
                        .iter()
                        .map(|p| (p.name.clone(), p.type_name.clone()))
                        .collect(),
                    method.return_type.clone(),
                ),
            );
        }
        self.trait_defs.insert(name.clone(), method_map);
    }

    fn type_name_to_spanda(&self, type_name: &str) -> SpandaType {
        resolve_type_name(type_name).unwrap_or(SpandaType::Named {
            name: type_name.to_string(),
        })
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
            tasks,
            state_machines,
            events,
            event_handlers,
            twin,
            verify,
            observe,
            trait_impls,
            ..
        } = robot;

        self.symbols.clear();
        self.state_machine_states.clear();
        self.agent_trait_methods.clear();
        for enum_name in self.enum_variants.keys() {
            self.symbols.insert(
                enum_name.clone(),
                SymbolEntry {
                    robo_type: SpandaType::Named {
                        name: enum_name.clone(),
                    },
                    kind: SymbolKind::Variable,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
        }
        for struct_name in self.struct_defs.keys() {
            self.symbols.insert(
                struct_name.clone(),
                SymbolEntry {
                    robo_type: SpandaType::Named {
                        name: struct_name.clone(),
                    },
                    kind: SymbolKind::Variable,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
        }

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
            let NodeDecl::NodeDecl {
                namespace, span, ..
            } = node;
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
                    robo_type: SpandaType::Named {
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
                    robo_type: SpandaType::Named {
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
        for trait_impl in trait_impls {
            self.check_trait_impl(trait_impl);
        }

        for sm in state_machines {
            let StateMachineDecl::StateMachineDecl {
                name,
                states,
                transitions,
                span,
            } = sm;
            if states.is_empty() {
                self.error(
                    format!("State machine '{name}' must declare at least one state"),
                    span.start.line,
                    span.start.column,
                );
            }
            let state_set: std::collections::HashSet<_> = states.iter().cloned().collect();
            for transition in transitions {
                if !state_set.contains(&transition.from) || !state_set.contains(&transition.to) {
                    self.error(
                        format!(
                            "Invalid transition {} -> {} in state machine '{name}'",
                            transition.from, transition.to
                        ),
                        transition.span.start.line,
                        transition.span.start.column,
                    );
                }
            }
            self.state_machine_states.extend(states.iter().cloned());
        }

        if let Some(twin_decl) = twin {
            let TwinDecl::TwinDecl {
                name,
                mirrors,
                span,
                ..
            } = twin_decl;
            if mirrors.is_empty() {
                self.error(
                    "Digital twin must mirror at least one field".into(),
                    span.start.line,
                    span.start.column,
                );
            }
            const ALLOWED_MIRROR_FIELDS: &[&str] =
                &["pose", "velocity", "battery", "status", "scan"];
            for mirror in mirrors {
                if !ALLOWED_MIRROR_FIELDS.contains(&mirror.as_str()) {
                    self.error(
                        format!("Unknown twin mirror field '{mirror}'"),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
            self.symbols.insert(
                name.clone(),
                SymbolEntry {
                    robo_type: SpandaType::Named {
                        name: "Twin".into(),
                    },
                    kind: SymbolKind::Variable,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
        }

        if let Some(verify_decl) = verify {
            let crate::foundations::VerifyDecl::VerifyDecl { rules, span } = verify_decl;
            let saved = self.symbols.clone();
            self.symbols.insert(
                "robot".into(),
                SymbolEntry {
                    robo_type: SpandaType::Named {
                        name: "Robot".into(),
                    },
                    kind: SymbolKind::Robot,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
            for rule in rules {
                let t = self.check_expr(rule);
                if !matches!(t, SpandaType::Bool) {
                    self.error(
                        "verify rule must be boolean".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
            self.symbols = saved;
        }

        if let Some(observe_decl) = observe {
            let crate::foundations::ObserveDecl::ObserveDecl { sensors, span } = observe_decl;
            if sensors.is_empty() {
                self.error(
                    "observe block must list at least one sensor".into(),
                    span.start.line,
                    span.start.column,
                );
            }
            for sensor_name in sensors {
                if self.symbols.get(sensor_name).map(|s| s.kind) != Some(SymbolKind::Sensor) {
                    self.error(
                        format!("observe references unknown sensor '{sensor_name}'"),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
            self.symbols.insert(
                "fusion".into(),
                SymbolEntry {
                    robo_type: SpandaType::Named {
                        name: "SensorFusion".into(),
                    },
                    kind: SymbolKind::Variable,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
        }

        for behavior in behaviors {
            let BehaviorDecl::BehaviorDecl {
                name,
                requires,
                ensures,
                invariant,
                body,
                ..
            } = behavior;
            if let Some(req) = requires {
                let t = self.check_expr(req);
                if !matches!(t, SpandaType::Bool) {
                    self.error("requires clause must be boolean".into(), 0, 0);
                }
            }
            if let Some(post) = ensures {
                let t = self.check_expr(post);
                if !matches!(t, SpandaType::Bool) {
                    self.error("ensures clause must be boolean".into(), 0, 0);
                }
            }
            if let Some(inv) = invariant {
                let t = self.check_expr(inv);
                if !matches!(t, SpandaType::Bool) {
                    self.error("invariant clause must be boolean".into(), 0, 0);
                }
            }
            self.symbols.insert(
                name.clone(),
                SymbolEntry {
                    robo_type: SpandaType::Void,
                    kind: SymbolKind::Behavior,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
            self.check_behavior(body);
        }

        for task in tasks {
            let TaskDecl::TaskDecl {
                name,
                interval_ms,
                requires,
                ensures,
                invariant,
                budget: _budget,
                body,
                span,
            } = task;
            if *interval_ms <= 0.0 {
                self.error(
                    "task interval must be positive".into(),
                    span.start.line,
                    span.start.column,
                );
            } else if *interval_ms < 1.0 {
                self.error(
                    "task interval must be at least 1ms".into(),
                    span.start.line,
                    span.start.column,
                );
            }
            if let Some(req) = requires {
                let t = self.check_expr(req);
                if !matches!(t, SpandaType::Bool) {
                    self.error(
                        "requires clause must be boolean".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
            if let Some(post) = ensures {
                let t = self.check_expr(post);
                if !matches!(t, SpandaType::Bool) {
                    self.error(
                        "ensures clause must be boolean".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
            if let Some(inv) = invariant {
                let t = self.check_expr(inv);
                if !matches!(t, SpandaType::Bool) {
                    self.error(
                        "invariant clause must be boolean".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
            self.symbols.insert(
                name.clone(),
                SymbolEntry {
                    robo_type: SpandaType::Void,
                    kind: SymbolKind::Behavior,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
            self.check_behavior(body);
        }

        for handler in event_handlers {
            let EventHandlerDecl::EventHandlerDecl {
                event_name,
                body,
                span,
            } = handler;
            let declared = events.iter().any(|e| {
                let EventDecl::EventDecl { name, .. } = e;
                name == event_name
            });
            if !declared {
                self.error(
                    format!("No event declared for handler '{event_name}'"),
                    span.start.line,
                    span.start.column,
                );
            }
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
                robo_type: message_type_for(message_type).unwrap_or(SpandaType::Void),
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
                robo_type: service_type_for(service_type).unwrap_or(SpandaType::Void),
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
                robo_type: action_type_for(action_type).unwrap_or(SpandaType::Void),
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
                robo_type: sensor_type_for(sensor_type).unwrap_or(SpandaType::Named {
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
                robo_type: actuator_type_for(actuator_type).unwrap_or(SpandaType::Named {
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
            SafetyRule::MaxSpeedRule {
                value, unit, span, ..
            } => {
                let t = self.check_expr(value);
                if !matches!(t, SpandaType::Number { .. }) || !units_compatible(t.unit(), *unit) {
                    self.error(
                        format!("Expected value with unit '{}' for max_speed", unit.as_str()),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
            SafetyRule::StopIfRule { condition, span } => {
                if !matches!(self.check_expr(condition), SpandaType::Bool) {
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
        if !matches!(self.check_expr(x), SpandaType::Number { .. })
            || !matches!(self.check_expr(y), SpandaType::Number { .. })
        {
            self.error(
                "Zone coordinates must be numeric".into(),
                span.start.line,
                span.start.column,
            );
        }
        if *shape == ZoneShape::Circle {
            if let Some(r) = radius {
                if !matches!(self.check_expr(r), SpandaType::Number { .. }) {
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
                if !matches!(self.check_expr(w), SpandaType::Number { .. }) {
                    self.error(
                        "Zone size must be numeric".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
            if let Some(h) = height {
                if !matches!(self.check_expr(h), SpandaType::Number { .. }) {
                    self.error(
                        "Zone size must be numeric".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
        }
    }

    fn check_trait_impl(&mut self, decl: &TraitImplDecl) {
        let TraitImplDecl::TraitImplDecl {
            trait_name,
            agent_name,
            methods,
            span,
        } = decl;
        let Some(trait_methods) = self.trait_defs.get(trait_name).cloned() else {
            self.error(
                format!("Unknown trait '{trait_name}'"),
                span.start.line,
                span.start.column,
            );
            return;
        };
        if self.symbols.get(agent_name).map(|s| s.kind) != Some(SymbolKind::Agent) {
            self.error(
                format!("Trait impl target '{agent_name}' is not a declared agent"),
                span.start.line,
                span.start.column,
            );
            return;
        }
        let mut registered: Vec<(String, SpandaType)> = Vec::new();
        for method in methods {
            let Some((expected_params, expected_return)) = trait_methods.get(&method.name) else {
                self.error(
                    format!("Trait '{trait_name}' has no method '{}'", method.name),
                    method.span.start.line,
                    method.span.start.column,
                );
                continue;
            };
            if method.return_type != *expected_return {
                self.error(
                    format!(
                        "Trait method '{}' return type mismatch: expected {}, got {}",
                        method.name, expected_return, method.return_type
                    ),
                    method.span.start.line,
                    method.span.start.column,
                );
            }
            if method.params.len() != expected_params.len() {
                self.error(
                    format!("Trait method '{}' parameter count mismatch", method.name),
                    method.span.start.line,
                    method.span.start.column,
                );
            }
            for (actual, (pname, ptype)) in method.params.iter().zip(expected_params.iter()) {
                if actual.name != *pname || actual.type_name != *ptype {
                    self.error(
                        format!(
                            "Trait method '{}' parameter '{}' type mismatch",
                            method.name, pname
                        ),
                        actual.span.start.line,
                        actual.span.start.column,
                    );
                }
            }
            let saved = self.symbols.clone();
            for param in &method.params {
                self.symbols.insert(
                    param.name.clone(),
                    SymbolEntry {
                        robo_type: self.type_name_to_spanda(&param.type_name),
                        kind: SymbolKind::Variable,
                        sensor_type: None,
                        actuator_type: None,
                    },
                );
            }
            for stmt in &method.body {
                self.check_stmt(stmt);
            }
            self.symbols = saved;
            registered.push((
                method.name.clone(),
                self.type_name_to_spanda(&method.return_type),
            ));
        }
        let agent_methods = self
            .agent_trait_methods
            .entry(agent_name.clone())
            .or_default();
        for (name, ret) in registered {
            agent_methods.insert(name, ret);
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
                robo_type: ai_model_type_for(model_type).unwrap_or(SpandaType::Void),
                kind: SymbolKind::AiModel,
                sensor_type: None,
                actuator_type: None,
            },
        );
    }

    fn check_capability(&mut self, agent_name: &str, cap: &CapabilityDecl) {
        let allowed = ["read", "propose_motion", "summarize", "detect", "plan"];
        if !allowed.contains(&cap.action.as_str()) {
            self.error(
                format!("Unknown capability '{}'", cap.action),
                cap.span.start.line,
                cap.span.start.column,
            );
            return;
        }
        if cap.action == "read" {
            if let Some(target) = &cap.target {
                if !self.symbols.contains_key(target) {
                    self.error(
                        format!("Agent '{agent_name}' capability read({target}) references unknown resource"),
                        cap.span.start.line,
                        cap.span.start.column,
                    );
                }
            } else {
                self.error(
                    format!("Agent '{agent_name}' read capability requires a target"),
                    cap.span.start.line,
                    cap.span.start.column,
                );
            }
        }
    }

    fn check_agent(&mut self, agent: &AgentDecl) {
        let AgentDecl::AgentDecl {
            name,
            uses_ai,
            tools,
            capabilities,
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
        for cap in capabilities {
            self.check_capability(name, cap);
        }
        self.symbols.insert(
            name.clone(),
            SymbolEntry {
                robo_type: SpandaType::Named {
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
                robo_type: SpandaType::Named {
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
            Stmt::VarDecl {
                name,
                type_annotation,
                init,
                span,
            } => {
                let inferred = init.as_ref().map(|e| self.check_expr(e));
                let t = match (type_annotation, inferred) {
                    (Some(expected), Some(actual)) => {
                        self.assert_compatible(
                            expected,
                            &actual,
                            span.start.line,
                            span.start.column,
                        );
                        expected.clone()
                    }
                    (Some(expected), None) => expected.clone(),
                    (None, Some(actual)) => actual,
                    (None, None) => SpandaType::Void,
                };
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
                if !matches!(self.check_expr(condition), SpandaType::Bool) {
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
                if self.symbols.get(service_name).map(|s| s.kind) != Some(SymbolKind::Service) {
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
                if self.symbols.get(action_name).map(|s| s.kind) != Some(SymbolKind::Action) {
                    self.error(
                        format!("Unknown action '{action_name}'"),
                        span.start.line,
                        span.start.column,
                    );
                } else {
                    let goal_t = self.check_expr(goal);
                    if !matches!(goal_t, SpandaType::Pose | SpandaType::Trajectory) {
                        self.error(
                            "Action goal must be pose or trajectory".into(),
                            span.start.line,
                            span.start.column,
                        );
                    }
                }
            }
            Stmt::EmergencyStopStmt { .. } | Stmt::ResetEmergencyStopStmt { .. } => {}
            Stmt::RememberStmt { value, .. } => {
                self.check_expr(value);
            }
            Stmt::EmitStmt { .. } => {}
            Stmt::EnterStmt { state_name, span } => {
                if !self.state_machine_states.contains(state_name) {
                    self.error(
                        format!("Unknown state '{state_name}' for enter statement"),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
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

    fn check_expr(&mut self, expr: &Expr) -> SpandaType {
        match expr {
            Expr::LiteralExpr { value, .. } => match value {
                LiteralValue::Bool(_) => SpandaType::Bool,
                LiteralValue::Number(_) => SpandaType::Number {
                    unit: UnitKind::None,
                },
                LiteralValue::String(_) => SpandaType::String,
                LiteralValue::Null => SpandaType::Void,
            },
            Expr::UnitLiteralExpr { value: _, unit, .. } => SpandaType::Number { unit: *unit },
            Expr::IdentExpr { name, span } => {
                if let Some(enum_name) = self.variant_owner.get(name) {
                    return SpandaType::EnumVariant {
                        enum_name: enum_name.clone(),
                        variant: name.clone(),
                    };
                }
                if let Some(sym) = self.symbols.get(name) {
                    sym.robo_type.clone()
                } else {
                    self.error(
                        format!("Undefined identifier '{name}'"),
                        span.start.line,
                        span.start.column,
                    );
                    SpandaType::Void
                }
            }
            Expr::BinaryExpr {
                op,
                left,
                right,
                span,
            } => {
                let l = self.check_expr(left);
                let r = self.check_expr(right);
                if matches!(
                    op,
                    BinaryOp::Add
                        | BinaryOp::Sub
                        | BinaryOp::Lt
                        | BinaryOp::Lte
                        | BinaryOp::Gt
                        | BinaryOp::Gte
                        | BinaryOp::Eq
                        | BinaryOp::Neq
                ) && !binary_physical_op_allowed(*op, &l, &r)
                {
                    self.error(
                        format!(
                            "Invalid operation '{}' between incompatible types ({}, {})",
                            op.as_str(),
                            l.kind_name(),
                            r.kind_name()
                        ),
                        span.start.line,
                        span.start.column,
                    );
                }
                if let Some(result) = result_unit_for_binary(*op, &l, &r) {
                    result
                } else {
                    self.error(
                        format!("Invalid operation '{}' for types", op.as_str()),
                        span.start.line,
                        span.start.column,
                    );
                    SpandaType::Void
                }
            }
            Expr::UnaryExpr { op, operand, span } => {
                let t = self.check_expr(operand);
                match op {
                    UnaryOp::Not if !matches!(t, SpandaType::Bool) => {
                        self.error(
                            "Operand of 'not' must be boolean".into(),
                            span.start.line,
                            span.start.column,
                        );
                    }
                    UnaryOp::Neg if !matches!(t, SpandaType::Number { .. }) => {
                        self.error(
                            "Operand of '-' must be numeric".into(),
                            span.start.line,
                            span.start.column,
                        );
                    }
                    _ => {}
                }
                if *op == UnaryOp::Not {
                    SpandaType::Bool
                } else {
                    t
                }
            }
            Expr::MemberExpr {
                object,
                property,
                span,
            } => self.check_member(object, property, span),
            Expr::CallExpr {
                callee,
                args,
                named_args,
                span,
            } => self.check_call(callee, args, named_args, span),
            Expr::MatchExpr {
                scrutinee,
                arms,
                span,
            } => self.check_match(scrutinee, arms, span),
            Expr::StructLiteralExpr {
                type_name,
                fields,
                span,
            } => self.check_struct_literal(type_name, fields, span),
        }
    }

    fn check_struct_literal(
        &mut self,
        type_name: &str,
        fields: &[crate::ast::StructFieldInit],
        span: &Span,
    ) -> SpandaType {
        let Some(def) = self.struct_defs.get(type_name).cloned() else {
            self.error(
                format!("Unknown struct type '{type_name}'"),
                span.start.line,
                span.start.column,
            );
            return SpandaType::Void;
        };
        let mut provided = std::collections::HashSet::new();
        for field in fields {
            if provided.contains(&field.name) {
                self.error(
                    format!("Duplicate struct field '{}'", field.name),
                    field.span.start.line,
                    field.span.start.column,
                );
            }
            provided.insert(field.name.clone());
            let expected = def
                .iter()
                .find(|(name, _)| name == &field.name)
                .map(|(_, t)| self.type_name_to_spanda(t));
            let Some(expected) = expected else {
                self.error(
                    format!("Struct '{type_name}' has no field '{}'", field.name),
                    field.span.start.line,
                    field.span.start.column,
                );
                continue;
            };
            let actual = self.check_expr(&field.value);
            self.assert_compatible(
                &expected,
                &actual,
                field.span.start.line,
                field.span.start.column,
            );
        }
        for (name, _) in &def {
            if !provided.contains(name) {
                self.error(
                    format!("Missing struct field '{name}' in '{type_name}' literal"),
                    span.start.line,
                    span.start.column,
                );
            }
        }
        SpandaType::Named {
            name: type_name.to_string(),
        }
    }

    fn check_match(&mut self, scrutinee: &Expr, arms: &[MatchArm], span: &Span) -> SpandaType {
        let _scrutinee_type = self.check_expr(scrutinee);
        if arms.is_empty() {
            self.error(
                "match expression requires at least one arm".into(),
                span.start.line,
                span.start.column,
            );
        }
        for arm in arms {
            for stmt in &arm.body {
                self.check_stmt(stmt);
            }
        }
        self.check_match_exhaustiveness(arms, span);
        SpandaType::Void
    }

    fn check_match_exhaustiveness(&mut self, arms: &[MatchArm], span: &Span) {
        use std::collections::HashSet;
        let arm_names: HashSet<String> = arms.iter().map(|a| a.variant.clone()).collect();
        if arm_names.is_empty() {
            return;
        }
        for variants in self.enum_variants.values() {
            let variant_set: HashSet<String> = variants.iter().cloned().collect();
            if arm_names.is_subset(&variant_set) {
                if arm_names.len() < variant_set.len() {
                    let missing: Vec<_> = variant_set.difference(&arm_names).cloned().collect();
                    self.error(
                        format!(
                            "Non-exhaustive match: missing variants {}",
                            missing.join(", ")
                        ),
                        span.start.line,
                        span.start.column,
                    );
                }
                return;
            }
        }
    }

    fn check_member(&mut self, object: &Expr, property: &str, span: &Span) -> SpandaType {
        if let Expr::IdentExpr { name, .. } = object {
            if let Some(sym) = self.symbols.get(name) {
                if sym.sensor_type.as_deref() == Some("Lidar") && property == "nearest_distance" {
                    return SpandaType::Number { unit: UnitKind::M };
                }
            }
        }

        let obj_type = self.check_expr(object);
        match &obj_type {
            SpandaType::Scan if property == "nearest_distance" => {
                SpandaType::Number { unit: UnitKind::M }
            }
            SpandaType::Pose => pose_property(property).unwrap_or_else(|| {
                self.error(
                    format!("Unknown pose property '{property}'"),
                    span.start.line,
                    span.start.column,
                );
                SpandaType::Void
            }),
            SpandaType::Velocity => velocity_property(property).unwrap_or_else(|| {
                self.error(
                    format!("Unknown velocity property '{property}'"),
                    span.start.line,
                    span.start.column,
                );
                SpandaType::Void
            }),
            SpandaType::Named { name } => {
                if let Some(variants) = self.enum_variants.get(name) {
                    if variants.iter().any(|v| v == property) {
                        return SpandaType::EnumVariant {
                            enum_name: name.clone(),
                            variant: property.to_string(),
                        };
                    }
                }
                if let Some(fields) = self.struct_defs.get(name) {
                    if let Some((_, type_name)) = fields.iter().find(|(field, _)| field == property)
                    {
                        return self.type_name_to_spanda(type_name);
                    }
                }
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
                SpandaType::Void
            }
            _ => {
                self.error(
                    format!("Unknown member '{property}'"),
                    span.start.line,
                    span.start.column,
                );
                SpandaType::Void
            }
        }
    }

    fn check_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        named_args: &[NamedArg],
        span: &Span,
    ) -> SpandaType {
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
            return SpandaType::Void;
        }

        let Expr::MemberExpr {
            object, property, ..
        } = callee
        else {
            self.error(
                "Invalid call target".into(),
                span.start.line,
                span.start.column,
            );
            return SpandaType::Void;
        };
        let Expr::IdentExpr {
            name: target_name, ..
        } = object.as_ref()
        else {
            self.error(
                "Invalid call target".into(),
                span.start.line,
                span.start.column,
            );
            return SpandaType::Void;
        };

        let Some(sym) = self.symbols.get(target_name).cloned() else {
            self.error(
                format!("Undefined identifier '{target_name}'"),
                span.start.line,
                span.start.column,
            );
            return SpandaType::Void;
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
            return SpandaType::Void;
        }

        if sym.kind == SymbolKind::Agent {
            if let Some(methods) = self.agent_trait_methods.get(target_name) {
                if let Some(return_type) = methods.get(property.as_str()) {
                    return return_type.clone();
                }
            }
        }

        let type_name = match sym.kind {
            SymbolKind::Sensor => sym.sensor_type.clone().unwrap_or_default(),
            SymbolKind::Actuator => sym.actuator_type.clone().unwrap_or_default(),
            SymbolKind::Safety => "Safety".into(),
            SymbolKind::AiModel => {
                if let SpandaType::Named { name } = sym.robo_type {
                    name
                } else {
                    String::new()
                }
            }
            SymbolKind::Agent => "Agent".into(),
            _ => {
                if let SpandaType::Named { name } = sym.robo_type {
                    name
                } else {
                    String::new()
                }
            }
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
            return SpandaType::Void;
        };
        let Some(method) = methods.get(property.as_str()) else {
            self.error(
                format!("Unknown method '{property}' on {type_name}"),
                span.start.line,
                span.start.column,
            );
            return SpandaType::Void;
        };

        for arg in named_args {
            if let Some(expected) = method.named_params.get(&arg.name) {
                if type_name == "Twin" && arg.name == "field" {
                    if let Expr::IdentExpr { name, span } = &arg.value {
                        const ALLOWED: &[&str] = &["pose", "velocity", "battery", "status", "scan"];
                        if !ALLOWED.contains(&name.as_str()) {
                            self.error(
                                format!("Unknown twin mirror field '{name}'"),
                                span.start.line,
                                span.start.column,
                            );
                        }
                        continue;
                    }
                }
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
            if type_name == "Safety" && property == "validate" && !is_action_proposal_type(&actual)
            {
                self.error(
                    "safety.validate() expects ActionProposal".into(),
                    span.start.line,
                    span.start.column,
                );
            }
            if type_name == "DifferentialDrive" && property == "execute" {
                if is_action_proposal_type(&actual) {
                    self.error(
                        "ActionProposal cannot be passed to actuator.execute() — call safety.validate() first".into(),
                        span.start.line,
                        span.start.column,
                    );
                } else if !crate::type_system::is_safe_action_type(&actual) {
                    self.error(
                        "actuator.execute() requires SafeAction from safety.validate()".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
            if type_name == "VisionModel" && property == "detect" {
                self.assert_named_type(&actual, "CameraFrame", span.start.line, span.start.column);
            }
        }

        method.returns.clone()
    }

    fn types_compatible(&self, expected: &SpandaType, actual: &SpandaType) -> bool {
        if std::mem::discriminant(expected) == std::mem::discriminant(actual) {
            match (expected, actual) {
                (SpandaType::Number { unit: eu, .. }, SpandaType::Number { unit: au, .. }) => {
                    units_compatible(*eu, *au)
                }
                (SpandaType::Named { name: en }, SpandaType::Named { name: an }) => {
                    en == an || an.contains(en.as_str())
                }
                (
                    SpandaType::EnumVariant {
                        enum_name: e1,
                        variant: v1,
                    },
                    SpandaType::EnumVariant {
                        enum_name: e2,
                        variant: v2,
                    },
                ) => e1 == e2 && v1 == v2,
                (SpandaType::Named { name }, SpandaType::EnumVariant { enum_name, .. }) => {
                    name == enum_name
                }
                (SpandaType::EnumVariant { enum_name, .. }, SpandaType::Named { name }) => {
                    name == enum_name
                }
                (
                    SpandaType::Generic {
                        name: n1,
                        type_args: a1,
                    },
                    SpandaType::Generic {
                        name: n2,
                        type_args: a2,
                    },
                ) => {
                    n1 == n2
                        && a1.len() == a2.len()
                        && a1
                            .iter()
                            .zip(a2.iter())
                            .all(|(e, a)| self.types_compatible(e, a))
                }
                _ => true,
            }
        } else if let (SpandaType::Named { name }, SpandaType::Scan) = (expected, actual) {
            name.contains("Lidar")
        } else if let (SpandaType::Scan, SpandaType::Named { name }) = (expected, actual) {
            ["Detection", "CameraFrame", "Completion"].contains(&name.as_str())
        } else if matches!(expected, SpandaType::Int)
            && matches!(
                actual,
                SpandaType::Number {
                    unit: UnitKind::None,
                    ..
                }
            )
            || matches!(expected, SpandaType::Float) && matches!(actual, SpandaType::Number { .. })
            || matches!(expected, SpandaType::Velocity)
                && matches!(
                    actual,
                    SpandaType::Number {
                        unit: UnitKind::MPerS,
                        ..
                    }
                )
            || matches!(actual, SpandaType::Velocity)
                && matches!(
                    expected,
                    SpandaType::Number {
                        unit: UnitKind::MPerS,
                        ..
                    }
                )
        {
            true
        } else if let (SpandaType::Named { name }, SpandaType::Number { unit, .. }) =
            (expected, actual)
        {
            match name.as_str() {
                "Distance" => *unit == UnitKind::M,
                "Duration" => matches!(*unit, UnitKind::Ms | UnitKind::S),
                "Angle" => matches!(*unit, UnitKind::Rad | UnitKind::Deg),
                "Acceleration" => *unit == UnitKind::MPerS2,
                "AngularVelocity" => *unit == UnitKind::RadPerS,
                _ => false,
            }
        } else if let (SpandaType::Named { name }, SpandaType::String) = (expected, actual) {
            name == "Goal"
        } else if let (SpandaType::String, SpandaType::Named { name }) = (expected, actual) {
            name == "Goal"
        } else {
            false
        }
    }

    fn assert_named_type(&mut self, actual: &SpandaType, type_name: &str, line: u32, column: u32) {
        if let SpandaType::Named { name } = actual {
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
        expected: &SpandaType,
        actual: &SpandaType,
        line: u32,
        column: u32,
    ) {
        if matches!(expected, SpandaType::Void) && matches!(actual, SpandaType::Void) {
            return;
        }
        if !self.types_compatible(expected, actual) {
            if let (SpandaType::Number { unit: eu, .. }, SpandaType::Number { unit: au, .. }) =
                (expected, actual)
            {
                self.error(
                    format!(
                        "Unit mismatch: expected '{}', got '{}'",
                        eu.as_str(),
                        au.as_str()
                    ),
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

trait SpandaTypeExt {
    fn unit(&self) -> UnitKind;
    fn kind_name(&self) -> &'static str;
}

impl SpandaTypeExt for SpandaType {
    fn unit(&self) -> UnitKind {
        match self {
            SpandaType::Number { unit, .. } => *unit,
            _ => UnitKind::None,
        }
    }

    fn kind_name(&self) -> &'static str {
        match self {
            SpandaType::Void => "void",
            SpandaType::Int => "int",
            SpandaType::Float => "float",
            SpandaType::Bool => "bool",
            SpandaType::Number { .. } => "number",
            SpandaType::String => "string",
            SpandaType::Char => "char",
            SpandaType::Bytes => "bytes",
            SpandaType::Null => "null",
            SpandaType::Named { .. } => "named",
            SpandaType::Generic { name, .. } => {
                let _ = name;
                "generic"
            }
            SpandaType::Scan => "scan",
            SpandaType::Pose => "pose",
            SpandaType::Velocity => "velocity",
            SpandaType::Trajectory => "trajectory",
            SpandaType::Transform => "transform",
            SpandaType::EnumVariant { .. } => "enum_variant",
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
    named_params: HashMap<String, SpandaType>,
    returns: SpandaType,
}

fn message_type_for(name: &str) -> Option<SpandaType> {
    match name {
        "Velocity" => Some(SpandaType::Velocity),
        "Pose" => Some(SpandaType::Pose),
        "Scan" => Some(SpandaType::Scan),
        "String" => Some(SpandaType::String),
        _ => None,
    }
}

fn service_type_for(name: &str) -> Option<SpandaType> {
    match name {
        "ResetCostmap" | "ClearCostmap" | "SetPose" => {
            Some(SpandaType::Named { name: name.into() })
        }
        _ => None,
    }
}

fn action_type_for(name: &str) -> Option<SpandaType> {
    match name {
        "NavigateTo" | "FollowPath" | "PickObject" => Some(SpandaType::Named { name: name.into() }),
        _ => None,
    }
}

fn sensor_type_for(name: &str) -> Option<SpandaType> {
    let base = match name {
        "Lidar" | "IMU" | "GPS" | "Camera" | "AltitudeSensor" | "ForceTorque" => {
            Some(SpandaType::Named { name: name.into() })
        }
        _ => None,
    };
    if base.is_some() {
        return base;
    }
    if all_library_sensor_types().contains_key(name) {
        Some(SpandaType::Named { name: name.into() })
    } else {
        None
    }
}

fn actuator_type_for(name: &str) -> Option<SpandaType> {
    match name {
        "DifferentialDrive" | "RoboticArm" | "DroneRotors" | "Gripper" => {
            Some(SpandaType::Named { name: name.into() })
        }
        _ => None,
    }
}

fn ai_model_type_for(name: &str) -> Option<SpandaType> {
    match name {
        "LLM" | "VisionModel" | "EmbeddingModel" => Some(SpandaType::Named { name: name.into() }),
        _ => None,
    }
}

fn pose_property(name: &str) -> Option<SpandaType> {
    match name {
        "x" | "y" | "z" => Some(SpandaType::Number { unit: UnitKind::M }),
        "theta" => Some(SpandaType::Number {
            unit: UnitKind::Rad,
        }),
        _ => None,
    }
}

fn velocity_property(name: &str) -> Option<SpandaType> {
    match name {
        "linear" => Some(SpandaType::Number {
            unit: UnitKind::MPerS,
        }),
        "angular" => Some(SpandaType::Number {
            unit: UnitKind::RadPerS,
        }),
        _ => None,
    }
}

fn object_property(type_name: &str, property: &str) -> Option<SpandaType> {
    match (type_name, property) {
        ("IMUReading", "yaw" | "roll" | "pitch") => Some(SpandaType::Number {
            unit: UnitKind::Rad,
        }),
        ("ForceTorqueReading", "force") => Some(SpandaType::Number {
            unit: UnitKind::None,
        }),
        ("GPSReading", "lat" | "lon") => Some(SpandaType::Number {
            unit: UnitKind::None,
        }),
        ("ActionProposal" | "SafeAction" | "NavigationPolicy", "linear") => {
            Some(SpandaType::Number {
                unit: UnitKind::MPerS,
            })
        }
        ("ActionProposal" | "SafeAction" | "NavigationPolicy", "angular") => {
            Some(SpandaType::Number {
                unit: UnitKind::RadPerS,
            })
        }
        ("ActionProposal", "trace") => Some(SpandaType::Named {
            name: "ReasoningTrace".into(),
        }),
        ("Goal", "text") => Some(SpandaType::String),
        ("Agent", "goal") => Some(SpandaType::Named {
            name: "Goal".into(),
        }),
        ("Detection", "label") => Some(SpandaType::String),
        ("Detection", "confidence") => Some(SpandaType::Number {
            unit: UnitKind::None,
        }),
        ("Detection", "nearest_distance") => Some(SpandaType::Number { unit: UnitKind::M }),
        ("Completion", "text") => Some(SpandaType::String),
        ("FusedObservation", "pose") => Some(SpandaType::Pose),
        ("FusedObservation", "count") => Some(SpandaType::Number {
            unit: UnitKind::None,
        }),
        _ => None,
    }
}

fn builtin_functions() -> HashMap<&'static str, FnSig> {
    HashMap::from([
        (
            "pose",
            FnSig {
                named_params: HashMap::from([
                    ("x".into(), SpandaType::Number { unit: UnitKind::M }),
                    ("y".into(), SpandaType::Number { unit: UnitKind::M }),
                    (
                        "theta".into(),
                        SpandaType::Number {
                            unit: UnitKind::Rad,
                        },
                    ),
                    ("z".into(), SpandaType::Number { unit: UnitKind::M }),
                ]),
                returns: SpandaType::Pose,
            },
        ),
        (
            "velocity",
            FnSig {
                named_params: HashMap::from([
                    (
                        "linear".into(),
                        SpandaType::Number {
                            unit: UnitKind::MPerS,
                        },
                    ),
                    (
                        "angular".into(),
                        SpandaType::Number {
                            unit: UnitKind::RadPerS,
                        },
                    ),
                ]),
                returns: SpandaType::Velocity,
            },
        ),
        (
            "trajectory",
            FnSig {
                named_params: HashMap::from([
                    ("from".into(), SpandaType::Pose),
                    ("to".into(), SpandaType::Pose),
                    (
                        "steps".into(),
                        SpandaType::Number {
                            unit: UnitKind::None,
                        },
                    ),
                ]),
                returns: SpandaType::Trajectory,
            },
        ),
        (
            "transform",
            FnSig {
                named_params: HashMap::from([
                    ("from".into(), SpandaType::String),
                    ("to".into(), SpandaType::String),
                    ("pose".into(), SpandaType::Pose),
                ]),
                returns: SpandaType::Transform,
            },
        ),
        (
            "goal",
            FnSig {
                named_params: HashMap::from([("text".into(), SpandaType::String)]),
                returns: SpandaType::Named {
                    name: "Goal".into(),
                },
            },
        ),
        (
            "recall",
            FnSig {
                named_params: HashMap::from([("key".into(), SpandaType::String)]),
                returns: SpandaType::Named {
                    name: "Memory".into(),
                },
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
                returns: SpandaType::Pose,
            },
        ),
        (
            "velocity",
            MethodSig {
                params: vec![],
                named_params: HashMap::new(),
                returns: SpandaType::Velocity,
            },
        ),
        (
            "in_zone",
            MethodSig {
                params: vec![SpandaType::String],
                named_params: HashMap::new(),
                returns: SpandaType::Bool,
            },
        ),
    ])
}

fn builtin_methods(type_name: &str) -> Option<HashMap<&'static str, MethodSig>> {
    let m = |params: Vec<SpandaType>, named: HashMap<&str, SpandaType>, returns: SpandaType| {
        MethodSig {
            params,
            named_params: named.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
            returns,
        }
    };

    match type_name {
        "Lidar" => Some(HashMap::from([
            ("read", m(vec![], HashMap::new(), SpandaType::Scan)),
            (
                "nearest_distance",
                m(
                    vec![],
                    HashMap::new(),
                    SpandaType::Number { unit: UnitKind::M },
                ),
            ),
        ])),
        "IMU" => Some(HashMap::from([(
            "read",
            m(
                vec![],
                HashMap::new(),
                SpandaType::Named {
                    name: "IMUReading".into(),
                },
            ),
        )])),
        "AltitudeSensor" => Some(HashMap::from([(
            "read",
            m(
                vec![],
                HashMap::new(),
                SpandaType::Number { unit: UnitKind::M },
            ),
        )])),
        "ForceTorque" => Some(HashMap::from([(
            "read",
            m(
                vec![],
                HashMap::new(),
                SpandaType::Named {
                    name: "ForceTorqueReading".into(),
                },
            ),
        )])),
        "Scan" => Some(HashMap::from([(
            "nearest_distance",
            m(
                vec![],
                HashMap::new(),
                SpandaType::Number { unit: UnitKind::M },
            ),
        )])),
        "Camera" => Some(HashMap::from([
            (
                "read",
                m(
                    vec![],
                    HashMap::new(),
                    SpandaType::Named {
                        name: "CameraFrame".into(),
                    },
                ),
            ),
            (
                "frame",
                m(
                    vec![],
                    HashMap::new(),
                    SpandaType::Named {
                        name: "CameraFrame".into(),
                    },
                ),
            ),
            (
                "analyze",
                m(
                    vec![],
                    HashMap::new(),
                    SpandaType::Named {
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
                        (
                            "linear",
                            SpandaType::Number {
                                unit: UnitKind::MPerS,
                            },
                        ),
                        (
                            "angular",
                            SpandaType::Number {
                                unit: UnitKind::RadPerS,
                            },
                        ),
                    ]),
                    SpandaType::Void,
                ),
            ),
            (
                "execute",
                m(
                    vec![SpandaType::Named {
                        name: "SafeAction".into(),
                    }],
                    HashMap::new(),
                    SpandaType::Void,
                ),
            ),
            (
                "follow",
                m(
                    vec![],
                    HashMap::from([("path", SpandaType::Trajectory)]),
                    SpandaType::Void,
                ),
            ),
            ("stop", m(vec![], HashMap::new(), SpandaType::Void)),
        ])),
        "RoboticArm" => Some(HashMap::from([
            (
                "move_to",
                m(
                    vec![],
                    HashMap::from([
                        ("x", SpandaType::Number { unit: UnitKind::M }),
                        ("y", SpandaType::Number { unit: UnitKind::M }),
                        ("z", SpandaType::Number { unit: UnitKind::M }),
                    ]),
                    SpandaType::Void,
                ),
            ),
            ("grip", m(vec![], HashMap::new(), SpandaType::Void)),
            ("release", m(vec![], HashMap::new(), SpandaType::Void)),
        ])),
        "DroneRotors" => Some(HashMap::from([
            (
                "set_thrust",
                m(
                    vec![],
                    HashMap::from([(
                        "thrust",
                        SpandaType::Number {
                            unit: UnitKind::None,
                        },
                    )]),
                    SpandaType::Void,
                ),
            ),
            ("hover", m(vec![], HashMap::new(), SpandaType::Void)),
        ])),
        "Gripper" => Some(HashMap::from([
            ("close", m(vec![], HashMap::new(), SpandaType::Void)),
            ("open", m(vec![], HashMap::new(), SpandaType::Void)),
        ])),
        "LLM" => Some(HashMap::from([
            (
                "reason",
                m(
                    vec![],
                    HashMap::from([
                        ("prompt", SpandaType::String),
                        ("input", SpandaType::Scan),
                        (
                            "goal",
                            SpandaType::Named {
                                name: "Goal".into(),
                            },
                        ),
                    ]),
                    SpandaType::Named {
                        name: "ActionProposal".into(),
                    },
                ),
            ),
            (
                "summarize",
                m(
                    vec![],
                    HashMap::from([("input", SpandaType::Scan)]),
                    SpandaType::Named {
                        name: "Completion".into(),
                    },
                ),
            ),
        ])),
        "VisionModel" => Some(HashMap::from([(
            "detect",
            m(
                vec![SpandaType::Named {
                    name: "CameraFrame".into(),
                }],
                HashMap::new(),
                SpandaType::Named {
                    name: "Detection".into(),
                },
            ),
        )])),
        "Twin" => Some(HashMap::from([
            (
                "frame_count",
                m(
                    vec![],
                    HashMap::new(),
                    SpandaType::Number {
                        unit: UnitKind::None,
                    },
                ),
            ),
            (
                "mirror",
                m(
                    vec![],
                    HashMap::from([("field", SpandaType::String)]),
                    SpandaType::Pose,
                ),
            ),
            (
                "replay",
                m(
                    vec![],
                    HashMap::from([
                        (
                            "index",
                            SpandaType::Number {
                                unit: UnitKind::None,
                            },
                        ),
                        ("field", SpandaType::String),
                    ]),
                    SpandaType::Pose,
                ),
            ),
            ("pose", m(vec![], HashMap::new(), SpandaType::Pose)),
            ("velocity", m(vec![], HashMap::new(), SpandaType::Velocity)),
        ])),
        "Agent" => Some(HashMap::from([(
            "plan",
            m(vec![], HashMap::new(), SpandaType::Void),
        )])),
        "Safety" => Some(HashMap::from([(
            "validate",
            m(
                vec![SpandaType::Named {
                    name: "ActionProposal".into(),
                }],
                HashMap::new(),
                SpandaType::Named {
                    name: "SafeAction".into(),
                },
            ),
        )])),
        "SensorFusion" => Some(HashMap::from([(
            "read",
            m(
                vec![],
                HashMap::new(),
                SpandaType::Named {
                    name: "FusedObservation".into(),
                },
            ),
        )])),
        other if all_library_sensor_types().contains_key(other) => {
            Some(library_sensor_methods(other))
        }
        _ => None,
    }
}

fn infer_read_return(type_name: &str) -> SpandaType {
    if type_name.contains("Lidar")
        || type_name.contains("Velodyne")
        || type_name.contains("Hokuyo")
        || type_name.contains("Ydlidar")
        || type_name.contains("RealSense")
    {
        return SpandaType::Scan;
    }
    if type_name.contains("BNO") || type_name.contains("LSM9") || type_name.contains("IMU") {
        return SpandaType::Named {
            name: "IMUReading".into(),
        };
    }
    if type_name.contains("BMP") || type_name.contains("VL53") || type_name.contains("UWMF") {
        return SpandaType::Number { unit: UnitKind::M };
    }
    SpandaType::Void
}

pub fn merge_library_methods(methods: &mut HashMap<String, HashMap<String, MethodSig>>) {
    for (type_name, info) in all_library_sensor_types() {
        methods.entry(type_name).or_insert_with(|| {
            let read_name = match info.robo_type {
                SpandaType::Named { ref name } => name.clone(),
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
                        returns: SpandaType::Void,
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
                returns: SpandaType::Void,
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
pub fn MESSAGE_TYPES() -> HashMap<String, SpandaType> {
    HashMap::from([
        ("Velocity".into(), SpandaType::Velocity),
        ("Pose".into(), SpandaType::Pose),
        ("Scan".into(), SpandaType::Scan),
        ("String".into(), SpandaType::String),
    ])
}

#[allow(non_snake_case)]
pub fn SERVICE_TYPES() -> HashMap<String, SpandaType> {
    HashMap::from([
        (
            "ResetCostmap".into(),
            SpandaType::Named {
                name: "ResetCostmap".into(),
            },
        ),
        (
            "ClearCostmap".into(),
            SpandaType::Named {
                name: "ClearCostmap".into(),
            },
        ),
        (
            "SetPose".into(),
            SpandaType::Named {
                name: "SetPose".into(),
            },
        ),
    ])
}

#[allow(non_snake_case)]
pub fn ACTION_TYPES() -> HashMap<String, SpandaType> {
    HashMap::from([
        (
            "NavigateTo".into(),
            SpandaType::Named {
                name: "NavigateTo".into(),
            },
        ),
        (
            "FollowPath".into(),
            SpandaType::Named {
                name: "FollowPath".into(),
            },
        ),
        (
            "PickObject".into(),
            SpandaType::Named {
                name: "PickObject".into(),
            },
        ),
    ])
}

#[allow(non_snake_case)]
pub fn SENSOR_TYPES() -> HashMap<String, SpandaType> {
    let mut map = HashMap::from([
        (
            "Lidar".into(),
            SpandaType::Named {
                name: "Lidar".into(),
            },
        ),
        ("IMU".into(), SpandaType::Named { name: "IMU".into() }),
        ("GPS".into(), SpandaType::Named { name: "GPS".into() }),
        (
            "Camera".into(),
            SpandaType::Named {
                name: "Camera".into(),
            },
        ),
        (
            "AltitudeSensor".into(),
            SpandaType::Named {
                name: "AltitudeSensor".into(),
            },
        ),
        (
            "ForceTorque".into(),
            SpandaType::Named {
                name: "ForceTorque".into(),
            },
        ),
    ]);
    for (type_name, info) in all_library_sensor_types() {
        map.insert(type_name, info.robo_type);
    }
    map
}

#[allow(non_snake_case)]
pub fn ACTUATOR_TYPES() -> HashMap<String, SpandaType> {
    HashMap::from([
        (
            "DifferentialDrive".into(),
            SpandaType::Named {
                name: "DifferentialDrive".into(),
            },
        ),
        (
            "RoboticArm".into(),
            SpandaType::Named {
                name: "RoboticArm".into(),
            },
        ),
        (
            "DroneRotors".into(),
            SpandaType::Named {
                name: "DroneRotors".into(),
            },
        ),
        (
            "Gripper".into(),
            SpandaType::Named {
                name: "Gripper".into(),
            },
        ),
    ])
}

#[allow(non_snake_case)]
pub fn AI_MODEL_TYPES() -> HashMap<String, SpandaType> {
    HashMap::from([
        ("LLM".into(), SpandaType::Named { name: "LLM".into() }),
        (
            "VisionModel".into(),
            SpandaType::Named {
                name: "VisionModel".into(),
            },
        ),
        (
            "EmbeddingModel".into(),
            SpandaType::Named {
                name: "EmbeddingModel".into(),
            },
        ),
    ])
}

#[allow(non_snake_case)]
pub fn AI_VALUE_TYPES() -> HashMap<String, SpandaType> {
    HashMap::from([
        (
            "ActionProposal".into(),
            SpandaType::Named {
                name: "ActionProposal".into(),
            },
        ),
        (
            "SafeAction".into(),
            SpandaType::Named {
                name: "SafeAction".into(),
            },
        ),
        (
            "Completion".into(),
            SpandaType::Named {
                name: "Completion".into(),
            },
        ),
        (
            "Detection".into(),
            SpandaType::Named {
                name: "Detection".into(),
            },
        ),
        (
            "Classification".into(),
            SpandaType::Named {
                name: "Classification".into(),
            },
        ),
        (
            "Plan".into(),
            SpandaType::Named {
                name: "Plan".into(),
            },
        ),
        (
            "Agent".into(),
            SpandaType::Named {
                name: "Agent".into(),
            },
        ),
        (
            "CameraFrame".into(),
            SpandaType::Named {
                name: "CameraFrame".into(),
            },
        ),
        (
            "Memory".into(),
            SpandaType::Named {
                name: "Memory".into(),
            },
        ),
        (
            "SensorFusion".into(),
            SpandaType::Named {
                name: "SensorFusion".into(),
            },
        ),
        (
            "FusedObservation".into(),
            SpandaType::Named {
                name: "FusedObservation".into(),
            },
        ),
        ("Prompt".into(), SpandaType::String),
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
        "Lidar",
        "Camera",
        "DifferentialDrive",
        "RoboticArm",
        "DroneRotors",
        "LLM",
        "VisionModel",
        "Agent",
        "Safety",
        "Twin",
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
pub fn SCAN_PROPERTIES() -> HashMap<String, SpandaType> {
    HashMap::from([(
        "nearest_distance".into(),
        SpandaType::Number { unit: UnitKind::M },
    )])
}

#[allow(non_snake_case)]
pub fn OBJECT_PROPERTIES() -> HashMap<String, HashMap<String, SpandaType>> {
    HashMap::from([
        (
            "IMUReading".into(),
            HashMap::from([
                (
                    "yaw".into(),
                    SpandaType::Number {
                        unit: UnitKind::Rad,
                    },
                ),
                (
                    "roll".into(),
                    SpandaType::Number {
                        unit: UnitKind::Rad,
                    },
                ),
                (
                    "pitch".into(),
                    SpandaType::Number {
                        unit: UnitKind::Rad,
                    },
                ),
            ]),
        ),
        (
            "Detection".into(),
            HashMap::from([
                ("label".into(), SpandaType::String),
                (
                    "confidence".into(),
                    SpandaType::Number {
                        unit: UnitKind::None,
                    },
                ),
                (
                    "nearest_distance".into(),
                    SpandaType::Number { unit: UnitKind::M },
                ),
            ]),
        ),
        (
            "FusedObservation".into(),
            HashMap::from([
                ("pose".into(), SpandaType::Pose),
                (
                    "count".into(),
                    SpandaType::Number {
                        unit: UnitKind::None,
                    },
                ),
            ]),
        ),
    ])
}

#[allow(non_snake_case)]
pub fn POSE_PROPERTIES() -> HashMap<String, SpandaType> {
    HashMap::from([
        ("x".into(), SpandaType::Number { unit: UnitKind::M }),
        ("y".into(), SpandaType::Number { unit: UnitKind::M }),
        (
            "theta".into(),
            SpandaType::Number {
                unit: UnitKind::Rad,
            },
        ),
        ("z".into(), SpandaType::Number { unit: UnitKind::M }),
    ])
}

#[allow(non_snake_case)]
pub fn VELOCITY_PROPERTIES() -> HashMap<String, SpandaType> {
    HashMap::from([
        (
            "linear".into(),
            SpandaType::Number {
                unit: UnitKind::MPerS,
            },
        ),
        (
            "angular".into(),
            SpandaType::Number {
                unit: UnitKind::RadPerS,
            },
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;

    fn check_source(source: &str) -> Result<(), SpandaError> {
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
        assert!(matches!(
            check_source(source),
            Err(SpandaError::TypeCheck { .. })
        ));
    }

    #[test]
    fn rejects_unknown_sensor_type() {
        let source = r#"
            robot R {
              sensor cam: UnknownSensor;
            }
        "#;
        assert!(matches!(
            check_source(source),
            Err(SpandaError::TypeCheck { .. })
        ));
    }

    #[test]
    fn rejects_unimported_library() {
        let source = r#"
            robot R {
              sensor imu: BoschBNO055 from bosch.bno055 on imu;
            }
        "#;
        assert!(matches!(
            check_source(source),
            Err(SpandaError::TypeCheck { .. })
        ));
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
