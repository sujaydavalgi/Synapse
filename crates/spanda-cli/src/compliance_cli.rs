//! CLI for compliance profile accreditation export bundles.
//!
use spanda_compliance::{format_accreditation_report, generate_accreditation_report};
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

/// `spanda compliance report <file.sd> --profile <name> [--json] [--format markdown]`
pub fn compliance_dispatch(args: &[String]) {
    // Dispatch compliance accreditation export commands.
    //
    // Parameters:
    // - `args` — CLI arguments after `spanda compliance`
    //
    // Returns:
    // None (prints report and may exit non-zero on failure).
    //
    // Options:
    // None.
    //
    // Example:
    // compliance_cli::compliance_dispatch(&args[2..]);

    if args.first().map(String::as_str) != Some("report") {
        eprintln!(
            "Usage: spanda compliance report <file.sd> --profile <name> [--json] [--format markdown]"
        );
        process::exit(1);
    }

    let file = args
        .iter()
        .skip(1)
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
            println!("{}", format_accreditation_report(&report, json && !markdown));
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
