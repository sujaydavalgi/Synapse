//! CLI for verify-time integrity verification.
//!
use spanda_config::{
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
use spanda_tamper::{
    apply_agent_integrity, compare_agent_integrity, format_integrity_report,
    generate_integrity_report, AgentIntegrityActual, AgentIntegrityExpected, IntegrityFormat,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
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

fn file_arg(args: &[String]) -> String {
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--baseline" | "--agent" | "--config" => index += 2,
            "--json" => index += 1,
            other if !other.starts_with('-') => return other.to_string(),
            _ => index += 1,
        }
    }
    eprintln!(
        "Usage: spanda integrity <file.sd> [--baseline <file.sd>] [--agent <Robot@Hardware>] [--config <spanda.toml>] [--json]"
    );
    process::exit(1);
}

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == flag)
        .map(|window| window[1].clone())
}

fn project_root_from_args(args: &[String]) -> PathBuf {
    if let Some(path) = flag_value(args, "--config") {
        let config_path = PathBuf::from(path);
        return config_path
            .parent()
            .unwrap_or(&config_path)
            .to_path_buf();
    }
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    SpandaManifest::find_project_root(&cwd).unwrap_or(cwd)
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

/// `spanda integrity <file.sd> [--baseline <file.sd>] [--agent <Robot@Hardware>] [--json]`
pub fn integrity_dispatch(args: &[String]) {
    let file = file_arg(args);
    let path = Path::new(&file);
    let program = parse_program(path);
    let baseline_path = flag_value(args, "--baseline");
    let (baseline_program, baseline_label) = if let Some(baseline_file) = baseline_path {
        let baseline = parse_program(Path::new(&baseline_file));
        (Some(baseline), Some(baseline_file))
    } else {
        (None, None)
    };
    let mut report = generate_integrity_report(
        &program,
        &file,
        baseline_program.as_ref(),
        baseline_label.as_deref(),
    );
    if let Some(agent_filter) = flag_value(args, "--agent") {
        let root = project_root_from_args(args);
        let config = ConfigResolver::new()
            .resolve_from_dir(&root)
            .ok();
        let program_hash = hash_program_artifact(&file);
        let expected_states = expected_agent_states(
            &program,
            config.as_ref(),
            program_hash.as_deref(),
        );
        let expected = expected_states
            .into_iter()
            .find(|state| {
                agent_filter == state.target_key || agent_filter == state.robot_name
            })
            .unwrap_or_else(|| {
                eprintln!("--agent '{agent_filter}' does not match any deploy target in program");
                process::exit(1);
            });
        let deploy_registry = load_agent_registry(&default_agents_path());
        let fleet_registry = load_fleet_agent_registry(&default_fleet_agents_path());
        let snapshot = fetch_agent_snapshot(
            &expected.target_key,
            &expected.robot_name,
            Some(&agent_filter),
            &deploy_registry,
            &fleet_registry,
        )
        .unwrap_or_else(|error| {
            eprintln!("Failed to fetch agent status: {error}");
            process::exit(1);
        });
        let checks = compare_agent_integrity(
            &AgentIntegrityExpected {
                program_hash: expected.program_hash,
                hardware_profile: expected.hardware_profile,
            },
            &AgentIntegrityActual {
                agent_id: snapshot.agent_id.clone(),
                program_hash: snapshot.program_hash,
                hardware_profile: snapshot.hardware_profile,
                healthy: snapshot.healthy,
            },
        );
        apply_agent_integrity(&mut report, &snapshot.agent_id, checks);
    }
    let format = if args.iter().any(|arg| arg == "--json") {
        IntegrityFormat::Json
    } else {
        IntegrityFormat::Text
    };
    println!("{}", format_integrity_report(&report, format));
    if !report.passed {
        process::exit(1);
    }
}
