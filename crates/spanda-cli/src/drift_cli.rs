//! CLI for configuration drift detection.
//!
use spanda_config::{append_program_drift, detect_config_drift, ConfigResolver, SpandaManifest};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

fn project_root_from_args(args: &[String]) -> PathBuf {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--config" {
            if let Some(path) = args.get(i + 1) {
                let p = PathBuf::from(path);
                return p.parent().unwrap_or(&p).to_path_buf();
            }
        }
    }
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    SpandaManifest::find_project_root(&cwd).unwrap_or(cwd)
}

fn root_from_flag(args: &[String], flag: &str) -> Option<PathBuf> {
    for (i, arg) in args.iter().enumerate() {
        if arg == flag {
            let path = args.get(i + 1)?;
            let p = PathBuf::from(path);
            if p.is_dir() {
                return Some(p);
            }
            return Some(p.parent().unwrap_or(&p).to_path_buf());
        }
    }
    None
}

fn load_resolved(root: &Path) -> spanda_config::ResolvedSystemConfig {
    ConfigResolver::new()
        .resolve_from_dir(root)
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            process::exit(1);
        })
}

fn parse_program_file(path: &Path) -> spanda_ast::nodes::Program {
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

/// Dispatch `spanda drift` and `spanda config drift`.
pub fn drift_dispatch(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let baseline_root = root_from_flag(args, "--baseline").unwrap_or_else(|| {
        eprintln!(
            "Usage:\n  \
             spanda drift --baseline <dir|spanda.toml> [--config <spanda.toml>] [program.sd] [--json]\n  \
             spanda config drift --baseline <dir|spanda.toml> [--config <spanda.toml>] [program.sd] [--json]"
        );
        process::exit(1);
    });
    let current_root = root_from_flag(args, "--config").unwrap_or_else(|| project_root_from_args(args));
    let program_path = positional_program_path(args);

    let baseline = load_resolved(&baseline_root);
    let current = load_resolved(&current_root);
    let mut report = detect_config_drift(&baseline, &current);
    if let Some(path) = program_path.as_deref() {
        let program = parse_program_file(path);
        append_program_drift(&mut report, &program, &current);
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        if report.findings.is_empty() {
            println!(
                "No configuration drift between '{}' and '{}'.",
                report.baseline_project, report.current_project
            );
        } else {
            println!(
                "Configuration drift: {} -> {}",
                report.baseline_project, report.current_project
            );
            for finding in &report.findings {
                let path = finding
                    .path
                    .as_deref()
                    .map(|p| format!(" @ {p}"))
                    .unwrap_or_default();
                println!(
                    "[{:?}/{:?}] {}{}",
                    finding.dimension, finding.severity, finding.message, path
                );
            }
            println!(
                "\nDrift check: {}",
                if report.passed { "PASSED" } else { "FAILED" }
            );
        }
    }
    if !report.passed {
        process::exit(1);
    }
}

fn positional_program_path(args: &[String]) -> Option<PathBuf> {
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--baseline" | "--config" => {
                i += 2;
            }
            "--json" => {
                i += 1;
            }
            other if !other.starts_with('-') => {
                let path = PathBuf::from(other);
                if path.extension().is_some_and(|ext| ext == "sd") {
                    return Some(path);
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    None
}
