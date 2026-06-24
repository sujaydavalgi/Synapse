import { afterEach, beforeEach, describe, expect, it } from "vitest";
import {
  envAutoPushEnabled,
  envPushIntervalMs,
} from "../src/telemetry-push.js";

const ENV_KEYS = [
  "SPANDA_OTLP_AUTO_PUSH",
  "SPANDA_OTLP_PUSH",
  "SPANDA_OTLP_PUSH_INTERVAL_MS",
] as const;

describe("telemetry push env", () => {
  let savedEnv: Partial<Record<(typeof ENV_KEYS)[number], string | undefined>>;

  beforeEach(() => {
    savedEnv = {};
    for (const key of ENV_KEYS) {
      savedEnv[key] = process.env[key];
      delete process.env[key];
    }
  });

  afterEach(() => {
    for (const key of ENV_KEYS) {
      if (savedEnv[key] === undefined) {
        delete process.env[key];
      } else {
        process.env[key] = savedEnv[key];
      }
    }
  });

  it("reads auto-push flags", () => {
    expect(envAutoPushEnabled()).toBe(false);
    process.env.SPANDA_OTLP_AUTO_PUSH = "1";
    expect(envAutoPushEnabled()).toBe(true);
    delete process.env.SPANDA_OTLP_AUTO_PUSH;
    process.env.SPANDA_OTLP_PUSH = "true";
    expect(envAutoPushEnabled()).toBe(true);
  });

  it("defaults push interval to 30s", () => {
    expect(envPushIntervalMs()).toBe(30_000);
    process.env.SPANDA_OTLP_PUSH_INTERVAL_MS = "5000";
    expect(envPushIntervalMs()).toBe(5_000);
  });
});
