//! OTA deploy CLI handlers (`spanda deploy plan|rollout|rollback|status|agent`).

use spanda_ast::nodes::Program;
use spanda_deploy_http::{ensure_agent_auth, DeployAgentTls};
use spanda_driver::{build_deploy_plan, compile};
use spanda_fleet::{
    agent_health as fleet_agent_health, agent_readiness as fleet_agent_readiness,
    default_fleet_agents_path, fleet_agent_state_path_for, load_fleet_agent_registry,
    lookup_fleet_agent, mesh_registry_path, orchestrate_fleets, orchestrate_fleets_mesh,
    orchestrate_fleets_remote, register_fleet_agent, run_fleet_agent_server,
    run_fleet_mesh_coordinator, save_fleet_agent_registry,
};
use spanda_ota::{
    agent_health, agent_readiness, agent_state_path_for, apply_rollout, build_deploy_bundle,
    default_agents_path, default_state_path, execute_remote_rollback, execute_remote_rollout,
    load_agent_registry, load_deploy_state, lookup_agent, plan_rollout, register_agent,
    rollback_targets, run_deploy_agent_server, save_agent_registry, save_deploy_state,
    sign_deploy_bundle, validate_rollout_certification, DeployAgentServerOptions, DeployState,
    RolloutOptions, RolloutResult, RolloutStrategy,
};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

fn read_source(path: &str) -> String {
    // Description:
    //     Read source.
    //
    // Inputs:
    //     path: &str
    //         Caller-supplied path.
    //
    // Outputs:
    //     result: String
    //         Return value from `read_source`.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::read_source(path);

    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading {path}: {e}");
        process::exit(1);
    })
}

fn parse_program(source: &str, file: &str) -> Program {
    // Description:
    //     Parse program.
    //
    // Inputs:
    //     source: &str
    //         Caller-supplied source.
    //     file: &str
    //         Caller-supplied file.
    //
    // Outputs:
    //     result: Program
    //         Return value from `parse_program`.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::parse_program(source, file);

    compile(source)
        .unwrap_or_else(|e| {
            eprintln!("Error compiling {file}: {e}");
            process::exit(1);
        })
        .program
}

fn state_path() -> std::path::PathBuf {
    // Description:
    //     State path.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: std::path::PathBuf
    //         Return value from `state_path`.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::state_path();

    env::var("SPANDA_DEPLOY_STATE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| default_state_path())
}

fn agents_path() -> std::path::PathBuf {
    // Description:
    //     Agents path.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: std::path::PathBuf
    //         Return value from `agents_path`.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::agents_path();

    env::var("SPANDA_DEPLOY_AGENTS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| default_agents_path())
}

fn fleet_agents_path() -> std::path::PathBuf {
    // Description:
    //     Fleet agents path.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: std::path::PathBuf
    //         Return value from `fleet_agents_path`.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::fleet_agents_path();

    env::var("SPANDA_FLEET_AGENTS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| default_fleet_agents_path())
}

pub fn deploy_dispatch(args: &[String]) {
    // Description:
    //     Deploy dispatch.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::deploy_dispatch(args);

    if args.is_empty() {
        usage();
        process::exit(1);
    }
    match args[0].as_str() {
        "plan" => cmd_plan(&args[1..]),
        "rollout" => cmd_rollout(&args[1..]),
        "rollback" => cmd_rollback(&args[1..]),
        "status" => cmd_status(&args[1..]),
        "gate" => cmd_gate(&args[1..]),
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
    // Description:
    //     Deploy usage lines.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: &'static str
    //         Return value from `deploy_usage_lines`.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::deploy_usage_lines();

    "           spanda deploy plan [--json] [--version <ver>] [--sign-key <material>] [--bundle-out <file>] <file.sd>\n\
     spanda deploy rollout [--json] [--remote] [--require-certify] [--sign-key <material>] [--strategy all|canary|staged] [--canary-percent N] [--version <ver>] [--dry-run] <file.sd>\n\
     spanda deploy rollback [--json] [--remote] <file.sd>\n\
     spanda deploy status [--json]\n\
     spanda deploy gate [--json] [--policy default|production] [--config <spanda.toml>] <file.sd>\n\
     spanda deploy agent start [--bind <addr>] [--target <Robot@Hardware>] [--token <t>] [--tls-cert <pem>] [--tls-key <pem>] [--require-hash] [--require-signature] [--require-certify] [--trust-key <material>]\n\
     spanda deploy agent register <Robot@Hardware> <http(s)://host:port> [--token <t>]\n\
     spanda deploy agent list [--json]\n\
     spanda deploy agent readiness <Robot@Hardware> [--runtime] [--inject-health-faults] [--json]\n\
     spanda deploy --target wasm|native [--out <file>] [--target-triple <triple>] [--hal-profile <name>] <file.sd>"
}

fn usage() {
    // Description:
    //     Usage.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::usage();

    eprintln!("Usage:\n{}", deploy_usage_lines());
}

fn cmd_plan(args: &[String]) {
    // Description:
    //     Cmd plan.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::cmd_plan(args);

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
    // Description:
    //     Cmd rollout.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::cmd_rollout(args);

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
    // Description:
    //     Cmd rollback.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::cmd_rollback(args);

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
    // Description:
    //     Cmd agent.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::cmd_agent(args);

    if args.is_empty() {
        eprintln!("Usage: spanda deploy agent start|register|list|readiness");
        process::exit(1);
    }
    match args[0].as_str() {
        "start" => cmd_agent_start(&args[1..]),
        "register" => cmd_agent_register(&args[1..]),
        "list" => cmd_agent_list(&args[1..]),
        "readiness" => cmd_agent_readiness(&args[1..]),
        other => {
            eprintln!("Unknown deploy agent subcommand '{other}'");
            process::exit(1);
        }
    }
}

fn cmd_agent_start(args: &[String]) {
    // Description:
    //     Cmd agent start.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::cmd_agent_start(args);

    let mut bind = "127.0.0.1:8765".to_string();
    let mut target = String::new();
    let mut token = None;
    let mut tls_cert = None;
    let mut tls_key = None;
    let mut require_hash = false;
    let mut require_signature = false;
    let mut require_certify = false;
    let mut trusted_public_key = None;
    let mut allow_unauthenticated = false;
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
            "--allow-unauthenticated" => allow_unauthenticated = true,
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
    if let Err(err) = ensure_agent_auth(&bind, &token, allow_unauthenticated) {
        eprintln!("{err}");
        process::exit(1);
    }
    if let Err(err) = run_deploy_agent_server(&DeployAgentServerOptions {
        bind: bind.clone(),
        target: target.clone(),
        token,
        state_path: agent_state_path_for(&target),
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
    // Description:
    //     Cmd agent register.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::cmd_agent_register(args);

    let mut target = None;
    let mut url = None;
    let mut token = None;
    for (idx, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "--token" if idx + 1 < args.len() => {
                token = Some(args[idx + 1].clone());
            }
            other if !other.starts_with('-') && target.is_none() => {
                target = Some(other.to_string())
            }
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
    // Description:
    //     Cmd agent list.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::cmd_agent_list(args);

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

fn cmd_agent_readiness(args: &[String]) {
    // Description:
    //     Cmd agent readiness.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::cmd_agent_readiness(args);

    let mut runtime = false;
    let mut inject_health_faults = false;
    let mut json = false;
    let mut target: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--runtime" => runtime = true,
            "--inject-health-faults" => {
                runtime = true;
                inject_health_faults = true;
            }
            "--json" => json = true,
            other if !other.starts_with('-') && target.is_none() => {
                target = Some(other.to_string())
            }
            other => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
        }
        i += 1;
    }
    let target = target.unwrap_or_else(|| {
        eprintln!("Missing target Robot@Hardware");
        process::exit(1);
    });
    let registry = load_agent_registry(&agents_path());
    let entry = lookup_agent(&registry, &target).unwrap_or_else(|| {
        eprintln!("No deploy agent registered for target {target}");
        process::exit(1);
    });
    let body = agent_readiness(entry, runtime, inject_health_faults).unwrap_or_else(|err| {
        eprintln!("Agent readiness failed: {err}");
        process::exit(1);
    });
    if json {
        println!("{}", serde_json::to_string_pretty(&body).unwrap());
    } else {
        let mission_ready = body
            .get("mission_ready")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let score = body
            .get("readiness")
            .and_then(|r| r.get("score"))
            .and_then(|s| s.get("total"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        println!("Agent readiness for {target}");
        println!(
            "Mission Ready: {}",
            if mission_ready { "YES" } else { "NO" }
        );
        println!("Score: {score}/100");
    }
    let mission_ready = body
        .get("mission_ready")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !mission_ready {
        process::exit(1);
    }
}

fn cmd_status(args: &[String]) {
    // Description:
    //     Cmd status.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::cmd_status(args);

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

fn cmd_gate(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let mut policy_name = "default".to_string();
    let mut config_path: Option<String> = None;
    let mut file: Option<String> = None;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {}
            "--policy" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--policy requires default or production");
                    process::exit(1);
                }
                policy_name = args[i].clone();
            }
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
        eprintln!("Usage: spanda deploy gate [--json] [--policy default|production] [--config <spanda.toml>] <file.sd>");
        process::exit(1);
    });
    let source = read_source(&file);
    let program = parse_program(&source, &file);
    let system_config = crate::config_load::load_system_config(
        std::path::Path::new(&file),
        config_path.as_deref().map(std::path::Path::new),
    );
    crate::config_load::ensure_config_valid(system_config.as_ref().map(|arc| arc.as_ref()));
    let mut options = spanda_readiness::readiness_options_from_flags(
        &program,
        system_config
            .as_ref()
            .and_then(|cfg| spanda_config::default_verify_target(cfg.as_ref())),
        false,
        false,
        false,
        false,
    );
    options.system_config = system_config;
    let policy = if policy_name == "production" {
        spanda_readiness::DeploymentGatePolicy::production()
    } else {
        spanda_readiness::DeploymentGatePolicy::default()
    };
    let report = spanda_readiness::evaluate_deployment_gates(&program, &source, &options, &policy);
    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        println!("Deployment gates for {file} (policy: {policy_name})");
        for gate in &report.gates {
            let tag = if gate.passed { "PASS" } else { "FAIL" };
            println!("  [{tag}] {} — {}", gate.name, gate.message);
        }
        println!(
            "\nGate check: {}",
            if report.passed { "PASSED" } else { "BLOCKED" }
        );
    }
    if !report.passed {
        process::exit(1);
    }
}

fn print_rollout(result: &RolloutResult, json: bool) {
    // Description:
    //     Print rollout.
    //
    // Inputs:
    //     resul: &RolloutResult
    //         Caller-supplied resul.
    //     json: bool
    //         Caller-supplied json.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::print_rollout(resul, json);

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
    // Description:
    //     Fleet orchestrate dispatch.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::fleet_orchestrate_dispatch(args);

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
        orchestrate_fleets_mesh(&program, &file, url, mesh_token.as_deref())
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
    // Description:
    //     Fleet mesh dispatch.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::fleet_mesh_dispatch(args);

    if args.first().map(String::as_str) != Some("start") {
        eprintln!("Usage: spanda fleet mesh start [--bind <addr>] [--token <t>] [--tls-cert <pem>] [--tls-key <pem>]");
        process::exit(1);
    }
    let mut bind = "127.0.0.1:8767".to_string();
    let mut token = None;
    let mut tls_cert = None;
    let mut tls_key = None;
    let mut allow_unauthenticated = false;
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
            "--allow-unauthenticated" => allow_unauthenticated = true,
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
    if let Err(err) = ensure_agent_auth(&bind, &token, allow_unauthenticated) {
        eprintln!("{err}");
        process::exit(1);
    }
    if let Err(err) = run_fleet_mesh_coordinator(&bind, &mesh_registry_path(), token, tls) {
        eprintln!("Fleet mesh failed: {err}");
        process::exit(1);
    }
}

pub fn fleet_agent_dispatch(args: &[String]) {
    // Description:
    //     Fleet agent dispatch.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::fleet_agent_dispatch(args);

    if args.is_empty() {
        eprintln!("Usage: spanda fleet agent start|register|list|readiness");
        process::exit(1);
    }
    match args[0].as_str() {
        "start" => cmd_fleet_agent_start(&args[1..]),
        "register" => cmd_fleet_agent_register(&args[1..]),
        "list" => cmd_fleet_agent_list(&args[1..]),
        "readiness" => cmd_fleet_agent_readiness(&args[1..]),
        other => {
            eprintln!("Unknown fleet agent subcommand '{other}'");
            process::exit(1);
        }
    }
}

fn cmd_fleet_agent_start(args: &[String]) {
    // Description:
    //     Cmd fleet agent start.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::cmd_fleet_agent_start(args);

    let mut bind = "127.0.0.1:8766".to_string();
    let mut robot_name = String::new();
    let mut token = None;
    let mut tls_cert = None;
    let mut tls_key = None;
    let mut allow_unauthenticated = false;
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
            "--allow-unauthenticated" => allow_unauthenticated = true,
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
    if let Err(err) = ensure_agent_auth(&bind, &token, allow_unauthenticated) {
        eprintln!("{err}");
        process::exit(1);
    }
    if let Err(err) = run_fleet_agent_server(
        &bind,
        &robot_name,
        token,
        &fleet_agent_state_path_for(&robot_name),
        tls,
    ) {
        eprintln!("Fleet agent failed: {err}");
        process::exit(1);
    }
}

fn cmd_fleet_agent_register(args: &[String]) {
    // Description:
    //     Cmd fleet agent register.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::cmd_fleet_agent_register(args);

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
    // Description:
    //     Cmd fleet agent list.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::cmd_fleet_agent_list(args);

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
        println!("  {} -> {} (healthy={health})", entry.robot_name, entry.url);
    }
}

fn cmd_fleet_agent_readiness(args: &[String]) {
    // Description:
    //     Cmd fleet agent readiness.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::deploy_ota::cmd_fleet_agent_readiness(args);

    let mut runtime = false;
    let mut inject_health_faults = false;
    let mut json = false;
    let mut robot: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--runtime" => runtime = true,
            "--inject-health-faults" => {
                runtime = true;
                inject_health_faults = true;
            }
            "--json" => json = true,
            other if !other.starts_with('-') && robot.is_none() => robot = Some(other.to_string()),
            other => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
        }
        i += 1;
    }
    let robot = robot.unwrap_or_else(|| {
        eprintln!("Missing robot name");
        process::exit(1);
    });
    let registry = load_fleet_agent_registry(&fleet_agents_path());
    let entry = lookup_fleet_agent(&registry, &robot).unwrap_or_else(|| {
        eprintln!("No fleet agent registered for robot {robot}");
        process::exit(1);
    });
    let body = fleet_agent_readiness(entry, runtime, inject_health_faults).unwrap_or_else(|err| {
        eprintln!("Fleet agent readiness failed: {err}");
        process::exit(1);
    });
    if json {
        println!("{}", serde_json::to_string_pretty(&body).unwrap());
    } else {
        let mission_ready = body
            .get("mission_ready")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let score = body
            .get("readiness")
            .and_then(|r| r.get("score"))
            .and_then(|s| s.get("total"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        println!("Fleet agent readiness for {robot}");
        println!(
            "Mission Ready: {}",
            if mission_ready { "YES" } else { "NO" }
        );
        println!("Score: {score}/100");
    }
    let mission_ready = body
        .get("mission_ready")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !mission_ready {
        process::exit(1);
    }
}
