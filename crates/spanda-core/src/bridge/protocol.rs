//! Shared JSON stdin/stdout protocol for subprocess FFI bridges.
//!
//! Defines request/response envelopes and helpers to spawn bridge processes
//! for Python and C++ extern function calls.

use crate::ast::SpandaType;
use crate::error::SpandaError;
use crate::foundations::ExternFnDecl;
use crate::runtime::RuntimeValue;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// JSON request envelope sent to a bridge subprocess on stdin.
#[derive(Serialize)]
pub struct BridgeRequest<'a> {
    /// Extern function name to invoke.
    #[serde(rename = "fn")]
    pub fn_name: &'a str,

    /// JSON-encoded argument values.
    pub args: Vec<serde_json::Value>,
}

/// JSON response envelope read from a bridge subprocess stdout.
#[derive(Deserialize)]
pub struct BridgeResponse {
    /// `true` when the handler succeeded.
    pub ok: bool,

    /// Handler return value when `ok` is true.
    pub result: Option<serde_json::Value>,

    /// Error message when `ok` is false.
    pub error: Option<String>,
}

pub fn runtime_value_to_json(value: &RuntimeValue) -> serde_json::Value {
    // Convert a [`RuntimeValue`] to JSON for bridge IPC.
    //
    // Parameters:
    //
    // - `value` — Runtime argument or result fragment.
    //
    // Returns:
    //
    // JSON value (numbers, bools, strings; opaque debug for other variants).
    //
    // Example:
    //
    // use spanda_core::bridge::protocol::runtime_value_to_json;
    // use spanda_core::runtime::RuntimeValue;
    // let json = runtime_value_to_json(&RuntimeValue::Bool { value: true });

    // assert_eq!(json, serde_json::json!(true));
    match value {
        RuntimeValue::Number { value, .. } => serde_json::Value::Number(
            serde_json::Number::from_f64(*value).unwrap_or_else(|| serde_json::Number::from(0)),
        ),
        RuntimeValue::Bool { value } => serde_json::Value::Bool(*value),
        RuntimeValue::String { value } => serde_json::Value::String(value.clone()),
        RuntimeValue::Void => serde_json::Value::Null,
        other => serde_json::Value::String(format!("{other:?}")),
    }
}

pub fn json_to_runtime_value(value: &serde_json::Value, return_type: &SpandaType) -> RuntimeValue {
    // Convert bridge JSON back to a [`RuntimeValue`] using the declared return type.
    //
    // Parameters:
    //
    // - `value` — JSON result from the bridge.
    // - `return_type` — Spanda type annotation for coercion.
    //
    // Returns:
    //

    // Coerced [`RuntimeValue`] (defaults for missing fields).
    use crate::ast::UnitKind;

    // Match on return type and handle each case.
    match return_type {
        SpandaType::Bool => RuntimeValue::Bool {
            value: value.as_bool().unwrap_or(false),
        },
        SpandaType::String => RuntimeValue::String {
            value: value.as_str().unwrap_or("").to_string(),
        },
        SpandaType::Int | SpandaType::Float | SpandaType::Number { .. } => RuntimeValue::Number {
            value: value.as_f64().unwrap_or(0.0),
            unit: UnitKind::None,
        },
        _ => match value {
            serde_json::Value::Number(n) => RuntimeValue::Number {
                value: n.as_f64().unwrap_or(0.0),
                unit: UnitKind::None,
            },
            serde_json::Value::Bool(b) => RuntimeValue::Bool { value: *b },
            serde_json::Value::String(s) => RuntimeValue::String { value: s.clone() },
            _ => RuntimeValue::Void,
        },
    }
}

pub fn call_subprocess_bridge(
    bridge_label: &str,
    executable: &Path,
    extra_args: &[&str],
    decl: &ExternFnDecl,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, SpandaError> {
    // Spawn a bridge executable, send a [`BridgeRequest`], and parse the response.
    //
    // Parameters:
    //
    // - `bridge_label` — Human label for error messages (`"Python"`, `"C++"`).
    // - `executable` — Path to the bridge interpreter or binary.
    // - `extra_args` — Additional argv entries (e.g. script path for Python).
    // - `decl` — Extern declaration (name, return type, span for errors).
    // - `args` — Runtime call arguments.
    //
    // Returns:
    //
    // Handler result as [`RuntimeValue`], or [`SpandaError`] on spawn/IO/JSON failure.
    //
    // Options:
    //
    // - Writes one JSON line to stdin and expects one JSON line on stdout.
    //
    // Example:
    //
    // use spanda_core::bridge::protocol::call_subprocess_bridge;

    // // Typically invoked via bridge::python::call_extern or bridge::cpp::call_extern.
    let line = decl.span.start.line;
    let request = BridgeRequest {
        fn_name: &decl.name,
        args: args.iter().map(runtime_value_to_json).collect(),
    };
    let request_json = serde_json::to_string(&request).map_err(|e| SpandaError::Runtime {
        message: format!("Failed to encode {bridge_label} bridge request: {e}"),
        line,
    })?;
    let mut command = Command::new(executable);
    command
        .args(extra_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|e| SpandaError::Runtime {
        message: format!("Failed to spawn {bridge_label} bridge: {e}"),
        line,
    })?;

    // Take this path when let Some(mut stdin) = child.stdin.take().
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(request_json.as_bytes())
            .map_err(|e| SpandaError::Runtime {
                message: format!("Failed to write {bridge_label} bridge request: {e}"),
                line,
            })?;
        stdin.write_all(b"\n").ok();
    }
    let output = child.wait_with_output().map_err(|e| SpandaError::Runtime {
        message: format!("{bridge_label} bridge process failed: {e}"),
        line,
    })?;

    // Handle output when the subprocess succeeds.
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SpandaError::Runtime {
            message: format!(
                "{bridge_label} bridge exited with {}: {}",
                output.status,
                stderr.trim()
            ),
            line,
        });
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let resp: BridgeResponse =
        serde_json::from_str(stdout.trim()).map_err(|e| SpandaError::Runtime {
            message: format!(
                "Invalid {bridge_label} bridge response: {e} (got: {})",
                stdout.trim()
            ),
            line,
        })?;

    // Take the branch when ok is false.
    if !resp.ok {
        return Err(SpandaError::Runtime {
            message: resp
                .error
                .unwrap_or_else(|| format!("{bridge_label} bridge call failed")),
            line,
        });
    }
    Ok(json_to_runtime_value(
        &resp.result.unwrap_or(serde_json::Value::Null),
        &decl.return_type,
    ))
}
