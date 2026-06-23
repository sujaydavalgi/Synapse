//! High-level run helpers from source through compile, certify, and interpreter.
//!
use spanda_ast::nodes::Program;
#[cfg(feature = "bridge")]
use spanda_bridge::default_ffi_registry;
#[cfg(feature = "certify")]
use spanda_certify::{certification_runtime_enabled_from_env, enforce_certification_runtime};
use spanda_error::SpandaError;
use spanda_interpreter::{run_program as interpreter_run_program, RunOptions, RunResult};
use spanda_typecheck::ModuleRegistry;

use crate::compile::{compile, compile_with_registry};

/// Compile, certify (when enabled), and execute Spanda source.
pub fn run(source: &str, options: RunOptions) -> Result<RunResult, SpandaError> {
    // Compile source, apply runtime gates, and execute via the interpreter.
    //
    // Parameters:
    // - `source` — full `.sd` source text
    // - `options` — run options including certify and FFI registry overrides
    //
    // Returns:
    // Interpreter run result, or a compile/certify/runtime error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = run(source, RunOptions::default())?;

    let program = if let Some(registry) = &options.module_registry {
        compile_with_registry(source, registry)?.program
    } else {
        compile(source)?.program
    };
    run_program(&program, options)
}

/// Apply certify and FFI defaults, then execute a type-checked program.
pub fn run_program(program: &Program, options: RunOptions) -> Result<RunResult, SpandaError> {
    let mut options = options;
    // Wire default FFI bridges and certification gate before interpreter execution.
    //
    // Parameters:
    // - `program` — type-checked AST
    // - `options` — run options
    //
    // Returns:
    // Interpreter run result, or a certify/runtime error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = run_program(&program, RunOptions::default())?;

    #[cfg(feature = "bridge")]
    if options.ffi_registry.is_none() {
        options.ffi_registry = Some(default_ffi_registry());
    }
    #[cfg(feature = "certify")]
    if options.enforce_certify || certification_runtime_enabled_from_env() {
        enforce_certification_runtime(program, true)?;
    }
    interpreter_run_program(program, options)
}

/// Type-check and run embedded module tests from source.
pub fn run_tests_with_registry(
    source: &str,
    registry: &ModuleRegistry,
) -> Result<spanda_interpreter::TestRunResult, SpandaError> {
    // Compile with project modules and delegate test execution to the interpreter.
    //
    // Parameters:
    // - `source` — full `.sd` source text
    // - `registry` — loaded project modules
    //
    // Returns:
    // Test pass/fail summary from the interpreter.
    //
    // Options:
    // None.
    //
    // Example:
    // let summary = run_tests_with_registry(source, &registry)?;

    let program = compile_with_registry(source, registry)?.program;
    spanda_interpreter::run_tests_with_registry(&program, registry)
}
