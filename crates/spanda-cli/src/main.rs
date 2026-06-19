use serde::Serialize;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;
use spanda_core::{check, format_source, run, RunOptions, SpandaError};

#[derive(Serialize)]
struct CheckResponse {
    ok: bool,
    diagnostics: Vec<spanda_core::Diagnostic>,
}

#[derive(Serialize)]
struct RunResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<spanda_core::RunResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostics: Option<Vec<spanda_core::Diagnostic>>,
}

fn usage() {
    eprintln!(
        "Spanda Programming Language\n\n\
         Usage:\n\
           spanda check [--json] <file.sd>\n\
           spanda run [--json] [--verbose] <file.sd>\n\
           spanda sim [--json] <file.sd>\n\
           spanda fmt <file.sd>\n"
    );
}

fn read_source(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading {path}: {e}");
        process::exit(1);
    })
}

fn print_check_json(err: Option<SpandaError>) {
    let resp = match err {
        None => CheckResponse {
            ok: true,
            diagnostics: vec![],
        },
        Some(e) => CheckResponse {
            ok: false,
            diagnostics: e.diagnostics(),
        },
    };
    println!("{}", serde_json::to_string(&resp).unwrap());
}

fn print_run_json(result: Result<spanda_core::RunResult, SpandaError>) {
    let resp = match result {
        Ok(result) => RunResponse {
            ok: true,
            result: Some(result),
            diagnostics: None,
        },
        Err(e) => RunResponse {
            ok: false,
            result: None,
            diagnostics: Some(e.diagnostics()),
        },
    };
    println!("{}", serde_json::to_string(&resp).unwrap());
}

fn human_check(source: &str, file: &str) {
    match check(source) {
        Ok(()) => {
            println!("✓ {file} — no type errors");
        }
        Err(e) => {
            eprintln!("Type errors:");
            for d in e.diagnostics() {
                eprintln!("  [{}:{}] {}", d.line, d.column, d.message);
            }
            process::exit(1);
        }
    }
}

fn human_run(source: &str, file: &str, verbose: bool) {
    let max_loop_iterations = if verbose { 20 } else { 10 };
    match run(
        source,
        RunOptions {
            max_loop_iterations,
            ..Default::default()
        },
    ) {
        Ok(result) => {
            let s = &result.state;
            println!("\n🤖 Running robot from {file}\n");
            println!("── Final State ──");
            println!(
                "  Pose:     x={:.3} m, y={:.3} m, θ={:.3} rad",
                s.pose.x, s.pose.y, s.pose.theta
            );
            if let Some(z) = s.pose.z {
                println!("  Altitude: z={z:.3} m");
            }
            println!(
                "  Velocity: linear={:.3} m/s, angular={:.3} rad/s",
                s.velocity.linear, s.velocity.angular
            );
            println!(
                "  E-stop:   {}",
                if s.emergency_stop { "ACTIVE" } else { "off" }
            );
            if verbose {
                println!("\n── Simulation Log ──");
                for event in &result.events {
                    println!("  {event}");
                }
                if !result.logs.is_empty() {
                    println!("\n── Runtime Log ──");
                    for log in &result.logs {
                        println!("  {log}");
                    }
                }
            }
            println!("\n✓ Simulation complete\n");
        }
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        usage();
        process::exit(if args.len() < 2 { 1 } else { 0 });
    }

    let mut json = false;
    let mut verbose = false;
    let mut command: Option<&str> = None;
    let mut file: Option<&str> = None;

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--json" => json = true,
            "--verbose" | "-v" => verbose = true,
            "check" | "run" | "sim" | "fmt" if command.is_none() => command = Some(arg),
            other if !other.starts_with('-') && file.is_none() => file = Some(other),
            other => {
                eprintln!("Unknown argument: {other}");
                usage();
                process::exit(1);
            }
        }
    }

    let command = command.unwrap_or_else(|| {
        eprintln!("Missing command");
        usage();
        process::exit(1);
    });

    let file = file.unwrap_or_else(|| {
        eprintln!("Missing file path");
        usage();
        process::exit(1);
    });

    let source = read_source(file);

    match command {
        "check" => {
            if json {
                print_check_json(check(&source).err());
            } else {
                human_check(&source, file);
            }
        }
        "run" | "sim" => {
            let max_loop_iterations = if command == "sim" || verbose {
                20
            } else {
                10
            };
            let opts = RunOptions {
                max_loop_iterations,
                ..Default::default()
            };
            if json {
                print_run_json(run(&source, opts));
            } else {
                human_run(&source, file, command == "sim" || verbose);
            }
        }
        "fmt" => {
            let formatted = format_source(&source);
            if formatted != source {
                fs::write(file, formatted).unwrap_or_else(|e| {
                    eprintln!("Error writing {file}: {e}");
                    process::exit(1);
                });
                println!("✓ formatted {file}");
            } else {
                println!("✓ {file} — already formatted");
            }
        }
        _ => {
            eprintln!("Unknown command: {command}");
            process::exit(1);
        }
    }

    let _ = io::stdout().flush();
}
