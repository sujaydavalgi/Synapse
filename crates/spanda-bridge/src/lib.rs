//! Python and C++ subprocess bridges for `extern` function calls.
//!
pub mod cpp;
#[cfg(feature = "cpp-native")]
pub mod cpp_native;
pub mod protocol;
pub mod python;
#[cfg(feature = "python-native")]
pub mod python_native;

use spanda_ffi::{ExternBridges, FfiRegistry};

/// Build an FFI registry wired to the default Python and C++ subprocess bridges.
pub fn default_ffi_registry() -> FfiRegistry {
    FfiRegistry::with_bridges(ExternBridges {
        python: Some(python::call_extern),
        cpp: Some(cpp::call_extern),
    })
}
