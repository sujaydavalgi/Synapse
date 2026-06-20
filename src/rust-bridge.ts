import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import { existsSync, statSync, unlinkSync, writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

export type Diagnostic = { message: string; line: number; column: number };
export type CheckResult = { ok: boolean; diagnostics: Diagnostic[] };

export type CompatSeverity = "pass" | "warning" | "error";

export type CompatItem = {
  category: string;
  message: string;
  severity: CompatSeverity;
  line: number;
  column: number;
};

export type MatrixCell = {
  robot: string;
  target: string;
  compatible: boolean;
};

export type VerifyResult = {
  ok: boolean;
  compatible?: boolean;
  target?: string;
  items: CompatItem[];
  matrix?: { cells: MatrixCell[] };
};
export type RunResult = {
  state: {
    pose: { x: number; y: number; theta: number; z?: number };
    velocity: { linear: number; angular: number };
    emergency_stop: boolean;
  };
  events: string[];
  logs: string[];
};

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..");

function cliPath(): string | null {
  const release = join(repoRoot, "target/release/spanda");
  const debug = join(repoRoot, "target/debug/spanda");
  const candidates = [release, debug].filter((p) => existsSync(p));
  if (candidates.length === 0) return null;
  if (candidates.length === 1) return candidates[0]!;
  const newest = candidates.reduce((a, b) =>
    statSync(a).mtimeMs >= statSync(b).mtimeMs ? a : b,
  );
  return newest;
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
  const tmp = join(repoRoot, ".spanda-check-tmp.sd");
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

export function verifyViaCli(
  source: string,
  args: string[] = [],
): VerifyResult {
  const bin = cliPath();
  if (!bin) {
    return {
      ok: false,
      items: [
        {
          category: "error",
          message: "Rust CLI not built (run: npm run build:rust)",
          severity: "error",
          line: 1,
          column: 1,
        },
      ],
    };
  }
  const tmp = join(repoRoot, ".spanda-verify-tmp.sd");
  writeFileSync(tmp, source);
  const result = spawnSync(bin, ["verify", tmp, "--json", ...args], { encoding: "utf-8" });
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }
  if (!result.stdout?.trim()) {
    return {
      ok: false,
      items: [
        {
          category: "error",
          message: result.stderr || "CLI verify failed",
          severity: "error",
          line: 1,
          column: 1,
        },
      ],
    };
  }
  return JSON.parse(result.stdout) as VerifyResult;
}

export function runViaCli(source: string): RunResult {
  const bin = cliPath();
  if (!bin) {
    throw new Error("Rust CLI not built (run: npm run build:rust)");
  }
  const tmp = join(repoRoot, ".spanda-run-tmp.sd");
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

export type FormatResult = { ok: boolean; changed: boolean; formatted: string };
export type LintIssue = {
  rule: string;
  message: string;
  line: number;
  column: number;
  severity: "warning" | "error";
};
export type LintResult = { ok: boolean; issues: LintIssue[] };
export type DocResult = { ok: boolean; markdown: string };
export type CodegenTarget = "native" | "wasm" | "esp32";
export type DebugPause = { line: number; reason: string };
export type DebugResult = { ok: boolean; pauses: DebugPause[] };

function withTempSource(
  source: string,
  suffix: string,
  run: (file: string) => SpawnSyncReturns<string>,
): SpawnSyncReturns<string> {
  const tmp = join(repoRoot, suffix);
  writeFileSync(tmp, source);
  const result = run(tmp);
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }
  return result;
}

export function fmtViaCli(source: string): FormatResult {
  const bin = cliPath();
  if (!bin) {
    return { ok: false, changed: false, formatted: source };
  }
  const result = withTempSource(source, ".spanda-fmt-tmp.sd", (file) =>
    spawnSync(bin, ["fmt", "--json", file], { encoding: "utf-8" }),
  );
  if (!result.stdout?.trim()) {
    return { ok: false, changed: false, formatted: source };
  }
  return JSON.parse(result.stdout) as FormatResult;
}

export function lintViaCli(source: string): LintResult {
  const bin = cliPath();
  if (!bin) {
    return {
      ok: false,
      issues: [{ rule: "cli", message: "Rust CLI not built", line: 1, column: 1, severity: "error" }],
    };
  }
  const result = withTempSource(source, ".spanda-lint-tmp.sd", (file) =>
    spawnSync(bin, ["lint", "--json", file], { encoding: "utf-8" }),
  );
  if (!result.stdout?.trim()) {
    return {
      ok: false,
      issues: [{ rule: "cli", message: result.stderr || "lint failed", line: 1, column: 1, severity: "error" }],
    };
  }
  return JSON.parse(result.stdout) as LintResult;
}

export function docViaCli(source: string): DocResult {
  const bin = cliPath();
  if (!bin) {
    return { ok: false, markdown: "" };
  }
  const result = withTempSource(source, ".spanda-doc-tmp.sd", (file) =>
    spawnSync(bin, ["doc", "--json", file], { encoding: "utf-8" }),
  );
  if (!result.stdout?.trim()) {
    return { ok: false, markdown: "" };
  }
  return JSON.parse(result.stdout) as DocResult;
}

export function codegenViaCli(source: string, target: CodegenTarget = "native"): string {
  const bin = cliPath();
  if (!bin) {
    throw new Error("Rust CLI not built (run: npm run build:rust)");
  }
  const result = withTempSource(source, ".spanda-codegen-tmp.sd", (file) =>
    spawnSync(bin, ["codegen", "--target", target, file], { encoding: "utf-8" }),
  );
  if (result.status !== 0) {
    throw new Error(result.stderr || "codegen failed");
  }
  return result.stdout ?? "";
}

export function deployViaCli(source: string): string {
  const bin = cliPath();
  if (!bin) {
    throw new Error("Rust CLI not built (run: npm run build:rust)");
  }
  const result = withTempSource(source, ".spanda-deploy-tmp.sd", (file) =>
    spawnSync(bin, ["deploy", "--target", "wasm", file], { encoding: "utf-8" }),
  );
  if (result.status !== 0) {
    throw new Error(result.stderr || "deploy failed");
  }
  return result.stdout ?? "";
}

export function debugViaCli(source: string, breakpoints: number[] = []): DebugResult {
  const bin = cliPath();
  if (!bin) {
    return { ok: false, pauses: [] };
  }
  const args = ["debug", ...breakpoints.flatMap((line) => ["--break", String(line)])];
  const result = withTempSource(source, ".spanda-debug-tmp.sd", (file) =>
    spawnSync(bin, [...args, file], { encoding: "utf-8" }),
  );
  if (result.status !== 0) {
    return { ok: false, pauses: [] };
  }
  const pauses: DebugPause[] = [];
  for (const line of (result.stdout ?? "").split("\n")) {
    const m = line.match(/^\s*line (\d+) — (.+)$/);
    if (m) {
      pauses.push({ line: Number(m[1]), reason: m[2]! });
    }
  }
  return { ok: true, pauses };
}
