//! OTA deploy CLI handlers (`spanda deploy plan|rollout|rollback|status|agent`).

use spanda_core::{
    agent_health, apply_rollout, build_deploy_plan, compile, default_agent_state_path,
    default_agents_path, default_state_path, execute_remote_rollout, execute_remote_rollback,
    load_agent_registry, load_deploy_state, orchestrate_fleets, plan_rollout, register_agent,
    rollback_targets, run_deploy_agent_server, save_agent_registry, save_deploy_state,
    DeployAgentTls, DeployState, RolloutOptions, RolloutStrategy,
};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

fn read_source(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading {path}: {e}");
        process::exit(1);
    })
}

fn parse_program(source: &str, file: &str) -> spanda_core::Program {
    compile(source).unwrap_or_else(|e| {
        eprintln!("Error compiling {file}: {e}");
        process::exit(1);
    }).program
}

fn state_path() -> std::path::PathBuf {
    env::var("SPANDA_DEPLOY_STATE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| default_state_path())
}

fn agents_path() -> std::path::PathBuf {
    env::var("SPANDA_DEPLOY_AGENTS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| default_agents_path())
}

pub fn deploy_dispatch(args: &[String]) {
    if args.is_empty() {
        usage();
        process::exit(1);
    }
    match args[0].as_str() {
        "plan" => cmd_plan(&args[1..]),
        "rollout" => cmd_rollout(&args[1..]),
        "rollback" => cmd_rollback(&args[1..]),
        "status" => cmd_status(&args[1..]),
        "agent" => cmd_agent(&args[1..]),
        other if !other.starts_with('-') => {
            eprintln!("Unknown deploy subcommand '{other}'");
            usage();
            process::exit(1);
        }
        _ => {
            usage();
            process::exit(1);
        }
    }
}

pub fn deploy_usage_lines() -> &'static str {
    "           spanda deploy plan [--json] [--version <ver>] <file.sd>\n\
     spanda deploy rollout [--json] [--remote] [--strategy all|canary|staged] [--canary-percent N] [--version <ver>] [--dry-run] <file.sd>\n\
     spanda deploy rollback [--json] [--remote] <file.sd>\n\
     spanda deploy status [--json]\n\
     spanda deploy agent start [--bind <addr>] [--target <Robot@Hardware>] [--token <t>] [--tls-cert <pem>] [--tls-key <pem>] [--require-hash]\n\
     spanda deploy agent register <Robot@Hardware> <http(s)://host:port> [--token <t>]\n\
     spanda deploy agent list [--json]\n\
     spanda deploy --target wasm [--out <file.json>] <file.sd>"
}

fn usage() {
    eprintln!("Usage:\n{}", deploy_usage_lines());
}

fn cmd_plan(args: &[String]) {
    let mut json = false;
    let mut version = "1.0.0".to_string();
    let mut file: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            "--version" if i + 1 < args.len() => {
                version = args[i + 1].clone();
                i += 1;
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
    let source = read_source(&file);
    let program = parse_program(&source, &file);
    let plan = build_deploy_plan(&program, &file, &version);
    if json {
        println!("{}", serde_json::to_string_pretty(&plan).unwrap());
    } else {
        println!("Deploy plan for {file} (version {version})");
        for a in &plan.assignments {
            println!("  {} -> {}", a.robot_name, a.hardware);
        }
        if !plan.certifications.is_empty() {
            println!("  certifications: {}", plan.certifications.join(", "));
        }
        if let Some(hash) = &plan.program_hash {
            println!("  program_hash: {hash}");
        }
    }
}

fn cmd_rollout(args: &[String]) {
    let mut json = false;
    let mut dry_run = false;
    let mut remote = false;
    let mut version = "1.0.0".to_string();
    let mut strategy = RolloutStrategy::All;
    let mut canary_percent = 10u8;
    let mut file: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            "--dry-run" => dry_run = true,
            "--remote" => remote = true,
            "--version" if i + 1 < args.len() => {
                version = args[i + 1].clone();
                i += 1;
            }
            "--strategy" if i + 1 < args.len() => {
                strategy = match args[i + 1].as_str() {
                    "all" => RolloutStrategy::All,
                    "canary" => RolloutStrategy::Canary,
                    "staged" => RolloutStrategy::Staged,
                    other => {
                        eprintln!("Unknown strategy '{other}'");
                        process::exit(1);
                    }
                };
                i += 1;
            }
            "--canary-percent" if i + 1 < args.len() => {
                canary_percent = args[i + 1].parse().unwrap_or(10);
                i += 1;
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
    let source = read_source(&file);
    let program = parse_program(&source, &file);
    let plan = build_deploy_plan(&program, &file, &version);
    let options = RolloutOptions {
        strategy,
        canary_percent,
        version: version.clone(),
        dry_run,
        ..Default::default()
    };
    let result = if remote {
        let registry = load_agent_registry(&agents_path());
        execute_remote_rollout(&plan, &options, &registry)
    } else {
        plan_rollout(&plan, &options)
    };
    if !dry_run {
        let path = state_path();
        let mut state = load_deploy_state(&path);
        apply_rollout(&mut state, &result);
        if let Err(e) = save_deploy_state(&path, &state) {
            eprintln!("Warning: could not save deploy state: {e}");
        }
    }
    print_rollout(&result, json);
}

fn cmd_rollback(args: &[String]) {
    let mut json = false;
    let mut remote = false;
    let mut file: Option<String> = None;
    for arg in args {
        match arg.as_str() {
            "--json" => json = true,
            "--remote" => remote = true,
            other if !other.starts_with('-') && file.is_none() => file = Some(other.to_string()),
            other => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
        }
    }
    let file = file.unwrap_or_else(|| {
        eprintln!("Missing file path");
        process::exit(1);
    });
    let source = read_source(&file);
    let program = parse_program(&source, &file);
    let plan = build_deploy_plan(&program, &file, "rollback");
    let path = state_path();
    let mut state = load_deploy_state(&path);
    let result = if remote {
        let registry = load_agent_registry(&agents_path());
        let remote_result = execute_remote_rollback(&plan, &registry);
        rollback_targets(&mut state, &plan, true);
        remote_result
    } else {
        rollback_targets(&mut state, &plan, true)
    };
    if let Err(e) = save_deploy_state(&path, &state) {
        eprintln!("Warning: could not save deploy state: {e}");
    }
    print_rollout(&result, json);
}

fn cmd_agent(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: spanda deploy agent start|register|list");
        process::exit(1);
    }
    match args[0].as_str() {
        "start" => cmd_agent_start(&args[1..]),
        "register" => cmd_agent_register(&args[1..]),
        "list" => cmd_agent_list(&args[1..]),
        other => {
            eprintln!("Unknown deploy agent subcommand '{other}'");
            process::exit(1);
        }
    }
}

fn cmd_agent_start(args: &[String]) {
    let mut bind = "127.0.0.1:8765".to_string();
    let mut target = String::new();
    let mut token = None;
    let mut tls_cert = None;
    let mut tls_key = None;
    let mut require_hash = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--bind" if i + 1 < args.len() => {
                bind = args[i + 1].clone();
                i += 1;
            }
            "--target" if i + 1 < args.len() => {
                target = args[i + 1].clone();
                i += 1;
            }
            "--token" if i + 1 < args.len() => {
                token = Some(args[i + 1].clone());
                i += 1;
            }
            "--tls-cert" if i + 1 < args.len() => {
                tls_cert = Some(args[i + 1].clone());
                i += 1;
            }
            "--tls-key" if i + 1 < args.len() => {
                tls_key = Some(args[i + 1].clone());
                i += 1;
            }
            "--require-hash" => require_hash = true,
            other => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
        }
        i += 1;
    }
    if target.is_empty() {
        eprintln!("Missing --target Robot@Hardware");
        process::exit(1);
    }
    let tls = match (tls_cert, tls_key) {
        (Some(cert_path), Some(key_path)) => Some(DeployAgentTls {
            cert_path,
            key_path,
        }),
        (None, None) => None,
        _ => {
            eprintln!("Both --tls-cert and --tls-key are required for HTTPS agents");
            process::exit(1);
        }
    };
    if let Err(err) = run_deploy_agent_server(
        &bind,
        &target,
        token,
        &default_agent_state_path(),
        tls,
        require_hash,
    ) {
        eprintln!("Deploy agent failed: {err}");
        process::exit(1);
    }
}

fn cmd_agent_register(args: &[String]) {
    let mut target = None;
    let mut url = None;
    let mut token = None;
    for (idx, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "--token" if idx + 1 < args.len() => {
                token = Some(args[idx + 1].clone());
            }
            other if !other.starts_with('-') && target.is_none() => target = Some(other.to_string()),
            other if !other.starts_with('-') && url.is_none() => url = Some(other.to_string()),
            other if other.starts_with('-') => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
            _ => {}
        }
    }
    let target = target.unwrap_or_else(|| {
        eprintln!("Missing target Robot@Hardware");
        process::exit(1);
    });
    let url = url.unwrap_or_else(|| {
        eprintln!("Missing agent URL (http(s)://host:port)");
        process::exit(1);
    });
    let path = agents_path();
    let mut registry = load_agent_registry(&path);
    if let Err(err) = register_agent(&mut registry, target, url, token) {
        eprintln!("Register failed: {err}");
        process::exit(1);
    }
    if let Err(err) = save_agent_registry(&path, &registry) {
        eprintln!("Warning: could not save agent registry: {err}");
        process::exit(1);
    }
    println!("Registered deploy agent in {}", path.display());
}

fn cmd_agent_list(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let registry = load_agent_registry(&agents_path());
    if json {
        println!("{}", serde_json::to_string_pretty(&registry).unwrap());
        return;
    }
    println!("Deploy agents ({})", agents_path().display());
    if registry.agents.is_empty() {
        println!("  (no agents registered)");
        return;
    }
    for entry in &registry.agents {
        let health = agent_health(entry).unwrap_or(false);
        println!("  {} -> {} (healthy={health})", entry.target, entry.url);
    }
}

fn cmd_status(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let path = state_path();
    let state: DeployState = load_deploy_state(&path);
    if json {
        println!("{}", serde_json::to_string_pretty(&state).unwrap());
    } else {
        println!("Deploy state ({})", path.display());
        for (key, ver) in &state.current_version {
            let prev = state
                .previous_version
                .get(key)
                .map(|s| s.as_str())
                .unwrap_or("-");
            println!("  {key}: {ver} (previous: {prev})");
        }
        if state.current_version.is_empty() {
            println!("  (no deployments recorded)");
        }
    }
}

fn print_rollout(result: &spanda_core::RolloutResult, json: bool) {
    if json {
        println!("{}", serde_json::to_string_pretty(result).unwrap());
    } else {
        println!(
            "Rollout {} ({:?}) — {}",
            result.version,
            result.strategy,
            if result.success { "ok" } else { "failed" }
        );
        for step in &result.steps {
            println!(
                "  {}@{} -> {:?} v{}",
                step.robot_name, step.hardware, step.status, step.version
            );
        }
    }
    let _ = io::stdout().flush();
}

pub fn fleet_orchestrate_dispatch(args: &[String]) {
    let mut json = false;
    let mut file: Option<String> = None;
    for arg in args {
        match arg.as_str() {
            "--json" => json = true,
            other if !other.starts_with('-') && file.is_none() => file = Some(other.to_string()),
            other => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
        }
    }
    let file = file.unwrap_or_else(|| {
        eprintln!("Missing file path");
        process::exit(1);
    });
    let source = read_source(&file);
    let program = parse_program(&source, &file);
    let result = orchestrate_fleets(&program, &file);
    if json {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else {
        println!("Fleet orchestration for {file}");
        for fleet in &result.fleets {
            println!("  fleet {} ({})", fleet.fleet_name, fleet.coordination_mode);
            for member in &fleet.members {
                println!(
                    "    {} mission={:?} state={} step='{}' peer={}",
                    member.robot_name,
                    member.mission_name,
                    member.mission_state,
                    member.current_step,
                    member.has_peer_link
                );
                for handoff in &member.peer_handoffs {
                    println!("      handoff: {handoff}");
                }
            }
            for message in &fleet.peer_messages {
                println!("    peer: {message}");
            }
        }
    }
}
