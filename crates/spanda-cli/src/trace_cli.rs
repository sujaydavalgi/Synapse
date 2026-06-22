//! CLI commands for traceability, capabilities, and health reporting.

use spanda_capability::{
    capability_traceability, check_minimum_capabilities, evaluate_health_checks,
    hardware_traceability, health_traceability, infer_robot_capabilities,
};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use std::fs;
use std::process;

pub fn cmd_trace(sub: &str, args: &[String]) {
    let (json, file) = parse_file_and_json(args);
    let file = file.unwrap_or_else(|| {
        eprintln!("Usage: spanda trace {sub} <file.sd> [--json]");
        process::exit(1);
    });
    let source = read_file(&file);

    let program = parse_program(&source);

    match sub {
        "hardware" => {
            let report = hardware_traceability(&program);
            if json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                print_hardware_trace(&report);
            }
            if !report.errors.is_empty() {
                process::exit(1);
            }
        }
        "capabilities" => {
            let report = capability_traceability(&program);
            if json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                print_capability_trace(&report);
            }
            if !report.errors.is_empty() {
                process::exit(1);
            }
        }
        "health" => {
            let rows = health_traceability(&program);
            if json {
                println!("{}", serde_json::to_string_pretty(&rows).unwrap());
            } else {
                for row in &rows {
                    println!(
                        "{} | {} | {} | {} | {} | {}",
                        row.component,
                        row.health_check,
                        row.metric,
                        row.threshold,
                        row.status,
                        row.action.as_deref().unwrap_or("-")
                    );
                }
            }
        }
        other => {
            eprintln!("Unknown trace subcommand: {other}");
            eprintln!("Usage: spanda trace {{hardware|capabilities|health}} <file.sd> [--json]");
            process::exit(1);
        }
    }
}

pub fn cmd_health(sub: &str, args: &[String]) {
    let (json, file) = parse_file_and_json(args);
    match sub {
        "robot" | "report" => {
            let file = file.unwrap_or_else(|| {
                eprintln!("Usage: spanda health {sub} <file.sd> [--json]");
                process::exit(1);
            });
            let source = read_file(&file);
            let program = parse_program(&source);
            let report = evaluate_health_checks(&program);
            if json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                println!("Overall: {:?}", report.overall);
                for check in &report.checks {
                    println!(
                        "  {} ({}): {} {} {}",
                        check.name, check.target, check.metric, check.operator, check.threshold
                    );
                }
                for policy in &report.policies {
                    println!("  policy: {policy}");
                }
            }
        }
        other => {
            eprintln!("Unknown health subcommand: {other}");
            eprintln!("Usage: spanda health {{robot|report}} <file.sd> [--json]");
            process::exit(1);
        }
    }
}

pub fn cmd_hardware_capabilities(args: &[String]) {
    let (json, file) = parse_file_and_json(args);
    let file = file.unwrap_or_else(|| {
        eprintln!("Usage: spanda hardware capabilities <file.sd> [--json]");
        process::exit(1);
    });
    let source = read_file(&file);
    let program = parse_program(&source);
    let report = hardware_traceability(&program);
    if json {
        println!("{}", serde_json::to_string_pretty(&report.hardware_rows).unwrap());
    } else {
        print_hardware_trace(&report);
    }
}

pub fn cmd_robot_capabilities(args: &[String]) {
    let (json, file) = parse_file_and_json(args);
    let file = file.unwrap_or_else(|| {
        eprintln!("Usage: spanda robot capabilities <file.sd> [--json]");
        process::exit(1);
    });
    let source = read_file(&file);
    let program = parse_program(&source);
    let reports = infer_robot_capabilities(&program);
    if json {
        println!("{}", serde_json::to_string_pretty(&reports).unwrap());
    } else {
        for report in &reports {
            println!("Robot: {}", report.robot);
            for row in &report.rows {
                println!(
                    "  {} | {} | {} | {} | {}",
                    row.capability,
                    row.source,
                    row.required_components.join("+"),
                    row.status,
                    row.notes.as_deref().unwrap_or("")
                );
            }
        }
    }
}

pub fn cmd_safety_check(args: &[String]) {
    let (json, file) = parse_file_and_json(args);
    let capabilities = args.iter().any(|a| a == "--capabilities");
    let file = file.unwrap_or_else(|| {
        eprintln!("Usage: spanda safety check <file.sd> [--capabilities] [--json]");
        process::exit(1);
    });
    let source = read_file(&file);
    let program = parse_program(&source);
    if capabilities {
        let report = check_minimum_capabilities(&program);
        if json {
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        } else {
            for row in &report.rows {
                println!(
                    "{} | {} | {} | missing: {}",
                    row.capability,
                    row.required_by,
                    row.status,
                    row.missing.join(", ")
                );
            }
            for err in &report.errors {
                eprintln!("ERROR: {err}");
            }
        }
        if !report.compatible {
            process::exit(1);
        }
    } else {
        eprintln!("Usage: spanda safety check <file.sd> --capabilities [--json]");
        process::exit(1);
    }
}

pub fn verify_extensions(source: &str, traceability: bool, capabilities: bool, health: bool, minimum: bool, json: bool) {
    let program = parse_program(source);
    let mut failed = false;

    if traceability {
        let report = capability_traceability(&program);
        if json {
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        } else {
            print_hardware_trace(&report);
            print_capability_trace(&report);
        }
        if !report.errors.is_empty() {
            failed = true;
        }
    }

    if capabilities {
        let reports = infer_robot_capabilities(&program);
        if json {
            println!("{}", serde_json::to_string_pretty(&reports).unwrap());
        } else {
            for report in &reports {
                println!("Robot {} capabilities:", report.robot);
                for row in &report.rows {
                    println!("  {} [{}]", row.capability, row.status);
                }
            }
        }
    }

    if health {
        let report = evaluate_health_checks(&program);
        if json {
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        } else {
            println!("Health checks: {}", report.checks.len());
        }
    }

    if minimum {
        let report = check_minimum_capabilities(&program);
        if json {
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
        } else {
            for err in &report.errors {
                eprintln!("ERROR: {err}");
            }
        }
        if !report.compatible {
            failed = true;
        }
    }

    if failed {
        process::exit(1);
    }
}

fn print_hardware_trace(report: &spanda_capability::TraceabilityReport) {
    if report.hardware_rows.is_empty() {
        println!("No hardware traceability rows.");
        return;
    }
    println!(
        "Hardware Component | Used By | Source | Capability | Provider | Verified | Safety Rule"
    );
    for row in &report.hardware_rows {
        println!(
            "{} | {} | {} | {} | {} | {} | {}",
            row.hardware_component,
            if row.used_by.is_empty() { "-" } else { &row.used_by },
            row.source_location,
            row.capability,
            row.provider,
            row.verified,
            row.safety_rule.as_deref().unwrap_or("-")
        );
    }
    for w in &report.warnings {
        eprintln!("WARN: {w}");
    }
    for e in &report.errors {
        eprintln!("ERROR: {e}");
    }
}

fn print_capability_trace(report: &spanda_capability::TraceabilityReport) {
    if report.capability_rows.is_empty() {
        return;
    }
    println!(
        "\nCapability | Required By | Provided By | Hardware | Package | Provider | Safety Rule | Status"
    );
    for row in &report.capability_rows {
        println!(
            "{} | {} | {} | {} | {} | {} | {} | {}",
            row.capability,
            row.required_by,
            row.provided_by,
            row.hardware,
            row.package,
            row.provider,
            row.safety_rule.as_deref().unwrap_or("-"),
            row.status
        );
    }
}

fn parse_program(source: &str) -> spanda_ast::nodes::Program {
    let tokens = tokenize(source).unwrap_or_else(|e| {
        eprintln!("Lexer error: {e}");
        process::exit(1);
    });
    parse(tokens).unwrap_or_else(|e| {
        eprintln!("Parse error: {e}");
        process::exit(1);
    })
}

fn parse_file_and_json(args: &[String]) -> (bool, Option<String>) {
    let mut json = false;
    let mut file = None;
    for arg in args {
        if arg == "--json" {
            json = true;
        } else if !arg.starts_with('-') {
            file = Some(arg.clone());
        }
    }
    (json, file)
}

fn read_file(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading {path}: {e}");
        process::exit(1);
    })
}
