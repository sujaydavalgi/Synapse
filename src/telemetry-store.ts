/**
 * Persistent telemetry store mirror for the TypeScript interpreter.
 * @module
 */

import { appendFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import type { RuntimeValue } from "./runtime/interpreter.js";
import {
  envBackendSqlite,
  resolveSqlitePath,
  sqliteAppendEvent,
  sqliteBackendAvailable,
  sqliteCompact,
  sqliteReadAll,
  sqliteReadHeartbeatIndex,
  sqliteUpsertHeartbeat,
} from "./telemetry-sqlite.js";

export type TelemetryEvent =
  | {
      kind: "device";
      device_id: string;
      metric: string;
      value: unknown;
      timestamp_ms: number;
      robot_id?: string;
      session_id?: string;
    }
  | {
      kind: "sensor";
      sensor_id: string;
      sensor_type: string;
      value: unknown;
      timestamp_ms: number;
      robot_id?: string;
      session_id?: string;
    }
  | {
      kind: "heartbeat";
      task_name: string;
      timestamp_ms: number;
      robot_id?: string;
      session_id?: string;
    }
  | {
      kind: "device_heartbeat";
      device_id: string;
      timestamp_ms: number;
      robot_id?: string;
      protocol?: string;
      session_id?: string;
    }
  | {
      kind: "health";
      target: string;
      status: string;
      timestamp_ms: number;
      session_id?: string;
    }
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

let sessionPersist = false;
let activeSessionId: string | undefined;
let maxEvents: number | undefined;
const lastHeartbeatHistory = new Map<string, number>();
const lastDeviceHeartbeatHistory = new Map<string, number>();

export function defaultStorePath(): string {
  return ".spanda/telemetry-store.jsonl";
}

export function defaultHeartbeatIndexPath(): string {
  return ".spanda/telemetry-heartbeats.json";
}

export function resolveStorePath(): string {
  if (envBackendSqlite()) {
    return resolveSqlitePath();
  }
  return process.env.SPANDA_TELEMETRY_STORE_PATH ?? defaultStorePath();
}

export function resolveHeartbeatIndexPath(storePath = resolveStorePath()): string {
  if (envBackendSqlite()) {
    return "";
  }
  return (
    process.env.SPANDA_TELEMETRY_HEARTBEAT_PATH ??
    `${dirname(storePath)}/telemetry-heartbeats.json`
  );
}

export function envPersistEnabled(): boolean {
  const value = process.env.SPANDA_TELEMETRY_STORE;
  return value === "1" || value?.toLowerCase() === "true";
}

export function usesSqliteBackend(): boolean {
  return envBackendSqlite();
}

function resolveMaxEvents(): number | undefined {
  const raw = process.env.SPANDA_TELEMETRY_MAX_EVENTS;
  if (!raw) {
    return undefined;
  }
  const parsed = Number(raw);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : undefined;
}

export function configureSessionPersist(enabled: boolean): void {
  sessionPersist = enabled;
  if (enabled) {
    maxEvents = resolveMaxEvents();
  }
  if (!enabled) {
    activeSessionId = undefined;
  }
}

export function beginRunSession(source?: string): string {
  if (!persistEnabled()) {
    return "";
  }
  const stem = source?.replace(/\.sd$/, "").split("/").pop() ?? "program";
  const sessionId = `${stem}-${Date.now()}`;
  activeSessionId = sessionId;
  appendEvent({
    kind: "session",
    session_id: sessionId,
    phase: "start",
    source,
    timestamp_ms: Date.now(),
  });
  return sessionId;
}

export function endRunSession(
  missionTracePath?: string,
  metrics?: unknown,
  timestampMs = Date.now(),
): void {
  if (!persistEnabled() || !activeSessionId) {
    return;
  }
  const sessionId = activeSessionId;
  appendEvent({
    kind: "session",
    session_id: sessionId,
    phase: "end",
    mission_trace_path: missionTracePath,
    timestamp_ms: timestampMs,
  });
  if (metrics !== undefined) {
    appendEvent({
      kind: "runtime_metrics",
      session_id: sessionId,
      metrics,
      timestamp_ms: timestampMs,
    });
  }
  activeSessionId = undefined;
  void import("./telemetry-push.js").then(({ maybeAutoPushAfterSession }) => maybeAutoPushAfterSession());
}

export function persistEnabled(): boolean {
  return sessionPersist || envPersistEnabled();
}

export function readAllEvents(): TelemetryEvent[] {
  const storePath = resolveStorePath();
  if (envBackendSqlite()) {
    return sqliteReadAll(storePath);
  }
  if (!existsSync(storePath)) {
    return [];
  }
  return readFileSync(storePath, "utf8")
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
    .map((line) => JSON.parse(line) as TelemetryEvent);
}

export type TelemetrySessionSummary = {
  session_id: string;
  source?: string;
  start_ms: number;
  end_ms?: number;
  mission_trace_path?: string;
  event_count: number;
};

function eventMatchesSession(event: TelemetryEvent, sessionId: string): boolean {
  if (event.kind === "session" && event.session_id === sessionId) {
    return true;
  }
  return "session_id" in event && event.session_id === sessionId;
}

export function listTelemetrySessions(): TelemetrySessionSummary[] {
  const events = readAllEvents();
  const summaries = new Map<string, TelemetrySessionSummary>();
  for (const event of events) {
    if (event.kind !== "session") {
      continue;
    }
    const existing = summaries.get(event.session_id) ?? {
      session_id: event.session_id,
      start_ms: event.timestamp_ms,
      event_count: 0,
    };
    if (event.phase === "start") {
      existing.start_ms = event.timestamp_ms;
      existing.source = event.source;
    } else if (event.phase === "end") {
      existing.end_ms = event.timestamp_ms;
      if (event.mission_trace_path) {
        existing.mission_trace_path = event.mission_trace_path;
      }
    }
    summaries.set(event.session_id, existing);
  }
  for (const summary of summaries.values()) {
    summary.event_count = events.filter((event) => eventMatchesSession(event, summary.session_id)).length;
  }
  return [...summaries.values()].sort((left, right) => right.start_ms - left.start_ms);
}

function ensureParent(path: string): void {
  const parent = dirname(path);
  if (!existsSync(parent)) {
    mkdirSync(parent, { recursive: true });
  }
}

function maybeCompactJsonl(storePath: string): void {
  const limit = maxEvents ?? resolveMaxEvents();
  if (!limit) {
    return;
  }
  const events = readAllEvents();
  if (events.length <= limit) {
    return;
  }
  const trimmed = events.slice(events.length - limit);
  writeFileSync(
    storePath,
    `${trimmed.map((event) => JSON.stringify(event)).join("\n")}\n`,
    "utf8",
  );
}

function maybeCompactStore(storePath: string): void {
  const limit = maxEvents ?? resolveMaxEvents();
  if (!limit) {
    return;
  }
  if (envBackendSqlite()) {
    const events = sqliteReadAll(storePath);
    if (events.length > limit) {
      sqliteCompact(storePath, limit);
    }
    return;
  }
  maybeCompactJsonl(storePath);
}

function appendEvent(event: TelemetryEvent): void {
  if (!persistEnabled()) {
    return;
  }
  if (envBackendSqlite() && !sqliteBackendAvailable()) {
    throw new Error(
      "SPANDA_TELEMETRY_BACKEND=sqlite requires Node.js 22+ with node:sqlite, or use the native Rust CLI",
    );
  }
  const stamped =
    activeSessionId &&
    event.kind !== "session" &&
    event.kind !== "runtime_metrics" &&
    !("session_id" in event && event.session_id)
      ? { ...event, session_id: activeSessionId }
      : event;
  const storePath = resolveStorePath();
  if (envBackendSqlite()) {
    sqliteAppendEvent(storePath, stamped);
    maybeCompactStore(storePath);
    return;
  }
  ensureParent(storePath);
  appendFileSync(storePath, `${JSON.stringify(stamped)}\n`, "utf8");
  maybeCompactStore(storePath);
}

function readHeartbeatIndex(path: string): HeartbeatIndex {
  if (!path || !existsSync(path)) {
    return { tasks: {}, devices: {} };
  }
  const parsed = JSON.parse(readFileSync(path, "utf8")) as HeartbeatIndex;
  parsed.devices ??= {};
  return parsed;
}

function writeHeartbeatIndex(path: string, index: HeartbeatIndex): void {
  ensureParent(path);
  writeFileSync(path, JSON.stringify(index, null, 2), "utf8");
}

function persistHeartbeatIndex(
  storePath: string,
  targetKind: "task" | "device",
  targetId: string,
  timestampMs: number,
): void {
  if (envBackendSqlite()) {
    sqliteUpsertHeartbeat(storePath, targetKind, targetId, timestampMs);
    return;
  }
  const heartbeatPath = resolveHeartbeatIndexPath(storePath);
  const index = readHeartbeatIndex(heartbeatPath);
  if (targetKind === "task") {
    index.tasks[targetId] = timestampMs;
  } else {
    index.devices ??= {};
    index.devices[targetId] = timestampMs;
  }
  writeHeartbeatIndex(heartbeatPath, index);
}

export function readHeartbeatIndexForStore(): HeartbeatIndex {
  const storePath = resolveStorePath();
  if (envBackendSqlite()) {
    return sqliteReadHeartbeatIndex(storePath);
  }
  return readHeartbeatIndex(resolveHeartbeatIndexPath(storePath));
}

function runtimeValueToJson(value: RuntimeValue): unknown {
  switch (value.kind) {
    case "number":
      return { kind: "number", value: value.value, unit: value.unit };
    case "bool":
      return { kind: "bool", value: value.value };
    case "string":
      return { kind: "string", value: value.value };
    case "scan":
      return { kind: "scan", nearest_distance: value.nearestDistance };
    case "pose":
      return { kind: "pose", x: value.x, y: value.y, theta: value.theta, z: value.z };
    case "object":
      return {
        kind: "object",
        fields: Object.fromEntries(
          Object.entries(value.fields).map(([key, field]) => [key, runtimeValueToJson(field)]),
        ),
      };
    default:
      return { kind: value.kind };
  }
}

export function recordSensorReading(
  sensorId: string,
  sensorType: string,
  value: RuntimeValue,
  timestampMs: number,
  robotId?: string,
): void {
  appendEvent({
    kind: "sensor",
    sensor_id: sensorId,
    sensor_type: sensorType,
    value: runtimeValueToJson(value),
    timestamp_ms: timestampMs,
    robot_id: robotId,
  });
}

export function recordTaskHeartbeat(
  taskName: string,
  timestampMs: number,
  robotId?: string,
  historyIntervalMs = 5000,
): void {
  if (!persistEnabled()) {
    return;
  }
  const storePath = resolveStorePath();
  persistHeartbeatIndex(storePath, "task", taskName, timestampMs);

  const last = lastHeartbeatHistory.get(taskName) ?? Number.NEGATIVE_INFINITY;
  if (timestampMs - last < historyIntervalMs) {
    return;
  }
  lastHeartbeatHistory.set(taskName, timestampMs);
  appendEvent({
    kind: "heartbeat",
    task_name: taskName,
    timestamp_ms: timestampMs,
    robot_id: robotId,
  });
}

export function isHeartbeatMetric(metric: string): boolean {
  const normalized = metric.toLowerCase();
  return normalized === "heartbeat" || normalized === "liveness" || normalized === "alive" || normalized === "ping";
}

export function recordDeviceHeartbeat(
  deviceId: string,
  timestampMs: number,
  robotId?: string,
  protocol?: string,
  historyIntervalMs = 5000,
): void {
  if (!persistEnabled()) {
    return;
  }
  const storePath = resolveStorePath();
  persistHeartbeatIndex(storePath, "device", deviceId, timestampMs);

  const last = lastDeviceHeartbeatHistory.get(deviceId) ?? Number.NEGATIVE_INFINITY;
  if (timestampMs - last < historyIntervalMs) {
    return;
  }
  lastDeviceHeartbeatHistory.set(deviceId, timestampMs);
  appendEvent({
    kind: "device_heartbeat",
    device_id: deviceId,
    timestamp_ms: timestampMs,
    robot_id: robotId,
    protocol,
  });
}

export function recordHealthEvent(target: string, status: string, timestampMs: number): void {
  appendEvent({
    kind: "health",
    target,
    status,
    timestamp_ms: timestampMs,
  });
}

export function recordTopicPublish(
  robotId: string | undefined,
  topicPath: string,
  value: RuntimeValue,
  timestampMs: number,
): void {
  recordDeviceTelemetry(robotId ?? "robot", topicPath, value, timestampMs, robotId);
}

export function recordDeviceTelemetry(
  deviceId: string,
  metric: string,
  value: RuntimeValue,
  timestampMs: number,
  robotId?: string,
): void {
  appendEvent({
    kind: "device",
    device_id: deviceId,
    metric,
    value: runtimeValueToJson(value),
    timestamp_ms: timestampMs,
    robot_id: robotId,
  });
  if (isHeartbeatMetric(metric)) {
    recordDeviceHeartbeat(deviceId, timestampMs, robotId);
  }
}
