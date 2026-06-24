/**
 * OTLP remote push helpers for one-shot and scheduled collector ingest.
 * @module
 */

import { remoteFetch } from "./http-fetch.js";
import { renderOtlpJson } from "./telemetry-otlp.js";

export type OtlpPushOptions = {
  endpoint: string;
  token?: string;
  intervalMs: number;
  once: boolean;
};

function envTruthy(name: string): boolean {
  const value = process.env[name];
  return value === "1" || value?.toLowerCase() === "true";
}

/** True when `SPANDA_OTLP_AUTO_PUSH=1` or `SPANDA_OTLP_PUSH=1`. */
export function envAutoPushEnabled(): boolean {
  return envTruthy("SPANDA_OTLP_AUTO_PUSH") || envTruthy("SPANDA_OTLP_PUSH");
}

/** Resolved OTLP collector URL from `SPANDA_OTLP_ENDPOINT`. */
export function envOtlpEndpoint(): string | undefined {
  return process.env.SPANDA_OTLP_ENDPOINT;
}

/** Bearer token from `SPANDA_OTLP_TOKEN`. */
export function envOtlpToken(): string | undefined {
  return process.env.SPANDA_OTLP_TOKEN;
}

/** Push interval for watch mode (`SPANDA_OTLP_PUSH_INTERVAL_MS`, default 30s). */
export function envPushIntervalMs(): number {
  const parsed = Number(process.env.SPANDA_OTLP_PUSH_INTERVAL_MS);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 30_000;
}

/** POST OTLP/JSON metrics to a remote collector. */
export async function pushOtlpJson(
  endpoint: string,
  body: string,
  token?: string,
): Promise<void> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }
  const response = await remoteFetch(endpoint, {
    method: "POST",
    headers,
    body,
  });
  if (response.ok) {
    return;
  }
  const text = await response.text();
  throw new Error(`HTTP ${response.status} from ${endpoint}${text ? `: ${text}` : ""}`);
}

/** Push the current persistent store snapshot to an OTLP endpoint. */
export async function pushGlobalStore(endpoint: string, token?: string): Promise<void> {
  await pushOtlpJson(endpoint, renderOtlpJson(), token);
}

/** Push after a run session ends when auto-push env vars are set. */
export async function maybeAutoPushAfterSession(): Promise<void> {
  if (!envAutoPushEnabled()) {
    return;
  }
  const endpoint = envOtlpEndpoint();
  if (!endpoint) {
    console.error("SPANDA_OTLP_AUTO_PUSH set but SPANDA_OTLP_ENDPOINT is missing");
    return;
  }
  try {
    await pushGlobalStore(endpoint, envOtlpToken());
    console.error(`Auto-pushed OTLP metrics to ${endpoint}`);
  } catch (error) {
    console.error(`OTLP auto-push failed: ${error instanceof Error ? error.message : error}`);
  }
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

/** Run a one-shot or periodic OTLP push loop until interrupted. */
export async function runOtlpPushLoop(options: OtlpPushOptions): Promise<void> {
  for (;;) {
    await pushGlobalStore(options.endpoint, options.token);
    if (options.once) {
      return;
    }
    await sleep(options.intervalMs);
  }
}
