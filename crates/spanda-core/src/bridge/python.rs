//! Subprocess Python bridge for `extern python fn` declarations.
//!
//! Invokes `scripts/spanda_python_bridge.py` (or `SPANDA_PYTHON_BRIDGE`) with a
//! JSON stdin/stdout protocol. This is a real (minimal) integration — not a stub.

use crate::ast::SpandaType;
use crate::error::SpandaError;
use crate::foundations::ExternFnDecl;
use crate::runtime::RuntimeValue;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Serialize)]
struct BridgeRequest<'a> {
    #[serde(rename = "fn")]
    fn_name: &'a str,
    args: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct BridgeResponse {
    ok: bool,
    result: Option<serde_json::Value>,
    error: Option<String>,
}

/// Resolve the Python bridge script path.
pub fn bridge_script_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("SPANDA_PYTHON_BRIDGE") {
        let p = PathBuf::from(path);
        if p.is_file() {
            return Some(p);
        }
    }
    candidate_script_paths()
        .into_iter()
        .find(|candidate| candidate.is_file())
}

fn candidate_script_paths() -> Vec<PathBuf> {
    let mut paths = vec![
        PathBuf::from("scripts/spanda_python_bridge.py"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/spanda_python_bridge.py"),
    ];
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join("scripts/spanda_python_bridge.py"));
    }
    paths
}

pub fn python_available() -> bool {
    python_command().is_some()
}

fn python_command() -> Option<String> {
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

fn runtime_value_to_json(value: &RuntimeValue) -> serde_json::Value {
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

fn json_to_runtime_value(value: &serde_json::Value, return_type: &SpandaType) -> RuntimeValue {
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

pub fn call_extern(
    decl: &ExternFnDecl,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, SpandaError> {
    let line = decl.span.start.line;
    let script = bridge_script_path().ok_or_else(|| SpandaError::Runtime {
        message: "Python bridge script not found — set SPANDA_PYTHON_BRIDGE or run from repo root"
            .into(),
        line,
    })?;
    let python = python_command().ok_or_else(|| SpandaError::Runtime {
        message: "Python interpreter not found (install python3 for extern python fn)".into(),
        line,
    })?;

    let request = BridgeRequest {
        fn_name: &decl.name,
        args: args.iter().map(runtime_value_to_json).collect(),
    };
    let request_json = serde_json::to_string(&request).map_err(|e| SpandaError::Runtime {
        message: format!("Failed to encode Python bridge request: {e}"),
        line,
    })?;

    let mut child = Command::new(&python)
        .arg(script.as_os_str())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| SpandaError::Runtime {
            message: format!("Failed to spawn Python bridge: {e}"),
            line,
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(request_json.as_bytes())
            .map_err(|e| SpandaError::Runtime {
                message: format!("Failed to write Python bridge request: {e}"),
                line,
            })?;
        stdin.write_all(b"\n").ok();
    }

    let output = child.wait_with_output().map_err(|e| SpandaError::Runtime {
        message: format!("Python bridge process failed: {e}"),
        line,
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SpandaError::Runtime {
            message: format!(
                "Python bridge exited with {}: {}",
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
                "Invalid Python bridge response: {e} (got: {})",
                stdout.trim()
            ),
            line,
        })?;

    if !resp.ok {
        return Err(SpandaError::Runtime {
            message: resp
                .error
                .unwrap_or_else(|| "Python bridge call failed".into()),
            line,
        });
    }

    Ok(json_to_runtime_value(
        &resp.result.unwrap_or(serde_json::Value::Null),
        &decl.return_type,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{SourceLocation, Span, SpandaType};
    use crate::foundations::BridgeKind;

    fn test_decl(name: &str) -> ExternFnDecl {
        ExternFnDecl {
            name: name.into(),
            library: Some("python".into()),
            bridge: BridgeKind::Python,
            params: vec![],
            return_type: SpandaType::Int,
            span: Span {
                start: SourceLocation {
                    line: 1,
                    column: 1,
                    offset: 0,
                },
                end: SourceLocation {
                    line: 1,
                    column: 1,
                    offset: 0,
                },
            },
        }
    }

    #[test]
    fn subprocess_py_add_when_python_available() {
        if !python_available() || bridge_script_path().is_none() {
            return;
        }
        let decl = test_decl("py_add");
        let result = call_extern(
            &decl,
            &[
                RuntimeValue::Number {
                    value: 4.0,
                    unit: crate::ast::UnitKind::None,
                },
                RuntimeValue::Number {
                    value: 5.0,
                    unit: crate::ast::UnitKind::None,
                },
            ],
        )
        .expect("py_add");
        assert!(matches!(
            result,
            RuntimeValue::Number { value, .. } if (value - 9.0).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn subprocess_unknown_fn_errors() {
        if !python_available() || bridge_script_path().is_none() {
            return;
        }
        let decl = test_decl("py_missing");
        let err = call_extern(&decl, &[]).unwrap_err();
        assert!(err.to_string().contains("Unknown python extern"));
    }
}
