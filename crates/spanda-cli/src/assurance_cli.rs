//! CLI commands for mission assurance and autonomous operations.

use spanda_assurance::{
    assure_program, check_resilience, diagnose_from_trace, diagnose_program, evaluate_prognostics,
    evaluate_state_assurance, format_anomaly, format_assurance, format_diagnosis,
    format_mission_assurance, format_prognostics, format_resilience, format_state, scan_anomalies,
    verify_mission_assurance,
};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_readiness::ReportFormat;
use std::fs;
use std::path::Path;
use std::process;

fn read_file(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Failed to read {path}: {e}");
        process::exit(1);
    })
}

fn parse_program(source: &str) -> spanda_ast::nodes::Program {
    let tokens = tokenize(source).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    });
    parse(tokens).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    })
}

fn parse_format(args: &[String]) -> ReportFormat {
    if args.iter().any(|a| a == "--json") {
        ReportFormat::Json
    } else if args.iter().any(|a| a == "--markdown") {
        ReportFormat::Markdown
    } else if args.iter().any(|a| a == "--html") {
        ReportFormat::Html
    } else {
        ReportFormat::Text
    }
}

fn file_arg(args: &[String]) -> String {
    args.iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Missing file path");
            process::exit(1);
        })
}

/// `spanda assure <file.sd> [--json|--markdown|--html]`
pub fn cmd_assure(args: &[String]) {
    let format = parse_format(args);
    let file = file_arg(args);
    let source = read_file(&file);
    let program = parse_program(&source);
    let summary = assure_program(&program, &file);
    let output = match format {
        ReportFormat::Json => serde_json::to_string_pretty(&summary).unwrap_or_default(),
        _ => format_assurance(&summary.assurance, format),
    };
    println!("{output}");
    if !summary.passed {
        process::exit(1);
    }
}

/// `spanda anomaly scan <file.sd> [--json|--markdown|--html]`
pub fn cmd_anomaly_scan(args: &[String]) {
    let format = parse_format(args);
    let file = file_arg(args);
    let source = read_file(&file);
    let program = parse_program(&source);
    let report = scan_anomalies(&program);
    println!("{}", format_anomaly(&report, format));
    if !report.passed {
        process::exit(1);
    }
}

/// `spanda diagnose <mission.trace|file.sd> [--json|--markdown|--html]`
pub fn cmd_diagnose_assurance(args: &[String]) {
    let format = parse_format(args);
    let file = file_arg(args);
    let report = if file.ends_with(".trace") {
        diagnose_from_trace(Path::new(&file)).unwrap_or_else(|e| {
            eprintln!("{e}");
            process::exit(1);
        })
    } else {
        let source = read_file(&file);
        let program = parse_program(&source);
        diagnose_program(&program)
    };
    println!("{}", format_diagnosis(&report, format));
    if !report.passed {
        process::exit(1);
    }
}

/// `spanda prognostics <file.sd> [--json|--markdown|--html]`
pub fn cmd_prognostics(args: &[String]) {
    let format = parse_format(args);
    let file = file_arg(args);
    let source = read_file(&file);
    let program = parse_program(&source);
    let report = evaluate_prognostics(&program);
    println!("{}", format_prognostics(&report, format));
    if !report.passed {
        process::exit(1);
    }
}

/// `spanda mission verify <file.sd> [--json|--markdown|--html]`
pub fn cmd_mission_verify(args: &[String]) {
    let format = parse_format(args);
    let file = file_arg(args);
    let source = read_file(&file);
    let program = parse_program(&source);
    let report = verify_mission_assurance(&program);
    println!("{}", format_mission_assurance(&report, format));
    if !report.passed {
        process::exit(1);
    }
}

/// `spanda resilience check <file.sd> [--json|--markdown|--html]`
pub fn cmd_resilience_check(args: &[String]) {
    let format = parse_format(args);
    let file = file_arg(args);
    let source = read_file(&file);
    let program = parse_program(&source);
    let report = check_resilience(&program);
    println!("{}", format_resilience(&report, format));
    if !report.passed {
        process::exit(1);
    }
}

/// Dispatch `spanda anomaly` subcommands.
pub fn anomaly_dispatch(args: &[String]) {
    let sub = args.first().map(String::as_str).unwrap_or("");
    match sub {
        "scan" => cmd_anomaly_scan(&args[1..]),
        _ => {
            eprintln!("Usage: spanda anomaly scan <file.sd>");
            process::exit(1);
        }
    }
}

/// Dispatch `spanda mission` subcommands.
pub fn mission_dispatch(args: &[String]) {
    let sub = args.first().map(String::as_str).unwrap_or("");
    match sub {
        "verify" => cmd_mission_verify(&args[1..]),
        _ => {
            eprintln!("Usage: spanda mission verify <file.sd>");
            process::exit(1);
        }
    }
}

/// Dispatch `spanda resilience` subcommands.
pub fn resilience_dispatch(args: &[String]) {
    let sub = args.first().map(String::as_str).unwrap_or("");
    match sub {
        "check" => cmd_resilience_check(&args[1..]),
        _ => {
            eprintln!("Usage: spanda resilience check <file.sd>");
            process::exit(1);
        }
    }
}

/// `spanda mitigation plan <file.sd> [--json|--markdown|--html]`
pub fn cmd_mitigation_plan(args: &[String]) {
    let format = parse_format(args);
    let file = file_arg(args);
    let source = read_file(&file);
    let program = parse_program(&source);
    let report = spanda_assurance::mitigation_report(&program);
    println!("{}", spanda_assurance::format_mitigation(&report, format));
}

/// `spanda state estimate <file.sd> [--json|--markdown|--html]`
pub fn cmd_state_estimate(args: &[String]) {
    let format = parse_format(args);
    let file = file_arg(args);
    let source = read_file(&file);
    let program = parse_program(&source);
    let report = evaluate_state_assurance(&program);
    println!("{}", format_state(&report, format));
    if !report.passed {
        process::exit(1);
    }
}

/// Dispatch `spanda state` subcommands.
pub fn state_dispatch(args: &[String]) {
    let sub = args.first().map(String::as_str).unwrap_or("");
    match sub {
        "estimate" => cmd_state_estimate(&args[1..]),
        _ => {
            eprintln!("Usage: spanda state estimate <file.sd>");
            process::exit(1);
        }
    }
}

/// Dispatch `spanda mitigation` subcommands.
pub fn mitigation_dispatch(args: &[String]) {
    let sub = args.first().map(String::as_str).unwrap_or("");
    match sub {
        "plan" => cmd_mitigation_plan(&args[1..]),
        _ => {
            eprintln!("Usage: spanda mitigation plan <file.sd>");
            process::exit(1);
        }
    }
}
