//! CLI commands for configuration resolution, validation, and reporting.

use spanda_config::{
    diff_configs, format_report_text, generate_report_bundle, load_config_value, ConfigResolver,
    SpandaManifest,
};
use std::env;
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

fn load_resolved(root: &Path, validate: bool) -> spanda_config::ResolvedSystemConfig {
    let resolver = ConfigResolver::new().with_validation(validate);
    resolver.resolve_from_dir(root).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    })
}

/// Dispatch `spanda config` subcommands.
pub fn config_dispatch(args: &[String]) {
    let sub = args.first().map(String::as_str).unwrap_or("");
    match sub {
        "resolve" => cmd_config_resolve(&args[1..]),
        "validate" => cmd_config_validate(&args[1..]),
        "graph" => cmd_config_graph(&args[1..]),
        "diff" => cmd_config_diff(&args[1..]),
        "report" => cmd_config_report(&args[1..]),
        _ => {
            eprintln!(
                "Usage:\n  \
                 spanda config resolve [--json] [--config <spanda.toml>]\n  \
                 spanda config validate [--json] [--config <spanda.toml>]\n  \
                 spanda config graph [--json] [--config <spanda.toml>]\n  \
                 spanda config diff <base.toml> <other.toml> [--json]\n  \
                 spanda config report [--json] [--config <spanda.toml>]"
            );
            process::exit(1);
        }
    }
}

fn cmd_config_resolve(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let root = project_root_from_args(args);
    let resolved = load_resolved(&root, false);
    if json {
        println!(
            "{}",
            resolved.raw_json_pretty().unwrap_or_else(|e| {
                eprintln!("{e}");
                process::exit(1);
            })
        );
    } else {
        println!("Resolved configuration for '{}'", resolved.project_name());
        println!("{}", resolved.raw_json_pretty().unwrap_or_default());
    }
}

fn cmd_config_validate(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let root = project_root_from_args(args);
    let resolved = load_resolved(&root, true);
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&resolved.validation).unwrap()
        );
    } else {
        for finding in &resolved.validation.findings {
            let tag = match finding.severity {
                spanda_config::ValidationSeverity::Error => "ERROR",
                spanda_config::ValidationSeverity::Warning => "WARN",
                spanda_config::ValidationSeverity::Info => "INFO",
            };
            let path = finding
                .path
                .as_deref()
                .map(|p| format!(" @ {p}"))
                .unwrap_or_default();
            println!("[{tag}] {}: {}{}", finding.code, finding.message, path);
        }
        println!(
            "\nValidation: {}",
            if resolved.validation.passed {
                "PASSED"
            } else {
                "FAILED"
            }
        );
    }
    if !resolved.validation.passed {
        process::exit(1);
    }
}

fn cmd_config_graph(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let root = project_root_from_args(args);
    let resolved = load_resolved(&root, false);
    if json {
        println!("{}", serde_json::to_string_pretty(&resolved.graph).unwrap());
    } else {
        println!("Configuration graph:");
        for node in &resolved.graph.nodes {
            println!("  node: {node}");
        }
        for edge in &resolved.graph.edges {
            println!("  edge: {} -> {} ({})", edge.from, edge.to, edge.layer_kind);
        }
        println!("\nMerge order:");
        for (i, node) in resolved.graph.merge_order.iter().enumerate() {
            println!("  {}. {node}", i + 1);
        }
    }
}

fn cmd_config_diff(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let paths: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    if paths.len() < 2 {
        eprintln!("Usage: spanda config diff <base.toml> <other.toml> [--json]");
        process::exit(1);
    }
    let left = load_config_value(Path::new(paths[0])).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    });
    let right = load_config_value(Path::new(paths[1])).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    });
    let lines = diff_configs(&left, &right);
    if json {
        println!("{}", serde_json::to_string_pretty(&lines).unwrap());
    } else {
        if lines.is_empty() {
            println!("No differences.");
        } else {
            for line in lines {
                println!("{line}");
            }
        }
    }
}

fn cmd_config_report(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let root = project_root_from_args(args);
    let resolved = load_resolved(&root, true);
    let bundle = generate_report_bundle(&resolved);
    if json {
        println!("{}", serde_json::to_string_pretty(&bundle).unwrap());
    } else {
        println!("{}", format_report_text(&bundle));
    }
}
