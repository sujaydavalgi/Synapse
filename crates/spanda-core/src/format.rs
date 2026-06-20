/// Basic source formatter for Spanda programs.
/// When parsing succeeds, applies AST-aware pretty printing; otherwise trims whitespace.
use crate::error::SpandaError;
use crate::pretty::pretty_print_program;

pub fn format_source(source: &str) -> String {
    match format_ast(source) {
        Ok(formatted) => formatted,
        Err(_) => normalize_whitespace(source),
    }
}

pub fn format_ast(source: &str) -> Result<String, SpandaError> {
    let tokens = crate::lexer::tokenize(source)?;
    let program = crate::parser::parse(tokens)?;
    Ok(pretty_print_program(source, &program))
}

fn normalize_whitespace(source: &str) -> String {
    let mut out = String::new();
    for line in source.lines() {
        out.push_str(line.trim_end());
        out.push('\n');
    }
    while out.ends_with("\n\n") {
        out.pop();
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_trailing_whitespace_and_adds_final_newline() {
        let input = "robot R {  \n  actuator wheels: DifferentialDrive; \n}\n\n";
        let formatted = format_source(input);
        assert!(formatted.ends_with('\n'));
        assert!(!formatted.contains("  \n"));
    }

    #[test]
    fn ast_format_normalizes_module_function() {
        let input = "module m;\nexport fn f(x:Int)->Int{return x;}\n";
        let formatted = format_source(input);
        assert!(formatted.contains("export fn f(x: Int) -> Int"));
    }
}
