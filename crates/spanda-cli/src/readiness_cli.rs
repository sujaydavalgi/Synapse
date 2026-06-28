//! CLI commands for operational readiness and mission assurance.

use crate::config_load::{ensure_config_valid, load_system_config};
use spanda_config::{expected_agent_states, AgentDriftSnapshot};
use spanda_fleet::{
    default_fleet_agents_path, fleet_agent_status, load_fleet_agent_registry, lookup_fleet_agent,
    FleetAgentStatusResponse,
};
use spanda_lexer::tokenize;
use spanda_ota::{
    agent_status, default_agents_path, hash_program_artifact, load_agent_registry, lookup_agent,
    AgentStatusResponse,
};
use spanda_parser::parse;
use spanda_readiness::{
    analyze_failure, analyze_readiness_trends, audit_program, build_runtime_context_with_config,
    default_readiness_history_path, evaluate_fleet_readiness, evaluate_readiness_with_runtime,
    evaluate_safety_coverage, evaluate_twin_readiness, format_audit, format_failure_analysis,
    format_fleet_readiness, format_mission_verification, format_readiness, format_readiness_trends,
    format_safety_coverage, format_safety_report, generate_safety_report, load_readiness_history,
    parse_forecast_horizon, readiness_options_from_flags, record_readiness_snapshot,
    verify_approvals, verify_fleet, verify_mission, ReadinessOptions, ReadinessPolicy,
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
    record: bool,
    compliance_profile: Option<String>,
    operational_policy: Option<String>,
    history_path: Option<String>,
    _system_config: Option<std::sync::Arc<spanda_config::ResolvedSystemConfig>>,
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
    let mut baseline_path: Option<String> = None;
    let mut agent_filter: Option<String> = None;
    let mut history_path: Option<String> = None;
    let mut record = false;
    let mut compliance_profile: Option<String> = None;
    let mut operational_policy: Option<String> = None;
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
            "--record" => record = true,
            "--profile" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--profile requires a compliance profile name");
                    process::exit(1);
                }
                compliance_profile = Some(args[i].clone());
            }
            "--policy" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--policy requires an operational policy name");
                    process::exit(1);
                }
                operational_policy = Some(args[i].clone());
            }
            "--history" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--history requires a path");
                    process::exit(1);
                }
                history_path = Some(args[i].clone());
            }
            "--config" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--config requires a path to spanda.toml");
                    process::exit(1);
                }
                config_path = Some(args[i].clone());
            }
            "--baseline" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--baseline requires a path to spanda.toml or project directory");
                    process::exit(1);
                }
                baseline_path = Some(args[i].clone());
            }
            "--agent" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--agent requires a deploy target (Robot@Hardware or Robot)");
                    process::exit(1);
                }
                agent_filter = Some(args[i].clone());
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
    let system_config = load_system_config(
        std::path::Path::new(&file),
        config_path.as_deref().map(std::path::Path::new),
    );
    ensure_config_valid(system_config.as_ref().map(|arc| arc.as_ref()));
    let baseline_config = baseline_path.as_ref().map(|path| {
        let p = std::path::Path::new(path);
        let root = if p.is_dir() {
            p.to_path_buf()
        } else {
            p.parent().unwrap_or(p).to_path_buf()
        };
        std::sync::Arc::new(
            spanda_config::ConfigResolver::new()
                .resolve_from_dir(&root)
                .unwrap_or_else(|e| {
                    eprintln!("{e}");
                    process::exit(1);
                }),
        )
    });
    if target.is_none() {
        if let Some(ref cfg) = system_config {
            target = spanda_config::default_verify_target(cfg);
        }
    }
    let mut options = readiness_options_from_flags(
        &program,
        target,
        include_runtime,
        inject_health_faults,
        simulate,
        strict,
    );
    options.system_config = system_config.clone();
    options.baseline_config = baseline_config;
    if let Some(filter) = agent_filter.as_deref() {
        populate_agent_drift(
            &program,
            &file,
            filter,
            system_config.as_deref(),
            &mut options,
        );
    }
    ParsedReadinessCli {
        format,
        file,
        options,
        agent_json,
        record,
        compliance_profile,
        operational_policy,
        history_path,
        _system_config: system_config,
    }
}

fn populate_agent_drift(
    program: &spanda_ast::nodes::Program,
    program_path: &str,
    agent_filter: &str,
    system_config: Option<&spanda_config::ResolvedSystemConfig>,
    options: &mut ReadinessOptions,
) {
    let program_hash = hash_program_artifact(program_path);
    let expected_states = expected_agent_states(program, system_config, program_hash.as_deref());
    let expected = expected_states
        .into_iter()
        .find(|state| agent_filter == state.target_key || agent_filter == state.robot_name)
        .unwrap_or_else(|| {
            eprintln!("--agent '{agent_filter}' does not match any deploy target in program");
            process::exit(1);
        });
    let deploy_registry = load_agent_registry(&default_agents_path());
    let fleet_registry = load_fleet_agent_registry(&default_fleet_agents_path());
    let snapshot = fetch_agent_snapshot(
        &expected.target_key,
        &expected.robot_name,
        Some(agent_filter),
        &deploy_registry,
        &fleet_registry,
    )
    .unwrap_or_else(|error| {
        eprintln!("Failed to fetch agent status: {error}");
        process::exit(1);
    });
    options.agent_drift.push((expected, snapshot));
}

fn fetch_agent_snapshot(
    target_key: &str,
    robot_name: &str,
    agent_filter: Option<&str>,
    deploy_registry: &spanda_ota::DeployAgentRegistry,
    fleet_registry: &spanda_fleet::FleetAgentRegistry,
) -> Result<AgentDriftSnapshot, String> {
    let deploy_key = if agent_filter.is_some_and(|filter| filter.contains('@')) {
        agent_filter.unwrap()
    } else {
        target_key
    };
    if let Some(entry) = lookup_agent(deploy_registry, deploy_key) {
        let status = agent_status(entry)?;
        return Ok(snapshot_from_deploy_status(deploy_key, &status));
    }
    let fleet_robot = agent_filter.unwrap_or(robot_name);
    let entry = lookup_fleet_agent(fleet_registry, fleet_robot)
        .ok_or_else(|| format!("no agent registered for '{fleet_robot}'"))?;
    let status = fleet_agent_status(entry)?;
    Ok(snapshot_from_fleet_status(fleet_robot, &status))
}

fn snapshot_from_deploy_status(id: &str, status: &AgentStatusResponse) -> AgentDriftSnapshot {
    AgentDriftSnapshot {
        agent_id: id.to_string(),
        target: Some(status.target.clone()),
        robot_name: status.robot_name.clone(),
        hardware_profile: status.hardware_profile.clone(),
        firmware_version: status.firmware_version.clone(),
        program_hash: status.program_hash.clone(),
        current_version: Some(status.current_version.clone()),
        packages: status.packages.clone(),
        healthy: status.healthy,
        attestation_contract: status.attestation_contract.clone(),
        attestation_verified: status.attestation_verified,
        boot_state: status.boot_state.clone(),
    }
}

fn snapshot_from_fleet_status(id: &str, status: &FleetAgentStatusResponse) -> AgentDriftSnapshot {
    AgentDriftSnapshot {
        agent_id: id.to_string(),
        target: None,
        robot_name: status.robot_name.clone(),
        hardware_profile: status.hardware_profile.clone(),
        firmware_version: status.firmware_version.clone(),
        program_hash: status.program_hash.clone(),
        current_version: None,
        packages: status.packages.clone(),
        healthy: status.healthy,
        ..AgentDriftSnapshot::default()
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

    let runtime = options.include_runtime.then(|| {
        build_runtime_context_with_config(
            program,
            options.inject_health_faults,
            options.system_config.as_deref(),
        )
    });
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
    let mut report = evaluate_with_options(&program, &parsed.options);
    let mut policy_passed = true;
    let mut policy_report: Option<spanda_policy::PolicyEvaluationReport> = None;
    if let Some(policy_name) = &parsed.operational_policy {
        match spanda_policy::evaluate_readiness_with_policy(
            &program,
            &parsed.file,
            &parsed.options,
            policy_name,
            report,
        ) {
            Ok((merged, evaluated)) => {
                report = merged;
                policy_passed = evaluated.passed;
                policy_report = Some(evaluated);
            }
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        }
    }
    if parsed.record {
        let default_history = default_readiness_history_path();
        let history_path = parsed
            .history_path
            .as_deref()
            .map(std::path::Path::new)
            .unwrap_or(&default_history);
        if let Err(error) = record_readiness_snapshot(&report, &parsed.file, history_path) {
            eprintln!("Failed to record readiness history: {error}");
            process::exit(1);
        }
    }
    println!("{}", format_readiness(&report, parsed.format));
    if let Some(policy_report) = &policy_report {
        println!(
            "{}",
            spanda_policy::format_policy_report(
                policy_report,
                matches!(parsed.format, ReportFormat::Json)
            )
        );
    }
    if let Some(profile_name) = &parsed.compliance_profile {
        if profile_name == "human_collaboration" {
            if let Some(ref cfg) = parsed.options.system_config {
                let human_report =
                    spanda_readiness::evaluate_human_collaboration(cfg.as_ref(), &program);
                println!(
                    "{}",
                    spanda_readiness::format_human_readiness(&human_report)
                );
                if !human_report.mission_ready {
                    process::exit(1);
                }
            } else {
                eprintln!("human_collaboration profile requires --config spanda.toml");
                process::exit(1);
            }
        }
        match spanda_compliance::evaluate_compliance_profile(&program, profile_name, &parsed.file) {
            Ok(compliance) => {
                println!(
                    "{}",
                    spanda_compliance::format_compliance_report(
                        &compliance,
                        matches!(parsed.format, ReportFormat::Json)
                    )
                );
                if !compliance.passed {
                    process::exit(1);
                }
            }
            Err(error) => {
                eprintln!("{error}");
                process::exit(1);
            }
        }
    }
    if !report.mission_ready || !policy_passed {
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

/// `spanda readiness trends <file.sd> [--forecast 7d] [--history <path>] [--json]`
pub fn cmd_readiness_trends(args: &[String]) {
    let mut file: Option<String> = None;
    let mut forecast_days: Option<u32> = None;
    let mut history_path: Option<String> = None;
    let mut json = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json = true,
            "--forecast" => {
                index += 1;
                if index >= args.len() {
                    eprintln!("--forecast requires a horizon such as 7d");
                    process::exit(1);
                }
                forecast_days = parse_forecast_horizon(&args[index]);
                if forecast_days.is_none() {
                    eprintln!("Invalid --forecast value: {}", args[index]);
                    process::exit(1);
                }
            }
            "--history" => {
                index += 1;
                if index >= args.len() {
                    eprintln!("--history requires a path");
                    process::exit(1);
                }
                history_path = Some(args[index].clone());
            }
            other if !other.starts_with('-') && file.is_none() => file = Some(other.to_string()),
            other => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
        }
        index += 1;
    }
    let file = file.unwrap_or_else(|| {
        eprintln!(
            "Usage: spanda readiness trends <file.sd> [--forecast 7d] [--history <path>] [--json]"
        );
        process::exit(1);
    });
    let default_history = default_readiness_history_path();
    let path = history_path
        .as_deref()
        .map(std::path::Path::new)
        .unwrap_or(&default_history);
    let history = load_readiness_history(path);
    let minimum_score = ReadinessPolicy::default().minimum_score;
    let report = analyze_readiness_trends(&history, &file, forecast_days, minimum_score);
    println!("{}", format_readiness_trends(&report, json));
    if report.sample_count < 2 || report.forecast.as_ref().is_some_and(|f| f.risk_warning) {
        process::exit(1);
    }
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

    if args.first().map(String::as_str) == Some("trends") {
        cmd_readiness_trends(&args[1..]);
        return;
    }

    if args.is_empty() {
        eprintln!(
            "Usage: spanda readiness <file.sd> [--baseline <dir|spanda.toml>] [--agent <Robot@Hardware>] [--record] [--profile <name>] [--policy <name>] [--target <profile>] [--runtime] [--inject-health-faults] [--json|--agent-json|--markdown|--html]\n\
             spanda readiness trends <file.sd> [--forecast 7d] [--history <path>] [--json]"
        );
        process::exit(1);
    }
    cmd_readiness(args);
}
