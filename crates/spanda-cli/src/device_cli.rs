//! CLI commands for device discovery, provisioning, and pool management.

use spanda_config::{
    discover_matches, export_device_mapping_json, persist_device_record, run_discovery_transports,
    run_provision_workflow, scan_subnet, AssignDeviceOptions, ConfigResolver, DiscoveryOptions,
    SpandaManifest,
};
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
        .with_validation(false)
        .resolve_from_dir(root)
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            process::exit(1);
        })
}

/// Dispatch `spanda device` subcommands.
pub fn device_dispatch(args: &[String]) {
    let sub = args.first().map(String::as_str).unwrap_or("");
    match sub {
        "discover" => cmd_discover(&args[1..]),
        "inspect" => cmd_inspect(&args[1..]),
        "provision" => cmd_provision(&args[1..]),
        "assign" => cmd_assign(&args[1..]),
        "unassign" => cmd_unassign(&args[1..]),
        "quarantine" => cmd_quarantine(&args[1..]),
        "trust" => cmd_trust(&args[1..]),
        "retire" => cmd_retire(&args[1..]),
        _ => {
            eprintln!(
                "Usage:\n  \
                 spanda device discover [--subnet CIDR] [--transport NAME] [--json] [--config <spanda.toml>]\n  \
                 spanda device inspect <id> [--json] [--config <spanda.toml>]\n  \
                 spanda device provision <id> [--robot ROBOT] [--json] [--config <spanda.toml>]\n  \
                 spanda device assign <id> --robot ROBOT [--logical NAME] [--json] [--config <spanda.toml>]\n  \
                 spanda device unassign <id> [--json] [--config <spanda.toml>]\n  \
                 spanda device quarantine <id> [--json] [--write] [--config <spanda.toml>]\n  \
                 spanda device trust <id> [--json] [--write] [--config <spanda.toml>]\n  \
                 spanda device retire <id> [--json] [--config <spanda.toml>]"
            );
            process::exit(1);
        }
    }
}

fn cmd_discover(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let mut subnet: Option<String> = None;
    let mut transport: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--subnet" {
            subnet = args.get(i + 1).cloned();
            i += 2;
            continue;
        }
        if args[i] == "--transport" {
            transport = args.get(i + 1).cloned();
            i += 2;
            continue;
        }
        i += 1;
    }
    let root = project_root(args);
    let resolved = load_resolved(&root);
    let options = DiscoveryOptions {
        subnet: subnet.clone(),
        timeout_ms: Some(200),
        transports: transport.map(|t| vec![t]).unwrap_or_default(),
    };
    let transport_results = run_discovery_transports(&options);
    let probes = subnet
        .as_deref()
        .map(|cidr| scan_subnet(cidr, &[], 200))
        .unwrap_or_default();
    let matches = discover_matches(&resolved.device_registry, &probes);
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "devices": resolved.device_registry.devices,
                "probes": probes,
                "matches": matches,
                "transports": transport_results,
            }))
            .unwrap()
        );
        return;
    }
    println!(
        "Configured devices: {}",
        resolved.device_registry.devices.len()
    );
    for device in &resolved.device_registry.devices {
        println!(
            "  {} logical={:?} ip={:?} lifecycle={:?}",
            device.id, device.logical_name, device.ip_address, device.lifecycle_state
        );
    }
    for result in transport_results {
        match result {
            Ok(r) => println!("\nTransport {}: {} matches", r.transport, r.matches.len()),
            Err(e) => eprintln!("\nTransport error: {e}"),
        }
    }
    if let Some(ref cidr) = subnet {
        println!("\nScan {cidr}: {} hosts probed", probes.len());
        for probe in probes.iter().filter(|p| p.reachable) {
            println!("  reachable {} ports={:?}", probe.ip, probe.open_ports);
        }
        if !matches.is_empty() {
            println!("\nMatched configured devices:");
            for m in &matches {
                println!("  {} <= {}", m.device_id, m.configured_ip);
            }
        }
    }
}

fn device_id_from_args(args: &[String]) -> String {
    args.iter()
        .find(|a| !a.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| {
            eprintln!("Missing device id");
            process::exit(1);
        })
}

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn cmd_inspect(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let device_id = device_id_from_args(args);
    let root = project_root(args);
    let resolved = load_resolved(&root);
    let device = resolved.device_registry.get(&device_id).unwrap_or_else(|| {
        eprintln!("Device '{device_id}' not found");
        process::exit(1);
    });
    if json {
        println!("{}", serde_json::to_string_pretty(device).unwrap());
    } else {
        println!("Device: {}", device.id);
        if let Some(ref logical) = device.logical_name {
            println!("Logical name: {logical}");
        }
        println!("Type: {}", device.device_type);
        if let Some(ref lifecycle) = device.lifecycle_state {
            println!("Lifecycle: {lifecycle}");
        }
        if let Some(ref provider) = device.provider {
            println!("Provider: {provider}");
        }
        if let Some(ref ip) = device.ip_address {
            println!("IP: {ip}");
        }
        if let Some(ref mac) = device.mac_address {
            println!("MAC: {mac}");
        }
        if let Some(ref endpoint) = device.endpoint_url {
            println!("Endpoint: {endpoint}");
        }
        if let Some(ref trust) = device.trust_level {
            println!("Trust: {trust}");
        }
        if !device.capabilities.is_empty() {
            println!("Capabilities: {}", device.capabilities.join(", "));
        }
    }
}

fn cmd_provision(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let device_id = device_id_from_args(args);
    let robot = flag_value(args, "--robot");
    let root = project_root(args);
    let resolved = load_resolved(&root);
    let report = run_provision_workflow(&device_id, &resolved.device_registry, robot.as_deref());
    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        println!(
            "Provision {}: {}",
            device_id,
            if report.ready { "READY" } else { "NOT READY" }
        );
        for step in &report.steps {
            let mark = if step.passed { "ok" } else { "FAIL" };
            println!("  [{mark}] {} — {}", step.step, step.message);
        }
    }
    if !report.ready {
        process::exit(1);
    }
}

fn cmd_assign(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let write_disk = args.iter().any(|a| a == "--write");
    let device_id = device_id_from_args(args);
    let robot = flag_value(args, "--robot").unwrap_or_else(|| {
        eprintln!("--robot is required");
        process::exit(1);
    });
    let logical = flag_value(args, "--logical");
    let root = project_root(args);
    let mut resolved = load_resolved(&root);
    let result = resolved
        .device_registry
        .assign_device(
            &device_id,
            &AssignDeviceOptions {
                robot_id: robot,
                logical_name: logical,
                ..Default::default()
            },
        )
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            process::exit(1);
        });
    if write_disk {
        let manifest = SpandaManifest::load_from_dir(&root).unwrap_or_else(|e| {
            eprintln!("{e}");
            process::exit(1);
        });
        if let Some(device) = resolved.device_registry.get(&device_id) {
            if let Err(e) = persist_device_record(&root, &manifest, device) {
                eprintln!("persist: {e}");
                process::exit(1);
            }
        }
    }
    let mapping = export_device_mapping_json(&resolved.device_registry, &resolved.logical_map);
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "result": result,
                "mapping": mapping,
            }))
            .unwrap()
        );
    } else {
        println!("{} — {}", result.device_id, result.message);
        println!("Mapping sensors: {}", resolved.logical_map.sensors.len());
    }
}

fn cmd_unassign(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let device_id = device_id_from_args(args);
    let root = project_root(args);
    let mut resolved = load_resolved(&root);
    let result = resolved
        .device_registry
        .unassign_device(&device_id)
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            process::exit(1);
        });
    if json {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else {
        println!("{} — {}", result.device_id, result.message);
    }
}

fn cmd_quarantine(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let write_disk = args.iter().any(|a| a == "--write");
    let device_id = device_id_from_args(args);
    let root = project_root(args);
    let mut resolved = load_resolved(&root);
    let result = resolved
        .device_registry
        .quarantine_device(&device_id)
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            process::exit(1);
        });
    if write_disk {
        persist_device_if_present(&root, &resolved, &device_id);
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else {
        println!("{} — {}", result.device_id, result.message);
    }
}

fn cmd_trust(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let write_disk = args.iter().any(|a| a == "--write");
    let device_id = device_id_from_args(args);
    let root = project_root(args);
    let mut resolved = load_resolved(&root);
    let result = resolved
        .device_registry
        .trust_device(&device_id)
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            process::exit(1);
        });
    if write_disk {
        persist_device_if_present(&root, &resolved, &device_id);
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else {
        println!("{} — {}", result.device_id, result.message);
    }
}

fn persist_device_if_present(
    root: &PathBuf,
    resolved: &spanda_config::ResolvedSystemConfig,
    device_id: &str,
) {
    let manifest = SpandaManifest::load_from_dir(root).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    });
    if let Some(device) = resolved.device_registry.get(device_id) {
        if let Err(e) = persist_device_record(root, &manifest, device) {
            eprintln!("persist: {e}");
            process::exit(1);
        }
    }
}

fn cmd_retire(args: &[String]) {
    let json = args.iter().any(|a| a == "--json");
    let device_id = device_id_from_args(args);
    let root = project_root(args);
    let mut resolved = load_resolved(&root);
    let result = resolved
        .device_registry
        .retire_device(&device_id)
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            process::exit(1);
        });
    if json {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else {
        println!("{} — {}", result.device_id, result.message);
    }
}
