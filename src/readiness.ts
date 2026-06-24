/**
 * TypeScript operational readiness evaluation (native CLI fallback).
 * @module
 */

import type { DeployDecl } from "./foundations.js";
import type { Program } from "./ast/nodes.js";
import { tokenize } from "./lexer/index.js";
import { parse } from "./parser/index.js";
import {
  verifyHardwareProgram,
  type VerifyHardwareTsOptions,
} from "./hardware-verify.js";
import { lineColumnForIssue } from "./readiness-spans.js";

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

export type ReadinessDashboard = {
  overall_score: number;
  mission_ready_count: number;
  degraded_count: number;
  not_ready_count: number;
  top_issues: string[];
  reports: ReadinessReport[];
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
  "Mission Requirements": 2,
  Assurance: 8,
} as const;

const RUNTIME_FAULTS = ["GPSDegraded", "CameraOffline", "RobotHealthCritical"];

const weightFor = (key: keyof typeof DEFAULT_WEIGHTS): number => DEFAULT_WEIGHTS[key];

function isValidDeployDecl(candidate: DeployDecl | undefined): candidate is DeployDecl {
  return !!candidate && "kind" in candidate && candidate.kind === "DeployDecl";
}

function defaultDeployTarget(program: Program): string | undefined {
  const deployments = program.deployments ?? [];
  const first = deployments[0];
  if (!isValidDeployDecl(first)) {
    return undefined;
  }
  if (!first.targets?.length) return undefined;
  return first.targets[0];
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

const HEALTH_SCORE_NO_CHECKS_OR_FAULTS = 85;
const HEALTH_SCORE_RUNTIME_FAULTS = 55;
const HEALTH_SCORE_HEALTHY = 100;
const CONNECTIVITY_SCORE_COMPATIBLE = 90;
const CONNECTIVITY_SCORE_INCOMPATIBLE = 70;
const SAFETY_SCORE_COMPATIBLE = 95;
const SAFETY_SCORE_INCOMPATIBLE = 45;
const DEFAULT_HIGH_FACTOR_SCORE = 90;
const DEFAULT_STANDARD_FACTOR_SCORE = 88;

function healthScoreFromProgram(
  program: Program,
  runtimeFaults: string[],
): { score: number; issues: ReadinessIssue[] } {
  const issues: ReadinessIssue[] = [];
  const checks = program.healthChecks ?? [];
  if (checks.length === 0 && runtimeFaults.length === 0) {
    return { score: HEALTH_SCORE_NO_CHECKS_OR_FAULTS, issues };
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
    return { score: HEALTH_SCORE_RUNTIME_FAULTS, issues };
  }
  return { score: HEALTH_SCORE_HEALTHY, issues };
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

  const isHardwareCompatible = hw.compatible === true;
  const hwErrors = hw.items.filter((i) => i.severity === "error");
  const hwScore = isHardwareCompatible && hwErrors.length === 0 ? 100 : isHardwareCompatible ? 85 : 40;
  factors.push(factorRow("Hardware", hwScore, weightFor("Hardware")));
  for (const item of hwErrors) {
    issues.push({
      factor: "Hardware",
      severity: "High",
      message: item.message,
    });
  }

  const capScore = isHardwareCompatible ? 90 : 50;
  factors.push(factorRow("Capabilities", capScore, weightFor("Capabilities")));

  const runtimeFaults =
    options.includeRuntime && options.injectHealthFaults ? [...RUNTIME_FAULTS] : [];
  const health = healthScoreFromProgram(program, runtimeFaults);
  factors.push(factorRow("Health", health.score, weightFor("Health")));
  issues.push(...health.issues);

  factors.push(
    factorRow(
      "Connectivity",
      isHardwareCompatible ? CONNECTIVITY_SCORE_COMPATIBLE : CONNECTIVITY_SCORE_INCOMPATIBLE,
      weightFor("Connectivity"),
    ),
  );
  factors.push(
    factorRow(
      "Safety",
      isHardwareCompatible ? SAFETY_SCORE_COMPATIBLE : SAFETY_SCORE_INCOMPATIBLE,
      weightFor("Safety"),
    ),
  );
  factors.push(factorRow("Battery", DEFAULT_HIGH_FACTOR_SCORE, weightFor("Battery")));
  factors.push(factorRow("Storage", DEFAULT_HIGH_FACTOR_SCORE, weightFor("Storage")));
  factors.push(factorRow("Compute", DEFAULT_STANDARD_FACTOR_SCORE, weightFor("Compute")));
  factors.push(factorRow("Packages", DEFAULT_STANDARD_FACTOR_SCORE, weightFor("Packages")));
  factors.push(factorRow("Providers", DEFAULT_STANDARD_FACTOR_SCORE, weightFor("Providers")));
  factors.push(factorRow("Mission Requirements", capScore, weightFor("Mission Requirements")));

  let assuranceScore = 100;
  const assuranceCases = program.assuranceCases ?? [];
  const knowledgeModels = program.knowledgeModels ?? [];
  const stateEstimators = program.stateEstimators ?? [];
  const anomalyDetectors = program.anomalyDetectors ?? [];
  const anomalyHandlers = program.anomalyHandlers ?? [];
  const mitigations = program.mitigations ?? [];

  if (assuranceCases.length > 0) {
    const allHaveEvidence = assuranceCases.every((c) => c.evidence.length > 0);
    if (!allHaveEvidence) {
      assuranceScore -= 40;
      issues.push({
        factor: "Assurance",
        severity: "High",
        message: "Assurance case missing evidence links",
        suggested_action: "Add evidence to assurance_case declarations",
      });
    }
  }
  if (
    knowledgeModels.length > 0 &&
    knowledgeModels.some((m) => m.components.length === 0)
  ) {
    assuranceScore -= 20;
    issues.push({
      factor: "Assurance",
      severity: "Medium",
      message: "Knowledge model has empty components",
    });
  }
  for (const est of stateEstimators) {
    if (est.inputs.length === 0) {
      assuranceScore -= 15;
      issues.push({
        factor: "Assurance",
        severity: "Medium",
        message: `State estimator '${est.name}' has no inputs`,
        suggested_action: "Add sensor inputs to state_estimator",
      });
    }
  }
  if (anomalyDetectors.length > 0) {
    const handlerNames = new Set(anomalyHandlers.map((h) => h.detector));
    for (const det of anomalyDetectors) {
      if (!handlerNames.has(det.name)) {
        assuranceScore -= 10;
        issues.push({
          factor: "Assurance",
          severity: "Low",
          message: `Anomaly detector '${det.name}' has no on anomaly handler`,
          suggested_action: "Add on anomaly handler",
        });
      }
    }
  }
  if (mitigations.length === 0 && anomalyDetectors.length > 0) {
    assuranceScore -= 10;
  }
  factors.push(factorRow("Assurance", assuranceScore, weightFor("Assurance")));

  const total = weightedTotal(factors);
  const hasHigh = issues.some((i) => i.severity === "High" || i.severity === "Critical");
  const mission_ready = total >= 80 && isHardwareCompatible && !hasHigh;
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
    const span = lineColumnForIssue(program, issue);
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

export function readinessDashboardFromReports(reports: ReadinessReport[]): ReadinessDashboard {
  const mission_ready_count = reports.filter((r) => r.mission_ready).length;
  const degraded_count = reports.filter((r) => r.status === "Degraded").length;
  const not_ready_count = reports.filter((r) => r.status === "NotReady").length;
  const overall_score =
    reports.length === 0
      ? 0
      : Math.round(reports.reduce((sum, r) => sum + r.score.total, 0) / reports.length);
  const top_issues = reports.flatMap((r) => r.issues.map((i) => i.message)).slice(0, 10);
  return {
    overall_score,
    mission_ready_count,
    degraded_count,
    not_ready_count,
    top_issues,
    reports,
  };
}
