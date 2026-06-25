//! CLI for guardrailed Spanda source generation and suggestions.
//!
use spanda_generate::{
    format_generation_report, format_suggest_report, generate_health_policy, generate_mission_program,
    generate_robot_program, suggest_program, GenerateBackend, GenerateOptions,
};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use std::fs;
use std::path::Path;
use std::process;

fn parse_program(path: &Path) -> spanda_ast::nodes::Program {
    let source = fs::read_to_string(path).unwrap_or_else(|error| {
        eprintln!("Failed to read {}: {error}", path.display());
        process::exit(1);
    });
    let tokens = tokenize(&source).unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(1);
    });
    parse(tokens).unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(1);
    })
}

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == flag)
        .map(|window| window[1].clone())
}

fn json_flag(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--json")
}

fn build_options(args: &[String]) -> GenerateOptions {
    let mut options = GenerateOptions::default();
    if let Some(name) = flag_value(args, "--robot") {
        options.robot_name = name;
    }
    if let Some(name) = flag_value(args, "--hardware") {
        options.hardware_name = name;
    }
    if let Some(name) = flag_value(args, "--mission") {
        options.mission_name = name;
    }
    if let Some(name) = flag_value(args, "--behavior") {
        options.behavior_name = name;
    }
    if let Some(name) = flag_value(args, "--health-policy") {
        options.health_policy_name = name;
    }
    if let Some(name) = flag_value(args, "--health-check") {
        options.health_check_name = name;
    }
    if let Some(backend) = flag_value(args, "--backend") {
        options.backend = if backend.eq_ignore_ascii_case("llm") {
            GenerateBackend::Llm
        } else {
            GenerateBackend::Template
        };
    }
    options
}

fn maybe_write_out(args: &[String], source: &str) {
    if let Some(path) = flag_value(args, "--out") {
        fs::write(&path, source).unwrap_or_else(|error| {
            eprintln!("Failed to write {path}: {error}");
            process::exit(1);
        });
    }
}

fn emit_generation(args: &[String], report: spanda_generate::GenerationReport) {
    println!("{}", format_generation_report(&report, json_flag(args)));
    maybe_write_out(args, &report.source);
    if !report.validated {
        process::exit(1);
    }
}

/// `spanda generate mission|robot|health-policy [options] [--json] [--out <file.sd>]`
pub fn generate_dispatch(args: &[String]) {
    let kind = args.first().map(String::as_str).unwrap_or("");
    let options = build_options(args);
    match kind {
        "mission" => emit_generation(args, generate_mission_program(&options)),
        "robot" => emit_generation(args, generate_robot_program(&options)),
        "health-policy" => emit_generation(args, generate_health_policy(&options)),
        _ => {
            eprintln!(
                "Usage:\n  spanda generate mission|robot|health-policy [--robot <name>] [--hardware <name>] [--mission <name>] [--backend template|llm] [--json] [--out <file.sd>]"
            );
            process::exit(1);
        }
    }
}

/// `spanda suggest <file.sd> [--json]`
pub fn suggest_dispatch(args: &[String]) {
    let file = args
        .iter()
        .find(|arg| !arg.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Usage: spanda suggest <file.sd> [--json]");
            process::exit(1);
        });
    let path = Path::new(&file);
    let program = parse_program(path);
    let report = suggest_program(&program, &file);
    println!("{}", format_suggest_report(&report, json_flag(args)));
    if !report.passed {
        process::exit(1);
    }
}
