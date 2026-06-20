//! error support for Spanda.
//!
use crate::ast::Program;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub message: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Error)]
pub enum SpandaError {
    #[error("{message} (line {line}, col {column})")]
    Lexer {
        message: String,
        line: u32,
        column: u32,
    },
    #[error("{message} (line {line}, col {column})")]
    Parse {
        message: String,
        line: u32,
        column: u32,
    },
    #[error("Type check failed")]
    TypeCheck { diagnostics: Vec<Diagnostic> },
    #[error("{message} (line {line})")]
    Runtime { message: String, line: u32 },
    #[error("Debug pause at line {line}: {reason}")]
    DebugPause { line: u32, reason: String },
}

impl SpandaError {
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        // Diagnostics.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Vec<Diagnostic>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.diagnostics();

        // Dispatch based on the enum variant or current state.
        match self {
            SpandaError::Lexer {
                message,
                line,
                column,
            } => vec![Diagnostic {
                message: message.clone(),
                line: *line,
                column: *column,
            }],
            SpandaError::Parse {
                message,
                line,
                column,
            } => vec![Diagnostic {
                message: message.clone(),
                line: *line,
                column: *column,
            }],
            SpandaError::TypeCheck { diagnostics } => diagnostics.clone(),
            SpandaError::Runtime { message, line } => vec![Diagnostic {
                message: message.clone(),
                line: *line,
                column: 1,
            }],
            SpandaError::DebugPause { line, reason } => vec![Diagnostic {
                message: format!("Debug pause: {reason}"),
                line: *line,
                column: 1,
            }],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileResult {
    pub program: Program,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotState {
    pub pose: PoseState,
    pub velocity: VelocityState,
    pub emergency_stop: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoseState {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityState {
    pub linear: f64,
    pub angular: f64,
}

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
}
