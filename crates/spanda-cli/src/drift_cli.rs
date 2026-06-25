//! CLI for configuration and agent drift detection.
//!
use spanda_config::{
    append_agent_drift, append_program_drift, detect_agent_drift, detect_config_drift,
    expected_agent_states, AgentDriftSnapshot, ConfigResolver, SpandaManifest,
};
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

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    for (i, arg) in args.iter().enumerate() {
        if arg == flag {
            return args.get(i + 1).cloned();
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
    let baseline_root = root_from_flag(args, "--baseline");
    let program_path = positional_program_path(args);
    if baseline_root.is_none() && program_path.is_none() {
        eprintln!(
            "Usage:\n  \
             spanda drift <program.sd> [--agent <Robot@Hardware|Robot>] [--config <spanda.toml>] [--json]\n  \
             spanda drift --baseline <dir|spanda.toml> [--config <spanda.toml>] [program.sd] [--json]\n  \
             spanda config drift <same flags>"
        );
        process::exit(1);
    }

    let current_root = root_from_flag(args, "--config").unwrap_or_else(|| project_root_from_args(args));
    let current = load_resolved(&current_root);
    let mut report = if let Some(base_root) = baseline_root.as_ref() {
        let baseline = load_resolved(base_root);
        detect_config_drift(&baseline, &current)
    } else {
        spanda_config::ConfigDriftReport {
            findings: Vec::new(),
            passed: true,
            baseline_project: "(expected)".into(),
            current_project: current.project_name().to_string(),
        }
    };

    let program_path = program_path.or_else(|| {
        baseline_root.as_ref().and_then(|_| {
            args.iter()
                .find(|a| !a.starts_with('-') && Path::new(a).extension().is_some_and(|e| e == "sd"))
                .map(PathBuf::from)
        })
    });

    if let Some(path) = program_path.as_deref() {
        let program = parse_program_file(path);
        append_program_drift(&mut report, &program, &current);
        let hash = hash_program_artifact(path.to_str().unwrap_or_default());
        let expected_states = expected_agent_states(&program, Some(&current), hash.as_deref());
        let agent_filter = flag_value(args, "--agent");
        if let Some(ref filter) = agent_filter {
            let matched = expected_states.iter().any(|state| {
                filter == &state.target_key || filter == &state.robot_name
            });
            if !matched {
                report.push(spanda_config::DriftFinding {
                    dimension: spanda_config::DriftDimension::Program,
                    severity: spanda_config::DriftSeverity::High,
                    message: format!("--agent '{filter}' does not match any deploy target in program"),
                    path: None,
                });
            }
        }
        let deploy_registry = load_agent_registry(&default_agents_path());
        let fleet_registry = load_fleet_agent_registry(&default_fleet_agents_path());
        for expected in expected_states {
            if let Some(ref filter) = agent_filter {
                if filter != &expected.target_key && filter != &expected.robot_name {
                    continue;
                }
            } else if lookup_agent(&deploy_registry, &expected.target_key).is_none()
                && lookup_fleet_agent(&fleet_registry, &expected.robot_name).is_none()
            {
                continue;
            }
            match fetch_agent_snapshot(
                &expected.target_key,
                &expected.robot_name,
                agent_filter.as_deref(),
                &deploy_registry,
                &fleet_registry,
            ) {
                Ok(snapshot) => {
                    append_agent_drift(&mut report, detect_agent_drift(&expected, &snapshot));
                }
                Err(err) => {
                    report.push(spanda_config::DriftFinding {
                        dimension: spanda_config::DriftDimension::Hardware,
                        severity: spanda_config::DriftSeverity::High,
                        message: format!(
                            "failed to fetch agent status for '{}': {err}",
                            expected.target_key
                        ),
                        path: Some(format!("agents.{}", expected.target_key)),
                    });
                }
            }
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        print_drift_report(&report);
    }
    if !report.passed {
        process::exit(1);
    }
}

fn print_drift_report(report: &spanda_config::ConfigDriftReport) {
    if report.findings.is_empty() {
        println!(
            "No drift detected for project '{}'.",
            report.current_project
        );
        return;
    }
    println!(
        "Drift report: {} -> {}",
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

fn fetch_agent_snapshot(
    target_key: &str,
    robot_name: &str,
    agent_filter: Option<&str>,
    deploy_registry: &spanda_ota::DeployAgentRegistry,
    fleet_registry: &spanda_fleet::FleetAgentRegistry,
) -> Result<AgentDriftSnapshot, String> {
    let deploy_key = if agent_filter.is_some_and(|f| f.contains('@')) {
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
    }
}

fn positional_program_path(args: &[String]) -> Option<PathBuf> {
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--baseline" | "--config" | "--agent" => {
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
