//! Fault detection logic — static scan and runtime signal evaluation.

use crate::types::{
    FaultEvidence, ResourcePressure, RuntimeFault, RuntimeFaultKind, RuntimeHealthStatus,
};
use crate::FaultScanOptions;
use spanda_ast::fault_decl::{
    HeartbeatDecl, MemoryWatchDecl, ResourceWatchDecl, RestartPolicyDecl, RuntimeFaultTriggerDecl,
};
use spanda_ast::foundations::WatchdogDecl;
use spanda_ast::nodes::{Program, RobotDecl};

/// Count configured monitors across all robots in a program.
pub fn collect_configured_monitors(program: &Program) -> (u32, u32, u32, u32) {
    // Count heartbeat, memory watch, resource watch, and restart policy declarations.
    //
    // Parameters:
    // - `program` — parsed program AST
    //
    // Returns:
    // Tuple of (heartbeats, memory_watches, resource_watches, restart_policies) counts.
    //
    // Options:
    // None.
    //
    // Example:
    // let (hb, mw, rw, rp) = collect_configured_monitors(&program);

    let Program::Program { robots, .. } = program;
    let mut hb = 0u32;
    let mut mw = 0u32;
    let mut rw = 0u32;
    let mut rp = 0u32;
    for robot in robots {
        let RobotDecl::RobotDecl {
            heartbeats,
            memory_watches,
            resource_watches,
            restart_policies,
            ..
        } = robot;
        hb += heartbeats.len() as u32;
        mw += memory_watches.len() as u32;
        rw += resource_watches.len() as u32;
        rp += restart_policies.len() as u32;
    }
    (hb, mw, rw, rp)
}

/// Static fault scan from program declarations (watchdogs, triggers, policies).
pub fn static_fault_scan(program: &Program) -> Vec<RuntimeFault> {
    // Scan program declarations for statically detectable fault risks.
    //
    // Parameters:
    // - `program` — parsed program AST
    //
    // Returns:
    // List of faults detectable from declarations alone.
    //
    // Options:
    // None.
    //
    // Example:
    // let faults = static_fault_scan(&program);

    let Program::Program {
        robots,
        runtime_fault_triggers,
        ..
    } = program;
    let mut faults = Vec::new();

    for robot in robots {
        let RobotDecl::RobotDecl {
            watchdogs,
            restart_policies,
            heartbeats,
            ..
        } = robot;

        for wd in watchdogs {
            let WatchdogDecl::WatchdogDecl { name, target, .. } = wd;
            if target.is_none() {
                faults.push(RuntimeFault {
                    kind: RuntimeFaultKind::WatchdogTimeout,
                    target: name.clone(),
                    status: RuntimeHealthStatus::Warning,
                    message: format!("Watchdog '{name}' has no target — coverage gap"),
                    evidence: FaultEvidence {
                        metric: Some("watchdog_coverage".into()),
                        value: None,
                        threshold: None,
                        boot_id: None,
                        exit_code: None,
                        stack_trace: None,
                        related_events: vec!["watchdog_untargeted".into()],
                    },
                    detected_at_ms: 0.0,
                });
            }
        }

        for hb in heartbeats {
            let HeartbeatDecl::HeartbeatDecl {
                target,
                timeout_ms,
                interval_ms,
                ..
            } = hb;
            if *timeout_ms < *interval_ms * 2.0 {
                faults.push(RuntimeFault {
                    kind: RuntimeFaultKind::HeartbeatLoss,
                    target: target.clone(),
                    status: RuntimeHealthStatus::Warning,
                    message: format!(
                        "Heartbeat '{target}' timeout ({timeout_ms}ms) may be too tight for interval ({interval_ms}ms)"
                    ),
                    evidence: FaultEvidence {
                        metric: Some("heartbeat_timeout".into()),
                        value: Some(format!("{timeout_ms}")),
                        threshold: Some(format!("{}", interval_ms * 2.0)),
                        boot_id: None,
                        exit_code: None,
                        stack_trace: None,
                        related_events: vec!["heartbeat_config_warning".into()],
                    },
                    detected_at_ms: 0.0,
                });
            }
        }

        for rp in restart_policies {
            let RestartPolicyDecl::RestartPolicyDecl {
                target,
                max_restarts,
                ..
            } = rp;
            if *max_restarts == 0 {
                faults.push(RuntimeFault {
                    kind: RuntimeFaultKind::RestartLoop,
                    target: target.clone(),
                    status: RuntimeHealthStatus::Warning,
                    message: format!("Restart policy for '{target}' allows zero restarts"),
                    evidence: FaultEvidence {
                        metric: Some("max_restarts".into()),
                        value: Some("0".into()),
                        threshold: None,
                        boot_id: None,
                        exit_code: None,
                        stack_trace: None,
                        related_events: vec!["restart_policy_zero".into()],
                    },
                    detected_at_ms: 0.0,
                });
            }
        }
    }

    for trigger in runtime_fault_triggers {
        let RuntimeFaultTriggerDecl::RuntimeFaultTriggerDecl { event, body, .. } = trigger;
        if body.is_empty() {
            faults.push(RuntimeFault {
                kind: map_trigger_event_to_kind(event),
                target: event.clone(),
                status: RuntimeHealthStatus::Warning,
                message: format!("Runtime fault trigger 'on {event}' has no recovery actions"),
                evidence: FaultEvidence {
                    metric: None,
                    value: None,
                    threshold: None,
                    boot_id: None,
                    exit_code: None,
                    stack_trace: None,
                    related_events: vec!["empty_fault_trigger".into()],
                },
                detected_at_ms: 0.0,
            });
        }
    }

    faults
}

/// Evaluate faults from injected or live runtime signals.
pub fn detect_from_runtime_signals(
    program: &Program,
    options: &FaultScanOptions,
) -> Vec<RuntimeFault> {
    // Detect faults from injected simulation signals or runtime context.
    //
    // Parameters:
    // - `program` — parsed program AST
    // - `options` — scan options with injection flags
    //
    // Returns:
    // List of runtime-detected faults.
    //
    // Options:
    // Set injection flags to simulate fault conditions.
    //
    // Example:
    // let faults = detect_from_runtime_signals(&program, &options);

    let Program::Program { robots, .. } = program;
    let mut faults = Vec::new();
    let robot_name = robots.first().map(|r| {
        let RobotDecl::RobotDecl { name, .. } = r;
        name.clone()
    });

    if options.inject_crash {
        faults.push(make_fault(
            RuntimeFaultKind::ProcessCrash,
            robot_name.clone().unwrap_or_else(|| "runtime".into()),
            RuntimeHealthStatus::Crashed,
            "Process crash detected (injected)",
            options.sim_time_ms,
            Some(-1),
        ));
    }

    if options.inject_memory_leak {
        faults.push(make_fault(
            RuntimeFaultKind::MemoryLeak,
            robot_name.clone().unwrap_or_else(|| "runtime".into()),
            RuntimeHealthStatus::Warning,
            "Memory growth exceeds threshold (injected)",
            options.sim_time_ms,
            None,
        ));
    }

    if options.inject_reboot {
        faults.push(make_fault(
            RuntimeFaultKind::UnexpectedReboot,
            robot_name.clone().unwrap_or_else(|| "system".into()),
            RuntimeHealthStatus::Rebooted,
            "Unexpected reboot detected (injected)",
            options.sim_time_ms,
            None,
        ));
    }

    if options.inject_heartbeat_loss {
        for robot in robots {
            let RobotDecl::RobotDecl { heartbeats, .. } = robot;
            for hb in heartbeats {
                let HeartbeatDecl::HeartbeatDecl { target, .. } = hb;
                faults.push(make_fault(
                    RuntimeFaultKind::HeartbeatLoss,
                    target.clone(),
                    RuntimeHealthStatus::Degraded,
                    &format!("Heartbeat missed for '{target}'"),
                    options.sim_time_ms,
                    None,
                ));
            }
        }
    }

    faults
}

/// Evaluate resource pressure from configured watches and injection flags.
pub fn evaluate_resource_pressure(
    program: &Program,
    options: &FaultScanOptions,
) -> Vec<RuntimeFault> {
    // Check resource watch conditions against current or injected pressure levels.
    //
    // Parameters:
    // - `program` — parsed program AST
    // - `options` — scan options with injection flags
    //
    // Returns:
    // List of resource pressure faults.
    //
    // Options:
    // Set `inject_resource_pressure` to simulate pressure conditions.
    //
    // Example:
    // let faults = evaluate_resource_pressure(&program, &options);

    let Program::Program { robots, .. } = program;
    let mut faults = Vec::new();

    for robot in robots {
        let RobotDecl::RobotDecl {
            resource_watches, ..
        } = robot;
        for rw in resource_watches {
            let ResourceWatchDecl::ResourceWatchDecl { conditions, .. } = rw;
            for cond in conditions {
                let pressured = options.inject_resource_pressure
                    || exceeds_threshold(&cond.resource, &cond.operator, &cond.threshold);
                if pressured {
                    let kind = resource_to_fault_kind(&cond.resource);
                    faults.push(RuntimeFault {
                        kind,
                        target: cond.resource.clone(),
                        status: RuntimeHealthStatus::Critical,
                        message: format!(
                            "Resource pressure: {} {} {}",
                            cond.resource, cond.operator, cond.threshold
                        ),
                        evidence: FaultEvidence {
                            metric: Some(cond.resource.clone()),
                            value: Some(cond.threshold.clone()),
                            threshold: Some(cond.threshold.clone()),
                            boot_id: None,
                            exit_code: None,
                            stack_trace: None,
                            related_events: vec!["resource_pressure".into()],
                        },
                        detected_at_ms: options.sim_time_ms,
                    });
                }
            }
        }
    }

    faults
}

/// Evaluate restart loop conditions from policies and injection.
pub fn evaluate_restart_loops(program: &Program, options: &FaultScanOptions) -> Vec<RuntimeFault> {
    // Detect restart loops from policy configuration and runtime restart counts.
    //
    // Parameters:
    // - `program` — parsed program AST
    // - `options` — scan options; inject_crash simulates restart loop
    //
    // Returns:
    // List of restart loop faults.
    //
    // Options:
    // None.
    //
    // Example:
    // let faults = evaluate_restart_loops(&program, &options);

    let Program::Program { robots, .. } = program;
    let mut faults = Vec::new();

    for robot in robots {
        let RobotDecl::RobotDecl {
            restart_policies, ..
        } = robot;
        for rp in restart_policies {
            let RestartPolicyDecl::RestartPolicyDecl {
                target,
                max_restarts,
                window,
                ..
            } = rp;
            if options.inject_crash && *max_restarts > 0 {
                faults.push(RuntimeFault {
                    kind: RuntimeFaultKind::RestartLoop,
                    target: target.clone(),
                    status: RuntimeHealthStatus::Critical,
                    message: format!(
                        "Restart loop detected for '{target}': exceeded {max_restarts} restarts within {window}"
                    ),
                    evidence: FaultEvidence {
                        metric: Some("restart_count".into()),
                        value: Some(max_restarts.to_string()),
                        threshold: Some(window.clone()),
                        boot_id: None,
                        exit_code: None,
                        stack_trace: None,
                        related_events: vec!["restart_loop_exceeded".into()],
                    },
                    detected_at_ms: options.sim_time_ms,
                });
            }
        }
    }

    faults
}

/// Parse memory watch threshold from declaration for leak detection.
pub fn parse_memory_watch_threshold(growth_threshold: &str, growth_window: &str) -> (f64, f64) {
    // Extract numeric MB threshold and window duration from declaration strings.
    //
    // Parameters:
    // - `growth_threshold` — e.g. "100 MB"
    // - `growth_window` — e.g. "10 min"
    //
    // Returns:
    // Tuple of (threshold_mb, window_ms).
    //
    // Options:
    // None.
    //
    // Example:
    // let (mb, ms) = parse_memory_watch_threshold("100 MB", "10 min");

    let threshold_mb = growth_threshold
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(100.0);
    let window_ms = parse_duration_to_ms(growth_window);
    (threshold_mb, window_ms)
}

/// Evaluate memory watches for leak conditions.
pub fn evaluate_memory_watches(
    program: &Program,
    current_mb: f64,
    baseline_mb: f64,
) -> Vec<RuntimeFault> {
    // Check memory watch declarations against observed memory growth.
    //
    // Parameters:
    // - `program` — parsed program AST
    // - `current_mb` — current memory usage in MB
    // - `baseline_mb` — baseline memory at start of window
    //
    // Returns:
    // Memory leak faults if growth exceeds configured thresholds.
    //
    // Options:
    // None.
    //
    // Example:
    // let faults = evaluate_memory_watches(&program, 250.0, 100.0);

    let Program::Program { robots, .. } = program;
    let mut faults = Vec::new();
    let growth = current_mb - baseline_mb;

    for robot in robots {
        let RobotDecl::RobotDecl { memory_watches, .. } = robot;
        for mw in memory_watches {
            let MemoryWatchDecl::MemoryWatchDecl {
                target,
                growth_threshold,
                growth_window,
                ..
            } = mw;
            let (threshold_mb, _) = parse_memory_watch_threshold(growth_threshold, growth_window);
            if growth > threshold_mb {
                faults.push(RuntimeFault {
                    kind: RuntimeFaultKind::MemoryLeak,
                    target: target.clone(),
                    status: RuntimeHealthStatus::Warning,
                    message: format!(
                        "Memory leak on '{target}': growth {growth:.1} MB exceeds {threshold_mb} MB threshold"
                    ),
                    evidence: FaultEvidence {
                        metric: Some("memory_growth_mb".into()),
                        value: Some(format!("{growth:.1}")),
                        threshold: Some(format!("{threshold_mb}")),
                        boot_id: None,
                        exit_code: None,
                        stack_trace: None,
                        related_events: vec!["memory_leak_detected".into()],
                    },
                    detected_at_ms: 0.0,
                });
            }
        }
    }

    faults
}

/// Build resource pressure snapshot from a condition string.
pub fn resource_pressure_from_condition(
    resource: &str,
    operator: &str,
    threshold: &str,
    value: f64,
) -> ResourcePressure {
    // Build a resource pressure record from a watch condition and observed value.
    //
    // Parameters:
    // - `resource` — resource name (cpu, memory, disk, etc.)
    // - `operator` — comparison operator
    // - `threshold` — threshold string
    // - `value` — observed value
    //
    // Returns:
    // Resource pressure snapshot with status.
    //
    // Options:
    // None.
    //
    // Example:
    // let pressure = resource_pressure_from_condition("cpu", ">", "90%", 95.0);

    let threshold_val = threshold
        .trim_end_matches('%')
        .parse::<f64>()
        .unwrap_or(0.0);
    let status = if exceeds_threshold(resource, operator, threshold) {
        RuntimeHealthStatus::Critical
    } else if value > threshold_val * 0.85 {
        RuntimeHealthStatus::Warning
    } else {
        RuntimeHealthStatus::Healthy
    };
    ResourcePressure {
        resource: resource.into(),
        value,
        threshold: threshold_val,
        unit: if threshold.contains('%') {
            "%".into()
        } else {
            "absolute".into()
        },
        duration_ms: None,
        status,
    }
}

/// Map live hardware monitor fault and event strings to runtime faults.
pub fn faults_from_hardware_signals(
    faults: &[String],
    events: &[String],
    sim_time_ms: f64,
) -> Vec<RuntimeFault> {
    // Convert hardware monitor fault/event names into structured runtime faults.
    //
    // Parameters:
    // - `faults` — active fault labels from the hardware monitor
    // - `events` — active event labels from the hardware monitor
    // - `sim_time_ms` — current simulation time in milliseconds
    //
    // Returns:
    // Structured runtime fault records for each signal.
    //
    // Options:
    // None.
    //
    // Example:
    // let faults = faults_from_hardware_signals(&hw_faults, &hw_events, sim_ms);

    let mut out = Vec::new();
    for fault in faults {
        let (kind, status) = hardware_label_to_fault(fault);
        out.push(RuntimeFault {
            kind,
            target: fault.clone(),
            status,
            message: format!("Hardware fault: {fault}"),
            evidence: FaultEvidence {
                related_events: vec![fault.clone()],
                ..Default::default()
            },
            detected_at_ms: sim_time_ms,
        });
    }
    for event in events {
        if faults.contains(event) {
            continue;
        }
        let (kind, status) = hardware_event_to_fault(event);
        out.push(RuntimeFault {
            kind,
            target: event.clone(),
            status,
            message: format!("Hardware event: {event}"),
            evidence: FaultEvidence {
                related_events: vec![event.clone()],
                ..Default::default()
            },
            detected_at_ms: sim_time_ms,
        });
    }
    out
}

fn hardware_label_to_fault(label: &str) -> (RuntimeFaultKind, RuntimeHealthStatus) {
    let lower = label.to_ascii_lowercase();
    if lower.contains("crash") || lower.contains("critical") {
        (RuntimeFaultKind::ProcessCrash, RuntimeHealthStatus::Crashed)
    } else if lower.contains("offline") || lower.contains("camera") {
        (
            RuntimeFaultKind::SensorDriverCrash,
            RuntimeHealthStatus::Degraded,
        )
    } else if lower.contains("gps") || lower.contains("degraded") {
        (
            RuntimeFaultKind::TaskStarvation,
            RuntimeHealthStatus::Degraded,
        )
    } else if lower.contains("heartbeat") {
        (
            RuntimeFaultKind::HeartbeatLoss,
            RuntimeHealthStatus::Degraded,
        )
    } else if lower.contains("memory") {
        (
            RuntimeFaultKind::MemoryPressure,
            RuntimeHealthStatus::Warning,
        )
    } else if lower.contains("watchdog") {
        (
            RuntimeFaultKind::WatchdogTimeout,
            RuntimeHealthStatus::Critical,
        )
    } else {
        (
            RuntimeFaultKind::AbnormalShutdown,
            RuntimeHealthStatus::Warning,
        )
    }
}

fn hardware_event_to_fault(event: &str) -> (RuntimeFaultKind, RuntimeHealthStatus) {
    hardware_label_to_fault(event)
}

fn make_fault(
    kind: RuntimeFaultKind,
    target: String,
    status: RuntimeHealthStatus,
    message: &str,
    time_ms: f64,
    exit_code: Option<i32>,
) -> RuntimeFault {
    RuntimeFault {
        kind,
        target,
        status,
        message: message.into(),
        evidence: FaultEvidence {
            metric: None,
            value: None,
            threshold: None,
            boot_id: None,
            exit_code,
            stack_trace: None,
            related_events: Vec::new(),
        },
        detected_at_ms: time_ms,
    }
}

fn map_trigger_event_to_kind(event: &str) -> RuntimeFaultKind {
    match event {
        "runtime crash" | "crash" => RuntimeFaultKind::ProcessCrash,
        "memory_leak" | "memory leak" => RuntimeFaultKind::MemoryLeak,
        "reboot unexpected" | "unexpected reboot" => RuntimeFaultKind::UnexpectedReboot,
        "restart_loop" | "restart loop" => RuntimeFaultKind::RestartLoop,
        "watchdog timeout" => RuntimeFaultKind::WatchdogTimeout,
        "out_of_memory" | "oom" => RuntimeFaultKind::OutOfMemory,
        "deadlock" => RuntimeFaultKind::Deadlock,
        "heartbeat loss" | "heartbeat_loss" => RuntimeFaultKind::HeartbeatLoss,
        _ => RuntimeFaultKind::AbnormalShutdown,
    }
}

fn resource_to_fault_kind(resource: &str) -> RuntimeFaultKind {
    match resource.to_lowercase().as_str() {
        "cpu" => RuntimeFaultKind::CpuOverload,
        "memory" => RuntimeFaultKind::MemoryPressure,
        "disk" | "disk_free" => RuntimeFaultKind::DiskPressure,
        "network" => RuntimeFaultKind::NetworkStackCrash,
        "gpu" => RuntimeFaultKind::CpuOverload,
        "battery" => RuntimeFaultKind::CpuOverload,
        _ => RuntimeFaultKind::MemoryPressure,
    }
}

fn exceeds_threshold(resource: &str, operator: &str, threshold: &str) -> bool {
    let _ = resource;
    let val: f64 = threshold.trim_end_matches('%').parse().unwrap_or(0.0);
    let simulated = match resource.to_lowercase().as_str() {
        "memory" => 90.0,
        "cpu" => 95.0,
        "disk_free" => 100.0,
        _ => 50.0,
    };
    match operator {
        ">" | ">=" => simulated > val,
        "<" | "<=" => simulated < val,
        _ => false,
    }
}

fn parse_duration_to_ms(s: &str) -> f64 {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.is_empty() {
        return 0.0;
    }
    let num: f64 = parts[0].parse().unwrap_or(0.0);
    let unit = parts.get(1).copied().unwrap_or("ms");
    match unit {
        "s" | "sec" => num * 1000.0,
        "min" => num * 60_000.0,
        "h" | "hr" | "hour" => num * 3_600_000.0,
        "ms" => num,
        _ => num,
    }
}
