/**
 * On-device Spanda deploy agent HTTP server (Node.js).
 * @module
 */

import { createServer, type IncomingMessage, type ServerResponse } from "node:http";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";

export type AgentState = {
  target: string;
  currentVersion: string;
  previousVersion?: string;
  token?: string;
};

export function defaultAgentStatePath(): string {
  return process.env.SPANDA_AGENT_STATE ?? ".spanda/agent-state.json";
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

export function createDeployAgentServer(
  state: AgentState,
  statePath = defaultAgentStatePath(),
): ReturnType<typeof createServer> {
  // Serve the deploy agent protocol over HTTP/1.1.
  return createServer(async (req, res) => {
    if (unauthorized(req, state)) {
      res.writeHead(401, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ ok: false, error: "unauthorized" }));
      return;
    }

    const url = req.url ?? "/";
    if (req.method === "GET" && url === "/v1/health") {
      res.writeHead(200, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ ok: true, agent: "spanda-deploy-agent", version: "0.1.0" }));
      return;
    }

    if (req.method === "GET" && url === "/v1/status") {
      res.writeHead(200, { "Content-Type": "application/json" });
      res.end(
        JSON.stringify({
          ok: true,
          target: state.target,
          current_version: state.currentVersion,
          previous_version: state.previousVersion ?? null,
          healthy: true,
        }),
      );
      return;
    }

    if (req.method === "POST" && url === "/v1/rollout") {
      const body = await readBody(req);
      const payload = JSON.parse(body) as { target?: string; version?: string };
      if (state.target && payload.target && payload.target !== state.target) {
        res.writeHead(400, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ ok: false, error: "target mismatch" }));
        return;
      }
      if (payload.target) state.target = payload.target;
      if (state.currentVersion) state.previousVersion = state.currentVersion;
      state.currentVersion = payload.version ?? state.currentVersion;
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
  });
}

export function startDeployAgentServer(options: {
  bind: string;
  target: string;
  token?: string;
  statePath?: string;
}): ReturnType<typeof createServer> {
  const statePath = options.statePath ?? defaultAgentStatePath();
  const state = readAgentStateFromDisk(statePath);
  if (!state.target) state.target = options.target;
  if (options.token) state.token = options.token;
  writeAgentStateToDisk(state, statePath);
  const server = createDeployAgentServer(state, statePath);
  server.listen(options.bind, () => {
    console.error(`Spanda deploy agent listening on ${options.bind} for target ${state.target}`);
  });
  return server;
}
