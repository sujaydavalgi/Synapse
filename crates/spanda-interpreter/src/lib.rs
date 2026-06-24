//! Spanda interpreter — native runtime tree and run API.
//!
mod fleet_http;
pub mod options;
pub mod run;
pub mod runtime;
pub mod simulator;

pub use options::{
    ContinuityRunOptions, ContinuityRunResult, ObstacleConfig, RecoveryRunOptions,
    RecoveryRunResult, RunOptions, RunResult, TestRunResult,
};
pub use run::{run_program, run_tests_with_registry};
pub use runtime::{
    continuity_context_from_request, execute_continuity_on_program, execute_recovery_on_program,
    ContinuityExecutionSnapshot, RecoveryExecutionSnapshot,
};
pub use runtime::{Interpreter, InterpreterOptions, RobotBackend};
pub use simulator::{create_default_simulator, Obstacle, Simulator, SimulatorConfig};
pub use spanda_error::SpandaError;
pub use spanda_runtime::replay::{MissionTrace, PlaybackReport, TraceVerification};
pub use spanda_runtime::robot_state::{PoseState, VelocityState};
pub use spanda_runtime::telemetry::ExecutionMetrics;
pub use spanda_runtime::RuntimeHost;

/// In-process simulator backend implementing [`RobotBackend`].
pub type SimRobotBackend = Simulator;
