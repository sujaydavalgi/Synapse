//! In-process Python bridge via PyO3 (optional `python-native` feature).
//!
//! Loads `scripts/spanda_python_bridge.py` handlers directly when enabled.
//! Falls back to subprocess bridge when this module is unavailable or fails.

use spanda_error::SpandaError;
use spanda_ast::foundations::ExternFnDecl;
use spanda_runtime::value::RuntimeValue;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use super::protocol::{json_to_runtime_value, runtime_value_to_json};
use super::python::bridge_script_path;

pub fn native_available() -> bool {
    // Returns `true` when the Python bridge script is available for in-process use.
    //
    // Returns:
    //

    // `true` if [`super::python::bridge_script_path`] resolves.
    bridge_script_path().is_some()
}

pub fn call_extern(
    decl: &ExternFnDecl,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, SpandaError> {
    // Invoke a Python extern via PyO3 in-process bridge.
    //
    // Parameters:
    //
    // - `decl` — `extern python fn` declaration.
    // - `args` — Runtime arguments.
    //
    // Returns:
    //
    // Handler result, or [`SpandaError`] on load/execution failure.
    //
    // Options:
    //

    // Requires `python-native` Cargo feature and bridge script on disk.
    let line = decl.span.start.line;
    let script = bridge_script_path().ok_or_else(|| SpandaError::Runtime {
        message: "Python bridge script not found for native bridge".into(),
        line,
    })?;
    let args_json =
        serde_json::to_string(&args.iter().map(runtime_value_to_json).collect::<Vec<_>>())
            .map_err(|e| SpandaError::Runtime {
                message: format!("Failed to encode native bridge args: {e}"),
                line,
            })?;
    Python::attach(|py| -> PyResult<RuntimeValue> {
        let locals = PyDict::new(py);
        locals.set_item("script_path", script.to_string_lossy().to_string())?;
        locals.set_item("fn_name", &decl.name)?;
        locals.set_item("args_json", args_json)?;
        py.run(
            c"import json, importlib.util
spec = importlib.util.spec_from_file_location('spanda_python_bridge', script_path)
if spec is None or spec.loader is None:
    raise RuntimeError('failed to load python bridge module')
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
args = json.loads(args_json)
handler = mod.HANDLERS.get(fn_name)
if handler is None:
    response = json.dumps({'ok': False, 'error': f\"Unknown python extern '{fn_name}'\"})
else:
    result = handler(*args)
    response = json.dumps({'ok': True, 'result': result})",
            None,
            Some(&locals),
        )?;
        let response: String = locals
            .get_item("response")?
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("missing response"))?
            .extract()?;
        let parsed: serde_json::Value = serde_json::from_str(&response).map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("invalid bridge json: {e}"))
        })?;

        // Take this path when parsed.
        if parsed
            .get("ok")
            .and_then(|v| v.as_bool())
            .is_some_and(|ok| !ok)
        {
            let msg = parsed
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Python native bridge call failed")
                .to_string();
            return Err(pyo3::exceptions::PyRuntimeError::new_err(msg));
        }
        Ok(json_to_runtime_value(
            parsed.get("result").unwrap_or(&serde_json::Value::Null),
            &decl.return_type,
        ))
    })
    .map_err(|e| SpandaError::Runtime {
        message: format!("Python native bridge error: {e}"),
        line,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_ast::nodes::{SourceLocation, Span, SpandaType};
    use spanda_ast::foundations::BridgeKind;

    fn test_decl(name: &str) -> ExternFnDecl {
        // Test decl.
        //
        // Parameters:
        // - `name` — input value
        //
        // Returns:
        // ExternFnDecl.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::python_native::test_decl(name);

        // Produce ExternFnDecl as the result.
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
    fn native_py_add_when_available() {
        // Native py add when available.
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
        // let result = spanda_core::python_native::native_py_add_when_available();

        if !native_available() {
            return;
        }
        let decl = test_decl("py_add");
        let result = call_extern(
            &decl,
            &[
                RuntimeValue::Number {
                    value: 3.0,
                    unit: spanda_ast::nodes::UnitKind::None,
                },
                RuntimeValue::Number {
                    value: 4.0,
                    unit: spanda_ast::nodes::UnitKind::None,
                },
            ],
        )
        .expect("py_add native");
        assert!(matches!(
            result,
            RuntimeValue::Number { value, .. } if (value - 7.0).abs() < f64::EPSILON
        ));
    }
}
