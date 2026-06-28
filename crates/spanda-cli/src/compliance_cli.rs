//! CLI for compliance profile accreditation export bundles.
//!
use spanda_compliance::{
    format_accreditation_report, generate_accreditation_report, list_compliance_profiles,
    load_signed_profile_catalog,
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

fn print_usage() {
    eprintln!(
        "Usage:\n\
           spanda compliance list [--json]\n\
           spanda compliance report <file.sd> --profile <name> [--json] [--format markdown]"
    );
}

/// Dispatch `spanda compliance` subcommands (`list`, `report`).
pub fn compliance_dispatch(args: &[String]) {
    // Route compliance list and report subcommands.
    //
    // Parameters:
    // - `args` — CLI arguments after `spanda compliance`
    //
    // Returns:
    // None (prints output and may exit non-zero on failure).
    //
    // Options:
    // None.
    //
    // Example:
    // compliance_cli::compliance_dispatch(&args[2..]);

    match args.first().map(String::as_str) {
        Some("list") => cmd_list(&args[1..]),
        Some("report") => cmd_report(&args[1..]),
        Some("--help") | Some("-h") | None => print_usage(),
        Some(other) => {
            eprintln!("Unknown compliance subcommand '{other}'");
            print_usage();
            process::exit(1);
        }
    }
}

fn cmd_list(args: &[String]) {
    let json = args.iter().any(|arg| arg == "--json");
    let builtins = list_compliance_profiles();
    let signed = load_signed_profile_catalog().unwrap_or_default();

    if json {
        let payload = serde_json::json!({
            "profiles": builtins,
            "signed_catalog": signed.iter().map(|entry| serde_json::json!({
                "name": entry.name,
                "version": entry.version,
                "verified": entry.verified,
                "description": entry.profile.description,
                "content_sha256": entry.content_sha256,
            })).collect::<Vec<_>>(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
        return;
    }

    println!("Built-in compliance profiles:");
    for name in &builtins {
        println!("  {name}");
    }
    if signed.is_empty() {
        println!("\nSigned catalog: unavailable");
        return;
    }
    println!("\nSigned catalog:");
    for entry in signed {
        let mark = if entry.verified {
            "verified"
        } else {
            "UNVERIFIED"
        };
        println!(
            "  {} ({}) — {}",
            entry.name, mark, entry.profile.description
        );
    }
}

fn cmd_report(args: &[String]) {
    let file = args
        .iter()
        .find(|arg| !arg.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Missing file path");
            process::exit(1);
        });

    let profile = args
        .windows(2)
        .find_map(|window| {
            if window[0] == "--profile" {
                Some(window[1].clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            eprintln!("--profile requires a compliance profile name");
            process::exit(1);
        });

    let json = args.iter().any(|arg| arg == "--json");
    let markdown = args
        .windows(2)
        .any(|window| window[0] == "--format" && window[1] == "markdown");

    let program = parse_program(Path::new(&file));
    match generate_accreditation_report(&program, &profile, &file) {
        Ok(report) => {
            println!(
                "{}",
                format_accreditation_report(&report, json && !markdown)
            );
            if !report.passed {
                process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}
