/**
 * On-device fleet peer relay HTTP server (Node.js).
 * @module
 */

import { createServer, type IncomingMessage, type ServerResponse } from "node:http";
import { createServer as createHttpsServer } from "node:https";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import type { ReadinessEvaluator } from "./deploy-agent.js";

const noopReadinessEvaluator: ReadinessEvaluator = () => ({
  status: "Unknown",
  mission_ready: false,
  score: { total: 0, maximum: 100, factors: [] },
  issues: [
    {
      factor: "Readiness",
      severity: "Critical",
      message: "Readiness evaluator not configured",
    },
  ],
  robots: [],
});

export type { ReadinessEvaluator };

export type FleetAgentState = {
  robotName: string;
  token?: string;
  program?: string;
  lastPeerMessages?: string[];
};

export function defaultFleetAgentStatePath(): string {
  return process.env.SPANDA_FLEET_AGENT_STATE ?? ".spanda/fleet-agent-state.json";
}

export function fleetAgentStatePathFor(robotName: string): string {
  const safeName = robotName.replace(/[/\\]/g, "_");
  return `.spanda/fleet-agent-state/${safeName}.json`;
}

function clearFleetAgentOnIdentityChange(state: FleetAgentState, newRobotName: string): void {
  if (state.robotName && state.robotName !== newRobotName) {
    state.lastPeerMessages = [];
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

export function emptyFleetAgentState(): FleetAgentState {
  return { robotName: "", lastPeerMessages: [] };
}

export function loadFleetAgentState(text: string | null): FleetAgentState {
  if (!text) return emptyFleetAgentState();
  try {
    const parsed = JSON.parse(text) as FleetAgentState & {
      robot_name?: string;
      last_peer_messages?: string[];
    };
    return {
      robotName: parsed.robotName ?? parsed.robot_name ?? "",
      token: parsed.token,
      program: parsed.program,
      lastPeerMessages: parsed.lastPeerMessages ?? parsed.last_peer_messages ?? [],
    };
  } catch {
    return emptyFleetAgentState();
  }
}

export function readFleetAgentStateFromDisk(path = defaultFleetAgentStatePath()): FleetAgentState {
  if (!existsSync(path)) return emptyFleetAgentState();
  return loadFleetAgentState(readFileSync(path, "utf-8"));
}

export function writeFleetAgentStateToDisk(state: FleetAgentState, path = defaultFleetAgentStatePath()): void {
  const abs = resolve(path);
  mkdirSync(dirname(abs), { recursive: true });
  writeFileSync(abs, JSON.stringify(state, null, 2));
}

function unauthorized(req: IncomingMessage, state: FleetAgentState): boolean {
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
  state: FleetAgentState,
  statePath: string,
  evaluateReadiness: ReadinessEvaluator = noopReadinessEvaluator,
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
    res.end(JSON.stringify({ ok: true, agent: "spanda-fleet-agent", version: "0.1.0" }));
    return;
  }

  if (req.method === "GET" && path === "/v1/readiness") {
    if (!state.program) {
      res.writeHead(503, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ ok: false, error: "no program deployed on fleet agent" }));
      return;
    }
    const includeRuntime = parsedUrl.searchParams.get("runtime") === "true";
    const injectHealthFaults = parsedUrl.searchParams.get("inject_health_faults") === "true";
    const report = evaluateReadiness(state.program, {
      target: state.robotName || undefined,
      includeRuntime,
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
    writeFleetAgentStateToDisk(state, statePath);
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(JSON.stringify({ ok: true }));
    return;
  }

  if (req.method === "GET" && path === "/v1/status") {
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(
      JSON.stringify({
        ok: true,
        robot_name: state.robotName,
        last_peer_messages: state.lastPeerMessages ?? [],
        has_program: Boolean(state.program),
        healthy: true,
      }),
    );
    return;
  }

  if (req.method === "POST" && path === "/v1/peer") {
    const body = await readBody(req);
    const payload = JSON.parse(body) as {
      from_robot?: string;
      to_robot?: string;
      topic?: string;
      step?: string;
    };
    if (state.robotName && payload.to_robot && payload.to_robot !== state.robotName) {
      res.writeHead(400, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ ok: false, error: "robot mismatch" }));
      return;
    }
    if (!state.robotName) {
      res.writeHead(500, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ ok: false, error: "fleet agent missing robot identity" }));
      return;
    }
    const message = `${payload.from_robot ?? ""}->${payload.to_robot ?? ""}:${payload.topic ?? ""}=${payload.step ?? ""}`;
    state.lastPeerMessages = [...(state.lastPeerMessages ?? []), message];
    writeFleetAgentStateToDisk(state, statePath);
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(
      JSON.stringify({
        ok: true,
        to_robot: payload.to_robot,
        topic: payload.topic,
        step: payload.step,
      }),
    );
    return;
  }

  res.writeHead(404, { "Content-Type": "application/json" });
  res.end(JSON.stringify({ ok: false, error: "not found" }));
}

export function startFleetAgentServer(options: {
  bind: string;
  robotName: string;
  token?: string;
  statePath?: string;
  tlsCert?: string;
  tlsKey?: string;
  evaluateReadiness?: ReadinessEvaluator;
}): ReturnType<typeof createServer> {
  const statePath = options.statePath ?? fleetAgentStatePathFor(options.robotName);
  const state = readFleetAgentStateFromDisk(statePath);
  clearFleetAgentOnIdentityChange(state, options.robotName);
  state.robotName = options.robotName;
  if (options.token) state.token = options.token;
  writeFleetAgentStateToDisk(state, statePath);
  const evaluateReadiness = options.evaluateReadiness ?? noopReadinessEvaluator;
  const withRequestLock = createRequestLock();
  const requestHandler = (req: IncomingMessage, res: ServerResponse) => {
    void withRequestLock(() => handleRequest(req, res, state, statePath, evaluateReadiness));
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
    console.error(
      `Spanda fleet agent listening on ${scheme}://${options.bind} for robot ${state.robotName}`,
    );
  });
  return server;
}
