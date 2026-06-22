//! Program type checker re-exported from `spanda-typecheck` with core host wiring.
//!
pub use spanda_typecheck::{
    format_type_name, units_compatible, MethodSig, TypeChecker, ACTUATOR_TYPES, AI_MODEL_TYPES,
    AI_VALUE_TYPES, BUILTIN_FUNCTIONS, MESSAGE_TYPES, OBJECT_PROPERTIES, ROBOT_METHODS,
    SCAN_PROPERTIES, SERVICE_TYPES, ACTION_TYPES,
};

use crate::error::SpandaError;
use crate::type_check_host::core_type_check_host;
use spanda_ast::nodes::Program;
use spanda_typecheck::{self, ModuleRegistry, TypeCheckError};

pub use spanda_typecheck::Diagnostic;

pub fn type_check(program: &Program) -> Result<(), SpandaError> {
    spanda_typecheck::type_check(program, core_type_check_host()).map_err(type_check_error)
}

pub fn check(program: &Program) -> Result<(), SpandaError> {
    spanda_typecheck::check(program, core_type_check_host()).map_err(type_check_error)
}

pub fn check_with_registry(
    program: &Program,
    registry: &ModuleRegistry,
) -> Result<(), SpandaError> {
    spanda_typecheck::check_with_registry(program, registry, core_type_check_host())
        .map_err(type_check_error)
}

fn type_check_error(err: TypeCheckError) -> SpandaError {
    SpandaError::TypeCheck {
        diagnostics: err.diagnostics,
    }
}

#[allow(non_snake_case)]
pub fn BUILTIN_METHODS() -> std::collections::HashMap<String, std::collections::HashMap<String, MethodSig>> {
    spanda_typecheck::BUILTIN_METHODS(core_type_check_host())
}

#[allow(non_snake_case)]
pub fn SENSOR_TYPES() -> std::collections::HashMap<String, spanda_ast::nodes::SpandaType> {
    spanda_typecheck::SENSOR_TYPES(core_type_check_host())
}

pub fn get_library_for_sensor_type(sensor_type: &str) -> Option<String> {
    spanda_typecheck::get_library_for_sensor_type(sensor_type, core_type_check_host())
}

pub fn merge_library_methods(
    methods: &mut std::collections::HashMap<String, std::collections::HashMap<String, MethodSig>>,
) {
    spanda_typecheck::merge_library_methods(methods, core_type_check_host());
}
