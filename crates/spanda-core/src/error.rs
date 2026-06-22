//! error support for Spanda.
//!
pub use spanda_error::{Diagnostic, SpandaError};

use crate::ast::Program;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileResult {
    pub program: Program,
    pub source: String,
}

pub use spanda_runtime::robot_state::{PoseState, RobotState, VelocityState};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunOptions {
    #[serde(default)]
    pub entry_behavior: Option<String>,
    #[serde(default = "default_max_loop_iterations")]
    pub max_loop_iterations: usize,
    #[serde(default)]
    pub simulation_steps: Option<usize>,
    #[serde(default)]
    pub obstacles: Vec<ObstacleConfig>,
    #[serde(default)]
    pub initial_pose: Option<PoseState>,
    #[serde(default = "default_lidar_range")]
    pub lidar_range: f64,
    #[serde(skip)]
    pub module_registry: Option<crate::modules::ModuleRegistry>,

    /// Emit scheduler multiplexing and tick diagnostics to runtime logs.
    #[serde(default)]
    pub trace_scheduler: bool,

    /// Emit per-task tick and deadline diagnostics to runtime logs.
    #[serde(default)]
    pub trace_tasks: bool,

    /// Log digital twin replay frame summaries (simulation).
    #[serde(default)]
    pub replay_trace: bool,

    /// Emit trigger dispatch diagnostics to runtime logs.
    #[serde(default)]
    pub trace_triggers: bool,

    /// Emit event trigger diagnostics to runtime logs.
    #[serde(default)]
    pub trace_events: bool,

    /// Enable all realtime trace channels (scheduler, tasks, triggers, events).
    #[serde(default)]
    pub trace_realtime: bool,

    /// Record a deterministic mission trace during run/sim.
    #[serde(default)]
    pub record_trace: bool,

    /// Source label or `.sd` path stored in mission traces.
    #[serde(default)]
    pub trace_source: Option<String>,

    /// Optional mission trace output path (defaults to `<trace_source>.trace`).
    #[serde(default)]
    pub trace_output: Option<String>,

    /// Emit metrics-only JSON (implies --json on CLI).
    #[serde(default)]
    pub metrics_json: bool,

    /// Replay deterministic trace playback mode.
    #[serde(default)]
    pub replay_deterministic: bool,

    /// Replay start offset in milliseconds.
    #[serde(default)]
    pub replay_from_ms: Option<f64>,

    /// Scheduler clock: sim-time (default) or wall-clock RTOS ticks.
    #[serde(default)]
    pub scheduler_clock: crate::scheduler::SchedulerClock,

    /// Sleep between playback frames using recorded timestamps.
    #[serde(default)]
    pub playback_wall_clock: bool,

    /// Write twin replay buffer JSON to this path after run/sim.
    #[serde(default)]
    pub twin_export_path: Option<String>,

    /// Enable strict secure communication enforcement at runtime.
    #[serde(default)]
    pub secure_mode: bool,

    /// Inject security fault scenarios during simulation.
    #[serde(default)]
    pub inject_security_faults: bool,

    /// Enforce certification metadata before run/sim (also via SPANDA_ENFORCE_CERTIFY=1).
    #[serde(default)]
    pub enforce_certify: bool,

    /// Official package names from project `spanda.toml` / `spanda.lock` (runtime provider wiring).
    #[serde(default)]
    pub official_packages: Vec<String>,
}

fn default_max_loop_iterations() -> usize {
    // Default max loop iterations.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::error::default_max_loop_iterations();

    // Produce 10 as the result.
    10
}

fn default_lidar_range() -> f64 {
    // Default lidar range.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::error::default_lidar_range();

    // Produce 0 as the result.
    10.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObstacleConfig {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    pub state: RobotState,
    pub events: Vec<String>,
    pub logs: Vec<String>,
    #[serde(default)]
    pub metrics: crate::telemetry::RuntimeTelemetry,
    #[serde(default)]
    pub mission_trace: Option<crate::replay::MissionTrace>,
    #[serde(default)]
    pub twin_replay: Option<serde_json::Value>,
}
