//! Static threat modeling for Spanda autonomous system programs.
//!
pub mod model;

pub use model::{
    analyze_threat_model, format_threat_report, AttackSurfaceItem, ThreatAssessment, ThreatCategory,
    ThreatReport, ThreatRisk,
};
