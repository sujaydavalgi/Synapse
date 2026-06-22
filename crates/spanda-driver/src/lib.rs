//! Spanda compile driver — wires lexer, parser, and type checker.
//!
mod compile;
mod debug_run;
pub mod debug_session;
mod deploy_plan;
pub mod pipeline;
mod replay;
mod run;
pub mod type_check;
mod verify;

pub use compile::{
    check, check_with_registry, compile, compile_with_registry, tokenize, CompileResult,
};
pub use debug_run::run_debug;
pub use debug_session::{DebugMachine, DebugStackFrame, DebugStepKind};
pub use deploy_plan::build_deploy_plan;
pub use pipeline::{lower_to_sir, run_tests};
pub use replay::{playback_mission, replay_mission};
pub use run::{run, run_program, run_tests_with_registry};
pub use spanda_interpreter::{RunOptions, RunResult, TestRunResult};
pub use verify::{
    verify_compatibility, verify_compatibility_target, verify_compatibility_with_registry,
};
