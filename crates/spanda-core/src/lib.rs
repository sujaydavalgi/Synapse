//! src crate public API and re-exports.
//!
pub mod ai;
pub mod ast;
pub mod audit;
pub mod bridge;
pub mod codegen;
pub mod comm;
pub mod concurrency;
pub mod debug;
pub mod debug_session;
pub mod docs;
mod error;
pub mod events;
pub mod ffi;
pub mod ffi_registry;
pub mod format;
pub mod foundations;
pub mod hal;
pub mod hardware;
pub mod hardware_monitor;
pub mod lexer;
pub mod lib_registry;
pub mod lint;
pub mod modules;
pub mod parser;
pub mod pretty;
pub mod regex_lang;
pub mod reliability;
pub mod reliability_runtime;
pub mod replay;
pub mod runtime;
pub mod safety;
pub mod security;
pub mod serialize;
pub mod simulator;
pub mod sir;
pub mod soc;
pub mod state_machine;
pub mod stdlib;
pub mod telemetry;
pub mod transport;
pub mod transport_live;
pub mod transport_rclrs;
mod transport_rclrs_daemon;
#[cfg(not(target_arch = "wasm32"))]
mod transport_rclrs_native;
#[cfg(target_arch = "wasm32")]
#[path = "transport_rclrs_native_stub.rs"]
mod transport_rclrs_native;
pub mod triggers;
pub mod twin;
pub mod type_system;
pub mod types;
pub mod units;

pub use ast::*;
pub use codegen::{generate as codegen, wasm_deploy_manifest, CodegenTarget};
pub use debug::{DebugCommand, DebugController, DebugOptions, DebugPause, DebugSession};
pub use debug_session::{DebugMachine, DebugStackFrame, DebugStepKind};
pub use docs::generate_markdown;
pub use error::*;
pub use ffi::FfiRegistry;
pub use format::{format_ast, format_source};
pub use hardware::{
    list_hardware_profiles, CompatItem, CompatSeverity, CompatibilityMatrix, CompatibilityReport,
    MatrixCell, VerifyOptions,
};
pub use lint::{lint, LintIssue, LintReport, LintSeverity};
pub use modules::{load_project_modules, ModuleRegistry};
pub use replay::{parse_replay_offset, MissionTrace};
pub use sir::{
    lower_program, SirBehavior, SirExtern, SirFunction, SirParam, SirProgram, SirStmt,
    SirVisibility,
};
pub use telemetry::{
    ExecutionMetrics, PipelineMetrics, RuntimeTelemetry, SchedulerMetrics, TaskMetrics,
    TriggerMetrics, WatchdogMetrics,
};

use runtime::{Interpreter, InterpreterOptions, RobotBackend};
use serde::{Deserialize, Serialize};
use simulator::{create_default_simulator, Obstacle, SimulatorConfig};
use std::cell::RefCell;
use std::rc::Rc;

pub fn compile(source: &str) -> Result<CompileResult, SpandaError> {
    // Compile.
    //
    // Parameters:
    // - `source` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::compile(source);

    // Tokenize the source before parsing.
    let tokens = lexer::tokenize(source)?;
    let program = parser::parse(tokens)?;
    types::type_check(&program)?;
    Ok(CompileResult {
        program,
        source: source.to_string(),
    })
}

pub fn check(source: &str) -> Result<(), SpandaError> {
    // Check input.
    //
    // Parameters:
    // - `source` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::check(source);

    // Tokenize the source before parsing.
    let tokens = lexer::tokenize(source)?;
    let program = parser::parse(tokens)?;
    types::check(&program)
}

pub fn check_with_registry(source: &str, registry: &ModuleRegistry) -> Result<(), SpandaError> {
    // Check with registry.
    //
    // Parameters:
    // - `source` — input value
    // - `registry` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::check_with_registry(source, registry);

    // Tokenize the source before parsing.
    let tokens = lexer::tokenize(source)?;
    let program = parser::parse(tokens)?;
    types::check_with_registry(&program, registry)
}

pub fn compile_with_registry(
    source: &str,
    registry: &ModuleRegistry,
) -> Result<CompileResult, SpandaError> {
    // Compile with registry.
    //
    // Parameters:
    // - `source` — input value
    // - `registry` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::compile_with_registry(source, registry);

    // Tokenize the source before parsing.
    let tokens = lexer::tokenize(source)?;
    let program = parser::parse(tokens)?;
    types::check_with_registry(&program, registry)?;
    Ok(CompileResult {
        program,
        source: source.to_string(),
    })
}

pub fn verify_compatibility(
    source: &str,
    options: &hardware::VerifyOptions,
) -> Result<hardware::CompatibilityReport, SpandaError> {
    // Verify compatibility.
    //
    // Parameters:
    // - `source` — input value
    // - `options` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::verify_compatibility(source, options);

    // Tokenize the source before parsing.
    let tokens = lexer::tokenize(source)?;
    let program = parser::parse(tokens)?;
    types::check(&program)?;
    Ok(hardware::verify_program_compatibility(&program, options))
}

pub fn verify_compatibility_target(
    source: &str,
    target: Option<&str>,
) -> Result<hardware::CompatibilityReport, SpandaError> {
    // Verify compatibility target.
    //
    // Parameters:
    // - `source` — input value
    // - `target` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::verify_compatibility_target(source, target);

    // Produce verify compatibility as the result.
    verify_compatibility(
        source,
        &hardware::VerifyOptions {
            target: target.map(str::to_string),
            all_targets: false,
            simulate: false,
        },
    )
}

pub fn lower_to_sir(source: &str) -> Result<SirProgram, SpandaError> {
    // Lower to sir.
    //
    // Parameters:
    // - `source` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::lower_to_sir(source);

    // Tokenize the source before parsing.
    let tokens = lexer::tokenize(source)?;
    let program = parser::parse(tokens)?;
    types::check(&program)?;
    Ok(sir::lower_program(&program))
}

pub fn run(source: &str, options: RunOptions) -> Result<RunResult, SpandaError> {
    // Run the operation.
    //
    // Parameters:
    // - `source` — input value
    // - `options` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::run(source, options);

    // Parse and type-check the source program.
    let program = if let Some(registry) = &options.module_registry {
        compile_with_registry(source, registry)?.program
    } else {
        compile(source)?.program
    };
    run_program(&program, options)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunResult {
    pub passed: usize,
    pub failed: usize,
    pub logs: Vec<String>,
}

pub fn run_tests(source: &str) -> Result<TestRunResult, SpandaError> {
    // Run tests.
    //
    // Parameters:
    // - `source` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::run_tests(source);

    // Produce new as the result.
    run_tests_with_registry(source, &ModuleRegistry::new())
}

pub fn run_tests_with_registry(
    source: &str,
    registry: &ModuleRegistry,
) -> Result<TestRunResult, SpandaError> {
    // Run tests with registry.
    //
    // Parameters:
    // - `source` — input value
    // - `registry` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::run_tests_with_registry(source, registry);

    // Tokenize the source before parsing.
    let tokens = lexer::tokenize(source)?;
    let program = parser::parse(tokens)?;
    types::check_with_registry(&program, registry)?;
    let sim = create_default_simulator(SimulatorConfig::default());
    let logs: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let logs_cb = logs.clone();
    let mut interp = Interpreter::new(
        sim,
        InterpreterOptions {
            max_loop_iterations: 10,
            on_log: Some(Rc::new(move |msg| logs_cb.borrow_mut().push(msg))),
            on_motion_blocked: None,
            module_registry: Some(registry.clone()),
            ..Default::default()
        },
    );
    let Program::Program { tests, .. } = &program;
    let total = tests.len();

    // Match on run tests and handle each case.
    match interp.run_tests(&program) {
        Ok(()) => Ok(TestRunResult {
            passed: total,
            failed: 0,
            logs: logs.borrow().clone(),
        }),
        Err(e) => Ok(TestRunResult {
            passed: 0,
            failed: total.max(1),
            logs: {
                let mut l = logs.borrow().clone();
                l.push(e.to_string());
                l
            },
        }),
    }
}

pub fn run_program(program: &Program, options: RunOptions) -> Result<RunResult, SpandaError> {
    // Run program.
    //
    // Parameters:
    // - `program` — input value
    // - `options` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::run_program(program, options);

    // Compute obstacles for the following logic.
    let obstacles: Vec<Obstacle> = options
        .obstacles
        .iter()
        .map(|o| Obstacle {
            x: o.x,
            y: o.y,
            radius: o.radius,
        })
        .collect();
    let initial_pose = options.initial_pose.clone().unwrap_or(PoseState {
        x: 0.0,
        y: 0.0,
        theta: 0.0,
        z: Some(0.0),
    });
    let sim_config = SimulatorConfig {
        obstacles: if obstacles.is_empty() {
            SimulatorConfig::default().obstacles
        } else {
            obstacles
        },
        initial_pose,
        lidar_range: options.lidar_range,
    };
    let sim = create_default_simulator(sim_config);
    let logs: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let logs_cb = logs.clone();
    let trace_realtime = options.trace_realtime;
    let trace_source = options.trace_source.clone();
    let record_trace = options.record_trace;
    let mut interp = Interpreter::new(
        sim,
        InterpreterOptions {
            max_loop_iterations: options.max_loop_iterations,
            on_log: Some(Rc::new(move |msg| logs_cb.borrow_mut().push(msg))),
            on_motion_blocked: None,
            module_registry: options.module_registry.clone(),
            trace_scheduler: options.trace_scheduler || trace_realtime,
            trace_tasks: options.trace_tasks || trace_realtime,
            trace_triggers: options.trace_triggers || trace_realtime,
            trace_events: options.trace_events || trace_realtime,
            replay_trace: options.replay_trace,
            record_trace,
            trace_source,
            ..Default::default()
        },
    );
    let state = interp.run(program, options.entry_behavior.as_deref())?;
    let events = interp.robot_backend().event_log();
    let metrics = interp.take_telemetry();
    let mission_trace = interp.take_mission_trace();
    if record_trace {
        if let Some(trace) = &mission_trace {
            let path = options.trace_output.clone().unwrap_or_else(|| {
                let source = options.trace_source.as_deref().unwrap_or("program.sd");
                if let Some(stripped) = source.strip_suffix(".sd") {
                    format!("{stripped}.trace")
                } else {
                    format!("{source}.trace")
                }
            });
            trace.save(&path)?;
        }
    }
    let run_logs = logs.borrow().clone();
    Ok(RunResult {
        state,
        events,
        logs: run_logs,
        metrics,
        mission_trace,
    })
}

pub fn run_debug(source: &str, options: DebugOptions) -> Result<DebugSession, SpandaError> {
    // Run debug.
    //
    // Parameters:
    // - `source` — input value
    // - `options` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::run_debug(source, options);

    // Compute step for the following logic.
    let step = if options.step {
        DebugStepKind::StepOver
    } else {
        DebugStepKind::Continue
    };
    let mut machine = DebugMachine::start(source, options)?;
    machine.run_until_pause(step)
}
