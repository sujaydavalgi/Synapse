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

const DEFAULT_WEIGHTS = {
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
} as const;

const RUNTIME_FAULTS = ["GPSDegraded", "CameraOffline", "RobotHealthCritical"];

const weightFor = (key: keyof typeof DEFAULT_WEIGHTS): number => DEFAULT_WEIGHTS[key];

function defaultDeployTarget(program: Program): string | undefined {
  const deployments = program.deployments ?? [];
  const first = deployments[0];
  if (!first || first.kind !== "DeployDecl") return undefined;
  return first.targets[0];
}

function lineColumnForFactor(program: Program, factor: string): { line: number; column: number } {
  const robot = program.robots[0];
  const deploy = program.deployments[0];
  const health = program.healthChecks[0];
  const fleet = program.fleets[0];
  const missionRobot = program.robots.find((r) => r.mission);
  if (factor === "Health" && health) {
    return { line: health.span.start.line, column: health.span.start.column };
  }
  if ((factor === "Capabilities" || factor === "Mission Requirements") && missionRobot?.mission) {
    return {
      line: missionRobot.mission.span.start.line,
      column: missionRobot.mission.span.start.column,
    };
  }
  if (factor === "Safety" && robot?.safety) {
    return { line: robot.safety.span.start.line, column: robot.safety.span.start.column };
  }
  if (factor === "Fleet" && fleet) {
    return { line: fleet.span.start.line, column: fleet.span.start.column };
  }
  if (deploy && deploy.kind === "DeployDecl") {
    return { line: deploy.span.start.line, column: deploy.span.start.column };
  }
  if (robot) {
    return { line: robot.span.start.line, column: robot.span.start.column };
  }
  return { line: 1, column: 1 };
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

  const hwErrors = hw.items.filter((i) => i.severity === "error");
  const hwScore = hw.compatible && hwErrors.length === 0 ? 100 : hw.compatible ? 85 : 40;
  factors.push(factorRow("Hardware", hwScore, weightFor("Hardware")));
  for (const item of hwErrors) {
    issues.push({
      factor: "Hardware",
      severity: "High",
      message: item.message,
    });
  }

  const capScore = hw.compatible ? 90 : 50;
  factors.push(factorRow("Capabilities", capScore, weightFor("Capabilities")));

  const runtimeFaults =
    options.includeRuntime && options.injectHealthFaults ? [...RUNTIME_FAULTS] : [];
  const health = healthScoreFromProgram(program, runtimeFaults);
  factors.push(factorRow("Health", health.score, weightFor("Health")));
  issues.push(...health.issues);

  factors.push(factorRow("Connectivity", hw.compatible ? 90 : 70, weightFor("Connectivity")));
  factors.push(factorRow("Safety", hw.compatible ? 95 : 45, weightFor("Safety")));
  factors.push(factorRow("Battery", 90, weightFor("Battery")));
  factors.push(factorRow("Storage", 90, weightFor("Storage")));
  factors.push(factorRow("Compute", 88, weightFor("Compute")));
  factors.push(factorRow("Packages", 88, weightFor("Packages")));
  factors.push(factorRow("Providers", 88, weightFor("Providers")));
  factors.push(factorRow("Mission Requirements", capScore, weightFor("Mission Requirements")));

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
  const program = parse(tokenize(source));
  return evaluateReadinessTs(program, options);
}

/** Agent-shaped readiness JSON (`GET /v1/readiness` envelope). */
export function evaluateAgentReadinessJson(
  source: string,
  options: ReadinessOptions = {},
): string {
  // Build the same JSON payload deploy and fleet agents return over HTTP.
  //
  // Parameters:
  // - `source` — deployed `.sd` program text
  // - `options` — target, runtime, and fault-injection flags
  //
  // Returns:
  // JSON string `{"ok":true,"mission_ready":...,"readiness":...}`.
  //
  // Options:
  // None.
  //
  // Example:
  // const body = evaluateAgentReadinessJson(programText, { includeRuntime: true });

  const report = evaluateReadinessSource(source, options);
  return JSON.stringify({
    ok: true,
    mission_ready: report.mission_ready,
    readiness: report,
  });
}

function mapSeverityToDiagnostic(severity: ReadinessSeverity): string {
  switch (severity) {
    case "Critical":
    case "High":
      return "error";
    case "Medium":
      return "warning";
    case "Low":
    case "Info":
    default:
      return "info";
  }
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
  const program = parse(tokenize(source));
  const report = evaluateReadinessTs(program, options);
  return report.issues.map((issue) => {
    const span = lineColumnForFactor(program, issue.factor);
    return {
      message: issue.message,
      line: span.line,
      column: span.column,
      severity: mapSeverityToDiagnostic(issue.severity),
      category: `readiness:${issue.factor.toLowerCase()}`,
      suggested_fix: issue.suggested_action,
    };
  });
}
