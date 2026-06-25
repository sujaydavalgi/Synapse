//! Guardrailed template generation and program suggestions for Spanda.
//!
pub mod generate;
pub mod llm;
pub mod suggest;
pub mod validate;

pub use generate::{
    format_generation_report, generate_health_policy, generate_mission_program, generate_robot_program,
    GenerateBackend, GenerateKind, GenerateOptions, GenerationReport,
};
pub use suggest::{format_suggest_report, suggest_program, SuggestReport, SuggestSeverity, Suggestion};
pub use validate::validate_generated_source;
