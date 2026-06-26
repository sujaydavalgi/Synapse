//! CLI for `spanda control-center serve`.
//!
use spanda_api::{run_control_center_server, ControlCenterOptions};
use std::path::PathBuf;
use std::process;

pub fn control_center_dispatch(args: &[String]) {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_usage();
        process::exit(if args.is_empty() { 1 } else { 0 });
    }
    match args[0].as_str() {
        "serve" => cmd_serve(&args[1..]),
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
         spanda control-center serve [--bind <addr>] [--config <spanda.toml>] [--program <file.sd>] [--once]\n\n\
         Serves the Control Center UI and REST API v1.\n\
         Set SPANDA_API_KEY for authenticated mutations (PATCH devices, POST alerts/test).\n\
         Set SPANDA_ALERT_WEBHOOK_URL or SPANDA_ALERT_EMAIL_TO for alert delivery."
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

fn missing(flag: &str) -> ! {
    eprintln!("Missing value for {flag}");
    process::exit(1);
}
