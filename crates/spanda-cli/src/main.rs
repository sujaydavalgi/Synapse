mod package;

use serde::Serialize;
use spanda_core::{
    check, format_source, generate_markdown, lint, run, verify_compatibility, CompatSeverity,
    RunOptions, SpandaError, VerifyOptions,
};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

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

#[derive(Serialize)]
struct VerifyResponse {
    ok: bool,
    compatible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<String>,
    items: Vec<spanda_core::CompatItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    matrix: Option<spanda_core::CompatibilityMatrix>,
}

#[derive(Serialize)]
struct LintResponse {
    ok: bool,
    issues: Vec<spanda_core::LintIssue>,
}

#[derive(Serialize)]
struct FormatResponse {
    ok: bool,
    changed: bool,
    formatted: String,
}

#[derive(Serialize)]
struct DocResponse {
    ok: bool,
    markdown: String,
}

fn usage() {
    eprintln!(
        "Spanda Programming Language\n\n\
         Usage:\n\
           spanda check [--json] [<file.sd> | --project]\n\
           spanda verify [--json] [--target <HardwareProfile>] [--all-targets] [--simulate] <file.sd>\n\
           spanda compatibility [--json] [--target <HardwareProfile>] [--all-targets] [--simulate] <file.sd>\n\
           spanda run [--json] [--verbose] <file.sd>\n\
           spanda sim [--json] <file.sd>\n\
           spanda fmt [--json] <file.sd>\n\
           spanda lint [--json] <file.sd>\n\
           spanda doc [--json] [--out <file.md>] <file.sd>\n\n\
         Package commands:\n\
           spanda init [name] [--description <text>]\n\
           spanda build [--project <dir>]\n\
           spanda test [--project <dir>]\n\
           spanda add <package> [--version <ver>] [--path <dir>] [--git <url>]\n\
           spanda remove <package>\n\
           spanda install [--project <dir>]\n\
           spanda publish [--project <dir>]\n\
           spanda registry search <query>\n"
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

fn human_verify(source: &str, file: &str, options: &VerifyOptions) {
    match verify_compatibility(source, options) {
        Ok(report) => {
            println!("Hardware compatibility: {file}");
            if let Some(t) = &report.target {
                println!("Target: {t}\n");
            }
            for item in &report.items {
                let icon = match item.severity {
                    CompatSeverity::Pass => "✓",
                    CompatSeverity::Warning => "⚠",
                    CompatSeverity::Error => "✗",
                };
                println!("  {icon} [{}] {}", item.category, item.message);
            }
            if let Some(matrix) = &report.matrix {
                println!("\n── Compatibility Matrix ──");
                for cell in &matrix.cells {
                    let icon = if cell.compatible { "✓" } else { "✗" };
                    println!("  {icon} {} → {}", cell.robot, cell.target);
                }
                let compatible = matrix.cells.iter().filter(|c| c.compatible).count();
                let total = matrix.cells.len();
                println!("\n{compatible}/{total} robot × target pairs compatible");
                return;
            }
            if report.compatible {
                println!("\n✓ Deployment compatible");
            } else {
                println!("\n✗ Deployment incompatible");
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            for d in e.diagnostics() {
                eprintln!("  [{}:{}] {}", d.line, d.column, d.message);
            }
            process::exit(1);
        }
    }
}

fn print_verify_json(result: Result<spanda_core::CompatibilityReport, SpandaError>) {
    let resp = match result {
        Ok(report) => VerifyResponse {
            ok: report.compatible,
            compatible: report.compatible,
            target: report.target.clone(),
            items: report.items.clone(),
            matrix: report.matrix.clone(),
        },
        Err(e) => VerifyResponse {
            ok: false,
            compatible: false,
            target: None,
            items: e
                .diagnostics()
                .into_iter()
                .map(|d| spanda_core::CompatItem {
                    category: "error".into(),
                    message: d.message,
                    severity: CompatSeverity::Error,
                    line: d.line,
                    column: d.column,
                })
                .collect(),
            matrix: None,
        },
    };
    println!("{}", serde_json::to_string(&resp).unwrap());
}

fn is_package_command(cmd: &str) -> bool {
    matches!(
        cmd,
        "init" | "build" | "test" | "add" | "remove" | "publish" | "install" | "registry"
    )
}

fn dispatch_package(command: &str, rest: &[String]) {
    match command {
        "init" => package::cmd_init(rest),
        "build" => package::cmd_build(rest),
        "test" => package::cmd_test(rest),
        "add" => package::cmd_add(rest),
        "remove" => package::cmd_remove(rest),
        "install" => package::cmd_install(rest),
        "publish" => package::cmd_publish(rest),
        "registry" => {
            if rest.first().map(String::as_str) != Some("search") {
                eprintln!("Usage: spanda registry search <query>");
                process::exit(1);
            }
            package::cmd_registry_search(&rest[1..]);
        }
        _ => {
            eprintln!("Unknown package command: {command}");
            package::usage_package();
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

    let command = args[1].as_str();
    if is_package_command(command) {
        dispatch_package(command, &args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    let mut json = false;
    let mut verbose = false;
    let mut target: Option<String> = None;
    let mut all_targets = false;
    let mut simulate = false;
    let mut project_mode = false;
    let mut out_path: Option<String> = None;
    let mut file: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            "--verbose" | "-v" => verbose = true,
            "--project" => project_mode = true,
            "--target" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--target requires a hardware profile name");
                    process::exit(1);
                }
                target = Some(args[i].clone());
            }
            "--all-targets" => all_targets = true,
            "--simulate" => simulate = true,
            "--out" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--out requires a file path");
                    process::exit(1);
                }
                out_path = Some(args[i].clone());
            }
            other if !other.starts_with('-') && file.is_none() => file = Some(other.to_string()),
            other => {
                eprintln!("Unknown argument: {other}");
                usage();
                process::exit(1);
            }
        }
        i += 1;
    }

    match command {
        "check" => {
            if project_mode || file.is_none() {
                package::cmd_check_project(&args[2..]);
            } else if let Some(ref file_path) = file {
                let source = read_source(file_path);
                if json {
                    print_check_json(check(&source).err());
                } else {
                    human_check(&source, file_path);
                }
            }
        }
        "verify" | "compatibility" => {
            let file = file.unwrap_or_else(|| {
                eprintln!("Missing file path");
                usage();
                process::exit(1);
            });
            let source = read_source(&file);
            let options = VerifyOptions {
                target: target.clone(),
                all_targets,
                simulate,
            };
            if json {
                let result = verify_compatibility(&source, &options);
                let failed = match &result {
                    Ok(report) => !options.all_targets && !report.compatible,
                    Err(_) => true,
                };
                print_verify_json(result);
                if failed {
                    process::exit(1);
                }
            } else {
                human_verify(&source, &file, &options);
            }
        }
        "run" | "sim" => {
            let file = file.unwrap_or_else(|| {
                eprintln!("Missing file path");
                usage();
                process::exit(1);
            });
            let source = read_source(&file);
            let max_loop_iterations = if command == "sim" || verbose { 20 } else { 10 };
            let opts = RunOptions {
                max_loop_iterations,
                ..Default::default()
            };
            if json {
                print_run_json(run(&source, opts));
            } else {
                human_run(&source, &file, command == "sim" || verbose);
            }
        }
        "fmt" => {
            let file = file.unwrap_or_else(|| {
                eprintln!("Missing file path");
                usage();
                process::exit(1);
            });
            let source = read_source(&file);
            let formatted = format_source(&source);
            let changed = formatted != source;
            if json {
                let resp = FormatResponse {
                    ok: true,
                    changed,
                    formatted: formatted.clone(),
                };
                println!("{}", serde_json::to_string(&resp).unwrap());
            } else if changed {
                fs::write(&file, &formatted).unwrap_or_else(|e| {
                    eprintln!("Error writing {file}: {e}");
                    process::exit(1);
                });
                println!("✓ formatted {file}");
            } else {
                println!("✓ {file} — already formatted");
            }
        }
        "lint" => {
            let file = file.unwrap_or_else(|| {
                eprintln!("Missing file path");
                usage();
                process::exit(1);
            });
            let source = read_source(&file);
            match lint(&source) {
                Ok(report) => {
                    if json {
                        let resp = LintResponse {
                            ok: !report.has_errors(),
                            issues: report.issues.clone(),
                        };
                        println!("{}", serde_json::to_string(&resp).unwrap());
                    } else if report.issues.is_empty() {
                        println!("✓ {file} — no lint issues");
                    } else {
                        println!("Lint issues in {file}:");
                        for issue in &report.issues {
                            let level = match issue.severity {
                                spanda_core::LintSeverity::Warning => "warning",
                                spanda_core::LintSeverity::Error => "error",
                            };
                            eprintln!(
                                "  [{level}] {} [{}:{}] {}",
                                issue.rule, issue.line, issue.column, issue.message
                            );
                        }
                    }
                    if report.has_errors() {
                        process::exit(1);
                    }
                }
                Err(e) => {
                    if json {
                        let resp = LintResponse {
                            ok: false,
                            issues: e
                                .diagnostics()
                                .into_iter()
                                .map(|d| spanda_core::LintIssue {
                                    rule: "parse".into(),
                                    message: d.message,
                                    line: d.line,
                                    column: d.column,
                                    severity: spanda_core::LintSeverity::Error,
                                })
                                .collect(),
                        };
                        println!("{}", serde_json::to_string(&resp).unwrap());
                    } else {
                        eprintln!("Error: {e}");
                    }
                    process::exit(1);
                }
            }
        }
        "doc" => {
            let file = file.unwrap_or_else(|| {
                eprintln!("Missing file path");
                usage();
                process::exit(1);
            });
            let source = read_source(&file);
            match generate_markdown(&source) {
                Ok(markdown) => {
                    if let Some(ref out) = out_path {
                        fs::write(out, &markdown).unwrap_or_else(|e| {
                            eprintln!("Error writing {out}: {e}");
                            process::exit(1);
                        });
                        if !json {
                            println!("✓ wrote docs to {out}");
                        }
                    } else if !json {
                        print!("{markdown}");
                    }
                    if json {
                        let resp = DocResponse { ok: true, markdown };
                        println!("{}", serde_json::to_string(&resp).unwrap());
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Unknown command: {command}");
            usage();
            process::exit(1);
        }
    }

    let _ = io::stdout().flush();
}
