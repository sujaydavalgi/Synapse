//! src crate public API and re-exports.
//!
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
pub mod transport_security;
pub mod transport_websocket;
pub mod transport_wire;
pub mod triggers;
pub mod twin;
mod type_check_host;
mod runtime_host;
pub mod type_system;
pub mod types;
pub mod units;

pub use spanda_connectivity::adapter_bridge::{invoke_nav2_bridge, invoke_slam_bridge};
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
pub use ffi::{new_with_core_bridges, FfiRegistry};
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

use runtime::RobotBackend;
use simulator::{create_default_simulator, SimulatorConfig};

pub fn compile(source: &str) -> Result<CompileResult, SpandaError> {
    spanda_driver::compile(source)
}

pub fn check(source: &str) -> Result<(), SpandaError> {
    spanda_driver::check(source)
}

pub fn check_with_registry(source: &str, registry: &ModuleRegistry) -> Result<(), SpandaError> {
    spanda_driver::check_with_registry(source, registry)
}

pub fn compile_with_registry(
    source: &str,
    registry: &ModuleRegistry,
) -> Result<CompileResult, SpandaError> {
    spanda_driver::compile_with_registry(source, registry)
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
    Ok(spanda_sir::lower_program(&program))
}

pub fn run(source: &str, options: RunOptions) -> Result<RunResult, SpandaError> {
    spanda_driver::run(source, options)
}

pub use spanda_driver::TestRunResult;

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

    spanda_driver::run_tests_with_registry(source, registry)
}

pub fn run_program(program: &Program, options: RunOptions) -> Result<RunResult, SpandaError> {
    spanda_driver::run_program(program, options)
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
