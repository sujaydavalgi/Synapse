//! Spanda program type checker.
//!
use crate::diagnostics::Diagnostic;
use crate::host::TypeCheckHost;
use crate::message_registry::{is_comm_capability, MessageRegistry};
use crate::module_registry::ModuleRegistry;
use crate::type_system::{
    binary_physical_op_allowed, generic_arity, is_action_proposal_type, physical_category,
    resolve_type_name,
};
use crate::units::{self, unit_matches_named_type};
use spanda_ast::comm_decl as comm;
use spanda_ast::foundations::MissionDecl;
use spanda_ast::foundations::{
    resolve_module_import, resolve_type_alias, CapabilityDecl, EnumDecl, EventDecl,
    EventHandlerDecl, ExternFnDecl, MatchArm, ModuleFnDecl, ResourceBudgetDecl, SecureBlockDecl,
    StateMachineDecl, StructDecl, TaskDecl, TraitDecl, TraitImplDecl, TriggerHandlerDecl,
    TriggerKind, TwinDecl, Visibility,
};
use spanda_ast::nodes::*;
use spanda_ast::robotics_decl::{CertifyDecl, FleetDecl, ProgramSafetyZoneDecl, SwarmDecl};
use std::collections::HashMap;

/// Type-check failure carrying structured diagnostics.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeCheckError {
    pub diagnostics: Vec<Diagnostic>,
}

pub fn type_check(program: &Program, host: &dyn TypeCheckHost) -> Result<(), TypeCheckError> {
    // Type check.
    //
    // Parameters:
    // - `program` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::type_check(program);

    // Produce check as the result.
    check(program, host)
}

pub fn check(program: &Program, host: &dyn TypeCheckHost) -> Result<(), TypeCheckError> {
    // Check input.
    //
    // Parameters:
    // - `program` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::check(program);

    // Produce new as the result.
    check_with_registry(program, &ModuleRegistry::new(), host)
}

pub fn check_with_registry(
    program: &Program,
    registry: &ModuleRegistry,
    host: &dyn TypeCheckHost,
) -> Result<(), TypeCheckError> {
    // Check with registry.
    //
    // Parameters:
    // - `program` — input value
    // - `registry` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::check_with_registry(program, registry);

    // Create mutable checker for accumulating results.
    let mut checker = TypeChecker::new(host);
    checker.module_registry = Some(registry.clone());
    checker.check_program(program);

    // Skip further work when errors is empty.
    if checker.errors.is_empty() {
        Ok(())
    } else {
        Err(TypeCheckError {
            diagnostics: checker.errors,
        })
    }
}

pub fn units_compatible(a: UnitKind, b: UnitKind) -> bool {
    // Units compatible.
    //
    // Parameters:
    // - `a` — input value
    // - `b` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::units_compatible(a, b);

    // Produce units compatible as the result.
    units::units_compatible(a, b)
}

fn physical_types_compatible(left: &SpandaType, right: &SpandaType) -> bool {
    // Physical types compatible.
    //
    // Parameters:
    // - `left` — input value
    // - `right` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::physical_types_compatible(left, right);

    // Compute cat l for the following logic.
    let cat_l = physical_category(left);
    let cat_r = physical_category(right);
    cat_l == cat_r && cat_l != units::PhysicalCategory::Scalar
}

fn result_number_for_physical(left: &SpandaType, right: &SpandaType) -> Option<SpandaType> {
    // Result number for physical.
    //
    // Parameters:
    // - `left` — input value
    // - `right` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::result_number_for_physical(left, right);

    // take this path when let SpandaType::Number { unit, .. } = left.
    if let SpandaType::Number { unit, .. } = left {
        return Some(SpandaType::Number { unit: *unit });
    }

    // Take this path when let SpandaType::Number { unit, .. } = right.
    if let SpandaType::Number { unit, .. } = right {
        return Some(SpandaType::Number { unit: *unit });
    }

    // Take this path when let SpandaType::Named { name } = left.
    if let SpandaType::Named { name } = left {
        // Emit output when named type default unit provides a unit.
        if let Some(unit) = named_type_default_unit(name) {
            return Some(SpandaType::Number { unit });
        }
    }

    // Take this path when let SpandaType::Named { name } = right.
    if let SpandaType::Named { name } = right {
        // Emit output when named type default unit provides a unit.
        if let Some(unit) = named_type_default_unit(name) {
            return Some(SpandaType::Number { unit });
        }
    }
    None
}

fn named_type_default_unit(name: &str) -> Option<UnitKind> {
    // Named type default unit.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::named_type_default_unit(name);

    // Produce canonical unit as the result.
    Some(units::canonical_unit(match name {
        "Distance" => units::PhysicalCategory::Distance,
        "Duration" => units::PhysicalCategory::Duration,
        "Velocity" => units::PhysicalCategory::Velocity,
        "Acceleration" => units::PhysicalCategory::Acceleration,
        "Angle" => units::PhysicalCategory::Angle,
        "AngularVelocity" => units::PhysicalCategory::AngularVelocity,
        "Mass" => units::PhysicalCategory::Mass,
        "Force" => units::PhysicalCategory::Force,
        "Power" => units::PhysicalCategory::Power,
        "Voltage" => units::PhysicalCategory::Voltage,
        "Current" => units::PhysicalCategory::Current,
        "Temperature" => units::PhysicalCategory::Temperature,
        "Pressure" => units::PhysicalCategory::Pressure,
        "Humidity" => units::PhysicalCategory::Humidity,
        "Illuminance" => units::PhysicalCategory::Illuminance,
        "Luminance" => units::PhysicalCategory::Luminance,
        "Concentration" => units::PhysicalCategory::Concentration,
        "SoundLevel" => units::PhysicalCategory::SoundLevel,
        "MagneticField" => units::PhysicalCategory::MagneticField,
        "RotationalSpeed" => units::PhysicalCategory::RotationalSpeed,
        "Torque" => units::PhysicalCategory::Torque,
        "Energy" => units::PhysicalCategory::Energy,
        "UvIndex" => units::PhysicalCategory::UvIndex,
        "Ph" => units::PhysicalCategory::Ph,
        "Conductivity" => units::PhysicalCategory::Conductivity,
        "ParticulateMatter" => units::PhysicalCategory::ParticulateMatter,
        "Turbidity" => units::PhysicalCategory::Turbidity,
        "Salinity" => units::PhysicalCategory::Salinity,
        "Radiation" => units::PhysicalCategory::Radiation,
        "SoilMoisture" => units::PhysicalCategory::SoilMoisture,
        _ => return None,
    }))
}

pub fn result_unit_for_binary(
    op: BinaryOp,
    left: &SpandaType,
    right: &SpandaType,
) -> Option<SpandaType> {
    // Result unit for binary.
    //
    // Parameters:
    // - `op` — input value
    // - `left` — input value
    // - `right` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::result_unit_for_binary(op, left, right);

    // Match on op and handle each case.
    match op {
        BinaryOp::And | BinaryOp::Or => {
            // Keep entries that match the expected pattern.
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
            // Keep entries that match the expected pattern.
            if matches!(left, SpandaType::Number { .. })
                && matches!(right, SpandaType::Number { .. })
            {
                let SpandaType::Number { unit: lu, .. } = left else {
                    unreachable!()
                };
                let SpandaType::Number { unit: ru, .. } = right else {
                    unreachable!()
                };

                // Take this path when units compatible(*lu, *ru).
                if units_compatible(*lu, *ru) {
                    return Some(SpandaType::Bool);
                }
            }

            // Keep entries that match the expected pattern.
            if matches!(left, SpandaType::Bool) && matches!(right, SpandaType::Bool) {
                return Some(SpandaType::Bool);
            }

            // Keep entries that match the expected pattern.
            if matches!(left, SpandaType::String) && matches!(right, SpandaType::String) {
                return Some(SpandaType::Bool);
            }

            // Take this path when physical types compatible(left, right).
            if physical_types_compatible(left, right) {
                return Some(SpandaType::Bool);
            }
            None
        }
        BinaryOp::Add | BinaryOp::Sub => {
            // Take this path when let (SpandaType::Number { unit: lu, .. }, SpandaType::Number { unit: r.
            if let (SpandaType::Number { unit: lu, .. }, SpandaType::Number { unit: ru, .. }) =
                (left, right)
            {
                // Take this path when units compatible(*lu, *ru).
                if units_compatible(*lu, *ru) {
                    let unit = if *lu != UnitKind::None { *lu } else { *ru };
                    return Some(SpandaType::Number { unit });
                }
            }

            // Take this path when physical types compatible(left, right).
            if physical_types_compatible(left, right) {
                return result_number_for_physical(left, right);
            }
            None
        }
        BinaryOp::Mul | BinaryOp::Div => {
            // Keep entries that match the expected pattern.
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

impl MethodSig {
    pub fn params(&self) -> &[SpandaType] {
        &self.params
    }

    pub fn named_params(&self) -> &HashMap<String, SpandaType> {
        &self.named_params
    }

    pub fn returns(&self) -> &SpandaType {
        &self.returns
    }
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

pub struct TypeChecker<'h> {
    host: &'h dyn TypeCheckHost,
    pub errors: Vec<Diagnostic>,
    symbols: HashMap<String, SymbolEntry>,
    enum_variants: HashMap<String, Vec<String>>,
    enum_payload_fields: HashMap<(String, String), Vec<String>>,
    variant_owner: HashMap<String, String>,
    struct_defs: HashMap<String, Vec<(String, String)>>,
    struct_type_params: HashMap<String, Vec<String>>,
    trait_defs: HashMap<String, HashMap<String, TraitMethodSig>>,
    agent_trait_methods: HashMap<String, HashMap<String, SpandaType>>,
    agent_traits: HashMap<String, std::collections::HashSet<String>>,
    state_machine_states: std::collections::HashSet<String>,
    message_registry: MessageRegistry,
    subscribed_topics: std::collections::HashSet<String>,
    agent_names: std::collections::HashSet<String>,
    device_names: std::collections::HashSet<String>,
    peer_robot_names: std::collections::HashSet<String>,
    module_registry: Option<ModuleRegistry>,
    module_functions: HashMap<String, ModuleFnDecl>,
    extern_functions: HashMap<String, ExternFnDecl>,
    type_param_scope: HashMap<String, SpandaType>,
    channel_payload_types: HashMap<String, SpandaType>,
    active_agent: Option<String>,
    program_fleets: bool,
    expected_return_type: Option<SpandaType>,
}

impl<'h> TypeChecker<'h> {
    pub fn new(host: &'h dyn TypeCheckHost) -> Self {
        // Create a new instance.
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
        // let value = spanda_core::types::new();

        // Assemble the struct fields and return it.
        Self {
            host,
            errors: Vec::new(),
            symbols: HashMap::new(),
            enum_variants: HashMap::new(),
            enum_payload_fields: HashMap::new(),
            variant_owner: HashMap::new(),
            struct_defs: HashMap::new(),
            struct_type_params: HashMap::new(),
            trait_defs: HashMap::new(),
            agent_trait_methods: HashMap::new(),
            agent_traits: HashMap::new(),
            state_machine_states: std::collections::HashSet::new(),
            message_registry: MessageRegistry::new(),
            subscribed_topics: std::collections::HashSet::new(),
            agent_names: std::collections::HashSet::new(),
            device_names: std::collections::HashSet::new(),
            peer_robot_names: std::collections::HashSet::new(),
            module_registry: None,
            module_functions: HashMap::new(),
            extern_functions: HashMap::new(),
            type_param_scope: HashMap::new(),
            channel_payload_types: HashMap::new(),
            active_agent: None,
            program_fleets: false,
            expected_return_type: None,
        }
    }

    pub fn check_program(&mut self, program: &Program) {
        // Check program.
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
        // let result = instance.check_program(program);

        // Destructure the program into its top-level sections.
        let Program::Program {
            imports,
            functions,
            extern_functions,
            tests,
            structs,
            enums,
            traits,
            messages,
            robots,
            fleets,
            swarms,
            program_safety_zones,
            certifications,
            ..
        } = program;
        let mut imported = std::collections::HashSet::new();

        // Process each import.
        for imp in imports {
            let ImportDecl::ImportDecl { path, span } = imp;
            let module_has = self
                .module_registry
                .as_ref()
                .is_some_and(|r| r.exports_for(path).is_some());
            let known =
                resolve_module_import(path) || self.host.import_path_known(path, module_has);

            // Take the branch when known is false.
            if !known {
                self.error(
                    format!("Unknown import '{path}'"),
                    span.start.line,
                    span.start.column,
                );
            } else {
                imported.insert(path.clone());

                // Emit output when module registry provides a registry.
                if let Some(registry) = &self.module_registry {
                    // Emit output when exports for provides a exports.
                    if let Some(exports) = registry.exports_for(path) {
                        // Iterate over functions with destructured elements.
                        for (fname, fdecl) in &exports.functions {
                            self.module_functions.insert(fname.clone(), fdecl.clone());
                        }
                    }
                }
            }
        }

        // Process each struct.
        for struct_decl in structs {
            self.check_struct(struct_decl);
        }

        // Process each enum.
        for enum_decl in enums {
            self.check_enum(enum_decl);
        }

        // Process each trait.
        for trait_decl in traits {
            self.check_trait(trait_decl);
        }
        self.check_extern_functions(extern_functions);
        self.check_module_functions(functions);

        // Run each test block in program order.
        for test in tests {
            // Execute each statement in sequence.
            for stmt in &test.body {
                self.check_stmt(stmt);
            }
        }
        self.message_registry = MessageRegistry::from_program(messages, structs);

        // Process each message.
        for msg in messages {
            self.check_message(msg);
        }

        let robot_names: Vec<String> = robots
            .iter()
            .map(|r| {
                let RobotDecl::RobotDecl { name, .. } = r;
                name.clone()
            })
            .collect();

        self.program_fleets = !fleets.is_empty();

        // Validate program-level fleet groupings against declared robots.
        for fleet in fleets {
            let FleetDecl::FleetDecl {
                name,
                members,
                span,
            } = fleet;
            if let Some(message) = self
                .host
                .validate_fleet_members(name, members, &robot_names)
            {
                self.error(message, span.start.line, span.start.column);
            }
        }

        let fleet_names: Vec<String> = fleets
            .iter()
            .map(|fleet| {
                let FleetDecl::FleetDecl { name, .. } = fleet;
                name.clone()
            })
            .collect();
        let mut swarm_names = std::collections::HashSet::new();
        for swarm in swarms {
            let SwarmDecl::SwarmDecl {
                name,
                fleet_name,
                span,
                ..
            } = swarm;
            if !swarm_names.insert(name.clone()) {
                self.error(
                    format!("Duplicate swarm '{name}'"),
                    span.start.line,
                    span.start.column,
                );
            }
            if let Some(message) = self
                .host
                .validate_swarm_fleet(name, fleet_name, &fleet_names)
            {
                self.error(message, span.start.line, span.start.column);
            }
        }

        // Register program-level safety zone policies for name uniqueness.
        let mut zone_names = std::collections::HashSet::new();
        for zone in program_safety_zones {
            let ProgramSafetyZoneDecl::ProgramSafetyZoneDecl { name, span, .. } = zone;
            if !zone_names.insert(name.clone()) {
                self.error(
                    format!("Duplicate safety_zone '{name}'"),
                    span.start.line,
                    span.start.column,
                );
            }
        }

        // Register program-level certification metadata for duplicate detection.
        let mut cert_standards = std::collections::HashSet::new();
        for cert in certifications {
            let CertifyDecl::CertifyDecl { standard, span, .. } = cert;
            let label = standard.as_str();
            if !cert_standards.insert(label) {
                self.error(
                    format!("Duplicate certify '{label}'"),
                    span.start.line,
                    span.start.column,
                );
            }
        }

        // Handle each robot declared in the program.
        for robot in robots {
            self.check_robot(robot, &imported);
        }
    }

    fn validate_type_annotation(&mut self, ty: &SpandaType, line: u32, column: u32) {
        // Validate type annotation.
        //
        // Parameters:
        // - `self` — method receiver
        // - `ty` — input value
        // - `line` — input value
        // - `column` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.validate_type_annotation(ty, line, column);

        // Match on ty and handle each case.
        match ty {
            SpandaType::Named { name } => {
                // Take this path when self.struct defs.contains key(name).
                if self.struct_defs.contains_key(name)
                    || self.type_param_scope.contains_key(name)
                    || self.enum_variants.contains_key(name)
                {
                    return;
                }

                // Take this path when resolve type name(name).is err().
                if resolve_type_name(name).is_err() {
                    self.error(format!("Unknown type '{name}'"), line, column);
                }
            }
            SpandaType::Generic { name, type_args } => {
                // Apply each command-line argument.
                for arg in type_args {
                    self.validate_type_annotation(arg, line, column);
                }

                // Take this path when resolve type name(name).is err() && generic arity(name).is none().
                if resolve_type_name(name).is_err() && generic_arity(name).is_none() {
                    self.error(format!("Unknown type '{name}'"), line, column);
                }
            }
            SpandaType::TraitObject { trait_name } if !self.trait_defs.contains_key(trait_name) => {
                self.error(format!("Unknown trait '{trait_name}'"), line, column);
            }
            _ => {}
        }
    }

    fn check_module_functions(&mut self, functions: &[ModuleFnDecl]) {
        // Check module functions.
        //
        // Parameters:
        // - `self` — method receiver
        // - `functions` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_module_functions(functions);

        // Generate code for each module function.
        for func in functions {
            let saved_scope = std::mem::take(&mut self.type_param_scope);

            // Process each type param.
            for tp in &func.type_params {
                self.type_param_scope
                    .insert(tp.clone(), SpandaType::TypeParam { name: tp.clone() });
            }

            // Bind each parameter before executing the body.
            for param in &func.params {
                self.validate_type_annotation(
                    &param.type_ann,
                    param.span.start.line,
                    param.span.start.column,
                );
                let resolved = self.resolve_type_ann(&param.type_ann);
                self.symbols.insert(
                    param.name.clone(),
                    SymbolEntry {
                        robo_type: resolved,
                        kind: SymbolKind::Variable,
                        sensor_type: None,
                        actuator_type: None,
                    },
                );
            }

            // Validate the function body against its declared return type.
            let expected_return = self.resolve_type_ann(&func.return_type);
            self.expected_return_type = Some(expected_return.clone());

            // Execute each statement in sequence.
            for stmt in &func.body {
                self.check_stmt(stmt);
            }

            self.expected_return_type = None;

            // Bind each parameter before executing the body.
            for param in &func.params {
                self.symbols.remove(&param.name);
            }

            // Keep entries that match the expected pattern.
            if matches!(func.visibility, Visibility::Export | Visibility::Public) {
                self.module_functions
                    .insert(func.name.clone(), func.clone());
            }
            let _ = self.resolve_type_ann(&func.return_type);
            self.type_param_scope = saved_scope;
        }
    }

    fn check_extern_functions(&mut self, functions: &[ExternFnDecl]) {
        // Check extern functions.
        //
        // Parameters:
        // - `self` — method receiver
        // - `functions` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_extern_functions(functions);

        // Generate code for each module function.
        for func in functions {
            // Bind each parameter before executing the body.
            for param in &func.params {
                self.validate_type_annotation(
                    &param.type_ann,
                    param.span.start.line,
                    param.span.start.column,
                );
            }
            let _ = self.resolve_type_ann(&func.return_type);
            self.extern_functions
                .insert(func.name.clone(), func.clone());
        }
    }

    fn future_type(inner: SpandaType) -> SpandaType {
        // Future type.
        //
        // Parameters:
        // - `inner` — input value
        //
        // Returns:
        // SpandaType.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::types::future_type(inner);

        // Produce Generic as the result.
        SpandaType::Generic {
            name: "Future".into(),
            type_args: vec![inner],
        }
    }

    fn task_handle_type(inner: SpandaType) -> SpandaType {
        // Task handle type.
        //
        // Parameters:
        // - `inner` — input value
        //
        // Returns:
        // SpandaType.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::types::task_handle_type(inner);

        // Produce Generic as the result.
        SpandaType::Generic {
            name: "TaskHandle".into(),
            type_args: vec![inner],
        }
    }

    fn resolve_type_ann(&self, ty: &SpandaType) -> SpandaType {
        // Resolve type ann.
        //
        // Parameters:
        // - `self` — method receiver
        // - `ty` — input value
        //
        // Returns:
        // SpandaType.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.resolve_type_ann(ty);

        // Match on ty and handle each case.
        match ty {
            SpandaType::Named { name } if self.type_param_scope.contains_key(name) => {
                self.type_param_scope[name].clone()
            }
            SpandaType::Generic { name, type_args } => SpandaType::Generic {
                name: name.clone(),
                type_args: type_args.iter().map(|a| self.resolve_type_ann(a)).collect(),
            },
            other => other.clone(),
        }
    }

    fn check_message(&mut self, decl: &comm::MessageDecl) {
        // Check message.
        //
        // Parameters:
        // - `self` — method receiver
        // - `decl` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_message(decl);

        // Compute comm for the following logic.
        let comm::MessageDecl::MessageDecl {
            name, fields, span, ..
        } = decl;

        // Check each struct field.
        for field in fields {
            let known = self
                .message_registry
                .resolve_type(&field.type_name)
                .is_some()
                || resolve_type_alias(&field.type_name).is_some()
                || crate::type_system::resolve_type_name(&field.type_name).is_ok();

            // Take the branch when known is false.
            if !known {
                self.error(
                    format!(
                        "Unknown field type '{}' in message '{name}'",
                        field.type_name
                    ),
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
        let _ = span;
    }

    fn check_struct(&mut self, decl: &StructDecl) {
        // Check struct.
        //
        // Parameters:
        // - `self` — method receiver
        // - `decl` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_struct(decl);

        // Compute StructDecl for the following logic.
        let StructDecl::StructDecl {
            name,
            type_params,
            fields,
            span,
        } = decl;

        // Skip further work when !type params is empty.
        if !type_params.is_empty() {
            self.struct_type_params
                .insert(name.clone(), type_params.clone());
        }

        // Check each struct field.
        for field in fields {
            let allowed_generic = type_params.contains(&field.type_name);

            // Take the branch when allowed generic is false.
            if !allowed_generic
                && resolve_type_alias(&field.type_name).is_none()
                && !matches!(
                    field.type_name.as_str(),
                    "Pose" | "Velocity" | "Scan" | "String" | "Bool" | "Path" | "Int" | "Float"
                )
                && !field.type_name.contains('<')
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
        // Check enum.
        //
        // Parameters:
        // - `self` — method receiver
        // - `decl` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_enum(decl);

        // Compute EnumDecl for the following logic.
        let EnumDecl::EnumDecl {
            name,
            variants,
            span,
        } = decl;

        // Skip further work when variants is empty.
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
        let variant_names: Vec<String> = variants.iter().map(|v| v.name.clone()).collect();
        self.enum_variants
            .insert(name.clone(), variant_names.clone());

        // Handle each enum variant arm.
        for variant in variants {
            // Skip further work when field types is empty.
            if !variant.field_types.is_empty() {
                self.enum_payload_fields.insert(
                    (name.clone(), variant.name.clone()),
                    variant.field_types.clone(),
                );
            }

            // Emit output when self provides a existing.
            if let Some(existing) = self
                .variant_owner
                .insert(variant.name.clone(), name.clone())
            {
                self.error(
                    format!(
                        "Enum variant '{}' already declared in enum '{existing}'",
                        variant.name
                    ),
                    span.start.line,
                    span.start.column,
                );
            }
        }
    }

    fn check_trait(&mut self, decl: &TraitDecl) {
        // Check trait.
        //
        // Parameters:
        // - `self` — method receiver
        // - `decl` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_trait(decl);

        // Compute TraitDecl for the following logic.
        let TraitDecl::TraitDecl {
            name,
            methods,
            span,
        } = decl;

        // Skip further work when methods is empty.
        if methods.is_empty() {
            self.error(
                format!("Trait '{name}' must declare at least one method"),
                span.start.line,
                span.start.column,
            );
        }
        let mut method_map = HashMap::new();

        // Process each method.
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
        // Type name to spanda.
        //
        // Parameters:
        // - `self` — method receiver
        // - `type_name` — input value
        //
        // Returns:
        // SpandaType.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.type_name_to_spanda(type_name);

        // Produce Named as the result.
        resolve_type_name(type_name).unwrap_or(SpandaType::Named {
            name: type_name.to_string(),
        })
    }

    fn check_robot(&mut self, robot: &RobotDecl, imported: &std::collections::HashSet<String>) {
        // Check robot.
        //
        // Parameters:
        // - `self` — method receiver
        // - `robot` — input value
        // - `imported` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_robot(robot, imported);

        // Compute RobotDecl for the following logic.
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
            agent_channels,
            twin_sync,
            mission,
            ..
        } = robot;
        self.subscribed_topics.clear();
        self.agent_names.clear();
        self.device_names.clear();
        self.peer_robot_names.clear();
        self.symbols.clear();
        self.state_machine_states.clear();
        self.agent_trait_methods.clear();

        if self.program_fleets {
            self.symbols.insert(
                "fleet".into(),
                SymbolEntry {
                    robo_type: SpandaType::Named {
                        name: "FleetCoordinator".into(),
                    },
                    kind: SymbolKind::Variable,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
        }

        // Expose SLAM adapter hooks when a SLAM-related import is present.
        if imported
            .iter()
            .any(|path| self.host.slam_import_known(path))
        {
            self.symbols.insert(
                "slam".into(),
                SymbolEntry {
                    robo_type: SpandaType::Named {
                        name: "Slam".into(),
                    },
                    kind: SymbolKind::Variable,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
        }

        // Process each key.
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

        // Process each key.
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

        // Emit output when soc provides a soc decl.
        if let Some(soc_decl) = soc {
            let SocDecl::SocDecl { profile, span } = soc_decl;

            // Take this path when get soc profile(profile).is none().
            if !self.host.soc_profile_known(profile) {
                self.error(
                    format!("Unknown SoC profile '{profile}'"),
                    span.start.line,
                    span.start.column,
                );
            }
        }

        // Take this path when let (Some(hal block), Some(soc decl)) = (hal, soc).
        if let (Some(hal_block), Some(soc_decl)) = (hal, soc) {
            let HalBlock::HalBlock { members, span, .. } = hal_block;
            let SocDecl::SocDecl { profile, .. } = soc_decl;
            {
                // Emit output when get soc profile provides a profile.
                for err in self.host.validate_hal_against_soc(profile, members) {
                    self.error(err, span.start.line, span.start.column);
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

        // Visit each AST node.
        for node in nodes {
            let NodeDecl::NodeDecl {
                namespace, span, ..
            } = node;

            // Take this path when namespace.is none().
            if namespace.is_none() {
                self.error(
                    "Node should specify namespace with 'on \"/namespace\"'".into(),
                    span.start.line,
                    span.start.column,
                );
            }
        }

        // Process each topic.
        for topic in topics {
            self.check_topic(topic);
        }

        // Process each service.
        for service in services {
            self.check_service(service);
        }

        // Process each action.
        for action in actions {
            self.check_action(action);
        }

        // Process each sensor.
        for sensor in sensors {
            self.check_sensor(sensor, imported, &hal_bus_names);
        }

        // Process each actuator.
        for actuator in actuators {
            self.check_actuator(actuator);
        }

        // Process each bus.
        for bus in buses {
            let comm::BusDecl::BusDecl {
                name,
                transport,
                encryption,
                authentication,
                span,
                ..
            } = bus;

            if comm::TransportKind::from_ident(name).is_none()
                && *transport == comm::TransportKind::Local
            {
                self.error(
                    format!("Unknown transport '{name}' in bus declaration"),
                    span.start.line,
                    span.start.column,
                );
            }

            for (field, value) in [
                ("encryption", encryption.as_deref()),
                ("authentication", authentication.as_deref()),
            ] {
                if let Some(v) = value {
                    let ok = match field {
                        "encryption" => ["none", "optional", "required"].contains(&v),
                        "authentication" => ["none", "signed", "mutual"].contains(&v),
                        _ => true,
                    };
                    if !ok {
                        self.error(
                            format!("invalid {field} mode '{v}' on bus '{name}'"),
                            span.start.line,
                            span.start.column,
                        );
                    }
                }
            }
        }

        // Process each peer robot.
        for peer in peer_robots {
            let comm::PeerRobotDecl::PeerRobotDecl { name, .. } = peer;
            self.peer_robot_names.insert(name.clone());
            self.symbols.insert(
                name.clone(),
                SymbolEntry {
                    robo_type: SpandaType::Named {
                        name: "PeerRobot".into(),
                    },
                    kind: SymbolKind::Robot,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
        }

        // Process each device.
        for device in devices {
            let comm::DeviceDecl::DeviceDecl {
                name,
                device_type,
                span,
            } = device;

            // Take the branch when ["Camera", "IMU", "Lidar", "GPS", "Microphone", "Speaker"] is false.
            if !["Camera", "IMU", "Lidar", "GPS", "Microphone", "Speaker"]
                .contains(&device_type.as_str())
            {
                self.error(
                    format!("Unknown device type '{device_type}'"),
                    span.start.line,
                    span.start.column,
                );
            }
            self.device_names.insert(name.clone());
            self.symbols.insert(
                name.clone(),
                SymbolEntry {
                    robo_type: SpandaType::Named {
                        name: device_type.clone(),
                    },
                    kind: SymbolKind::Sensor,
                    sensor_type: Some(device_type.clone()),
                    actuator_type: None,
                },
            );
        }

        // Emit output when twin sync provides a sync.
        if let Some(sync) = twin_sync {
            let comm::TwinSyncDecl::TwinSyncDecl {
                telemetry,
                replay,
                faults,
                events,
                span,
            } = sync;

            // Take the branch when value is false.
            if !(*telemetry || *replay || *faults || *events) {
                self.error(
                    "twin sync block must declare at least one of telemetry, replay, faults, or events"
                        .into(),
                    span.start.line,
                    span.start.column,
                );
            }
        }

        // Emit output when safety provides a safety block.
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

            // Process each rule.
            for rule in safety_block.rules() {
                self.check_safety_rule(rule);
            }

            // Process each zone.
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

        // Process each ai model.
        for model in ai_models {
            self.check_ai_model(model);
        }

        // Process each agent.
        for agent in agents {
            self.check_agent(agent);
        }

        // Process each agent channel.
        for channel in agent_channels {
            let comm::AgentChannelDecl::AgentChannelDecl {
                from_agent,
                to_agent,
                span,
                ..
            } = channel;

            // Check membership before continuing.
            if !self.agent_names.contains(from_agent) {
                self.error(
                    format!("Agent channel source '{from_agent}' is not declared"),
                    span.start.line,
                    span.start.column,
                );
            }

            // Check membership before continuing.
            if !self.agent_names.contains(to_agent) {
                self.error(
                    format!("Agent channel target '{to_agent}' is not declared"),
                    span.start.line,
                    span.start.column,
                );
            }
        }

        // Process each trait impl.
        for trait_impl in trait_impls {
            self.check_trait_impl(trait_impl);
        }

        // Process each state machine.
        for sm in state_machines {
            let StateMachineDecl::StateMachineDecl {
                name,
                states,
                transitions,
                span,
            } = sm;

            // Skip further work when states is empty.
            if states.is_empty() {
                self.error(
                    format!("State machine '{name}' must declare at least one state"),
                    span.start.line,
                    span.start.column,
                );
            }
            let state_set: std::collections::HashSet<_> = states.iter().cloned().collect();

            // Process each transition.
            for transition in transitions {
                // Check membership before continuing.
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

        // Emit output when twin provides a twin decl.
        if let Some(twin_decl) = twin {
            let TwinDecl::TwinDecl {
                name,
                mirrors,
                span,
                ..
            } = twin_decl;

            // Skip further work when mirrors is empty.
            if mirrors.is_empty() {
                self.error(
                    "Digital twin must mirror at least one field".into(),
                    span.start.line,
                    span.start.column,
                );
            }
            const ALLOWED_MIRROR_FIELDS: &[&str] =
                &["pose", "velocity", "battery", "status", "scan"];

            // Process each mirror.
            for mirror in mirrors {
                // Check membership before continuing.
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

        // Emit output when verify provides a verify decl.
        if let Some(verify_decl) = verify {
            let spanda_ast::foundations::VerifyDecl::VerifyDecl {
                rules,
                warnings,
                span,
            } = verify_decl;
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

            // Process each rule.
            for rule in rules {
                let t = self.check_expr(rule);

                // Keep entries that match the expected pattern.
                if !matches!(t, SpandaType::Bool) {
                    self.error(
                        "verify rule must be boolean".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
            }

            // Process each warning.
            for rule in warnings {
                let t = self.check_expr(rule);

                // Keep entries that match the expected pattern.
                if !matches!(t, SpandaType::Bool) {
                    self.error(
                        "verify warning must be boolean".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
            self.symbols = saved;
        }

        // Emit output when observe provides a observe decl.
        if let Some(observe_decl) = observe {
            let spanda_ast::foundations::ObserveDecl::ObserveDecl { sensors, span } = observe_decl;

            // Skip further work when sensors is empty.
            if sensors.is_empty() {
                self.error(
                    "observe block must list at least one sensor".into(),
                    span.start.line,
                    span.start.column,
                );
            }

            // Process each sensor.
            for sensor_name in sensors {
                // Take the branch when kind) differs from Sensor).
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

        // Validate mission declarations for duration or step sequences.
        if let Some(mission_decl) = mission {
            let MissionDecl::MissionDecl {
                name,
                duration_hours,
                steps,
                span,
                ..
            } = mission_decl;
            if let Some(message) = self
                .host
                .validate_mission_decl(name, *duration_hours, steps)
            {
                self.error(message, span.start.line, span.start.column);
            }
            self.symbols.insert(
                "mission".into(),
                SymbolEntry {
                    robo_type: SpandaType::Named {
                        name: "Mission".into(),
                    },
                    kind: SymbolKind::Variable,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
            self.symbols.insert(
                "navigation".into(),
                SymbolEntry {
                    robo_type: SpandaType::Named {
                        name: "Navigation".into(),
                    },
                    kind: SymbolKind::Variable,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
        }

        // Emit output when identity provides a identity decl.
        if let Some(identity_decl) = identity {
            let spanda_ast::foundations::IdentityDecl::IdentityDecl { fields, span, .. } =
                identity_decl;
            let has_id = fields.iter().any(|(k, _)| k == "id");

            // Take the branch when has id is false.
            if !has_id {
                self.error(
                    "identity block must declare an 'id' field".into(),
                    span.start.line,
                    span.start.column,
                );
            }
            self.symbols.insert(
                "identity".into(),
                SymbolEntry {
                    robo_type: SpandaType::Named {
                        name: "RobotIdentity".into(),
                    },
                    kind: SymbolKind::Variable,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
        }

        // Emit output when audit provides a audit decl.
        if let Some(audit_decl) = audit {
            let spanda_ast::foundations::AuditDecl::AuditDecl { records, span, .. } = audit_decl;

            // Skip further work when records is empty.
            if records.is_empty() {
                self.error(
                    "audit block must record at least one field".into(),
                    span.start.line,
                    span.start.column,
                );
            }
            self.symbols.insert(
                "audit".into(),
                SymbolEntry {
                    robo_type: SpandaType::Named {
                        name: "AuditLog".into(),
                    },
                    kind: SymbolKind::Variable,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
        }
        self.symbols.insert(
            String::from("mock_ledger"),
            SymbolEntry {
                robo_type: SpandaType::Named {
                    name: "MockLedger".into(),
                },
                kind: SymbolKind::Variable,
                sensor_type: None,
                actuator_type: None,
            },
        );
        self.symbols.insert(
            String::from("world_model"),
            SymbolEntry {
                robo_type: SpandaType::Named {
                    name: "WorldModel".into(),
                },
                kind: SymbolKind::Variable,
                sensor_type: None,
                actuator_type: None,
            },
        );

        // Emit output when provenance provides a provenance decl.
        if let Some(provenance_decl) = provenance {
            let spanda_ast::foundations::ProvenanceDecl::ProvenanceDecl {
                hash_algo,
                signed_by,
                span,
                ..
            } = provenance_decl;

            // Take the branch when hash algo differs from "sha256".
            if hash_algo != "sha256" {
                self.error(
                    format!("unsupported provenance hash algorithm '{hash_algo}' — only sha256 is supported in MVP"),
                    span.start.line,
                    span.start.column,
                );
            }

            // Skip further work when signed by is empty.
            if signed_by.is_empty() {
                self.error(
                    "provenance block must declare signed_by".into(),
                    span.start.line,
                    span.start.column,
                );
            }
        }

        // Process each signed record.
        for signed in signed_records {
            let spanda_ast::foundations::SignedRecordDecl::SignedRecordDecl {
                signed_by, span, ..
            } = signed;

            // Skip further work when signed by is empty.
            if signed_by.is_empty() {
                self.error(
                    "signed record must specify signed_by".into(),
                    span.start.line,
                    span.start.column,
                );
            }
        }

        // Process each secret.
        for secret in secrets {
            let spanda_ast::foundations::SecretDecl::SecretDecl { name, span, .. } = secret;

            // Skip further work when name is empty.
            if name.is_empty() {
                self.error(
                    "secret declaration requires a name".into(),
                    span.start.line,
                    span.start.column,
                );
            }
            self.symbols.insert(
                name.clone(),
                SymbolEntry {
                    robo_type: SpandaType::Named {
                        name: "Secret".into(),
                    },
                    kind: SymbolKind::Variable,
                    sensor_type: None,
                    actuator_type: None,
                },
            );
        }

        // Emit output when trust provides a trust decl.
        if let Some(trust_decl) = trust {
            let spanda_ast::foundations::TrustDecl::TrustDecl { level, span } = trust_decl;

            // Check membership before continuing.
            if !["untrusted", "restricted", "trusted", "certified"].contains(&level.as_str()) {
                self.error(
                    format!("unknown trust level '{level}'"),
                    span.start.line,
                    span.start.column,
                );
            }
        }

        // Emit output when permissions provides a perm decl.
        if let Some(perm_decl) = permissions {
            let spanda_ast::foundations::PermissionsDecl::PermissionsDecl {
                capabilities,
                span,
                ..
            } = perm_decl;

            // Skip further work when capabilities is empty.
            if capabilities.is_empty() {
                self.error(
                    "permissions block must grant at least one capability".into(),
                    span.start.line,
                    span.start.column,
                );
            }

            // Validate each requested capability.
            for cap in capabilities {
                // Take this path when spanda security::is known capability(cap).
                if self.host.security_capability_known(cap) {
                    continue;
                }
                self.error(
                    format!("unknown package capability '{cap}'"),
                    span.start.line,
                    span.start.column,
                );
            }
        }

        // Process each behavior.
        for behavior in behaviors {
            let BehaviorDecl::BehaviorDecl {
                name,
                requires,
                ensures,
                invariant,
                body,
                ..
            } = behavior;

            // Emit output when requires provides a req.
            if let Some(req) = requires {
                let t = self.check_expr(req);

                // Keep entries that match the expected pattern.
                if !matches!(t, SpandaType::Bool) {
                    self.error("requires clause must be boolean".into(), 0, 0);
                }
            }

            // Emit output when ensures provides a post.
            if let Some(post) = ensures {
                let t = self.check_expr(post);

                // Keep entries that match the expected pattern.
                if !matches!(t, SpandaType::Bool) {
                    self.error("ensures clause must be boolean".into(), 0, 0);
                }
            }

            // Emit output when invariant provides a inv.
            if let Some(inv) = invariant {
                let t = self.check_expr(inv);

                // Keep entries that match the expected pattern.
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

        // Process each task.
        for task in tasks {
            let TaskDecl::TaskDecl {
                name,
                priority: _priority,
                interval_ms,
                deadline_ms: _,
                jitter_ms_max: _,
                isolated: _isolated,
                requires,
                ensures,
                invariant,
                budget,
                body,
                span,
            } = task;

            // Validate declared timing and priority constraints.
            for diag in self.host.validate_task_timing(task) {
                self.error(diag.message, diag.line, diag.column);
            }
            for diag in self.host.validate_task_priority(task) {
                self.error(diag.message, diag.line, diag.column);
            }

            // Take this path when *interval ms <= 0.0.
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

            // Take this path when let Some(ResourceBudgetDecl::ResourceBudgetDecl.
            if let Some(ResourceBudgetDecl::ResourceBudgetDecl {
                battery_pct_max,
                memory_mb_max,
                cpu_pct_max,
                gpu_pct_max: _,
                network_mbps_max,
                storage_mb_max,
                ..
            }) = budget
            {
                for diag in self
                    .host
                    .validate_resource_budget(budget.as_ref().unwrap(), *span)
                {
                    self.error(diag.message, diag.line, diag.column);
                }
                let validate_non_negative =
                    |checker: &mut TypeChecker, label: &str, value: f64, line: u32, column: u32| {
                        // Take this path when value < 0.0.
                        if value < 0.0 {
                            checker.error(
                                format!("{label} budget must be non-negative"),
                                line,
                                column,
                            );
                        }
                    };

                // Emit output when battery pct max provides a v.
                if let Some(v) = battery_pct_max {
                    // Take this path when *v > 100.0.
                    if *v > 100.0 {
                        self.error(
                            "battery budget must be <= 100%".into(),
                            span.start.line,
                            span.start.column,
                        );
                    }
                    validate_non_negative(self, "battery", *v, span.start.line, span.start.column);
                }

                // Emit output when memory mb max provides a v.
                if let Some(v) = memory_mb_max {
                    validate_non_negative(self, "memory", *v, span.start.line, span.start.column);
                }

                // Emit output when cpu pct max provides a v.
                if let Some(v) = cpu_pct_max {
                    // Take this path when *v > 100.0.
                    if *v > 100.0 {
                        self.error(
                            "cpu budget must be <= 100%".into(),
                            span.start.line,
                            span.start.column,
                        );
                    }
                    validate_non_negative(self, "cpu", *v, span.start.line, span.start.column);
                }

                // Emit output when network mbps max provides a v.
                if let Some(v) = network_mbps_max {
                    validate_non_negative(self, "network", *v, span.start.line, span.start.column);
                }

                // Emit output when storage mb max provides a v.
                if let Some(v) = storage_mb_max {
                    validate_non_negative(self, "storage", *v, span.start.line, span.start.column);
                }
            }

            // Emit output when requires provides a req.
            if let Some(req) = requires {
                let t = self.check_expr(req);

                // Keep entries that match the expected pattern.
                if !matches!(t, SpandaType::Bool) {
                    self.error(
                        "requires clause must be boolean".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
            }

            // Emit output when ensures provides a post.
            if let Some(post) = ensures {
                let t = self.check_expr(post);

                // Keep entries that match the expected pattern.
                if !matches!(t, SpandaType::Bool) {
                    self.error(
                        "ensures clause must be boolean".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
            }

            // Emit output when invariant provides a inv.
            if let Some(inv) = invariant {
                let t = self.check_expr(inv);

                // Keep entries that match the expected pattern.
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

        // Invoke each registered handler.
        for handler in event_handlers {
            let EventHandlerDecl::EventHandlerDecl {
                event_name,
                body,
                span,
            } = handler;
            let event_declared = events.iter().any(|e| {
                let EventDecl::EventDecl { name, .. } = e;
                name == event_name
            });
            let topic_declared = topics.iter().any(|t| {
                let TopicDecl::TopicDecl { name, .. } = t;
                name == event_name
            });

            // Take the branch when event declared && !topic declared is false.
            if !event_declared && !topic_declared {
                self.error(
                    format!("No event or topic declared for handler '{event_name}'"),
                    span.start.line,
                    span.start.column,
                );
            }
            self.check_behavior(body);
        }
        self.check_trigger_handlers(trigger_handlers, events, topics, state_machines, agents);
    }

    fn check_trigger_handlers(
        &mut self,
        trigger_handlers: &[TriggerHandlerDecl],
        events: &[EventDecl],
        topics: &[TopicDecl],
        state_machines: &[StateMachineDecl],
        agents: &[AgentDecl],
    ) {
        // Check trigger handlers.
        //
        // Parameters:
        // - `self` — method receiver
        // - `trigger_handlers` — input value
        // - `events` — input value
        // - `topics` — input value
        // - `state_machines` — input value
        // - `agents` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_trigger_handlers(trigger_handlers, events, topics, state_machines, agents);

        // Compute event names for the following logic.
        let event_names: std::collections::HashSet<_> = events
            .iter()
            .map(|e| {
                let EventDecl::EventDecl { name, .. } = e;
                name.clone()
            })
            .collect();
        let topic_names: std::collections::HashSet<_> = topics
            .iter()
            .map(|t| {
                let TopicDecl::TopicDecl { name, .. } = t;
                name.clone()
            })
            .collect();
        let mut sm_states: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Process each state machine.
        for sm in state_machines {
            let StateMachineDecl::StateMachineDecl { states, .. } = sm;

            // Process each state.
            for state in states {
                sm_states.insert(state.clone());
            }
        }

        // Invoke each registered handler.
        for handler in trigger_handlers {
            let TriggerHandlerDecl::TriggerHandlerDecl {
                trigger_kind,
                body,
                span,
                ..
            } = handler;

            // Match on trigger kind and handle each case.
            match trigger_kind {
                TriggerKind::Event { name } => {
                    // Check membership before continuing.
                    if !event_names.contains(name) && !topic_names.contains(name) {
                        self.error(
                            format!("No event or topic declared for trigger handler '{name}'"),
                            span.start.line,
                            span.start.column,
                        );
                    }
                }
                TriggerKind::Message { topic } => {
                    // Check membership before continuing.
                    if !topic_names.contains(topic) {
                        self.error(
                            format!("No topic declared for message trigger '{topic}'"),
                            span.start.line,
                            span.start.column,
                        );
                    }
                }
                TriggerKind::Timer { interval_ms } => {
                    // Take this path when *interval ms <= 0.0.
                    if *interval_ms <= 0.0 {
                        self.error(
                            "Timer trigger interval must be positive".into(),
                            span.start.line,
                            span.start.column,
                        );
                    }
                }
                TriggerKind::Condition { expr, level: _ } => {
                    let ty = self.check_expr(expr);

                    // Take the branch when ty differs from Bool.
                    if ty != SpandaType::Bool {
                        self.error(
                            "Condition trigger expression must be boolean".into(),
                            span.start.line,
                            span.start.column,
                        );
                    }
                }
                TriggerKind::StateEntered { state } | TriggerKind::StateExited { state } => {
                    // Check membership before continuing.
                    if !sm_states.contains(state) {
                        self.error(
                            format!("Unknown state '{state}' in state trigger"),
                            span.start.line,
                            span.start.column,
                        );
                    }
                }
                TriggerKind::Safety { event: _ }
                | TriggerKind::Hardware { event: _ }
                | TriggerKind::Ai { event: _ }
                | TriggerKind::Verification { event: _ }
                | TriggerKind::Twin { event: _ }
                | TriggerKind::Connectivity { .. }
                | TriggerKind::Geofence { .. }
                | TriggerKind::SensorEvent { .. } => {}
                TriggerKind::LogMatch { pattern } | TriggerKind::MessageMatch { pattern, .. } => {
                    if let Err(err) = pattern.compile() {
                        self.error(err.message, err.line, err.column);
                    }
                }
            }
            self.check_behavior(body);
        }

        // Process each agent.
        for agent in agents {
            let AgentDecl::AgentDecl {
                name: agent_name,
                trigger_handlers,
                ..
            } = agent;

            // Invoke each registered handler.
            for handler in trigger_handlers {
                let TriggerHandlerDecl::TriggerHandlerDecl {
                    trigger_kind,
                    body,
                    span,
                    ..
                } = handler;

                // Match on trigger kind and handle each case.
                match trigger_kind {
                    TriggerKind::Message { topic }

                        // Check membership before continuing.
                        if !topic_names.contains(topic) && !event_names.contains(topic) =>
                    {
                        self.error(
                            format!(
                                "Agent '{agent_name}' trigger '{topic}' must reference a declared topic or event"
                            ),
                            span.start.line,
                            span.start.column,
                        );
                    }
                    _ => {}
                }
                self.check_behavior(body);
            }
        }
    }

    fn check_topic(&mut self, topic: &TopicDecl) {
        // Check topic.
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
        // let result = instance.check_topic(topic);

        // Compute TopicDecl for the following logic.
        let TopicDecl::TopicDecl {
            name,
            message_type,
            topic: topic_path,
            role,
            qos,
            transport,
            secure,
            span,
        } = topic;

        // Take this path when resolve message type(&self.message registry, message type).is none().
        if resolve_message_type(&self.message_registry, message_type).is_none() {
            self.error(
                format!("Unknown message type '{message_type}'"),
                span.start.line,
                span.start.column,
            );
        }

        // Keep entries that match the expected pattern.
        if topic_path.is_none() && transport.is_none() && matches!(role, comm::TopicRole::Publish) {
            self.error(
                format!("Topic '{name}' publisher must specify path or transport"),
                span.start.line,
                span.start.column,
            );
        }

        // Keep entries that match the expected pattern.
        if matches!(role, comm::TopicRole::Subscribe | comm::TopicRole::Both) {
            // Emit output when topic path provides a path.
            if let Some(path) = topic_path {
                self.subscribed_topics.insert(path.clone());
            }
            self.subscribed_topics.insert(name.clone());
        }

        // Emit output when qos provides a q.
        if let Some(q) = qos {
            // Emit output when rate hz provides a rate.
            if let Some(rate) = q.rate_hz {
                // Take this path when rate <= 0.0.
                if rate <= 0.0 {
                    self.error(
                        "Topic rate must be positive".into(),
                        q.span.start.line,
                        q.span.start.column,
                    );
                }
            }

            // Emit output when deadline ms provides a deadline.
            if let Some(deadline) = q.deadline_ms {
                // Take this path when deadline <= 0.0.
                if deadline <= 0.0 {
                    self.error(
                        "Topic deadline must be positive".into(),
                        q.span.start.line,
                        q.span.start.column,
                    );
                }
            }
        }

        // Emit output when secure provides a sec.
        if let Some(sec) = secure {
            self.check_secure_block(sec);
        }
        self.symbols.insert(
            name.clone(),
            SymbolEntry {
                robo_type: resolve_message_type(&self.message_registry, message_type)
                    .unwrap_or(SpandaType::Void),
                kind: SymbolKind::Topic,
                sensor_type: None,
                actuator_type: None,
            },
        );
    }

    fn check_service(&mut self, service: &ServiceDecl) {
        // Check service.
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
        // let result = instance.check_service(service);

        // Compute ServiceDecl for the following logic.
        let ServiceDecl::ServiceDecl {
            name,
            service_type,
            request_type,
            response_type,
            secure,
            span,
        } = service;

        // Take this path when let (Some(req), Some(res)) = (request type, response type).
        if let (Some(req), Some(res)) = (request_type, response_type) {
            // Take this path when resolve message type(&self.message registry, req).is none().
            if resolve_message_type(&self.message_registry, req).is_none() {
                self.error(
                    format!("Unknown service request type '{req}'"),
                    span.start.line,
                    span.start.column,
                );
            }

            // Take this path when resolve message type(&self.message registry, res).is none().
            if resolve_message_type(&self.message_registry, res).is_none() {
                self.error(
                    format!("Unknown service response type '{res}'"),
                    span.start.line,
                    span.start.column,
                );
            }
        } else if let Some(st) = service_type {
            if service_type_for(st).is_none()
                && resolve_message_type(&self.message_registry, st).is_none()
                && !st.starts_with("Service<")
            {
                self.error(
                    format!("Unknown service type '{st}'"),
                    span.start.line,
                    span.start.column,
                );
            }
        } else {
            self.error(
                format!("Service '{name}' must specify type or request/response"),
                span.start.line,
                span.start.column,
            );
        }

        // Emit output when secure provides a sec.
        if let Some(sec) = secure {
            self.check_secure_block(sec);
        }
        self.symbols.insert(
            name.clone(),
            SymbolEntry {
                robo_type: SpandaType::Named { name: name.clone() },
                kind: SymbolKind::Service,
                sensor_type: None,
                actuator_type: None,
            },
        );
    }

    fn check_action(&mut self, action: &ActionDecl) {
        // Check action.
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
        // let result = instance.check_action(action);

        // Compute ActionDecl for the following logic.
        let ActionDecl::ActionDecl {
            name,
            action_type,
            request_type,
            feedback_type,
            result_type,
            secure,
            span,
        } = action;

        // Take this path when let (Some(req), Some(fb), Some(res)) = (request type, feedback type, r.
        if let (Some(req), Some(fb), Some(res)) = (request_type, feedback_type, result_type) {
            // Iterate over [req, fb, res].
            for t in [req, fb, res] {
                // Take this path when resolve message type(&self.message registry, t).is none().
                if resolve_message_type(&self.message_registry, t).is_none() {
                    self.error(
                        format!("Unknown action type '{t}'"),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
        } else if let Some(at) = action_type {
            // Take this path when action type for(at).is none().
            if action_type_for(at).is_none() {
                self.error(
                    format!("Unknown action type '{at}'"),
                    span.start.line,
                    span.start.column,
                );
            }
        } else {
            self.error(
                format!("Action '{name}' must specify type or request/feedback/result"),
                span.start.line,
                span.start.column,
            );
        }

        // Emit output when secure provides a sec.
        if let Some(sec) = secure {
            self.check_secure_block(sec);
        }
        self.symbols.insert(
            name.clone(),
            SymbolEntry {
                robo_type: SpandaType::Named { name: name.clone() },
                kind: SymbolKind::Action,
                sensor_type: None,
                actuator_type: None,
            },
        );
    }

    fn check_secure_block(&mut self, block: &SecureBlockDecl) {
        // Check secure block.
        //
        // Parameters:
        // - `self` — method receiver
        // - `block` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_secure_block(block);

        // Validate encryption/authentication/integrity modes in secure blocks.
        for (field, value) in [
            ("encryption", block.encryption.as_deref()),
            ("authentication", block.authentication.as_deref()),
            ("integrity", block.integrity.as_deref()),
        ] {
            if let Some(v) = value {
                let ok = match field {
                    "encryption" => ["none", "optional", "required"].contains(&v),
                    "authentication" => ["none", "signed", "mutual"].contains(&v),
                    "integrity" => ["none", "required"].contains(&v),
                    _ => true,
                };
                if !ok {
                    self.error(
                        format!("invalid {field} mode '{v}' in secure block"),
                        block.span.start.line,
                        block.span.start.column,
                    );
                }
            }
        }

        // Emit output when min trust provides a level.
        if let Some(level) = &block.min_trust {
            // Check membership before continuing.
            if !["untrusted", "restricted", "trusted", "certified"].contains(&level.as_str()) {
                self.error(
                    format!("unknown trust level '{level}' in secure block"),
                    block.span.start.line,
                    block.span.start.column,
                );
            }
        }

        // Validate each requested capability.
        for cap in &block.requires {
            // Take the branch when is known capability is false.
            if !self.host.security_capability_known(cap) {
                self.error(
                    format!("unknown capability '{cap}' in secure block"),
                    block.span.start.line,
                    block.span.start.column,
                );
            }
        }
    }

    fn check_sensor(
        &mut self,
        sensor: &SensorDecl,
        imported: &std::collections::HashSet<String>,
        hal_bus_names: &std::collections::HashSet<String>,
    ) {
        // Check sensor.
        //
        // Parameters:
        // - `self` — method receiver
        // - `sensor` — input value
        // - `imported` — input value
        // - `hal_bus_names` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_sensor(sensor, imported, hal_bus_names);

        // Compute SensorDecl for the following logic.
        let SensorDecl::SensorDecl {
            name,
            sensor_type,
            library,
            binding,
            span,
            ..
        } = sensor;

        // Take this path when sensor type for(sensor type).is none().
        if sensor_type_for(sensor_type, self.host).is_none() {
            self.error(
                format!("Unknown sensor type '{sensor_type}'"),
                span.start.line,
                span.start.column,
            );
        }

        // Emit output when library provides a lib.
        if let Some(lib) = library {
            // Check membership before continuing.
            if !imported.contains(lib) {
                self.error(
                    format!("Library '{lib}' must be imported before use"),
                    span.start.line,
                    span.start.column,
                );
            }

            // Emit output when resolve import provides a module.
            if let Some(exports) = self.host.library_exports_sensor(lib, sensor_type) {
                if !exports {
                    self.error(
                        format!("Sensor type '{sensor_type}' not provided by library '{lib}'"),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
        }

        // Take this path when let Some(SensorBinding::Hal { bus name }) = binding.
        if let Some(SensorBinding::Hal { bus_name }) = binding {
            // Check membership before continuing.
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
                robo_type: sensor_type_for(sensor_type, self.host).unwrap_or(SpandaType::Named {
                    name: sensor_type.clone(),
                }),
                kind: SymbolKind::Sensor,
                sensor_type: Some(sensor_type.clone()),
                actuator_type: None,
            },
        );
    }

    fn check_actuator(&mut self, actuator: &ActuatorDecl) {
        // Check actuator.
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
        // let result = instance.check_actuator(actuator);

        // Compute ActuatorDecl for the following logic.
        let ActuatorDecl::ActuatorDecl {
            name,
            actuator_type,
            span,
            ..
        } = actuator;

        // Take this path when actuator type for(actuator type).is none().
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
        // Check safety rule.
        //
        // Parameters:
        // - `self` — method receiver
        // - `rule` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_safety_rule(rule);

        // Match on rule and handle each case.
        match rule {
            SafetyRule::MaxSpeedRule {
                value, unit, span, ..
            } => {
                let t = self.check_expr(value);

                // Keep entries that match the expected pattern.
                if !matches!(t, SpandaType::Number { .. }) || !units_compatible(t.unit(), *unit) {
                    self.error(
                        format!("Expected value with unit '{}' for max_speed", unit.as_str()),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
            SafetyRule::StopIfRule { condition, span } => {
                // Keep entries that match the expected pattern.
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
        // Check safety zone.
        //
        // Parameters:
        // - `self` — method receiver
        // - `zone` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_safety_zone(zone);

        // Compute SafetyZoneDecl for the following logic.
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

        // Keep entries that match the expected pattern.
        if !matches!(self.check_expr(x), SpandaType::Number { .. })
            || !matches!(self.check_expr(y), SpandaType::Number { .. })
        {
            self.error(
                "Zone coordinates must be numeric".into(),
                span.start.line,
                span.start.column,
            );
        }

        // Take the branch when *shape equals Circle.
        if *shape == ZoneShape::Circle {
            // Emit output when radius provides a r.
            if let Some(r) = radius {
                // Keep entries that match the expected pattern.
                if !matches!(self.check_expr(r), SpandaType::Number { .. }) {
                    self.error(
                        "Zone radius must be numeric".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
        }

        // Take the branch when *shape equals Rect.
        if *shape == ZoneShape::Rect {
            // Emit output when width provides a w.
            if let Some(w) = width {
                // Keep entries that match the expected pattern.
                if !matches!(self.check_expr(w), SpandaType::Number { .. }) {
                    self.error(
                        "Zone size must be numeric".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
            }

            // Emit output when height provides a h.
            if let Some(h) = height {
                // Keep entries that match the expected pattern.
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
        // Check trait impl.
        //
        // Parameters:
        // - `self` — method receiver
        // - `decl` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_trait_impl(decl);

        // Compute TraitImplDecl for the following logic.
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

        // Take the branch when kind) differs from Agent).
        if self.symbols.get(agent_name).map(|s| s.kind) != Some(SymbolKind::Agent) {
            self.error(
                format!("Trait impl target '{agent_name}' is not a declared agent"),
                span.start.line,
                span.start.column,
            );
            return;
        }
        let mut registered: Vec<(String, SpandaType)> = Vec::new();

        // Process each method.
        for method in methods {
            let Some((expected_params, expected_return)) = trait_methods.get(&method.name) else {
                self.error(
                    format!("Trait '{trait_name}' has no method '{}'", method.name),
                    method.span.start.line,
                    method.span.start.column,
                );
                continue;
            };

            // Take the branch when return type differs from *expected return.
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

            // Take the branch when len differs from len.
            if method.params.len() != expected_params.len() {
                self.error(
                    format!("Trait method '{}' parameter count mismatch", method.name),
                    method.span.start.line,
                    method.span.start.column,
                );
            }

            // Iterate over iter with destructured elements.
            for (actual, (pname, ptype)) in method.params.iter().zip(expected_params.iter()) {
                // Take the branch when name differs from type name != *ptype.
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

            // Bind each parameter before executing the body.
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

            // Execute each statement in sequence.
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

        // Iterate over registered with destructured elements.
        for (name, ret) in registered {
            agent_methods.insert(name, ret);
        }
        self.agent_traits
            .entry(agent_name.clone())
            .or_default()
            .insert(trait_name.clone());
    }

    fn check_ai_model(&mut self, model: &AiModelDecl) {
        // Check ai model.
        //
        // Parameters:
        // - `self` — method receiver
        // - `model` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_ai_model(model);

        // Compute AiModelDecl for the following logic.
        let AiModelDecl::AiModelDecl {
            name,
            model_type,
            span,
            ..
        } = model;

        // Take this path when ai model type for(model type).is none().
        if ai_model_type_for(model_type).is_none() {
            self.error(
                format!("Unknown AI model type '{model_type}'"),
                span.start.line,
                span.start.column,
            );
        }

        // Take this path when self.symbols.contains key(name).
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
        // Check capability.
        //
        // Parameters:
        // - `self` — method receiver
        // - `agent_name` — input value
        // - `cap` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_capability(agent_name, cap);

        // Compute allowed for the following logic.
        let allowed = [
            "read",
            "propose_motion",
            "summarize",
            "detect",
            "plan",
            "subscribe",
            "publish",
            "call",
            "execute",
            "discover",
        ];

        // Check membership before continuing.
        if !allowed.contains(&cap.action.as_str()) && !is_comm_capability(&cap.action) {
            self.error(
                format!("Unknown capability '{}'", cap.action),
                cap.span.start.line,
                cap.span.start.column,
            );
            return;
        }

        // Take the branch when action equals "read".
        if cap.action == "read"
            || cap.action == "subscribe"
            || cap.action == "publish"
            || cap.action == "call"
            || cap.action == "execute"
        {
            // Emit output when target provides a target.
            if let Some(target) = &cap.target {
                let valid = self.symbols.contains_key(target)
                    || self.peer_robot_names.contains(target)
                    || self.device_names.contains(target);

                // Take the branch when valid is false.
                if !valid {
                    self.error(
                        format!(
                            "Agent '{agent_name}' capability {}({target}) references unknown resource",
                            cap.action
                        ),
                        cap.span.start.line,
                        cap.span.start.column,
                    );
                }
            } else if cap.action == "read" || cap.action == "subscribe" || cap.action == "publish" {
                self.error(
                    format!(
                        "Agent '{agent_name}' {} capability requires a target",
                        cap.action
                    ),
                    cap.span.start.line,
                    cap.span.start.column,
                );
            }
        }
    }

    fn check_agent(&mut self, agent: &AgentDecl) {
        // Check agent.
        //
        // Parameters:
        // - `self` — method receiver
        // - `agent` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_agent(agent);

        // Compute AgentDecl for the following logic.
        let AgentDecl::AgentDecl {
            name,
            uses_ai,
            tools,
            capabilities,
            plan_body,
            span,
            ..
        } = agent;

        // Take this path when self.symbols.contains key(name).
        if self.symbols.contains_key(name) {
            self.error(
                format!("Duplicate agent name '{name}'"),
                span.start.line,
                span.start.column,
            );
        }

        // Iterate over uses ai.
        for model_name in uses_ai {
            let sym = self.symbols.get(model_name);

            // Take the branch when kind) differs from AiModel).
            if sym.map(|s| s.kind) != Some(SymbolKind::AiModel) {
                self.error(
                    format!("Agent '{name}' references unknown ai model '{model_name}'"),
                    span.start.line,
                    span.start.column,
                );
            }
        }

        // Process each tool.
        for tool in tools {
            // Take the branch when contains key is false.
            if !self.symbols.contains_key(tool) {
                self.error(
                    format!("Agent '{name}' references unknown tool '{tool}'"),
                    span.start.line,
                    span.start.column,
                );
            }
        }

        // Validate each requested capability.
        for cap in capabilities {
            self.check_capability(name, cap);
        }
        self.agent_names.insert(name.clone());
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
        let prev_agent = self.active_agent.clone();
        self.active_agent = Some(name.clone());

        // Execute each statement in sequence.
        for stmt in plan_body {
            self.check_stmt(stmt);
        }
        self.active_agent = prev_agent;
        self.symbols = saved;
    }

    fn check_behavior(&mut self, body: &[Stmt]) {
        // Check behavior.
        //
        // Parameters:
        // - `self` — method receiver
        // - `body` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_behavior(body);

        // Compute parent for the following logic.
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

        // Execute each statement in sequence.
        for stmt in body {
            self.check_stmt(stmt);
        }
        self.symbols = parent;
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        // Check stmt.
        //
        // Parameters:
        // - `self` — method receiver
        // - `stmt` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_stmt(stmt);

        // Match on stmt and handle each case.
        match stmt {
            Stmt::VarDecl {
                name,
                type_annotation,
                init,
                span,
            } => {
                // Emit output when type annotation provides a expected.
                if let Some(expected) = type_annotation {
                    self.validate_type_annotation(expected, span.start.line, span.start.column);
                }

                // Take this path when let (.
                if let (
                    Some(SpandaType::TraitObject { trait_name }),
                    Some(Expr::IdentExpr { name: agent, .. }),
                ) = (type_annotation.as_ref(), init.as_ref())
                {
                    // Take the branch when self is false.
                    if !self
                        .agent_traits
                        .get(agent)
                        .is_some_and(|traits| traits.contains(trait_name))
                    {
                        self.error(
                            format!("Agent '{agent}' does not implement trait '{trait_name}'"),
                            span.start.line,
                            span.start.column,
                        );
                    }
                }
                let trait_agent_ok = matches!(
                    (type_annotation.as_ref(), init.as_ref()),
                    (
                        Some(SpandaType::TraitObject { trait_name }),
                        Some(Expr::IdentExpr { name: agent, .. })
                    ) if self
                        .agent_traits
                        .get(agent)
                        .is_some_and(|traits| traits.contains(trait_name))
                );
                let inferred = init.as_ref().map(|e| self.check_expr(e));
                let t = match (type_annotation, inferred) {
                    (Some(expected), Some(_actual)) if trait_agent_ok => expected.clone(),
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
                // Keep entries that match the expected pattern.
                if !matches!(self.check_expr(condition), SpandaType::Bool) {
                    self.error(
                        "if condition must be boolean".into(),
                        span.start.line,
                        span.start.column,
                    );
                }

                // Iterate over then branch.
                for s in then_branch {
                    self.check_stmt(s);
                }

                // Emit output when else branch provides a else branch.
                if let Some(else_branch) = else_branch {
                    // Iterate over else branch.
                    for s in else_branch {
                        self.check_stmt(s);
                    }
                }
            }
            Stmt::LoopStmt { body, .. } => {
                // Iterate over body.
                for s in body {
                    self.check_stmt(s);
                }
            }
            Stmt::PublishStmt {
                topic_name,
                value,
                span,
            } => {
                // Emit output when cloned provides a topic.
                if let Some(topic) = self.symbols.get(topic_name).cloned() {
                    // Take the branch when kind differs from Topic.
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
                // Take the branch when kind) differs from Service).
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
                // Take the branch when kind) differs from Action).
                if self.symbols.get(action_name).map(|s| s.kind) != Some(SymbolKind::Action) {
                    self.error(
                        format!("Unknown action '{action_name}'"),
                        span.start.line,
                        span.start.column,
                    );
                } else {
                    let goal_t = self.check_expr(goal);

                    // Keep entries that match the expected pattern.
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
                // Check membership before continuing.
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
            Stmt::ReturnStmt { value, span } => {
                if let Some(v) = value {
                    let actual = self.check_expr(v);
                    if let Some(expected) = self.expected_return_type.clone() {
                        self.assert_compatible(
                            &expected,
                            &actual,
                            span.start.line,
                            span.start.column,
                        );
                    }
                } else if self.expected_return_type.is_some() {
                    self.error(
                        "return statement missing value for non-unit function".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
            }
            Stmt::ExpectCompileErrorStmt { .. } => {
                // compile-fail bodies are validated by the test runner, not program check
            }
            Stmt::SubscribeStmt {
                target,
                filter,
                span,
            } => {
                let (topic_name, _) = target.split_once('.').unwrap_or((target.as_str(), ""));

                // Take the branch when contains key is false.
                if !self.symbols.contains_key(topic_name)
                    && !self.peer_robot_names.contains(topic_name)
                {
                    self.error(
                        format!("Unknown subscribe target '{target}'"),
                        span.start.line,
                        span.start.column,
                    );
                }
                if let Some(filter) = filter {
                    if let Err(err) = filter.pattern.compile() {
                        self.error(err.message, err.line, err.column);
                    }
                }
                self.subscribed_topics.insert(target.clone());
            }
            Stmt::ExecuteStmt {
                action_name,
                goal,
                span,
            } => {
                // Take the branch when kind) differs from Action).
                if self.symbols.get(action_name).map(|s| s.kind) != Some(SymbolKind::Action) {
                    self.error(
                        format!("Unknown action '{action_name}'"),
                        span.start.line,
                        span.start.column,
                    );
                } else {
                    self.check_expr(goal);
                }
            }
            Stmt::DiscoverStmt { .. } => {}
            Stmt::ReceiveStmt {
                topic_name,
                var_name,
                span,
            } => {
                let (root, _) = topic_name
                    .split_once('.')
                    .unwrap_or((topic_name.as_str(), ""));

                // Take the branch when kind) differs from Topic).
                if self.symbols.get(topic_name).map(|s| s.kind) != Some(SymbolKind::Topic)
                    && !self.peer_robot_names.contains(root)
                {
                    self.error(
                        format!("Unknown topic '{topic_name}' for receive"),
                        span.start.line,
                        span.start.column,
                    );
                }
                let topic_type = self
                    .symbols
                    .get(topic_name)
                    .map(|s| s.robo_type.clone())
                    .unwrap_or(SpandaType::Void);
                self.symbols.insert(
                    var_name.clone(),
                    SymbolEntry {
                        robo_type: topic_type,
                        kind: SymbolKind::Variable,
                        sensor_type: None,
                        actuator_type: None,
                    },
                );
            }
            Stmt::SpawnStmt { callee, args, span } => {
                // Take this path when let Expr::IdentExpr { name, .. } = callee.
                if let Expr::IdentExpr { name, .. } = callee {
                    // Take the branch when contains key is false.
                    if !self.module_functions.contains_key(name) {
                        self.error(
                            format!("Unknown spawn target '{name}'"),
                            span.start.line,
                            span.start.column,
                        );
                    }
                }

                // Apply each command-line argument.
                for arg in args {
                    self.check_expr(arg);
                }
            }
            Stmt::SelectStmt { arms, .. } => {
                // Process each arm.
                for arm in arms {
                    self.check_expr(&arm.channel);

                    // Execute each statement in sequence.
                    for stmt in &arm.body {
                        self.check_stmt(stmt);
                    }
                }
            }
            Stmt::ParallelStmt { body, .. } => {
                // Execute each statement in sequence.
                for stmt in body {
                    self.check_stmt(stmt);
                }
                self.symbols.insert(
                    "_parallel".into(),
                    SymbolEntry {
                        robo_type: SpandaType::Named {
                            name: "ParallelResults".into(),
                        },
                        kind: SymbolKind::Variable,
                        sensor_type: None,
                        actuator_type: None,
                    },
                );
            }
            Stmt::EnterModeStmt { .. }
            | Stmt::UseFallbackStmt { .. }
            | Stmt::StopAllActuatorsStmt { .. }
            | Stmt::RunPipelineStmt { .. }
            | Stmt::NavigateStmt { .. } => {}
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> SpandaType {
        // Check expr.
        //
        // Parameters:
        // - `self` — method receiver
        // - `expr` — input value
        //
        // Returns:
        // SpandaType.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_expr(expr);

        // Match on expr and handle each case.
        match expr {
            Expr::LiteralExpr { value, .. } => match value {
                LiteralValue::Bool(_) => SpandaType::Bool,
                LiteralValue::Number(_) => SpandaType::Number {
                    unit: UnitKind::None,
                },
                LiteralValue::String(_) => SpandaType::String,
                LiteralValue::Null => SpandaType::Void,
                LiteralValue::Regex(pattern) => {
                    if let Err(err) = pattern.compile() {
                        self.error(err.message, err.line, err.column);
                    }
                    SpandaType::Regex
                }
            },
            Expr::UnitLiteralExpr { value: _, unit, .. } => SpandaType::Number { unit: *unit },
            Expr::IdentExpr { name, span } => {
                // Emit output when get provides a enum name.
                if let Some(enum_name) = self.variant_owner.get(name) {
                    return SpandaType::EnumVariant {
                        enum_name: enum_name.clone(),
                        variant: name.clone(),
                    };
                }

                // Emit output when get provides a sym.
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

                // Keep entries that match the expected pattern.
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

                // Emit output when result unit for binary provides a result.
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

                // Match on op and handle each case.
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

                // Take the branch when *op equals Not.
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
            Expr::ServiceCallExpr { service_name, span } => {
                // Take the branch when kind) differs from Service).
                if self.symbols.get(service_name).map(|s| s.kind) != Some(SymbolKind::Service) {
                    self.error(
                        format!("Unknown service '{service_name}'"),
                        span.start.line,
                        span.start.column,
                    );
                }
                SpandaType::Named {
                    name: "ServiceResponse".into(),
                }
            }
            Expr::ExecuteExpr {
                action_name,
                goal,
                span,
            } => {
                // Take the branch when kind) differs from Action).
                if self.symbols.get(action_name).map(|s| s.kind) != Some(SymbolKind::Action) {
                    self.error(
                        format!("Unknown action '{action_name}'"),
                        span.start.line,
                        span.start.column,
                    );
                } else {
                    self.check_expr(goal);
                }
                SpandaType::Named {
                    name: "ActionResult".into(),
                }
            }
            Expr::AwaitExpr { operand, span } => {
                let inner = self.check_expr(operand);

                // Take this path when let SpandaType::Generic { name, type args } = &inner.
                if let SpandaType::Generic { name, type_args } = &inner {
                    // Take the branch when name equals "Future".
                    if name == "Future" {
                        // Emit output when first provides a t.
                        if let Some(t) = type_args.first() {
                            return t.clone();
                        }
                    }
                }
                self.error(
                    "await requires a Future value".into(),
                    span.start.line,
                    span.start.column,
                );
                SpandaType::Void
            }
            Expr::SpawnExpr { callee, args, span } => {
                // Take this path when let Expr::IdentExpr { name, .. } = callee.as ref().
                if let Expr::IdentExpr { name, .. } = callee.as_ref() {
                    // Emit output when cloned provides a func.
                    if let Some(func) = self.module_functions.get(name).cloned() {
                        // Iterate over enumerate with destructured elements.
                        for (i, arg) in args.iter().enumerate() {
                            // Emit output when get provides a param.
                            if let Some(param) = func.params.get(i) {
                                let expected = self.resolve_type_ann(&param.type_ann);
                                let actual = self.check_expr(arg);
                                self.assert_compatible(
                                    &expected,
                                    &actual,
                                    span.start.line,
                                    span.start.column,
                                );
                            }
                        }
                        return Self::task_handle_type(self.resolve_type_ann(&func.return_type));
                    }
                    self.error(
                        format!("Unknown spawn target '{name}'"),
                        span.start.line,
                        span.start.column,
                    );
                } else {
                    self.error(
                        "spawn requires function name".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
                SpandaType::Void
            }
            Expr::DiscoverExpr { .. } => SpandaType::Named {
                name: "DiscoveryResult".into(),
            },
        }
    }

    fn check_struct_literal(
        &mut self,
        type_name: &str,
        fields: &[StructFieldInit],
        span: &Span,
    ) -> SpandaType {
        // Check struct literal.
        //
        // Parameters:
        // - `self` — method receiver
        // - `type_name` — input value
        // - `fields` — input value
        // - `span` — input value
        //
        // Returns:
        // SpandaType.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_struct_literal(type_name, fields, span);

        // Bind a local value for the next steps.
        let (base_name, type_arg_names) = split_instantiated_type_name(type_name);
        let Some(def) = self.struct_defs.get(&base_name).cloned() else {
            self.error(
                format!("Unknown struct type '{type_name}'"),
                span.start.line,
                span.start.column,
            );
            return SpandaType::Void;
        };
        let type_params = self
            .struct_type_params
            .get(&base_name)
            .cloned()
            .unwrap_or_default();

        // Take the branch when len differs from len.
        if type_params.len() != type_arg_names.len() {
            self.error(
                format!(
                    "Struct '{base_name}' expects {} type argument(s), got {}",
                    type_params.len(),
                    type_arg_names.len()
                ),
                span.start.line,
                span.start.column,
            );
            return SpandaType::Void;
        }
        let substitutions: HashMap<String, String> = type_params
            .into_iter()
            .zip(type_arg_names.iter().cloned())
            .collect();
        let mut provided = std::collections::HashSet::new();

        // Check each struct field.
        for field in fields {
            // Check membership before continuing.
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
                .map(|(_, t)| instantiate_type_name(t, &substitutions));
            let Some(expected) = expected else {
                self.error(
                    format!("Struct '{base_name}' has no field '{}'", field.name),
                    field.span.start.line,
                    field.span.start.column,
                );
                continue;
            };
            let expected = self.type_name_to_spanda(&expected);
            let actual = self.check_expr(&field.value);
            self.assert_compatible(
                &expected,
                &actual,
                field.span.start.line,
                field.span.start.column,
            );
        }

        // Iterate over def with destructured elements.
        for (name, _) in &def {
            // Check membership before continuing.
            if !provided.contains(name) {
                self.error(
                    format!("Missing struct field '{name}' in '{type_name}' literal"),
                    span.start.line,
                    span.start.column,
                );
            }
        }

        // Skip further work when type arg names is empty.
        if type_arg_names.is_empty() {
            SpandaType::Named { name: base_name }
        } else {
            SpandaType::Generic {
                name: base_name,
                type_args: type_arg_names
                    .iter()
                    .map(|arg| self.type_name_to_spanda(arg))
                    .collect(),
            }
        }
    }

    fn check_match(&mut self, scrutinee: &Expr, arms: &[MatchArm], span: &Span) -> SpandaType {
        // Check match.
        //
        // Parameters:
        // - `self` — method receiver
        // - `scrutinee` — input value
        // - `arms` — input value
        // - `span` — input value
        //
        // Returns:
        // SpandaType.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_match(scrutinee, arms, span);

        // Compute scrutinee type for the following logic.
        let scrutinee_type = self.check_expr(scrutinee);

        // Skip further work when arms is empty.
        if arms.is_empty() {
            self.error(
                "match expression requires at least one arm".into(),
                span.start.line,
                span.start.column,
            );
        }
        let scrutinee_enum = match &scrutinee_type {
            SpandaType::Named { name } => Some(name.clone()),
            _ => None,
        };

        // Process each arm.
        for arm in arms {
            // Skip further work when bindings is empty.
            if !arm.bindings.is_empty() {
                // Emit output when scrutinee enum provides a enum name.
                if let Some(enum_name) = &scrutinee_enum {
                    // Emit output when self provides a field types.
                    if let Some(field_types) = self
                        .enum_payload_fields
                        .get(&(enum_name.clone(), arm.variant.clone()))
                        .cloned()
                    {
                        // Take the branch when len differs from len.
                        if arm.bindings.len() != field_types.len() {
                            self.error(
                                format!(
                                    "Match arm '{}' expects {} binding(s), got {}",
                                    arm.variant,
                                    field_types.len(),
                                    arm.bindings.len()
                                ),
                                arm.span.start.line,
                                arm.span.start.column,
                            );
                        }

                        // Iterate over enumerate with destructured elements.
                        for (i, binding) in arm.bindings.iter().enumerate() {
                            // Emit output when get provides a type name.
                            if let Some(type_name) = field_types.get(i) {
                                self.symbols.insert(
                                    binding.clone(),
                                    SymbolEntry {
                                        robo_type: self.type_name_to_spanda(type_name),
                                        kind: SymbolKind::Variable,
                                        sensor_type: None,
                                        actuator_type: None,
                                    },
                                );
                            }
                        }
                    } else {
                        self.error(
                            format!("Variant '{}' has no payload bindings", arm.variant),
                            arm.span.start.line,
                            arm.span.start.column,
                        );
                    }
                }
            }

            // Execute each statement in sequence.
            for stmt in &arm.body {
                self.check_stmt(stmt);
            }

            // Process each binding.
            for binding in &arm.bindings {
                self.symbols.remove(binding);
            }
        }
        self.check_match_exhaustiveness(arms, &scrutinee_type, span);
        SpandaType::Void
    }

    fn check_match_exhaustiveness(
        &mut self,
        arms: &[MatchArm],
        scrutinee_type: &SpandaType,
        span: &Span,
    ) {
        // Check match exhaustiveness.
        //
        // Parameters:
        // - `self` — method receiver
        // - `arms` — input value
        // - `scrutinee_type` — input value
        // - `span` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_match_exhaustiveness(arms, scrutinee_type, span);

        // Import the items needed by the logic below.
        use std::collections::HashSet;
        let arm_names: HashSet<String> = arms.iter().map(|a| a.variant.clone()).collect();

        // Skip further work when arm names is empty.
        if arm_names.is_empty() {
            return;
        }

        // Take this path when let SpandaType::Generic { name, .. } = scrutinee type.
        if let SpandaType::Generic { name, .. } = scrutinee_type {
            // Take the branch when name equals "Result".
            if name == "Result" {
                // Iterate over ["Ok", "Err"].
                for required in ["Ok", "Err"] {
                    // Check membership before continuing.
                    if !arm_names.contains(required) {
                        self.error(
                            format!("Non-exhaustive match on Result: missing '{required}' arm"),
                            span.start.line,
                            span.start.column,
                        );
                    }
                }
                return;
            }

            // Take the branch when name equals "Option".
            if name == "Option" {
                // Iterate over ["Some", "None"].
                for required in ["Some", "None"] {
                    // Check membership before continuing.
                    if !arm_names.contains(required) {
                        self.error(
                            format!("Non-exhaustive match on Option: missing '{required}' arm"),
                            span.start.line,
                            span.start.column,
                        );
                    }
                }
                return;
            }
        }

        // Process each value.
        for variants in self.enum_variants.values() {
            let variant_set: HashSet<String> = variants.iter().cloned().collect();

            // Take this path when arm names.is subset(&variant set).
            if arm_names.is_subset(&variant_set) {
                // Take this path when arm names.len() < variant set.len().
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

    fn check_result_option_ctor(&mut self, name: &str, args: &[Expr], span: &Span) -> SpandaType {
        // Check result option ctor.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `args` — input value
        // - `span` — input value
        //
        // Returns:
        // SpandaType.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_result_option_ctor(name, args, span);

        // Match on name and handle each case.
        match name {
            "Ok" | "Some" => {
                // Emit output when first provides a arg.
                if let Some(arg) = args.first() {
                    let inner = self.check_expr(arg);
                    let ctor = if name == "Ok" { "Result" } else { "Option" };

                    // Take the branch when ctor equals "Result".
                    if ctor == "Result" {
                        SpandaType::Generic {
                            name: "Result".into(),
                            type_args: vec![
                                inner,
                                SpandaType::Named {
                                    name: "Error".into(),
                                },
                            ],
                        }
                    } else {
                        SpandaType::Generic {
                            name: "Option".into(),
                            type_args: vec![inner],
                        }
                    }
                } else {
                    self.error(
                        format!("'{name}' requires a value argument"),
                        span.start.line,
                        span.start.column,
                    );
                    SpandaType::Void
                }
            }
            "Err" => {
                let inner = args
                    .first()
                    .map(|a| self.check_expr(a))
                    .unwrap_or(SpandaType::Named {
                        name: "Error".into(),
                    });
                SpandaType::Generic {
                    name: "Result".into(),
                    type_args: vec![SpandaType::Void, inner],
                }
            }
            "None" => SpandaType::Generic {
                name: "Option".into(),
                type_args: vec![SpandaType::Void],
            },
            _ => SpandaType::Void,
        }
    }

    fn check_member(&mut self, object: &Expr, property: &str, span: &Span) -> SpandaType {
        // Check member.
        //
        // Parameters:
        // - `self` — method receiver
        // - `object` — input value
        // - `property` — input value
        // - `span` — input value
        //
        // Returns:
        // SpandaType.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_member(object, property, span);

        // take this path when let Expr::IdentExpr { name, .. } = object.
        if let Expr::IdentExpr { name, .. } = object {
            // Emit output when get provides a sym.
            if let Some(sym) = self.symbols.get(name) {
                // Take the branch when as deref equals Some.
                if sym.sensor_type.as_deref() == Some("Lidar") && property == "nearest_distance" {
                    return SpandaType::Number { unit: UnitKind::M };
                }
            }
        }
        let obj_type = self.check_expr(object);

        // Match on obj type and handle each case.
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
                // Emit output when get provides a variants.
                if let Some(variants) = self.enum_variants.get(name) {
                    // Take the branch when any equals property).
                    if variants.iter().any(|v| v == property) {
                        return SpandaType::EnumVariant {
                            enum_name: name.clone(),
                            variant: property.to_string(),
                        };
                    }
                }

                // Emit output when get provides a fields.
                if let Some(fields) = self.struct_defs.get(name) {
                    // Take the branch when find equals property).
                    if let Some((_, type_name)) = fields.iter().find(|(field, _)| field == property)
                    {
                        return self.type_name_to_spanda(type_name);
                    }
                }

                // Emit output when object property provides a prop.
                if let Some(prop) = object_property(name, property) {
                    return prop;
                }

                // Emit output when builtin methods provides a methods.
                if let Some(methods) = builtin_methods(name, self.host) {
                    // Emit output when get provides a method.
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
            SpandaType::Generic { name, type_args } => {
                // Emit output when get provides a fields.
                if let Some(fields) = self.struct_defs.get(name) {
                    // Take the branch when find equals property).
                    if let Some((_, type_name)) = fields.iter().find(|(field, _)| field == property)
                    {
                        let type_params = self
                            .struct_type_params
                            .get(name)
                            .cloned()
                            .unwrap_or_default();
                        let substitutions: HashMap<String, String> = type_params
                            .into_iter()
                            .zip(type_args.iter().map(type_name_from_spanda))
                            .collect();
                        return self.type_name_to_spanda(&instantiate_type_name(
                            type_name,
                            &substitutions,
                        ));
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
        // Check call.
        //
        // Parameters:
        // - `self` — method receiver
        // - `callee` — input value
        // - `args` — input value
        // - `named_args` — input value
        // - `span` — input value
        //
        // Returns:
        // SpandaType.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_call(callee, args, named_args, span);

        // take this path when let Expr::IdentExpr { name, .. } = callee.
        if let Expr::IdentExpr { name, .. } = callee {
            // Take the branch when name equals "channel".
            if name == "channel" {
                // Skip further work when !args is empty.
                if !args.is_empty() || !named_args.is_empty() {
                    self.error(
                        "channel takes no arguments".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
                return SpandaType::Named {
                    name: "Channel".into(),
                };
            }

            // Take the branch when name equals "send".
            if name == "send" {
                // Take this path when args.len() < 2.
                if args.len() < 2 {
                    self.error(
                        "send requires (channel, value)".into(),
                        span.start.line,
                        span.start.column,
                    );
                    return SpandaType::Void;
                }
                let channel_ty = self.check_expr(&args[0]);

                // Keep entries that match the expected pattern.
                if !matches!(channel_ty, SpandaType::Named { ref name } if name == "Channel") {
                    self.error(
                        "send first argument must be Channel".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
                let payload_ty = self.check_expr(&args[1]);

                // Take this path when let Expr::IdentExpr.
                if let Expr::IdentExpr {
                    name: channel_name, ..
                } = &args[0]
                {
                    // Emit output when cloned provides a existing.
                    if let Some(existing) = self.channel_payload_types.get(channel_name).cloned() {
                        self.assert_compatible(
                            &existing,
                            &payload_ty,
                            span.start.line,
                            span.start.column,
                        );
                    } else {
                        self.channel_payload_types
                            .insert(channel_name.clone(), payload_ty.clone());
                    }
                }
                return SpandaType::Void;
            }

            // Take the branch when name equals "recv".
            if name == "recv" {
                // Skip further work when args is empty.
                if args.is_empty() {
                    self.error(
                        "recv requires (channel)".into(),
                        span.start.line,
                        span.start.column,
                    );
                    return SpandaType::Void;
                }
                let channel_ty = self.check_expr(&args[0]);

                // Keep entries that match the expected pattern.
                if !matches!(channel_ty, SpandaType::Named { ref name } if name == "Channel") {
                    self.error(
                        "recv argument must be Channel".into(),
                        span.start.line,
                        span.start.column,
                    );
                }

                // Take this path when let Expr::IdentExpr.
                if let Expr::IdentExpr {
                    name: channel_name, ..
                } = &args[0]
                {
                    // Emit output when get provides a existing.
                    if let Some(existing) = self.channel_payload_types.get(channel_name) {
                        return existing.clone();
                    }
                }
                return SpandaType::Void;
            }

            if name == "geo" {
                if args.len() != 2 {
                    self.error(
                        "geo requires (latitude, longitude)".into(),
                        span.start.line,
                        span.start.column,
                    );
                } else {
                    self.check_expr(&args[0]);
                    self.check_expr(&args[1]);
                }
                return SpandaType::Named {
                    name: "GeoPoint".into(),
                };
            }

            // Take the branch when name equals "join".
            if name == "join" {
                // Skip further work when args is empty.
                if args.is_empty() {
                    self.error(
                        "join requires (handle)".into(),
                        span.start.line,
                        span.start.column,
                    );
                    return SpandaType::Void;
                }
                let joined = self.check_expr(&args[0]);

                // Take this path when let SpandaType::Generic { name, type args } = &joined.
                if let SpandaType::Generic { name, type_args } = &joined {
                    // Take the branch when name equals "Future" || name == "TaskHandle".
                    if name == "Future" || name == "TaskHandle" {
                        // Emit output when first provides a inner.
                        if let Some(inner) = type_args.first() {
                            return inner.clone();
                        }
                    }
                }
                self.error(
                    "join requires a Future or TaskHandle value".into(),
                    span.start.line,
                    span.start.column,
                );
                return SpandaType::Void;
            }

            // Take the branch when name equals "send agent".
            if name == "send_agent" {
                // Take this path when args.len() < 2.
                if args.len() < 2 {
                    self.error(
                        "send_agent requires (to, value)".into(),
                        span.start.line,
                        span.start.column,
                    );
                    return SpandaType::Void;
                }

                // Take this path when self.active agent.is none().
                if self.active_agent.is_none() {
                    self.error(
                        "send_agent requires active agent context".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
                self.check_expr(&args[1]);
                return SpandaType::Void;
            }

            // Take the branch when name equals "recv agent".
            if name == "recv_agent" {
                // Take this path when self.active agent.is none().
                if self.active_agent.is_none() {
                    self.error(
                        "recv_agent requires active agent context".into(),
                        span.start.line,
                        span.start.column,
                    );
                }
                return SpandaType::Void;
            }

            // Take the branch when name equals "peer send".
            if name == "peer_send" {
                // Take this path when args.len() < 3.
                if args.len() < 3 {
                    self.error(
                        "peer_send requires (peer, topic, value)".into(),
                        span.start.line,
                        span.start.column,
                    );
                    return SpandaType::Void;
                }
                self.check_expr(&args[2]);
                return SpandaType::Void;
            }

            // Emit output when cloned provides a func.
            if let Some(func) = self.module_functions.get(name.as_str()).cloned() {
                let mut call_scope = HashMap::new();

                // Iterate over enumerate with destructured elements.
                for (i, tp) in func.type_params.iter().enumerate() {
                    // Emit output when get provides a arg.
                    if let Some(arg) = args.get(i) {
                        call_scope.insert(tp.clone(), self.check_expr(arg));
                    }
                }
                let saved = std::mem::take(&mut self.type_param_scope);
                self.type_param_scope.extend(call_scope);

                // Iterate over enumerate with destructured elements.
                for (i, arg) in args.iter().enumerate() {
                    // Emit output when get provides a param.
                    if let Some(param) = func.params.get(i) {
                        let expected = self.resolve_type_ann(&param.type_ann);
                        let actual = self.check_expr(arg);
                        self.assert_compatible(
                            &expected,
                            &actual,
                            span.start.line,
                            span.start.column,
                        );
                    }
                }
                let ret = self.resolve_type_ann(&func.return_type);
                self.type_param_scope = saved;

                // Skip synchronous handling for async functions.
                if func.is_async {
                    return Self::future_type(ret);
                }
                return ret;
            }

            // Emit output when cloned provides a func.
            if let Some(func) = self.extern_functions.get(name.as_str()).cloned() {
                // Iterate over enumerate with destructured elements.
                for (i, arg) in args.iter().enumerate() {
                    // Emit output when get provides a param.
                    if let Some(param) = func.params.get(i) {
                        let expected = self.resolve_type_ann(&param.type_ann);
                        let actual = self.check_expr(arg);
                        self.assert_compatible(
                            &expected,
                            &actual,
                            span.start.line,
                            span.start.column,
                        );
                    }
                }
                return self.resolve_type_ann(&func.return_type);
            }

            // Take the branch when name equals "assert".
            if name == "assert" {
                // Emit output when first provides a arg.
                if let Some(arg) = args.first() {
                    let t = self.check_expr(arg);

                    // Keep entries that match the expected pattern.
                    if !matches!(t, SpandaType::Bool) {
                        self.error(
                            "assert requires a boolean condition".into(),
                            span.start.line,
                            span.start.column,
                        );
                    }
                }
                return SpandaType::Void;
            }

            // Keep entries that match the expected pattern.
            if matches!(name.as_str(), "Ok" | "Err" | "Some" | "None") {
                return self.check_result_option_ctor(name, args, span);
            }

            // Emit output when cloned provides a enum name.
            if let Some(enum_name) = self.variant_owner.get(name).cloned() {
                let key = (enum_name.clone(), name.clone());

                // Emit output when cloned provides a field types.
                if let Some(field_types) = self.enum_payload_fields.get(&key).cloned() {
                    // Take the branch when len differs from len.
                    if args.len() != field_types.len() {
                        self.error(
                            format!(
                                "Variant '{name}' expects {} payload argument(s), got {}",
                                field_types.len(),
                                args.len()
                            ),
                            span.start.line,
                            span.start.column,
                        );
                    }

                    // Iterate over enumerate with destructured elements.
                    for (i, arg) in args.iter().enumerate() {
                        // Emit output when get provides a type name.
                        if let Some(type_name) = field_types.get(i) {
                            let expected = self.type_name_to_spanda(type_name);
                            let actual = self.check_expr(arg);
                            self.assert_compatible(
                                &expected,
                                &actual,
                                span.start.line,
                                span.start.column,
                            );
                        }
                    }
                    return SpandaType::Named {
                        name: enum_name.clone(),
                    };
                }

                // Take this path when self.
                if self
                    .enum_variants
                    .get(&enum_name)
                    .is_some_and(|variants| variants.iter().any(|v| v == name))
                {
                    // Skip further work when !args is empty.
                    if !args.is_empty() {
                        self.error(
                            format!("Unit variant '{name}' takes no arguments"),
                            span.start.line,
                            span.start.column,
                        );
                    }
                    return SpandaType::Named {
                        name: enum_name.clone(),
                    };
                }
            }

            // Emit output when as str provides a sig.
            if let Some(sig) = builtin_functions().get(name.as_str()) {
                // Apply each command-line argument.
                for arg in named_args {
                    // Emit output when name) provides a expected.
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
        // Handle any remaining cases.
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
        // Handle any remaining cases.
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

        // Take the branch when kind equals Robot.
        if sym.kind == SymbolKind::Robot {
            // Emit output when as str provides a method.
            if let Some(method) = robot_methods().get(property.as_str()) {
                // Iterate over enumerate with destructured elements.
                for (i, arg) in args.iter().enumerate() {
                    // Emit output when get provides a expected.
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

        // Take the branch when kind equals Agent.
        if sym.kind == SymbolKind::Agent {
            // Emit output when get provides a methods.
            if let Some(methods) = self.agent_trait_methods.get(target_name) {
                // Emit output when as str provides a return type.
                if let Some(return_type) = methods.get(property.as_str()) {
                    return return_type.clone();
                }
            }
        }

        // Take this path when let SpandaType::TraitObject { trait name } = &sym.robo type.
        if let SpandaType::TraitObject { trait_name } = &sym.robo_type {
            // Emit output when get provides a methods.
            if let Some(methods) = self.trait_defs.get(trait_name) {
                // Take this path when let Some(( , return type)) = methods.get(property.as str()).
                if let Some((_, return_type)) = methods.get(property.as_str()) {
                    return self.type_name_to_spanda(return_type);
                }
            }
            self.error(
                format!("Unknown trait method '{property}' on '{trait_name}'"),
                span.start.line,
                span.start.column,
            );
            return SpandaType::Void;
        }
        let type_name = if sym.robo_type == SpandaType::String {
            "String".into()
        } else {
            match sym.kind {
                SymbolKind::Sensor => sym.sensor_type.clone().unwrap_or_default(),
                SymbolKind::Actuator => sym.actuator_type.clone().unwrap_or_default(),
                SymbolKind::Safety => "Safety".into(),
                SymbolKind::AiModel => {
                    // Take this path when let SpandaType::Named { name } = sym.robo type.
                    if let SpandaType::Named { name } = sym.robo_type {
                        name
                    } else {
                        String::new()
                    }
                }
                SymbolKind::Agent => "Agent".into(),
                _ => {
                    // Take this path when let SpandaType::Named { name } = sym.robo type.
                    if let SpandaType::Named { name } = sym.robo_type {
                        name
                    } else if sym.robo_type == SpandaType::Regex {
                        "Regex".into()
                    } else {
                        String::new()
                    }
                }
            }
        };

        // Take the branch when type name equals "LLM" && property == "drive".
        if type_name == "LLM" && property == "drive" {
            self.error(
                "AI models cannot control actuators directly — use reason(), safety.validate(), then actuator.execute()".into(),
                span.start.line,
                span.start.column,
            );
        }
        let Some(methods) = builtin_methods(&type_name, self.host) else {
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

        // Apply each command-line argument.
        for arg in named_args {
            // Emit output when name) provides a expected.
            if let Some(expected) = method.named_params.get(&arg.name) {
                // Take the branch when type name equals name == "field".
                if type_name == "Twin" && arg.name == "field" {
                    // Take this path when let Expr::IdentExpr { name, span } = &arg.value.
                    if let Expr::IdentExpr { name, span } = &arg.value {
                        const ALLOWED: &[&str] = &["pose", "velocity", "battery", "status", "scan"];

                        // Check membership before continuing.
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

        // Apply each command-line argument.
        for arg in args {
            let actual = self.check_expr(arg);

            // Take the branch when type name equals "Safety" && property == "validate" && !is action proposal type.
            if type_name == "Safety" && property == "validate" && !is_action_proposal_type(&actual)
            {
                self.error(
                    "safety.validate() expects ActionProposal".into(),
                    span.start.line,
                    span.start.column,
                );
            }

            // Take the branch when type name equals "DifferentialDrive" && property == "execute".
            if type_name == "DifferentialDrive" && property == "execute" {
                // Take this path when is action proposal type(&actual).
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

            // Take the branch when type name equals "VisionModel" && property == "detect".
            if type_name == "VisionModel" && property == "detect" {
                self.assert_named_type(&actual, "CameraFrame", span.start.line, span.start.column);
            }
        }
        method.returns.clone()
    }

    fn types_compatible(&self, expected: &SpandaType, actual: &SpandaType) -> bool {
        // Types compatible.
        //
        // Parameters:
        // - `self` — method receiver
        // - `expected` — input value
        // - `actual` — input value
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.types_compatible(expected, actual);

        // take the branch when discriminant equals discriminant.
        if std::mem::discriminant(expected) == std::mem::discriminant(actual) {
            // Match on value and handle each case.
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
                    SpandaType::Number { unit, .. }

                        // Take the branch when unit category equals Velocity.
                        if units::unit_category(*unit) == crate::units::PhysicalCategory::Velocity
                )
            || matches!(actual, SpandaType::Velocity)
                && matches!(
                    expected,
                    SpandaType::Number { unit, .. }

                        // Take the branch when unit category equals Velocity.
                        if units::unit_category(*unit) == crate::units::PhysicalCategory::Velocity
                )
        {
            true
        } else if let (SpandaType::Named { name }, SpandaType::Number { unit, .. }) =
            (expected, actual)
        {
            unit_matches_named_type(name, *unit)
        } else if let (SpandaType::Named { name }, SpandaType::String) = (expected, actual) {
            name == "Goal"
        } else if let (SpandaType::String, SpandaType::Named { name }) = (expected, actual) {
            name == "Goal"
        } else if matches!(expected, SpandaType::TypeParam { .. })
            || matches!(actual, SpandaType::TypeParam { .. })
            || (matches!(actual, SpandaType::Number { .. })
                && matches!(
                    expected,
                    SpandaType::Int | SpandaType::Float | SpandaType::TypeParam { .. }
                ))
        {
            true
        } else if let (
            SpandaType::Generic {
                name: en,
                type_args: ea,
            },
            SpandaType::Generic {
                name: an,
                type_args: aa,
            },
        ) = (expected, actual)
        {
            en == an && ea.len() == aa.len()
        } else {
            false
        }
    }

    fn assert_named_type(&mut self, actual: &SpandaType, type_name: &str, line: u32, column: u32) {
        // Assert named type.
        //
        // Parameters:
        // - `self` — method receiver
        // - `actual` — input value
        // - `type_name` — input value
        // - `line` — input value
        // - `column` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.assert_named_type(actual, type_name, line, column);

        // take this path when let SpandaType::Named { name } = actual.
        if let SpandaType::Named { name } = actual {
            // Take the branch when name equals type name.
            if name == type_name {
                return;
            }
        }
        self.error(
            format!("Expected {type_name}, found {}", display_type(actual)),
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
        // Assert compatible.
        //
        // Parameters:
        // - `self` — method receiver
        // - `expected` — input value
        // - `actual` — input value
        // - `line` — input value
        // - `column` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.assert_compatible(expected, actual, line, column);

        // keep entries that match the expected pattern.
        if matches!(expected, SpandaType::Void) && matches!(actual, SpandaType::Void) {
            return;
        }

        // Take the branch when types compatible is false.
        if !self.types_compatible(expected, actual) {
            // Take this path when let (SpandaType::Number { unit: eu, .. }, SpandaType::Number { unit: a.
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
                        "Type mismatch: expected {}, found {}",
                        display_type(expected),
                        display_type(actual)
                    ),
                    line,
                    column,
                );
            }
        }
    }

    fn error(&mut self, message: String, line: u32, column: u32) {
        // Error.
        //
        // Parameters:
        // - `self` — method receiver
        // - `message` — input value
        // - `line` — input value
        // - `column` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.error(message, line, column);

        // Append into self.
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

fn split_instantiated_type_name(type_name: &str) -> (String, Vec<String>) {
    // Split instantiated type name.
    //
    // Parameters:
    // - `type_name` — input value
    //
    // Returns:
    // (String, Vec<String>).
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::split_instantiated_type_name(type_name);

    // use lt when find is present.

    // Emit output when find provides a lt.
    if let Some(lt) = type_name.find('<') {
        // Take this path when type name.ends with('>').
        if type_name.ends_with('>') {
            let base = type_name[..lt].trim().to_string();
            let args = type_name[lt + 1..type_name.len() - 1]
                .split(',')
                .map(|part| part.trim().to_string())
                .filter(|part| !part.is_empty())
                .collect();
            return (base, args);
        }
    }
    (type_name.to_string(), Vec::new())
}

fn instantiate_type_name(type_name: &str, substitutions: &HashMap<String, String>) -> String {
    // Instantiate type name.
    //
    // Parameters:
    // - `type_name` — input value
    // - `substitutions` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::instantiate_type_name(type_name, substitutions);

    // Produce substitutions as the result.
    substitutions
        .get(type_name)
        .cloned()
        .unwrap_or_else(|| type_name.to_string())
}

fn type_name_from_spanda(ty: &SpandaType) -> String {
    // Type name from spanda.
    //
    // Parameters:
    // - `ty` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::type_name_from_spanda(ty);

    // Match on ty and handle each case.
    match ty {
        SpandaType::Int => "Int".into(),
        SpandaType::Float => "Float".into(),
        SpandaType::Bool => "Bool".into(),
        SpandaType::String => "String".into(),
        SpandaType::Named { name } => name.clone(),
        SpandaType::Generic { name, type_args } => {
            let args = type_args
                .iter()
                .map(type_name_from_spanda)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}<{args}>")
        }
        other => format!("{:?}", other),
    }
}

fn display_type(ty: &SpandaType) -> String {
    // Display type.
    //
    // Parameters:
    // - `ty` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::display_type(ty);

    // Match on ty and handle each case.
    match ty {
        SpandaType::Void => "Void".into(),
        SpandaType::Int => "Int".into(),
        SpandaType::Float => "Float".into(),
        SpandaType::Bool => "Bool".into(),
        SpandaType::String => "String".into(),
        SpandaType::Number { unit, .. } => format!("Number({})", unit.as_str()),
        SpandaType::Named { name } => name.clone(),
        SpandaType::Generic { name, type_args } => {
            let args: Vec<_> = type_args.iter().map(display_type).collect();
            format!("{name}<{}>", args.join(", "))
        }
        SpandaType::TypeParam { name } => name.clone(),
        SpandaType::TraitObject { trait_name } => format!("dyn {trait_name}"),
        SpandaType::Pose => "Pose".into(),
        SpandaType::Velocity => "Velocity".into(),
        SpandaType::Trajectory => "Path".into(),
        SpandaType::Scan => "Scan".into(),
        SpandaType::EnumVariant { enum_name, variant } => format!("{enum_name}.{variant}"),
        SpandaType::Transform => "Transform".into(),
        SpandaType::Char => "Char".into(),
        SpandaType::Bytes => "Bytes".into(),
        SpandaType::Null => "Null".into(),
        SpandaType::Regex => "Regex".into(),
        SpandaType::Match => "Match".into(),
        SpandaType::Capture => "Capture".into(),
        SpandaType::CaptureGroup => "CaptureGroup".into(),
    }
}

impl SpandaTypeExt for SpandaType {
    fn unit(&self) -> UnitKind {
        // Unit.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // UnitKind.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.unit();

        // Dispatch based on the enum variant or current state.
        match self {
            SpandaType::Number { unit, .. } => *unit,
            _ => UnitKind::None,
        }
    }

    fn kind_name(&self) -> &'static str {
        // Kind name.
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
        // let result = instance.kind_name();

        // Dispatch based on the enum variant or current state.
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
            SpandaType::TypeParam { name } => {
                let _ = name;
                "type_param"
            }
            SpandaType::TraitObject { trait_name } => {
                let _ = trait_name;
                "trait_object"
            }
            SpandaType::Regex => "regex",
            SpandaType::Match => "match_type",
            SpandaType::Capture => "capture",
            SpandaType::CaptureGroup => "capture_group",
        }
    }
}

trait HalMemberDeclExt {
    fn name(&self) -> &str;
}

impl HalMemberDeclExt for HalMemberDecl {
    fn name(&self) -> &str {
        // Name.
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
        // let result = instance.name();

        // Dispatch based on the enum variant or current state.
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
            SafetyBlock::SafetyBlock { rules, .. } => rules,
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
            SafetyBlock::SafetyBlock { zones, .. } => zones,
        }
    }
}

pub struct FnSig {
    named_params: HashMap<String, SpandaType>,
    returns: SpandaType,
}

impl FnSig {
    pub fn named_params(&self) -> &HashMap<String, SpandaType> {
        &self.named_params
    }

    pub fn returns(&self) -> &SpandaType {
        &self.returns
    }
}

/// Format a [`SpandaType`] for documentation and diagnostics.
pub fn format_type_name(ty: &SpandaType) -> String {
    display_type(ty)
}

fn resolve_message_type(registry: &MessageRegistry, name: &str) -> Option<SpandaType> {
    // Resolve message type.
    //
    // Parameters:
    // - `registry` — input value
    // - `name` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::resolve_message_type(registry, name);

    // Produce registry as the result.
    registry
        .resolve_type(name)
        .or_else(|| message_type_for(name))
}

fn message_type_for(name: &str) -> Option<SpandaType> {
    // Message type for.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::message_type_for(name);

    // Match on name and handle each case.
    match name {
        "Velocity" => Some(SpandaType::Velocity),
        "Pose" => Some(SpandaType::Pose),
        "Scan" => Some(SpandaType::Scan),
        "String" => Some(SpandaType::String),
        _ => None,
    }
}

fn service_type_for(name: &str) -> Option<SpandaType> {
    // Service type for.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::service_type_for(name);

    // Match on name and handle each case.
    match name {
        "ResetCostmap" | "ClearCostmap" | "SetPose" => {
            Some(SpandaType::Named { name: name.into() })
        }
        _ => None,
    }
}

fn action_type_for(name: &str) -> Option<SpandaType> {
    // Action type for.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::action_type_for(name);

    // Match on name and handle each case.
    match name {
        "NavigateTo" | "FollowPath" | "PickObject" => Some(SpandaType::Named { name: name.into() }),
        _ => None,
    }
}

fn sensor_type_for(name: &str, host: &dyn TypeCheckHost) -> Option<SpandaType> {
    // Sensor type for.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::sensor_type_for(name);

    // Compute base for the following logic.
    let base = match name {
        "Lidar" | "IMU" | "GPS" | "GNSS" | "Camera" | "AltitudeSensor" | "ForceTorque" => {
            Some(SpandaType::Named { name: name.into() })
        }
        _ => None,
    };

    // Proceed only when is some is available.
    if base.is_some() {
        return base;
    }

    // Take this path when all library sensor types().contains key(name).
    if host.library_sensor_type_known(name) {
        Some(SpandaType::Named { name: name.into() })
    } else {
        None
    }
}

fn actuator_type_for(name: &str) -> Option<SpandaType> {
    // Actuator type for.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::actuator_type_for(name);

    // Match on name and handle each case.
    match name {
        "DifferentialDrive" | "RoboticArm" | "DroneRotors" | "Gripper" => {
            Some(SpandaType::Named { name: name.into() })
        }
        _ => None,
    }
}

fn ai_model_type_for(name: &str) -> Option<SpandaType> {
    // Ai model type for.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::ai_model_type_for(name);

    // Match on name and handle each case.
    match name {
        "LLM" | "VisionModel" | "EmbeddingModel" => Some(SpandaType::Named { name: name.into() }),
        _ => None,
    }
}

fn pose_property(name: &str) -> Option<SpandaType> {
    // Pose property.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::pose_property(name);

    // Match on name and handle each case.
    match name {
        "x" | "y" | "z" => Some(SpandaType::Number { unit: UnitKind::M }),
        "theta" => Some(SpandaType::Number {
            unit: UnitKind::Rad,
        }),
        _ => None,
    }
}

fn velocity_property(name: &str) -> Option<SpandaType> {
    // Velocity property.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::velocity_property(name);

    // Match on name and handle each case.
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
    // Object property.
    //
    // Parameters:
    // - `type_name` — input value
    // - `property` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::object_property(type_name, property);

    // Match on value and handle each case.
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
        ("GpsFix", "lat" | "lon" | "altitude" | "heading" | "speed" | "fix_quality") => {
            Some(SpandaType::Number {
                unit: UnitKind::None,
            })
        }
        ("GnssFix", "lat" | "lon" | "altitude" | "fix_quality" | "satellites") => {
            Some(SpandaType::Number {
                unit: UnitKind::None,
            })
        }
        ("GeoPoint", "lat" | "lon") => Some(SpandaType::Number {
            unit: UnitKind::None,
        }),
        ("SimIdentity", "iccid" | "carrier") => Some(SpandaType::String),
        ("SimIdentity", "esim" | "attested") => Some(SpandaType::Bool),
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
    // Builtin functions.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<&'static str, FnSig>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::builtin_functions();

    // Produce from as the result.
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
        (
            "channel",
            FnSig {
                named_params: HashMap::new(),
                returns: SpandaType::Named {
                    name: "Channel".into(),
                },
            },
        ),
        (
            "send",
            FnSig {
                named_params: HashMap::new(),
                returns: SpandaType::Void,
            },
        ),
        (
            "recv",
            FnSig {
                named_params: HashMap::new(),
                returns: SpandaType::Void,
            },
        ),
        (
            "send_agent",
            FnSig {
                named_params: HashMap::from([
                    ("to".into(), SpandaType::String),
                    ("value".into(), SpandaType::Void),
                ]),
                returns: SpandaType::Void,
            },
        ),
        (
            "recv_agent",
            FnSig {
                named_params: HashMap::new(),
                returns: SpandaType::Void,
            },
        ),
        (
            "peer_send",
            FnSig {
                named_params: HashMap::from([
                    ("peer".into(), SpandaType::String),
                    ("topic".into(), SpandaType::String),
                    ("value".into(), SpandaType::Void),
                ]),
                returns: SpandaType::Void,
            },
        ),
        (
            "serialize",
            FnSig {
                named_params: HashMap::from([("format".into(), SpandaType::String)]),
                returns: SpandaType::String,
            },
        ),
        (
            "deserialize",
            FnSig {
                named_params: HashMap::from([("format".into(), SpandaType::String)]),
                returns: SpandaType::Void,
            },
        ),
        (
            "assert",
            FnSig {
                named_params: HashMap::new(),
                returns: SpandaType::Void,
            },
        ),
    ])
}

fn robot_methods() -> HashMap<&'static str, MethodSig> {
    // Robot methods.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<&'static str, MethodSig>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::robot_methods();

    // Produce from as the result.
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
        (
            "in_geofence",
            MethodSig {
                params: vec![SpandaType::String],
                named_params: HashMap::new(),
                returns: SpandaType::Bool,
            },
        ),
        (
            "connectivity_link",
            MethodSig {
                params: vec![],
                named_params: HashMap::new(),
                returns: SpandaType::String,
            },
        ),
        (
            "sim_identity",
            MethodSig {
                params: vec![],
                named_params: HashMap::new(),
                returns: SpandaType::Named {
                    name: "SimIdentity".into(),
                },
            },
        ),
        (
            "identity",
            MethodSig {
                params: vec![],
                named_params: HashMap::new(),
                returns: SpandaType::Named {
                    name: "RobotIdentity".into(),
                },
            },
        ),
    ])
}

fn builtin_methods(
    type_name: &str,
    host: &dyn TypeCheckHost,
) -> Option<HashMap<&'static str, MethodSig>> {
    // Builtin methods.
    //
    // Parameters:
    // - `type_name` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::builtin_methods(type_name);

    // Compute m for the following logic.
    let m = |params: Vec<SpandaType>, named: HashMap<&str, SpandaType>, returns: SpandaType| {
        MethodSig {
            params,
            named_params: named.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
            returns,
        }
    };

    // Match on type name and handle each case.
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
        "AuditLog" => Some(HashMap::from([
            (
                "record",
                m(
                    vec![SpandaType::String],
                    HashMap::new(),
                    SpandaType::Named {
                        name: "RecordId".into(),
                    },
                ),
            ),
            ("export", m(vec![], HashMap::new(), SpandaType::String)),
            ("count", m(vec![], HashMap::new(), SpandaType::Int)),
            (
                "root_hash",
                m(
                    vec![],
                    HashMap::new(),
                    SpandaType::Named {
                        name: "Hash".into(),
                    },
                ),
            ),
            (
                "create_provenance",
                m(
                    vec![
                        SpandaType::String,
                        SpandaType::Named {
                            name: "RecordId".into(),
                        },
                    ],
                    HashMap::new(),
                    SpandaType::Named {
                        name: "ProvenanceRecord".into(),
                    },
                ),
            ),
        ])),
        "WorldModel" => Some(HashMap::from([
            (
                "update",
                m(
                    vec![SpandaType::Named {
                        name: "FusedObservation".into(),
                    }],
                    HashMap::new(),
                    SpandaType::Number {
                        unit: UnitKind::None,
                    },
                ),
            ),
            (
                "belief",
                m(
                    vec![],
                    HashMap::new(),
                    SpandaType::Number {
                        unit: UnitKind::None,
                    },
                ),
            ),
            ("export", m(vec![], HashMap::new(), SpandaType::String)),
        ])),
        "MockLedger" => Some(HashMap::from([
            (
                "anchor",
                m(
                    vec![SpandaType::Named {
                        name: "Hash".into(),
                    }],
                    HashMap::new(),
                    SpandaType::Named {
                        name: "TransactionId".into(),
                    },
                ),
            ),
            (
                "verify",
                m(
                    vec![SpandaType::Named {
                        name: "Hash".into(),
                    }],
                    HashMap::new(),
                    SpandaType::Bool,
                ),
            ),
        ])),
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
        "FusedObservation" => Some(HashMap::from([
            ("pose", m(vec![], HashMap::new(), SpandaType::Pose)),
            ("count", m(vec![], HashMap::new(), SpandaType::Int)),
            ("confidence", m(vec![], HashMap::new(), SpandaType::Float)),
            (
                "state_estimate",
                m(
                    vec![],
                    HashMap::new(),
                    SpandaType::Named {
                        name: "StateEstimate".into(),
                    },
                ),
            ),
        ])),
        "FleetCoordinator" => Some(HashMap::from([
            (
                "members",
                m(vec![SpandaType::String], HashMap::new(), SpandaType::Int),
            ),
            ("names", m(vec![], HashMap::new(), SpandaType::Int)),
        ])),
        "Mission" => Some(HashMap::from([
            ("start", m(vec![], HashMap::new(), SpandaType::String)),
            ("pause", m(vec![], HashMap::new(), SpandaType::String)),
            ("resume", m(vec![], HashMap::new(), SpandaType::String)),
            ("advance", m(vec![], HashMap::new(), SpandaType::String)),
            ("complete", m(vec![], HashMap::new(), SpandaType::String)),
            ("fail", m(vec![], HashMap::new(), SpandaType::String)),
            ("state", m(vec![], HashMap::new(), SpandaType::String)),
            ("step", m(vec![], HashMap::new(), SpandaType::String)),
        ])),
        "Navigation" => Some(HashMap::from([
            (
                "goal",
                m(
                    vec![SpandaType::String],
                    HashMap::new(),
                    SpandaType::Named {
                        name: "NavigationGoal".into(),
                    },
                ),
            ),
            (
                "path",
                m(
                    vec![],
                    HashMap::new(),
                    SpandaType::Named {
                        name: "Path".into(),
                    },
                ),
            ),
            (
                "navigate",
                m(
                    vec![],
                    HashMap::new(),
                    SpandaType::Named {
                        name: "Trajectory".into(),
                    },
                ),
            ),
            (
                "cost_map",
                m(
                    vec![],
                    HashMap::new(),
                    SpandaType::Named {
                        name: "CostMap".into(),
                    },
                ),
            ),
        ])),
        "Slam" => Some(HashMap::from([
            (
                "localize",
                m(
                    vec![],
                    HashMap::new(),
                    SpandaType::Named {
                        name: "LocalizationEstimate".into(),
                    },
                ),
            ),
            (
                "map",
                m(
                    vec![],
                    HashMap::new(),
                    SpandaType::Named {
                        name: "OccupancyGrid".into(),
                    },
                ),
            ),
        ])),
        other if host.library_sensor_type_known(other) => Some(library_sensor_methods(other)),
        "String" => Some(HashMap::from([
            (
                "matches",
                m(vec![SpandaType::Regex], HashMap::new(), SpandaType::Bool),
            ),
            (
                "find",
                m(vec![SpandaType::Regex], HashMap::new(), SpandaType::String),
            ),
            (
                "replace",
                m(
                    vec![SpandaType::Regex, SpandaType::String],
                    HashMap::new(),
                    SpandaType::String,
                ),
            ),
            (
                "split",
                m(
                    vec![SpandaType::Regex],
                    HashMap::new(),
                    SpandaType::Named {
                        name: "StringList".into(),
                    },
                ),
            ),
            (
                "capture",
                m(vec![SpandaType::Regex], HashMap::new(), SpandaType::Capture),
            ),
        ])),
        _ => None,
    }
}

fn infer_read_return(type_name: &str) -> SpandaType {
    // Infer read return.
    //
    // Parameters:
    // - `type_name` — input value
    //
    // Returns:
    // SpandaType.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::infer_read_return(type_name);

    // Check membership before continuing.
    if type_name.contains("Lidar")
        || type_name.contains("Velodyne")
        || type_name.contains("Hokuyo")
        || type_name.contains("Ydlidar")
        || type_name.contains("Ouster")
        || type_name.contains("RealSense")
    {
        return SpandaType::Scan;
    }

    // Check membership before continuing.
    if type_name.contains("BNO") || type_name.contains("LSM9") || type_name.contains("IMU") {
        return SpandaType::Named {
            name: "IMUReading".into(),
        };
    }

    // Check membership before continuing.
    if type_name.contains("BMP") || type_name.contains("VL53") || type_name.contains("UWMF") {
        return SpandaType::Number { unit: UnitKind::M };
    }

    // Check membership before continuing.
    if type_name.contains("BME") {
        return SpandaType::Number { unit: UnitKind::Rh };
    }

    // Check membership before continuing.
    if type_name.contains("BH1750") || type_name.contains("Light") {
        return SpandaType::Number {
            unit: UnitKind::Lux,
        };
    }

    // Check membership before continuing.
    if type_name.contains("VEML") || type_name.contains("UV") || type_name.contains("Si1145") {
        return SpandaType::Number {
            unit: UnitKind::Uvi,
        };
    }

    // Check membership before continuing.
    if type_name.contains("pH") || type_name.ends_with("PH") {
        return SpandaType::Number { unit: UnitKind::Ph };
    }

    // Check membership before continuing.
    if type_name.contains("EC") || type_name.contains("Conduct") {
        return SpandaType::Number {
            unit: UnitKind::MicroSPerCm,
        };
    }

    // Check membership before continuing.
    if type_name.contains("PMS") || type_name.contains("Particulate") {
        return SpandaType::Number {
            unit: UnitKind::UgPerM3,
        };
    }

    // Check membership before continuing.
    if type_name.contains("Turbid") || type_name.contains("NTU") {
        return SpandaType::Number {
            unit: UnitKind::Ntu,
        };
    }

    // Check membership before continuing.
    if type_name.contains("Salinity") {
        return SpandaType::Number {
            unit: UnitKind::Ppt,
        };
    }

    // Check membership before continuing.
    if type_name.contains("Geiger") || type_name.contains("Radiation") {
        return SpandaType::Number {
            unit: UnitKind::MicroSvPerH,
        };
    }

    // Check membership before continuing.
    if type_name.contains("Soil") || type_name.contains("VWC") {
        return SpandaType::Number {
            unit: UnitKind::PercentVwc,
        };
    }
    SpandaType::Void
}

pub fn merge_library_methods(
    methods: &mut HashMap<String, HashMap<String, MethodSig>>,
    host: &dyn TypeCheckHost,
) {
    // Merge library methods.
    //
    // Parameters:
    // - `methods` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::merge_library_methods(methods);

    // Iterate over all library sensor types with destructured elements.
    for (type_name, robo_type) in host.library_sensor_robo_types() {
        methods.entry(type_name).or_insert_with(|| {
            let read_name = match robo_type {
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
    // Library sensor methods.
    //
    // Parameters:
    // - `type_name` — input value
    //
    // Returns:
    // HashMap<&'static str, MethodSig>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::library_sensor_methods(type_name);

    // Produce from as the result.
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

pub fn get_library_for_sensor_type(sensor_type: &str, host: &dyn TypeCheckHost) -> Option<String> {
    //
    // Parameters:
    // - `sensor_type` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::get_library_for_sensor_type(sensor_type);

    // Produce all library sensor types as the result.
    host.library_for_sensor_type(sensor_type)
}

#[allow(non_snake_case)]
pub fn MESSAGE_TYPES() -> HashMap<String, SpandaType> {
    // MESSAGE TYPES.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<String, SpandaType>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::MESSAGE_TYPES();

    // Produce from as the result.
    HashMap::from([
        ("Velocity".into(), SpandaType::Velocity),
        ("Pose".into(), SpandaType::Pose),
        ("Scan".into(), SpandaType::Scan),
        ("String".into(), SpandaType::String),
    ])
}

#[allow(non_snake_case)]
pub fn SERVICE_TYPES() -> HashMap<String, SpandaType> {
    // SERVICE TYPES.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<String, SpandaType>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::SERVICE_TYPES();

    // Produce from as the result.
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
    // ACTION TYPES.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<String, SpandaType>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::ACTION_TYPES();

    // Produce from as the result.
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
pub fn SENSOR_TYPES(host: &dyn TypeCheckHost) -> HashMap<String, SpandaType> {
    // SENSOR TYPES.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<String, SpandaType>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::SENSOR_TYPES();

    // Create mutable map for accumulating results.
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
            "GNSS".into(),
            SpandaType::Named {
                name: "GNSS".into(),
            },
        ),
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

    // Iterate over all library sensor types with destructured elements.
    for (type_name, robo_type) in host.library_sensor_robo_types() {
        map.insert(type_name, robo_type);
    }
    map
}

#[allow(non_snake_case)]
pub fn ACTUATOR_TYPES() -> HashMap<String, SpandaType> {
    // ACTUATOR TYPES.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<String, SpandaType>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::ACTUATOR_TYPES();

    // Produce from as the result.
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
    // AI MODEL TYPES.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<String, SpandaType>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::AI_MODEL_TYPES();

    // Produce from as the result.
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
    // AI VALUE TYPES.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<String, SpandaType>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::AI_VALUE_TYPES();

    // Produce from as the result.
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
    // BUILTIN FUNCTIONS.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<&'static str, FnSig>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::BUILTIN_FUNCTIONS();

    // Produce builtin functions as the result.
    builtin_functions()
}

#[allow(non_snake_case)]
pub fn ROBOT_METHODS() -> HashMap<&'static str, MethodSig> {
    // ROBOT METHODS.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<&'static str, MethodSig>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::ROBOT_METHODS();

    // Produce robot methods as the result.
    robot_methods()
}

#[allow(non_snake_case)]
pub fn BUILTIN_METHODS(host: &dyn TypeCheckHost) -> HashMap<String, HashMap<String, MethodSig>> {
    // BUILTIN METHODS.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<String, HashMap<String, MethodSig>>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::BUILTIN_METHODS();

    // Create mutable map for accumulating results.
    let mut map: HashMap<String, HashMap<String, MethodSig>> = HashMap::new();

    // Iterate over [.
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
        // Emit output when builtin methods provides a methods.
        if let Some(methods) = builtin_methods(ty, host) {
            map.insert(
                ty.to_string(),
                methods
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect(),
            );
        }
    }
    merge_library_methods(&mut map, host);
    map
}

#[allow(non_snake_case)]
pub fn SCAN_PROPERTIES() -> HashMap<String, SpandaType> {
    // SCAN PROPERTIES.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<String, SpandaType>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::SCAN_PROPERTIES();

    // Produce from as the result.
    HashMap::from([(
        "nearest_distance".into(),
        SpandaType::Number { unit: UnitKind::M },
    )])
}

#[allow(non_snake_case)]
pub fn OBJECT_PROPERTIES() -> HashMap<String, HashMap<String, SpandaType>> {
    // OBJECT PROPERTIES.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<String, HashMap<String, SpandaType>>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::OBJECT_PROPERTIES();

    // Produce from as the result.
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
    // POSE PROPERTIES.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<String, SpandaType>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::POSE_PROPERTIES();

    // Produce from as the result.
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
    // VELOCITY PROPERTIES.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<String, SpandaType>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::types::VELOCITY_PROPERTIES();

    // Produce from as the result.
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
