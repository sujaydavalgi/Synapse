//! Shared JSON stdin/stdout protocol for subprocess FFI bridges.

use crate::ast::SpandaType;
use crate::error::SpandaError;
use crate::foundations::ExternFnDecl;
use crate::runtime::RuntimeValue;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Serialize)]
pub struct BridgeRequest<'a> {
    #[serde(rename = "fn")]
    pub fn_name: &'a str,
    pub args: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct BridgeResponse {
    pub ok: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub fn runtime_value_to_json(value: &RuntimeValue) -> serde_json::Value {
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
    use crate::ast::UnitKind;
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
