/**
 * On-device Spanda deploy agent HTTP server (Node.js).
 * @module
 */

import { createServer, type IncomingMessage, type ServerResponse } from "node:http";
import { createServer as createHttpsServer } from "node:https";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { evaluateReadinessSource } from "./readiness.js";

export type AgentState = {
  target: string;
  currentVersion: string;
  previousVersion?: string;
  token?: string;
  program?: string;
  programHash?: string;
  requireHash?: boolean;
  requireSignature?: boolean;
  requireCertify?: boolean;
  trustedPublicKey?: string;
};

export function defaultAgentStatePath(): string {
  return process.env.SPANDA_AGENT_STATE ?? ".spanda/agent-state.json";
}

export function agentStatePathFor(target: string): string {
  const safeTarget = target.replace(/[/\\@:]/g, "_");
  return `.spanda/agent-state/${safeTarget}.json`;
}

function clearAgentDeploymentOnIdentityChange(state: AgentState, newTarget: string): void {
  if (state.target && state.target !== newTarget) {
    state.currentVersion = "0.0.0";
    delete state.previousVersion;
    delete state.program;
    delete state.programHash;
    delete state.requireHash;
    delete state.requireSignature;
    delete state.requireCertify;
    delete state.trustedPublicKey;
    delete state.token;
  }
}

function createRequestLock(): <T>(fn: () => Promise<T>) => Promise<T> {
  let chain: Promise<void> = Promise.resolve();
  return <T>(fn: () => Promise<T>) => {
    const run = chain.then(fn, fn);
    chain = run.then(
      () => undefined,
      () => undefined,
    );
    return run;
  };
}

export function emptyAgentState(): AgentState {
  return { target: "", currentVersion: "0.0.0" };
}

export function loadAgentState(text: string | null): AgentState {
  if (!text) return emptyAgentState();
  try {
    return JSON.parse(text) as AgentState;
  } catch {
    return emptyAgentState();
  }
}

export function readAgentStateFromDisk(path = defaultAgentStatePath()): AgentState {
  if (!existsSync(path)) return emptyAgentState();
  return loadAgentState(readFileSync(path, "utf-8"));
}

export function writeAgentStateToDisk(state: AgentState, path = defaultAgentStatePath()): void {
  const abs = resolve(path);
  mkdirSync(dirname(abs), { recursive: true });
  writeFileSync(abs, JSON.stringify(state, null, 2));
}

function unauthorized(req: IncomingMessage, state: AgentState): boolean {
  const header = req.headers.authorization;
  if (!state.token) return false;
  return header !== `Bearer ${state.token}`;
}

function readBody(req: IncomingMessage): Promise<string> {
  return new Promise((resolveBody, reject) => {
    const chunks: Buffer[] = [];
    req.on("data", (chunk) => chunks.push(Buffer.from(chunk)));
    req.on("end", () => resolveBody(Buffer.concat(chunks).toString("utf-8")));
    req.on("error", reject);
  });
}

async function handleRequest(
  req: IncomingMessage,
  res: ServerResponse,
  state: AgentState,
  statePath: string,
): Promise<void> {
  if (unauthorized(req, state)) {
    res.writeHead(401, { "Content-Type": "application/json" });
    res.end(JSON.stringify({ ok: false, error: "unauthorized" }));
    return;
  }

  const url = req.url ?? "/";
  const parsedUrl = new URL(url, "http://localhost");
  const path = parsedUrl.pathname;

  if (req.method === "GET" && path === "/v1/health") {
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(JSON.stringify({ ok: true, agent: "spanda-deploy-agent", version: "0.1.0" }));
    return;
  }

  if (req.method === "GET" && path === "/v1/readiness") {
    if (!state.program) {
      res.writeHead(503, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ ok: false, error: "no program deployed on agent" }));
      return;
    }
    const includeRuntime = parsedUrl.searchParams.get("runtime") === "true";
    const injectHealthFaults = parsedUrl.searchParams.get("inject_health_faults") === "true";
    const report = evaluateReadinessSource(state.program, {
      target: state.target || undefined,
      includeRuntime: includeRuntime,
      injectHealthFaults,
    });
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(JSON.stringify({ ok: true, mission_ready: report.mission_ready, readiness: report }));
    return;
  }

  if (req.method === "POST" && path === "/v1/program") {
    const body = await readBody(req);
    const payload = JSON.parse(body) as { program?: string };
    if (!payload.program) {
      res.writeHead(400, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ ok: false, error: "program field required" }));
      return;
    }
    state.program = payload.program;
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(JSON.stringify({ ok: true }));
    return;
  }

  if (req.method === "GET" && path === "/v1/status") {
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(
      JSON.stringify({
        ok: true,
        target: state.target,
        current_version: state.currentVersion,
        previous_version: state.previousVersion ?? null,
        program: state.program ?? null,
        program_hash: state.programHash ?? null,
        healthy: true,
      }),
    );
    return;
  }

  if (req.method === "POST" && path === "/v1/rollout") {
    const body = await readBody(req);
    const payload = JSON.parse(body) as {
      target?: string;
      version?: string;
      program?: string;
      program_hash?: string;
      assignments?: Array<{ robot_name: string; hardware: string }>;
      certifications?: string[];
      certification_proof?: {
        passed?: boolean;
        passed_strict?: boolean;
        summary?: string;
        error_count?: number;
        warning_count?: number;
      };
      artifact_signature?: string;
      artifact_public_key?: string;
    };
    if (state.target && payload.target && payload.target !== state.target) {
      res.writeHead(400, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ ok: false, error: "target mismatch" }));
      return;
    }
    if (state.requireHash && !payload.program_hash) {
      res.writeHead(400, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ ok: false, error: "program_hash required" }));
      return;
    }
    if (state.requireSignature) {
      if (!state.trustedPublicKey || !payload.artifact_signature) {
        res.writeHead(400, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ ok: false, error: "invalid artifact signature" }));
        return;
      }
      const { verifyDeployBundle } = await import("./deploy-bundle.js");
      const ok = await verifyDeployBundle(
        {
          version: payload.version ?? state.currentVersion,
          program: payload.program ?? "",
          programHash: payload.program_hash,
          assignments: (payload.assignments ?? []).map((a) => ({
            robotName: a.robot_name,
            hardware: a.hardware,
          })),
          certifications: payload.certifications ?? [],
          signature: payload.artifact_signature,
          publicKey: payload.artifact_public_key,
        },
        state.trustedPublicKey,
      );
      if (!ok) {
        res.writeHead(400, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ ok: false, error: "invalid artifact signature" }));
        return;
      }
    }
    if (state.requireCertify) {
      const proofOk = payload.certification_proof?.passed_strict === true;
      if (!proofOk) {
        res.writeHead(400, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ ok: false, error: "strict certification proof required" }));
        return;
      }
    }
    if (state.currentVersion) state.previousVersion = state.currentVersion;
    state.currentVersion = payload.version ?? state.currentVersion;
    state.program = payload.program;
    state.programHash = payload.program_hash;
    writeAgentStateToDisk(state, statePath);
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(
      JSON.stringify({
        ok: true,
        target: state.target,
        version: state.currentVersion,
        previous_version: state.previousVersion ?? null,
      }),
    );
    return;
  }

  if (req.method === "POST" && url === "/v1/rollback") {
    if (!state.previousVersion) {
      res.writeHead(409, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ ok: false, error: "no previous version" }));
      return;
    }
    const current = state.currentVersion;
    state.currentVersion = state.previousVersion;
    state.previousVersion = current;
    writeAgentStateToDisk(state, statePath);
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(
      JSON.stringify({
        ok: true,
        target: state.target,
        version: state.currentVersion,
        previous_version: state.previousVersion ?? null,
      }),
    );
    return;
  }

  res.writeHead(404, { "Content-Type": "application/json" });
  res.end(JSON.stringify({ ok: false, error: "not found" }));
}

export function createDeployAgentServer(
  state: AgentState,
  statePath = state.target ? agentStatePathFor(state.target) : defaultAgentStatePath(),
): ReturnType<typeof createServer> {
  // Serve the deploy agent protocol over HTTP/1.1.
  const withRequestLock = createRequestLock();
  return createServer((req, res) => {
    void withRequestLock(() => handleRequest(req, res, state, statePath));
  });
}

export function startDeployAgentServer(options: {
  bind: string;
  target: string;
  token?: string;
  statePath?: string;
  tlsCert?: string;
  tlsKey?: string;
  requireHash?: boolean;
  requireSignature?: boolean;
  requireCertify?: boolean;
  trustedPublicKey?: string;
}): ReturnType<typeof createServer> {
  const statePath = options.statePath ?? agentStatePathFor(options.target);
  const state = readAgentStateFromDisk(statePath);
  clearAgentDeploymentOnIdentityChange(state, options.target);
  state.target = options.target;
  if (options.token) state.token = options.token;
  if (options.requireHash) state.requireHash = true;
  if (options.requireSignature) state.requireSignature = true;
  if (options.requireCertify) state.requireCertify = true;
  if (options.trustedPublicKey) state.trustedPublicKey = options.trustedPublicKey;
  writeAgentStateToDisk(state, statePath);
  const withRequestLock = createRequestLock();
  const requestHandler = (req: IncomingMessage, res: ServerResponse) => {
    void withRequestLock(() => handleRequest(req, res, state, statePath));
  };
  const scheme = options.tlsCert && options.tlsKey ? "https" : "http";
  const server =
    options.tlsCert && options.tlsKey
      ? createHttpsServer(
          {
            cert: readFileSync(options.tlsCert),
            key: readFileSync(options.tlsKey),
          },
          requestHandler,
        )
      : createServer(requestHandler);
  server.listen(options.bind, () => {
    console.error(`Spanda deploy agent listening on ${scheme}://${options.bind} for target ${state.target}`);
  });
  return server;
}
