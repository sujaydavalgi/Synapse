//! Builtin hardware profile catalog, compatibility verification, and adapter checks.
//!
pub mod adapter_verify;
mod compat;
pub mod connectivity_validate;
mod profiles;
pub mod verify;

pub use compat::{CompatItem, CompatSeverity};
pub use profiles::{builtin_profiles, list_hardware_profiles, HardwareProfile};
pub use verify::{
    build_profile_registry, hardware_profile_from_decl, verify_program_compatibility,
    verify_program_compatibility_legacy, CompatibilityMatrix, CompatibilityReport, MatrixCell,
    VerifyOptions,
};
