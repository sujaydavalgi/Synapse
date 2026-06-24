//! CLI commands for querying the persistent telemetry store.

use spanda_telemetry_store::{
    global_store, resolve_store_path, TelemetryEvent, TelemetryQuery, TelemetryStats,
};
use std::fs;
use std::path::PathBuf;
use std::process;

pub fn cmd_telemetry(sub: &str, args: &[String]) {
    match sub {
        "list" => cmd_list(args),
        "latest" => cmd_latest(args),
        "export" => cmd_export(args),
        "stats" => cmd_stats(args),
        "heartbeats" => cmd_heartbeats(args),
        "devices" => cmd_devices(args),
        other => {
            eprintln!("Unknown telemetry subcommand: {other}");
            usage();
            process::exit(1);
        }
    }
}

fn usage() {
    eprintln!(
        "Usage:\n\
         spanda telemetry list [--device <id>] [--sensor <id>] [--task <name>] [--kind device|sensor|heartbeat|device_heartbeat|health] [--since <ms>] [--limit <n>] [--json]\n\
         spanda telemetry latest [--device <id> [--metric <name>] | --sensor <id> | --task <name>] [--json]\n\
         spanda telemetry export [--out <file.jsonl>]\n\
         spanda telemetry stats [--json]\n\
         spanda telemetry heartbeats [--json]\n\
         spanda telemetry devices [--json]"
    );
}

fn cmd_list(args: &[String]) {
    let parsed = parse_query_args(args);
    let store = global_store().lock().unwrap();
    let events = store.query(&parsed.query).unwrap_or_else(|error| {
        eprintln!("telemetry list failed: {error}");
        process::exit(1);
    });
    if parsed.json {
        println!("{}", serde_json::to_string_pretty(&events).unwrap());
        return;
    }
    if events.is_empty() {
        println!("No telemetry events found in {}", store.store_path().display());
        return;
    }
    for event in events {
        print_event(&event);
    }
}

fn cmd_latest(args: &[String]) {
    let parsed = parse_query_args(args);
    let store = global_store().lock().unwrap();
    let event: Option<TelemetryEvent> = if let Some(device_id) = &parsed.device_id {
        if let Some(metric) = &parsed.metric {
            store
                .latest_device(device_id, metric)
                .unwrap_or_else(|error| {
                    eprintln!("telemetry latest failed: {error}");
                    process::exit(1);
                })
        } else {
            store
                .heartbeat_index()
                .devices
                .get(device_id)
                .copied()
                .map(|timestamp_ms| TelemetryEvent::DeviceHeartbeat {
                    device_id: device_id.clone(),
                    timestamp_ms,
                    robot_id: None,
                    protocol: None,
                })
        }
    } else if let Some(sensor_id) = &parsed.sensor_id {
        store.latest_sensor(sensor_id).unwrap_or_else(|error| {
            eprintln!("telemetry latest failed: {error}");
            process::exit(1);
        })
    } else if let Some(task_name) = &parsed.task_name {
        let heartbeat = store.heartbeat_index().tasks.get(task_name).copied();
        heartbeat.map(|timestamp_ms| TelemetryEvent::Heartbeat {
            task_name: task_name.clone(),
            timestamp_ms,
            robot_id: None,
        })
    } else {
        eprintln!("telemetry latest requires --device, --sensor, or --task");
        process::exit(1);
    };
    if parsed.json {
        println!("{}", serde_json::to_string_pretty(&event).unwrap());
        return;
    }
    match event {
        Some(event) => print_event(&event),
        None => println!("No matching telemetry event found"),
    }
}

fn cmd_export(args: &[String]) {
    let mut out: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                i += 1;
                out = args.get(i).map(PathBuf::from);
            }
            other => {
                eprintln!("Unknown telemetry export flag: {other}");
                usage();
                process::exit(1);
            }
        }
        i += 1;
    }
    let store = global_store().lock().unwrap();
    let events = store.read_all().unwrap_or_else(|error| {
        eprintln!("telemetry export failed: {error}");
        process::exit(1);
    });
    let out = out.unwrap_or_else(resolve_store_path);
    if let Some(parent) = out.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let mut lines = String::new();
    for event in events {
        lines.push_str(&serde_json::to_string(&event).unwrap());
        lines.push('\n');
    }
    fs::write(&out, lines).unwrap_or_else(|error| {
        eprintln!("telemetry export failed: {error}");
        process::exit(1);
    });
    println!("Exported telemetry to {}", out.display());
}

fn cmd_stats(args: &[String]) {
    let json = args.iter().any(|arg| arg == "--json");
    let store = global_store().lock().unwrap();
    let stats: TelemetryStats = store.stats().unwrap_or_else(|error| {
        eprintln!("telemetry stats failed: {error}");
        process::exit(1);
    });
    if json {
        println!("{}", serde_json::to_string_pretty(&stats).unwrap());
        return;
    }
    println!("Store: {}", store.store_path().display());
    println!("Total events: {}", stats.total_events);
    println!("Device events: {}", stats.device_events);
    println!("Sensor events: {}", stats.sensor_events);
    println!("Heartbeat events: {}", stats.heartbeat_events);
    println!("Device heartbeat events: {}", stats.device_heartbeat_events);
    println!("Health events: {}", stats.health_events);
    println!("Tracked tasks: {}", stats.tracked_tasks);
    println!("Tracked devices: {}", stats.tracked_devices);
}

fn cmd_devices(args: &[String]) {
    let json = args.iter().any(|arg| arg == "--json");
    let store = global_store().lock().unwrap();
    let index = store.heartbeat_index();
    if json {
        println!("{}", serde_json::to_string_pretty(&index.devices).unwrap());
        return;
    }
    if index.devices.is_empty() {
        println!("No device heartbeats recorded");
        return;
    }
    for (device, timestamp_ms) in &index.devices {
        println!("{device}: {timestamp_ms} ms");
    }
}

fn cmd_heartbeats(args: &[String]) {
    let json = args.iter().any(|arg| arg == "--json");
    let store = global_store().lock().unwrap();
    let index = store.heartbeat_index();
    if json {
        println!("{}", serde_json::to_string_pretty(index).unwrap());
        return;
    }
    if index.tasks.is_empty() && index.devices.is_empty() {
        println!("No task heartbeats recorded");
        return;
    }
    if !index.tasks.is_empty() {
        println!("Tasks:");
        for (task, timestamp_ms) in &index.tasks {
            println!("  {task}: {timestamp_ms} ms");
        }
    }
    if !index.devices.is_empty() {
        println!("Devices:");
        for (device, timestamp_ms) in &index.devices {
            println!("  {device}: {timestamp_ms} ms");
        }
    }
}

struct ParsedQueryArgs {
    query: TelemetryQuery,
    json: bool,
    device_id: Option<String>,
    sensor_id: Option<String>,
    task_name: Option<String>,
    metric: Option<String>,
}

fn parse_query_args(args: &[String]) -> ParsedQueryArgs {
    let mut query = TelemetryQuery::default();
    let mut json = false;
    let mut device_id = None;
    let mut sensor_id = None;
    let mut task_name = None;
    let mut metric = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            "--device" => {
                i += 1;
                device_id = args.get(i).cloned();
                query.device_id = device_id.clone();
            }
            "--sensor" => {
                i += 1;
                sensor_id = args.get(i).cloned();
                query.sensor_id = sensor_id.clone();
            }
            "--task" => {
                i += 1;
                task_name = args.get(i).cloned();
                query.task_name = task_name.clone();
            }
            "--metric" => {
                i += 1;
                metric = args.get(i).cloned();
            }
            "--kind" => {
                i += 1;
                query.kind = args.get(i).cloned();
            }
            "--since" => {
                i += 1;
                if let Some(value) = args.get(i) {
                    query.since_ms = value.parse().ok();
                }
            }
            "--limit" => {
                i += 1;
                if let Some(value) = args.get(i) {
                    query.limit = value.parse().ok();
                }
            }
            other => {
                eprintln!("Unknown telemetry flag: {other}");
                usage();
                process::exit(1);
            }
        }
        i += 1;
    }
    ParsedQueryArgs {
        query,
        json,
        device_id,
        sensor_id,
        task_name,
        metric,
    }
}

fn print_event(event: &TelemetryEvent) {
    match event {
        TelemetryEvent::Device {
            device_id,
            metric,
            value,
            timestamp_ms,
            robot_id,
        } => println!(
            "[device] {timestamp_ms}ms {device_id} {metric} = {value}{}",
            robot_id
                .as_ref()
                .map(|id| format!(" robot={id}"))
                .unwrap_or_default()
        ),
        TelemetryEvent::Sensor {
            sensor_id,
            sensor_type,
            value,
            timestamp_ms,
            robot_id,
        } => println!(
            "[sensor] {timestamp_ms}ms {sensor_id} ({sensor_type}) = {value}{}",
            robot_id
                .as_ref()
                .map(|id| format!(" robot={id}"))
                .unwrap_or_default()
        ),
        TelemetryEvent::Heartbeat {
            task_name,
            timestamp_ms,
            robot_id,
        } => println!(
            "[heartbeat] {timestamp_ms}ms task={task_name}{}",
            robot_id
                .as_ref()
                .map(|id| format!(" robot={id}"))
                .unwrap_or_default()
        ),
        TelemetryEvent::DeviceHeartbeat {
            device_id,
            timestamp_ms,
            robot_id,
            protocol,
        } => println!(
            "[device_heartbeat] {timestamp_ms}ms device={device_id}{}",
            [
                robot_id.as_ref().map(|id| format!(" robot={id}")),
                protocol.as_ref().map(|name| format!(" protocol={name}")),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join("")
        ),
        TelemetryEvent::Health {
            target,
            status,
            timestamp_ms,
        } => println!("[health] {timestamp_ms}ms {target} -> {status}"),
    }
}
