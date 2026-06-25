//! Autonomous systems scorecard composition for Spanda programs.
//!
pub mod scorecard;

pub use scorecard::{
    evaluate_scorecard, format_scorecard, ScorecardCategory, ScorecardFormat, ScorecardOptions,
    ScorecardReport,
};
