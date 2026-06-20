//! Subprocess C++ bridge for `extern cpp fn` declarations.
//!
//! Invokes a small C++ helper binary (built via `build.rs`, or `SPANDA_CPP_BRIDGE`)
//! with the same JSON stdin/stdout protocol as the Python bridge.

use crate::error::SpandaError;
use crate::foundations::ExternFnDecl;
use crate::runtime::RuntimeValue;
use std::path::PathBuf;

use super::protocol::call_subprocess_bridge;

/// Resolve the C++ bridge executable path.
pub fn bridge_binary_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("SPANDA_CPP_BRIDGE") {
        let p = PathBuf::from(path);
        if p.is_file() {
            return Some(p);
        }
    }
    candidate_binary_paths()
        .into_iter()
        .find(|candidate| candidate.is_file())
}

fn candidate_binary_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(path) = option_env!("SPANDA_CPP_BRIDGE_BIN") {
        paths.push(PathBuf::from(path));
    }
    paths.push(PathBuf::from("scripts/spanda_cpp_bridge"));
    paths.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/spanda_cpp_bridge"));
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join("scripts/spanda_cpp_bridge"));
    }
    paths
}

pub fn bridge_available() -> bool {
    bridge_binary_path().is_some()
}

pub fn call_extern(
    decl: &ExternFnDecl,
    args: &[RuntimeValue],
) -> Result<RuntimeValue, SpandaError> {
    let line = decl.span.start.line;
    let binary = bridge_binary_path().ok_or_else(|| SpandaError::Runtime {
        message:
            "C++ bridge binary not found — set SPANDA_CPP_BRIDGE or rebuild with a C++ compiler"
                .into(),
        line,
    })?;
    call_subprocess_bridge("C++", &binary, &[], decl, args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{SourceLocation, Span, SpandaType};
    use crate::foundations::BridgeKind;

    fn test_decl(name: &str) -> ExternFnDecl {
        ExternFnDecl {
            name: name.into(),
            library: Some("cpp".into()),
            bridge: BridgeKind::Cpp,
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
    fn subprocess_cpp_add_when_binary_available() {
        if !bridge_available() {
            return;
        }
        let decl = test_decl("cpp_add");
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
        .expect("cpp_add");
        assert!(matches!(
            result,
            RuntimeValue::Number { value, .. } if (value - 9.0).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn subprocess_unknown_fn_errors() {
        if !bridge_available() {
            return;
        }
        let decl = test_decl("cpp_missing");
        let err = call_extern(&decl, &[]).unwrap_err();
        assert!(err.to_string().contains("Unknown cpp extern"));
    }
}
