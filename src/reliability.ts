/**
 * Real-time reliability validation helpers for tasks, pipelines, and watchdogs.
 * @module
 */

import type { Span, Stmt } from "./ast/nodes.js";
import type {
  PipelineDecl,
  RecoverDecl,
  ResourceBudgetDecl,
  TaskDecl,
  TaskPriority,
  WatchdogDecl,
} from "./foundations.js";

export type Diagnostic = {
  message: string;
  line: number;
  column: number;
};

export function validateTaskTiming(task: TaskDecl): Diagnostic[] {
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
  // const diags = validateTaskTiming(task);

  const { name, intervalMs, deadlineMs, jitterMsMax, span } = task;
  const diags: Diagnostic[] = [];

  if (intervalMs <= 0) {
    diags.push({
      message: `Task '${name}' period must be positive (got ${intervalMs}ms). Suggestion: use \`every 10ms\` or larger.`,
      line: span.start.line,
      column: span.start.column,
    });
  }

  if (deadlineMs !== undefined && deadlineMs !== null) {
    if (deadlineMs <= 0) {
      diags.push({
        message: `Task '${name}' deadline must be positive (got ${deadlineMs}ms).`,
        line: span.start.line,
        column: span.start.column,
      });
    } else if (deadlineMs > intervalMs) {
      diags.push({
        message: `Task '${name}' deadline (${deadlineMs}ms) must be <= period (${intervalMs}ms). Suggestion: increase period or reduce deadline.`,
        line: span.start.line,
        column: span.start.column,
      });
    }
  }

  if (jitterMsMax !== undefined && jitterMsMax !== null) {
    if (jitterMsMax < 0) {
      diags.push({
        message: `Task '${name}' jitter must be non-negative.`,
        line: span.start.line,
        column: span.start.column,
      });
    }
    const slack = deadlineMs ?? intervalMs;
    if (jitterMsMax > slack) {
      diags.push({
        message: `Task '${name}' jitter (${jitterMsMax}ms) exceeds allowable slack (${slack}ms). Suggestion: reduce jitter or increase deadline/period.`,
        line: span.start.line,
        column: span.start.column,
      });
    }
  }

  return diags;
}

export function validateTaskPriority(task: TaskDecl): Diagnostic[] {
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
  // const diags = validateTaskPriority(task);

  const { name, priority, isolated, span } = task;
  const diags: Diagnostic[] = [];

  if (isolated && priority !== "critical" && priority !== "high") {
    diags.push({
      message: `Task '${name}' is marked isolated but priority is ${priority}. Suggestion: use \`critical isolated\` or \`high isolated\`.`,
      line: span.start.line,
      column: span.start.column,
    });
  }

  return diags;
}

export function validatePipeline(pipeline: PipelineDecl): Diagnostic[] {
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
  // const diags = validatePipeline(pipeline);

  const { name, budgetMs, span } = pipeline;
  const diags: Diagnostic[] = [];

  if (budgetMs <= 0) {
    diags.push({
      message: `Pipeline '${name}' budget must be positive (got ${budgetMs}ms).`,
      line: span.start.line,
      column: span.start.column,
    });
  }

  return diags;
}

export function validateWatchdog(watchdog: WatchdogDecl, taskNames: string[]): Diagnostic[] {
  // Validate watchdog timeout and monitored task target.
  //
  // Parameters:
  // - `watchdog` — watchdog declaration
  // - `taskNames` — known task names in the robot block
  //
  // Returns:
  // Diagnostics for invalid watchdog configuration.
  //
  // Options:
  // None.
  //
  // Example:
  // const diags = validateWatchdog(watchdog, taskNames);

  const { name, target, timeoutMs, span } = watchdog;
  const diags: Diagnostic[] = [];

  if (timeoutMs <= 0) {
    diags.push({
      message: `Watchdog '${name}' timeout must be positive.`,
      line: span.start.line,
      column: span.start.column,
    });
  }

  if (target && !taskNames.includes(target)) {
    diags.push({
      message: `Watchdog '${name}' target task '${target}' not found. Suggestion: declare the task before the watchdog or fix the task name.`,
      line: span.start.line,
      column: span.start.column,
    });
  }

  return diags;
}

export function validateResourceBudget(budget: ResourceBudgetDecl, span: Span): Diagnostic[] {
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
  // const diags = validateResourceBudget(budget, taskSpan);

  const {
    batteryPctMax,
    memoryMbMax,
    cpuPctMax,
    gpuPctMax,
    networkMbpsMax,
    storageMbMax,
  } = budget;
  const diags: Diagnostic[] = [];

  const checkPct = (label: string, value: number | null | undefined) => {
    if (value !== undefined && value !== null) {
      if (value <= 0 || value > 100) {
        diags.push({
          message: `Resource budget ${label} must be in (0, 100] (got ${value}).`,
          line: span.start.line,
          column: span.start.column,
        });
      }
    }
  };

  checkPct("cpu", cpuPctMax);
  checkPct("gpu", gpuPctMax);
  checkPct("battery", batteryPctMax);

  for (const [label, value] of [
    ["memory", memoryMbMax],
    ["network", networkMbpsMax],
    ["storage", storageMbMax],
  ] as const) {
    if (value !== undefined && value !== null && value <= 0) {
      diags.push({
        message: `Resource budget ${label} must be positive (got ${value}).`,
        line: span.start.line,
        column: span.start.column,
      });
    }
  }

  if (
    cpuPctMax !== undefined &&
    cpuPctMax !== null &&
    gpuPctMax !== undefined &&
    gpuPctMax !== null &&
    cpuPctMax + gpuPctMax > 100
  ) {
    diags.push({
      message: `Resource budget cpu (${cpuPctMax}%) + gpu (${gpuPctMax}%) exceeds 100%. Suggestion: reduce one ceiling.`,
      line: span.start.line,
      column: span.start.column,
    });
  }

  return diags;
}

function hasSafeRecoverAction(body: Stmt[]): boolean {
  return body.some(
    (stmt) => stmt.kind === "StopAllActuatorsStmt" || stmt.kind === "EnterModeStmt",
  );
}

export function validateRecover(recover: RecoverDecl): Diagnostic[] {
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
  // const diags = validateRecover(recover);

  const { errorName, body, span } = recover;
  const diags: Diagnostic[] = [];

  if (errorName === "RuntimeError" && !hasSafeRecoverAction(body)) {
    diags.push({
      message:
        "Recovery from RuntimeError should stop actuators or enter degraded mode. Suggestion: add stop_all_actuators() or enter degraded_mode;",
      line: span.start.line,
      column: span.start.column,
    });
  }

  return diags;
}

export function taskPriorityLabel(priority: TaskPriority): string {
  return priority;
}
