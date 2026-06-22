//! ffi support re-exported from `spanda-ffi` with core bridge wiring.
//!
pub use spanda_ffi::*;

/// Build an FFI registry with Python and C++ bridge fallbacks from `spanda-core`.
pub fn new_with_core_bridges() -> spanda_ffi::FfiRegistry {
    spanda_ffi::FfiRegistry::with_bridges(spanda_ffi::ExternBridges {
        python: Some(crate::bridge::python::call_extern),
        cpp: Some(crate::bridge::cpp::call_extern),
    })
}
