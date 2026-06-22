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

pub use spanda_interpreter::{
    ObstacleConfig, RunOptions, RunResult,
};
