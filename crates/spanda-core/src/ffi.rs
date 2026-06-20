use crate::ast::UnitKind;
use crate::error::SpandaError;
use crate::foundations::{BridgeKind, ExternFnDecl};
use crate::runtime::RuntimeValue;
use std::collections::HashMap;

pub type FfiHandler = fn(&[RuntimeValue]) -> Result<RuntimeValue, SpandaError>;

pub struct FfiRegistry {
    handlers: HashMap<String, FfiHandler>,
}

impl Default for FfiRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FfiRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            handlers: HashMap::new(),
        };
        registry.register("stub_echo", stub_echo);
        registry.register("stub_add", stub_add);
        registry
    }

    pub fn register(&mut self, name: &str, handler: FfiHandler) {
        self.handlers.insert(name.to_string(), handler);
    }

    pub fn has_handler(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }

    pub fn call(
        &self,
        decl: &ExternFnDecl,
        args: &[RuntimeValue],
    ) -> Result<RuntimeValue, SpandaError> {
        if matches!(decl.bridge, BridgeKind::Python | BridgeKind::Cpp)
            && !self.handlers.contains_key(&decl.name)
        {
            return Err(SpandaError::Runtime {
                message: format!(
                    "Bridge '{}' extern fn '{}' is declared but not linked — \
                     native Python/C++ shims are not yet available (see docs/ffi-and-ecosystem.md)",
                    decl.bridge.as_str(),
                    decl.name
                ),
                line: decl.span.start.line,
            });
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
    Ok(args.first().cloned().unwrap_or(RuntimeValue::Void))
}

fn stub_add(args: &[RuntimeValue]) -> Result<RuntimeValue, SpandaError> {
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
    use crate::ast::SpandaType;

    #[test]
    fn stub_add_sums_integers() {
        let registry = FfiRegistry::new();
        let decl = ExternFnDecl {
            name: "stub_add".into(),
            library: None,
            bridge: BridgeKind::Native,
            params: vec![],
            return_type: SpandaType::Int,
            span: crate::ast::Span {
                start: crate::ast::SourceLocation {
                    line: 1,
                    column: 1,
                    offset: 0,
                },
                end: crate::ast::SourceLocation {
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
        let registry = FfiRegistry::new();
        let decl = ExternFnDecl {
            name: "detect_objects".into(),
            library: Some("python".into()),
            bridge: BridgeKind::Python,
            params: vec![],
            return_type: SpandaType::Int,
            span: crate::ast::Span {
                start: crate::ast::SourceLocation {
                    line: 1,
                    column: 1,
                    offset: 0,
                },
                end: crate::ast::SourceLocation {
                    line: 1,
                    column: 1,
                    offset: 0,
                },
            },
        };
        let err = registry.call(&decl, &[]).unwrap_err();
        assert!(err.to_string().contains("not linked"));
    }
}
