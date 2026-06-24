//! Run options and results for the Spanda interpreter.
//!
use serde::{Deserialize, Serialize};
use spanda_ffi::FfiRegistry;
use spanda_runtime::replay::MissionTrace;
use spanda_runtime::robot_state::{PoseState, RobotState};
use spanda_runtime::scheduler::SchedulerClock;
use spanda_runtime::telemetry::RuntimeTelemetry;
use spanda_typecheck::ModuleRegistry;

#[derive(Clone, Default, Serialize, Deserialize)]
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
    pub module_registry: Option<ModuleRegistry>,
    #[serde(default)]
    pub trace_scheduler: bool,
    #[serde(default)]
    pub trace_tasks: bool,
    #[serde(default)]
    pub replay_trace: bool,
    #[serde(default)]
    pub trace_triggers: bool,
    #[serde(default)]
    pub trace_events: bool,
    #[serde(default)]
    pub trace_providers: bool,
    #[serde(default)]
    pub trace_realtime: bool,
    #[serde(default)]
    pub record_trace: bool,
    #[serde(default)]
    pub trace_source: Option<String>,
    #[serde(default)]
    pub trace_output: Option<String>,
    #[serde(default)]
    pub metrics_json: bool,
    #[serde(default)]
    pub replay_deterministic: bool,
    #[serde(default)]
    pub replay_from_ms: Option<f64>,
    #[serde(default)]
    pub scheduler_clock: SchedulerClock,
    #[serde(default)]
    pub playback_wall_clock: bool,
    #[serde(default)]
    pub twin_export_path: Option<String>,
    #[serde(default)]
    pub secure_mode: bool,
    #[serde(default)]
    pub inject_security_faults: bool,
    #[serde(default)]
    pub enforce_certify: bool,
    #[serde(default)]
    pub official_packages: Vec<String>,
    #[serde(default)]
    pub trigger_kill_switch: Option<String>,
    #[serde(default)]
    pub kill_switch_signature: Option<String>,
    #[serde(default)]
    pub inject_health_faults: bool,
    #[serde(default)]
    pub persist_telemetry: bool,

    /// Inbound comm payloads queued before each recovery approval poll (test/sim hook).
    #[serde(default)]
    pub inbound_comm_messages: Vec<(String, String)>,
    #[serde(skip)]
    pub ffi_registry: Option<FfiRegistry>,
}

fn default_max_loop_iterations() -> usize {
    // Description:
    //     Default max loop iterations.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: usize
    //         Return value from `default_max_loop_iterations`.
    //
    // Example:

    //     let result = spanda_interpreter::options::default_max_loop_iterations();

    10
}

fn default_lidar_range() -> f64 {
    // Description:
    //     Default lidar range.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: f64
    //         Return value from `default_lidar_range`.
    //
    // Example:

    //     let result = spanda_interpreter::options::default_lidar_range();

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
    pub metrics: RuntimeTelemetry,
    #[serde(default)]
    pub mission_trace: Option<MissionTrace>,
    #[serde(default)]
    pub twin_replay: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunResult {
    pub passed: usize,
    pub failed: usize,
    pub logs: Vec<String>,
}

/// Options for targeted interpreter recovery without a full program run loop.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecoveryRunOptions {
    #[serde(default)]
    pub robot_name: Option<String>,
    #[serde(default)]
    pub grant_operator_approval: bool,
    #[serde(default)]
    pub inbound_comm_messages: Vec<(String, String)>,
}

/// Outcome of interpreter-backed recovery execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryRunResult {
    pub recovery: spanda_assurance::RecoveryResult,
    pub logs: Vec<String>,
    pub active_mode: String,
    pub mission_paused: bool,
    pub speed_cap: Option<f64>,
}

/// Options for targeted interpreter continuity takeover without a full run loop.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContinuityRunOptions {
    #[serde(default)]
    pub robot_name: Option<String>,
    #[serde(default)]
    pub successor: Option<String>,
    #[serde(default)]
    pub grant_operator_approval: bool,
    #[serde(default)]
    pub inbound_comm_messages: Vec<(String, String)>,
}

/// Outcome of interpreter-backed continuity takeover execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuityRunResult {
    pub takeover: spanda_assurance::TakeoverReport,
    pub logs: Vec<String>,
    pub mission_progress_percent: f64,
    pub handoff_from: Option<String>,
    pub mission_paused: bool,
}
