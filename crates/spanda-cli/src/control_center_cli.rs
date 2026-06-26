//! CLI for `spanda control-center` (serve + remote API client).
//!
use crate::control_center_client::ControlCenterClient;
use spanda_api::{run_control_center_server, ControlCenterOptions};
use spanda_deploy_http::HttpResponse;
use std::path::PathBuf;
use std::process;

pub fn control_center_dispatch(args: &[String]) {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_usage();
        process::exit(if args.is_empty() { 1 } else { 0 });
    }
    match args[0].as_str() {
        "serve" => cmd_serve(&args[1..]),
        "api" => cmd_api(&args[1..]),
        "dashboard" => cmd_dashboard(&args[1..]),
        "drift" => cmd_drift(&args[1..]),
        "incidents" => cmd_incidents(&args[1..]),
        "approvals" => cmd_approvals(&args[1..]),
        "evidence" => cmd_evidence(&args[1..]),
        _ => {
            eprintln!("Unknown control-center subcommand: {}", args[0]);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!(
        "Usage:\n  \
         spanda control-center serve [--bind <addr>] [--grpc-bind <addr>] [--config <spanda.toml>] [--program <file.sd>] [--once]\n  \
         spanda control-center api <get|post> <path> [--body <json>] [--url <base>] [--auth|--no-auth]\n  \
         spanda control-center dashboard [--url <base>]\n  \
         spanda control-center drift --baseline-id <id> [--url <base>]\n  \
         spanda control-center incidents list|create|ack|resolve ... [--url <base>]\n  \
         spanda control-center approvals list|submit|approve|reject ... [--url <base>]\n  \
         spanda control-center evidence list [--url <base>]\n\n\
         Remote calls use SPANDA_CONTROL_CENTER_URL (default http://127.0.0.1:8080) and SPANDA_API_KEY for mutations.\n\
         serve: set SPANDA_API_KEY for authenticated mutations (PATCH devices, POST alerts/test).\n\
         serve: set SPANDA_ALERT_WEBHOOK_URL or SPANDA_ALERT_EMAIL_TO for alert delivery."
    );
}

fn cmd_serve(args: &[String]) {
    let mut options = ControlCenterOptions::default();
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--bind" => {
                index += 1;
                options.bind = args
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| missing("--bind"));
            }
            "--config" => {
                index += 1;
                let path = args
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| missing("--config"));
                options.config_path = Some(PathBuf::from(path));
            }
            "--program" => {
                index += 1;
                let path = args
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| missing("--program"));
                options.program_path = Some(PathBuf::from(path));
            }
            "--once" => options.once = true,
            "--grpc-bind" => {
                index += 1;
                let addr = args
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| missing("--grpc-bind"));
                options.grpc_bind = Some(addr);
            }
            other => {
                eprintln!("Unknown flag: {other}");
                print_usage();
                process::exit(1);
            }
        }
        index += 1;
    }
    if let Err(error) = run_control_center_server(&options) {
        eprintln!("control-center serve failed: {error}");
        process::exit(1);
    }
}

fn client_from_args(args: &[String]) -> ControlCenterClient {
    let mut client = ControlCenterClient::from_env();
    if let Some(url) = flag_value(args, "--url") {
        client = client.with_url(url);
    }
    client
}

fn cmd_api(args: &[String]) {
    if args.len() < 2 || args[0] == "--help" || args[0] == "-h" {
        eprintln!("Usage: spanda control-center api <get|post> <path> [--body <json>] [--url <base>] [--auth|--no-auth]");
        process::exit(if args.is_empty() { 1 } else { 0 });
    }
    let method = args[0].to_ascii_lowercase();
    let path = &args[1];
    let body = flag_value(args, "--body").unwrap_or_else(|| "{}".into());
    let auth = if args.iter().any(|a| a == "--no-auth") {
        false
    } else if args.iter().any(|a| a == "--auth") {
        true
    } else {
        method == "post"
    };
    let client = client_from_args(args);
    let response = match method.as_str() {
        "get" => client.get(path, auth),
        "post" => client.post(path, &body, auth),
        other => {
            eprintln!("Unknown api method: {other} (use get or post)");
            process::exit(1);
        }
    }
    .unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(1);
    });
    print_response(response);
}

fn cmd_dashboard(args: &[String]) {
    let client = client_from_args(args);
    let response = client.get("/v1/dashboard", false).unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(1);
    });
    print_response(response);
}

fn cmd_drift(args: &[String]) {
    let baseline_id = flag_value(args, "--baseline-id").unwrap_or_else(|| {
        eprintln!("Missing --baseline-id");
        process::exit(1);
    });
    let client = client_from_args(args);
    let path = format!("/v1/drift?baseline_id={baseline_id}");
    let response = client.get(&path, false).unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(1);
    });
    print_response(response);
}

fn cmd_incidents(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: spanda control-center incidents list|create|ack|resolve ...");
        process::exit(1);
    }
    let client = client_from_args(args);
    match args[0].as_str() {
        "list" => {
            let response = client.get("/v1/sre/incidents", false).unwrap_or_else(|error| {
                eprintln!("{error}");
                process::exit(1);
            });
            print_response(response);
        }
        "create" => {
            let title = flag_value(args, "--title").unwrap_or_else(|| {
                eprintln!("Missing --title");
                process::exit(1);
            });
            let description = flag_value(args, "--description").unwrap_or_default();
            let severity = flag_value(args, "--severity").unwrap_or_else(|| "warning".into());
            let body = serde_json::json!({
                "title": title,
                "description": description,
                "severity": severity,
            });
            let response = client
                .post("/v1/sre/incidents", &body.to_string(), true)
                .unwrap_or_else(|error| {
                    eprintln!("{error}");
                    process::exit(1);
                });
            print_response(response);
        }
        "ack" => {
            let incident_id = positional_arg(args, 1).unwrap_or_else(|| {
                eprintln!("Missing incident id");
                process::exit(1);
            });
            let assignee = flag_value(args, "--assignee");
            let body = if let Some(assignee) = assignee {
                serde_json::json!({ "assignee": assignee }).to_string()
            } else {
                "{}".into()
            };
            let path = format!("/v1/sre/incidents/{incident_id}/ack");
            let response = client.post(&path, &body, true).unwrap_or_else(|error| {
                eprintln!("{error}");
                process::exit(1);
            });
            print_response(response);
        }
        "resolve" => {
            let incident_id = positional_arg(args, 1).unwrap_or_else(|| {
                eprintln!("Missing incident id");
                process::exit(1);
            });
            let path = format!("/v1/sre/incidents/{incident_id}/resolve");
            let response = client.post(&path, "{}", true).unwrap_or_else(|error| {
                eprintln!("{error}");
                process::exit(1);
            });
            print_response(response);
        }
        other => {
            eprintln!("Unknown incidents subcommand: {other}");
            process::exit(1);
        }
    }
}

fn cmd_approvals(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: spanda control-center approvals list|submit|approve|reject ...");
        process::exit(1);
    }
    let client = client_from_args(args);
    match args[0].as_str() {
        "list" => {
            let response = client.get("/v1/config/approvals", false).unwrap_or_else(|error| {
                eprintln!("{error}");
                process::exit(1);
            });
            print_response(response);
        }
        "submit" => {
            let snapshot_id = flag_value(args, "--snapshot-id").unwrap_or_else(|| {
                eprintln!("Missing --snapshot-id");
                process::exit(1);
            });
            let note = flag_value(args, "--note");
            let body = if let Some(note) = note {
                serde_json::json!({ "snapshot_id": snapshot_id, "note": note })
            } else {
                serde_json::json!({ "snapshot_id": snapshot_id })
            };
            let response = client
                .post("/v1/config/approvals", &body.to_string(), true)
                .unwrap_or_else(|error| {
                    eprintln!("{error}");
                    process::exit(1);
                });
            print_response(response);
        }
        "approve" | "reject" => {
            let approval_id = positional_arg(args, 1).unwrap_or_else(|| {
                eprintln!("Missing approval id");
                process::exit(1);
            });
            let note = flag_value(args, "--note");
            let body = if let Some(note) = note {
                serde_json::json!({ "note": note }).to_string()
            } else {
                "{}".into()
            };
            let path = format!("/v1/config/approvals/{approval_id}/{}", args[0]);
            let response = client.post(&path, &body, true).unwrap_or_else(|error| {
                eprintln!("{error}");
                process::exit(1);
            });
            print_response(response);
        }
        other => {
            eprintln!("Unknown approvals subcommand: {other}");
            process::exit(1);
        }
    }
}

fn cmd_evidence(args: &[String]) {
    if !args.is_empty() && args[0] != "list" {
        eprintln!("Usage: spanda control-center evidence list [--url <base>]");
        process::exit(1);
    }
    let client = client_from_args(args);
    let response = client
        .get("/v1/compliance/evidence", true)
        .unwrap_or_else(|error| {
            eprintln!("{error}");
            process::exit(1);
        });
    print_response(response);
}

fn print_response(response: HttpResponse) {
    if response.status >= 400 {
        eprintln!("HTTP {}", response.status);
        eprintln!("{}", response.body);
        process::exit(1);
    }
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&response.body) {
        println!("{}", serde_json::to_string_pretty(&value).unwrap_or(response.body));
    } else {
        println!("{}", response.body);
    }
}

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    for (index, arg) in args.iter().enumerate() {
        if arg == flag {
            return args.get(index + 1).cloned();
        }
    }
    None
}

fn positional_arg(args: &[String], index: usize) -> Option<String> {
    let mut position = 0usize;
    let mut cursor = 0usize;
    while cursor < args.len() {
        let arg = &args[cursor];
        if arg.starts_with("--") {
            if flag_value(args, arg).is_some() {
                cursor += 2;
                continue;
            }
            cursor += 1;
            continue;
        }
        if position == index {
            return Some(arg.clone());
        }
        position += 1;
        cursor += 1;
    }
    None
}

fn missing(flag: &str) -> ! {
    eprintln!("Missing value for {flag}");
    process::exit(1);
}
