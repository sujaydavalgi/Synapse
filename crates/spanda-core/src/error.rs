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
}

impl SpandaError {
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
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
}

fn default_max_loop_iterations() -> usize {
    10
}

fn default_lidar_range() -> f64 {
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
}
