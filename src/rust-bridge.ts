import { spawnSync } from "node:child_process";
import { existsSync, unlinkSync, writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

export type Diagnostic = { message: string; line: number; column: number };
export type CheckResult = { ok: boolean; diagnostics: Diagnostic[] };
export type RunResult = {
  state: {
    pose: { x: number; y: number; theta: number; z?: number };
    velocity: { linear: number; angular: number };
    emergency_stop: boolean;
  };
  events: string[];
  logs: string[];
};

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "../..");

function cliPath(): string | null {
  const release = join(repoRoot, "target/release/synapse");
  const debug = join(repoRoot, "target/debug/synapse");
  if (existsSync(release)) return release;
  if (existsSync(debug)) return debug;
  return null;
}

export function isCliAvailable(): boolean {
  return cliPath() !== null;
}

export function checkViaCli(source: string): CheckResult {
  const bin = cliPath();
  if (!bin) {
    return {
      ok: false,
      diagnostics: [{ message: "Rust CLI not built (run: npm run build:rust)", line: 1, column: 1 }],
    };
  }
  const tmp = join(repoRoot, ".synapse-check-tmp.syn");
  writeFileSync(tmp, source);
  const result = spawnSync(bin, ["check", "--json", tmp], { encoding: "utf-8" });
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }
  if (!result.stdout?.trim()) {
    return {
      ok: false,
      diagnostics: [{ message: result.stderr || "CLI check failed", line: 1, column: 1 }],
    };
  }
  return JSON.parse(result.stdout) as CheckResult;
}

export function runViaCli(source: string): RunResult {
  const bin = cliPath();
  if (!bin) {
    throw new Error("Rust CLI not built (run: npm run build:rust)");
  }
  const tmp = join(repoRoot, ".synapse-run-tmp.syn");
  writeFileSync(tmp, source);
  const result = spawnSync(bin, ["run", "--json", tmp], { encoding: "utf-8" });
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }
  const parsed = JSON.parse(result.stdout || "{}") as {
    ok: boolean;
    result?: RunResult;
    diagnostics?: Diagnostic[];
  };
  if (!parsed.ok || !parsed.result) {
    throw new Error(parsed.diagnostics?.[0]?.message ?? "Run failed");
  }
  return parsed.result;
}
