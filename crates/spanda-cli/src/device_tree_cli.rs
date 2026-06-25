//! CLI commands for device tree inspection and mapping verification.

use spanda_config::{generate_report_bundle, ConfigResolver, SpandaManifest};
use std::env;
use std::path::PathBuf;
use std::process;

fn project_root(args: &[String]) -> PathBuf {
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

fn load_resolved(root: &PathBuf) -> spanda_config::ResolvedSystemConfig {
    ConfigResolver::new()
        .with_validation(true)
        .resolve_from_dir(root)
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            process::exit(1);
        })
}

/// Dispatch `spanda device-tree` subcommands.
pub fn device_tree_dispatch(args: &[String]) {
    let sub = args.first().map(String::as_str).unwrap_or("");
    match sub {
        "inspect" => cmd_inspect(&args[1..]),
        "graph" => cmd_graph(&args[1..]),
        _ => {
            eprintln!(
                "Usage:\n  \
                 spanda device-tree inspect <robot-id> [--json] [--config <spanda.toml>]\n  \
                 spanda device-tree graph [--json] [--config <spanda.toml>]"
            );
            process::exit(1);
        }
    }
}

fn cmd_inspect(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let robot_id = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Missing robot id");
            process::exit(1);
        });
    let root = project_root(args);
    let resolved = load_resolved(&root);
    let robot = resolved.device_tree.robot(&robot_id).unwrap_or_else(|| {
        eprintln!("Robot '{robot_id}' not found in device tree");
        process::exit(1);
    });
    if json {
        println!("{}", serde_json::to_string_pretty(robot).unwrap());
    } else {
        println!("Robot: {}", robot.id);
        if let Some(ref model) = robot.model {
            println!("Model: {model}");
        }
        if let Some(ref profile) = robot.hardware_profile {
            println!("Hardware profile: {profile}");
        }
        if let Some(ref compute) = robot.compute {
            println!("Compute: {} [{}]", compute.id, compute.compute_type);
            for device in &compute.devices {
                println!(
                    "  - {} ({}) caps=[{}]",
                    device.id,
                    device.device_type,
                    device.capabilities.join(", ")
                );
            }
        }
    }
}

fn cmd_graph(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let root = project_root(args);
    let resolved = load_resolved(&root);
    let bundle = generate_report_bundle(&resolved);
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&bundle.device_hierarchy).unwrap()
        );
    } else {
        println!("Device tree:");
        for line in &bundle.device_hierarchy {
            println!("{line}");
        }
    }
}

/// `spanda map verify <file.sd> [--config <spanda.toml>]`
pub fn cmd_map_verify(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let root = project_root(args);
    let resolved = load_resolved(&root);
    let issues = resolved.logical_map.verify();
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&issues).unwrap_or_else(|_| "[]".into())
        );
    } else if issues.is_empty() {
        println!("Logical-to-physical mapping: OK");
    } else {
        for issue in &issues {
            println!("{issue}");
        }
    }
    if !issues.is_empty() {
        process::exit(1);
    }
}
