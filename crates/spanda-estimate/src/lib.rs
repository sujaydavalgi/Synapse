//! Mission resource estimation for Spanda autonomous system programs.
//!
pub mod estimate;

pub use estimate::{
    estimate_mission, format_mission_estimate, EstimateFormat, EstimateOptions,
    MissionEstimateReport, ResourceConfidence, ResourceEstimate,
};
