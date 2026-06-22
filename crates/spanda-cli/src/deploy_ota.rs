//! OTA deploy CLI handlers (`spanda deploy plan|rollout|rollback|status|agent`).

use spanda_core::{
    agent_health, apply_rollout, build_deploy_bundle, build_deploy_plan, compile,
    default_agent_state_path, default_agents_path, default_fleet_agent_state_path,
    default_fleet_agents_path, default_state_path, execute_remote_rollout,
    execute_remote_rollback, fleet_agent_health, load_agent_registry, load_deploy_state,
    load_fleet_agent_registry, mesh_registry_path, orchestrate_fleets, orchestrate_fleets_mesh,
    orchestrate_fleets_remote, plan_rollout, register_agent, register_fleet_agent, rollback_targets,
    run_deploy_agent_server, run_fleet_agent_server, run_fleet_mesh_coordinator, save_agent_registry,
    save_deploy_state, save_fleet_agent_registry, sign_deploy_bundle, validate_rollout_certification,
    DeployAgentServerOptions, DeployAgentTls, DeployState, RolloutOptions, RolloutStrategy,
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

fn fleet_agents_path() -> std::path::PathBuf {
    env::var("SPANDA_FLEET_AGENTS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| default_fleet_agents_path())
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
    "           spanda deploy plan [--json] [--version <ver>] [--sign-key <material>] [--bundle-out <file>] <file.sd>\n\
     spanda deploy rollout [--json] [--remote] [--require-certify] [--sign-key <material>] [--strategy all|canary|staged] [--canary-percent N] [--version <ver>] [--dry-run] <file.sd>\n\
     spanda deploy rollback [--json] [--remote] <file.sd>\n\
     spanda deploy status [--json]\n\
     spanda deploy agent start [--bind <addr>] [--target <Robot@Hardware>] [--token <t>] [--tls-cert <pem>] [--tls-key <pem>] [--require-hash] [--require-signature] [--require-certify] [--trust-key <material>]\n\
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
    let mut sign_key = None;
    let mut bundle_out = None;
    let mut file: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            "--version" if i + 1 < args.len() => {
                version = args[i + 1].clone();
                i += 1;
            }
            "--sign-key" if i + 1 < args.len() => {
                sign_key = Some(args[i + 1].clone());
                i += 1;
            }
            "--bundle-out" if i + 1 < args.len() => {
                bundle_out = Some(args[i + 1].clone());
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
    let mut bundle = build_deploy_bundle(&plan);
    if let Some(key) = sign_key.or_else(|| std::env::var("SPANDA_DEPLOY_SIGN_KEY").ok()) {
        if let Err(err) = sign_deploy_bundle(&mut bundle, &key) {
            eprintln!("Sign bundle failed: {err}");
            process::exit(1);
        }
    }
    if let Some(out) = bundle_out {
        if let Err(err) = fs::write(&out, serde_json::to_string_pretty(&bundle).unwrap()) {
            eprintln!("Write bundle failed: {err}");
            process::exit(1);
        }
        println!("Wrote signed deploy bundle to {out}");
    }
    if json {
        if bundle.signature.is_some() {
            println!("{}", serde_json::to_string_pretty(&bundle).unwrap());
        } else {
            println!("{}", serde_json::to_string_pretty(&plan).unwrap());
        }
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
        if let Some(signature) = &bundle.signature {
            println!("  artifact_signature: {signature}");
        }
        if let Some(proof) = &plan.certification_proof {
            let status = if proof.passed_strict {
                "passed (strict)"
            } else if proof.passed {
                "passed (relaxed)"
            } else {
                "failed"
            };
            println!("  certification_proof: {status} — {}", proof.summary);
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
    let mut sign_key = None;
    let mut require_certify = false;
    let mut file: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            "--dry-run" => dry_run = true,
            "--remote" => remote = true,
            "--require-certify" => require_certify = true,
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
            "--sign-key" if i + 1 < args.len() => {
                sign_key = Some(args[i + 1].clone());
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
    let mut bundle = build_deploy_bundle(&plan);
    if let Some(key) = sign_key.or_else(|| std::env::var("SPANDA_DEPLOY_SIGN_KEY").ok()) {
        if let Err(err) = sign_deploy_bundle(&mut bundle, &key) {
            eprintln!("Sign bundle failed: {err}");
            process::exit(1);
        }
    }
    let options = RolloutOptions {
        strategy,
        canary_percent,
        version: version.clone(),
        dry_run,
        require_certify,
        ..Default::default()
    };
    if let Err(err) = validate_rollout_certification(&plan, &options) {
        eprintln!("{err}");
        process::exit(1);
    }
    let result = if remote {
        let registry = load_agent_registry(&agents_path());
        execute_remote_rollout(&plan, &options, &registry, &bundle)
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
    let mut require_signature = false;
    let mut require_certify = false;
    let mut trusted_public_key = None;
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
            "--require-signature" => require_signature = true,
            "--require-certify" => require_certify = true,
            "--trust-key" if i + 1 < args.len() => {
                trusted_public_key = Some(args[i + 1].clone());
                i += 1;
            }
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
    if require_signature && trusted_public_key.is_none() {
        eprintln!("Missing --trust-key when --require-signature is set");
        process::exit(1);
    }
    if let Err(err) = run_deploy_agent_server(&DeployAgentServerOptions {
        bind: bind.clone(),
        target: target.clone(),
        token,
        state_path: default_agent_state_path(),
        tls,
        require_hash,
        require_signature,
        require_certify,
        trusted_public_key,
    }) {
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
    let mut remote = false;
    let mut mesh_url: Option<String> = None;
    let mut mesh_token: Option<String> = None;
    let mut file: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            "--remote" => remote = true,
            "--mesh-url" if i + 1 < args.len() => {
                mesh_url = Some(args[i + 1].clone());
                remote = true;
                i += 1;
            }
            "--mesh-token" if i + 1 < args.len() => {
                mesh_token = Some(args[i + 1].clone());
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
    let result = if let Some(ref url) = mesh_url {
        orchestrate_fleets_mesh(
            &program,
            &file,
            url,
            mesh_token.as_deref(),
        )
    } else if remote {
        let registry = load_fleet_agent_registry(&fleet_agents_path());
        orchestrate_fleets_remote(&program, &file, &registry)
    } else {
        orchestrate_fleets(&program, &file)
    };
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
            for delivery in &fleet.peer_deliveries {
                println!(
                    "    mesh: {} -> {} topic={} step={} delivered={}",
                    delivery.from_robot,
                    delivery.to_robot,
                    delivery.topic,
                    delivery.step,
                    delivery.delivered
                );
            }
            if remote || mesh_url.is_some() {
                println!(
                    "    remote: relayed={} failed={}",
                    fleet.remote_relayed, fleet.remote_failed
                );
            }
        }
    }
}

pub fn fleet_mesh_dispatch(args: &[String]) {
    if args.first().map(String::as_str) != Some("start") {
        eprintln!("Usage: spanda fleet mesh start [--bind <addr>] [--token <t>] [--tls-cert <pem>] [--tls-key <pem>]");
        process::exit(1);
    }
    let mut bind = "127.0.0.1:8767".to_string();
    let mut token = None;
    let mut tls_cert = None;
    let mut tls_key = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--bind" if i + 1 < args.len() => {
                bind = args[i + 1].clone();
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
            other => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
        }
        i += 1;
    }
    let tls = match (tls_cert, tls_key) {
        (Some(cert_path), Some(key_path)) => Some(DeployAgentTls {
            cert_path,
            key_path,
        }),
        (None, None) => None,
        _ => {
            eprintln!("Both --tls-cert and --tls-key are required for HTTPS fleet mesh");
            process::exit(1);
        }
    };
    if let Err(err) = run_fleet_mesh_coordinator(&bind, &mesh_registry_path(), token, tls) {
        eprintln!("Fleet mesh failed: {err}");
        process::exit(1);
    }
}

pub fn fleet_agent_dispatch(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: spanda fleet agent start|register|list");
        process::exit(1);
    }
    match args[0].as_str() {
        "start" => cmd_fleet_agent_start(&args[1..]),
        "register" => cmd_fleet_agent_register(&args[1..]),
        "list" => cmd_fleet_agent_list(&args[1..]),
        other => {
            eprintln!("Unknown fleet agent subcommand '{other}'");
            process::exit(1);
        }
    }
}

fn cmd_fleet_agent_start(args: &[String]) {
    let mut bind = "127.0.0.1:8766".to_string();
    let mut robot_name = String::new();
    let mut token = None;
    let mut tls_cert = None;
    let mut tls_key = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--bind" if i + 1 < args.len() => {
                bind = args[i + 1].clone();
                i += 1;
            }
            "--robot" if i + 1 < args.len() => {
                robot_name = args[i + 1].clone();
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
            other => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
        }
        i += 1;
    }
    if robot_name.is_empty() {
        eprintln!("Missing --robot <RobotName>");
        process::exit(1);
    }
    let tls = match (tls_cert, tls_key) {
        (Some(cert_path), Some(key_path)) => Some(DeployAgentTls {
            cert_path,
            key_path,
        }),
        (None, None) => None,
        _ => {
            eprintln!("Both --tls-cert and --tls-key are required for HTTPS fleet agents");
            process::exit(1);
        }
    };
    if let Err(err) = run_fleet_agent_server(
        &bind,
        &robot_name,
        token,
        &default_fleet_agent_state_path(),
        tls,
    ) {
        eprintln!("Fleet agent failed: {err}");
        process::exit(1);
    }
}

fn cmd_fleet_agent_register(args: &[String]) {
    let mut robot_name = None;
    let mut url = None;
    let mut token = None;
    for (idx, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "--token" if idx + 1 < args.len() => {
                token = Some(args[idx + 1].clone());
            }
            other if !other.starts_with('-') && robot_name.is_none() => {
                robot_name = Some(other.to_string());
            }
            other if !other.starts_with('-') && url.is_none() => url = Some(other.to_string()),
            other if other.starts_with('-') => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
            _ => {}
        }
    }
    let robot_name = robot_name.unwrap_or_else(|| {
        eprintln!("Missing robot name");
        process::exit(1);
    });
    let url = url.unwrap_or_else(|| {
        eprintln!("Missing agent URL (http(s)://host:port)");
        process::exit(1);
    });
    let path = fleet_agents_path();
    let mut registry = load_fleet_agent_registry(&path);
    if let Err(err) = register_fleet_agent(&mut registry, robot_name, url, token) {
        eprintln!("Register failed: {err}");
        process::exit(1);
    }
    if let Err(err) = save_fleet_agent_registry(&path, &registry) {
        eprintln!("Warning: could not save fleet agent registry: {err}");
        process::exit(1);
    }
    println!("Registered fleet agent in {}", path.display());
}

fn cmd_fleet_agent_list(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let registry = load_fleet_agent_registry(&fleet_agents_path());
    if json {
        println!("{}", serde_json::to_string_pretty(&registry).unwrap());
        return;
    }
    println!("Fleet agents ({})", fleet_agents_path().display());
    if registry.agents.is_empty() {
        println!("  (no agents registered)");
        return;
    }
    for entry in &registry.agents {
        let health = fleet_agent_health(entry).unwrap_or(false);
        println!(
            "  {} -> {} (healthy={health})",
            entry.robot_name, entry.url
        );
    }
}
