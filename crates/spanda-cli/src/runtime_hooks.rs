//! CLI-injected runtime hooks wiring platform services into the interpreter.
//!
use spanda_assurance::enforce_runtime_certification;
use spanda_ast::nodes::Program;
use spanda_runtime::hooks::RuntimeHooks;

/// Full runtime hooks for default `spanda` CLI runs.
#[derive(Debug, Default, Clone, Copy)]
pub struct CliRuntimeHooks;

impl RuntimeHooks for CliRuntimeHooks {
    fn enforce_certification(&self, program: &Program, enforce: bool) -> Result<(), String> {
        enforce_runtime_certification(program, enforce).map_err(|error| error.to_string())
    }
}

/// Shared CLI hooks for injection into [`RunOptions`](spanda_interpreter::RunOptions).
pub fn default_runtime_hooks() -> std::sync::Arc<dyn RuntimeHooks> {
    std::sync::Arc::new(CliRuntimeHooks)
}
