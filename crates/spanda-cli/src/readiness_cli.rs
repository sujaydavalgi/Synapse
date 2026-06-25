//! CLI commands for operational readiness and mission assurance.

use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_readiness::{
    analyze_failure, audit_program, build_runtime_context, evaluate_fleet_readiness,
    evaluate_readiness_with_runtime, evaluate_safety_coverage, evaluate_twin_readiness,
    format_audit, format_failure_analysis, format_fleet_readiness, format_mission_verification,
    format_readiness, format_safety_coverage, format_safety_report, generate_safety_report,
    readiness_options_from_flags, verify_approvals, verify_fleet, verify_mission, ReadinessOptions,
    ReportFormat,
};
use std::fs;
use std::path::Path;
use std::process;

struct ParsedReadinessCli {
    format: ReportFormat,
    file: String,
    options: ReadinessOptions,
    agent_json: bool,
    system_config: Option<spanda_config::ResolvedSystemConfig>,
}

fn read_file(path: &str) -> String {
    // Description:
    //     Read file.
    //
    // Inputs:
    //     path: &str
    //         Caller-supplied path.
    //
    // Outputs:
    //     result: String
    //         Return value from `read_file`.
    //
    // Example:

    //     let result = spanda_cli::readiness_cli::read_file(path);

    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Failed to read {path}: {e}");
        process::exit(1);
    })
}

fn parse_program(source: &str) -> spanda_ast::nodes::Program {
    // Description:
    //     Parse program.
    //
    // Inputs:
    //     source: &str
    //         Caller-supplied source.
    //
    // Outputs:
    //     result: spanda_ast::nodes::Program
    //         Return value from `parse_program`.
    //
    // Example:

    //     let result = spanda_cli::readiness_cli::parse_program(source);

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
    // Description:
    //     Parse format.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     result: ReportFormat
    //         Return value from `parse_format`.
    //
    // Example:

    //     let result = spanda_cli::readiness_cli::parse_format(args);

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

fn parse_readiness_cli(args: &[String]) -> ParsedReadinessCli {
    // Description:
    //     Parse readiness cli.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     result: ParsedReadinessCli
    //         Return value from `parse_readiness_cli`.
    //
    // Example:

    //     let result = spanda_cli::readiness_cli::parse_readiness_cli(args);

    let format = parse_format(args);
    let mut target: Option<String> = None;
    let mut include_runtime = false;
    let mut inject_health_faults = false;
    let mut simulate = false;
    let mut strict = false;
    let mut agent_json = false;
    let mut config_path: Option<String> = None;
    let mut file: Option<String> = None;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--json" | "--markdown" | "--html" | "--agent-json" => {
                if args[i].as_str() == "--agent-json" {
                    agent_json = true;
                }
            }
            "--target" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--target requires a hardware profile name");
                    process::exit(1);
                }
                target = Some(args[i].clone());
            }
            "--runtime" => include_runtime = true,
            "--inject-health-faults" => {
                include_runtime = true;
                inject_health_faults = true;
            }
            "--simulate" => simulate = true,
            "--strict" => strict = true,
            "--config" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--config requires a path to spanda.toml");
                    process::exit(1);
                }
                config_path = Some(args[i].clone());
            }
            other if !other.starts_with('-') && file.is_none() => file = Some(other.to_string()),
            other => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
        }
        i += 1;
    }
    let file = file.unwrap_or_else(|| {
        eprintln!("Missing file path");
        process::exit(1);
    });
    let source = read_file(&file);
    let program = parse_program(&source);
    let system_config = config_path.as_ref().map(|path| {
        let root = std::path::Path::new(path)
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        spanda_config::ConfigResolver::new()
            .with_validation(true)
            .resolve_from_dir(root)
            .unwrap_or_else(|e| {
                eprintln!("Failed to load config: {e}");
                process::exit(1);
            })
    });
    if target.is_none() {
        if let Some(ref cfg) = system_config {
            if let Some(robot) = cfg
                .device_tree
                .fleet
                .as_ref()
                .and_then(|f| f.robots.first())
            {
                target = robot.hardware_profile.clone();
            }
        }
    }
    let options = readiness_options_from_flags(
        &program,
        target,
        include_runtime,
        inject_health_faults,
        simulate,
        strict,
    );
    ParsedReadinessCli {
        format,
        file,
        options,
        agent_json,
        system_config,
    }
}

fn evaluate_with_options(
    program: &spanda_ast::nodes::Program,
    options: &ReadinessOptions,
) -> spanda_readiness::ReadinessReport {
    // Description:
    //     Evaluate with options.
    //
    // Inputs:
    //     progra: &spanda_ast::nodes::Program
    //         Caller-supplied progra.
    //     options: &ReadinessOptions
    //         Caller-supplied options.
    //
    // Outputs:
    //     result: spanda_readiness::ReadinessReport
    //         Return value from `evaluate_with_options`.
    //
    // Example:

    //     let result = spanda_cli::readiness_cli::evaluate_with_options(progra, options);

    let runtime = options
        .include_runtime
        .then(|| build_runtime_context(program, options.inject_health_faults));
    evaluate_readiness_with_runtime(program, options, runtime.as_ref())
}

/// `spanda readiness <file.sd> [--target T] [--runtime] [--inject-health-faults] [--json|--agent-json]`
pub fn cmd_readiness(args: &[String]) {
    // Description:
    //     Cmd readiness.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::readiness_cli::cmd_readiness(args);

    let parsed = parse_readiness_cli(args);
    if let Some(ref cfg) = parsed.system_config {
        if !cfg.validation.passed {
            eprintln!(
                "Configuration validation failed ({} errors); see `spanda config validate`",
                cfg.validation.error_count()
            );
            if parsed.format == ReportFormat::Json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&cfg.validation).unwrap_or_default()
                );
            }
            process::exit(1);
        }
    }
    let source = read_file(&parsed.file);

    // Emit the same JSON envelope as deploy/fleet `GET /v1/readiness`.
    if parsed.agent_json {
        let body = evaluate_agent_readiness_json(
            &source,
            parsed.options.target.as_deref(),
            parsed.options.include_runtime,
            parsed.options.inject_health_faults,
        )
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            process::exit(1);
        });
        println!("{body}");
        let mission_ready = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|value| value.get("mission_ready").and_then(|m| m.as_bool()))
            .unwrap_or(false);
        if !mission_ready {
            process::exit(1);
        }
        return;
    }

    let program = parse_program(&source);
    let report = evaluate_with_options(&program, &parsed.options);
    println!("{}", format_readiness(&report, parsed.format));
    if !report.mission_ready {
        process::exit(1);
    }
}

/// `spanda verify mission <file.sd> [--target T] [--json]`
pub fn cmd_verify_mission(args: &[String]) {
    // Description:
    //     Cmd verify mission.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::readiness_cli::cmd_verify_mission(args);

    let parsed = parse_readiness_cli(args);
    let source = read_file(&parsed.file);
    let program = parse_program(&source);
    let reports = verify_mission(&program, parsed.options.target.as_deref());
    if parsed.format == ReportFormat::Json {
        println!("{}", serde_json::to_string_pretty(&reports).unwrap());
    } else {
        print!("{}", format_mission_verification(&reports));
    }
    if reports.iter().any(|r| !r.achievable) {
        process::exit(1);
    }
}

/// `spanda analyze-failure <file.sd>`
pub fn cmd_analyze_failure(args: &[String]) {
    // Description:
    //     Cmd analyze failure.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::readiness_cli::cmd_analyze_failure(args);

    let json = args.iter().any(|a| a == "--json");
    let file = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Missing file path");
            process::exit(1);
        });
    let source = read_file(&file);
    let program = parse_program(&source);
    let report = analyze_failure(&program);
    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        print!("{}", format_failure_analysis(&report));
    }
}

/// `spanda safety-report <file.sd>`
pub fn cmd_safety_report(args: &[String]) {
    // Description:
    //     Cmd safety report.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::readiness_cli::cmd_safety_report(args);

    let parsed = parse_readiness_cli(args);
    let source = read_file(&parsed.file);
    let program = parse_program(&source);
    let report = generate_safety_report(&program, &parsed.file);
    println!("{}", format_safety_report(&report, parsed.format));
    if !report.deployable {
        process::exit(1);
    }
}

/// `spanda twin readiness <file.sd> [--trace <path>]`
pub fn cmd_twin_readiness(args: &[String]) {
    // Description:
    //     Cmd twin readiness.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::readiness_cli::cmd_twin_readiness(args);

    let json = args.iter().any(|a| a == "--json");
    let file = args
        .iter()
        .find(|a| !a.starts_with('-') && *a != "--trace")
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Missing file path");
            process::exit(1);
        });
    let trace_path = args
        .windows(2)
        .find(|w| w[0] == "--trace")
        .map(|w| w[1].clone());
    let source = read_file(&file);
    let program = parse_program(&source);
    let status = evaluate_twin_readiness(&program, trace_path.as_deref().map(Path::new));
    if json {
        println!("{}", serde_json::to_string_pretty(&status).unwrap());
    } else {
        print!("{}", spanda_readiness::format_twin_readiness(&status));
    }
}

/// `spanda fleet readiness <file.sd> [--target T] [--runtime]`
pub fn cmd_fleet_readiness(args: &[String]) {
    // Description:
    //     Cmd fleet readiness.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::readiness_cli::cmd_fleet_readiness(args);

    let parsed = parse_readiness_cli(args);
    let source = read_file(&parsed.file);
    let program = parse_program(&source);
    let report = evaluate_fleet_readiness(&program, &parsed.options);
    if parsed.format == ReportFormat::Json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        print!("{}", format_fleet_readiness(&report));
    }
}

/// `spanda audit <file.sd>`
pub fn cmd_audit(args: &[String]) {
    // Description:
    //     Cmd audit.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::readiness_cli::cmd_audit(args);

    let json = args.iter().any(|a| a == "--json");
    let file = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Missing file path");
            process::exit(1);
        });
    let source = read_file(&file);
    let program = parse_program(&source);
    let report = audit_program(&program, &source);
    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        print!("{}", format_audit(&report));
    }
    if report.critical_count > 0 {
        process::exit(1);
    }
}

/// `spanda verify-fleet <file.sd>`
pub fn cmd_verify_fleet(args: &[String]) {
    // Description:
    //     Cmd verify fleet.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::readiness_cli::cmd_verify_fleet(args);

    let json = args.iter().any(|a| a == "--json");
    let file = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Missing file path");
            process::exit(1);
        });
    let source = read_file(&file);
    let program = parse_program(&source);
    let report = verify_fleet(&program);
    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        for f in &report.findings {
            println!("[{}] {} — {}", f.severity, f.category, f.message);
        }
    }
    if !report.compatible {
        process::exit(1);
    }
}

/// `spanda verify-approval <file.sd>`
pub fn cmd_verify_approval(args: &[String]) {
    // Description:
    //     Cmd verify approval.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::readiness_cli::cmd_verify_approval(args);

    let json = args.iter().any(|a| a == "--json");
    let file = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Missing file path");
            process::exit(1);
        });
    let source = read_file(&file);
    let program = parse_program(&source);
    let report = verify_approvals(&program);
    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        for row in &report.rows {
            println!(
                "{} / {} — path:{} actor:{} fallback:{} [{}]",
                row.actor,
                row.action,
                row.approval_path_exists,
                row.actor_exists,
                row.fallback_exists,
                row.status
            );
        }
    }
    if !report.compatible {
        process::exit(1);
    }
}

/// Agent-shaped readiness JSON for CLI and service mirrors (`GET /v1/readiness`).
pub fn evaluate_agent_readiness_json(
    source: &str,
    target: Option<&str>,
    include_runtime: bool,
    inject_health_faults: bool,
) -> Result<String, String> {
    // Description:
    //     Evaluate agent readiness json.
    //
    // Inputs:
    //     source: &str
    //         Caller-supplied source.
    //     arge: Option<&str>
    //         Caller-supplied arge.
    //     include_runtime: bool
    //         Caller-supplied include runtime.
    //     inject_health_faults: bool
    //         Caller-supplied inject health faults.
    //
    // Outputs:
    //     result: Result<String, String>
    //         Return value from `evaluate_agent_readiness_json`.
    //
    // Example:

    //     let result = spanda_cli::readiness_cli::evaluate_agent_readiness_json(source, arge, include_runtime, inject_health_faults);

    spanda_readiness::evaluate_agent_readiness_json(
        source,
        target,
        include_runtime,
        inject_health_faults,
    )
}

/// `spanda safety-coverage <file.sd> [--json] [--format markdown]`
pub fn cmd_safety_coverage(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let markdown = args.iter().any(|a| {
        a == "--format"
            && args
                .windows(2)
                .any(|w| w[0] == "--format" && w[1] == "markdown")
    }) || args.iter().any(|a| a == "--markdown");
    let file = args
        .iter()
        .find(|a| !a.starts_with('-') && *a != "markdown")
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Missing file path");
            process::exit(1);
        });
    let source = read_file(&file);
    let program = parse_program(&source);
    let report = evaluate_safety_coverage(&program, &file);
    println!("{}", format_safety_coverage(&report, json, markdown));
}

/// Top-level readiness dispatch for subcommands.
pub fn readiness_dispatch(args: &[String]) {
    // Description:
    //     Readiness dispatch.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::readiness_cli::readiness_dispatch(args);

    if args.is_empty() {
        eprintln!(
            "Usage: spanda readiness <file.sd> [--target <profile>] [--runtime] [--inject-health-faults] [--json|--agent-json|--markdown|--html]"
        );
        process::exit(1);
    }
    cmd_readiness(args);
}
