//! ffi support for Spanda.
//!
use spanda_ast::nodes::UnitKind;
use spanda_error::SpandaError;
use spanda_ast::foundations::{BridgeKind, ExternFnDecl};
use spanda_runtime::value::RuntimeValue;
use std::collections::HashMap;

pub type FfiHandler = fn(&[RuntimeValue]) -> Result<RuntimeValue, SpandaError>;
pub type ExternBridgeFn =
    fn(&ExternFnDecl, &[RuntimeValue]) -> Result<RuntimeValue, SpandaError>;

#[derive(Clone, Copy, Default)]
pub struct ExternBridges {
    pub python: Option<ExternBridgeFn>,
    pub cpp: Option<ExternBridgeFn>,
}

#[derive(Clone)]
pub struct FfiRegistry {
    handlers: HashMap<String, FfiHandler>,
    bridges: ExternBridges,
}

impl Default for FfiRegistry {
    fn default() -> Self {
        //
        // Parameters:
        // None.
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_ffi::default();

        // Build the result via new.
        Self::new()
    }
}

impl FfiRegistry {
    pub fn new() -> Self {
        // Create a new instance.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_ffi::new();

        // Create mutable registry for accumulating results.
        let mut registry = Self {
            handlers: HashMap::new(),
            bridges: ExternBridges::default(),
        };
        registry.register("stub_echo", stub_echo);
        registry.register("stub_add", stub_add);
        registry
    }

    pub fn with_bridges(bridges: ExternBridges) -> Self {
        Self {
            bridges,
            ..Self::new()
        }
    }

    pub fn set_bridges(&mut self, bridges: ExternBridges) {
        self.bridges = bridges;
    }

    pub fn register(&mut self, name: &str, handler: FfiHandler) {
        // Register the value.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `handler` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.register(name, handler);

        // Append into self.
        self.handlers.insert(name.to_string(), handler);
    }

    pub fn has_handler(&self, name: &str) -> bool {
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.has_handler(name);

        // Call contains key on the current instance.
        self.handlers.contains_key(name)
    }

    pub fn call(
        &self,
        decl: &ExternFnDecl,
        args: &[RuntimeValue],
    ) -> Result<RuntimeValue, SpandaError> {
        // Call.
        //
        // Parameters:
        // - `self` — method receiver
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
        // let result = instance.call(decl, args);

        // keep entries that match the expected pattern.
        if matches!(decl.bridge, BridgeKind::Python) && !self.handlers.contains_key(&decl.name) {
            if let Some(python) = self.bridges.python {
                return python(decl, args);
            }
        }

        if matches!(decl.bridge, BridgeKind::Cpp) && !self.handlers.contains_key(&decl.name) {
            if let Some(cpp) = self.bridges.cpp {
                return cpp(decl, args);
            }
        }
        let handler = self
            .handlers
            .get(&decl.name)
            .ok_or_else(|| SpandaError::Runtime {
                message: format!(
                    "No native binding for extern fn '{}'{}",
                    decl.name,
                    decl.library
                        .as_ref()
                        .map(|l| format!(" (library \"{l}\")"))
                        .unwrap_or_default()
                ),
                line: decl.span.start.line,
            })?;
        handler(args)
    }
}

fn stub_echo(args: &[RuntimeValue]) -> Result<RuntimeValue, SpandaError> {
    // Stub echo.
    //
    // Parameters:
    // - `args` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_ffi::stub_echo(args);

    // Return the success value to the caller.
    Ok(args.first().cloned().unwrap_or(RuntimeValue::Void))
}

fn stub_add(args: &[RuntimeValue]) -> Result<RuntimeValue, SpandaError> {
    // Stub add.
    //
    // Parameters:
    // - `args` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_ffi::stub_add(args);

    // Compute a for the following logic.
    let a = match args.first() {
        Some(RuntimeValue::Number { value, .. }) => *value,
        _ => 0.0,
    };
    let b = match args.get(1) {
        Some(RuntimeValue::Number { value, .. }) => *value,
        _ => 0.0,
    };
    Ok(RuntimeValue::Number {
        value: a + b,
        unit: UnitKind::None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_ast::nodes::SpandaType;

    #[test]
    fn stub_add_sums_integers() {
        // Stub add sums integers.
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
        // let result = spanda_ffi::stub_add_sums_integers();

        let registry = FfiRegistry::new();
        let decl = ExternFnDecl {
            name: "stub_add".into(),
            library: None,
            bridge: BridgeKind::Native,
            params: vec![],
            return_type: SpandaType::Int,
            span: spanda_ast::nodes::Span {
                start: spanda_ast::nodes::SourceLocation {
                    line: 1,
                    column: 1,
                    offset: 0,
                },
                end: spanda_ast::nodes::SourceLocation {
                    line: 1,
                    column: 1,
                    offset: 0,
                },
            },
        };
        let result = registry
            .call(
                &decl,
                &[
                    RuntimeValue::Number {
                        value: 2.0,
                        unit: UnitKind::None,
                    },
                    RuntimeValue::Number {
                        value: 3.0,
                        unit: UnitKind::None,
                    },
                ],
            )
            .expect("stub_add");
        assert!(matches!(
            result,
            RuntimeValue::Number {
                value,
                unit: UnitKind::None
            } if (value - 5.0).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn python_bridge_without_handler_errors_clearly() {
        // Python bridge without handler errors clearly.
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
        // let result = spanda_ffi::python_bridge_without_handler_errors_clearly();

        let registry = FfiRegistry::new();
        let decl = ExternFnDecl {
            name: "detect_objects".into(),
            library: Some("python".into()),
            bridge: BridgeKind::Python,
            params: vec![],
            return_type: SpandaType::Int,
            span: spanda_ast::nodes::Span {
                start: spanda_ast::nodes::SourceLocation {
                    line: 1,
                    column: 1,
                    offset: 0,
                },
                end: spanda_ast::nodes::SourceLocation {
                    line: 1,
                    column: 1,
                    offset: 0,
                },
            },
        };
        // Without python/script, call goes to bridge which may error differently;
        // with script present, unknown fn errors clearly.
        let err = registry.call(&decl, &[]).unwrap_err();
        assert!(
            err.to_string().contains("Unknown python extern")
                || err.to_string().contains("not found")
                || err.to_string().contains("Python")
                || err.to_string().contains("No native binding")
        );
    }
}
