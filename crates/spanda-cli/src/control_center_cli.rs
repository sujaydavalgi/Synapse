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
        "sre" => cmd_sre(&args[1..]),
        "devices" => cmd_devices(&args[1..]),
        "ota" => cmd_ota(&args[1..]),
        "readiness" => cmd_readiness(&args[1..]),
        "compliance" => cmd_compliance(&args[1..]),
        "alerts" => cmd_alerts(&args[1..]),
        "snapshots" => cmd_snapshots(&args[1..]),
        "trust" => cmd_trust(&args[1..]),
        "scorecard" => cmd_scorecard(&args[1..]),
        "digital-thread" => cmd_digital_thread(&args[1..]),
        "reports" => cmd_reports(&args[1..]),
        "provision" => cmd_provision(&args[1..]),
        "secrets" => cmd_secrets(&args[1..]),
        "audit" => cmd_audit(&args[1..]),
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
         spanda control-center evidence list [--url <base>]\n  \
         spanda control-center sre summary [--url <base>]\n  \
         spanda control-center devices list|get|assign|quarantine|trust|provision|patch ... [--url <base>]\n  \
         spanda control-center ota plan|execute|status ... [--url <base>]\n  \
         spanda control-center readiness run [--url <base>]\n  \
         spanda control-center compliance export [--profile <name>] [--url <base>]\n  \
         spanda control-center alerts list|test [--url <base>]\n  \
         spanda control-center snapshots list|save [--label <name>] [--url <base>]\n  \
         spanda control-center trust package --name <pkg> [--version <ver>] [--url <base>]\n  \
         spanda control-center scorecard [--url <base>]\n  \
         spanda control-center digital-thread query [--capability <name>] [--device-id <id>] [--url <base>]\n  \
         spanda control-center reports export [--format markdown|json|pdf] [--url <base>]\n  \
         spanda control-center provision run [--body <json>] [--url <base>]\n  \
         spanda control-center secrets list [--url <base>]\n  \
         spanda control-center audit list [--url <base>]\n\n\
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
    remote_get(&client, "/v1/compliance/evidence", true);
}

fn cmd_sre(args: &[String]) {
    if args.first().map(String::as_str) != Some("summary") {
        eprintln!("Usage: spanda control-center sre summary [--url <base>]");
        process::exit(1);
    }
    let client = client_from_args(args);
    remote_get(&client, "/v1/sre/summary", false);
}

fn cmd_devices(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: spanda control-center devices list|get|assign|quarantine|trust|provision|patch ...");
        process::exit(1);
    }
    let client = client_from_args(args);
    match args[0].as_str() {
        "list" => remote_get(&client, "/v1/devices", false),
        "get" => {
            let device_id = positional_arg(args, 1).unwrap_or_else(|| {
                eprintln!("Missing device id");
                process::exit(1);
            });
            let path = format!("/v1/devices/{device_id}");
            remote_get(&client, &path, false);
        }
        "assign" => {
            let device_id = positional_arg(args, 1).unwrap_or_else(|| {
                eprintln!("Missing device id");
                process::exit(1);
            });
            let robot_id = flag_value(args, "--robot-id").unwrap_or_else(|| {
                eprintln!("Missing --robot-id");
                process::exit(1);
            });
            let mut body = serde_json::Map::new();
            body.insert("robot_id".into(), serde_json::json!(robot_id));
            if let Some(logical_name) = flag_value(args, "--logical-name") {
                body.insert("logical_name".into(), serde_json::json!(logical_name));
            }
            if let Some(group) = flag_value(args, "--redundant-group") {
                body.insert("redundant_group".into(), serde_json::json!(group));
            }
            let path = format!("/v1/devices/{device_id}/assign");
            remote_post(
                &client,
                &path,
                &serde_json::Value::Object(body).to_string(),
                true,
            );
        }
        "quarantine" | "trust" | "provision" => {
            let device_id = positional_arg(args, 1).unwrap_or_else(|| {
                eprintln!("Missing device id");
                process::exit(1);
            });
            let path = format!("/v1/devices/{device_id}/{}", args[0]);
            let body = flag_value(args, "--body").unwrap_or_else(|| "{}".into());
            remote_post(&client, &path, &body, true);
        }
        "patch" => {
            let device_id = positional_arg(args, 1).unwrap_or_else(|| {
                eprintln!("Missing device id");
                process::exit(1);
            });
            let lifecycle = flag_value(args, "--lifecycle-state").unwrap_or_else(|| {
                eprintln!("Missing --lifecycle-state");
                process::exit(1);
            });
            let body = serde_json::json!({ "lifecycle_state": lifecycle }).to_string();
            let path = format!("/v1/devices/{device_id}");
            remote_patch(&client, &path, &body, true);
        }
        other => {
            eprintln!("Unknown devices subcommand: {other}");
            process::exit(1);
        }
    }
}

fn cmd_ota(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: spanda control-center ota plan|execute|status ...");
        process::exit(1);
    }
    let client = client_from_args(args);
    match args[0].as_str() {
        "status" => remote_get(&client, "/v1/ota/status", false),
        "plan" | "execute" => {
            let body = ota_body_from_args(args);
            let path = format!("/v1/ota/{}", args[0]);
            remote_post(&client, &path, &body, true);
        }
        other => {
            eprintln!("Unknown ota subcommand: {other}");
            process::exit(1);
        }
    }
}

fn ota_body_from_args(args: &[String]) -> String {
    if let Some(body) = flag_value(args, "--body") {
        return body;
    }
    let strategy = flag_value(args, "--strategy").unwrap_or_else(|| "canary".into());
    let version = flag_value(args, "--version").unwrap_or_else(|| {
        eprintln!("Missing --version (or pass --body <json>)");
        process::exit(1);
    });
    let dry_run = args.iter().any(|arg| arg == "--dry-run");
    let canary_percent = flag_value(args, "--canary-percent")
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(10);
    serde_json::json!({
        "strategy": strategy,
        "version": version,
        "dry_run": dry_run,
        "canary_percent": canary_percent,
        "assignments": [],
    })
    .to_string()
}

fn cmd_readiness(args: &[String]) {
    if args.first().map(String::as_str) != Some("run") {
        eprintln!("Usage: spanda control-center readiness run [--url <base>]");
        process::exit(1);
    }
    let client = client_from_args(args);
    let body = flag_value(args, "--body").unwrap_or_else(|| "{}".into());
    remote_post(&client, "/v1/readiness/run", &body, false);
}

fn cmd_compliance(args: &[String]) {
    if args.first().map(String::as_str) != Some("export") {
        eprintln!("Usage: spanda control-center compliance export [--profile <name>] [--url <base>]");
        process::exit(1);
    }
    let profile = flag_value(args, "--profile").unwrap_or_else(|| "defense".into());
    let client = client_from_args(args);
    let path = format!("/v1/compliance/export?profile={profile}");
    remote_get(&client, &path, true);
}

fn cmd_alerts(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: spanda control-center alerts list|test [--url <base>]");
        process::exit(1);
    }
    let client = client_from_args(args);
    match args[0].as_str() {
        "list" => remote_get(&client, "/v1/alerts", false),
        "test" => remote_post(&client, "/v1/alerts/test", "{}", true),
        other => {
            eprintln!("Unknown alerts subcommand: {other}");
            process::exit(1);
        }
    }
}

fn cmd_snapshots(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: spanda control-center snapshots list|save [--label <name>] [--url <base>]");
        process::exit(1);
    }
    let client = client_from_args(args);
    match args[0].as_str() {
        "list" => remote_get(&client, "/v1/config/snapshots", false),
        "save" => {
            let label = flag_value(args, "--label");
            let body = if let Some(label) = label {
                serde_json::json!({ "label": label }).to_string()
            } else {
                "{}".into()
            };
            remote_post(&client, "/v1/config/snapshots", &body, true);
        }
        other => {
            eprintln!("Unknown snapshots subcommand: {other}");
            process::exit(1);
        }
    }
}

fn cmd_trust(args: &[String]) {
    if args.first().map(String::as_str) != Some("package") {
        eprintln!("Usage: spanda control-center trust package --name <pkg> [--version <ver>] [--url <base>]");
        process::exit(1);
    }
    let name = flag_value(args, "--name").unwrap_or_else(|| {
        eprintln!("Missing --name");
        process::exit(1);
    });
    let client = client_from_args(args);
    let path = if let Some(version) = flag_value(args, "--version") {
        format!("/v1/trust/package?name={name}&version={version}")
    } else {
        format!("/v1/trust/package?name={name}")
    };
    remote_get(&client, &path, false);
}

fn cmd_scorecard(args: &[String]) {
    let client = client_from_args(args);
    remote_get(&client, "/v1/executive/scorecard", false);
}

fn cmd_digital_thread(args: &[String]) {
    if args.first().map(String::as_str) != Some("query") {
        eprintln!("Usage: spanda control-center digital-thread query [--capability <name>] [--device-id <id>]");
        process::exit(1);
    }
    let mut query = Vec::new();
    if let Some(capability) = flag_value(args, "--capability") {
        query.push(format!("capability={capability}"));
    }
    if let Some(device_id) = flag_value(args, "--device-id") {
        query.push(format!("device_id={device_id}"));
    }
    let path = if query.is_empty() {
        "/v1/digital-thread/query".into()
    } else {
        format!("/v1/digital-thread/query?{}", query.join("&"))
    };
    let client = client_from_args(args);
    remote_get(&client, &path, false);
}

fn cmd_reports(args: &[String]) {
    if args.first().map(String::as_str) != Some("export") {
        eprintln!("Usage: spanda control-center reports export [--format markdown|json|pdf]");
        process::exit(1);
    }
    let format = flag_value(args, "--format").unwrap_or_else(|| "markdown".into());
    let client = client_from_args(args);
    let path = format!("/v1/reports/export?format={format}");
    remote_get(&client, &path, true);
}

fn cmd_provision(args: &[String]) {
    if args.first().map(String::as_str) != Some("run") {
        eprintln!("Usage: spanda control-center provision run [--body <json>]");
        process::exit(1);
    }
    let client = client_from_args(args);
    let body = flag_value(args, "--body").unwrap_or_else(|| "{}".into());
    remote_post(&client, "/v1/provision", &body, true);
}

fn cmd_secrets(args: &[String]) {
    if args.first().map(String::as_str) != Some("list") {
        eprintln!("Usage: spanda control-center secrets list");
        process::exit(1);
    }
    let client = client_from_args(args);
    remote_get(&client, "/v1/secrets", true);
}

fn cmd_audit(args: &[String]) {
    if args.first().map(String::as_str) != Some("list") {
        eprintln!("Usage: spanda control-center audit list");
        process::exit(1);
    }
    let client = client_from_args(args);
    remote_get(&client, "/v1/audit/mutations", true);
}

fn remote_get(client: &ControlCenterClient, path: &str, auth: bool) {
    let response = client.get(path, auth).unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(1);
    });
    print_response(response);
}

fn remote_post(client: &ControlCenterClient, path: &str, body: &str, auth: bool) {
    let response = client.post(path, body, auth).unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(1);
    });
    print_response(response);
}

fn remote_patch(client: &ControlCenterClient, path: &str, body: &str, auth: bool) {
    let response = client.patch(path, body, auth).unwrap_or_else(|error| {
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
