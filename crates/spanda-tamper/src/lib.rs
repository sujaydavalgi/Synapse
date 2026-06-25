//! Verify-time tamper and integrity analysis for Spanda programs.
//!
pub mod detect;

pub use detect::{
    format_tamper_report, generate_tamper_check, TamperFinding, TamperFormat, TamperReport,
    TamperSeverity, TamperStatus,
};
