//! Compile pipeline entry points for Spanda source programs.
//!
use spanda_ast::nodes::Program;
use spanda_error::SpandaError;
use spanda_lexer::Token;
use spanda_parser::parse;
use spanda_runtime_host::core_type_check_host;
use spanda_typecheck::{self, ModuleRegistry, TypeCheckError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileResult {
    pub program: Program,
    pub source: String,
}

/// Tokenize Spanda source (maps lexer diagnostics to `SpandaError`).
pub fn tokenize(source: &str) -> Result<Vec<Token>, SpandaError> {
    // Tokenize source text for tooling that needs tokens without full compile.
    //
    // Parameters:
    // - `source` — full `.sd` source text
    //
    // Returns:
    // Token vector, or a lexer diagnostic error.
    //
    // Options:
    // None.
    //
    // Example:
    // let tokens = tokenize(source)?;

    tokenize_source(source)
}

fn tokenize_source(source: &str) -> Result<Vec<Token>, SpandaError> {
    spanda_lexer::tokenize(source).map_err(SpandaError::from)
}

pub fn compile(source: &str) -> Result<CompileResult, SpandaError> {
    // Tokenize, parse, and type-check a Spanda source program.
    //
    // Parameters:
    // - `source` — full `.sd` source text
    //
    // Returns:
    // Compiled program and source snapshot, or a lexer/parser/type error.
    //
    // Options:
    // None.
    //
    // Example:
    // let compiled = compile(source)?;

    let tokens = tokenize_source(source)?;
    let program = parse(tokens)?;
    spanda_typecheck::type_check(&program, core_type_check_host()).map_err(type_check_error)?;
    Ok(CompileResult {
        program,
        source: source.to_string(),
    })
}

pub fn check(source: &str) -> Result<(), SpandaError> {
    // Type-check source without retaining the parsed program.
    //
    // Parameters:
    // - `source` — full `.sd` source text
    //
    // Returns:
    // Unit on success, or a compile diagnostic error.
    //
    // Options:
    // None.
    //
    // Example:
    // check(source)?;

    let tokens = tokenize_source(source)?;
    let program = parse(tokens)?;
    spanda_typecheck::check(&program, core_type_check_host()).map_err(type_check_error)
}

pub fn check_with_registry(source: &str, registry: &ModuleRegistry) -> Result<(), SpandaError> {
    // Type-check source with a project module registry for import resolution.
    //
    // Parameters:
    // - `source` — full `.sd` source text
    // - `registry` — loaded project modules
    //
    // Returns:
    // Unit on success, or a compile diagnostic error.
    //
    // Options:
    // None.
    //
    // Example:
    // check_with_registry(source, &registry)?;

    let tokens = tokenize_source(source)?;
    let program = parse(tokens)?;
    spanda_typecheck::check_with_registry(&program, registry, core_type_check_host())
        .map_err(type_check_error)
}

pub fn compile_with_registry(
    source: &str,
    registry: &ModuleRegistry,
) -> Result<CompileResult, SpandaError> {
    // Compile source with a project module registry for import resolution.
    //
    // Parameters:
    // - `source` — full `.sd` source text
    // - `registry` — loaded project modules
    //
    // Returns:
    // Compiled program and source snapshot, or a compile error.
    //
    // Options:
    // None.
    //
    // Example:
    // let compiled = compile_with_registry(source, &registry)?;

    let tokens = tokenize_source(source)?;
    let program = parse(tokens)?;
    spanda_typecheck::check_with_registry(&program, registry, core_type_check_host())
        .map_err(type_check_error)?;
    Ok(CompileResult {
        program,
        source: source.to_string(),
    })
}

fn type_check_error(err: TypeCheckError) -> SpandaError {
    SpandaError::TypeCheck {
        diagnostics: err.diagnostics,
    }
}
