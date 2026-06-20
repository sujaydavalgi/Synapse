mod package;

use serde::Serialize;
use spanda_core::{
    check, codegen, format_source, generate_markdown, lint, lower_to_sir, run, run_debug,
    verify_compatibility, wasm_deploy_manifest, CodegenTarget, CompatSeverity, DebugOptions,
    RunOptions, SpandaError, VerifyOptions,
};
use spanda_llvm::{compile_native, emit_module_ir_with_options, CompileNativeOptions};
use std::collections::HashSet;
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

#[derive(Serialize)]
struct IrResponse {
    ok: bool,
    sir: spanda_core::SirProgram,
}

fn usage() {
    eprintln!(
        "Spanda Programming Language\n\n\
         Usage:\n\
           spanda check [--json] [<file.sd> | --project]\n\
           spanda verify [--json] [--target <HardwareProfile>] [--all-targets] [--simulate] <file.sd>\n\
           spanda compatibility [--json] [--target <HardwareProfile>] [--all-targets] [--simulate] <file.sd>\n\
           spanda run [--json] [--verbose] [--trace-scheduler] [--trace-tasks] [--trace-triggers] [--trace-events] <file.sd>\n\
           spanda sim [--json] [--replay] [--trace-scheduler] [--trace-tasks] [--trace-triggers] [--trace-events] <file.sd>\n\
           spanda fleet run [--json] [--trace-scheduler] [--trace-tasks] [--trace-triggers] [--trace-events] <file.sd>\n\
           spanda fmt [--json] <file.sd>\n\
           spanda lint [--json] <file.sd>\n\
           spanda doc [--json] [--out <file.md>] <file.sd>\n\
           spanda codegen [--target native|wasm|esp32] [--out <file>] <file.sd>\n\
           spanda deploy --target wasm [--out <file.json>] <file.sd>\n\
           spanda debug [--break <line>] <file.sd>\n\
           spanda ir [--json] <file.sd>\n\
           spanda llvm-ir [--out <file.ll>] [--target-triple <triple>] [--hal-profile <name>] <file.sd>\n\
           spanda compile-native [--out <binary>] [--target-triple <triple>] [--hal-profile <name>] <file.sd>\n\n\
         Package commands:\n\
           spanda init [name] [--description <text>]\n\
           spanda build [--project <dir>]\n\
           spanda test [--project <dir>]\n\
           spanda add <package> [--version <ver>] [--path <dir>] [--git <url>]\n\
           spanda remove <package>\n\
           spanda install [--project <dir>]\n\
           spanda publish [--project <dir>]\n\
           spanda registry search <query>\n\
           spanda registry info <package>\n"
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

fn human_run(
    source: &str,
    file: &str,
    verbose: bool,
    trace_scheduler: bool,
    trace_tasks: bool,
    trace_triggers: bool,
    trace_events: bool,
) {
    let max_loop_iterations = if verbose { 20 } else { 10 };
    match run(
        source,
        RunOptions {
            max_loop_iterations,
            trace_scheduler,
            trace_tasks,
            trace_triggers,
            trace_events,
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
            if verbose || trace_scheduler || trace_tasks || trace_triggers || trace_events {
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
            if trace_scheduler || trace_tasks || trace_triggers || trace_events {
                println!("\n── Runtime Metrics ──");
                if trace_scheduler {
                    println!(
                        "  Scheduler: {} tick(s), base {}ms, {} multiplexed task(s)",
                        result.metrics.scheduler.scheduler_ticks,
                        result.metrics.scheduler.base_tick_ms,
                        result.metrics.scheduler.multiplexed_tasks
                    );
                }
                if trace_tasks && !result.metrics.tasks.is_empty() {
                    println!("  Tasks:");
                    for task in result.metrics.tasks.values() {
                        println!(
                            "    {} [{}]: ticks={}, skipped={}, missed_deadlines={}",
                            task.name,
                            task.priority,
                            task.ticks,
                            task.skipped,
                            task.missed_deadlines
                        );
                    }
                }
                if trace_triggers && !result.metrics.triggers.is_empty() {
                    println!("  Triggers:");
                    for trigger in result.metrics.triggers.values() {
                        println!(
                            "    {} [{}]: executions={}, failures={}, missed_deadlines={}",
                            trigger.name,
                            trigger.priority,
                            trigger.executions,
                            trigger.failures,
                            trigger.missed_deadlines
                        );
                    }
                }
                if result.metrics.execution.spawns > 0
                    || result.metrics.execution.joins > 0
                    || result.metrics.execution.parallel_blocks > 0
                {
                    println!(
                        "  Execution: spawns={}, joins={}, parallel_blocks={}",
                        result.metrics.execution.spawns,
                        result.metrics.execution.joins,
                        result.metrics.execution.parallel_blocks
                    );
                }
                if result.metrics.replay_frames > 0 {
                    println!("  Replay frames: {}", result.metrics.replay_frames);
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

fn fleet_dispatch(args: &[String]) {
    if args.first().map(String::as_str) != Some("run") {
        eprintln!("Usage: spanda fleet run [--json] [--trace-scheduler] [--trace-tasks] [--trace-triggers] [--trace-events] <file.sd>");
        process::exit(1);
    }
    let mut json = false;
    let mut trace_scheduler = false;
    let mut trace_tasks = false;
    let mut trace_triggers = false;
    let mut trace_events = false;
    let mut file: Option<String> = None;
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--json" => json = true,
            "--trace-scheduler" => trace_scheduler = true,
            "--trace-tasks" => trace_tasks = true,
            "--trace-triggers" => trace_triggers = true,
            "--trace-events" => trace_events = true,
            other if !other.starts_with('-') && file.is_none() => file = Some(other.to_string()),
            other => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
        }
    }
    let file = file.unwrap_or_else(|| {
        eprintln!("Missing file path");
        process::exit(1);
    });
    let source = read_source(&file);
    if json {
        print_fleet_json(
            &source,
            &file,
            trace_scheduler,
            trace_tasks,
            trace_triggers,
            trace_events,
        );
    } else {
        human_fleet_run(
            &source,
            &file,
            trace_scheduler,
            trace_tasks,
            trace_triggers,
            trace_events,
        );
    }
}

fn human_fleet_run(
    source: &str,
    file: &str,
    trace_scheduler: bool,
    trace_tasks: bool,
    trace_triggers: bool,
    trace_events: bool,
) {
    use spanda_core::ast::{Program, RobotDecl};
    use spanda_core::foundations::DeployDecl;

    println!("\n🛰️  Fleet run from {file}\n");
    if let Ok(tokens) = spanda_core::lexer::tokenize(source) {
        if let Ok(program) = spanda_core::parser::parse(tokens) {
            let Program::Program {
                deployments,
                robots,
                ..
            } = program;
            let has_peers = robots.iter().any(|r| {
                matches!(r, RobotDecl::RobotDecl { peer_robots, .. } if !peer_robots.is_empty())
            });
            for deploy in &deployments {
                let DeployDecl::DeployDecl {
                    robot_name,
                    targets,
                    ..
                } = deploy;
                for target in targets {
                    println!("  deploy {robot_name} -> {target}");
                }
            }
            for robot in &robots {
                let RobotDecl::RobotDecl {
                    name, peer_robots, ..
                } = robot;
                for peer in peer_robots {
                    let spanda_core::comm::PeerRobotDecl::PeerRobotDecl {
                        name: peer_name, ..
                    } = peer;
                    println!("  peer robot {name} knows {peer_name}");
                }
            }
            if !deployments.is_empty() || has_peers {
                println!();
            }
        }
    }
    let opts = RunOptions {
        max_loop_iterations: 20,
        trace_scheduler,
        trace_tasks,
        trace_triggers,
        trace_events,
        replay_trace: true,
        ..Default::default()
    };
    match run(source, opts) {
        Ok(result) => {
            let s = &result.state;
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
            if (trace_scheduler || trace_tasks) && !result.logs.is_empty() {
                println!("\n── Runtime Log ──");
                for log in &result.logs {
                    println!("  {log}");
                }
            }
            println!("\n✓ Fleet simulation complete\n");
        }
        Err(err) => {
            for d in err.diagnostics() {
                eprintln!("  [{}:{}] {}", d.line, d.column, d.message);
            }
            process::exit(1);
        }
    }
}

fn print_fleet_json(
    source: &str,
    _file: &str,
    trace_scheduler: bool,
    trace_tasks: bool,
    trace_triggers: bool,
    trace_events: bool,
) {
    let opts = RunOptions {
        max_loop_iterations: 20,
        trace_scheduler,
        trace_tasks,
        trace_triggers,
        trace_events,
        replay_trace: true,
        ..Default::default()
    };
    print_run_json(run(source, opts));
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
        "registry" => match rest.first().map(String::as_str) {
            Some("search") => package::cmd_registry_search(&rest[1..]),
            Some("info") => package::cmd_registry_info(&rest[1..]),
            _ => {
                eprintln!("Usage: spanda registry search <query> | spanda registry info <package>");
                process::exit(1);
            }
        },
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
    if command == "fleet" {
        fleet_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }
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
    let mut target_triple: Option<String> = None;
    let mut hal_profile: Option<String> = None;
    let mut codegen_target = CodegenTarget::Native;
    let mut breakpoints: Vec<u32> = Vec::new();
    let mut file: Option<String> = None;
    let mut trace_scheduler = false;
    let mut trace_tasks = false;
    let mut trace_triggers = false;
    let mut trace_events = false;
    let mut replay_trace = false;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            "--verbose" | "-v" => verbose = true,
            "--trace-scheduler" => trace_scheduler = true,
            "--trace-tasks" => trace_tasks = true,
            "--trace-triggers" => trace_triggers = true,
            "--trace-events" => trace_events = true,
            "--replay" => replay_trace = true,
            "--project" => project_mode = true,
            "--target" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--target requires a value");
                    process::exit(1);
                }
                match command {
                    "codegen" | "deploy" => {
                        codegen_target = match args[i].as_str() {
                            "native" => CodegenTarget::Native,
                            "wasm" => CodegenTarget::Wasm,
                            "esp32" => CodegenTarget::Esp32,
                            other => {
                                eprintln!("Unknown codegen target: {other}");
                                process::exit(1);
                            }
                        };
                    }
                    _ => target = Some(args[i].clone()),
                }
            }
            "--break" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--break requires a line number");
                    process::exit(1);
                }
                breakpoints.push(args[i].parse().unwrap_or_else(|_| {
                    eprintln!("Invalid line number for --break");
                    process::exit(1);
                }));
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
            "--target-triple" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--target-triple requires a value");
                    process::exit(1);
                }
                target_triple = Some(args[i].clone());
            }
            "--hal-profile" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--hal-profile requires a value");
                    process::exit(1);
                }
                hal_profile = Some(args[i].clone());
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
                trace_scheduler,
                trace_tasks,
                trace_triggers,
                trace_events,
                replay_trace: command == "sim" && replay_trace,
                ..Default::default()
            };
            if json {
                print_run_json(run(&source, opts));
            } else {
                human_run(
                    &source,
                    &file,
                    command == "sim" || verbose,
                    trace_scheduler,
                    trace_tasks,
                    trace_triggers,
                    trace_events,
                );
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
        "codegen" => {
            let file = file.unwrap_or_else(|| {
                eprintln!("Missing file path");
                usage();
                process::exit(1);
            });
            let source = read_source(&file);
            match codegen(&source, codegen_target) {
                Ok(output) => {
                    if let Some(ref out) = out_path {
                        fs::write(out, &output).unwrap_or_else(|e| {
                            eprintln!("Error writing {out}: {e}");
                            process::exit(1);
                        });
                        println!("✓ wrote codegen output to {out}");
                    } else {
                        print!("{output}");
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    process::exit(1);
                }
            }
        }
        "deploy" => {
            let file = file.unwrap_or_else(|| {
                eprintln!("Missing file path");
                usage();
                process::exit(1);
            });
            if codegen_target != CodegenTarget::Wasm {
                eprintln!("deploy currently supports --target wasm only");
                process::exit(1);
            }
            let source = read_source(&file);
            match wasm_deploy_manifest(&source) {
                Ok(manifest) => {
                    if let Some(ref out) = out_path {
                        fs::write(out, &manifest).unwrap_or_else(|e| {
                            eprintln!("Error writing {out}: {e}");
                            process::exit(1);
                        });
                        println!("✓ wrote wasm deploy manifest to {out}");
                    } else {
                        print!("{manifest}");
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    process::exit(1);
                }
            }
        }
        "ir" => {
            let file = file.unwrap_or_else(|| {
                eprintln!("Missing file path");
                usage();
                process::exit(1);
            });
            let source = read_source(&file);
            match lower_to_sir(&source) {
                Ok(sir) => {
                    if json {
                        let resp = IrResponse { ok: true, sir };
                        println!("{}", serde_json::to_string(&resp).unwrap());
                    } else {
                        println!("Spanda IR for {file}:");
                        println!("  module: {:?}", sir.module_name);
                        println!("  functions: {}", sir.functions.len());
                        println!("  externs: {}", sir.externs.len());
                        println!("  robots: {}", sir.robot_names.join(", "));
                        for ext in &sir.externs {
                            println!(
                                "  extern {} fn {} -> {}",
                                ext.bridge.as_str(),
                                ext.name,
                                ext.return_type
                            );
                        }
                    }
                }
                Err(e) => {
                    if json {
                        println!(
                            "{}",
                            serde_json::to_string(&serde_json::json!({
                                "ok": false,
                                "error": e.to_string()
                            }))
                            .unwrap()
                        );
                    } else {
                        eprintln!("Error: {e}");
                    }
                    process::exit(1);
                }
            }
        }
        "llvm-ir" => {
            let file = file.unwrap_or_else(|| {
                eprintln!("Missing file path");
                usage();
                process::exit(1);
            });
            let source = read_source(&file);
            match lower_to_sir(&source) {
                Ok(sir) => {
                    let ir = emit_module_ir_with_options(
                        &sir,
                        target_triple.as_deref(),
                        hal_profile.as_deref(),
                    );
                    if let Some(ref out) = out_path {
                        fs::write(out, &ir).unwrap_or_else(|e| {
                            eprintln!("Error writing {out}: {e}");
                            process::exit(1);
                        });
                        println!("✓ wrote LLVM IR to {out}");
                    } else {
                        print!("{ir}");
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    process::exit(1);
                }
            }
        }
        "compile-native" => {
            let file = file.unwrap_or_else(|| {
                eprintln!("Missing file path");
                usage();
                process::exit(1);
            });
            let source = read_source(&file);
            match lower_to_sir(&source) {
                Ok(sir) => {
                    let workspace = std::env::current_dir().unwrap_or_else(|_| ".".into());
                    let output = out_path
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|| workspace.join("target/spanda-native/spanda-program"));
                    match compile_native(
                        &sir,
                        &CompileNativeOptions {
                            output,
                            clang: None,
                            workspace_root: workspace,
                            target_triple,
                            hal_profile,
                        },
                    ) {
                        Ok(result) => {
                            println!("✓ wrote LLVM IR to {}", result.llvm_ir_path.display());
                            println!("✓ linked native binary to {}", result.executable.display());
                        }
                        Err(e) => {
                            eprintln!("Error: {e}");
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    process::exit(1);
                }
            }
        }
        "debug" => {
            let file = file.unwrap_or_else(|| {
                eprintln!("Missing file path");
                usage();
                process::exit(1);
            });
            let source = read_source(&file);
            let mut bp = HashSet::new();
            bp.extend(breakpoints);
            match run_debug(
                &source,
                DebugOptions {
                    breakpoints: bp,
                    step: false,
                    source_path: None,
                },
            ) {
                Ok(session) => {
                    if session.pauses.is_empty() {
                        println!("✓ {file} — completed without hitting breakpoints");
                    } else {
                        println!("Debug pauses in {file}:");
                        for pause in session.pauses {
                            println!("  line {} — {}", pause.line, pause.reason);
                        }
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
