//! Real-time reliability validation helpers for tasks, pipelines, and watchdogs.
//!
use crate::ast::Span;
use crate::error::Diagnostic;
use crate::foundations::{
    PipelineDecl, RecoverDecl, ResourceBudgetDecl, TaskDecl, TaskPriority, WatchdogDecl,
};

pub fn validate_task_timing(task: &TaskDecl) -> Vec<Diagnostic> {
    // Validate periodic task period, deadline, and jitter constraints.
    //
    // Parameters:
    // - `task` — task declaration to inspect
    //
    // Returns:
    // Diagnostics for invalid timing configuration.
    //
    // Options:
    // None.
    //
    // Example:
    // let diags = validate_task_timing(&task);

    // Unpack task fields for timing validation.
    let TaskDecl::TaskDecl {
        name,
        interval_ms,
        deadline_ms,
        jitter_ms_max,
        span,
        ..
    } = task;
    let mut diags = Vec::new();

    // Reject non-positive task periods.
    if *interval_ms <= 0.0 {
        diags.push(Diagnostic {
            message: format!(
                "Task '{name}' period must be positive (got {interval_ms}ms). Suggestion: use `every 10ms` or larger."
            ),
            line: span.start.line,
            column: span.start.column,
        });
    }

    // Enforce deadline <= period when a deadline is declared.
    if let Some(deadline) = deadline_ms {
        if *deadline <= 0.0 {
            diags.push(Diagnostic {
                message: format!("Task '{name}' deadline must be positive (got {deadline}ms)."),
                line: span.start.line,
                column: span.start.column,
            });
        } else if *deadline > *interval_ms {
            diags.push(Diagnostic {
                message: format!(
                    "Task '{name}' deadline ({deadline}ms) must be <= period ({interval_ms}ms). Suggestion: increase period or reduce deadline."
                ),
                line: span.start.line,
                column: span.start.column,
            });
        }
    }

    // Validate jitter against period and deadline slack.
    if let Some(jitter) = jitter_ms_max {
        if *jitter < 0.0 {
            diags.push(Diagnostic {
                message: format!("Task '{name}' jitter must be non-negative."),
                line: span.start.line,
                column: span.start.column,
            });
        }
        let slack = deadline_ms.unwrap_or(*interval_ms);
        if *jitter > slack {
            diags.push(Diagnostic {
                message: format!(
                    "Task '{name}' jitter ({jitter}ms) exceeds allowable slack ({slack}ms). Suggestion: reduce jitter or increase deadline/period."
                ),
                line: span.start.line,
                column: span.start.column,
            });
        }
    }
    diags
}

pub fn validate_task_priority(task: &TaskDecl) -> Vec<Diagnostic> {
    // Validate priority/isolation combinations for safety-critical tasks.
    //
    // Parameters:
    // - `task` — task declaration to inspect
    //
    // Returns:
    // Diagnostics for invalid priority configuration.
    //
    // Options:
    // None.
    //
    // Example:
    // let diags = validate_task_priority(&task);

    // Unpack task fields for priority validation.
    let TaskDecl::TaskDecl {
        name,
        priority,
        isolated,
        span,
        ..
    } = task;
    let mut diags = Vec::new();

    // Isolated tasks must be critical or high priority to guarantee isolation semantics.
    if *isolated && !matches!(priority, TaskPriority::Critical | TaskPriority::High) {
        diags.push(Diagnostic {
            message: format!(
                "Task '{name}' is marked isolated but priority is {priority:?}. Suggestion: use `critical isolated` or `high isolated`."
            ),
            line: span.start.line,
            column: span.start.column,
        });
    }
    diags
}

pub fn validate_pipeline(pipeline: &PipelineDecl) -> Vec<Diagnostic> {
    // Validate pipeline latency budget configuration.
    //
    // Parameters:
    // - `pipeline` — pipeline declaration
    //
    // Returns:
    // Diagnostics for invalid pipeline budgets.
    //
    // Options:
    // None.
    //
    // Example:
    // let diags = validate_pipeline(&pipeline);

    // Unpack pipeline fields for budget validation.
    let PipelineDecl::PipelineDecl {
        name,
        budget_ms,
        span,
        ..
    } = pipeline;
    let mut diags = Vec::new();

    // Reject non-positive pipeline budgets.
    if *budget_ms <= 0.0 {
        diags.push(Diagnostic {
            message: format!("Pipeline '{name}' budget must be positive (got {budget_ms}ms)."),
            line: span.start.line,
            column: span.start.column,
        });
    }
    diags
}

pub fn validate_watchdog(watchdog: &WatchdogDecl, task_names: &[String]) -> Vec<Diagnostic> {
    // Validate watchdog timeout and monitored task target.
    //
    // Parameters:
    // - `watchdog` — watchdog declaration
    // - `task_names` — known task names in the robot block
    //
    // Returns:
    // Diagnostics for invalid watchdog configuration.
    //
    // Options:
    // None.
    //
    // Example:
    // let diags = validate_watchdog(&watchdog, &task_names);

    // Unpack watchdog fields for validation.
    let WatchdogDecl::WatchdogDecl {
        name,
        target,
        timeout_ms,
        span,
        ..
    } = watchdog;
    let mut diags = Vec::new();

    // Reject non-positive watchdog timeouts.
    if *timeout_ms <= 0.0 {
        diags.push(Diagnostic {
            message: format!("Watchdog '{name}' timeout must be positive."),
            line: span.start.line,
            column: span.start.column,
        });
    }

    // Ensure the monitored task exists when a target is declared.
    if let Some(task) = target {
        if !task_names.iter().any(|n| n == task) {
            diags.push(Diagnostic {
                message: format!(
                    "Watchdog '{name}' target task '{task}' not found. Suggestion: declare the task before the watchdog or fix the task name."
                ),
                line: span.start.line,
                column: span.start.column,
            });
        }
    }
    diags
}

pub fn validate_resource_budget(budget: &ResourceBudgetDecl, span: Span) -> Vec<Diagnostic> {
    // Validate per-task resource budget ceilings for conflicts and invalid values.
    //
    // Parameters:
    // - `budget` — resource budget block
    // - `span` — enclosing task span for diagnostics
    //
    // Returns:
    // Diagnostics for invalid or conflicting budgets.
    //
    // Options:
    // None.
    //
    // Example:
    // let diags = validate_resource_budget(&budget, task_span);

    // Unpack budget limits for range validation.
    let ResourceBudgetDecl::ResourceBudgetDecl {
        battery_pct_max,
        memory_mb_max,
        cpu_pct_max,
        gpu_pct_max,
        network_mbps_max,
        storage_mb_max,
        ..
    } = budget;
    let mut diags = Vec::new();
    let check_pct = |label: &str, value: Option<f64>, diags: &mut Vec<Diagnostic>| {
        if let Some(v) = value {
            if v <= 0.0 || v > 100.0 {
                diags.push(Diagnostic {
                    message: format!("Resource budget {label} must be in (0, 100] (got {v})."),
                    line: span.start.line,
                    column: span.start.column,
                });
            }
        }
    };
    check_pct("cpu", *cpu_pct_max, &mut diags);
    check_pct("gpu", *gpu_pct_max, &mut diags);
    check_pct("battery", *battery_pct_max, &mut diags);

    // Reject non-positive memory/network/storage ceilings.
    for (label, value) in [
        ("memory", *memory_mb_max),
        ("network", *network_mbps_max),
        ("storage", *storage_mb_max),
    ] {
        if let Some(v) = value {
            if v <= 0.0 {
                diags.push(Diagnostic {
                    message: format!("Resource budget {label} must be positive (got {v})."),
                    line: span.start.line,
                    column: span.start.column,
                });
            }
        }
    }

    // Flag impossible combined CPU/GPU budgets above 100%.
    if let (Some(cpu), Some(gpu)) = (cpu_pct_max, gpu_pct_max) {
        if cpu + gpu > 100.0 {
            diags.push(Diagnostic {
                message: format!(
                    "Resource budget cpu ({cpu}%) + gpu ({gpu}%) exceeds 100%. Suggestion: reduce one ceiling."
                ),
                line: span.start.line,
                column: span.start.column,
            });
        }
    }
    diags
}

pub fn validate_recover(recover: &RecoverDecl) -> Vec<Diagnostic> {
    // Validate recovery handler safety requirements.
    //
    // Parameters:
    // - `recover` — recovery handler declaration
    //
    // Returns:
    // Diagnostics when recovery blocks omit safety actions for runtime errors.
    //
    // Options:
    // None.
    //
    // Example:
    // let diags = validate_recover(&recover);

    // Unpack recovery handler metadata.
    let RecoverDecl::RecoverDecl {
        error_name,
        body,
        span,
        ..
    } = recover;
    let mut diags = Vec::new();

    // RuntimeError recovery must include actuator stop or degraded mode entry.
    if error_name == "RuntimeError" {
        let has_safe_action = body.iter().any(|stmt| {
            matches!(
                stmt,
                crate::ast::Stmt::StopAllActuatorsStmt { .. }
                    | crate::ast::Stmt::EnterModeStmt { .. }
            )
        });
        if !has_safe_action {
            diags.push(Diagnostic {
                message: "Recovery from RuntimeError should stop actuators or enter degraded mode. Suggestion: add stop_all_actuators() or enter degraded_mode;".into(),
                line: span.start.line,
                column: span.start.column,
            });
        }
    }
    diags
}
