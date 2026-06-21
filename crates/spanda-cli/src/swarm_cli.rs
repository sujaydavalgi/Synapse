//! Swarm coordinator CLI (`spanda swarm coordinate`).

use spanda_core::{
    compile, coordinate_swarms, coordinate_swarms_mesh, default_swarm_state_path,
    load_swarm_state, save_swarm_state,
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

fn swarm_state_path() -> std::path::PathBuf {
    env::var("SPANDA_SWARM_STATE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| default_swarm_state_path())
}

pub fn swarm_dispatch(args: &[String]) {
    if args.is_empty() {
        usage();
        process::exit(1);
    }
    match args[0].as_str() {
        "coordinate" => cmd_coordinate(&args[1..]),
        other if !other.starts_with('-') => {
            eprintln!("Unknown swarm subcommand '{other}'");
            usage();
            process::exit(1);
        }
        _ => {
            usage();
            process::exit(1);
        }
    }
}

pub fn swarm_usage_lines() -> &'static str {
    "           spanda swarm coordinate [--json] [--mesh-url <http(s)://host:port>] [--mesh-token <t>] <file.sd>"
}

fn usage() {
    eprintln!("Usage:\n{}", swarm_usage_lines());
}

fn cmd_coordinate(args: &[String]) {
    let mut json = false;
    let mut mesh_url: Option<String> = None;
    let mut mesh_token: Option<String> = None;
    let mut file: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            "--mesh-url" if i + 1 < args.len() => {
                mesh_url = Some(args[i + 1].clone());
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
    let path = swarm_state_path();
    let mut state = load_swarm_state(&path);
    let result = if let Some(ref url) = mesh_url {
        coordinate_swarms_mesh(
            &program,
            &file,
            &mut state,
            url,
            mesh_token.as_deref(),
        )
    } else {
        coordinate_swarms(&program, &file, &mut state)
    };
    if let Err(err) = save_swarm_state(&path, &state) {
        eprintln!("Warning: could not save swarm state: {err}");
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else {
        println!("Swarm coordination for {file}");
        for swarm in &result.swarms {
            println!(
                "  swarm {} -> fleet {} ({}, cursor={})",
                swarm.swarm_name, swarm.fleet_name, swarm.policy, swarm.round_robin_cursor
            );
            if let Some(active) = &swarm.active_member {
                println!("    active_member: {active}");
            }
            for member in &swarm.members {
                println!(
                    "    {} mission={:?} state={} step='{}'",
                    member.robot_name, member.mission_name, member.mission_state, member.current_step
                );
            }
            for delivery in &swarm.peer_deliveries {
                println!(
                    "    follow: {} -> {} step={}",
                    delivery.from_robot, delivery.to_robot, delivery.step
                );
            }
            if mesh_url.is_some() {
                println!(
                    "    mesh: relayed={} failed={}",
                    swarm.remote_relayed, swarm.remote_failed
                );
            }
        }
    }
    if !result.success {
        process::exit(1);
    }
    let _ = io::stdout().flush();
}
