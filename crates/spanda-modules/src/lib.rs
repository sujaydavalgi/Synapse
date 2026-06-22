//! Multi-file module linking and export registry.
//! and export registry.
//!
pub use spanda_typecheck::{ModuleExports, ModuleRegistry};

use spanda_ast::nodes::Program;
use spanda_error::{Diagnostic, SpandaError};
use std::path::Path;
pub fn load_project_modules(project_root: &Path) -> Result<ModuleRegistry, SpandaError> {
    // Load project modules.
    //
    // Parameters:
    // - `project_root` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::modules::load_project_modules(project_root);

    // Create mutable entries for accumulating results.
    let mut entries = Vec::new();

    // Iterate over ["src", "tests"].
    for sub in ["src", "tests"] {
        let dir = project_root.join(sub);

        // Treat the path as a directory and scan its contents.
        if dir.is_dir() {
            collect_modules(&dir, &mut entries)?;
        }
    }
    let vendor_root = project_root.join(".spanda/packages");

    // Treat the path as a directory and scan its contents.
    if vendor_root.is_dir() {
        // Process each registry entry.
        for entry in std::fs::read_dir(&vendor_root).map_err(|e| SpandaError::Runtime {
            message: e.to_string(),
            line: 0,
        })? {
            let entry = entry.map_err(|e| SpandaError::Runtime {
                message: e.to_string(),
                line: 0,
            })?;
            let path = entry.path();

            // Treat the path as a directory and scan its contents.
            if path.is_dir() {
                let src = path.join("src");

                // Treat the path as a directory and scan its contents.
                if src.is_dir() {
                    collect_modules(&src, &mut entries)?;
                }
            }
        }
    }

    // Skip further work when entries is empty.
    if entries.is_empty() {
        return Ok(ModuleRegistry::new());
    }
    Ok(ModuleRegistry::from_programs(&entries))
}

fn collect_modules(dir: &Path, out: &mut Vec<(String, Program)>) -> Result<(), SpandaError> {
    // Collect modules.
    //
    // Parameters:
    // - `dir` — input value
    // - `out` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::modules::collect_modules(dir, out);

    // Process each registry entry.
    for entry in std::fs::read_dir(dir).map_err(|e| SpandaError::Runtime {
        message: e.to_string(),
        line: 0,
    })? {
        let entry = entry.map_err(|e| SpandaError::Runtime {
            message: e.to_string(),
            line: 0,
        })?;
        let path = entry.path();

        // Treat the path as a directory and scan its contents.
        if path.is_dir() {
            collect_modules(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "sd") {
            // Skip macOS AppleDouble resource-fork sidecar files.
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("._"))
            {
                continue;
            }
            let source = std::fs::read_to_string(&path).map_err(|e| SpandaError::Runtime {
                message: format!("{}: {e}", path.display()),
                line: 0,
            })?;
            let tokens = spanda_lexer::tokenize(&source)?;
            let program = spanda_parser::parse(tokens)?;
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
    // Module name from path.
    //
    // Parameters:
    // - `path` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::modules::module_name_from_path(path);

    // Produce file stem as the result.
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main")
        .replace('-', "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_ast::foundations::{ModuleFnDecl, Visibility};
    use spanda_ast::nodes::{Span, SpandaType, Stmt};

    fn empty_span() -> Span {
        // Empty span.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Span.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::modules::empty_span();

        // Produce Span as the result.
        Span {
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
        }
    }

    fn sample_program(functions: Vec<ModuleFnDecl>) -> Program {
        // Sample program.
        //
        // Parameters:
        // - `functions` — input value
        //
        // Returns:
        // Program.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::modules::sample_program(functions);

        // Produce Program as the result.
        Program::Program {
            module_name: Some("navigation.path_planning".into()),
            imports: vec![],
            functions,
            tests: vec![],
            extern_functions: vec![],
            structs: vec![],
            enums: vec![],
            traits: vec![],
            hardware_profiles: vec![],
            deployments: vec![],
            requires_hardware: None,
            requires_network: None,
            requires_connectivity: None,
            geofences: vec![],
            fleets: vec![],
            swarms: vec![],
            program_safety_zones: vec![],
            certifications: vec![],
            connectivity_policies: vec![],
            ble_services: vec![],
            simulate_compatibility: None,
            messages: vec![],
            validate_rules: vec![],
            kill_switches: vec![],
            health_checks: vec![],
            health_policies: vec![],
            requires_capabilities: vec![],
            robots: vec![],
            span: empty_span(),
        }
    }

    #[test]
    fn registry_exports_public_functions_only() {
        // Registry exports public functions only.
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
        // let result = spanda_core::modules::registry_exports_public_functions_only();

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
