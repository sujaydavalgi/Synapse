//! main support for Spanda.
//!
mod certify_cli;
mod deploy_ota;
mod swarm_cli;
mod package;

use serde::Serialize;
use spanda_core::{
    check, codegen, format_source, generate_cli_man_pages, generate_language_reference,
    generate_markdown, lint, lower_to_sir, playback_mission, replay_mission, run, run_debug,
    security_audit, security_check, verify_compatibility, wasm_deploy_manifest, CodegenTarget,
    CompatSeverity, DebugOptions, RunOptions, SchedulerClock, SecurityReport, SecuritySeverity,
    SpandaError, VerifyOptions,
};
use spanda_llvm::{compile_native, emit_module_ir_with_options, CompileNativeOptions};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process;

fn run_options_for_file(file: &str, opts: RunOptions) -> RunOptions {
    let mut opts = opts;
    opts.official_packages = package::official_packages_for_source(Path::new(file));
    opts
}

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
    // Usage.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::main::usage();

    // Produce eprintln! as the result.
    eprintln!(
        "Spanda Programming Language\n\n\
         Usage:\n\
           spanda check [--json] [<file.sd> | --project]\n\
           spanda verify [--json] [--target <HardwareProfile>] [--all-targets] [--simulate] [--strict-certify] <file.sd>\n\
           spanda certify prove [--json] [--strict] [--out <file.json>] <file.sd>\n\
           spanda compatibility [--json] [--target <HardwareProfile>] [--all-targets] [--simulate] [--strict-certify] <file.sd>\n\
           spanda run [--json] [--verbose] [--twin-export <replay.json>] [--trace-scheduler] [--trace-tasks] [--trace-triggers] [--trace-events] [--trace-realtime] [--metrics-json] [--record] [--enforce-certify] <file.sd>\n\
           spanda sim [--json] [--replay] [--twin-export <replay.json>] [--trace-realtime] [--metrics-json] [--record] [--trace-scheduler] [--trace-tasks] [--trace-triggers] [--trace-events] [--enforce-certify] <file.sd>\n\
           spanda replay <mission.trace> [--from T+mm:ss] [--deterministic] [--playback]\n\
           spanda twin export <file.sd> --out <replay.json>\n\
           spanda fleet run [--json] [--trace-scheduler] [--trace-tasks] [--trace-triggers] [--trace-events] <file.sd>\n\
           spanda fleet orchestrate [--json] [--remote] [--mesh-url <http(s)://host:port>] [--mesh-token <t>] <file.sd>\n\
           spanda swarm coordinate [--json] [--mesh-url <http(s)://host:port>] [--mesh-token <t>] <file.sd>\n\
           spanda fleet mesh start [--bind <addr>] [--token <t>] [--tls-cert <pem>] [--tls-key <pem>]\n\
           spanda fleet agent start [--bind <addr>] [--robot <name>] [--token <t>] [--tls-cert <pem>] [--tls-key <pem>]\n\
           spanda fleet agent register <RobotName> <http(s)://host:port> [--token <t>]\n\
           spanda fleet agent list [--json]\n\
           spanda fmt [--json] <file.sd>\n\
           spanda lint [--json] <file.sd>\n\
           spanda doc [--json] [--out <file.md>] <file.sd>\n\
           spanda reference [--json] [--out <file.md>] [--man-dir <dir>]\n\
           spanda codegen [--target native|wasm|esp32] [--out <file>] <file.sd>\n\
           {}\n\
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
           spanda verify-adapter [--project <dir>] [--import <path>] [--package <name>]\n\
           spanda registry search <query>\n\
           spanda registry info <package>\n\n\
         Security commands:\n\
           spanda security check [--json] <file.sd>\n\
           spanda security audit [--json] <file.sd>\n",
        deploy_ota::deploy_usage_lines()
    );
}

fn read_source(path: &str) -> String {
    // Read source.
    //
    // Parameters:
    // - `path` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::main::read_source(path);

    // Produce unwrap or else as the result.
    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading {path}: {e}");
        process::exit(1);
    })
}

fn print_check_json(err: Option<SpandaError>) {
    // Print check json.
    //
    // Parameters:
    // - `err` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::main::print_check_json(err);

    // Compute resp for the following logic.
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
    // Print run json.
    //
    // Parameters:
    // - `result` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::main::print_run_json(result);

    // Compute resp for the following logic.
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
    // Human check.
    //
    // Parameters:
    // - `source` — input value
    // - `file` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::main::human_check(source, file);

    // Match on check and handle each case.
    match check(source) {
        Ok(()) => {
            println!("✓ {file} — no type errors");
        }
        Err(e) => {
            eprintln!("Type errors:");

            // Process each diagnostic.
            for d in e.diagnostics() {
                eprintln!("  [{}:{}] {}", d.line, d.column, d.message);
            }
            process::exit(1);
        }
    }
}

fn human_replay(
    trace_file: &str,
    from: Option<&str>,
    deterministic: bool,
    playback: bool,
    as_json: bool,
) {
    use spanda_core::replay::{parse_replay_offset, MissionTrace};
    let trace = MissionTrace::load(trace_file).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    });
    let offset_ms = if let Some(raw) = from {
        parse_replay_offset(raw).unwrap_or_else(|e| {
            eprintln!("{e}");
            process::exit(1);
        })
    } else {
        0.0
    };
    let frames = trace.frames_from(offset_ms);

    // Apply recorded state snapshots without re-running program logic.
    if playback {
        let (report, state) = playback_mission(
            trace_file,
            RunOptions {
                replay_from_ms: Some(offset_ms),
                playback_wall_clock: true,
                ..Default::default()
            },
        )
        .unwrap_or_else(|e| {
            eprintln!("Playback failed: {e}");
            process::exit(1);
        });
        if as_json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "ok": true,
                    "mode": "playback",
                    "frames_applied": report.frames_applied,
                    "states_applied": report.states_applied,
                    "offset_ms": offset_ms,
                    "state": state,
                }))
                .unwrap()
            );
            return;
        }
        println!(
            "Playback {}: {} frames ({} with state) from {:.0}ms",
            trace_file, report.frames_applied, report.states_applied, offset_ms
        );
        println!(
            "  Final pose: x={:.3} y={:.3} θ={:.3}",
            state.pose.x, state.pose.y, state.pose.theta
        );
        return;
    }

    // Re-run the traced program and verify deterministic replay when requested.
    if deterministic {
        let source_path = resolve_trace_source(trace_file, &trace.source);
        let source = fs::read_to_string(&source_path).unwrap_or_else(|e| {
            eprintln!("Failed to read trace source '{source_path}': {e}");
            process::exit(1);
        });
        let (_, verification) = replay_mission(
            &source,
            trace_file,
            RunOptions {
                max_loop_iterations: 20,
                record_trace: true,
                trace_source: Some(trace.source.clone()),
                replay_from_ms: Some(offset_ms),
                replay_deterministic: true,
                ..Default::default()
            },
        )
        .unwrap_or_else(|e| {
            eprintln!("Replay failed: {e}");
            process::exit(1);
        });
        if as_json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "ok": verification.ok,
                    "source": trace.source,
                    "deterministic": true,
                    "offset_ms": offset_ms,
                    "matched": verification.matched,
                    "mismatches": verification.mismatches,
                }))
                .unwrap()
            );
        } else if verification.ok {
            println!(
                "✓ Deterministic replay verified for {} ({} frames from {:.0}ms)",
                trace_file, verification.matched, offset_ms
            );
        } else {
            eprintln!("✗ Deterministic replay mismatch for {trace_file}:");
            for mismatch in &verification.mismatches {
                eprintln!("  {mismatch}");
            }
            process::exit(1);
        }
        return;
    }

    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "ok": true,
                "source": trace.source,
                "deterministic": trace.deterministic,
                "offset_ms": offset_ms,
                "frames": frames,
            }))
            .unwrap()
        );
        return;
    }
    println!(
        "Replay {} ({} frames from {:.0}ms)",
        trace_file,
        frames.len(),
        offset_ms
    );
    for frame in frames.iter().take(20) {
        println!(
            "  t={:.1}ms {} {:?}",
            frame.sim_time_ms, frame.event, frame.payload
        );
    }
    if frames.len() > 20 {
        println!("  ... {} more frames", frames.len() - 20);
    }
}

fn resolve_trace_source(trace_file: &str, source: &str) -> String {
    // Resolve a trace source label to a readable `.sd` path.
    //
    // Parameters:
    // - `trace_file` — path to the `.trace` file
    // - `source` — source label stored in the trace
    //
    // Returns:
    // Best-effort source path for replay verification.
    //
    // Options:
    // None.
    //
    // Example:
    // let path = resolve_trace_source("mission.trace", "rover.sd");

    // Prefer an existing path verbatim when available.
    if Path::new(source).is_file() {
        return source.to_string();
    }

    // Fall back to a sibling of the trace file directory.
    if let Some(parent) = Path::new(trace_file).parent() {
        let candidate = parent.join(source);
        if candidate.is_file() {
            return candidate.to_string_lossy().into_owned();
        }
    }
    source.to_string()
}

fn human_run(source: &str, file: &str, command: &str, opts: RunOptions) {
    // Human run.
    //
    // Parameters:
    // - `source` — input value
    // - `file` — input value
    // - `verbose` — input value
    // - `trace_scheduler` — input value
    // - `trace_tasks` — input value
    // - `trace_triggers` — input value
    // - `trace_events` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::main::human_run(source, file, verbose, trace_scheduler, trace_tasks, trace_triggers, trace_events);

    // Compute max loop iterations for the following logic.
    let verbose = opts.max_loop_iterations > 10;
    let trace_scheduler = opts.trace_scheduler;
    let trace_tasks = opts.trace_tasks;
    let trace_triggers = opts.trace_triggers;
    let trace_events = opts.trace_events;

    // Match on run and handle each case.
    match run(source, opts.clone()) {
        Ok(result) => {
            let s = &result.state;
            println!("\n🤖 Running robot from {file}\n");
            println!("── Final State ──");
            println!(
                "  Pose:     x={:.3} m, y={:.3} m, θ={:.3} rad",
                s.pose.x, s.pose.y, s.pose.theta
            );

            // Emit output when z provides a z.
            if let Some(z) = s.pose.z {
                println!("  Altitude: z={z:.3} m");
            }
            println!(
                "  Velocity: linear={:.3} m/s, angular={:.3} rad/s",
                s.velocity.linear, s.velocity.angular
            );
            println!(
                "  E-stop:   {}",
                // Take this path when s.emergency stop { "ACTIVE" } else { "off" }.
                if s.emergency_stop { "ACTIVE" } else { "off" }
            );

            // Log scheduler decisions when scheduler tracing is enabled.
            if verbose || trace_scheduler || trace_tasks || trace_triggers || trace_events {
                println!("\n── Simulation Log ──");

                // Process each event.
                for event in &result.events {
                    println!("  {event}");
                }

                // Skip further work when logs is empty.
                if !result.logs.is_empty() {
                    println!("\n── Runtime Log ──");

                    // Process each log.
                    for log in &result.logs {
                        println!("  {log}");
                    }
                }
            }

            // Log scheduler decisions when scheduler tracing is enabled.
            if trace_scheduler || trace_tasks || trace_triggers || trace_events {
                println!("\n── Runtime Metrics ──");

                // Log scheduler decisions when scheduler tracing is enabled.
                if trace_scheduler {
                    println!(
                        "  Scheduler: {} tick(s), base {}ms, {} multiplexed task(s)",
                        result.metrics.scheduler.scheduler_ticks,
                        result.metrics.scheduler.base_tick_ms,
                        result.metrics.scheduler.multiplexed_tasks
                    );
                }

                // Skip further work when tasks is empty.
                if trace_tasks && !result.metrics.tasks.is_empty() {
                    println!("  Tasks:");

                    // Process each value.
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

                // Skip further work when triggers is empty.
                if trace_triggers && !result.metrics.triggers.is_empty() {
                    println!("  Triggers:");

                    // Evaluate each trigger definition.
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

                // Take this path when result.metrics.execution.spawns > 0.
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

                // Take this path when result.metrics.replay frames > 0.
                if result.metrics.replay_frames > 0 {
                    println!("  Replay frames: {}", result.metrics.replay_frames);
                }
            }

            // Report mission trace output path when recording is enabled.
            if opts.record_trace && result.mission_trace.is_some() {
                let path = opts.trace_output.clone().unwrap_or_else(|| {
                    if let Some(stem) = file.strip_suffix(".sd") {
                        format!("{stem}.trace")
                    } else {
                        format!("{file}.trace")
                    }
                });
                println!("  Mission trace: {path}");
            }
            let label = if command == "sim" {
                "Simulation"
            } else {
                "Run"
            };
            println!("\n✓ {label} complete\n");
        }
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}

fn human_verify(source: &str, file: &str, options: &VerifyOptions) {
    // Human verify.
    //
    // Parameters:
    // - `source` — input value
    // - `file` — input value
    // - `options` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::main::human_verify(source, file, options);

    // Match on verify compatibility and handle each case.
    match verify_compatibility(source, options) {
        Ok(report) => {
            println!("Hardware compatibility: {file}");

            // Emit output when target provides a t.
            if let Some(t) = &report.target {
                println!("Target: {t}\n");
            }

            // Handle each entry in items.
            for item in &report.items {
                let icon = match item.severity {
                    CompatSeverity::Pass => "✓",
                    CompatSeverity::Warning => "⚠",
                    CompatSeverity::Error => "✗",
                };
                println!("  {icon} [{}] {}", item.category, item.message);
            }

            // Emit output when matrix provides a matrix.
            if let Some(matrix) = &report.matrix {
                println!("\n── Compatibility Matrix ──");

                // Process each cell.
                for cell in &matrix.cells {
                    let icon = if cell.compatible { "✓" } else { "✗" };
                    println!("  {icon} {} → {}", cell.robot, cell.target);
                }
                let compatible = matrix.cells.iter().filter(|c| c.compatible).count();
                let total = matrix.cells.len();
                println!("\n{compatible}/{total} robot × target pairs compatible");
                return;
            }

            // Take this path when report.compatible.
            if report.compatible {
                println!("\n✓ Deployment compatible");
            } else {
                println!("\n✗ Deployment incompatible");
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");

            // Process each diagnostic.
            for d in e.diagnostics() {
                eprintln!("  [{}:{}] {}", d.line, d.column, d.message);
            }
            process::exit(1);
        }
    }
}

fn print_verify_json(result: Result<spanda_core::CompatibilityReport, SpandaError>) {
    // Print verify json.
    //
    // Parameters:
    // - `result` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::main::print_verify_json(result);

    // Compute resp for the following logic.
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
    //
    // Parameters:
    // - `cmd` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::main::is_package_command(cmd);

    // Produce matches! as the result.
    matches!(
        cmd,
        "init" | "build" | "test" | "add" | "remove" | "publish" | "install" | "registry"
            | "verify-adapter"
    )
}

fn twin_dispatch(args: &[String]) {
    // Twin dispatch.
    //
    // Parameters:
    // - `args` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::main::twin_dispatch(args);

    if args.first().map(String::as_str) != Some("export") {
        eprintln!("Usage: spanda twin export <file.sd> --out <replay.json>");
        process::exit(1);
    }
    let mut file: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--out requires a path");
                    process::exit(1);
                }
                out_path = Some(args[i].clone());
            }
            other if !other.starts_with('-') && file.is_none() => file = Some(other.to_string()),
            other => {
                eprintln!("Unknown argument: {other}");
                process::exit(1);
            }
        }
        i += 1;
    }
    let file = file.unwrap_or_else(|| {
        eprintln!("Missing file path");
        process::exit(1);
    });
    let out_path = out_path.unwrap_or_else(|| {
        eprintln!("--out is required");
        process::exit(1);
    });
    let source = read_source(&file);
    let entry_behavior = spanda_core::compile(&source).ok().and_then(|compiled| {
        let spanda_core::Program::Program { robots, .. } = compiled.program;
        robots.first().and_then(|robot| {
            let spanda_core::RobotDecl::RobotDecl {
                behaviors, tasks, ..
            } = robot;
            behaviors
                .first()
                .map(|behavior| {
                    let spanda_core::BehaviorDecl::BehaviorDecl { name, .. } = behavior;
                    name.clone()
                })
                .or_else(|| {
                    tasks.first().map(|task| {
                        let spanda_core::foundations::TaskDecl::TaskDecl { name, .. } = task;
                        name.clone()
                    })
                })
        })
    });
    let opts = RunOptions {
        max_loop_iterations: 50,
        entry_behavior,
        twin_export_path: Some(out_path.clone()),
        ..Default::default()
    };
    match run(&source, opts) {
        Ok(result) => {
            if result.twin_replay.is_some() {
                println!("✓ twin replay exported to {out_path}");
            } else {
                eprintln!("No twin replay buffer in {file} — add `twin {{ ... replay true; }}`");
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}

fn fleet_dispatch(args: &[String]) {
    // Fleet dispatch.
    //
    // Parameters:
    // - `args` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::main::fleet_dispatch(args);

    // take the branch when as str) differs from Some.
    if args.first().map(String::as_str) == Some("orchestrate") {
        deploy_ota::fleet_orchestrate_dispatch(&args[1..]);
        return;
    }
    if args.first().map(String::as_str) == Some("agent") {
        deploy_ota::fleet_agent_dispatch(&args[1..]);
        return;
    }
    if args.first().map(String::as_str) == Some("mesh") {
        deploy_ota::fleet_mesh_dispatch(&args[1..]);
        return;
    }
    if args.first().map(String::as_str) != Some("run") {
        eprintln!("Usage: spanda fleet run [--json] [--trace-*] <file.sd>");
        eprintln!("       spanda fleet orchestrate [--json] [--remote] [--mesh-url <url>] <file.sd>");
        eprintln!("       spanda fleet agent start|register|list");
        eprintln!("       spanda fleet mesh start [--bind <addr>]");
        process::exit(1);
    }
    let mut json = false;
    let mut trace_scheduler = false;
    let mut trace_tasks = false;
    let mut trace_triggers = false;
    let mut trace_events = false;
    let mut file: Option<String> = None;

    // Apply each command-line argument.
    for arg in args.iter().skip(1) {
        // Match on as str and handle each case.
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

    // Take this path when json.
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
    // Human fleet run.
    //
    // Parameters:
    // - `source` — input value
    // - `file` — input value
    // - `trace_scheduler` — input value
    // - `trace_tasks` — input value
    // - `trace_triggers` — input value
    // - `trace_events` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::main::human_fleet_run(source, file, trace_scheduler, trace_tasks, trace_triggers, trace_events);

    // Import the items needed by the logic below.
    use spanda_core::ast::{Program, RobotDecl};
    use spanda_core::foundations::DeployDecl;
    println!("\n🛰️  Fleet run from {file}\n");

    // Handle the success value from tokenize.
    if let Ok(tokens) = spanda_core::lexer::tokenize(source) {
        // Handle the success value from parse.
        if let Ok(program) = spanda_core::parser::parse(tokens) {
            let Program::Program {
                deployments,
                robots,
                ..
            } = program;
            let has_peers = robots.iter().any(|r| {
                matches!(r, RobotDecl::RobotDecl { peer_robots, .. } if !peer_robots.is_empty())
            });

            // Process each deployment.
            for deploy in &deployments {
                let DeployDecl::DeployDecl {
                    robot_name,
                    targets,
                    ..
                } = deploy;

                // Process each target.
                for target in targets {
                    println!("  deploy {robot_name} -> {target}");
                }
            }

            // Handle each robot declared in the program.
            for robot in &robots {
                let RobotDecl::RobotDecl {
                    name, peer_robots, ..
                } = robot;

                // Process each peer robot.
                for peer in peer_robots {
                    let spanda_core::comm::PeerRobotDecl::PeerRobotDecl {
                        name: peer_name, ..
                    } = peer;
                    println!("  peer robot {name} knows {peer_name}");
                }
            }

            // Skip further work when !deployments is empty.
            if !deployments.is_empty() || has_peers {
                println!();
            }
        }
    }
    let opts = run_options_for_file(
        file,
        RunOptions {
            max_loop_iterations: 20,
            trace_scheduler,
            trace_tasks,
            trace_triggers,
            trace_events,
            replay_trace: true,
            ..Default::default()
        },
    );

    // Match on run and handle each case.
    match run(source, opts) {
        Ok(result) => {
            let s = &result.state;
            println!("── Final State ──");
            println!(
                "  Pose:     x={:.3} m, y={:.3} m, θ={:.3} rad",
                s.pose.x, s.pose.y, s.pose.theta
            );

            // Emit output when z provides a z.
            if let Some(z) = s.pose.z {
                println!("  Altitude: z={z:.3} m");
            }
            println!(
                "  Velocity: linear={:.3} m/s, angular={:.3} rad/s",
                s.velocity.linear, s.velocity.angular
            );
            println!(
                "  E-stop:   {}",
                // Take this path when s.emergency stop { "ACTIVE" } else { "off" }.
                if s.emergency_stop { "ACTIVE" } else { "off" }
            );

            // Skip further work when logs is empty.
            if (trace_scheduler || trace_tasks) && !result.logs.is_empty() {
                println!("\n── Runtime Log ──");

                // Process each log.
                for log in &result.logs {
                    println!("  {log}");
                }
            }
            println!("\n✓ Fleet simulation complete\n");
        }
        Err(err) => {
            // Process each diagnostic.
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
    // Print fleet json.
    //
    // Parameters:
    // - `source` — input value
    // - `_file` — input value
    // - `trace_scheduler` — input value
    // - `trace_tasks` — input value
    // - `trace_triggers` — input value
    // - `trace_events` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::main::print_fleet_json(source, _file, trace_scheduler, trace_tasks, trace_triggers, trace_events);

    // Compute opts for the following logic.
    let opts = run_options_for_file(
        _file,
        RunOptions {
            max_loop_iterations: 20,
            trace_scheduler,
            trace_tasks,
            trace_triggers,
            trace_events,
            replay_trace: true,
            ..Default::default()
        },
    );
    print_run_json(run(source, opts));
}

fn security_dispatch(rest: &[String]) {
    let sub = rest.first().map(String::as_str).unwrap_or("");
    if sub != "check" && sub != "audit" {
        eprintln!("Usage: spanda security check|audit [--json] <file.sd>");
        process::exit(1);
    }
    let mut json = false;
    let mut file: Option<String> = None;
    let mut i = 1;
    while i < rest.len() {
        match rest[i].as_str() {
            "--json" => json = true,
            other if !other.starts_with('-') && file.is_none() => file = Some(other.to_string()),
            other => {
                eprintln!("Unknown argument: {other}");
                eprintln!("Usage: spanda security check|audit [--json] <file.sd>");
                process::exit(1);
            }
        }
        i += 1;
    }
    let file = file.unwrap_or_else(|| {
        eprintln!("Missing file path");
        process::exit(1);
    });
    let source = read_source(&file);
    let result = if sub == "audit" {
        security_audit(&source)
    } else {
        security_check(&source)
    };
    match result {
        Ok(report) => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string(&SecurityReportJson::from(&report)).unwrap()
                );
            } else {
                human_security_report(&file, sub, &report);
            }
            if report.has_errors() {
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    }
}

#[derive(Serialize)]
struct SecurityReportJson {
    ok: bool,
    findings: Vec<SecurityFindingJson>,
}

#[derive(Serialize)]
struct SecurityFindingJson {
    severity: String,
    message: String,
    line: u32,
    column: u32,
}

impl From<&SecurityReport> for SecurityReportJson {
    fn from(report: &SecurityReport) -> Self {
        Self {
            ok: !report.has_errors(),
            findings: report
                .findings
                .iter()
                .map(|f| SecurityFindingJson {
                    severity: match f.severity {
                        SecuritySeverity::Error => "error".into(),
                        SecuritySeverity::Warning => "warning".into(),
                        SecuritySeverity::Info => "info".into(),
                    },
                    message: f.message.clone(),
                    line: f.line,
                    column: f.column,
                })
                .collect(),
        }
    }
}

fn human_security_report(file: &str, mode: &str, report: &SecurityReport) {
    if report.findings.is_empty() {
        println!("✓ {file} — no security {mode} findings");
        return;
    }
    for f in &report.findings {
        let tag = match f.severity {
            SecuritySeverity::Error => "error",
            SecuritySeverity::Warning => "warn",
            SecuritySeverity::Info => "info",
        };
        println!("  [{tag}] [{}:{}] {}", f.line, f.column, f.message);
    }
    if report.has_errors() {
        eprintln!("Security {mode} failed for {file}");
    } else {
        println!("✓ {file} — security {mode} passed with warnings/info");
    }
}

fn dispatch_package(command: &str, rest: &[String]) {
    //
    // Parameters:
    // - `command` — input value
    // - `rest` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::main::dispatch_package(command, rest);

    // Match on command and handle each case.
    match command {
        "init" => package::cmd_init(rest),
        "build" => package::cmd_build(rest),
        "test" => package::cmd_test(rest),
        "add" => package::cmd_add(rest),
        "remove" => package::cmd_remove(rest),
        "install" => package::cmd_install(rest),
        "publish" => package::cmd_publish(rest),
        "verify-adapter" => package::cmd_verify_adapter(rest),
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
    // Main.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_cli::main::main();

    // Compute args for the following logic.
    let args: Vec<String> = env::args().collect();

    // Take the branch when len equals "--help" || args[1] == "-h".
    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        usage();
        process::exit(if args.len() < 2 { 1 } else { 0 });
    }
    let command = args[1].as_str();

    if command == "certify" {
        certify_cli::certify_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    // Take the branch when command equals "fleet".
    if command == "fleet" {
        fleet_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "swarm" {
        swarm_cli::swarm_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "deploy" && args.len() > 2 {
        match args[2].as_str() {
            "plan" | "rollout" | "rollback" | "status" => {
                deploy_ota::deploy_dispatch(&args[2..]);
                let _ = io::stdout().flush();
                return;
            }
            _ => {}
        }
    }

    // Take the branch when command equals "twin".
    if command == "twin" {
        twin_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "security" {
        security_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    // Take this path when is package command(command).
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
    let mut strict_certify = false;
    let mut project_mode = false;
    let mut out_path: Option<String> = None;
    let mut man_dir: Option<String> = None;
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
    let mut trace_realtime = false;
    let mut record_trace = false;
    let mut metrics_json = false;
    let mut replay_deterministic = false;
    let mut replay_playback = false;
    let mut replay_from: Option<String> = None;
    let mut trace_output: Option<String> = None;
    let mut twin_export_path: Option<String> = None;
    let mut wall_clock = false;
    let mut secure_mode = false;
    let mut inject_security_faults = false;
    let mut enforce_certify = false;
    let mut i = 2;

    // Repeat while i < args.len().
    while i < args.len() {
        // Match on as str and handle each case.
        match args[i].as_str() {
            "--json" => json = true,
            "--verbose" | "-v" => verbose = true,
            "--trace-scheduler" => trace_scheduler = true,
            "--trace-tasks" => trace_tasks = true,
            "--trace-triggers" => trace_triggers = true,
            "--trace-events" => trace_events = true,
            "--replay" => replay_trace = true,
            "--trace-realtime" => trace_realtime = true,
            "--metrics-json" => {
                metrics_json = true;
                json = true;
            }
            "--record" => record_trace = true,
            "--secure" => secure_mode = true,
            "--inject-security-faults" => inject_security_faults = true,
            "--enforce-certify" => enforce_certify = true,
            "--twin-export" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--twin-export requires a path");
                    process::exit(1);
                }
                twin_export_path = Some(args[i].clone());
            }
            "--trace-out" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--trace-out requires a path");
                    process::exit(1);
                }
                trace_output = Some(args[i].clone());
            }
            "--deterministic" => replay_deterministic = true,
            "--playback" => replay_playback = true,
            "--wall-clock" => wall_clock = true,
            "--from" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--from requires a replay offset (e.g. T+00:30)");
                    process::exit(1);
                }
                replay_from = Some(args[i].clone());
            }
            "--project" => project_mode = true,
            "--target" => {
                i += 1;

                // Take this path when i >= args.len().
                if i >= args.len() {
                    eprintln!("--target requires a value");
                    process::exit(1);
                }

                // Match on command and handle each case.
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

                // Take this path when i >= args.len().
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
            "--strict-certify" => strict_certify = true,
            "--out" => {
                i += 1;

                // Take this path when i >= args.len().
                if i >= args.len() {
                    eprintln!("--out requires a file path");
                    process::exit(1);
                }
                out_path = Some(args[i].clone());
            }
            "--man-dir" => {
                i += 1;

                // Take this path when i >= args.len().
                if i >= args.len() {
                    eprintln!("--man-dir requires a directory path");
                    process::exit(1);
                }
                man_dir = Some(args[i].clone());
            }
            "--target-triple" => {
                i += 1;

                // Take this path when i >= args.len().
                if i >= args.len() {
                    eprintln!("--target-triple requires a value");
                    process::exit(1);
                }
                target_triple = Some(args[i].clone());
            }
            "--hal-profile" => {
                i += 1;

                // Take this path when i >= args.len().
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

    if command == "replay" {
        let trace_file = file.unwrap_or_else(|| {
            eprintln!("Missing trace file path");
            usage();
            process::exit(1);
        });
        human_replay(
            &trace_file,
            replay_from.as_deref(),
            replay_deterministic,
            replay_playback,
            json,
        );
        let _ = io::stdout().flush();
        return;
    }

    // Match on command and handle each case.
    match command {
        "check" => {
            // Take this path when project mode || file.is none().
            if project_mode || file.is_none() {
                package::cmd_check_project(&args[2..]);
            } else if let Some(ref file_path) = file {
                let source = read_source(file_path);

                // Take this path when json.
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
                strict_certify,
            };

            // Take this path when json.
            if json {
                let result = verify_compatibility(&source, &options);
                let failed = match &result {
                    Ok(report) => !options.all_targets && !report.compatible,
                    Err(_) => true,
                };
                print_verify_json(result);

                // Take this path when failed.
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
            let opts = run_options_for_file(
                &file,
                RunOptions {
                    max_loop_iterations,
                    trace_scheduler: trace_scheduler || trace_realtime,
                    trace_tasks: trace_tasks || trace_realtime,
                    trace_triggers: trace_triggers || trace_realtime,
                    trace_events: trace_events || trace_realtime,
                    trace_realtime,
                    record_trace: record_trace || (command == "sim" && replay_trace),
                    trace_source: Some(file.clone()),
                    trace_output,
                    metrics_json,
                    replay_trace: command == "sim" && replay_trace,
                    replay_deterministic,
                    scheduler_clock: if wall_clock {
                        SchedulerClock::Wall
                    } else {
                        SchedulerClock::Sim
                    },
                    twin_export_path,
                    secure_mode,
                    inject_security_faults,
                    enforce_certify,
                    ..Default::default()
                },
            );

            // Take this path when json.
            if json || metrics_json {
                print_run_json(run(&source, opts));
            } else {
                human_run(&source, &file, command, opts);
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

            // Take this path when json.
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

            // Match on lint and handle each case.
            match lint(&source) {
                Ok(report) => {
                    // Take this path when json.
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

                        // Process each issue.
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

                    // Take this path when report.has errors().
                    if report.has_errors() {
                        process::exit(1);
                    }
                }
                Err(e) => {
                    // Take this path when json.
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

            // Match on generate markdown and handle each case.
            match generate_markdown(&source) {
                Ok(markdown) => {
                    // Take this path when let Some(ref out) = out path.
                    if let Some(ref out) = out_path {
                        fs::write(out, &markdown).unwrap_or_else(|e| {
                            eprintln!("Error writing {out}: {e}");
                            process::exit(1);
                        });

                        // Take the branch when json is false.
                        if !json {
                            println!("✓ wrote docs to {out}");
                        }
                    } else if !json {
                        print!("{markdown}");
                    }

                    // Take this path when json.
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
        "reference" => {
            let markdown = generate_language_reference();

            // Take this path when let Some(ref out) = out path.
            if let Some(ref out) = out_path {
                fs::write(out, &markdown).unwrap_or_else(|e| {
                    eprintln!("Error writing {out}: {e}");
                    process::exit(1);
                });

                // Take the branch when json is false.
                if !json {
                    println!("✓ wrote language reference to {out}");
                }
            } else if !json {
                print!("{markdown}");
            }

            // Write man-page markdown files when --man-dir is set.
            if let Some(ref dir) = man_dir {
                fs::create_dir_all(dir).unwrap_or_else(|e| {
                    eprintln!("Error creating {dir}: {e}");
                    process::exit(1);
                });
                for (name, body) in generate_cli_man_pages() {
                    let path = Path::new(dir).join(&name);
                    fs::write(&path, &body).unwrap_or_else(|e| {
                        eprintln!("Error writing {}: {e}", path.display());
                        process::exit(1);
                    });
                }
                if !json {
                    println!("✓ wrote man pages to {dir}");
                }
            }

            // Take this path when json.
            if json {
                let resp = DocResponse { ok: true, markdown };
                println!("{}", serde_json::to_string(&resp).unwrap());
            }
        }
        "codegen" => {
            let file = file.unwrap_or_else(|| {
                eprintln!("Missing file path");
                usage();
                process::exit(1);
            });
            let source = read_source(&file);

            // Match on codegen and handle each case.
            match codegen(&source, codegen_target) {
                Ok(output) => {
                    // Take this path when let Some(ref out) = out path.
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

            // Take the branch when codegen target differs from Wasm.
            if codegen_target != CodegenTarget::Wasm {
                eprintln!("deploy currently supports --target wasm only");
                process::exit(1);
            }
            let source = read_source(&file);

            // Match on wasm deploy manifest and handle each case.
            match wasm_deploy_manifest(&source) {
                Ok(manifest) => {
                    // Take this path when let Some(ref out) = out path.
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

            // Match on lower to sir and handle each case.
            match lower_to_sir(&source) {
                Ok(sir) => {
                    // Take this path when json.
                    if json {
                        let resp = IrResponse { ok: true, sir };
                        println!("{}", serde_json::to_string(&resp).unwrap());
                    } else {
                        println!("Spanda IR for {file}:");
                        println!("  module: {:?}", sir.module_name);
                        println!("  functions: {}", sir.functions.len());
                        println!("  externs: {}", sir.externs.len());
                        println!("  robots: {}", sir.robot_names.join(", "));

                        // Declare each extern function in the generated output.
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
                    // Take this path when json.
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

            // Match on lower to sir and handle each case.
            match lower_to_sir(&source) {
                Ok(sir) => {
                    let ir = emit_module_ir_with_options(
                        &sir,
                        target_triple.as_deref(),
                        hal_profile.as_deref(),
                    );

                    // Take this path when let Some(ref out) = out path.
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

            // Match on lower to sir and handle each case.
            match lower_to_sir(&source) {
                Ok(sir) => {
                    let workspace = std::env::current_dir().unwrap_or_else(|_| ".".into());
                    let output = out_path
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|| workspace.join("target/spanda-native/spanda-program"));

                    // Match on compile native and handle each case.
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

            // Match on run debug and handle each case.
            match run_debug(
                &source,
                DebugOptions {
                    breakpoints: bp,
                    step: false,
                    source_path: None,
                },
            ) {
                Ok(session) => {
                    // Skip further work when pauses is empty.
                    if session.pauses.is_empty() {
                        println!("✓ {file} — completed without hitting breakpoints");
                    } else {
                        println!("Debug pauses in {file}:");

                        // Process each pause.
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
