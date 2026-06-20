//! Subprocess Python bridge for `extern python fn` declarations.
//!
//! Invokes `scripts/spanda_python_bridge.py` (or `SPANDA_PYTHON_BRIDGE`) with a
//! JSON stdin/stdout protocol. This is a real (minimal) integration — not a stub.

use crate::error::SpandaError;
use crate::foundations::ExternFnDecl;
use crate::runtime::RuntimeValue;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use super::protocol::call_subprocess_bridge;

/// Resolve the Python bridge script path.
pub fn bridge_script_path() -> Option<PathBuf> {
    // Bridge script path.
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
    // let result = spanda_core::python::bridge_script_path();

    // handle the success value from var.
    if let Ok(path) = std::env::var("SPANDA_PYTHON_BRIDGE") {
        let p = PathBuf::from(path);

        // Continue only when the path is a regular file.
        if p.is_file() {
            return Some(p);
        }
    }
    candidate_script_paths()
        .into_iter()
        .find(|candidate| candidate.is_file())
}

fn candidate_script_paths() -> Vec<PathBuf> {
    // Candidate script paths.
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
    // let result = spanda_core::python::candidate_script_paths();

    // Create mutable paths for accumulating results.
    let mut paths = vec![
        PathBuf::from("scripts/spanda_python_bridge.py"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/spanda_python_bridge.py"),
    ];

    // Handle the success value from current dir.
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join("scripts/spanda_python_bridge.py"));
    }
    paths
}

pub fn python_available() -> bool {
    // Python available.
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
    // let result = spanda_core::python::python_available();

    // Produce is some as the result.
    python_command().is_some()
}

fn python_command() -> Option<String> {
    // Python command.
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
    // let result = spanda_core::python::python_command();

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

pub fn call_extern(
    decl: &ExternFnDecl,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, SpandaError> {
    // Call extern.
    //
    // Parameters:
    // - `decl` — input value
    // - `args` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::python::call_extern(decl, args);

    // Produce #[cfg as the result.
    #[cfg(feature = "python-native")]
    // Take this path when std::env::var("SPANDA PYTHON SUBPROCESS").is err().
    if std::env::var("SPANDA_PYTHON_SUBPROCESS").is_err() {
        // Take this path when super::python native::native available().
        if super::python_native::native_available() {
            return super::python_native::call_extern(decl, args);
        }
    }
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
    call_subprocess_bridge(
        "Python",
        &PathBuf::from(&python),
        &[script.to_str().unwrap()],
        decl,
        args,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{SourceLocation, Span, SpandaType};
    use crate::foundations::BridgeKind;

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
        // let result = spanda_core::python::test_decl(name);

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
    fn subprocess_py_add_when_python_available() {
        // Subprocess py add when python available.
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
        // let result = spanda_core::python::subprocess_py_add_when_python_available();

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
        // Subprocess unknown fn errors.
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
        // let result = spanda_core::python::subprocess_unknown_fn_errors();

        if !python_available() || bridge_script_path().is_none() {
            return;
        }
        let decl = test_decl("py_missing");
        let err = call_extern(&decl, &[]).unwrap_err();
        assert!(err.to_string().contains("Unknown python extern"));
    }
}
