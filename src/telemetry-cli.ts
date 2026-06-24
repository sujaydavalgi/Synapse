/**
 * Pure TypeScript telemetry store query CLI (JSONL fallback).
 * @module
 */

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import http from "node:http";
import { dirname } from "node:path";
import {
  loadMissionTrace,
  parseReplayOffset,
  playbackFrames,
  replayMissionDeterministic,
  traceFramesFrom,
} from "./replay.js";
import {
  envPersistEnabled,
  listTelemetrySessions,
  readAllEvents,
  readHeartbeatIndexForStore,
  resolveHeartbeatIndexPath,
  resolveStorePath,
  usesSqliteBackend,
} from "./telemetry-store.js";
import { envBackendSqlite, sqliteBackendAvailable } from "./telemetry-sqlite.js";
import { renderOtlpJson } from "./telemetry-otlp.js";
import {
  envOtlpEndpoint,
  envOtlpToken,
  envPushIntervalMs,
  runOtlpPushLoop,
} from "./telemetry-push.js";

export type TelemetryEvent =
  | {
      kind: "device";
      device_id: string;
      metric: string;
      value: unknown;
      timestamp_ms: number;
      robot_id?: string;
    }
  | {
      kind: "sensor";
      sensor_id: string;
      sensor_type: string;
      value: unknown;
      timestamp_ms: number;
      robot_id?: string;
    }
  | {
      kind: "heartbeat";
      task_name: string;
      timestamp_ms: number;
      robot_id?: string;
    }
  | {
      kind: "device_heartbeat";
      device_id: string;
      timestamp_ms: number;
      robot_id?: string;
      protocol?: string;
    }
  | { kind: "health"; target: string; status: string; timestamp_ms: number }
  | {
      kind: "session";
      session_id: string;
      phase: string;
      source?: string;
      mission_trace_path?: string;
      timestamp_ms: number;
    }
  | {
      kind: "runtime_metrics";
      session_id: string;
      metrics: unknown;
      timestamp_ms: number;
    };

type HeartbeatIndex = {
  tasks: Record<string, number>;
  devices?: Record<string, number>;
};

type TelemetryQuery = {
  deviceId?: string;
  sensorId?: string;
  taskName?: string;
  kind?: string;
  sessionId?: string;
  sinceMs?: number;
  limit?: number;
};

type TelemetryStats = {
  total_events: number;
  device_events: number;
  sensor_events: number;
  heartbeat_events: number;
  device_heartbeat_events: number;
  health_events: number;
  session_events: number;
  runtime_metrics_events: number;
  tracked_tasks: number;
  tracked_devices: number;
};

function readHeartbeatIndex(): HeartbeatIndex {
  return readHeartbeatIndexForStore();
}

function sessionTimeWindow(events: TelemetryEvent[], sessionId: string): [number, number] | null {
  let startMs: number | undefined;
  let endMs: number | undefined;
  for (const event of events) {
    if (event.kind !== "session" || event.session_id !== sessionId) {
      continue;
    }
    if (event.phase === "start") {
      startMs = event.timestamp_ms;
    } else if (event.phase === "end") {
      endMs = event.timestamp_ms;
    }
  }
  if (startMs === undefined || endMs === undefined) {
    return null;
  }
  return [startMs, endMs];
}

function matchesQuery(event: TelemetryEvent, query: TelemetryQuery): boolean {
  if (query.sinceMs !== undefined && event.timestamp_ms < query.sinceMs) {
    return false;
  }
  if (query.kind !== undefined && event.kind !== query.kind) {
    return false;
  }
  switch (event.kind) {
    case "device":
      return (
        !query.sensorId &&
        !query.taskName &&
        (!query.deviceId || query.deviceId === event.device_id)
      );
    case "device_heartbeat":
      return (
        !query.sensorId &&
        !query.taskName &&
        (!query.deviceId || query.deviceId === event.device_id)
      );
    case "sensor":
      return (
        !query.deviceId &&
        !query.taskName &&
        (!query.sensorId || query.sensorId === event.sensor_id)
      );
    case "heartbeat":
      return (
        !query.deviceId &&
        !query.sensorId &&
        (!query.taskName || query.taskName === event.task_name)
      );
    case "health":
    case "session":
    case "runtime_metrics":
      return !query.deviceId && !query.sensorId && !query.taskName;
    default:
      return true;
  }
}

function queryEvents(query: TelemetryQuery): TelemetryEvent[] {
  const all = readAllEvents();
  let events = all.filter((event) => {
    if (query.sessionId) {
      const eventSession =
        event.kind === "session" || event.kind === "runtime_metrics"
          ? event.session_id
          : "session_id" in event
            ? event.session_id
            : undefined;
      if (eventSession) {
        if (eventSession !== query.sessionId) {
          return false;
        }
      } else {
        const window = sessionTimeWindow(all, query.sessionId);
        if (!window) {
          return false;
        }
        const [startMs, endMs] = window;
        if (event.timestamp_ms < startMs || event.timestamp_ms > endMs) {
          return false;
        }
      }
    }
    return matchesQuery(event, query);
  });
  if (query.limit !== undefined && events.length > query.limit) {
    events = events.slice(events.length - query.limit);
  }
  return events;
}

function formatEvent(event: TelemetryEvent): string {
  switch (event.kind) {
    case "device":
      return `[device] ${event.timestamp_ms}ms ${event.device_id} ${event.metric} = ${JSON.stringify(event.value)}${event.robot_id ? ` robot=${event.robot_id}` : ""}`;
    case "sensor":
      return `[sensor] ${event.timestamp_ms}ms ${event.sensor_id} (${event.sensor_type}) = ${JSON.stringify(event.value)}${event.robot_id ? ` robot=${event.robot_id}` : ""}`;
    case "heartbeat":
      return `[heartbeat] ${event.timestamp_ms}ms task=${event.task_name}${event.robot_id ? ` robot=${event.robot_id}` : ""}`;
    case "device_heartbeat":
      return `[device_heartbeat] ${event.timestamp_ms}ms device=${event.device_id}${event.robot_id ? ` robot=${event.robot_id}` : ""}${event.protocol ? ` protocol=${event.protocol}` : ""}`;
    case "health":
      return `[health] ${event.timestamp_ms}ms ${event.target} -> ${event.status}`;
    case "session":
      return `[session] ${event.timestamp_ms}ms ${event.session_id} phase=${event.phase}${event.source ? ` source=${event.source}` : ""}${event.mission_trace_path ? ` trace=${event.mission_trace_path}` : ""}`;
    case "runtime_metrics":
      return `[runtime_metrics] ${event.timestamp_ms}ms session=${event.session_id}`;
    default:
      return JSON.stringify(event);
  }
}

function missionTraceForSession(sessionId: string): string | undefined {
  return listTelemetrySessions().find((session) => session.session_id === sessionId)?.mission_trace_path;
}

function computeStats(): TelemetryStats {
  const events = readAllEvents();
  const index = readHeartbeatIndex();
  const stats: TelemetryStats = {
    total_events: events.length,
    device_events: 0,
    sensor_events: 0,
    heartbeat_events: 0,
    device_heartbeat_events: 0,
    health_events: 0,
    session_events: 0,
    runtime_metrics_events: 0,
    tracked_tasks: Object.keys(index.tasks).length,
    tracked_devices: Object.keys(index.devices ?? {}).length,
  };
  for (const event of events) {
    switch (event.kind) {
      case "device":
        stats.device_events += 1;
        break;
      case "sensor":
        stats.sensor_events += 1;
        break;
      case "heartbeat":
        stats.heartbeat_events += 1;
        break;
      case "device_heartbeat":
        stats.device_heartbeat_events += 1;
        break;
      case "health":
        stats.health_events += 1;
        break;
      case "session":
        stats.session_events += 1;
        break;
      case "runtime_metrics":
        stats.runtime_metrics_events += 1;
        break;
      default:
        break;
    }
  }
  return stats;
}

function parseQueryArgs(args: string[]): { query: TelemetryQuery; json: boolean; metric?: string } {
  const query: TelemetryQuery = {};
  let json = false;
  let metric: string | undefined;
  for (let i = 0; i < args.length; i += 1) {
    const arg = args[i];
    switch (arg) {
      case "--json":
        json = true;
        break;
      case "--device":
        query.deviceId = args[++i];
        break;
      case "--sensor":
        query.sensorId = args[++i];
        break;
      case "--task":
        query.taskName = args[++i];
        break;
      case "--metric":
        metric = args[++i];
        break;
      case "--kind":
        query.kind = args[++i];
        break;
      case "--session":
        query.sessionId = args[++i];
        break;
      case "--since":
        query.sinceMs = Number(args[++i]);
        break;
      case "--limit":
        query.limit = Number(args[++i]);
        break;
      default:
        throw new Error(`Unknown telemetry flag: ${arg}`);
    }
  }
  return { query, json, metric };
}

function escapeLabel(value: string): string {
  return value.replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/\n/g, "\\n");
}

function writeMetric(
  lines: string[],
  name: string,
  labels: Record<string, string>,
  value: number,
): void {
  const labelText = Object.entries(labels)
    .map(([key, labelValue]) => `${key}="${escapeLabel(labelValue)}"`)
    .join(",");
  const suffix = labelText.length > 0 ? `{${labelText}}` : "";
  lines.push(`${name}${suffix} ${Number.isFinite(value) ? value : 0}`);
}

function renderPrometheus(): string {
  const events = readAllEvents();
  const index = readHeartbeatIndex();
  const stats = computeStats();
  const lines: string[] = [];

  lines.push("# HELP spanda_telemetry_events_total Telemetry events in the persistent store by kind.");
  lines.push("# TYPE spanda_telemetry_events_total gauge");
  for (const [kind, count] of [
    ["device", stats.device_events],
    ["sensor", stats.sensor_events],
    ["heartbeat", stats.heartbeat_events],
    ["device_heartbeat", stats.device_heartbeat_events],
    ["health", stats.health_events],
    ["session", stats.session_events],
    ["runtime_metrics", stats.runtime_metrics_events],
  ] as const) {
    writeMetric(lines, "spanda_telemetry_events_total", { kind }, count);
  }

  lines.push("# HELP spanda_task_heartbeat_last_timestamp_ms Latest task heartbeat timestamp in simulation milliseconds.");
  lines.push("# TYPE spanda_task_heartbeat_last_timestamp_ms gauge");
  for (const [task, timestamp] of Object.entries(index.tasks)) {
    writeMetric(lines, "spanda_task_heartbeat_last_timestamp_ms", { task }, timestamp);
  }

  const latestMetrics = [...events].reverse().find((event) => event.kind === "runtime_metrics");
  if (latestMetrics?.kind === "runtime_metrics") {
    const metrics = latestMetrics.metrics as Record<string, unknown>;
    const scheduler = metrics.scheduler as Record<string, number> | undefined;
    lines.push("# HELP spanda_scheduler_ticks Scheduler ticks from the latest runtime_metrics snapshot.");
    lines.push("# TYPE spanda_scheduler_ticks gauge");
    writeMetric(lines, "spanda_scheduler_ticks", {}, scheduler?.scheduler_ticks ?? 0);
  }

  return `${lines.join("\n")}\n`;
}

function renderOtlp(): string {
  return renderOtlpJson();
}

export async function runTelemetryPush(args: string[]): Promise<number> {
  let endpoint = envOtlpEndpoint();
  let token = envOtlpToken();
  let watch = false;
  let intervalMs: number | undefined;
  for (let i = 0; i < args.length; i += 1) {
    if (args[i] === "--endpoint") {
      endpoint = args[++i];
    } else if (args[i] === "--token") {
      token = args[++i];
    } else if (args[i] === "--watch") {
      watch = true;
    } else if (args[i] === "--interval") {
      intervalMs = Number(args[++i]);
    } else {
      throw new Error(`Unknown telemetry push flag: ${args[i]}`);
    }
  }
  if (!endpoint) {
    console.error("telemetry push requires --endpoint <url> or SPANDA_OTLP_ENDPOINT");
    return 1;
  }
  if (watch) {
    const resolvedInterval = intervalMs ?? envPushIntervalMs();
    console.error(
      `Watching telemetry store; pushing OTLP every ${resolvedInterval}ms (Ctrl+C to stop)`,
    );
    try {
      await runOtlpPushLoop({
        endpoint,
        token,
        intervalMs: resolvedInterval,
        once: false,
      });
      return 0;
    } catch (error) {
      console.error(`telemetry push failed: ${error instanceof Error ? error.message : error}`);
      return 1;
    }
  }
  try {
    await runOtlpPushLoop({
      endpoint,
      token,
      intervalMs: intervalMs ?? envPushIntervalMs(),
      once: true,
    });
    console.log(`Pushed OTLP metrics to ${endpoint}`);
    return 0;
  } catch (error) {
    console.error(`telemetry push failed: ${error instanceof Error ? error.message : error}`);
    return 1;
  }
}

function parseReplayArgs(args: string[]): {
  sessionId?: string;
  from?: string;
  deterministic: boolean;
  playback: boolean;
  json: boolean;
} {
  let sessionId: string | undefined;
  let from: string | undefined;
  let deterministic = false;
  let playback = false;
  let json = false;
  for (let i = 0; i < args.length; i += 1) {
    const arg = args[i];
    switch (arg) {
      case "--session":
        sessionId = args[++i];
        break;
      case "--from":
        from = args[++i];
        break;
      case "--deterministic":
        deterministic = true;
        break;
      case "--playback":
        playback = true;
        break;
      case "--json":
        json = true;
        break;
      default:
        throw new Error(`Unknown telemetry replay flag: ${arg}`);
    }
  }
  return { sessionId, from, deterministic, playback, json };
}

function runTelemetryReplay(args: string[]): number {
  const { sessionId, from, deterministic, playback, json } = parseReplayArgs(args);
  if (!sessionId) {
    console.error("telemetry replay requires --session <id>");
    return 1;
  }
  const tracePath = missionTraceForSession(sessionId);
  if (!tracePath) {
    console.error(`No mission trace linked for session ${sessionId}`);
    console.error("Run with --record to link a mission trace on session end.");
    return 1;
  }
  if (!existsSync(tracePath)) {
    console.error(`Mission trace file not found: ${tracePath}`);
    return 1;
  }
  if (deterministic) {
    const offsetMs = from ? parseReplayOffset(from) : 0;
    const verification = replayMissionDeterministic(tracePath, { fromMs: offsetMs });
    if (json) {
      console.log(
        JSON.stringify(
          {
            ok: verification.ok,
            source: loadMissionTrace(tracePath).source,
            deterministic: true,
            offset_ms: offsetMs,
            matched: verification.matched,
            mismatches: verification.mismatches,
          },
          null,
          2,
        ),
      );
    } else if (verification.ok) {
      console.log(
        `✓ Deterministic replay verified for ${tracePath} (${verification.matched} frames from ${offsetMs}ms)`,
      );
    } else {
      console.error(`✗ Deterministic replay mismatch for ${tracePath}:`);
      for (const mismatch of verification.mismatches) {
        console.error(`  ${mismatch}`);
      }
      return 1;
    }
    return verification.ok ? 0 : 1;
  }
  const trace = loadMissionTrace(tracePath);
  const offsetMs = from ? parseReplayOffset(from) : 0;
  const frames = traceFramesFrom(trace, offsetMs);
  if (playback) {
    const report = playbackFrames(
      frames,
      () => {
        /* state snapshots applied by caller when available */
      },
      true,
    );
    if (json) {
      console.log(
        JSON.stringify(
          {
            ok: true,
            mode: "playback",
            frames_applied: report.framesApplied,
            states_applied: report.statesApplied,
            offset_ms: offsetMs,
          },
          null,
          2,
        ),
      );
    } else {
      console.log(
        `Playback ${tracePath}: ${report.framesApplied} frames (${report.statesApplied} with state) from ${offsetMs}ms`,
      );
    }
    return 0;
  }
  if (json) {
    console.log(
      JSON.stringify(
        {
          ok: true,
          source: trace.source,
          deterministic: trace.deterministic,
          offset_ms: offsetMs,
          frames,
        },
        null,
        2,
      ),
    );
    return 0;
  }
  console.log(`Replay ${tracePath} (${frames.length} frames from ${offsetMs}ms)`);
  for (const frame of frames.slice(0, 20)) {
    console.log(`  t=${frame.simTimeMs.toFixed(1)}ms ${frame.event}`, frame.payload ?? {});
  }
  if (frames.length > 20) {
    console.log(`  ... ${frames.length - 20} more frames`);
  }
  return 0;
}

function runTelemetryServe(args: string[]): number {
  let bind = "127.0.0.1:9090";
  let once = false;
  for (let i = 0; i < args.length; i += 1) {
    if (args[i] === "--bind") {
      bind = args[++i] ?? bind;
    } else if (args[i] === "--once") {
      once = true;
    } else {
      throw new Error(`Unknown telemetry serve flag: ${args[i]}`);
    }
  }
  const [host, portText] = bind.includes(":") ? bind.split(":") : ["127.0.0.1", bind];
  const port = Number(portText);
  const server = http.createServer((req, res) => {
    const path = req.url?.split("?")[0] ?? "/";
    if (path === "/healthz") {
      res.writeHead(200, { "Content-Type": "text/plain; charset=utf-8" });
      res.end("ok");
      return;
    }
    if (path === "/metrics") {
      res.writeHead(200, { "Content-Type": "text/plain; version=0.0.4; charset=utf-8" });
      res.end(renderPrometheus());
      return;
    }
    if (path === "/otlp/v1/metrics") {
      res.writeHead(200, { "Content-Type": "application/json; charset=utf-8" });
      res.end(renderOtlp());
      return;
    }
    res.writeHead(404, { "Content-Type": "text/plain; charset=utf-8" });
    res.end("not found");
  });
  server.listen(port, host, () => {
    console.error(`Spanda telemetry server listening on http://${bind}`);
    console.error("  GET /metrics         Prometheus text");
    console.error("  GET /otlp/v1/metrics OTLP/JSON metrics");
    console.error("  GET /healthz         liveness");
  });
  if (once) {
    server.once("request", () => {
      setTimeout(() => server.close(), 50);
    });
  }
  return 0;
}

export function runTelemetryCli(sub: string, args: string[]): number {
  try {
    switch (sub) {
      case "list": {
        const { query, json } = parseQueryArgs(args);
        const events = queryEvents(query);
        if (json) {
          console.log(JSON.stringify(events, null, 2));
        } else if (events.length === 0) {
          console.log(`No telemetry events found in ${resolveStorePath()}`);
        } else {
          for (const event of events) {
            console.log(formatEvent(event));
          }
        }
        return 0;
      }
      case "latest": {
        const { query, json, metric } = parseQueryArgs(args);
        let event: TelemetryEvent | undefined;
        if (query.deviceId) {
          if (metric) {
            event = [...readAllEvents()]
              .reverse()
              .find(
                (entry) =>
                  entry.kind === "device" &&
                  entry.device_id === query.deviceId &&
                  entry.metric === metric,
              );
          } else {
            const index = readHeartbeatIndex();
            const timestamp = index.devices?.[query.deviceId];
            if (timestamp !== undefined) {
              event = {
                kind: "device_heartbeat",
                device_id: query.deviceId,
                timestamp_ms: timestamp,
              };
            }
          }
        } else if (query.sensorId) {
          event = [...readAllEvents()]
            .reverse()
            .find((entry) => entry.kind === "sensor" && entry.sensor_id === query.sensorId);
        } else if (query.taskName) {
          const index = readHeartbeatIndex();
          const timestamp = index.tasks[query.taskName];
          if (timestamp !== undefined) {
            event = {
              kind: "heartbeat",
              task_name: query.taskName,
              timestamp_ms: timestamp,
            };
          }
        } else {
          console.error("telemetry latest requires --device, --sensor, or --task");
          return 1;
        }
        if (json) {
          console.log(JSON.stringify(event ?? null, null, 2));
        } else if (event) {
          console.log(formatEvent(event));
        } else {
          console.log("No matching telemetry event found");
        }
        return 0;
      }
      case "stats": {
        const json = args.includes("--json");
        const stats = computeStats();
        if (json) {
          console.log(JSON.stringify({ store: resolveStorePath(), ...stats }, null, 2));
        } else {
          console.log(`Store: ${resolveStorePath()}`);
          console.log(`Total events: ${stats.total_events}`);
          console.log(`Device events: ${stats.device_events}`);
          console.log(`Sensor events: ${stats.sensor_events}`);
          console.log(`Heartbeat events: ${stats.heartbeat_events}`);
          console.log(`Device heartbeat events: ${stats.device_heartbeat_events}`);
          console.log(`Health events: ${stats.health_events}`);
          console.log(`Session events: ${stats.session_events}`);
          console.log(`Runtime metrics events: ${stats.runtime_metrics_events}`);
          console.log(`Tracked tasks: ${stats.tracked_tasks}`);
          console.log(`Tracked devices: ${stats.tracked_devices}`);
        }
        return 0;
      }
      case "heartbeats": {
        const json = args.includes("--json");
        const index = readHeartbeatIndex();
        if (json) {
          console.log(JSON.stringify(index, null, 2));
        } else if (Object.keys(index.tasks).length === 0 && Object.keys(index.devices ?? {}).length === 0) {
          console.log("No task heartbeats recorded");
        } else {
          if (Object.keys(index.tasks).length > 0) {
            console.log("Tasks:");
            for (const [task, timestamp] of Object.entries(index.tasks)) {
              console.log(`  ${task}: ${timestamp} ms`);
            }
          }
          if (Object.keys(index.devices ?? {}).length > 0) {
            console.log("Devices:");
            for (const [device, timestamp] of Object.entries(index.devices ?? {})) {
              console.log(`  ${device}: ${timestamp} ms`);
            }
          }
        }
        return 0;
      }
      case "devices": {
        const json = args.includes("--json");
        const index = readHeartbeatIndex();
        if (json) {
          console.log(JSON.stringify(index.devices ?? {}, null, 2));
        } else if (Object.keys(index.devices ?? {}).length === 0) {
          console.log("No device heartbeats recorded");
        } else {
          for (const [device, timestamp] of Object.entries(index.devices ?? {})) {
            console.log(`${device}: ${timestamp} ms`);
          }
        }
        return 0;
      }
      case "export": {
        let out = resolveStorePath();
        for (let i = 0; i < args.length; i += 1) {
          if (args[i] === "--out") {
            out = args[++i] ?? out;
          }
        }
        const events = readAllEvents();
        const parent = dirname(out);
        if (!existsSync(parent)) {
          mkdirSync(parent, { recursive: true });
        }
        writeFileSync(
          out,
          events.length > 0 ? `${events.map((event) => JSON.stringify(event)).join("\n")}\n` : "",
          "utf8",
        );
        console.log(`Exported telemetry to ${out}`);
        return 0;
      }
      case "info": {
        const json = args.includes("--json");
        const storePath = resolveStorePath();
        const info = {
          backend: usesSqliteBackend() ? "sqlite" : "jsonl",
          store_path: storePath,
          heartbeat_path: usesSqliteBackend() ? null : resolveHeartbeatIndexPath(storePath),
          persist_enabled: envPersistEnabled(),
          sqlite_available: sqliteBackendAvailable(),
          max_events: process.env.SPANDA_TELEMETRY_MAX_EVENTS ?? null,
          event_count: readAllEvents().length,
        };
        if (json) {
          console.log(JSON.stringify(info, null, 2));
        } else {
          console.log(`Backend: ${info.backend}`);
          console.log(`Store: ${info.store_path}`);
          if (info.heartbeat_path) {
            console.log(`Heartbeat index: ${info.heartbeat_path}`);
          }
          console.log(`Events: ${info.event_count}`);
          if (info.max_events) {
            console.log(`Retention max: ${info.max_events}`);
          }
          if (envBackendSqlite() && !info.sqlite_available) {
            console.log("SQLite backend requires Node.js 22+ or the native Rust CLI");
          }
        }
        return 0;
      }
      case "prometheus": {
        let out: string | undefined;
        for (let i = 0; i < args.length; i += 1) {
          if (args[i] === "--out") {
            out = args[++i];
          }
        }
        const body = renderPrometheus();
        if (out) {
          const parent = dirname(out);
          if (!existsSync(parent)) {
            mkdirSync(parent, { recursive: true });
          }
          writeFileSync(out, body, "utf8");
          console.log(`Exported Prometheus metrics to ${out}`);
        } else {
          process.stdout.write(body);
        }
        return 0;
      }
      case "otlp": {
        let out: string | undefined;
        for (let i = 0; i < args.length; i += 1) {
          if (args[i] === "--out") {
            out = args[++i];
          }
        }
        const body = renderOtlp();
        if (out) {
          const parent = dirname(out);
          if (!existsSync(parent)) {
            mkdirSync(parent, { recursive: true });
          }
          writeFileSync(out, body, "utf8");
          console.log(`Exported OTLP metrics to ${out}`);
        } else {
          console.log(body);
        }
        return 0;
      }
      case "serve":
        return runTelemetryServe(args);
      case "sessions": {
        const json = args.includes("--json");
        const sessions = listTelemetrySessions();
        if (json) {
          console.log(JSON.stringify(sessions, null, 2));
        } else if (sessions.length === 0) {
          console.log("No telemetry sessions recorded");
        } else {
          for (const session of sessions) {
            console.log(
              `${session.session_id} start=${session.start_ms}ms end=${session.end_ms ?? "open"} events=${session.event_count}${session.source ? ` source=${session.source}` : ""}${session.mission_trace_path ? ` trace=${session.mission_trace_path}` : ""}`,
            );
          }
        }
        return 0;
      }
      case "replay":
        return runTelemetryReplay(args);
      default:
        console.error(`Unknown telemetry subcommand: ${sub}`);
        return 1;
    }
  } catch (error) {
    console.error(`telemetry ${sub} failed: ${String(error)}`);
    return 1;
  }
}
