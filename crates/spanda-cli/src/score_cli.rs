//! CLI for autonomous systems scorecard rollup.
//!
use crate::config_load::{ensure_config_valid, load_system_config};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_score::{evaluate_scorecard, format_scorecard, ScorecardFormat, ScorecardOptions};
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

fn parse_format(args: &[String]) -> ScorecardFormat {
    if args.iter().any(|a| a == "--json") {
        return ScorecardFormat::Json;
    }
    for (index, arg) in args.iter().enumerate() {
        if arg == "--format" {
            if let Some(value) = args.get(index + 1) {
                return match value.as_str() {
                    "markdown" | "md" => ScorecardFormat::Markdown,
                    "json" => ScorecardFormat::Json,
                    _ => ScorecardFormat::Text,
                };
            }
        }
    }
    ScorecardFormat::Text
}

fn file_arg(args: &[String]) -> String {
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--config" | "--format" => index += 2,
            "--json" => index += 1,
            other if !other.starts_with('-') => return other.to_string(),
            _ => index += 1,
        }
    }
    eprintln!(
        "Usage: spanda score <file.sd> [--json] [--format markdown] [--config <spanda.toml>]"
    );
    process::exit(1);
}

/// `spanda score <file.sd> [--json] [--format markdown] [--config <spanda.toml>]`
pub fn score_dispatch(args: &[String]) {
    let file = file_arg(args);
    let path = Path::new(&file);
    let program = parse_program(path);
    let system_config = load_system_config(
        path,
        spanda_config::config_flag_from_args(args).as_deref(),
    );
    ensure_config_valid(system_config.as_ref().map(|arc| arc.as_ref()));
    let report = evaluate_scorecard(
        &program,
        &file,
        &ScorecardOptions {
            system_config,
        },
    );
    let format = parse_format(args);
    println!("{}", format_scorecard(&report, format));
    if report.overall_score < 60 {
        process::exit(1);
    }
}
