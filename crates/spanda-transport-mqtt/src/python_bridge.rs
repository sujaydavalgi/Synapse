//! Subprocess Python bridge for MQTT live transport fallbacks.
//!
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(serde::Serialize)]
struct BridgeRequest<'a> {
    #[serde(rename = "fn")]
    fn_name: &'a str,
    args: Vec<String>,
}

#[derive(serde::Deserialize)]
struct BridgeResponse {
    ok: bool,
    #[allow(dead_code)]
    result: Option<serde_json::Value>,
    #[allow(dead_code)]
    error: Option<String>,
}

fn python_cmd() -> Option<String> {
    for cmd in ["python3", "python"] {
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

fn bridge_script_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("SPANDA_PYTHON_BRIDGE") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }
    let mut paths = vec![
        PathBuf::from("scripts/spanda_python_bridge.py"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/spanda_python_bridge.py"),
    ];
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join("scripts/spanda_python_bridge.py"));
    }
    paths.into_iter().find(|candidate| candidate.is_file())
}

fn invoke_python_bridge(fn_name: &str, args: &[String]) -> bool {
    let python = match python_cmd() {
        Some(cmd) => cmd,
        None => return false,
    };
    let script = match bridge_script_path() {
        Some(path) => path,
        None => return false,
    };
    let request = BridgeRequest {
        fn_name,
        args: args.to_vec(),
    };
    let request_json = match serde_json::to_string(&request) {
        Ok(text) => text,
        Err(_) => return false,
    };
    let mut child = match Command::new(&python)
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return false,
    };
    if let Some(mut stdin) = child.stdin.take() {
        if stdin.write_all(request_json.as_bytes()).is_err() {
            return false;
        }
        let _ = stdin.write_all(b"\n");
    }
    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(_) => return false,
    };
    if !output.status.success() {
        return false;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let resp: BridgeResponse = match serde_json::from_str(stdout.trim()) {
        Ok(resp) => resp,
        Err(_) => return false,
    };
    resp.ok
}

pub fn mqtt_live_enabled() -> bool {
    std::env::var("SPANDA_MQTT_LIVE").is_ok()
}

pub fn try_mqtt_publish(topic: &str, payload: &str) -> bool {
    if !mqtt_live_enabled() {
        return false;
    }
    invoke_python_bridge(
        "mqtt_publish",
        &[topic.to_string(), payload.to_string()],
    )
}
