/**
 * Remote OTA deploy agent client and registry.
 * @module
 */

import { readFileSync, writeFileSync, mkdirSync, existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import {
  applyRollout,
  deployTargetKey,
  planRollout,
  type DeployPlan,
  type RolloutOptions,
  type RolloutResult,
  type RolloutStep,
} from "./deploy-service.js";
import type { DeployArtifactBundle } from "./deploy-bundle.js";
import type { CertificationProofSummary } from "./certify-prover.js";
import { remoteFetch } from "./http-fetch.js";

function certificationProofPayload(
  proof?: CertificationProofSummary,
): Record<string, unknown> | undefined {
  if (!proof) return undefined;
  return {
    passed: proof.passed,
    passed_strict: proof.passedStrict,
    summary: proof.summary,
    error_count: proof.errorCount,
    warning_count: proof.warningCount,
  };
}

export type DeployAgentEntry = {
  target: string;
  url: string;
  token?: string;
};

export type DeployAgentRegistry = {
  agents: DeployAgentEntry[];
};

export type AgentStatusResponse = {
  ok: boolean;
  target: string;
  currentVersion: string;
  previousVersion?: string;
  healthy: boolean;
};

export function defaultAgentsPath(): string {
  return process.env.SPANDA_DEPLOY_AGENTS ?? ".spanda/deploy-agents.json";
}

export function emptyAgentRegistry(): DeployAgentRegistry {
  return { agents: [] };
}

export function loadAgentRegistry(text: string | null): DeployAgentRegistry {
  if (!text) return emptyAgentRegistry();
  try {
    const parsed = JSON.parse(text) as DeployAgentRegistry;
    return { agents: parsed.agents ?? [] };
  } catch {
    return emptyAgentRegistry();
  }
}

export function serializeAgentRegistry(registry: DeployAgentRegistry): string {
  return JSON.stringify(registry, null, 2);
}

export function readAgentRegistryFromDisk(path = defaultAgentsPath()): DeployAgentRegistry {
  if (!existsSync(path)) return emptyAgentRegistry();
  return loadAgentRegistry(readFileSync(path, "utf-8"));
}

export function writeAgentRegistryToDisk(registry: DeployAgentRegistry, path = defaultAgentsPath()): void {
  const abs = resolve(path);
  mkdirSync(dirname(abs), { recursive: true });
  writeFileSync(abs, serializeAgentRegistry(registry));
}

export function registerAgent(
  registry: DeployAgentRegistry,
  target: string,
  url: string,
  token?: string,
): DeployAgentRegistry {
  if (!url.startsWith("http://") && !url.startsWith("https://")) {
    throw new Error(`deploy agent URL must start with http:// or https:// (got ${url})`);
  }
  const agents = registry.agents.filter((entry) => entry.target !== target);
  agents.push({ target, url, token });
  agents.sort((a, b) => a.target.localeCompare(b.target));
  return { agents };
}

export function lookupAgent(registry: DeployAgentRegistry, target: string): DeployAgentEntry | undefined {
  return registry.agents.find((entry) => entry.target === target);
}

async function agentFetch(
  entry: DeployAgentEntry,
  method: string,
  path: string,
  body?: string,
): Promise<Response> {
  const base = entry.url.replace(/\/$/, "");
  const headers: Record<string, string> = { Accept: "application/json" };
  if (body) headers["Content-Type"] = "application/json";
  if (entry.token) headers.Authorization = `Bearer ${entry.token}`;
  return remoteFetch(`${base}${path}`, { method, headers, body });
}

export async function agentHealth(entry: DeployAgentEntry): Promise<boolean> {
  const response = await agentFetch(entry, "GET", "/v1/health");
  if (!response.ok) return false;
  const body = (await response.json()) as { ok?: boolean };
  return body.ok === true;
}

export async function agentReadiness(
  entry: DeployAgentEntry,
  runtime = false,
  injectHealthFaults = false,
): Promise<{ ok: boolean; mission_ready?: boolean; readiness?: unknown }> {
  const query = new URLSearchParams();
  if (runtime) query.set("runtime", "true");
  if (injectHealthFaults) query.set("inject_health_faults", "true");
  const suffix = query.toString() ? `?${query.toString()}` : "";
  const response = await agentFetch(entry, "GET", `/v1/readiness${suffix}`);
  if (!response.ok) {
    throw new Error(`agent readiness HTTP ${response.status}`);
  }
  return (await response.json()) as { ok: boolean; mission_ready?: boolean; readiness?: unknown };
}

export async function agentUploadProgram(entry: DeployAgentEntry, program: string): Promise<void> {
  const response = await agentFetch(entry, "POST", "/v1/program", JSON.stringify({ program }));
  if (!response.ok) {
    throw new Error(`agent program upload HTTP ${response.status}`);
  }
  const body = (await response.json()) as { ok?: boolean };
  if (!body.ok) {
    throw new Error("agent program upload failed");
  }
}

export async function agentStatus(entry: DeployAgentEntry): Promise<AgentStatusResponse> {
  const response = await agentFetch(entry, "GET", "/v1/status");
  if (!response.ok) {
    throw new Error(`agent status HTTP ${response.status}`);
  }
  const body = (await response.json()) as AgentStatusResponse & { current_version?: string; previous_version?: string };
  return {
    ok: body.ok,
    target: body.target,
    currentVersion: body.currentVersion ?? body.current_version ?? "",
    previousVersion: body.previousVersion ?? body.previous_version,
    healthy: body.healthy,
  };
}

export async function executeRemoteRollout(
  plan: DeployPlan,
  options: RolloutOptions,
  registry: DeployAgentRegistry,
  bundle: DeployArtifactBundle,
): Promise<RolloutResult> {
  const local = planRollout(plan, options);
  if (options.dryRun) return local;

  const steps: RolloutStep[] = [];
  let success = true;
  for (const step of local.steps) {
    if (step.status === "skipped") {
      steps.push(step);
      continue;
    }
    const key = deployTargetKey(step.robotName, step.hardware);
    const agent = lookupAgent(registry, key);
    if (!agent) {
      steps.push({ ...step, status: "failed" });
      success = false;
      continue;
    }
    try {
      const response = await agentFetch(
        agent,
        "POST",
        "/v1/rollout",
        JSON.stringify({
          target: key,
          version: bundle.version,
          program: bundle.program,
          program_hash: bundle.programHash,
          assignments: bundle.assignments.map((assignment) => ({
            robot_name: assignment.robotName,
            hardware: assignment.hardware,
          })),
          certifications: bundle.certifications,
          certification_proof: certificationProofPayload(plan.certificationProof),
          artifact_signature: bundle.signature,
          artifact_public_key: bundle.publicKey,
        }),
      );
      const body = (await response.json()) as { ok?: boolean; version?: string };
      if (response.ok && body.ok) {
        steps.push({ ...step, status: "deployed" });
      } else {
        steps.push({ ...step, status: "failed", version: body.version ?? step.version });
        success = false;
      }
    } catch {
      steps.push({ ...step, status: "failed" });
      success = false;
    }
  }
  return {
    strategy: options.strategy,
    version: options.version,
    dryRun: false,
    steps,
    success,
  };
}

export async function executeRemoteRollback(
  plan: DeployPlan,
  registry: DeployAgentRegistry,
): Promise<RolloutResult> {
  const steps: RolloutStep[] = [];
  let success = false;
  for (const assignment of plan.assignments) {
    const key = deployTargetKey(assignment.robotName, assignment.hardware);
    const agent = lookupAgent(registry, key);
    const base: RolloutStep = {
      robotName: assignment.robotName,
      hardware: assignment.hardware,
      status: "skipped",
      version: "unknown",
      phasePercent: null,
    };
    if (!agent) {
      steps.push(base);
      continue;
    }
    try {
      const response = await agentFetch(agent, "POST", "/v1/rollback", JSON.stringify({ target: key }));
      const body = (await response.json()) as { ok?: boolean; version?: string };
      if (response.ok && body.ok) {
        success = true;
        steps.push({ ...base, status: "rolled_back", version: body.version ?? "unknown" });
      } else {
        steps.push({ ...base, status: "failed", version: body.version ?? "unknown" });
      }
    } catch {
      steps.push({ ...base, status: "failed" });
    }
  }
  return {
    strategy: "all",
    version: "rollback",
    dryRun: false,
    steps,
    success,
  };
}

export function applyRemoteRolloutToState(
  state: import("./deploy-service.js").DeployState,
  result: RolloutResult,
): void {
  applyRollout(state, result);
}
