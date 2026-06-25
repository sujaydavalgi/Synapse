//! main support for Spanda.
//!
mod assurance_cli;
mod certify_cli;
mod config_cli;
mod config_load;
mod continuity_cli;
mod contract_cli;
mod decision_cli;
mod demo_cli;
mod deploy_ota;
mod device_cli;
mod device_tree_cli;
mod drift_cli;
mod explain_cli;
mod graph_cli;
mod fault_cli;
mod network_cli;
mod package;
mod readiness_cli;
mod recovery_cli;
mod replay_cli;
mod ros2_cli;
mod swarm_cli;
mod telemetry_cli;
mod trace_cli;

use serde::Serialize;
use spanda_ast::comm_decl::PeerRobotDecl;
use spanda_ast::foundations::{DeployDecl, TaskDecl};
use spanda_ast::nodes::{BehaviorDecl, Program, RobotDecl};
use spanda_codegen::{generate as codegen, wasm_deploy_manifest, CodegenTarget};
use spanda_debug::DebugOptions;
use spanda_docs::{
    generate_cli_man_pages, generate_docs_for_path, generate_html, generate_json_docs,
    generate_language_reference, generate_markdown, list_man_pages, lookup_man_page,
    markdown_man_to_roff,
};
use spanda_driver::{
    check, compile, compile_with_registry, lower_to_sir, run, run_debug, tokenize, RunOptions,
    RunResult,
};
use spanda_error::SpandaError;
use spanda_format::format_source;
use spanda_hardware::{
    CompatItem, CompatSeverity, CompatibilityMatrix, CompatibilityReport, VerifyOptions,
};
use spanda_lint::{lint, LintIssue, LintSeverity};
#[cfg(feature = "llvm")]
use spanda_llvm::{compile_native, emit_module_ir_with_options, CompileNativeOptions};
use spanda_parser::parse;
use spanda_runtime::scheduler::SchedulerClock;
use spanda_security::validate::{security_audit, security_check, SecurityReport, SecuritySeverity};
use spanda_sir::SirProgram;
use spanda_typecheck::Diagnostic;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process;

fn run_options_for_file(file: &str, opts: RunOptions, config_flag: Option<&Path>) -> RunOptions {
    // Description:
    //     Run options for file.
    //
    // Inputs:
    //     file: &str
    //         Caller-supplied file.
    //     opts: RunOptions
    //         Caller-supplied opts.
    //     config_flag: Option<&Path>
    //         Optional explicit spanda.toml path.
    //
    // Outputs:
    //     result: RunOptions
    //         Return value from `run_options_for_file`.
    //
    // Example:

    //     let result = spanda_cli::main::run_options_for_file(file, opts, None);

    let path = Path::new(file);
    let cfg = config_load::load_system_config(path, config_flag);
    config_load::ensure_config_valid(cfg.as_ref().map(|a| a.as_ref()));
    config_load::apply_system_config_to_run_options(cfg, opts, path)
}

#[cfg(not(feature = "llvm"))]
fn llvm_unavailable() -> ! {
    // Description:
    //     Llvm unavailable.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: !
    //         Return value from `llvm_unavailable`.
    //
    // Example:

    //     let result = spanda_cli::main::llvm_unavailable();

    eprintln!("LLVM commands require the `llvm` feature (enabled by default)");
    eprintln!("Rebuild with: cargo build -p spanda --features llvm");
    process::exit(1);
}

#[derive(Serialize)]
struct CheckResponse {
    ok: bool,
    diagnostics: Vec<Diagnostic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verification: Option<Vec<spanda_capability::VerificationDiagnostic>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    readiness: Option<spanda_readiness::ReadinessReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    readiness_diagnostics: Option<Vec<spanda_capability::VerificationDiagnostic>>,
}

#[derive(Serialize)]
struct RunResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<RunResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostics: Option<Vec<Diagnostic>>,
}

#[derive(Serialize)]
struct VerifyResponse {
    ok: bool,
    compatible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<String>,
    items: Vec<CompatItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    matrix: Option<CompatibilityMatrix>,
}

#[derive(Serialize)]
struct LintResponse {
    ok: bool,
    issues: Vec<LintIssue>,
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
    sir: SirProgram,
}

fn usage() {
    // Description:
    //     Usage.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:
    //     let result = spanda_cli::main::usage();

    // Produce eprintln! as the result.
    eprintln!(
        "Spanda Programming Language\n\n\
         Usage:\n\
           spanda check [--json] [--verification-json] [<file.sd> | --project]\n\
           spanda verify [--json] [--target <HardwareProfile>] [--all-targets] [--simulate] [--strict-certify] <file.sd>\n\
           spanda certify prove [--json] [--strict] [--out <file.json>] <file.sd>\n\
           spanda compatibility [--json] [--target <HardwareProfile>] [--all-targets] [--simulate] [--strict-certify] <file.sd>\n\
           spanda run [--json] [--verbose] [--twin-export <replay.json>] [--trace-scheduler] [--trace-tasks] [--trace-triggers] [--trace-events] [--trace-providers] [--trace-realtime] [--metrics-json] [--record] [--persist-telemetry] [--enforce-certify] <file.sd>\n\
           spanda sim [--json] [--replay] [--twin-export <replay.json>] [--trace-realtime] [--metrics-json] [--record] [--persist-telemetry] [--trace-scheduler] [--trace-tasks] [--trace-triggers] [--trace-events] [--trace-providers] [--enforce-certify] <file.sd>\n\
           spanda replay <mission.trace> [--from T+mm:ss] [--deterministic] [--playback] [--config <spanda.toml>]\n\
           spanda twin export <file.sd> --out <replay.json>\n\
           spanda fleet run [--json] [--trace-scheduler] [--trace-tasks] [--trace-triggers] [--trace-events] [--persist-telemetry] <file.sd>\n\
           spanda fleet orchestrate [--json] [--remote] [--mesh-url <http(s)://host:port>] [--mesh-token <t>] <file.sd>\n\
           spanda swarm coordinate [--json] [--mesh-url <http(s)://host:port>] [--mesh-token <t>] <file.sd>\n\
           spanda fleet mesh start [--bind <addr>] [--token <t>] [--tls-cert <pem>] [--tls-key <pem>]\n\
           spanda fleet agent start [--bind <addr>] [--robot <name>] [--token <t>] [--tls-cert <pem>] [--tls-key <pem>]\n\
           spanda fleet agent register <RobotName> <http(s)://host:port> [--token <t>]\n\
           spanda fleet agent list [--json]\n\
           spanda fmt [--json] <file.sd>\n\
           spanda lint [--json] <file.sd>\n\
           spanda doc [--json] [--html] [--out <file|dir>] <file.sd|dir/>\n\
           spanda man [<command>] [--roff]\n\
           spanda reference [--json] [--out <file.md>] [--man-dir <dir>]\n\
           spanda codegen [--target native|wasm|esp32] [--out <file>] <file.sd>\n\
           {}\n\
           spanda debug [--break <line>] <file.sd>\n\
           spanda ir [--json] <file.sd>\n\
           spanda llvm-ir [--out <file.ll>] [--target-triple <triple>] [--hal-profile <name>] <file.sd>\n\
           spanda compile-native [--out <binary>] [--target-triple <triple>] [--hal-profile <name>] <file.sd>\n\n\
         Demo commands:\n\
           spanda demo <rover|safety|verify|fleet|health|readiness|assurance|self-healing|maturity>\n\n\
         ROS 2 commands:\n\
           spanda ros2 check [--json]\n\n\
         Package commands:\n\
           spanda init [name] [--description <text>]\n\
           spanda build [--project <dir>]\n\
           spanda test [--project <dir>]\n\
           spanda add <package> [--version <ver>] [--path <dir>] [--git <url>]\n\
           spanda remove <package>\n\
           spanda install [--project <dir>]\n\
           spanda update [--project <dir>]\n\
           spanda publish [--project <dir>]\n\
           spanda verify-adapter [--project <dir>] [--import <path>] [--package <name>]\n\
           spanda registry search <query>\n\
           spanda registry info <package>\n\n\
         Configuration commands:\n\
           spanda config resolve [--json] [--config <spanda.toml>]\n\
           spanda config validate [--json] [--config <spanda.toml>]\n\
           spanda config graph [--json] [--config <spanda.toml>]\n\
           spanda config diff <base.toml> <other.toml> [--json]\n\
           spanda config drift --baseline <dir|spanda.toml> [--config <spanda.toml>] [program.sd] [--json]\n\
           spanda config report [--json] [--network] [--config <spanda.toml>]\n\
           spanda drift <file.sd> [--agent <Robot@Hardware>] [--config <spanda.toml>] [--json]\n\
           spanda drift --baseline <dir|spanda.toml> [--config <spanda.toml>] [program.sd] [--json]\n\
           spanda device discover [--subnet CIDR] [--json] [--config <spanda.toml>]\n\
           spanda device inspect <id> [--json] [--config <spanda.toml>]\n\
           spanda device-tree inspect <robot-id> [--json] [--config <spanda.toml>]\n\
           spanda device-tree graph [--json] [--config <spanda.toml>]\n\
           spanda network scan --subnet <CIDR> [--json] [--ports 80,443,554]\n\
           spanda map verify <file.sd> [--config <spanda.toml>] [--json]\n\n\
         Security commands:\n\
           spanda security check [--json] <file.sd>\n\
           spanda security audit [--json] <file.sd>\n\n\
         Analysis commands:\n\
           spanda graph <file.sd> [--format json|mermaid|dot|text] [--json] [--config <spanda.toml>]\n\
           spanda trust <package> [--version <ver>] [--project <dir>] [--json]\n",
        deploy_ota::deploy_usage_lines()
    );
}

fn read_source(path: &str) -> String {
    // Description:
    //     Read source.
    //
    // Inputs:
    //     path: &str
    //         Caller-supplied path.
    //
    // Outputs:
    //     result: String
    //         Return value from `read_source`.
    //
    // Example:
    //     let result = spanda_cli::main::read_source(path);

    // Produce unwrap or else as the result.
    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading {path}: {e}");
        process::exit(1);
    })
}

fn print_check_json(
    err: Option<SpandaError>,
    verification: Option<Vec<spanda_capability::VerificationDiagnostic>>,
    readiness: Option<spanda_readiness::ReadinessReport>,
    readiness_diagnostics: Option<Vec<spanda_capability::VerificationDiagnostic>>,
) {
    // Description:
    //     Print check json.
    //
    // Inputs:
    //     err: Option<SpandaError>
    //         Caller-supplied err.
    //     verification: Option<Vec<spanda_capability::VerificationDiagnostic>>
    //         Caller-supplied verification.
    //     readiness: Option<spanda_readiness::ReadinessReport>
    //         Caller-supplied readiness.
    //     readiness_diagnostics: Option<Vec<spanda_capability::VerificationDiagnostic>>
    //         Caller-supplied readiness diagnostics.
    //
    // Outputs:
    //     None.
    //
    // Example:
    //     let result = spanda_cli::main::print_check_json(err, verification, readiness, readiness_diagnostics);

    // Compute resp for the following logic.
    let resp = match err {
        None => CheckResponse {
            ok: true,
            diagnostics: vec![],
            verification,
            readiness,
            readiness_diagnostics,
        },
        Some(e) => CheckResponse {
            ok: false,
            diagnostics: e.diagnostics(),
            verification: None,
            readiness: None,
            readiness_diagnostics: None,
        },
    };
    println!("{}", serde_json::to_string(&resp).unwrap());
}

fn print_run_json(result: Result<RunResult, SpandaError>) {
    // Description:
    //     Print run json.
    //
    // Inputs:
    //     resul: Result<RunResult, SpandaError>
    //         Caller-supplied resul.
    //
    // Outputs:
    //     None.
    //
    // Example:
    //     let result = spanda_cli::main::print_run_json(resul);

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
    // Description:
    //     Human check.
    //
    // Inputs:
    //     source: &str
    //         Caller-supplied source.
    //     file: &str
    //         Caller-supplied file.
    //
    // Outputs:
    //     None.
    //
    // Example:
    //     let result = spanda_cli::main::human_check(source, file);

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

fn human_run(source: &str, file: &str, command: &str, opts: RunOptions) {
    // Description:
    //     Human run.
    //
    // Inputs:
    //     source: &str
    //         Caller-supplied source.
    //     file: &str
    //         Caller-supplied file.
    //     command: &str
    //         Caller-supplied command.
    //     opts: RunOptions
    //         Caller-supplied opts.
    //
    // Outputs:
    //     None.
    //
    // Example:
    //     let result = spanda_cli::main::human_run(source, file, command, opts);

    // Compute max loop iterations for the following logic.
    let verbose = opts.max_loop_iterations > 10;
    let trace_scheduler = opts.trace_scheduler;
    let trace_tasks = opts.trace_tasks;
    let trace_triggers = opts.trace_triggers;
    let trace_events = opts.trace_events;
    let trace_providers = opts.trace_providers;

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
            if verbose
                || trace_scheduler
                || trace_tasks
                || trace_triggers
                || trace_events
                || trace_providers
            {
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
            if trace_scheduler || trace_tasks || trace_triggers || trace_events || trace_providers {
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

                // Print provider dispatch metrics when provider tracing is enabled.
                if trace_providers && !result.metrics.providers.is_empty() {
                    println!("  Providers:");

                    // Report each provider call aggregate.
                    for provider in result.metrics.providers.values() {
                        println!(
                            "    {} [{}]: calls={}, failures={}, max_duration_ms={:.2}",
                            provider.provider_key,
                            provider.category,
                            provider.calls,
                            provider.failures,
                            provider.max_duration_ms
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

fn human_verify(
    source: &str,
    file: &str,
    options: &VerifyOptions,
    system_config: Option<&spanda_config::ResolvedSystemConfig>,
) {
    // Description:
    //     Human verify.
    //
    // Inputs:
    //     source: &str
    //         Caller-supplied source.
    //     file: &str
    //         Caller-supplied file.
    //     options: &VerifyOptions
    //         Caller-supplied options.
    //
    // Outputs:
    //     None.
    //
    // Example:
    //     let result = spanda_cli::main::human_verify(source, file, options);

    let registry = system_config
        .and_then(|c| spanda_modules::load_project_modules(&c.project_root).ok())
        .or_else(|| package::module_registry_for_source(Path::new(file)));
    let program = if let Some(ref reg) = registry {
        match compile_with_registry(source, reg) {
            Ok(c) => c.program,
            Err(e) => {
                eprintln!("Error: {e}");
                process::exit(1);
            }
        }
    } else {
        match compile(source) {
            Ok(c) => c.program,
            Err(e) => {
                eprintln!("Error: {e}");
                process::exit(1);
            }
        }
    };
    let report = config_load::verify_with_system_config(&program, system_config, options.clone());
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

fn print_verify_json(result: Result<CompatibilityReport, SpandaError>) {
    // Description:
    //     Print verify json.
    //
    // Inputs:
    //     resul: Result<CompatibilityReport, SpandaError>
    //         Caller-supplied resul.
    //
    // Outputs:
    //     None.
    //
    // Example:
    //     let result = spanda_cli::main::print_verify_json(resul);

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
                .map(|d| CompatItem {
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
    // Description:
    //     Is package command.
    //
    // Inputs:
    //     cmd: &str
    //         Caller-supplied cmd.
    //
    // Outputs:
    //     result: bool
    //         Return value from `is_package_command`.
    //
    // Example:
    //     let result = spanda_cli::main::is_package_command(cmd);

    // Produce matches! as the result.
    matches!(
        cmd,
        "init"
            | "build"
            | "test"
            | "add"
            | "remove"
            | "publish"
            | "install"
            | "update"
            | "registry"
            | "verify-adapter"
    )
}

fn twin_dispatch(args: &[String]) {
    // Description:
    //     Twin dispatch.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::main::twin_dispatch(args);

    if args.first().map(String::as_str) == Some("readiness") {
        readiness_cli::cmd_twin_readiness(&args[1..]);
        return;
    }
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
    let entry_behavior = compile(&source).ok().and_then(|compiled| {
        let Program::Program { robots, .. } = compiled.program;
        robots.first().and_then(|robot| {
            let RobotDecl::RobotDecl {
                behaviors, tasks, ..
            } = robot;
            behaviors
                .first()
                .map(|behavior| {
                    let BehaviorDecl::BehaviorDecl { name, .. } = behavior;
                    name.clone()
                })
                .or_else(|| {
                    tasks.first().map(|task| {
                        let TaskDecl::TaskDecl { name, .. } = task;
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
    // Description:
    //     Fleet dispatch.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:
    //     let result = spanda_cli::main::fleet_dispatch(args);

    // take the branch when as str) differs from Some.
    if args.first().map(String::as_str) == Some("readiness") {
        readiness_cli::cmd_fleet_readiness(&args[1..]);
        return;
    }
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
        eprintln!("Usage: spanda fleet run [--json] [--trace-*] [--persist-telemetry] <file.sd>");
        eprintln!(
            "       spanda fleet orchestrate [--json] [--remote] [--mesh-url <url>] <file.sd>"
        );
        eprintln!("       spanda fleet agent start|register|list");
        eprintln!("       spanda fleet mesh start [--bind <addr>]");
        process::exit(1);
    }
    let mut json = false;
    let mut trace_scheduler = false;
    let mut trace_tasks = false;
    let mut trace_triggers = false;
    let mut trace_events = false;
    let mut persist_telemetry = false;
    let mut config_path: Option<String> = None;
    let mut file: Option<String> = None;
    let fleet_args: Vec<String> = args.iter().skip(1).cloned().collect();
    let mut i = 0usize;
    while i < fleet_args.len() {
        match fleet_args[i].as_str() {
            "--json" => json = true,
            "--trace-scheduler" => trace_scheduler = true,
            "--trace-tasks" => trace_tasks = true,
            "--trace-triggers" => trace_triggers = true,
            "--trace-events" => trace_events = true,
            "--persist-telemetry" => persist_telemetry = true,
            "--config" => {
                i += 1;
                if i >= fleet_args.len() {
                    eprintln!("--config requires a path to spanda.toml");
                    process::exit(1);
                }
                config_path = Some(fleet_args[i].clone());
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
    let source = read_source(&file);
    let config_flag = config_path.as_deref().map(Path::new);

    // Take this path when json.
    if json {
        print_fleet_json(
            &source,
            &file,
            trace_scheduler,
            trace_tasks,
            trace_triggers,
            trace_events,
            persist_telemetry,
            config_flag,
        );
    } else {
        human_fleet_run(
            &source,
            &file,
            trace_scheduler,
            trace_tasks,
            trace_triggers,
            trace_events,
            persist_telemetry,
            config_flag,
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
    persist_telemetry: bool,
    config_flag: Option<&Path>,
) {
    // Description:
    //     Human fleet run.
    //
    // Inputs:
    //     source: &str
    //         Caller-supplied source.
    //     file: &str
    //         Caller-supplied file.
    //     race_scheduler: bool
    //         Caller-supplied race scheduler.
    //     race_tasks: bool
    //         Caller-supplied race tasks.
    //     race_triggers: bool
    //         Caller-supplied race triggers.
    //     race_events: bool
    //         Caller-supplied race events.
    //
    // Outputs:
    //     None.
    //
    // Example:
    //     let result = spanda_cli::main::human_fleet_run(source, file, race_scheduler, race_tasks, race_triggers, race_events);

    // Import the items needed by the logic below.
    println!("\n🛰️  Fleet run from {file}\n");

    // Handle the success value from tokenize.
    if let Ok(tokens) = tokenize(source) {
        // Handle the success value from parse.
        if let Ok(program) = parse(tokens) {
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
                    let PeerRobotDecl::PeerRobotDecl {
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
            persist_telemetry,
            ..Default::default()
        },
        config_flag,
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
    persist_telemetry: bool,
    config_flag: Option<&Path>,
) {
    // Description:
    //     Print fleet json.
    //
    // Inputs:
    //     source: &str
    //         Caller-supplied source.
    //     _file: &str
    //         Caller-supplied file.
    //     race_scheduler: bool
    //         Caller-supplied race scheduler.
    //     race_tasks: bool
    //         Caller-supplied race tasks.
    //     race_triggers: bool
    //         Caller-supplied race triggers.
    //     race_events: bool
    //         Caller-supplied race events.
    //
    // Outputs:
    //     None.
    //
    // Example:
    //     let result = spanda_cli::main::print_fleet_json(source, _file, race_scheduler, race_tasks, race_triggers, race_events);

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
            persist_telemetry,
            ..Default::default()
        },
        config_flag,
    );
    print_run_json(run(source, opts));
}

fn security_dispatch(rest: &[String]) {
    // Description:
    //     Security dispatch.
    //
    // Inputs:
    //     res: &[String]
    //         Caller-supplied res.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::main::security_dispatch(res);

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
        // Description:
        //     From.
        //
        // Inputs:
        //     repor: &SecurityReport
        //         Caller-supplied repor.
        //
        // Outputs:
        //     result: Self
        //         Return value from `from`.
        //
        // Example:

        //     let result = spanda_cli::main::from(repor);

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
    // Description:
    //     Human security report.
    //
    // Inputs:
    //     file: &str
    //         Caller-supplied file.
    //     ode: &str
    //         Caller-supplied ode.
    //     repor: &SecurityReport
    //         Caller-supplied repor.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::main::human_security_report(file, ode, repor);

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

fn dispatch_man(args: &[String]) {
    // Description:
    //     Dispatch man.
    //
    // Inputs:
    //     args: &[String]
    //         Caller-supplied args.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_cli::main::dispatch_man(args);

    let mut roff = false;
    let mut query: Option<String> = None;
    for arg in args {
        if arg == "--roff" {
            roff = true;
        } else if !arg.starts_with('-') && query.is_none() {
            query = Some(arg.clone());
        }
    }
    if let Some(q) = query {
        match lookup_man_page(&q) {
            Some((name, body)) => {
                if roff {
                    let page = name.strip_suffix(".md").unwrap_or(&name);
                    print!("{}", markdown_man_to_roff(&body, page));
                } else {
                    print!("{body}");
                }
            }
            None => {
                eprintln!("No man page for: {q}");
                eprintln!("Available: {}", list_man_pages().join(", "));
                process::exit(1);
            }
        }
    } else {
        println!("Spanda manual pages:\n");
        for page in list_man_pages() {
            let short = page.strip_prefix("spanda-").unwrap_or(&page);
            println!("  {short}");
        }
    }
}

fn dispatch_package(command: &str, rest: &[String]) {
    // Description:
    //     Dispatch package.
    //
    // Inputs:
    //     command: &str
    //         Caller-supplied command.
    //     res: &[String]
    //         Caller-supplied res.
    //
    // Outputs:
    //     None.
    //
    // Example:
    //     let result = spanda_cli::main::dispatch_package(command, res);

    // Match on command and handle each case.
    match command {
        "init" => package::cmd_init(rest),
        "build" => package::cmd_build(rest),
        "test" => package::cmd_test(rest),
        "add" => package::cmd_add(rest),
        "remove" => package::cmd_remove(rest),
        "install" => package::cmd_install(rest),
        "update" => package::cmd_update(rest),
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
    // Description:
    //     Main.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:
    //     let result = spanda_cli::main::main();

    // Compute args for the following logic.
    let args: Vec<String> = env::args().collect();

    if args.len() >= 2 && (args[1] == "--version" || args[1] == "-V") {
        println!("spanda {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // Take the branch when len equals "--help" || args[1] == "-h".
    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        usage();
        process::exit(if args.len() < 2 { 1 } else { 0 });
    }
    let command = args[1].as_str();

    if command == "demo" {
        demo_cli::demo_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "ros2" {
        match args.get(2).map(String::as_str) {
            Some("check") => ros2_cli::ros2_dispatch(&args[3..]),
            _ => {
                eprintln!("Usage: spanda ros2 check [--json]");
                process::exit(1);
            }
        }
        let _ = io::stdout().flush();
        return;
    }

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
            "plan" | "rollout" | "rollback" | "status" | "gate" | "agent" => {
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

    if command == "config" {
        config_cli::config_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "drift" {
        drift_cli::drift_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "device-tree" {
        device_tree_cli::device_tree_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "device" {
        device_cli::device_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "network" {
        if args.get(2).map(String::as_str) == Some("scan") {
            network_cli::cmd_network_scan(&args[3..]);
        } else {
            eprintln!("Usage: spanda network scan --subnet <CIDR> [--json] [--ports 80,443,554]");
            process::exit(1);
        }
        let _ = io::stdout().flush();
        return;
    }

    if command == "map" {
        if args.get(2).map(String::as_str) == Some("verify") {
            device_tree_cli::cmd_map_verify(&args[3..]);
        } else {
            eprintln!("Usage: spanda map verify <file.sd> [--config <spanda.toml>] [--json]");
            process::exit(1);
        }
        let _ = io::stdout().flush();
        return;
    }

    if command == "contract" {
        contract_cli::contract_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "graph" {
        graph_cli::graph_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "trust" {
        package::cmd_trust(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "explain" {
        explain_cli::explain_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "safety-coverage" {
        readiness_cli::cmd_safety_coverage(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "recovery-coverage" {
        assurance_cli::cmd_recovery_coverage(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "assure" {
        assurance_cli::cmd_assure(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "anomaly" {
        assurance_cli::anomaly_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "prognostics" {
        assurance_cli::cmd_prognostics(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "mission" {
        assurance_cli::mission_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "resilience" {
        assurance_cli::resilience_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "mitigation" {
        assurance_cli::mitigation_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "state" {
        assurance_cli::state_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "heal" {
        recovery_cli::cmd_heal(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "continuity" {
        continuity_cli::cmd_continuity(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "takeover" {
        continuity_cli::cmd_takeover(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "delegate" {
        continuity_cli::cmd_delegate(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "succession" {
        continuity_cli::cmd_succession(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "recover" {
        recovery_cli::cmd_recover(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "recovery-report" || command == "recovery" {
        if command == "recovery" {
            recovery_cli::recovery_dispatch(&args[2..]);
        } else {
            recovery_cli::cmd_recovery_report(&args[2..]);
        }
        let _ = io::stdout().flush();
        return;
    }

    if command == "readiness" {
        readiness_cli::readiness_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "analyze-failure" {
        if args.iter().any(|a| a == "--with-recovery") {
            recovery_cli::cmd_analyze_failure_recovery(&args[2..]);
        } else {
            readiness_cli::cmd_analyze_failure(&args[2..]);
        }
        let _ = io::stdout().flush();
        return;
    }

    if command == "safety-report" {
        readiness_cli::cmd_safety_report(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "diagnose" {
        assurance_cli::cmd_diagnose_assurance(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "audit" {
        if args.get(2).map(String::as_str) == Some("decisions") {
            decision_cli::audit_dispatch(&args[2..]);
        } else {
            readiness_cli::cmd_audit(&args[2..]);
        }
        let _ = io::stdout().flush();
        return;
    }

    if command == "verify-approval" {
        readiness_cli::cmd_verify_approval(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "verify-fleet" {
        readiness_cli::cmd_verify_fleet(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "verify" && args.get(2).map(String::as_str) == Some("mission") {
        readiness_cli::cmd_verify_mission(&args[3..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "trace" {
        let sub = args.get(2).map(String::as_str).unwrap_or("");
        trace_cli::cmd_trace(sub, &args[3..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "health" {
        let sub = args.get(2).map(String::as_str).unwrap_or("robot");
        trace_cli::cmd_health(sub, &args[3..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "fault" {
        fault_cli::fault_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "runtime" {
        fault_cli::runtime_dispatch(&args[2..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "telemetry" {
        let sub = args.get(2).map(String::as_str).unwrap_or("");
        telemetry_cli::cmd_telemetry(sub, &args[3..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "hardware" && args.get(2).map(String::as_str) == Some("capabilities") {
        trace_cli::cmd_hardware_capabilities(&args[3..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "robot" && args.get(2).map(String::as_str) == Some("capabilities") {
        trace_cli::cmd_robot_capabilities(&args[3..]);
        let _ = io::stdout().flush();
        return;
    }

    if command == "safety" {
        let sub = args.get(2).map(String::as_str).unwrap_or("");
        if sub == "check" {
            trace_cli::cmd_safety_check(&args[3..]);
        } else {
            eprintln!("Usage: spanda safety check <file.sd> [--capabilities] [--json]");
            process::exit(1);
        }
        let _ = io::stdout().flush();
        return;
    }

    if command == "man" {
        dispatch_man(&args[2..]);
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
    let mut html = false;
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
    let mut trace_providers = false;
    let mut replay_trace = false;
    let mut trace_realtime = false;
    let mut record_trace = false;
    let mut metrics_json = false;
    let mut replay_deterministic = false;
    let mut replay_playback = false;
    let mut replay_show_faults = false;
    let mut replay_from: Option<String> = None;
    let mut trace_output: Option<String> = None;
    let mut twin_export_path: Option<String> = None;
    let mut wall_clock = false;
    let mut secure_mode = false;
    let mut inject_security_faults = false;
    let mut enforce_certify = false;
    let mut traceability = false;
    let mut traceability_json = false;
    let mut verify_capabilities = false;
    let mut verify_health = false;
    let mut minimum_capabilities = false;
    let mut verification_json = false;
    let mut readiness_json = false;
    let mut trigger_kill_switch: Option<String> = None;
    let mut inject_health_faults = false;
    let mut persist_telemetry = false;
    let mut inject_failure: Option<String> = None;
    let mut config_path: Option<String> = None;
    let mut i = 2;

    // Repeat while i < args.len().
    while i < args.len() {
        // Match on as str and handle each case.
        match args[i].as_str() {
            "--json" => json = true,
            "--html" => html = true,
            "--verification-json" => {
                verification_json = true;
                json = true;
            }
            "--readiness-json" => {
                readiness_json = true;
                json = true;
            }
            "--verbose" | "-v" => verbose = true,
            "--trace-scheduler" => trace_scheduler = true,
            "--trace-tasks" => trace_tasks = true,
            "--trace-triggers" => trace_triggers = true,
            "--trace-events" => trace_events = true,
            "--trace-providers" => trace_providers = true,
            "--replay" => replay_trace = true,
            "--trace-realtime" => trace_realtime = true,
            "--metrics-json" => {
                metrics_json = true;
                json = true;
            }
            "--record" => record_trace = true,
            "--persist-telemetry" => persist_telemetry = true,
            "--secure" => secure_mode = true,
            "--inject-security-faults" => inject_security_faults = true,
            "--enforce-certify" => enforce_certify = true,
            "--traceability" => traceability = true,
            "--traceability-json" => {
                traceability = true;
                traceability_json = true;
                json = true;
            }
            "--capabilities" => verify_capabilities = true,
            "--capabilities-json" => {
                verify_capabilities = true;
                json = true;
            }
            "--health" => verify_health = true,
            "--minimum-capabilities" => minimum_capabilities = true,
            "--trigger-kill-switch" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--trigger-kill-switch requires a kill switch name");
                    process::exit(1);
                }
                trigger_kill_switch = Some(args[i].clone());
            }
            "--inject-health-faults" => inject_health_faults = true,
            "--config" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--config requires a path to spanda.toml");
                    process::exit(1);
                }
                config_path = Some(args[i].clone());
            }
            "--inject-failure" => {
                i += 1;
                inject_failure = args.get(i).cloned();
            }
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
            "--show-faults" => replay_show_faults = true,
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
        replay_cli::human_replay(
            &trace_file,
            replay_from.as_deref(),
            replay_deterministic,
            replay_playback,
            replay_show_faults,
            json,
            config_path.as_deref().map(Path::new),
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
                    let check_result = check(&source);
                    let program = trace_cli::parse_program(&source);
                    let verification = if verification_json && check_result.is_ok() {
                        Some(spanda_capability::collect_verification_diagnostics(
                            &program,
                        ))
                    } else {
                        None
                    };
                    let readiness = if readiness_json && check_result.is_ok() {
                        let options = spanda_readiness::readiness_options_from_flags(
                            &program,
                            target.clone(),
                            inject_health_faults,
                            inject_health_faults,
                            simulate,
                            strict_certify,
                        );
                        Some(spanda_readiness::evaluate_readiness_with_runtime(
                            &program,
                            &options,
                            inject_health_faults
                                .then(|| spanda_readiness::build_runtime_context(&program, true))
                                .as_ref(),
                        ))
                    } else {
                        None
                    };
                    let readiness_diagnostics = if readiness_json && check_result.is_ok() {
                        let options = spanda_readiness::readiness_options_from_flags(
                            &program,
                            target.clone(),
                            inject_health_faults,
                            inject_health_faults,
                            simulate,
                            strict_certify,
                        );
                        let mut diags =
                            spanda_readiness::collect_readiness_diagnostics(&program, &options);
                        diags.extend(spanda_assurance::collect_recovery_diagnostics(&program));
                        diags.extend(spanda_assurance::collect_continuity_diagnostics(&program));
                        Some(diags)
                    } else {
                        None
                    };
                    print_check_json(
                        check_result.err(),
                        verification,
                        readiness,
                        readiness_diagnostics,
                    );
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
            let config_flag = config_path.as_deref().map(Path::new);
            let system_config = config_load::load_system_config(Path::new(&file), config_flag);
            config_load::ensure_config_valid(system_config.as_ref().map(|a| a.as_ref()));
            let mut options = VerifyOptions {
                target: target.clone().or_else(|| {
                    system_config
                        .as_ref()
                        .and_then(|c| spanda_config::default_verify_target(c))
                }),
                all_targets,
                simulate,
                strict_certify,
            };
            if options.target.is_some() {
                options.all_targets = false;
            }
            let registry = system_config
                .as_ref()
                .and_then(|c| spanda_modules::load_project_modules(&c.project_root).ok())
                .or_else(|| package::module_registry_for_source(Path::new(&file)));

            // Take this path when json.
            if json {
                let result = (|| {
                    let program = if let Some(ref reg) = registry {
                        compile_with_registry(&source, reg)?.program
                    } else {
                        compile(&source)?.program
                    };
                    Ok(config_load::verify_with_system_config(
                        &program,
                        system_config.as_deref(),
                        options.clone(),
                    ))
                })();
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
                human_verify(&source, &file, &options, system_config.as_deref());
            }
            if traceability || verify_capabilities || verify_health || minimum_capabilities {
                trace_cli::verify_extensions(
                    &source,
                    traceability,
                    verify_capabilities,
                    verify_health,
                    minimum_capabilities,
                    json || traceability_json,
                );
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
            let config_flag = config_path.as_deref().map(Path::new);
            let opts = run_options_for_file(
                &file,
                RunOptions {
                    max_loop_iterations,
                    trace_scheduler: trace_scheduler || trace_realtime,
                    trace_tasks: trace_tasks || trace_realtime,
                    trace_triggers: trace_triggers || trace_realtime,
                    trace_events: trace_events || trace_realtime,
                    trace_providers: trace_providers || trace_realtime,
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
                    trigger_kill_switch,
                    inject_health_faults,
                    persist_telemetry,
                    ..Default::default()
                },
                config_flag,
            );

            // Take this path when json.
            if json || metrics_json {
                print_run_json(run(&source, opts));
            } else {
                human_run(&source, &file, command, opts);
            }
            if command == "sim" {
                if let Some(failure_kind) = inject_failure.as_deref() {
                    let program = parse(tokenize(&source).unwrap()).unwrap();
                    let recovery =
                        spanda_assurance::simulate_failure_recovery(&program, failure_kind);
                    eprintln!("\n--- Recovery Simulation ---");
                    eprintln!(
                        "{}",
                        spanda_assurance::format_recovery(
                            &recovery,
                            spanda_readiness::ReportFormat::Text
                        )
                    );
                }
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
                                LintSeverity::Warning => "warning",
                                LintSeverity::Error => "error",
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
                                .map(|d| LintIssue {
                                    rule: "parse".into(),
                                    message: d.message,
                                    line: d.line,
                                    column: d.column,
                                    severity: LintSeverity::Error,
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
                eprintln!("Missing file or directory path");
                usage();
                process::exit(1);
            });
            let path = Path::new(&file);
            if path.is_dir() || out_path.is_some() {
                let out = out_path.as_deref().map(Path::new);
                match generate_docs_for_path(path, html, out) {
                    Ok(batch) => {
                        for (failed, err) in &batch.errors {
                            eprintln!("Warning: {}: {err}", failed.display());
                        }
                        if batch.outputs.is_empty() && !batch.errors.is_empty() {
                            process::exit(1);
                        }
                        if json {
                            let resp = serde_json::json!({
                                "ok": batch.errors.is_empty(),
                                "count": batch.outputs.len(),
                                "errors": batch.errors.iter().map(|(p, e)| {
                                    serde_json::json!({"file": p.display().to_string(), "error": e})
                                }).collect::<Vec<_>>(),
                                "files": batch.outputs.iter().map(|(p, _)| p.display().to_string()).collect::<Vec<_>>(),
                            });
                            println!("{}", serde_json::to_string(&resp).unwrap());
                        } else if let Some(out) = out {
                            println!(
                                "✓ wrote {} doc file(s) to {} ({} skipped)",
                                batch.outputs.len(),
                                out.display(),
                                batch.errors.len()
                            );
                        } else if batch.outputs.len() == 1 {
                            print!("{}", batch.outputs[0].1);
                        } else {
                            for (p, _) in &batch.outputs {
                                println!("{}", p.display());
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        process::exit(1);
                    }
                }
            } else {
                let source = read_source(&file);
                if json {
                    match generate_json_docs(&source, html) {
                        Ok(resp) => println!("{}", serde_json::to_string(&resp).unwrap()),
                        Err(e) => {
                            eprintln!("Error: {e}");
                            process::exit(1);
                        }
                    }
                } else if html {
                    match generate_html(&source, None) {
                        Ok(content) => {
                            if let Some(ref out) = out_path {
                                fs::write(out, &content).unwrap_or_else(|e| {
                                    eprintln!("Error writing {out}: {e}");
                                    process::exit(1);
                                });
                                println!("✓ wrote HTML docs to {out}");
                            } else {
                                print!("{content}");
                            }
                        }
                        Err(e) => {
                            eprintln!("Error: {e}");
                            process::exit(1);
                        }
                    }
                } else {
                    match generate_markdown(&source) {
                        Ok(markdown) => {
                            if let Some(ref out) = out_path {
                                fs::write(out, &markdown).unwrap_or_else(|e| {
                                    eprintln!("Error writing {out}: {e}");
                                    process::exit(1);
                                });
                                println!("✓ wrote docs to {out}");
                            } else {
                                print!("{markdown}");
                            }
                        }
                        Err(e) => {
                            eprintln!("Error: {e}");
                            process::exit(1);
                        }
                    }
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
            let source = read_source(&file);

            if codegen_target == CodegenTarget::Native {
                #[cfg(feature = "llvm")]
                {
                    match lower_to_sir(&source) {
                        Ok(sir) => {
                            let workspace = std::env::current_dir().unwrap_or_else(|_| ".".into());
                            let output =
                                out_path.map(std::path::PathBuf::from).unwrap_or_else(|| {
                                    workspace.join("target/spanda-native/spanda-program")
                                });
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
                                    println!(
                                        "✓ wrote LLVM IR to {}",
                                        result.llvm_ir_path.display()
                                    );
                                    println!(
                                        "✓ linked native binary to {}",
                                        result.executable.display()
                                    );
                                }
                                Err(e) => {
                                    eprintln!("Native deploy failed: {e}");
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
                #[cfg(not(feature = "llvm"))]
                {
                    llvm_unavailable();
                }
            } else if codegen_target != CodegenTarget::Wasm {
                eprintln!("deploy supports --target wasm or --target native");
                process::exit(1);
            } else {
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
            #[cfg(not(feature = "llvm"))]
            llvm_unavailable();
            #[cfg(feature = "llvm")]
            {
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
        }
        "compile-native" => {
            #[cfg(not(feature = "llvm"))]
            llvm_unavailable();
            #[cfg(feature = "llvm")]
            {
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
                        let output = out_path.map(std::path::PathBuf::from).unwrap_or_else(|| {
                            workspace.join("target/spanda-native/spanda-program")
                        });

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
                                println!(
                                    "✓ linked native binary to {}",
                                    result.executable.display()
                                );
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
