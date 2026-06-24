/**
 * rust bridge module (rust-bridge.ts).
 * @module
 */

import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import { existsSync, readFileSync, statSync, unlinkSync, writeFileSync } from "node:fs";
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
  // CliPath.
  //
  // Parameters:
  // None.
  //
  // Returns:
  // `Some` / non-null value on success, otherwise `None` / null.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = cliPath();
  const release = join(repoRoot, "target/release/spanda");
  const debug = join(repoRoot, "target/debug/spanda");
  const cargoTarget = process.env.CARGO_TARGET_DIR;
  const candidates = [
    ...(cargoTarget
      ? [join(cargoTarget, "release/spanda"), join(cargoTarget, "debug/spanda")]
      : []),
    release,
    debug,
  ].filter((p) => existsSync(p));

  // continue when length equals 0.
  if (candidates.length === 0) return null;

  // continue when length equals 1.
  if (candidates.length === 1) return candidates[0]!;
  const newest = candidates.reduce((a, b) =>
    statSync(a).mtimeMs >= statSync(b).mtimeMs ? a : b,
  );
  return newest;
}

export function isCliAvailable(): boolean {
  // IsCliAvailable.
  //
  // Parameters:
  // None.
  //
  // Returns:
  // `true` or `false`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = isCliAvailable();
  return cliPath() !== null;
}

export function checkViaCli(source: string): CheckResult {
  // CheckViaCli.
  //
  // Parameters:
  // - `source` — input value
  //
  // Returns:
  // `CheckResult`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = checkViaCli(source);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    return {
      ok: false,
      diagnostics: [{ message: "Rust CLI not built (run: npm run build:rust)", line: 1, column: 1 }],
    };
  }
  const tmp = join(repoRoot, `.spanda-check-${process.pid}-${Date.now()}.sd`);
  writeFileSync(tmp, source);
  const result = spawnSync(bin, ["check", "--json", tmp], { encoding: "utf-8" });

  // Try the operation and handle failures below.
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }

  // continue when trim is falsy.
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
  // VerifyViaCli.
  //
  // Parameters:
  // - `source` — input value
  // - `args` — optional input
  //
  // Returns:
  // `VerifyResult`.
  //
  // Options:
  // - `args` — optional parameter
  //
  // Example:

  // const result = verifyViaCli(source, args);
  const bin = cliPath();

  // continue when bin is falsy.
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
  const tmp = join(repoRoot, `.spanda-verify-${process.pid}-${Date.now()}.sd`);
  writeFileSync(tmp, source);
  const result = spawnSync(bin, ["verify", tmp, "--json", ...args], { encoding: "utf-8" });

  // Try the operation and handle failures below.
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }

  // continue when trim is falsy.
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
  // RunViaCli.
  //
  // Parameters:
  // - `source` — input value
  //
  // Returns:
  // `RunResult`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = runViaCli(source);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    throw new Error("Rust CLI not built (run: npm run build:rust)");
  }
  const tmp = join(repoRoot, `.spanda-run-${process.pid}-${Date.now()}.sd`);
  writeFileSync(tmp, source);
  const result = spawnSync(bin, ["run", "--json", tmp], { encoding: "utf-8" });

  // Try the operation and handle failures below.
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

  // continue when result is falsy.
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
): SpawnSyncReturns<string> {  // Compute tmp for the following logic.
  const tmp = join(
    repoRoot,
    `${suffix.replace(/\.sd$/, "")}-${process.pid}-${Date.now()}.sd`,
  );
  writeFileSync(tmp, source);
  const result = run(tmp);

  // Try the operation and handle failures below.
  try {
    unlinkSync(tmp);
  } catch {
    /* ignore */
  }
  return result;
}

export function fmtViaCli(source: string): FormatResult {
  // FmtViaCli.
  //
  // Parameters:
  // - `source` — input value
  //
  // Returns:
  // `FormatResult`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = fmtViaCli(source);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    return { ok: false, changed: false, formatted: source };
  }
  const result = withTempSource(source, ".spanda-fmt-tmp.sd", (file) =>
    spawnSync(bin, ["fmt", "--json", file], { encoding: "utf-8" }),
  );

  // continue when trim is falsy.
  if (!result.stdout?.trim()) {
    return { ok: false, changed: false, formatted: source };
  }
  return JSON.parse(result.stdout) as FormatResult;
}

export function lintViaCli(source: string): LintResult {
  // LintViaCli.
  //
  // Parameters:
  // - `source` — input value
  //
  // Returns:
  // `LintResult`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = lintViaCli(source);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    return {
      ok: false,
      issues: [{ rule: "cli", message: "Rust CLI not built", line: 1, column: 1, severity: "error" }],
    };
  }
  const result = withTempSource(source, ".spanda-lint-tmp.sd", (file) =>
    spawnSync(bin, ["lint", "--json", file], { encoding: "utf-8" }),
  );

  // continue when trim is falsy.
  if (!result.stdout?.trim()) {
    return {
      ok: false,
      issues: [{ rule: "cli", message: result.stderr || "lint failed", line: 1, column: 1, severity: "error" }],
    };
  }
  return JSON.parse(result.stdout) as LintResult;
}

export function docViaCli(source: string): DocResult {
  // DocViaCli.
  //
  // Parameters:
  // - `source` — input value
  //
  // Returns:
  // `DocResult`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = docViaCli(source);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    return { ok: false, markdown: "" };
  }
  const result = withTempSource(source, ".spanda-doc-tmp.sd", (file) =>
    spawnSync(bin, ["doc", "--json", file], { encoding: "utf-8" }),
  );

  // continue when trim is falsy.
  if (!result.stdout?.trim()) {
    return { ok: false, markdown: "" };
  }
  const parsed = JSON.parse(result.stdout) as {
    ok: boolean;
    markdown?: string;
    content?: string;
  };
  return {
    ok: parsed.ok,
    markdown: parsed.markdown ?? parsed.content ?? "",
  };
}

export function codegenViaCli(source: string, target: CodegenTarget = "native"): string {
  // CodegenViaCli.
  //
  // Parameters:
  // - `source` — input value
  // - `target` — optional input
  //
  // Returns:
  // Text result.
  //
  // Options:
  // - `target` — optional parameter
  //
  // Example:

  // const result = codegenViaCli(source, target);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    throw new Error("Rust CLI not built (run: npm run build:rust)");
  }
  const result = withTempSource(source, ".spanda-codegen-tmp.sd", (file) =>
    spawnSync(bin, ["codegen", "--target", target, file], { encoding: "utf-8" }),
  );

  // continue when status differs from 0.
  if (result.status !== 0) {
    throw new Error(result.stderr || "codegen failed");
  }
  return result.stdout ?? "";
}

export function deployViaCli(source: string): string {
  // DeployViaCli.
  //
  // Parameters:
  // - `source` — input value
  //
  // Returns:
  // Text result.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = deployViaCli(source);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    throw new Error("Rust CLI not built (run: npm run build:rust)");
  }
  const result = withTempSource(source, ".spanda-deploy-tmp.sd", (file) =>
    spawnSync(bin, ["deploy", "--target", "wasm", file], { encoding: "utf-8" }),
  );

  // continue when status differs from 0.
  if (result.status !== 0) {
    throw new Error(result.stderr || "deploy failed");
  }
  return result.stdout ?? "";
}

export function debugViaCli(source: string, breakpoints: number[] = []): DebugResult {
  // DebugViaCli.
  //
  // Parameters:
  // - `source` — input value
  // - `breakpoints` — optional input
  //
  // Returns:
  // `DebugResult`.
  //
  // Options:
  // - `breakpoints` — optional parameter
  //
  // Example:

  // const result = debugViaCli(source, breakpoints);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    return { ok: false, pauses: [] };
  }
  const args = ["debug", ...breakpoints.flatMap((line) => ["--break", String(line)])];
  const result = withTempSource(source, ".spanda-debug-tmp.sd", (file) =>
    spawnSync(bin, [...args, file], { encoding: "utf-8" }),
  );

  // continue when status differs from 0.
  if (result.status !== 0) {
    return { ok: false, pauses: [] };
  }
  const pauses: DebugPause[] = [];

  // Handle each input line.
  for (const line of (result.stdout ?? "").split("\n")) {
    const m = line.match(/^\s*line (\d+) — (.+)$/);

    // continue when m.
    if (m) {
      pauses.push({ line: Number(m[1]), reason: m[2]! });
    }
  }
  return { ok: true, pauses };
}

export function runNativeCli(args: string[]): SpawnSyncReturns<string> {
  // RunNativeCli.
  //
  // Parameters:
  // - `args` — input value
  //
  // Returns:
  // `SpawnSyncReturns<string>`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = runNativeCli(args);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    return {
      status: 1,
      signal: null,
      output: ["", "Rust CLI not built (run: npm run build:rust)", ""],
      stdout: "",
      stderr: "Rust CLI not built (run: npm run build:rust)",
      pid: 0,
      error: new Error("Rust CLI not built"),
    } as SpawnSyncReturns<string>;
  }
  return spawnSync(bin, args, { encoding: "utf-8" });
}

export function verifyFileViaCli(filePath: string, extraArgs: string[] = []): VerifyResult {
  // VerifyFileViaCli.
  //
  // Parameters:
  // - `filePath` — input value
  // - `extraArgs` — optional input
  //
  // Returns:
  // `VerifyResult`.
  //
  // Options:
  // - `extraArgs` — optional parameter
  //
  // Example:

  // const result = verifyFileViaCli(filePath, extraArgs);
  const source = readFileSync(filePath, "utf-8");
  return verifyViaCli(source, extraArgs);
}

export type SecurityCliReport = {
  findings: Array<{
    severity: string;
    message: string;
    line: number;
    column: number;
  }>;
};

function securityViaCli(source: string, mode: "check" | "audit"): SecurityCliReport {
  // Run native spanda security check or audit against source text.
  const bin = cliPath();
  if (!bin) {
    return {
      findings: [
        {
          severity: "error",
          message: "Rust CLI not built (run: npm run build:rust)",
          line: 1,
          column: 1,
        },
      ],
    };
  }
  const result = withTempSource(source, `.spanda-security-${mode}-tmp.sd`, (file) =>
    spawnSync(bin, ["security", mode, "--json", file], { encoding: "utf-8" }),
  );
  if (!result.stdout?.trim()) {
    return {
      findings: [
        {
          severity: "error",
          message: result.stderr || `security ${mode} failed`,
          line: 1,
          column: 1,
        },
      ],
    };
  }
  return JSON.parse(result.stdout) as SecurityCliReport;
}

export function securityCheckViaCli(source: string): SecurityCliReport {
  return securityViaCli(source, "check");
}

export function securityAuditViaCli(source: string): SecurityCliReport {
  return securityViaCli(source, "audit");
}
