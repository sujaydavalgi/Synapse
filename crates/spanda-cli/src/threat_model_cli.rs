//! CLI for static threat modeling.
//!
use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_threat::{analyze_threat_model, format_threat_report};
use std::fs;
use std::path::Path;
use std::process;

fn parse_program(path: &Path) -> spanda_ast::nodes::Program {
    let source = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {e}", path.display());
        process::exit(1);
    });
    let tokens = tokenize(&source).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    });
    parse(tokens).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    })
}

fn file_arg(args: &[String]) -> String {
    args.iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Usage: spanda threat-model <file.sd> [--json]");
            process::exit(1);
        })
}

/// `spanda threat-model <file.sd> [--json]`
pub fn threat_model_dispatch(args: &[String]) {
    let file = file_arg(args);
    let path = Path::new(&file);
    let program = parse_program(path);
    let json = args.iter().any(|a| a == "--json");
    let report = analyze_threat_model(&program, &file);
    println!("{}", format_threat_report(&report, json));
    if !report.passed {
        process::exit(1);
    }
}
