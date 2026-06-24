/**
 * cli bridge module (cli-bridge.ts).
 * @module
 */

import { spawnSync } from "node:child_process";
import { existsSync, unlinkSync, writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import type { CheckResult, RunResult } from "./index.js";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "../..");

function cliPath(): string | null {
  // Description:
  //     CliPath.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: string | null
  //         Return value from `cliPath`.
  //
  // Example:
  //     const result = cliPath();
  // Description:
  //     CliPath.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: string | null
  //         Return value from `cliPath`.
  //
  // Example:
  //     const result = cliPath();

  // const result = cliPath();
  const release = join(repoRoot, "target/release/spanda");
  const debug = join(repoRoot, "target/debug/spanda");

  // continue when existsSync(release).
  if (existsSync(release)) return release;

  // continue when existsSync(debug).
  if (existsSync(debug)) return debug;
  return null;
}

export function isCliAvailable(): boolean {
  // Description:
  //     IsCliAvailable.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `isCliAvailable`.
  //
  // Example:
  //     const result = isCliAvailable();
  // Description:
  //     IsCliAvailable.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: boolean
  //         Return value from `isCliAvailable`.
  //
  // Example:
  //     const result = isCliAvailable();

  // const result = isCliAvailable();
  return cliPath() !== null;
}

export function checkViaCli(source: string): CheckResult {
  // Description:
  //     CheckViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: CheckResult
  //         Return value from `checkViaCli`.
  //
  // Example:
  //     const result = checkViaCli(source);
  // Description:
  //     CheckViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: CheckResult
  //         Return value from `checkViaCli`.
  //
  // Example:
  //     const result = checkViaCli(source);

  // const result = checkViaCli(source);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    return { ok: false, diagnostics: [{ message: "Rust CLI not built (run: cargo build -p spanda-cli)", line: 1, column: 1 }] };
  }
  const tmp = join(repoRoot, ".spanda-check-tmp.sd");
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

export function runViaCli(source: string): RunResult {
  // Description:
  //     RunViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: RunResult
  //         Return value from `runViaCli`.
  //
  // Example:
  //     const result = runViaCli(source);
  // Description:
  //     RunViaCli.
  //
  // Inputs:
  //     source: string
  //         Caller-supplied source.
  //
  // Outputs:
  //     result: RunResult
  //         Return value from `runViaCli`.
  //
  // Example:
  //     const result = runViaCli(source);

  // const result = runViaCli(source);
  const bin = cliPath();

  // continue when bin is falsy.
  if (!bin) {
    throw new Error("Rust CLI not built (run: cargo build -p spanda-cli)");
  }
  const tmp = join(repoRoot, ".spanda-run-tmp.sd");
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
    diagnostics?: CheckResult["diagnostics"];
  };

  // continue when result is falsy.
  if (!parsed.ok || !parsed.result) {
    throw new Error(parsed.diagnostics?.[0]?.message ?? "Run failed");
  }
  return parsed.result;
}
