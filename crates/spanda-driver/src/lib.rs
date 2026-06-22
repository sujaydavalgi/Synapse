//! Spanda compile driver — wires lexer, parser, and type checker.
//!
mod compile;
mod run;

pub use compile::{
    check, check_with_registry, compile, compile_with_registry, CompileResult,
};
pub use run::{run, run_program, run_tests_with_registry};
pub use spanda_interpreter::{RunOptions, RunResult, TestRunResult};
