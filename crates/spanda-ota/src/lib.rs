//! OTA deployment runtime extracted from Spanda core for lean-core package architecture.
//!
pub mod agent;
pub mod bundle;
pub mod remote;
pub mod service;
pub mod types;

pub use agent::*;
pub use bundle::*;
pub use remote::*;
pub use service::*;
pub use types::*;
