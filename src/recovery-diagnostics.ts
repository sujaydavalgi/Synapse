/**
 * Span-aware recovery policy diagnostics for IDE and check JSON fallbacks.
 * @module
 */

import type { Program } from "./ast/nodes.js";
import { tokenize } from "./lexer/index.js";
import { parse } from "./parser/index.js";

export type RecoveryDiagnostic = {
  message: string;
  line: number;
  column: number;
  severity: string;
  category: string;
  suggested_fix?: string;
};

function normalizeAction(action: string): string {
  return action.toLowerCase().replace(/\s+/g, "");
}

function actionIsHighRisk(action: string): boolean {
  const lower = normalizeAction(action);
  return (
    lower.includes("resumemission") ||
    lower.includes("restartfleet") ||
    lower.includes("unsafe") ||
    lower.includes("opengate")
  );
}

function robotHasApprovalTopic(program: Program): boolean {
  for (const robot of program.robots ?? []) {
    for (const topic of robot.topics ?? []) {
      if (topic.messageType === "Approval") {
        return true;
      }
    }
  }
  return false;
}

function firstHealthSpan(program: Program): { line: number; column: number } {
  const health = program.healthChecks?.[0];
  if (health?.span) {
    return { line: health.span.start.line, column: health.span.start.column };
  }
  const policy = program.healthPolicies?.[0];
  if (policy?.span) {
    return { line: policy.span.start.line, column: policy.span.start.column };
  }
  const handler = program.anomalyHandlers?.[0];
  if (handler?.span) {
    return { line: handler.span.start.line, column: handler.span.start.column };
  }
  return { line: 1, column: 1 };
}

/** Collect recovery-policy diagnostics mirroring the Rust assurance crate. */
export function collectRecoveryDiagnostics(program: Program): RecoveryDiagnostic[] {
  const diags: RecoveryDiagnostic[] = [];
  const recoveryPolicies = program.recoveryPolicies ?? [];
  const mitigations = program.mitigations ?? [];
  const healthChecks = program.healthChecks ?? [];
  const healthPolicies = program.healthPolicies ?? [];
  const anomalyHandlers = program.anomalyHandlers ?? [];
  const fleets = program.fleets ?? [];

  const hasHealth =
    healthChecks.length > 0 || healthPolicies.length > 0 || anomalyHandlers.length > 0;
  const hasRecovery = recoveryPolicies.length > 0 || mitigations.length > 0;

  if (hasHealth && !hasRecovery) {
    const span = firstHealthSpan(program);
    diags.push({
      message: "Health or anomaly handling declared without recovery_policy or mitigation",
      line: span.line,
      column: span.column,
      severity: "warning",
      category: "recovery:policy",
      suggested_fix:
        "Add recovery_policy or mitigation branches for detected failure modes",
    });
  }

  const approvalPath = robotHasApprovalTopic(program);

  for (const policy of recoveryPolicies) {
    if (policy.branches.length === 0) {
      diags.push({
        message: `recovery_policy '${policy.name}' has no on branches`,
        line: policy.span.start.line,
        column: policy.span.start.column,
        severity: "warning",
        category: "recovery:policy",
        suggested_fix: "Add on <condition> { actions; } branches",
      });
      continue;
    }
    for (const branch of policy.branches) {
      const triggerLower = branch.condition.toLowerCase();
      if (
        (triggerLower.includes("fleet") || triggerLower.includes("swarm")) &&
        fleets.length === 0
      ) {
        diags.push({
          message: `recovery_policy '${policy.name}' references fleet failures but no fleet is declared`,
          line: branch.span.start.line,
          column: branch.span.start.column,
          severity: "error",
          category: "recovery:fleet",
          suggested_fix: "Declare fleet <Name> { members; } or adjust trigger",
        });
      }
      for (const action of branch.actions) {
        if (actionIsHighRisk(action) && !approvalPath) {
          diags.push({
            message: `High-risk recovery action '${action}' should have an Approval topic or operator path`,
            line: branch.span.start.line,
            column: branch.span.start.column,
            severity: "warning",
            category: "recovery:approval",
            suggested_fix:
              'Add topic approval: Approval subscribe on "/ops/approval"; or mission requires approval Operator',
          });
        }
      }
    }
  }

  return diags;
}

/** Parse source and collect recovery diagnostics for LSP/check JSON. */
export function recoveryDiagnosticsFromSource(source: string): RecoveryDiagnostic[] {
  const program = parse(tokenize(source));
  return collectRecoveryDiagnostics(program);
}
