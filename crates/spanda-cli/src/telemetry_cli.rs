//! CLI commands for querying the persistent telemetry store.

use crate::replay_cli;
use spanda_fleet::fetch_fleet_telemetry;
use spanda_telemetry_store::{
    global_store, push_otlp_json, render_otlp_json, render_prometheus, resolve_store_path,
    run_otlp_push_loop, run_telemetry_server, OtlpPushOptions, TelemetryEvent, TelemetryQuery,
    TelemetryServeOptions, TelemetrySessionSummary, TelemetryStats, TelemetryStoreInfo,
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
        "prometheus" => cmd_prometheus(args),
        "otlp" => cmd_otlp(args),
        "push" => cmd_push(args),
        "fleet-push" => cmd_fleet_push(args),
        "serve" => cmd_serve(args),
        "sessions" => cmd_sessions(args),
        "replay" => cmd_replay(args),
        "info" => cmd_info(args),
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
         spanda telemetry list [--device <id>] [--sensor <id>] [--task <name>] [--session <id>] [--kind device|sensor|heartbeat|device_heartbeat|health|session|runtime_metrics] [--since <ms>] [--limit <n>] [--json]\n\
         spanda telemetry latest [--device <id> [--metric <name>] | --sensor <id> | --task <name>] [--json]\n\
         spanda telemetry export [--out <file.jsonl>]\n\
         spanda telemetry stats [--json]\n\
         spanda telemetry heartbeats [--json]\n\
         spanda telemetry devices [--json]\n\
         spanda telemetry prometheus [--out <file.prom>]\n\
         spanda telemetry otlp [--out <file.json>]\n\
         spanda telemetry push --endpoint <url> [--token <t>] [--watch] [--interval <ms>]\n\
         spanda telemetry fleet-push --mesh <url> --endpoint <collector> [--token <t>]\n\
         spanda telemetry serve [--bind <addr>] [--once]\n\
         spanda telemetry sessions [--json]\n\
         spanda telemetry replay --session <id> [--from T+mm:ss] [--deterministic] [--playback] [--json]\n\
         spanda telemetry info [--json]"
    );
}

fn cmd_otlp(args: &[String]) {
    let mut out: Option<PathBuf> = None;
    for (index, arg) in args.iter().enumerate() {
        if arg == "--out" {
            out = args.get(index + 1).map(PathBuf::from);
        }
    }
    let store = global_store().lock().unwrap();
    let body = render_otlp_json(&store).unwrap_or_else(|error| {
        eprintln!("telemetry otlp failed: {error}");
        process::exit(1);
    });
    if let Some(path) = out {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&path, &body).unwrap_or_else(|error| {
            eprintln!("telemetry otlp failed: {error}");
            process::exit(1);
        });
        println!("Exported OTLP metrics to {}", path.display());
        return;
    }
    print!("{body}");
}

fn cmd_push(args: &[String]) {
    let mut endpoint: Option<String> = None;
    let mut token: Option<String> = None;
    let mut watch = false;
    let mut interval_ms: Option<u64> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--endpoint" => {
                i += 1;
                endpoint = args.get(i).cloned();
            }
            "--token" => {
                i += 1;
                token = args.get(i).cloned();
            }
            "--watch" => watch = true,
            "--interval" => {
                i += 1;
                interval_ms = args.get(i).and_then(|value| value.parse::<u64>().ok());
            }
            other => {
                eprintln!("Unknown telemetry push flag: {other}");
                process::exit(1);
            }
        }
        i += 1;
    }
    let endpoint = endpoint
        .or_else(|| std::env::var("SPANDA_OTLP_ENDPOINT").ok())
        .unwrap_or_else(|| {
            eprintln!("telemetry push requires --endpoint <url> or SPANDA_OTLP_ENDPOINT");
            process::exit(1);
        });
    let token = token.or_else(|| std::env::var("SPANDA_OTLP_TOKEN").ok());
    if watch {
        let options = OtlpPushOptions {
            endpoint,
            token,
            interval_ms: interval_ms.unwrap_or_else(spanda_telemetry_store::env_push_interval_ms),
            once: false,
        };
        eprintln!(
            "Watching telemetry store; pushing OTLP every {}ms (Ctrl+C to stop)",
            options.interval_ms
        );
        if let Err(error) = run_otlp_push_loop(&options) {
            eprintln!("telemetry push failed: {error}");
            process::exit(1);
        }
        return;
    }
    let options = OtlpPushOptions {
        endpoint: endpoint.clone(),
        token,
        interval_ms: interval_ms.unwrap_or_else(spanda_telemetry_store::env_push_interval_ms),
        once: true,
    };
    if let Err(error) = run_otlp_push_loop(&options) {
        eprintln!("telemetry push failed: {error}");
        process::exit(1);
    }
    println!("Pushed OTLP metrics to {endpoint}");
}

fn cmd_fleet_push(args: &[String]) {
    let mut mesh_url: Option<String> = None;
    let mut endpoint: Option<String> = None;
    let mut token: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--mesh" => {
                i += 1;
                mesh_url = args.get(i).cloned();
            }
            "--endpoint" => {
                i += 1;
                endpoint = args.get(i).cloned();
            }
            "--token" => {
                i += 1;
                token = args.get(i).cloned();
            }
            other => {
                eprintln!("Unknown telemetry fleet-push flag: {other}");
                process::exit(1);
            }
        }
        i += 1;
    }
    let mesh_url = mesh_url
        .or_else(|| std::env::var("SPANDA_FLEET_MESH_URL").ok())
        .unwrap_or_else(|| {
            eprintln!("telemetry fleet-push requires --mesh <url> or SPANDA_FLEET_MESH_URL");
            process::exit(1);
        });
    let endpoint = endpoint
        .or_else(|| std::env::var("SPANDA_OTLP_ENDPOINT").ok())
        .unwrap_or_else(|| {
            eprintln!("telemetry fleet-push requires --endpoint <url> or SPANDA_OTLP_ENDPOINT");
            process::exit(1);
        });
    let token = token.or_else(|| std::env::var("SPANDA_OTLP_TOKEN").ok());
    let mesh_token = std::env::var("SPANDA_FLEET_MESH_TOKEN").ok();
    let body = fetch_fleet_telemetry(&mesh_url, mesh_token.as_deref()).unwrap_or_else(|error| {
        eprintln!("telemetry fleet-push failed: {error}");
        process::exit(1);
    });
    if let Err(error) = push_otlp_json(&endpoint, &body, token.as_deref()) {
        eprintln!("telemetry fleet-push failed: {error}");
        process::exit(1);
    }
    println!("Pushed fleet OTLP metrics from {mesh_url} to {endpoint}");
}

fn cmd_serve(args: &[String]) {
    let mut options = TelemetryServeOptions::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--bind" => {
                i += 1;
                if let Some(bind) = args.get(i) {
                    options.bind = bind.clone();
                }
            }
            "--once" => options.once = true,
            other => {
                eprintln!("Unknown telemetry serve flag: {other}");
                usage();
                process::exit(1);
            }
        }
        i += 1;
    }
    if let Err(error) = run_telemetry_server(&options) {
        eprintln!("telemetry serve failed: {error}");
        process::exit(1);
    }
}

fn cmd_prometheus(args: &[String]) {
    let mut out: Option<PathBuf> = None;
    for (index, arg) in args.iter().enumerate() {
        if arg == "--out" {
            out = args.get(index + 1).map(PathBuf::from);
        }
    }
    let store = global_store().lock().unwrap();
    let body = render_prometheus(&store).unwrap_or_else(|error| {
        eprintln!("telemetry prometheus failed: {error}");
        process::exit(1);
    });
    if let Some(path) = out {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&path, &body).unwrap_or_else(|error| {
            eprintln!("telemetry prometheus failed: {error}");
            process::exit(1);
        });
        println!("Exported Prometheus metrics to {}", path.display());
        return;
    }
    print!("{body}");
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
        println!(
            "No telemetry events found in {}",
            store.store_path().display()
        );
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
                    session_id: None,
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
            session_id: None,
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
    println!("Session events: {}", stats.session_events);
    println!("Runtime metrics events: {}", stats.runtime_metrics_events);
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

fn cmd_sessions(args: &[String]) {
    let json = args.iter().any(|arg| arg == "--json");
    let store = global_store().lock().unwrap();
    let sessions = store.list_sessions().unwrap_or_else(|error| {
        eprintln!("telemetry sessions failed: {error}");
        process::exit(1);
    });
    if json {
        println!("{}", serde_json::to_string_pretty(&sessions).unwrap());
        return;
    }
    if sessions.is_empty() {
        println!("No telemetry sessions recorded");
        return;
    }
    for session in sessions {
        print_session_summary(&session);
    }
}

fn cmd_replay(args: &[String]) {
    let mut session_id = None;
    let mut from = None;
    let mut deterministic = false;
    let mut playback = false;
    let mut json = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--session" => {
                i += 1;
                session_id = args.get(i).cloned();
            }
            "--from" => {
                i += 1;
                from = args.get(i).cloned();
            }
            "--deterministic" => deterministic = true,
            "--playback" => playback = true,
            "--json" => json = true,
            other => {
                eprintln!("Unknown telemetry replay flag: {other}");
                usage();
                process::exit(1);
            }
        }
        i += 1;
    }
    let Some(session_id) = session_id else {
        eprintln!("telemetry replay requires --session <id>");
        usage();
        process::exit(1);
    };
    let store = global_store().lock().unwrap();
    let trace_path = store
        .mission_trace_for_session(&session_id)
        .unwrap_or_else(|error| {
            eprintln!("telemetry replay failed: {error}");
            process::exit(1);
        });
    let Some(trace_path) = trace_path else {
        eprintln!("No mission trace linked for session {session_id}");
        eprintln!("Run with --record to link a mission trace on session end.");
        process::exit(1);
    };
    replay_cli::human_replay(
        &trace_path,
        from.as_deref(),
        deterministic,
        playback,
        false,
        json,
        None,
    );
}

fn cmd_info(args: &[String]) {
    let json = args.iter().any(|arg| arg == "--json");
    let store = global_store().lock().unwrap();
    let info: TelemetryStoreInfo = store.info().unwrap_or_else(|error| {
        eprintln!("telemetry info failed: {error}");
        process::exit(1);
    });
    if json {
        println!("{}", serde_json::to_string_pretty(&info).unwrap());
        return;
    }
    println!("Backend: {}", info.backend);
    println!("Store: {}", info.store_path);
    if let Some(path) = &info.heartbeat_path {
        println!("Heartbeat index: {path}");
    }
    println!("Events: {}", info.event_count);
    if let Some(max) = info.max_events {
        println!("Retention max: {max}");
    }
    if let Some(backup) = &info.migrated_jsonl_backup {
        println!("Migrated JSONL backup: {backup}");
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
            "--session" => {
                i += 1;
                query.session_id = args.get(i).cloned();
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

fn print_session_summary(session: &TelemetrySessionSummary) {
    println!(
        "{} start={:.0}ms end={} events={}{}{}",
        session.session_id,
        session.start_ms,
        session
            .end_ms
            .map(|value| format!("{value:.0}ms"))
            .unwrap_or_else(|| "open".into()),
        session.event_count,
        session
            .source
            .as_ref()
            .map(|value| format!(" source={value}"))
            .unwrap_or_default(),
        session
            .mission_trace_path
            .as_ref()
            .map(|value| format!(" trace={value}"))
            .unwrap_or_default()
    );
}

fn print_event(event: &TelemetryEvent) {
    let session = event
        .session_id()
        .map(|id| format!(" session={id}"))
        .unwrap_or_default();
    match event {
        TelemetryEvent::Device {
            device_id,
            metric,
            value,
            timestamp_ms,
            robot_id,
            ..
        } => println!(
            "[device] {timestamp_ms}ms {device_id} {metric} = {value}{}{session}",
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
            ..
        } => println!(
            "[sensor] {timestamp_ms}ms {sensor_id} ({sensor_type}) = {value}{}{session}",
            robot_id
                .as_ref()
                .map(|id| format!(" robot={id}"))
                .unwrap_or_default()
        ),
        TelemetryEvent::Heartbeat {
            task_name,
            timestamp_ms,
            robot_id,
            ..
        } => println!(
            "[heartbeat] {timestamp_ms}ms task={task_name}{}{session}",
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
            ..
        } => println!(
            "[device_heartbeat] {timestamp_ms}ms device={device_id}{}{session}",
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
            ..
        } => println!("[health] {timestamp_ms}ms {target} -> {status}{session}"),
        TelemetryEvent::Session {
            session_id,
            phase,
            source,
            mission_trace_path,
            timestamp_ms,
        } => println!(
            "[session] {timestamp_ms}ms {session_id} phase={phase}{}{}",
            source
                .as_ref()
                .map(|value| format!(" source={value}"))
                .unwrap_or_default(),
            mission_trace_path
                .as_ref()
                .map(|value| format!(" trace={value}"))
                .unwrap_or_default()
        ),
        TelemetryEvent::RuntimeMetrics {
            session_id,
            metrics,
            timestamp_ms,
        } => println!(
            "[runtime_metrics] {timestamp_ms}ms session={session_id} keys={}",
            metrics.as_object().map(|object| object.len()).unwrap_or(0)
        ),
        TelemetryEvent::Platform {
            event_type,
            source,
            entity_id,
            timestamp_ms,
            ..
        } => println!(
            "[platform] {timestamp_ms}ms type={event_type} source={source}{}{session}",
            entity_id
                .as_ref()
                .map(|id| format!(" entity={id}"))
                .unwrap_or_default()
        ),
    }
}
