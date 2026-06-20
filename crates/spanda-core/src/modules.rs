//! Multi-file module linking and export registry.

use crate::ast::Program;
use crate::error::{Diagnostic, SpandaError};
use crate::foundations::{ModuleFnDecl, Visibility};
use std::collections::HashMap;
use std::path::Path;

/// Exported symbols from a single module.
#[derive(Debug, Clone, Default)]
pub struct ModuleExports {
    pub functions: HashMap<String, ModuleFnDecl>,
}

/// Registry of parsed modules keyed by fully-qualified module name.
#[derive(Debug, Clone, Default)]
pub struct ModuleRegistry {
    modules: HashMap<String, ModuleExports>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, module_name: &str, program: &Program) {
        let Program::Program { functions, .. } = program;
        let mut exports = ModuleExports::default();
        for func in functions {
            let ModuleFnDecl {
                name, visibility, ..
            } = func;
            if matches!(visibility, Visibility::Export | Visibility::Public) {
                exports.functions.insert(name.clone(), func.clone());
            }
        }
        self.modules.insert(module_name.to_string(), exports);
    }

    pub fn exports_for(&self, import_path: &str) -> Option<&ModuleExports> {
        self.modules.get(import_path)
    }

    pub fn function(&self, import_path: &str, name: &str) -> Option<&ModuleFnDecl> {
        self.exports_for(import_path)
            .and_then(|e| e.functions.get(name))
    }

    /// Build a registry from parsed programs. Each entry is `(module_name, program)`.
    pub fn from_programs(entries: &[(String, Program)]) -> Self {
        let mut registry = Self::new();
        for (name, program) in entries {
            registry.register(name, program);
        }
        registry
    }

    pub fn module_count(&self) -> usize {
        self.modules.len()
    }
}

/// Parse all `.sd` files under `src/` and `tests/`, building a module registry.
pub fn load_project_modules(project_root: &Path) -> Result<ModuleRegistry, SpandaError> {
    let mut entries = Vec::new();
    for sub in ["src", "tests"] {
        let dir = project_root.join(sub);
        if dir.is_dir() {
            collect_modules(&dir, &mut entries)?;
        }
    }
    if entries.is_empty() {
        return Ok(ModuleRegistry::new());
    }
    Ok(ModuleRegistry::from_programs(&entries))
}

fn collect_modules(dir: &Path, out: &mut Vec<(String, Program)>) -> Result<(), SpandaError> {
    for entry in std::fs::read_dir(dir).map_err(|e| SpandaError::Runtime {
        message: e.to_string(),
        line: 0,
    })? {
        let entry = entry.map_err(|e| SpandaError::Runtime {
            message: e.to_string(),
            line: 0,
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_modules(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "sd") {
            let source = std::fs::read_to_string(&path).map_err(|e| SpandaError::Runtime {
                message: format!("{}: {e}", path.display()),
                line: 0,
            })?;
            let tokens = crate::lexer::tokenize(&source)?;
            let program = crate::parser::parse(tokens)?;
            let Program::Program { module_name, .. } = &program;
            let name = module_name.clone().ok_or_else(|| SpandaError::TypeCheck {
                diagnostics: vec![Diagnostic {
                    message: format!(
                        "Module file '{}' must declare `module <name>;`",
                        path.display()
                    ),
                    line: 1,
                    column: 1,
                }],
            })?;
            out.push((name, program));
        }
    }
    Ok(())
}

/// Infer module name from file path when `module` declaration is absent (single-file mode).
pub fn module_name_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main")
        .replace('-', "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Span, SpandaType, Stmt};
    use crate::foundations::{ModuleFnDecl, Visibility};

    fn empty_span() -> Span {
        Span {
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
        }
    }

    fn sample_program(functions: Vec<ModuleFnDecl>) -> Program {
        Program::Program {
            module_name: Some("navigation.path_planning".into()),
            imports: vec![],
            functions,
            tests: vec![],
            structs: vec![],
            enums: vec![],
            traits: vec![],
            hardware_profiles: vec![],
            deployments: vec![],
            requires_hardware: None,
            requires_network: None,
            simulate_compatibility: None,
            messages: vec![],
            robots: vec![],
            span: empty_span(),
        }
    }

    #[test]
    fn registry_exports_public_functions_only() {
        let funcs = vec![
            ModuleFnDecl {
                name: "plan_path".into(),
                visibility: Visibility::Export,
                type_params: vec![],
                params: vec![],
                return_type: SpandaType::Named {
                    name: "Path".into(),
                },
                is_async: false,
                body: vec![Stmt::ReturnStmt {
                    value: None,
                    span: empty_span(),
                }],
                span: empty_span(),
            },
            ModuleFnDecl {
                name: "helper".into(),
                visibility: Visibility::Private,
                type_params: vec![],
                params: vec![],
                return_type: SpandaType::Void,
                is_async: false,
                body: vec![],
                span: empty_span(),
            },
        ];
        let registry = ModuleRegistry::from_programs(&[(
            "navigation.path_planning".into(),
            sample_program(funcs),
        )]);
        let exports = registry.exports_for("navigation.path_planning").unwrap();
        assert!(exports.functions.contains_key("plan_path"));
        assert!(!exports.functions.contains_key("helper"));
    }
}
