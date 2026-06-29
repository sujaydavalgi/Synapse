//! High-level run helpers for parsed Spanda programs.
//!
use crate::options::{RunOptions, RunResult, TestRunResult};
use crate::platform_events::emit_mission_completed;
use crate::runtime::{Interpreter, InterpreterOptions, RobotBackend};
use crate::simulator::{create_default_simulator, Obstacle, SimulatorConfig};
use spanda_ast::nodes::Program;
use spanda_error::SpandaError;
use spanda_providers::bootstrap_providers_for_packages;
use spanda_runtime::robot_state::PoseState;
use spanda_runtime::scheduler::SchedulerClock;
use spanda_typecheck::ModuleRegistry;
use std::cell::RefCell;
use std::rc::Rc;

/// Execute a type-checked program with simulation and optional provider wiring.
pub fn run_program(program: &Program, options: RunOptions) -> Result<RunResult, SpandaError> {
    // Description:
    //     Run program.
    //
    // Inputs:
    //     progra: &Program
    //         Caller-supplied progra.
    //     options: RunOptions
    //         Caller-supplied options.
    //
    // Outputs:
    //     result: Result<RunResult, SpandaError>
    //         Return value from `run_program`.
    //
    // Example:

    //     let result = spanda_interpreter::run::run_program(progra, options);

    if let Some(hooks) = &options.runtime_hooks {
        hooks
            .enforce_certification(program, options.enforce_certify)
            .map_err(|message| SpandaError::Runtime { message, line: 0 })?;
    }

    spanda_telemetry_store::configure_session_persist(options.persist_telemetry);
    let trace_source = options.trace_source.clone();
    if options.persist_telemetry {
        let _ = spanda_telemetry_store::begin_run_session(trace_source.as_deref());
    }

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
    let record_trace = options.record_trace;
    let scheduler_clock = if options.replay_deterministic {
        SchedulerClock::Sim
    } else {
        options.scheduler_clock
    };
    let package_names: Vec<String> = if let Some(ref cfg) = options.system_config {
        spanda_config::provider_packages_for_runtime(cfg)
    } else {
        options.official_packages.clone()
    };
    let provider_registry = if package_names.is_empty() {
        None
    } else {
        Some(bootstrap_providers_for_packages(
            &package_names.iter().map(String::as_str).collect::<Vec<_>>(),
        ))
    };
    let ffi_registry = options.ffi_registry.clone().unwrap_or_default();
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
            trace_providers: options.trace_providers || trace_realtime,
            replay_trace: options.replay_trace,
            record_trace,
            trace_source,
            scheduler_clock,
            replay_deterministic: options.replay_deterministic,
            secure_mode: options.secure_mode,
            inject_security_faults: options.inject_security_faults,
            trigger_kill_switch: options.trigger_kill_switch.clone(),
            kill_switch_signature: options.kill_switch_signature.clone(),
            inject_health_faults: options.inject_health_faults,
            inbound_comm_messages: options.inbound_comm_messages.clone(),
            official_packages: package_names,
            provider_registry,
            ffi_registry,
            enforce_policy: options.enforce_policy.clone(),
            ..Default::default()
        },
    );
    let trace_source = options.trace_source.clone();
    let run_outcome = interp.run(program, options.entry_behavior.as_deref());
    if run_outcome.is_err() {
        emit_mission_completed(
            interp.audit_runtime_mut(),
            program,
            trace_source.as_deref(),
            false,
        );
    }
    let state = run_outcome?;
    let sim_time_ms = interp.sim_time_ms();
    let events = interp.robot_backend().event_log();
    let metrics = interp.take_telemetry();
    let mission_trace = interp.take_mission_trace();
    let mut mission_trace_path = None;
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
            mission_trace_path = Some(path);
        }
    }
    if options.persist_telemetry {
        let _ = spanda_telemetry_store::end_run_session(
            mission_trace_path.as_deref(),
            Some(&metrics),
            sim_time_ms,
        );
    }
    let twin_replay = interp.twin_replay_export();
    if let Some(path) = &options.twin_export_path {
        if let Some(export) = &twin_replay {
            let body = serde_json::to_string_pretty(export).map_err(|e| SpandaError::Runtime {
                message: format!("twin export JSON: {e}"),
                line: 0,
            })?;
            std::fs::write(path, body).map_err(|e| SpandaError::Runtime {
                message: format!("twin export write {path}: {e}"),
                line: 0,
            })?;
        }
    }
    let run_logs = logs.borrow().clone();
    Ok(RunResult {
        state,
        events,
        logs: run_logs,
        metrics,
        mission_trace,
        twin_replay,
    })
}

/// Run module tests embedded in a parsed program.
pub fn run_tests_with_registry(
    program: &Program,
    registry: &ModuleRegistry,
) -> Result<TestRunResult, SpandaError> {
    // Description:
    //     Run tests with registry.
    //
    // Inputs:
    //     progra: &Program
    //         Caller-supplied progra.
    //     registry: &ModuleRegistry
    //         Caller-supplied registry.
    //
    // Outputs:
    //     result: Result<TestRunResult, SpandaError>
    //         Return value from `run_tests_with_registry`.
    //
    // Example:

    //     let result = spanda_interpreter::run::run_tests_with_registry(progra, registry);

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
    let Program::Program { tests, .. } = program;
    let total = tests.len();
    match interp.run_tests(program) {
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
