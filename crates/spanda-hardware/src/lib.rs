//! Builtin hardware profile catalog for package validation and verify.
//!
mod compat;
mod profiles;

pub use compat::{CompatItem, CompatSeverity};
pub use profiles::{builtin_profiles, list_hardware_profiles, HardwareProfile};
