//! Robot pose, velocity, and aggregate state for provider and simulator contracts.
//!
use serde::{Deserialize, Serialize};

/// Aggregate robot pose, velocity, and safety stop flag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotState {
    pub pose: PoseState,
    pub velocity: VelocityState,
    pub emergency_stop: bool,
}

/// Planar or 3D pose snapshot used by actuators and simulators.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoseState {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z: Option<f64>,
}

/// Linear and angular velocity snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VelocityState {
    pub linear: f64,
    pub angular: f64,
}
