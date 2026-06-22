//! ffi support re-exported from `spanda-ffi` with default bridge wiring.
//!
pub use spanda_ffi::*;

/// Build an FFI registry with Python and C++ bridge fallbacks.
pub fn new_with_core_bridges() -> spanda_ffi::FfiRegistry {
    spanda_bridge::default_ffi_registry()
}
