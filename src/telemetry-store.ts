/**
 * Persistent telemetry store mirror for the TypeScript interpreter.
 * @module
 */

import { appendFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import type { RuntimeValue } from "./runtime/interpreter.js";

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
      kind: "health";
      target: string;
      status: string;
      timestamp_ms: number;
    };

type HeartbeatIndex = {
  tasks: Record<string, number>;
};

let sessionPersist = false;
const lastHeartbeatHistory = new Map<string, number>();

export function defaultStorePath(): string {
  return ".spanda/telemetry-store.jsonl";
}

export function defaultHeartbeatIndexPath(): string {
  return ".spanda/telemetry-heartbeats.json";
}

export function resolveStorePath(): string {
  return process.env.SPANDA_TELEMETRY_STORE_PATH ?? defaultStorePath();
}

export function resolveHeartbeatIndexPath(storePath = resolveStorePath()): string {
  return (
    process.env.SPANDA_TELEMETRY_HEARTBEAT_PATH ??
    dirname(storePath) + "/telemetry-heartbeats.json"
  );
}

export function envPersistEnabled(): boolean {
  const value = process.env.SPANDA_TELEMETRY_STORE;
  return value === "1" || value?.toLowerCase() === "true";
}

export function configureSessionPersist(enabled: boolean): void {
  sessionPersist = enabled;
}

export function persistEnabled(): boolean {
  return sessionPersist || envPersistEnabled();
}

function ensureParent(path: string): void {
  const parent = dirname(path);
  if (!existsSync(parent)) {
    mkdirSync(parent, { recursive: true });
  }
}

function appendEvent(event: TelemetryEvent): void {
  if (!persistEnabled()) {
    return;
  }
  const storePath = resolveStorePath();
  ensureParent(storePath);
  appendFileSync(storePath, `${JSON.stringify(event)}\n`, "utf8");
}

function readHeartbeatIndex(path: string): HeartbeatIndex {
  if (!existsSync(path)) {
    return { tasks: {} };
  }
  return JSON.parse(readFileSync(path, "utf8")) as HeartbeatIndex;
}

function writeHeartbeatIndex(path: string, index: HeartbeatIndex): void {
  ensureParent(path);
  writeFileSync(path, JSON.stringify(index, null, 2), "utf8");
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
  const heartbeatPath = resolveHeartbeatIndexPath();
  const index = readHeartbeatIndex(heartbeatPath);
  index.tasks[taskName] = timestampMs;
  writeHeartbeatIndex(heartbeatPath, index);

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

export function recordHealthEvent(target: string, status: string, timestampMs: number): void {
  appendEvent({
    kind: "health",
    target,
    status,
    timestamp_ms: timestampMs,
  });
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
}
