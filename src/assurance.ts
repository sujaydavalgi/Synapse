/**
 * TypeScript mission assurance analysis (native CLI fallback).
 * @module
 */

import type { Program } from "./ast/nodes.js";
import {
  evaluateReadinessTs,
  type ReadinessOptions,
} from "./readiness.js";
import { verifyHardwareProgram } from "./hardware-verify.js";

export type MissionVerificationReport = {
  achievable: boolean;
  mission_name: string | null;
  robot: string | null;
  required_capabilities: string[];
  hardware_satisfied: boolean;
  capabilities_satisfied: boolean;
  connectivity_satisfied: boolean;
  battery_sufficient: boolean;
  compute_sufficient: boolean;
  safety_satisfied: boolean;
  issues: string[];
};

export type AssuranceCase = {
  name: string;
  evidence: Array<{ source: string; kind: string; status: string }>;
};

export type AssuranceReport = {
  cases: AssuranceCase[];
  verification: { compatible: boolean; items: string[] };
  safety: { rules: string[]; kill_switches: string[] };
  traceability: { rows: string[] };
  safety_case: {
    program: string;
    deployable: boolean;
    known_risks: string[];
    safety_rules: string[];
    kill_switch_validation: string[];
  };
  passed: boolean;
};

export type Anomaly = {
  detector: string;
  metric: string;
  expected: string;
  observed: string;
  severity: string;
};

export type AnomalyReport = {
  detectors: Array<{ detector: string; rules: string[] }>;
  handlers: string[];
  anomalies: Anomaly[];
  learned: Array<{ detector: string; backend: string }>;
  passed: boolean;
};

export type PrognosticsReport = {
  models: Array<{ name: string; target: string; rules: string[] }>;
  rul_predictions: Array<{ component: string; estimate: string; confidence: number }>;
  failure_predictions: unknown[];
  trends: unknown[];
  warnings: string[];
  passed: boolean;
};

export type ResilienceReport = {
  policies: Array<{ name: string; strategies: Array<{ name: string; description: string }> }>;
  recovery: Array<{ name: string; actions: string[] }>;
  redundancy: unknown[];
  readiness_score: number;
  passed: boolean;
};

export type MissionAssuranceReport = {
  plans: Array<{ name: string; steps: string[]; constraints: string[] }>;
  execution: { plan: string; current_step: string | null; status: string };
  verification: MissionVerificationReport;
  abort_reasons: Array<{ reason: string; severity: string }>;
  passed: boolean;
};

export type MitigationReport = {
  plans: Array<{
    name: string;
    actions: Array<{ description: string; condition: string | null }>;
    fallback: { mode: string } | null;
  }>;
  transitions: Array<{ from_mode: string; to_mode: string; trigger: string }>;
  passed: boolean;
};

export type DiagnosisReport = {
  static_diagnoses: Array<{
    subject: string;
    root_causes: Array<{ description: string; confidence: number; contributing: string[] }>;
    fault_tree: { top_event: string; gates: string[] };
  }>;
  trace_diagnosis: null;
  causal_graph: { nodes: string[]; edges: Array<[string, string]> };
  passed: boolean;
};

export type StateAssuranceReport = {
  estimators: Array<{
    estimator: string;
    inputs: string[];
    fused: { name: string; value: string; confidence: number; sources: string[] } | null;
  }>;
  belief: {
    estimates: Array<{ name: string; value: string; confidence: number; sources: string[] }>;
  };
  issues: string[];
  passed: boolean;
};

export type MissionAssuranceSummary = {
  assurance: AssuranceReport;
  anomalies: AnomalyReport;
  prognostics: PrognosticsReport;
  resilience: ResilienceReport;
  mission: MissionAssuranceReport;
  state: StateAssuranceReport;
  issues: string[];
  passed: boolean;
};

function evidenceKind(source: string): string {
  if (source.includes("hardware")) return "Hardware";
  if (source.includes("capability") || source.includes("traceability")) return "Capability";
  if (source.includes("health")) return "Health";
  if (source.includes("replay") || source.includes("simulation")) return "Replay";
  if (source.includes("safety")) return "Safety";
  return "Verification";
}

function buildSafetyCase(program: Program, sourceLabel: string): AssuranceReport["safety_case"] {
  const hw = verifyHardwareProgram(program);
  const known_risks = hw.items.filter((i) => i.severity === "warning").map((i) => i.message);
  return {
    program: sourceLabel,
    deployable: Boolean(hw.compatible) && known_risks.length === 0,
    known_risks,
    safety_rules: (program.robots ?? []).flatMap((r) =>
      r.safety ? [`${r.name}: max_speed and stop rules`] : [],
    ),
    kill_switch_validation: (program.killSwitches ?? []).map((k) => k.name),
  };
}

function verifyMissionForAssurance(program: Program, target?: string): MissionVerificationReport[] {
  const hw = verifyHardwareProgram(program, { target, allTargets: !target });
  const reports: MissionVerificationReport[] = [];

  for (const robot of program.robots ?? []) {
    if (!robot.mission) continue;
    const required = robot.mission.requiredCapabilities ?? [];
    const issues: string[] = [];
    let capsOk = true;
    for (const cap of required) {
      const has =
        robot.exposesCapabilities.includes(cap) ||
        robot.sensors.some((s) => s.sensorType.toLowerCase().includes(cap.toLowerCase()));
      if (!has) {
        capsOk = false;
        issues.push(`Missing required capability: ${cap}`);
      }
    }
    const batteryOk = !hw.items.some(
      (i) => i.message.toLowerCase().includes("battery") && i.severity === "error",
    );
    const hwErrors = hw.items.filter((i) => i.severity === "error");
    const hwOk = Boolean(hw.compatible) && hwErrors.length === 0;
    reports.push({
      achievable: capsOk && hwOk && batteryOk,
      mission_name: robot.mission.name,
      robot: robot.name,
      required_capabilities: required,
      hardware_satisfied: hwOk,
      capabilities_satisfied: capsOk,
      connectivity_satisfied: Boolean(hw.compatible),
      battery_sufficient: batteryOk,
      compute_sufficient: true,
      safety_satisfied: Boolean(hw.compatible),
      issues,
    });
  }
  return reports;
}

export function buildAssuranceReport(program: Program, sourceLabel: string): AssuranceReport {
  const cases = (program.assuranceCases ?? []).map((decl) => ({
    name: decl.name,
    evidence: decl.evidence.map((e) => ({
      source: e,
      kind: evidenceKind(e),
      status: "linked",
    })),
  }));

  const hw = verifyHardwareProgram(program);
  const hwErrors = hw.items.filter((i) => i.severity === "error");
  const verification = {
    compatible: Boolean(hw.compatible) && hwErrors.length === 0,
    items: hw.items.slice(0, 10).map((i) => i.message),
  };

  const safetyCase = buildSafetyCase(program, sourceLabel);
  const safety = {
    rules: safetyCase.safety_rules,
    kill_switches: safetyCase.kill_switch_validation,
  };

  const traceability = {
    rows: (program.requiresCapabilities ?? []).map((r) => `capability: ${r.capability}`),
  };

  const passed = verification.compatible && cases.length > 0;

  return {
    cases,
    verification,
    safety,
    traceability,
    safety_case: safetyCase,
    passed,
  };
}

export function learnedModelsTs(program: Program): Array<{ detector: string; backend: string }> {
  const importBackend = (program.imports ?? []).find(
    (imp) => imp.path.includes("assurance.anomaly") || imp.path.endsWith("anomaly"),
  )?.path;
  const learned: Array<{ detector: string; backend: string }> = [];
  for (const decl of program.anomalyDetectors ?? []) {
    const backend = decl.learnedBackend ?? importBackend ?? null;
    if (backend) {
      learned.push({ detector: decl.name, backend });
    }
  }
  return learned;
}

export function scanAnomaliesTs(program: Program): AnomalyReport {
  const detectors = (program.anomalyDetectors ?? []).map((decl) => ({
    detector: decl.name,
    rules: decl.expected.map((e) => `${e.metric} ${e.operator} ${e.threshold}`),
  }));

  const handlers = (program.anomalyHandlers ?? []).map(
    (h) => `${h.detector}@${h.severity}: ${h.actions.join(", ")}`,
  );

  const handlerNames = new Set((program.anomalyHandlers ?? []).map((h) => h.detector));
  const anomalies: Anomaly[] = [];

  for (const decl of program.anomalyDetectors ?? []) {
    if (decl.expected.length === 0) {
      anomalies.push({
        detector: decl.name,
        metric: "expected",
        expected: "at least one rule",
        observed: "none",
        severity: "Medium",
      });
    }
    if (!handlerNames.has(decl.name)) {
      anomalies.push({
        detector: decl.name,
        metric: "handler",
        expected: "on anomaly handler",
        observed: "missing",
        severity: "Low",
      });
    }
  }

  const passed = !anomalies.some((a) => a.severity === "Critical" || a.severity === "High");

  return { detectors, handlers, anomalies, learned: learnedModelsTs(program), passed };
}

export function evaluatePrognosticsTs(program: Program): PrognosticsReport {
  const models: PrognosticsReport["models"] = [];
  const rul_predictions: PrognosticsReport["rul_predictions"] = [];
  const warnings: string[] = [];

  for (const decl of program.prognostics ?? []) {
    const ruleStrs = decl.rules.map((r) =>
      r.threshold ? `${r.kind} ${r.target} ${r.threshold}` : `${r.kind} ${r.target}`,
    );
    models.push({
      name: decl.name,
      target: decl.rules[0]?.target ?? "",
      rules: ruleStrs,
    });
    for (const rule of decl.rules) {
      if (rule.kind === "predict") {
        rul_predictions.push({
          component: rule.target,
          estimate: rule.threshold ?? "unknown",
          confidence: 0.75,
        });
      }
      if (rule.kind === "warn_if" && rule.threshold) {
        warnings.push(`Prognostics '${decl.name}': warn if ${rule.target} < ${rule.threshold}`);
      }
    }
  }

  const passed = warnings.length === 0 || (program.prognostics?.length ?? 0) > 0;

  return {
    models,
    rul_predictions,
    failure_predictions: [],
    trends: [],
    warnings,
    passed,
  };
}

export function checkResilienceTs(
  program: Program,
  options: ReadinessOptions = {},
): ResilienceReport {
  const policies = (program.resiliencePolicies ?? []).map((decl) => ({
    name: decl.name,
    strategies: decl.strategies.map((s) => ({ name: s, description: `Strategy: ${s}` })),
  }));

  const recovery = (program.mitigations ?? []).map((m) => ({
    name: m.name,
    actions: m.branches.flatMap((b) => b.actions),
  }));

  const readiness = evaluateReadinessTs(program, options);
  const resiliencePolicies = program.resiliencePolicies ?? [];
  const passed =
    (readiness.mission_ready && policies.length > 0) || resiliencePolicies.length === 0;

  return {
    policies,
    recovery,
    redundancy: [],
    readiness_score: readiness.score.total,
    passed,
  };
}

export function verifyMissionAssuranceTs(
  program: Program,
  target?: string,
): MissionAssuranceReport {
  const plans = (program.missionPlans ?? []).map((decl) => ({
    name: decl.name,
    steps: decl.steps.map((s) => s.name),
    constraints: decl.constraints.map((c) => c.constraint),
  }));

  const verifications = verifyMissionForAssurance(program, target);
  const verification = verifications[0] ?? {
    achievable: true,
    mission_name: null,
    robot: null,
    required_capabilities: [],
    hardware_satisfied: true,
    capabilities_satisfied: true,
    connectivity_satisfied: true,
    battery_sufficient: true,
    compute_sufficient: true,
    safety_satisfied: true,
    issues: [],
  };

  const missionPlans = program.missionPlans ?? [];
  const passed =
    (verifications.every((v) => v.achievable) && plans.length > 0) || missionPlans.length === 0;

  return {
    plans,
    execution: {
      plan: plans[0]?.name ?? "",
      current_step: plans[0]?.steps[0] ?? null,
      status: passed ? "ready" : "blocked",
    },
    verification,
    abort_reasons: verification.issues.map((reason) => ({ reason, severity: "High" })),
    passed,
  };
}

export function extractMitigationsTs(program: Program): MitigationReport {
  const plans = (program.mitigations ?? []).map((decl) => {
    const actions = decl.branches.flatMap((b) =>
      b.actions.map((a) => ({ description: a, condition: b.condition })),
    );
    const fallbackAction = decl.branches
      .flatMap((b) => b.actions)
      .find((a) => a.includes("degraded") || a.includes("safe"));
    return {
      name: decl.name,
      actions,
      fallback: fallbackAction ? { mode: fallbackAction } : null,
    };
  });

  const transitions = (program.operatingModes ?? []).map((m) => ({
    from_mode: "normal",
    to_mode: m.name,
    trigger: m.modeKind,
  }));

  const mitigations = program.mitigations ?? [];
  const passed = plans.length > 0 || mitigations.length === 0;

  return { plans, transitions, passed };
}

export function diagnoseProgramTs(program: Program): DiagnosisReport {
  const static_diagnoses = (program.mitigations ?? []).map((mit) => ({
    subject: mit.name,
    root_causes: mit.branches.map((b) => ({
      description: b.condition,
      confidence: 0.7,
      contributing: b.actions,
    })),
    fault_tree: {
      top_event: mit.name,
      gates: mit.branches.flatMap((b) => b.actions),
    },
  }));

  const nodes = ["system"];
  const edges: Array<[string, string]> = [];

  for (const det of program.anomalyDetectors ?? []) {
    nodes.push(det.name);
    edges.push(["system", det.name]);
  }
  for (const handler of program.anomalyHandlers ?? []) {
    for (const action of handler.actions) {
      nodes.push(action);
      edges.push([handler.detector, action]);
    }
  }

  const passed = static_diagnoses.length > 0 || (program.anomalyDetectors?.length ?? 0) > 0;

  return {
    static_diagnoses,
    trace_diagnosis: null,
    causal_graph: { nodes, edges },
    passed,
  };
}

function validateKnowledgeModels(program: Program): string[] {
  const issues: string[] = [];
  const models = program.knowledgeModels ?? [];
  for (const model of models) {
    if (model.components.length === 0) {
      issues.push(`Knowledge model '${model.name}' has no components declared`);
    }
    for (const dep of model.dependencies) {
      if (dep.requires.length === 0) {
        issues.push(
          `Knowledge model '${model.name}': dependency '${dep.capability}' has empty requires list`,
        );
      }
    }
  }
  if (models.length === 0 && (program.robots?.length ?? 0) > 0) {
    issues.push("Robot declared but no knowledge_model defined");
  }
  return issues;
}

function validateStateEstimators(program: Program): string[] {
  const issues: string[] = [];
  for (const est of program.stateEstimators ?? []) {
    if (est.inputs.length === 0) {
      issues.push(`State estimator '${est.name}' has no inputs`);
    }
  }
  return issues;
}

function validateModes(program: Program): string[] {
  const modes = program.operatingModes ?? [];
  if (modes.length === 0) return [];
  const hasSafe = modes.some((m) => /safe/i.test(m.modeKind));
  const hasDegraded = modes.some((m) => /degraded/i.test(m.modeKind));
  const issues: string[] = [];
  if (!hasSafe) issues.push("No safe mode declared");
  if (!hasDegraded) issues.push("No degraded mode declared");
  return issues;
}

export function evaluateStateAssuranceTs(program: Program): StateAssuranceReport {
  const estimators = (program.stateEstimators ?? []).map((decl) => ({
    estimator: decl.name,
    inputs: decl.inputs,
    fused: {
      name: decl.outputType,
      value: "synthetic",
      confidence: 0.85,
      sources: decl.inputs,
    },
  }));
  const belief = {
    estimates: estimators.flatMap((e) => (e.fused ? [e.fused] : [])),
  };
  const issues = validateStateEstimators(program);
  return { estimators, belief, issues, passed: issues.length === 0 };
}

export function assureProgramTs(program: Program, sourceLabel: string): MissionAssuranceSummary {
  const assurance = buildAssuranceReport(program, sourceLabel);
  const anomalies = scanAnomaliesTs(program);
  const prognostics = evaluatePrognosticsTs(program);
  const resilience = checkResilienceTs(program);
  const mission = verifyMissionAssuranceTs(program);
  const state = evaluateStateAssuranceTs(program);

  const issues = [...validateKnowledgeModels(program), ...state.issues, ...validateModes(program)];

  const passed =
    assurance.passed &&
    anomalies.passed &&
    prognostics.passed &&
    resilience.passed &&
    mission.passed &&
    state.passed &&
    issues.length === 0;

  return {
    assurance,
    anomalies,
    prognostics,
    resilience,
    mission,
    state,
    issues,
    passed,
  };
}

export function formatAssuranceReport(report: AssuranceReport): string {
  return `Assurance Report\nPassed: ${report.passed}\nCases: ${report.cases.length}\n`;
}

export function formatAnomalyReport(report: AnomalyReport): string {
  if (report.anomalies.length === 0) {
    return `Anomaly Report\nPassed: ${report.passed}\n`;
  }
  const lines = report.anomalies.map(
    (a) => `- ${a.detector} ${a.metric} (expected ${a.expected}, observed ${a.observed})`,
  );
  return `Anomaly Report\nPassed: ${report.passed}\n\n${lines.join("\n")}\n`;
}

export function formatPrognosticsReport(report: PrognosticsReport): string {
  const warnings =
    report.warnings.length > 0 ? `\nWarnings:\n${report.warnings.map((w) => `- ${w}`).join("\n")}` : "";
  return `Prognostics Report\nPassed: ${report.passed}\nModels: ${report.models.length}${warnings}\n`;
}

export function formatResilienceReport(report: ResilienceReport): string {
  return `Resilience Report\nPassed: ${report.passed}\nPolicies: ${report.policies.length}\nReadiness score: ${report.readiness_score}\n`;
}

export function formatStateReport(report: StateAssuranceReport): string {
  const lines = report.estimators.map(
    (e) => `* ${e.estimator} inputs [${e.inputs.join(", ")}]`,
  );
  return `State Estimation Report\nPassed: ${report.passed}\nEstimators: ${report.estimators.length}\n${lines.join("\n")}\n`;
}

export function formatMissionAssuranceReport(report: MissionAssuranceReport): string {
  return `Mission Assurance\nPassed: ${report.passed}\nPlans: ${report.plans.length}\nStatus: ${report.execution.status}\n`;
}

export function formatMitigationReport(report: MitigationReport): string {
  return `Mitigation Plan\nPassed: ${report.passed}\nPlans: ${report.plans.length}\n`;
}

export function formatDiagnosisReport(report: DiagnosisReport): string {
  const lines = report.static_diagnoses.map(
    (d) => `${d.subject}: ${d.root_causes.map((r) => r.description).join("; ")}`,
  );
  return `Diagnosis Report\nPassed: ${report.passed}\n${lines.join("\n")}\n`;
}
