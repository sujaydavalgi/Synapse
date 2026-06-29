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
  type CertificationProofSummary,
} from "./deploy-service.js";
import type { DeployArtifactBundle } from "./deploy-bundle.js";
import { remoteFetch } from "./http-fetch.js";

function certificationProofPayload(
  proof?: CertificationProofSummary,
): Record<string, unknown> | undefined {
  // Description:
  //     CertificationProofPayload.
  //
  // Inputs:
  //     proof?: CertificationProofSummary
  //         Caller-supplied proof?.
  //
  // Outputs:
  //     result: Record<string, unknown> | undefined
  //         Return value from `certificationProofPayload`.
  //
  // Example:

  //     const result = certificationProofPayload(proof?);

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
  // Description:
  //     DefaultAgentsPath.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: string
  //         Return value from `defaultAgentsPath`.
  //
  // Example:

  //     const result = defaultAgentsPath();

  return process.env.SPANDA_DEPLOY_AGENTS ?? ".spanda/deploy-agents.json";
}

export function emptyAgentRegistry(): DeployAgentRegistry {
  // Description:
  //     EmptyAgentRegistry.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: DeployAgentRegistry
  //         Return value from `emptyAgentRegistry`.
  //
  // Example:

  //     const result = emptyAgentRegistry();

  return { agents: [] };
}

export function loadAgentRegistry(text: string | null): DeployAgentRegistry {
  // Description:
  //     LoadAgentRegistry.
  //
  // Inputs:
  //     text: string | null
  //         Caller-supplied text.
  //
  // Outputs:
  //     result: DeployAgentRegistry
  //         Return value from `loadAgentRegistry`.
  //
  // Example:

  //     const result = loadAgentRegistry(text);

  if (!text) return emptyAgentRegistry();
  try {
    const parsed = JSON.parse(text) as DeployAgentRegistry;
    return { agents: parsed.agents ?? [] };
  } catch {
    return emptyAgentRegistry();
  }
}

export function serializeAgentRegistry(registry: DeployAgentRegistry): string {
  // Description:
  //     SerializeAgentRegistry.
  //
  // Inputs:
  //     registry: DeployAgentRegistry
  //         Caller-supplied registry.
  //
  // Outputs:
  //     result: string
  //         Return value from `serializeAgentRegistry`.
  //
  // Example:

  //     const result = serializeAgentRegistry(registry);

  return JSON.stringify(registry, null, 2);
}

export function readAgentRegistryFromDisk(path = defaultAgentsPath()): DeployAgentRegistry {
  // Description:
  //     ReadAgentRegistryFromDisk.
  //
  // Inputs:
  //     path = defaultAgentsPath(): input value
  //         Caller-supplied path = defaultAgentsPath().
  //
  // Outputs:
  //     result: DeployAgentRegistry
  //         Return value from `readAgentRegistryFromDisk`.
  //
  // Example:

  //     const result = readAgentRegistryFromDisk(path = defaultAgentsPath());

  if (!existsSync(path)) return emptyAgentRegistry();
  return loadAgentRegistry(readFileSync(path, "utf-8"));
}

export function writeAgentRegistryToDisk(registry: DeployAgentRegistry, path = defaultAgentsPath()): void {
  // Description:
  //     WriteAgentRegistryToDisk.
  //
  // Inputs:
  //     registry: DeployAgentRegistry
  //         Caller-supplied registry.
  //     path = defaultAgentsPath(): input value
  //         Caller-supplied path = defaultAgentsPath().
  //
  // Outputs:
  //     None.
  //
  // Example:

  //     const result = writeAgentRegistryToDisk(registry, path = defaultAgentsPath());

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
  // Description:
  //     RegisterAgent.
  //
  // Inputs:
  //     registry: DeployAgentRegistry
  //         Caller-supplied registry.
  //     target: string
  //         Caller-supplied target.
  //     url: string
  //         Caller-supplied url.
  //     token?: string
  //         Caller-supplied token?.
  //
  // Outputs:
  //     result: DeployAgentRegistry
  //         Return value from `registerAgent`.
  //
  // Example:

  //     const result = registerAgent(registry, target, url, token?);

  if (!url.startsWith("http://") && !url.startsWith("https://")) {
    throw new Error(`deploy agent URL must start with http:// or https:// (got ${url})`);
  }
  const agents = registry.agents.filter((entry) => entry.target !== target);
  agents.push({ target, url, token });
  agents.sort((a, b) => a.target.localeCompare(b.target));
  return { agents };
}

export function lookupAgent(registry: DeployAgentRegistry, target: string): DeployAgentEntry | undefined {
  // Description:
  //     LookupAgent.
  //
  // Inputs:
  //     registry: DeployAgentRegistry
  //         Caller-supplied registry.
  //     target: string
  //         Caller-supplied target.
  //
  // Outputs:
  //     result: DeployAgentEntry | undefined
  //         Return value from `lookupAgent`.
  //
  // Example:

  //     const result = lookupAgent(registry, target);

  return registry.agents.find((entry) => entry.target === target);
}

async function agentFetch(
  entry: DeployAgentEntry,
  method: string,
  path: string,
  body?: string,
): Promise<Response> {
  // Description:
  //     AgentFetch.
  //
  // Inputs:
  //     entry: DeployAgentEntry
  //         Caller-supplied entry.
  //     method: string
  //         Caller-supplied method.
  //     path: string
  //         Caller-supplied path.
  //     body?: string
  //         Caller-supplied body?.
  //
  // Outputs:
  //     result: Promise<Response>
  //         Return value from `agentFetch`.
  //
  // Example:

  //     const result = agentFetch(entry, method, path, body?);

  const base = entry.url.replace(/\/$/, "");
  const headers: Record<string, string> = { Accept: "application/json" };
  if (body) headers["Content-Type"] = "application/json";
  if (entry.token) headers.Authorization = `Bearer ${entry.token}`;
  return remoteFetch(`${base}${path}`, { method, headers, body });
}

export async function agentHealth(entry: DeployAgentEntry): Promise<boolean> {
  // Description:
  //     AgentHealth.
  //
  // Inputs:
  //     entry: DeployAgentEntry
  //         Caller-supplied entry.
  //
  // Outputs:
  //     result: Promise<boolean>
  //         Return value from `agentHealth`.
  //
  // Example:

  //     const result = agentHealth(entry);

  const response = await agentFetch(entry, "GET", "/v1/health");
  if (!response.ok) return false;
  const body = (await response.json()) as { ok?: boolean };
  return body.ok === true;
}

export async function agentReadiness(
  entry: DeployAgentEntry,
  runtime = false,
  injectHealthFaults = false,
): Promise<{
  // Description:
  //     AgentReadiness.
  //
  // Inputs:
  //     entry: DeployAgentEntry
  //         Caller-supplied entry.
  //     runtime = false: input value
  //         Caller-supplied runtime = false.
  //     injectHealthFaults = false: input value
  //         Caller-supplied injectHealthFaults = false.
  //
  // Outputs:
  //     result: Promise<
  //         Return value from `agentReadiness`.
  //
  // Example:

 // const result = agentReadiness(entry, runtime = false, injectHealthFaults = false);
 ok: boolean; mission_ready?: boolean; readiness?: unknown }> {
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
  // Description:
  //     AgentUploadProgram.
  //
  // Inputs:
  //     entry: DeployAgentEntry
  //         Caller-supplied entry.
  //     program: string
  //         Caller-supplied program.
  //
  // Outputs:
  //     result: Promise<void>
  //         Return value from `agentUploadProgram`.
  //
  // Example:

  //     const result = agentUploadProgram(entry, program);

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
  // Description:
  //     AgentStatus.
  //
  // Inputs:
  //     entry: DeployAgentEntry
  //         Caller-supplied entry.
  //
  // Outputs:
  //     result: Promise<AgentStatusResponse>
  //         Return value from `agentStatus`.
  //
  // Example:

  //     const result = agentStatus(entry);

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
  // Description:
  //     ExecuteRemoteRollout.
  //
  // Inputs:
  //     plan: DeployPlan
  //         Caller-supplied plan.
  //     options: RolloutOptions
  //         Caller-supplied options.
  //     registry: DeployAgentRegistry
  //         Caller-supplied registry.
  //     bundle: DeployArtifactBundle
  //         Caller-supplied bundle.
  //
  // Outputs:
  //     result: Promise<RolloutResult>
  //         Return value from `executeRemoteRollout`.
  //
  // Example:

  //     const result = executeRemoteRollout(plan, options, registry, bundle);

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
  // Description:
  //     ExecuteRemoteRollback.
  //
  // Inputs:
  //     plan: DeployPlan
  //         Caller-supplied plan.
  //     registry: DeployAgentRegistry
  //         Caller-supplied registry.
  //
  // Outputs:
  //     result: Promise<RolloutResult>
  //         Return value from `executeRemoteRollback`.
  //
  // Example:

  //     const result = executeRemoteRollback(plan, registry);

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
  // Description:
  //     ApplyRemoteRolloutToState.
  //
  // Inputs:
  //     state: import("./deploy-service.js").DeployState
  //         Caller-supplied state.
  //     result: RolloutResult
  //         Caller-supplied result.
  //
  // Outputs:
  //     None.
  //
  // Example:

  //     const result = applyRemoteRolloutToState(state, result);

  applyRollout(state, result);
}
