//! Driver helpers for targeted interpreter recovery on deployed programs.
//!
use spanda_ast::nodes::Program;
use spanda_error::SpandaError;
use spanda_interpreter::{
    execute_recovery_on_program as interpreter_execute_recovery_on_program, RecoveryRunOptions,
    RecoveryRunResult,
};

use crate::compile::compile;

/// Compile source and run interpreter-backed recovery for a failure issue.
pub fn execute_recovery_source(
    source: &str,
    issue: &str,
    options: RecoveryRunOptions,
) -> Result<RecoveryRunResult, SpandaError> {
    // Compile source and run interpreter-backed recovery for a failure issue.
    //
    // Parameters:
    // - `source` — deployed `.sd` program text
    // - `issue` — failure trigger such as `fleet.failed`
    // - `options` — robot binding and approval hooks
    //
    // Returns:
    // Recovery outcome with runtime logs and interpreter snapshot fields.
    //
    // Options:
    // None.
    //
    // Example:
    // let outcome = execute_recovery_source(source, "fleet.failed", RecoveryRunOptions::default())?;

    let program = compile(source)?.program;
    execute_recovery_on_program(&program, issue, options)
}

/// Run interpreter-backed recovery on an already parsed program.
pub fn execute_recovery_on_program(
    program: &Program,
    issue: &str,
    options: RecoveryRunOptions,
) -> Result<RecoveryRunResult, SpandaError> {
    // Run interpreter-backed recovery on an already parsed program.
    //
    // Parameters:
    // - `program` — parsed program deployed on a fleet agent
    // - `issue` — failure trigger such as `fleet.failed`
    // - `options` — robot binding and approval hooks
    //
    // Returns:
    // Recovery outcome with runtime logs and interpreter snapshot fields.
    //
    // Options:
    // None.
    //
    // Example:
    // let outcome = execute_recovery_on_program(&program, "fleet.failed", RecoveryRunOptions::default())?;

    interpreter_execute_recovery_on_program(program, issue, options)
}
