//! Persistent ROS2 daemon subprocess (rclpy) for `SPANDA_ROS2_RCLRS` in-process I/O.

use crate::bridge::python::python_available;
use crate::runtime::RuntimeValue;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Mutex;

static DAEMON: Mutex<Option<Ros2Daemon>> = Mutex::new(None);

struct Ros2Daemon {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
}

impl Ros2Daemon {
    fn start() -> Result<Self, String> {
        // Start.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::transport_rclrs_daemon::start();

        // Compute script for the following logic.
        let script = daemon_script_path()?;
        let python = python_cmd().ok_or_else(|| "python3 not found for ROS2 daemon".to_string())?;
        let mut child = Command::new(&python)
            .arg(&script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("failed to start ROS2 daemon: {e}"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "daemon stdin unavailable".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "daemon stdout unavailable".to_string())?;
        Ok(Self {
            child,
            stdin,
            reader: BufReader::new(stdout),
        })
    }

    fn request(&mut self, op: &str, args: &[String]) -> bool {
        // Request.
        //
        // Parameters:
        // - `self` — method receiver
        // - `op` — input value
        // - `args` — input value
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.request(op, args);

        // Compute payload for the following logic.
        let payload = serde_json::json!({ "op": op, "args": args });
        let line = match serde_json::to_string(&payload) {
            Ok(text) => text,
            Err(_) => return false,
        };

        // Take this path when writeln!(self.stdin, "{line}").is err().
        if writeln!(self.stdin, "{line}").is_err() {
            return false;
        }

        // Take this path when self.stdin.flush().is err().
        if self.stdin.flush().is_err() {
            return false;
        }
        let mut response = String::new();

        // Take this path when self.reader.read line(&mut response).is err().
        if self.reader.read_line(&mut response).is_err() {
            return false;
        }
        serde_json::from_str::<serde_json::Value>(&response)
            .ok()
            .and_then(|value| value.get("ok").and_then(|ok| ok.as_bool()))
            .unwrap_or(false)
    }
}

fn python_cmd() -> Option<String> {
    // Python cmd.
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
    // let result = spanda_core::transport_rclrs_daemon::python_cmd();

    // Iterate over ["python3", "python"].
    for cmd in ["python3", "python"] {
        // Take this path when Command::new(cmd).
        if Command::new(cmd)
            .arg("-c")
            .arg("import sys")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return Some(cmd.to_string());
        }
    }
    None
}

pub fn daemon_script_path() -> Result<PathBuf, String> {
    // Daemon script path.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::transport_rclrs_daemon::daemon_script_path();

    // handle the success value from var.
    if let Ok(path) = std::env::var("SPANDA_ROS2_DAEMON_SCRIPT") {
        let path = PathBuf::from(path);

        // Continue only when the path is a regular file.
        if path.is_file() {
            return Ok(path);
        }
    }

    // Handle the success value from var.
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = PathBuf::from(manifest)
            .join("../../scripts/spanda_ros2_daemon.py")
            .canonicalize()
            .ok();

        // Emit output when path provides a path.
        if let Some(path) = path {
            // Continue only when the path is a regular file.
            if path.is_file() {
                return Ok(path);
            }
        }
    }
    let path = PathBuf::from("scripts/spanda_ros2_daemon.py");

    // Continue only when the path is a regular file.
    if path.is_file() {
        return Ok(path);
    }
    Err("spanda_ros2_daemon.py not found".into())
}

fn with_daemon<F>(f: F) -> bool
where
    F: FnOnce(&mut Ros2Daemon) -> bool,
{
    // take the branch when python available is false.
    if !python_available() {
        return false;
    }
    let mut guard = match DAEMON.lock() {
        Ok(guard) => guard,
        Err(_) => return false,
    };

    // Take this path when guard.is none().
    if guard.is_none() {
        // Match on start and handle each case.
        match Ros2Daemon::start() {
            Ok(daemon) => *guard = Some(daemon),
            Err(_) => return false,
        }
    }
    let daemon = guard.as_mut().expect("daemon");

    // Proceed only when is some is available.
    if daemon.child.try_wait().ok().flatten().is_some() {
        // Match on start and handle each case.
        match Ros2Daemon::start() {
            Ok(restarted) => *daemon = restarted,
            Err(_) => {
                *guard = None;
                return false;
            }
        }
    }
    f(guard.as_mut().expect("daemon"))
}

pub fn daemon_publish(topic: &str, value: &RuntimeValue) -> bool {
    // Daemon publish.
    //
    // Parameters:
    // - `topic` — input value
    // - `value` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::transport_rclrs_daemon::daemon_publish(topic, value);

    // Compute payload for the following logic.
    let payload = match value {
        RuntimeValue::String { value } => value.clone(),
        RuntimeValue::Number { value, .. } => value.to_string(),
        RuntimeValue::Bool { value } => value.to_string(),
        other => format!("{other:?}"),
    };
    with_daemon(|daemon| daemon.request("publish", &[topic.to_string(), payload]))
}

pub fn daemon_subscribe(topic: &str) -> bool {
    // Daemon subscribe.
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
    // let result = spanda_core::transport_rclrs_daemon::daemon_subscribe(topic);

    // Produce to string as the result.
    with_daemon(|daemon| daemon.request("subscribe", &[topic.to_string()]))
}

pub fn daemon_service_call(service: &str, service_type: &str, request: &str) -> bool {
    // Daemon service call.
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
    // let result = spanda_core::transport_rclrs_daemon::daemon_service_call(service, service_type, request);

    // Produce with daemon as the result.
    with_daemon(|daemon| {
        daemon.request(
            "service_call",
            &[
                service.to_string(),
                service_type.to_string(),
                request.to_string(),
            ],
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daemon_script_resolves_in_repo() {
        // Daemon script resolves in repo.
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
        // let result = spanda_core::transport_rclrs_daemon::daemon_script_resolves_in_repo();

        if std::env::var("CARGO_MANIFEST_DIR").is_ok() {
            assert!(daemon_script_path().is_ok());
        }
    }
}
