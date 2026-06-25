//! CLI for verify-time and runtime tamper analysis.
//!
use spanda_fleet::fetch_live_fleet_tamper_report;
use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_tamper::{
    correlate_fleet_tamper, diagnose_tamper_trace, format_fleet_tamper_report,
    format_tamper_diagnosis, format_tamper_report, generate_runtime_tamper_check,
    generate_tamper_check, MissionTrace, TamperDiagnosisFormat, TamperFormat,
};
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

fn load_trace(path: &Path) -> MissionTrace {
    let raw = fs::read_to_string(path).unwrap_or_else(|error| {
        eprintln!("Failed to read {}: {error}", path.display());
        process::exit(1);
    });
    serde_json::from_str(&raw).unwrap_or_else(|error| {
        eprintln!("Failed to parse trace {}: {error}", path.display());
        process::exit(1);
    })
}

fn fleet_manifest_arg(args: &[String]) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == "--fleet")
        .map(|window| window[1].clone())
}

fn mesh_url_arg(args: &[String]) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == "--mesh-url")
        .map(|window| window[1].clone())
}

fn fleet_name_arg(args: &[String]) -> String {
    args.windows(2)
        .find(|window| window[0] == "--fleet-name")
        .map(|window| window[1].clone())
        .unwrap_or_else(|| "fleet".into())
}

fn mesh_token() -> Option<String> {
    std::env::var("SPANDA_FLEET_MESH_TOKEN").ok()
}

fn json_flag(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--json")
}

fn file_arg(args: &[String]) -> String {
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" | "--runtime" => index += 1,
            "--fleet" | "--mesh-url" | "--fleet-name" => index += 2,
            other if !other.starts_with('-') => return other.to_string(),
            _ => index += 1,
        }
    }
    eprintln!(
        "Usage:\n  spanda tamper-check <file.sd|file.trace> [--runtime] [--json]\n  spanda tamper-check --fleet <manifest.json> [--json]\n  spanda tamper-check --mesh-url <url> [--fleet-name <name>] [--json]\n  spanda diagnose tamper <file.trace> [--json]\n  spanda diagnose tamper --fleet <manifest.json> [--json]\n  spanda diagnose tamper --mesh-url <url> [--fleet-name <name>] [--json]"
    );
    process::exit(1);
}

fn run_fleet_tamper_check(manifest: &str, args: &[String]) {
    let report = correlate_fleet_tamper(Path::new(manifest)).unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(1);
    });
    let format = if json_flag(args) {
        TamperFormat::Json
    } else {
        TamperFormat::Text
    };
    println!("{}", format_fleet_tamper_report(&report, format));
    if !report.passed {
        process::exit(1);
    }
}

fn run_mesh_tamper_check(mesh_url: &str, args: &[String]) {
    let fleet_name = fleet_name_arg(args);
    let output = fetch_live_fleet_tamper_report(
        mesh_url,
        &fleet_name,
        mesh_token().as_deref(),
        json_flag(args),
    )
    .unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(1);
    });
    println!("{output}");
    if !json_flag(args) && output.contains("Result: FAIL") {
        process::exit(1);
    }
    if json_flag(args) {
        let report: spanda_tamper::FleetTamperReport =
            serde_json::from_str(&output).unwrap_or_else(|error| {
                eprintln!("invalid fleet tamper JSON: {error}");
                process::exit(1);
            });
        if !report.passed {
            process::exit(1);
        }
    }
}

/// `spanda tamper-check <file.sd|file.trace> [--runtime] [--json]`
/// `spanda tamper-check --fleet <manifest.json> [--json]`
/// `spanda tamper-check --mesh-url <url> [--fleet-name <name>] [--json]`
pub fn tamper_check_dispatch(args: &[String]) {
    if let Some(mesh_url) = mesh_url_arg(args) {
        run_mesh_tamper_check(&mesh_url, args);
        return;
    }
    if let Some(manifest) = fleet_manifest_arg(args) {
        run_fleet_tamper_check(&manifest, args);
        return;
    }

    let file = file_arg(args);
    let path = Path::new(&file);
    let format = if json_flag(args) {
        TamperFormat::Json
    } else {
        TamperFormat::Text
    };
    let runtime_mode = args.iter().any(|arg| arg == "--runtime")
        || path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("trace"))
            .unwrap_or(false);

    let report = if runtime_mode {
        let trace = load_trace(path);
        generate_runtime_tamper_check(&trace, &file)
    } else {
        let program = parse_program(path);
        generate_tamper_check(&program, &file)
    };

    println!("{}", format_tamper_report(&report, format));
    if !report.passed {
        process::exit(1);
    }
}

/// `spanda diagnose tamper <file.trace> [--json]`
/// `spanda diagnose tamper --fleet <manifest.json> [--json]`
/// `spanda diagnose tamper --mesh-url <url> [--fleet-name <name>] [--json]`
pub fn tamper_diagnose_dispatch(args: &[String]) {
    if let Some(mesh_url) = mesh_url_arg(args) {
        run_mesh_tamper_check(&mesh_url, args);
        return;
    }
    if let Some(manifest) = fleet_manifest_arg(args) {
        run_fleet_tamper_check(&manifest, args);
        return;
    }

    let file = file_arg(args);
    let path = Path::new(&file);
    let trace = load_trace(path);
    let report = diagnose_tamper_trace(&trace, &file);
    let format = if json_flag(args) {
        TamperDiagnosisFormat::Json
    } else {
        TamperDiagnosisFormat::Text
    };
    println!("{}", format_tamper_diagnosis(&report, format));
    if !report.passed {
        process::exit(1);
    }
}
