//! src crate public API and re-exports.
//!
pub mod adapter_bridge;
pub mod adapter_verify;
pub mod ai;
pub mod ast;
pub mod audit;
pub mod bridge;
pub mod certify_prover;
pub mod certify_runtime;
pub mod certify_verify;
pub mod codegen;
pub mod comm;
pub mod concurrency;
pub mod connectivity_positioning;
pub mod debug;
pub mod debug_session;
pub mod deploy_agent;
pub mod deploy_bundle;
pub mod deploy_http;
pub mod deploy_remote;
pub mod deploy_service;
pub mod docs;
mod error;
pub mod events;
pub mod ffi;
pub mod ffi_registry;
pub mod fleet_agent;
pub mod fleet_mesh;
pub mod fleet_orchestrator;
pub mod fleet_remote;
pub mod format;
pub mod foundations;
pub mod hal;
pub mod hardware;
pub mod hardware_monitor;
pub mod language_reference;
pub mod lexer;
pub mod lib_registry;
pub mod lint;
pub mod modules;
pub mod nav2_adapter;
pub mod parser;
pub mod pretty;
pub mod providers;
pub mod regex_lang;
pub mod reliability;
pub mod reliability_runtime;
pub mod replay;
pub mod robotics_platform;
pub mod runtime;
pub mod safety;
pub mod scheduler;
pub mod security;
pub mod security_validate;
pub mod serialize;
pub mod simulator;
pub mod sir;
pub mod slam_adapter;
pub mod soc;
pub mod state_machine;
pub mod stdlib;
pub mod swarm_coordinator;
pub mod telemetry;
pub mod transport;
pub mod transport_dds;
pub mod transport_live;
pub mod transport_mqtt;
pub mod transport_rclrs;
mod transport_rclrs_daemon;
mod transport_rclrs_native;
pub mod transport_security;
mod transport_tls;
pub mod transport_websocket;
pub mod transport_wire;
pub mod triggers;
pub mod twin;
mod type_check_host;
mod runtime_host;
pub mod type_system;
pub mod types;
pub mod units;

pub use adapter_bridge::{invoke_nav2_bridge, invoke_slam_bridge};
pub use ast::*;
pub use certify_prover::{
    build_certification_proof, build_certification_proof_summary, CertificationEntry,
    CertificationProofReport, CertificationProofSummary, DeployTargetEntry,
};
pub use certify_runtime::{certification_runtime_enabled_from_env, enforce_certification_runtime};
pub use certify_verify::verify_certification_proof;
pub use codegen::{generate as codegen, wasm_deploy_manifest, CodegenTarget};
pub use debug::{DebugCommand, DebugController, DebugOptions, DebugPause, DebugSession};
pub use debug_session::{DebugMachine, DebugStackFrame, DebugStepKind};
pub use deploy_agent::{
    agent_entry_for_port, default_agent_state_path, handle_agent_request, load_agent_state,
    run_deploy_agent_server, save_agent_state, spawn_test_agent, spawn_test_agent_with_options,
    DeployAgentServerOptions,
    AgentState,
};
pub use deploy_bundle::{
    build_deploy_bundle, bundle_canonical_json, sign_deploy_bundle, verify_deploy_bundle,
    verify_rollout_artifact, DeployArtifactBundle,
};
pub use deploy_http::{parse_http_url, DeployAgentTls};
pub use deploy_remote::{
    agent_health, agent_rollout, agent_status, default_agents_path, execute_remote_rollback,
    execute_remote_rollout, load_agent_registry, lookup_agent, missing_remote_targets,
    register_agent, save_agent_registry, AgentStatusResponse, DeployAgentEntry,
    DeployAgentRegistry,
};
pub use deploy_service::{
    apply_rollout, build_deploy_plan, default_state_path, deploy_target_key, hash_program_artifact,
    load_deploy_state, plan_rollout, rollback_targets, save_deploy_state,
    validate_rollout_certification, DeployAssignment, DeployPlan, DeployState, RolloutOptions,
    RolloutResult, RolloutStep, RolloutStepStatus, RolloutStrategy,
};
pub use docs::generate_markdown;
pub use error::*;
pub use ffi::FfiRegistry;
pub use fleet_agent::{
    default_fleet_agent_state_path, fleet_entry_for_port, handle_fleet_agent_request,
    load_fleet_agent_state, run_fleet_agent_server, save_fleet_agent_state, spawn_test_fleet_agent,
    FleetAgentState,
};
pub use fleet_mesh::{
    default_fleet_mesh_state_path, mesh_registry_path, relay_deliveries_via_mesh,
    run_fleet_mesh_coordinator, spawn_test_fleet_mesh, FleetMeshState, MeshRelayResponse,
};
pub use fleet_orchestrator::{
    fleet_registry_from_program, orchestrate_fleets, orchestrate_fleets_mesh,
    orchestrate_fleets_remote, FleetMemberState, FleetOrchestrationReport,
    FleetOrchestrationResult, PeerDelivery,
};
pub use fleet_remote::{
    agent_health as fleet_agent_health, default_fleet_agents_path, load_fleet_agent_registry,
    lookup_fleet_agent, register_fleet_agent, relay_peer_deliveries, relay_peer_delivery,
    save_fleet_agent_registry, FleetAgentEntry, FleetAgentRegistry, PeerRelayResponse,
};
pub use format::{format_ast, format_source};
pub use hardware::{
    list_hardware_profiles, CompatItem, CompatSeverity, CompatibilityMatrix, CompatibilityReport,
    MatrixCell, VerifyOptions,
};
pub use language_reference::{generate_cli_man_pages, generate_language_reference};
pub use lint::{lint, LintIssue, LintReport, LintSeverity};
pub use modules::{load_project_modules, ModuleRegistry};
pub use replay::{
    parse_replay_offset, playback_frames, verify_traces, MissionTrace, PlaybackReport,
    ReplayStateSnapshot, ReplayStateTarget, TraceVerification,
};
pub use robotics_platform::SwarmPolicy;
pub use scheduler::SchedulerClock;
pub use security_validate::{
    security_audit, security_check, SecurityFinding, SecurityReport, SecuritySeverity,
};
pub use sir::{
    lower_program, SirBehavior, SirExtern, SirFunction, SirParam, SirProgram, SirStmt,
    SirVisibility,
};
pub use swarm_coordinator::{
    coordinate_swarms, coordinate_swarms_mesh, default_swarm_state_path, load_swarm_state,
    save_swarm_state, SwarmCoordinationReport, SwarmCoordinationResult, SwarmState,
};
pub use telemetry::{
    ExecutionMetrics, PipelineMetrics, RuntimeTelemetry, SchedulerMetrics, TaskMetrics,
    TopicMetrics, TriggerMetrics, WatchdogMetrics,
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
            strict_certify: false,
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

    if options.enforce_certify || certification_runtime_enabled_from_env() {
        enforce_certification_runtime(program, true)?;
    }

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
    let scheduler_clock = if options.replay_deterministic {
        scheduler::SchedulerClock::Sim
    } else {
        options.scheduler_clock
    };
    let provider_registry = if options.official_packages.is_empty() {
        None
    } else {
        Some(crate::providers::bootstrap_providers_for_packages(
            &options
                .official_packages
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        ))
    };
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
            scheduler_clock,
            replay_deterministic: options.replay_deterministic,
            secure_mode: options.secure_mode,
            inject_security_faults: options.inject_security_faults,
            official_packages: options.official_packages.clone(),
            provider_registry,
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

pub fn replay_mission(
    source: &str,
    trace_path: &str,
    mut options: RunOptions,
) -> Result<(RunResult, TraceVerification), SpandaError> {
    // Re-run a program and verify the recorded mission trace matches a reference trace.
    //
    // Parameters:
    // - `source` — `.sd` program source text
    // - `trace_path` — reference `.trace` file path
    // - `options` — run options; `replay_from_ms` selects comparison offset
    //
    // Returns:
    // Run result plus deterministic trace verification report.
    //
    // Options:
    // None.
    //
    // Example:
    // let (result, report) = replay_mission(source, "mission.trace", RunOptions::default())?;

    // Load the reference trace and record a fresh trace during replay.
    let expected = MissionTrace::load(trace_path)?;
    let from_ms = options.replay_from_ms.unwrap_or(0.0);
    options.record_trace = true;
    options.replay_deterministic = true;
    if options.trace_source.is_none() {
        options.trace_source = Some(expected.source.clone());
    }
    let result = run(source, options)?;
    let actual = result
        .mission_trace
        .as_ref()
        .ok_or_else(|| SpandaError::Runtime {
            message: "Replay run did not produce a mission trace".into(),
            line: 0,
        })?;
    let verification = verify_traces(&expected, actual, from_ms);
    Ok((result, verification))
}

pub fn playback_mission(
    trace_path: &str,
    options: RunOptions,
) -> Result<(PlaybackReport, RobotState), SpandaError> {
    // Play back recorded mission frames without re-executing program logic.
    //
    // Parameters:
    // - `trace_path` — input `.trace` file
    // - `options` — playback offset and wall-clock pacing options
    //
    // Returns:
    // Playback report and final robot state after applying snapshots.
    //
    // Options:
    // None.
    //
    // Example:

    // let (report, state) = playback_mission("mission.trace", RunOptions::default())?;

    let trace = MissionTrace::load(trace_path)?;
    let from_ms = options.replay_from_ms.unwrap_or(0.0);
    let frames: Vec<_> = trace.frames_from(from_ms).to_vec();
    let mut sim = create_default_simulator(SimulatorConfig::default());
    let report = playback_frames(&frames, &mut sim, options.playback_wall_clock);
    Ok((report, sim.get_state()))
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
