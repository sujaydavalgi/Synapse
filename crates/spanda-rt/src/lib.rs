//! Native runtime linked with LLVM-generated Spanda programs.
//!
//! Exposes a stable C ABI (`spanda_rt_*`) for actuator calls, logging, and
//! emergency stop. The interpreter/simulator remains authoritative for now;
//! this crate is the link target for Milestone 2 LLVM codegen.

use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::Mutex;

static EVENTS: Mutex<Vec<String>> = Mutex::new(Vec::new());

#[no_mangle]
pub extern "C" fn spanda_rt_drive(actuator: *const c_char, linear: f64, angular: f64) {
    let name = ptr_to_str(actuator);
    log_event(format!("drive:{name}:{linear}:{angular}"));
}

#[no_mangle]
pub extern "C" fn spanda_rt_stop(actuator: *const c_char) {
    let name = ptr_to_str(actuator);
    log_event(format!("stop:{name}"));
}

#[no_mangle]
pub extern "C" fn spanda_rt_emergency_stop() {
    log_event("emergency_stop".into());
}

#[no_mangle]
pub extern "C" fn spanda_rt_log_i32(tag: *const c_char, value: i32) {
    let name = ptr_to_str(tag);
    log_event(format!("log:{name}:{value}"));
}

/// Test helper: drain and return recorded runtime events.
pub fn take_events() -> Vec<String> {
    EVENTS.lock().unwrap().drain(..).collect()
}

fn log_event(msg: String) {
    if let Ok(mut events) = EVENTS.lock() {
        events.push(msg);
    }
}

fn ptr_to_str(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return "<null>".into();
    }
    unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn c_abi_records_actuator_calls() {
        let wheels = CString::new("wheels").unwrap();
        spanda_rt_drive(wheels.as_ptr(), 0.5, 0.1);
        spanda_rt_stop(wheels.as_ptr());
        let events = take_events();
        assert_eq!(events.len(), 2);
        assert!(events[0].starts_with("drive:wheels:"));
        assert_eq!(events[1], "stop:wheels");
    }
}
