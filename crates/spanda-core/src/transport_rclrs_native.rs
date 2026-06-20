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
    let mut paths = Vec::new();
    if let Ok(path) = std::env::var("SPANDA_ROS2_RCLRS_LIB") {
        paths.push(PathBuf::from(path));
    }
    if let Ok(exe) = std::env::current_exe() {
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
    NATIVE
        .get_or_init(|| {
            for path in library_candidates() {
                if path.is_file() {
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
    CString::new(value).ok()
}

pub fn sdk_available() -> bool {
    native()
        .map(|lib| unsafe { (lib.sdk_available)() })
        .unwrap_or(false)
}

pub fn init_node(name: &str) -> Result<(), String> {
    let Some(lib) = native() else {
        return Err("native rclrs library not loaded — set SPANDA_ROS2_RCLRS_LIB".into());
    };
    let name = c_string(name).ok_or_else(|| "invalid node name".to_string())?;
    let code = unsafe { (lib.init_node)(name.as_ptr()) };
    if code == 0 {
        Ok(())
    } else {
        Err("rclrs init_node failed".into())
    }
}

pub fn publish(topic: &str, payload: &str) -> bool {
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
    let Some(lib) = native() else {
        return false;
    };
    let Some(topic) = c_string(topic) else {
        return false;
    };
    unsafe { (lib.subscribe)(topic.as_ptr()) }
}

pub fn service_call(service: &str, service_type: &str, request: &str) -> bool {
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
        assert!(!sdk_available());
        assert!(!publish("/x", "y"));
    }
}
