//! Program loading helpers for Control Center govern-and-trace APIs.
//!
use spanda_ast::nodes::Program;
use spanda_lexer::tokenize;
use spanda_parser::parse;
use std::path::Path;

/// Parse a Spanda program from a `.sd` file path.
pub fn parse_program_file(path: &Path) -> Result<(Program, String, String), String> {
    let source = std::fs::read_to_string(path)
        .map_err(|error| format!("read {} failed: {error}", path.display()))?;
    let label = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("program.sd")
        .to_string();
    let tokens = tokenize(&source).map_err(|error| format!("tokenize failed: {error}"))?;
    let program = parse(tokens).map_err(|error| format!("parse failed: {error}"))?;
    Ok((program, source, label))
}
