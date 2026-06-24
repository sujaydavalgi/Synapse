import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  beginRunSession,
  configureSessionPersist,
  endRunSession,
  listTelemetrySessions,
  readAllEvents,
  recordSensorReading,
} from "../src/telemetry-store.js";
import { ReliabilityRuntime } from "../src/runtime/reliability-runtime.js";

const ENV_KEYS = [
  "SPANDA_TELEMETRY_STORE",
  "SPANDA_TELEMETRY_STORE_PATH",
  "SPANDA_TELEMETRY_BACKEND",
  "SPANDA_TELEMETRY_MAX_EVENTS",
] as const;

describe("telemetry store", () => {
  let storeDir: string;
  let savedEnv: Partial<Record<(typeof ENV_KEYS)[number], string | undefined>>;

  beforeEach(() => {
    storeDir = mkdtempSync(join(tmpdir(), "spanda-telemetry-"));
    savedEnv = {};
    for (const key of ENV_KEYS) {
      savedEnv[key] = process.env[key];
      delete process.env[key];
    }
    process.env.SPANDA_TELEMETRY_STORE = "1";
    process.env.SPANDA_TELEMETRY_STORE_PATH = join(storeDir, "telemetry.jsonl");
    configureSessionPersist(false);
  });

  afterEach(() => {
    for (const key of ENV_KEYS) {
      if (savedEnv[key] === undefined) {
        delete process.env[key];
      } else {
        process.env[key] = savedEnv[key];
      }
    }
    rmSync(storeDir, { recursive: true, force: true });
  });

  it("records session boundaries and tags events with session_id", () => {
    configureSessionPersist(true);
    const sessionId = beginRunSession("demo.sd");
    recordSensorReading("lidar", "Lidar", { kind: "scan", nearestDistance: 1.2 }, 100);
    endRunSession();

    const events = readAllEvents();
    expect(events.some((event) => event.kind === "session" && event.phase === "start")).toBe(true);
    expect(events.some((event) => event.kind === "session" && event.phase === "end")).toBe(true);
    const sensor = events.find((event) => event.kind === "sensor");
    expect(sensor?.kind === "sensor" && sensor.session_id).toBe(sessionId);
  });

  it("summarizes sessions with linked mission traces", () => {
    configureSessionPersist(true);
    const sessionId = beginRunSession("demo.sd");
    endRunSession("demo.trace", { replay_frames: 3 });

    const sessions = listTelemetrySessions();
    expect(sessions).toHaveLength(1);
    expect(sessions[0]?.session_id).toBe(sessionId);
    expect(sessions[0]?.mission_trace_path).toBe("demo.trace");
    expect(sessions[0]?.event_count).toBeGreaterThanOrEqual(3);
  });

  it("trims JSONL history when SPANDA_TELEMETRY_MAX_EVENTS is set", () => {
    process.env.SPANDA_TELEMETRY_MAX_EVENTS = "2";
    configureSessionPersist(true);
    beginRunSession("demo.sd");
    recordSensorReading("lidar", "Lidar", { kind: "scan", nearestDistance: 1 }, 1);
    recordSensorReading("lidar", "Lidar", { kind: "scan", nearestDistance: 2 }, 2);
    recordSensorReading("lidar", "Lidar", { kind: "scan", nearestDistance: 3 }, 3);
    endRunSession();

    const lines = readFileSync(process.env.SPANDA_TELEMETRY_STORE_PATH!, "utf8")
      .trim()
      .split("\n");
    expect(lines.length).toBeLessThanOrEqual(2);
  });
});

describe("runtime telemetry snapshot", () => {
  it("matches the Rust RuntimeTelemetry JSON shape", () => {
    const runtime = new ReliabilityRuntime();
    runtime.configureScheduler(2, 10);
    runtime.recordSchedulerTick();
    runtime.recordTaskTick("driver", "normal", 10, 5);
    runtime.recordSpawn(true);
    runtime.recordParallelBlock();
    const snapshot = runtime.snapshotRuntimeMetrics(4);
    expect(snapshot.scheduler).toMatchObject({
      multiplexed_tasks: 2,
      scheduler_ticks: 1,
      base_tick_ms: 10,
    });
    expect(snapshot.execution).toMatchObject({
      spawns: 1,
      fire_and_forget_spawns: 1,
      parallel_blocks: 1,
    });
    expect(snapshot.replay_frames).toBe(4);
    expect((snapshot.tasks as Record<string, { ticks: number }>).driver?.ticks).toBe(1);
  });
});
