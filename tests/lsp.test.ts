import { describe, it, expect } from "vitest";
import { spawnSync } from "node:child_process";
import { existsSync, writeFileSync, unlinkSync } from "node:fs";
import { join } from "node:path";

const repoRoot = join(import.meta.dirname, "..");
const cliRelease = join(repoRoot, "target/release/spanda");
const cliDebug = join(repoRoot, "target/debug/spanda");
const cli = existsSync(cliRelease) ? cliRelease : existsSync(cliDebug) ? cliDebug : null;

describe("LSP diagnostics via Rust CLI", () => {
  it("returns JSON diagnostics for invalid program", () => {
    if (!cli) {
      console.warn("Skipping LSP test — build Rust CLI with: npm run build:rust");
      return;
    }

    const tmp = join(repoRoot, ".spanda-lsp-test.sd");
    writeFileSync(
      tmp,
      `robot R { actuator wheels: DifferentialDrive; behavior run() { let x = unknown_var; } }`,
    );
    const result = spawnSync(cli, ["check", "--json", tmp], { encoding: "utf-8" });
    try {
      unlinkSync(tmp);
    } catch {
      /* ignore */
    }

    expect(result.stdout.trim().length).toBeGreaterThan(0);
    const parsed = JSON.parse(result.stdout) as { ok: boolean; diagnostics?: Array<{ message: string }> };
    expect(parsed.ok).toBe(false);
    expect(parsed.diagnostics?.length).toBeGreaterThan(0);
  });

  it("returns ok for valid program", () => {
    if (!cli) return;

    const tmp = join(repoRoot, ".spanda-lsp-test-valid.sd");
    writeFileSync(
      tmp,
      `robot R { actuator wheels: DifferentialDrive; behavior run() { wheels.stop(); } }`,
    );
    const result = spawnSync(cli, ["check", "--json", tmp], { encoding: "utf-8" });
    try {
      unlinkSync(tmp);
    } catch {
      /* ignore */
    }

    const parsed = JSON.parse(result.stdout) as { ok: boolean };
    expect(parsed.ok).toBe(true);
  });
});
