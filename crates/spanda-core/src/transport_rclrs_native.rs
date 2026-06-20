//! Runtime loader for `libspanda_ros2_rclrs_native` (built separately with sourced ROS 2).

use std::ffi::CString;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

type SdkFn = unsafe extern "C" fn() -> bool;
type PublishFn =
    unsafe extern "C" fn(*const std::os::raw::c_char, *const std::os::raw::c_char) -> bool;
type SubscribeFn = unsafe extern "C" fn(*const std::os::raw::c_char) -> bool;
type ServiceCallFn = unsafe extern "C" fn(
    *const std::os::raw::c_char,
    *const std::os::raw::c_char,
    *const std::os::raw::c_char,
) -> bool;
type InitNodeFn = unsafe extern "C" fn(*const std::os::raw::c_char) -> i32;

struct NativeLib {
    sdk_available: SdkFn,
    publish: PublishFn,
    subscribe: SubscribeFn,
    service_call: ServiceCallFn,
    init_node: InitNodeFn,
    _library: libloading::Library,
}

static NATIVE: OnceLock<Option<NativeLib>> = OnceLock::new();

fn library_candidates() -> Vec<PathBuf> {
    // Library candidates.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Vec<PathBuf>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::transport_rclrs_native::library_candidates();

    // Create mutable paths for accumulating results.
    let mut paths = Vec::new();

    // Handle the success value from var.
    if let Ok(path) = std::env::var("SPANDA_ROS2_RCLRS_LIB") {
        paths.push(PathBuf::from(path));
    }

    // Handle the success value from current exe.
    if let Ok(exe) = std::env::current_exe() {
        // Emit output when parent provides a dir.
        if let Some(dir) = exe.parent() {
            #[cfg(target_os = "macos")]
            paths.push(dir.join("libspanda_ros2_rclrs_native.dylib"));
            #[cfg(target_os = "windows")]
            paths.push(dir.join("spanda_ros2_rclrs_native.dll"));
            #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
            paths.push(dir.join("libspanda_ros2_rclrs_native.so"));
        }
    }
    paths.push(PathBuf::from(
        "target/release/libspanda_ros2_rclrs_native.so",
    ));
    paths.push(PathBuf::from("target/debug/libspanda_ros2_rclrs_native.so"));
    paths
}

fn load_library(path: &Path) -> Option<NativeLib> {
    // Load library.
    //
    // Parameters:
    // - `path` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::transport_rclrs_native::load_library(path);

    // Compute library for the following logic.
    let library = unsafe { libloading::Library::new(path).ok()? };
    unsafe {
        let sdk_available = *library.get(b"spanda_ros2_rclrs_sdk_available").ok()?;
        let publish = *library.get(b"spanda_ros2_rclrs_publish").ok()?;
        let subscribe = *library.get(b"spanda_ros2_rclrs_subscribe").ok()?;
        let service_call = *library.get(b"spanda_ros2_rclrs_service_call").ok()?;
        let init_node = *library.get(b"spanda_ros2_rclrs_init_node").ok()?;
        Some(NativeLib {
            sdk_available,
            publish,
            subscribe,
            service_call,
            init_node,
            _library: library,
        })
    }
}

fn native() -> Option<&'static NativeLib> {
    // Native.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::transport_rclrs_native::native();

    // Produce NATIVE as the result.
    NATIVE
        .get_or_init(|| {
            // Process each filesystem path.
            for path in library_candidates() {
                // Continue only when the path is a regular file.
                if path.is_file() {
                    // Emit output when load library provides a lib.
                    if let Some(lib) = load_library(&path) {
                        return Some(lib);
                    }
                }
            }
            None
        })
        .as_ref()
}

fn c_string(value: &str) -> Option<CString> {
    // C string.
    //
    // Parameters:
    // - `value` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::transport_rclrs_native::c_string(value);

    // Produce ok as the result.
    CString::new(value).ok()
}

pub fn sdk_available() -> bool {
    // Sdk available.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::transport_rclrs_native::sdk_available();

    // Produce native as the result.
    native()
        .map(|lib| unsafe { (lib.sdk_available)() })
        .unwrap_or(false)
}

pub fn init_node(name: &str) -> Result<(), String> {
    // Init node.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::transport_rclrs_native::init_node(name);

    // Compute Some for the following logic.
    let Some(lib) = native() else {
        return Err("native rclrs library not loaded — set SPANDA_ROS2_RCLRS_LIB".into());
    };
    let name = c_string(name).ok_or_else(|| "invalid node name".to_string())?;
    let code = unsafe { (lib.init_node)(name.as_ptr()) };

    // Take the branch when code equals 0.
    if code == 0 {
        Ok(())
    } else {
        Err("rclrs init_node failed".into())
    }
}

pub fn publish(topic: &str, payload: &str) -> bool {
    // Publish.
    //
    // Parameters:
    // - `topic` — input value
    // - `payload` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::transport_rclrs_native::publish(topic, payload);

    // Compute Some for the following logic.
    let Some(lib) = native() else {
        return false;
    };
    let Some(topic) = c_string(topic) else {
        return false;
    };
    let Some(payload) = c_string(payload) else {
        return false;
    };
    unsafe { (lib.publish)(topic.as_ptr(), payload.as_ptr()) }
}

pub fn subscribe(topic: &str) -> bool {
    // Subscribe.
    //
    // Parameters:
    // - `topic` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::transport_rclrs_native::subscribe(topic);

    // Compute Some for the following logic.
    let Some(lib) = native() else {
        return false;
    };
    let Some(topic) = c_string(topic) else {
        return false;
    };
    unsafe { (lib.subscribe)(topic.as_ptr()) }
}

pub fn service_call(service: &str, service_type: &str, request: &str) -> bool {
    // Service call.
    //
    // Parameters:
    // - `service` — input value
    // - `service_type` — input value
    // - `request` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::transport_rclrs_native::service_call(service, service_type, request);

    // Compute Some for the following logic.
    let Some(lib) = native() else {
        return false;
    };
    let Some(service) = c_string(service) else {
        return false;
    };
    let Some(service_type) = c_string(service_type) else {
        return false;
    };
    let Some(request) = c_string(request) else {
        return false;
    };
    unsafe { (lib.service_call)(service.as_ptr(), service_type.as_ptr(), request.as_ptr()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_library_does_not_panic() {
        // Missing library does not panic.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::transport_rclrs_native::missing_library_does_not_panic();

        assert!(!sdk_available());
        assert!(!publish("/x", "y"));
    }
}
