//! HAL simulation backend, SoC profiles, and hardware health monitoring for Spanda.
//!
pub mod hal;
pub mod hardware_monitor;
pub mod soc;

pub use hal::{
    create_sim_hal, hal_member_from_decl, HalBackend, HalBusKind, SimHalBackend,
};
pub use hardware_monitor::HardwareMonitor;
pub use soc::{
    get_soc_profile, list_soc_profiles, SocCapability, SocProfile, SocValidationError,
};
