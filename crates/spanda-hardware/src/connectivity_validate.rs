//! Connectivity declaration validation — re-exported from spanda-connectivity-runtime.
//!
//! The validation logic and its AST dependency now live in `spanda-connectivity-runtime`
//! (layer 2). This module re-exports the public surface for backwards compatibility.

pub use spanda_connectivity::{CompatItem, CompatSeverity, HardwareProfile};
