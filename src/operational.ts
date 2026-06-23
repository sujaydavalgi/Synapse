/**
 * TypeScript operational assurance mirror (native CLI fallback).
 * @module
 */

import type { Program } from "./ast/nodes.js";
import { tokenize } from "./lexer/index.js";
import { parse } from "./parser/index.js";
import { verifyHardwareProgram } from "./hardware-verify.js";
import { analyzeProgram } from "./security/validate.js";
import { deserializeMissionTrace } from "./replay.js";
import {
  evaluateReadinessSource,
  evaluateReadinessTs,
  type ReadinessReport,
  type ReadinessOptions,
} from "./readiness.js";
import { readFileSync } from "node:fs";

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

export type FailureImpact = {
  component: string;
  consequence: string;
  mitigation: string;
  severity: string;
};

export type FailureAnalysisReport = {
  robot: string | null;
  impacts: FailureImpact[];
};

export type FleetVerifyFinding = {
  category: string;
  severity: string;
  message: string;
};

export type FleetVerifyReport = {
  compatible: boolean;
  findings: FleetVerifyFinding[];
};

export type ApprovalVerifyRow = {
  actor: string;
  action: string;
  approval_path_exists: boolean;
  actor_exists: boolean;
  fallback_exists: boolean;
  status: string;
};

export type ApprovalVerifyReport = {
  compatible: boolean;
  rows: ApprovalVerifyRow[];
};

export type AuditFinding = {
  severity: string;
  category: string;
  message: string;
  line: number;
  column: number;
};

export type SafetyAuditReport = {
  findings: AuditFinding[];
  critical_count: number;
  high_count: number;
  medium_count: number;
  low_count: number;
};

export type SafetyCaseReport = {
  program: string;
  deployable: boolean;
  known_risks: string[];
  safety_rules: string[];
  kill_switch_validation: string[];
};

export type FleetReadinessReport = {
  fleet_score: number;
  healthy_robots: number;
  degraded_robots: number;
  not_ready_robots: number;
  mission_capacity_percent: number;
  robot_reports: ReadinessReport[];
};

export type TwinReadinessStatus = {
  physical_ready: boolean;
  twin_ready: boolean;
  configuration_drift: string[];
  capability_drift: string[];
  health_drift: string[];
  overall: string;
};

export type RootCauseReport = {
  root_cause: string;
  contributing_factors: string[];
  timeline: Array<{ sim_time_ms: number; event: string; detail: string }>;
  recommended_actions: string[];
};

export type ReadinessDashboard = {
  overall_score: number;
  mission_ready_count: number;
  degraded_count: number;
  not_ready_count: number;
  top_issues: string[];
  reports: ReadinessReport[];
};

const FAILURE_SCENARIOS: Array<[string, string, string, string]> = [
  ["GPS", "Navigation degraded; position uncertainty increases", "Switch to visual odometry", "High"],
  ["Camera", "Obstacle avoidance degraded; perception limited", "Reduce speed and rely on Lidar", "High"],
  ["Lidar", "Obstacle avoidance offline; collision risk elevated", "Halt autonomous motion; require operator takeover", "Critical"],
  ["LTE", "Cloud telemetry and remote commands unavailable", "Offline mode activated; queue telemetry locally", "Medium"],
  ["WiFi", "Local network commands unavailable", "Fall back to LTE or autonomous mode", "Medium"],
  ["Battery", "Mission endurance reduced; forced return-to-base likely", "Return to charging dock; reduce mission scope", "High"],
  ["Provider", "Dependent capability unavailable at runtime", "Use bundled fallback provider or safe stop", "High"],
  ["Package", "Imported capability module missing or outdated", "Pin package version or install from registry", "Medium"],
];

export function parseProgramSource(source: string): Program {
  return parse(tokenize(source));
}

export function lineColumnForFactor(program: Program, factor: string): { line: number; column: number } {
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

export function verifyMissionTs(program: Program, target?: string): MissionVerificationReport[] {
  const hw = verifyHardwareProgram(program, { target, allTargets: !target });
  const reports: MissionVerificationReport[] = [];

  for (const robot of program.robots) {
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
      connectivity_satisfied: hw.compatible,
      battery_sufficient: batteryOk,
      compute_sufficient: true,
      safety_satisfied: hw.compatible,
      issues,
    });
  }
  return reports;
}

export function analyzeFailureTs(program: Program): FailureAnalysisReport {
  const robot = program.robots[0]?.name ?? null;
  const components = new Set<string>();
  for (const r of program.robots) {
    for (const s of r.sensors) components.add(s.sensorType);
    for (const a of r.actuators) components.add(a.actuatorType);
  }
  const impacts = FAILURE_SCENARIOS.filter(([component]) => {
    if (component === "Provider" || component === "Package" || component === "Battery") return true;
    if (component === "LTE" || component === "WiFi") return true;
    return [...components].some((c) => c.toLowerCase().includes(component.toLowerCase()));
  }).map(([component, consequence, mitigation, severity]) => ({
    component,
    consequence,
    mitigation,
    severity,
  }));
  return { robot, impacts };
}

export function verifyFleetTs(program: Program): FleetVerifyReport {
  const findings: FleetVerifyFinding[] = [];
  const names = new Set(program.robots.map((r) => r.name));
  if (program.robots.length > 1 && program.programSafetyZones.length === 0) {
    findings.push({
      category: "collision",
      severity: "warning",
      message: "Multiple robots without shared safety zones — collision risk",
    });
  }
  for (const fleet of program.fleets) {
    for (const member of fleet.members) {
      if (!names.has(member)) {
        findings.push({
          category: "communication",
          severity: "error",
          message: `Fleet '${fleet.name}' references unknown robot '${member}'`,
        });
      }
    }
  }
  return { compatible: !findings.some((f) => f.severity === "error"), findings };
}

export function verifyApprovalsTs(program: Program): ApprovalVerifyReport {
  const rows: ApprovalVerifyRow[] = [];
  const robotNames = new Set(program.robots.map((r) => r.name));
  for (const robot of program.robots) {
    if (!robot.mission) continue;
    const approvalTopics = robot.topics.filter(
      (t) => t.messageType === "Approval" || t.name.toLowerCase().includes("approval"),
    );
    if (approvalTopics.length > 0) {
      rows.push({
        actor: "Operator",
        action: robot.mission.steps[0] ?? "mission",
        approval_path_exists: true,
        actor_exists: robotNames.has(robot.name),
        fallback_exists: robot.modes.length > 0,
        status: robot.modes.length > 0 ? "PASS" : "FAIL",
      });
    }
  }
  return { compatible: rows.every((r) => r.status === "PASS"), rows };
}

export function auditProgramTs(program: Program, source: string): SafetyAuditReport {
  const findings: AuditFinding[] = [];
  if (program.killSwitches.length === 0) {
    findings.push({
      severity: "Critical",
      category: "kill-switch",
      message: "Missing kill switch declaration",
      line: 1,
      column: 1,
    });
  }
  for (const ks of program.killSwitches) {
    if (!ks.remoteSigned) {
      findings.push({
        severity: "High",
        category: "kill-switch",
        message: `Kill switch '${ks.name}' is not signed`,
        line: ks.span.start.line,
        column: ks.span.start.column,
      });
    }
  }
  if (program.healthChecks.length === 0) {
    findings.push({
      severity: "Medium",
      category: "health",
      message: "Missing health check declarations",
      line: 1,
      column: 1,
    });
  }
  const security = analyzeProgram(program);
  for (const issue of security.findings) {
    findings.push({
      severity: issue.severity === "error" ? "Critical" : issue.severity === "warning" ? "High" : "Low",
      category: "security",
      message: issue.message,
      line: issue.line,
      column: issue.column,
    });
  }
  const count = (sev: string) => findings.filter((f) => f.severity === sev).length;
  return {
    findings,
    critical_count: count("Critical"),
    high_count: count("High"),
    medium_count: count("Medium"),
    low_count: count("Low"),
  };
}

export function generateSafetyReportTs(program: Program, label: string): SafetyCaseReport {
  const missions = verifyMissionTs(program);
  const hw = verifyHardwareProgram(program);
  const known_risks = hw.items.filter((i) => i.severity === "warning").map((i) => i.message);
  const deployable =
    hw.compatible && missions.every((m) => m.achievable) && known_risks.length === 0;
  return {
    program: label,
    deployable,
    known_risks,
    safety_rules: program.robots.flatMap((r) =>
      r.safety ? [`${r.name}: max_speed and stop rules`] : [],
    ),
    kill_switch_validation: program.killSwitches.map((k) => k.name),
  };
}

export function evaluateFleetReadinessTs(
  program: Program,
  options: ReadinessOptions = {},
): FleetReadinessReport {
  const robot_reports = program.robots.map((robot) => {
    const report = evaluateReadinessTs(program, options);
    return { ...report, robots: [robot.name] };
  });
  let healthy = 0;
  let degraded = 0;
  let not_ready = 0;
  for (const report of robot_reports) {
    if (report.mission_ready && report.status !== "Degraded") healthy += 1;
    else if (report.status === "Degraded") degraded += 1;
    else not_ready += 1;
  }
  const fleet_score =
    robot_reports.length === 0
      ? 0
      : Math.round(
          robot_reports.reduce((sum, r) => sum + r.score.total, 0) / robot_reports.length,
        );
  return {
    fleet_score,
    healthy_robots: healthy,
    degraded_robots: degraded,
    not_ready_robots: not_ready,
    mission_capacity_percent: robot_reports.length
      ? Math.round((healthy * 100) / robot_reports.length)
      : 0,
    robot_reports,
  };
}

export function evaluateTwinReadinessTs(
  program: Program,
  tracePath?: string,
): TwinReadinessStatus {
  const physical = evaluateReadinessTs(program);
  const configuration_drift: string[] = [];
  const hasTwin = program.robots.some((r) => r.twin);
  if (!hasTwin && !tracePath) {
    configuration_drift.push("No twin declaration or trace export provided");
  }
  if (tracePath) {
    try {
      const trace = deserializeMissionTrace(readFileSync(tracePath, "utf-8"));
      if (trace.frames.length === 0) {
        configuration_drift.push("Twin trace contains no frames");
      }
    } catch {
      configuration_drift.push(`Could not load twin trace: ${tracePath}`);
    }
  }
  const capability_drift = physical.issues
    .filter((i) => i.factor === "Capabilities")
    .map((i) => i.message);
  const health_drift = physical.issues.filter((i) => i.factor === "Health").map((i) => i.message);
  const twin_ready =
    configuration_drift.length === 0 && capability_drift.length === 0 && health_drift.length === 0;
  const overall =
    physical.mission_ready && twin_ready
      ? "Ready"
      : physical.mission_ready || twin_ready
        ? "Degraded"
        : "NotReady";
  return {
    physical_ready: physical.mission_ready,
    twin_ready,
    configuration_drift,
    capability_drift,
    health_drift,
    overall,
  };
}

export function diagnoseTraceTs(tracePath: string): RootCauseReport {
  const trace = deserializeMissionTrace(readFileSync(tracePath, "utf-8"));
  const timeline = trace.frames.map((frame) => ({
    sim_time_ms: frame.simTimeMs,
    event: frame.event,
    detail: JSON.stringify(frame.payload ?? {}),
  }));
  const failure = trace.frames.find(
    (f) =>
      f.event.toLowerCase().includes("fail") ||
      (f.payload as { failed?: boolean } | undefined)?.failed === true,
  );
  const root_cause = failure
    ? `${failure.event}: ${JSON.stringify(failure.payload ?? {})}`
    : trace.frames.length === 0
      ? "Empty trace — no runtime events recorded"
      : "No explicit failure event; inspect timeline for anomalies";
  return {
    root_cause,
    contributing_factors: [],
    timeline,
    recommended_actions: ["Review mission trace timeline", "Re-run with --record enabled"],
  };
}

export function readinessDashboardFromReports(reports: ReadinessReport[]): ReadinessDashboard {
  const mission_ready_count = reports.filter((r) => r.mission_ready).length;
  const degraded_count = reports.filter((r) => r.status === "Degraded").length;
  const not_ready_count = reports.length - mission_ready_count - degraded_count;
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

export function formatMissionVerification(reports: MissionVerificationReport[]): string {
  return reports
    .map(
      (r) =>
        `Mission ${r.mission_name ?? "(unnamed)"} on ${r.robot ?? "?"}: ${
          r.achievable ? "ACHIEVABLE" : "BLOCKED"
        }\n` + r.issues.map((i) => `  - ${i}`).join("\n"),
    )
    .join("\n\n");
}

export function formatFailureAnalysis(report: FailureAnalysisReport): string {
  return report.impacts
    .map((i) => `[${i.severity}] ${i.component}: ${i.consequence}\n  Mitigation: ${i.mitigation}`)
    .join("\n\n");
}

export function formatFleetVerify(report: FleetVerifyReport): string {
  return report.findings.map((f) => `[${f.severity}] ${f.category}: ${f.message}`).join("\n");
}

export function formatAudit(report: SafetyAuditReport): string {
  return report.findings
    .map((f) => `[${f.severity}] ${f.category} @ ${f.line}:${f.column}: ${f.message}`)
    .join("\n");
}

export function formatRootCause(report: RootCauseReport): string {
  return `Root cause: ${report.root_cause}\n\nTimeline:\n${report.timeline
    .map((t) => `  T+${t.sim_time_ms}ms ${t.event}: ${t.detail}`)
    .join("\n")}`;
}

export function formatFleetReadiness(report: FleetReadinessReport): string {
  return `Fleet score: ${report.fleet_score}/100\nHealthy: ${report.healthy_robots}  Degraded: ${report.degraded_robots}  Not ready: ${report.not_ready_robots}`;
}

export function formatTwinReadiness(status: TwinReadinessStatus): string {
  return `Physical ready: ${status.physical_ready}\nTwin ready: ${status.twin_ready}\nOverall: ${status.overall}`;
}

export function runOperationalCommand(
  command: string,
  positional: string[],
  flags: Map<string, string | boolean>,
): { exitCode: number; output: string } {
  const json = flags.has("json");
  const target = typeof flags.get("target") === "string" ? (flags.get("target") as string) : undefined;
  const includeRuntime = flags.has("runtime") || flags.has("inject-health-faults");
  const injectHealthFaults = flags.has("inject-health-faults");
  const options: ReadinessOptions = { target, includeRuntime, injectHealthFaults };

  if (command === "fleet" && positional[0] === "readiness") {
    const file = positional[1];
    if (!file) throw new Error("Missing file path");
    const program = parseProgramSource(readFileSync(file, "utf-8"));
    const report = evaluateFleetReadinessTs(program, options);
    return {
      exitCode: report.not_ready_robots > 0 ? 1 : 0,
      output: json ? JSON.stringify(report, null, 2) : formatFleetReadiness(report),
    };
  }

  if (command === "verify" && positional[0] === "mission") {
    const file = positional[1];
    if (!file) throw new Error("Missing file path");
    const program = parseProgramSource(readFileSync(file, "utf-8"));
    const reports = verifyMissionTs(program, target);
    return {
      exitCode: reports.some((r) => !r.achievable) ? 1 : 0,
      output: json ? JSON.stringify(reports, null, 2) : formatMissionVerification(reports),
    };
  }

  if (command === "twin" && positional[0] === "readiness") {
    const file = positional[1];
    if (!file) throw new Error("Missing file path");
    const traceIdx = positional.indexOf("--trace");
    const tracePath = traceIdx >= 0 ? positional[traceIdx + 1] : undefined;
    const program = parseProgramSource(readFileSync(file, "utf-8"));
    const status = evaluateTwinReadinessTs(program, tracePath);
    return {
      exitCode: status.overall === "NotReady" ? 1 : 0,
      output: json ? JSON.stringify(status, null, 2) : formatTwinReadiness(status),
    };
  }

  const file = positional[0];
  if (!file) throw new Error("Missing file path");
  const source = readFileSync(file, "utf-8");
  const program = parseProgramSource(source);

  switch (command) {
    case "readiness": {
      if (flags.has("agent-json")) {
        const body = JSON.stringify({
          ok: true,
          mission_ready: evaluateReadinessSource(source, options).mission_ready,
          readiness: evaluateReadinessSource(source, options),
        });
        const parsed = JSON.parse(body) as { mission_ready?: boolean };
        return { exitCode: parsed.mission_ready ? 0 : 1, output: body };
      }
      const report = evaluateReadinessSource(source, options);
      const output = json
        ? JSON.stringify(report, null, 2)
        : `Mission Ready: ${report.mission_ready ? "YES" : "NO"}\nScore: ${report.score.total}/${report.score.maximum}`;
      return { exitCode: report.mission_ready ? 0 : 1, output };
    }
    case "analyze-failure": {
      const report = analyzeFailureTs(program);
      return {
        exitCode: 0,
        output: json ? JSON.stringify(report, null, 2) : formatFailureAnalysis(report),
      };
    }
    case "safety-report": {
      const report = generateSafetyReportTs(program, file);
      return {
        exitCode: report.deployable ? 0 : 1,
        output: json ? JSON.stringify(report, null, 2) : JSON.stringify(report, null, 2),
      };
    }
    case "diagnose": {
      const report = diagnoseTraceTs(file);
      return {
        exitCode: 0,
        output: json ? JSON.stringify(report, null, 2) : formatRootCause(report),
      };
    }
    case "audit": {
      const report = auditProgramTs(program, source);
      return {
        exitCode: report.critical_count > 0 ? 1 : 0,
        output: json ? JSON.stringify(report, null, 2) : formatAudit(report),
      };
    }
    case "verify-fleet": {
      const report = verifyFleetTs(program);
      return {
        exitCode: report.compatible ? 0 : 1,
        output: json ? JSON.stringify(report, null, 2) : formatFleetVerify(report),
      };
    }
    case "verify-approval": {
      const report = verifyApprovalsTs(program);
      return {
        exitCode: report.compatible ? 0 : 1,
        output: json ? JSON.stringify(report, null, 2) : JSON.stringify(report, null, 2),
      };
    }
    default:
      throw new Error(`Unsupported operational command: ${command}`);
  }
}
