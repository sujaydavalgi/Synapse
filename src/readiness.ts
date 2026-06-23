/**
 * TypeScript operational readiness evaluation (native CLI fallback).
 * @module
 */

import type { Program } from "./ast/nodes.js";
import { tokenize } from "./lexer/index.js";
import { parse } from "./parser/index.js";
import {
  verifyHardwareProgram,
  type VerifyHardwareTsOptions,
} from "./hardware-verify.js";

export type ReadinessSeverity = "Critical" | "High" | "Medium" | "Low" | "Info";
export type ReadinessStatus = "Ready" | "Degraded" | "NotReady" | "Unknown";

export type ReadinessIssue = {
  factor: string;
  severity: ReadinessSeverity;
  message: string;
  suggested_action?: string;
};

export type ReadinessFactorScore = {
  factor: string;
  score: number;
  weight: number;
  weighted: number;
};

export type ReadinessReport = {
  status: ReadinessStatus;
  mission_ready: boolean;
  score: { total: number; maximum: number; factors: ReadinessFactorScore[] };
  issues: ReadinessIssue[];
  target?: string;
  robots: string[];
};

export type ReadinessOptions = {
  target?: string;
  includeRuntime?: boolean;
  injectHealthFaults?: boolean;
  simulate?: boolean;
  strictCertify?: boolean;
};

const DEFAULT_WEIGHTS: Record<string, number> = {
  Hardware: 12,
  Capabilities: 12,
  Health: 12,
  Connectivity: 8,
  Safety: 12,
  Battery: 8,
  Storage: 4,
  Compute: 6,
  Packages: 8,
  Providers: 8,
  "Mission Requirements": 10,
};

const RUNTIME_FAULTS = ["GPSDegraded", "CameraOffline", "RobotHealthCritical"];

function defaultDeployTarget(program: Program): string | undefined {
  const deployments = program.deployments ?? [];
  const first = deployments[0];
  if (!first || first.kind !== "DeployDecl") return undefined;
  return first.targets[0];
}

function parseProgramSource(source: string): Program {
  return parse(tokenize(source));
}

function factorRow(factor: string, score: number, weight: number): ReadinessFactorScore {
  return {
    factor,
    score,
    weight,
    weighted: (score * weight) / 100,
  };
}

function weightedTotal(factors: ReadinessFactorScore[]): number {
  const sum = factors.reduce((acc, f) => acc + f.weight, 0);
  if (sum === 0) return 0;
  const weighted = factors.reduce((acc, f) => acc + f.weighted, 0);
  return Math.round((weighted * 100) / sum);
}

function healthScoreFromProgram(
  program: Program,
  runtimeFaults: string[],
): { score: number; issues: ReadinessIssue[] } {
  const issues: ReadinessIssue[] = [];
  const checks = program.healthChecks ?? [];
  if (checks.length === 0 && runtimeFaults.length === 0) {
    return { score: 85, issues };
  }
  if (runtimeFaults.length > 0) {
    for (const fault of runtimeFaults) {
      issues.push({
        factor: "Health",
        severity: "Medium",
        message: `Runtime fault active: ${fault}`,
        suggested_action: "Review health policy reactions",
      });
    }
    return { score: 55, issues };
  }
  return { score: 100, issues };
}

/** Evaluate readiness from parsed program (TypeScript mirror). */
export function evaluateReadinessTs(
  program: Program,
  options: ReadinessOptions = {},
): ReadinessReport {
  const target = options.target ?? defaultDeployTarget(program);
  const verifyOpts: VerifyHardwareTsOptions = {
    target,
    allTargets: !target,
    simulate: options.simulate,
    strictCertify: options.strictCertify,
  };
  const hw = verifyHardwareProgram(program, verifyOpts);
  const issues: ReadinessIssue[] = [];
  const factors: ReadinessFactorScore[] = [];

  const hwErrors = hw.items.filter((i: { severity: string }) => i.severity === "error");
  const hwScore = hw.compatible && hwErrors.length === 0 ? 100 : hw.compatible ? 85 : 40;
  factors.push(factorRow("Hardware", hwScore, DEFAULT_WEIGHTS.Hardware!));
  for (const item of hwErrors) {
    issues.push({
      factor: "Hardware",
      severity: "High",
      message: item.message,
    });
  }

  const capScore = hw.compatible ? 90 : 50;
  factors.push(factorRow("Capabilities", capScore, DEFAULT_WEIGHTS.Capabilities!));

  const runtimeFaults =
    options.includeRuntime && options.injectHealthFaults ? [...RUNTIME_FAULTS] : [];
  const health = healthScoreFromProgram(program, runtimeFaults);
  factors.push(factorRow("Health", health.score, DEFAULT_WEIGHTS.Health!));
  issues.push(...health.issues);

  factors.push(factorRow("Connectivity", hw.compatible ? 90 : 70, DEFAULT_WEIGHTS.Connectivity!));
  factors.push(factorRow("Safety", hw.compatible ? 95 : 45, DEFAULT_WEIGHTS.Safety!));
  factors.push(factorRow("Battery", 90, DEFAULT_WEIGHTS.Battery!));
  factors.push(factorRow("Storage", 90, DEFAULT_WEIGHTS.Storage!));
  factors.push(factorRow("Compute", 88, DEFAULT_WEIGHTS.Compute!));
  factors.push(factorRow("Packages", 88, DEFAULT_WEIGHTS.Packages!));
  factors.push(factorRow("Providers", 88, DEFAULT_WEIGHTS.Providers!));
  factors.push(factorRow("Mission Requirements", capScore, DEFAULT_WEIGHTS["Mission Requirements"]!));

  const total = weightedTotal(factors);
  const hasHigh = issues.some((i) => i.severity === "High" || i.severity === "Critical");
  const mission_ready = total >= 80 && (hw.compatible ?? false) && !hasHigh;
  const status: ReadinessStatus = mission_ready
    ? issues.length > 0
      ? "Degraded"
      : "Ready"
    : total >= 65
      ? "Degraded"
      : "NotReady";

  const robots = (program.robots ?? []).map((r) => r.name);

  return {
    status,
    mission_ready,
    score: { total, maximum: 100, factors },
    issues,
    target,
    robots,
  };
}

/** Evaluate readiness from `.sd` source text. */
export function evaluateReadinessSource(
  source: string,
  options: ReadinessOptions = {},
): ReadinessReport {
  const program = parseProgramSource(source);
  return evaluateReadinessTs(program, options);
}

/** Map readiness issues to verification-style diagnostics for LSP/check JSON. */
export function readinessDiagnostics(
  source: string,
  options: ReadinessOptions = {},
): Array<{
  message: string;
  line: number;
  column: number;
  severity: string;
  category: string;
  suggested_fix?: string;
}> {
  const report = evaluateReadinessSource(source, options);
  return report.issues.map((issue) => ({
    message: issue.message,
    line: 1,
    column: 1,
    severity:
      issue.severity === "Critical" || issue.severity === "High"
        ? "error"
        : issue.severity === "Medium"
          ? "warning"
          : "info",
    category: `readiness:${issue.factor.toLowerCase()}`,
    suggested_fix: issue.suggested_action,
  }));
}
