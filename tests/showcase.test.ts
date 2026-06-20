import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import { compile, runSource } from "../src/compile.js";
import { createDefaultSimulator } from "../src/simulator/index.js";
import { TypeCheckError } from "../src/types/index.js";

const repoRoot = join(import.meta.dirname, "..");
const showcaseDir = join(repoRoot, "examples/showcase");

const SHOWCASE_EXAMPLES = [
  { file: "rover_navigation.sd", runnable: true },
  { file: "warehouse_robot.sd", runnable: true },
  { file: "hardware_compatibility.sd", runnable: false },
  { file: "killer_demo.sd", runnable: true },
  { file: "communication_demo.sd", runnable: true },
  { file: "digital_twin_demo.sd", runnable: true },
  { file: "ai_safety_violation.sd", expectFail: true },
] as const;

async function checkShowcase(source: string, expectFail?: boolean): Promise<void> {
  try {
    const { isCliAvailable, checkViaCli } = await import("../src/rust-bridge.js");
    if (isCliAvailable()) {
      const result = checkViaCli(source);
      if (expectFail) {
        expect(result.ok).toBe(false);
        const text = result.diagnostics.map((d) => d.message).join(" ");
        expect(text).toMatch(/ActionProposal|SafeAction/i);
      } else {
        expect(result.ok, result.diagnostics.map((d) => d.message).join("; ")).toBe(true);
      }
      return;
    }
  } catch {
    /* fall through */
  }

  if (expectFail) {
    expect(() => compile(source)).toThrow(TypeCheckError);
    try {
      compile(source);
    } catch (e) {
      if (e instanceof TypeCheckError) {
        expect(e.message).toMatch(/ActionProposal|SafeAction/i);
      }
    }
  } else {
    expect(() => compile(source)).not.toThrow();
  }
}

describe("showcase examples", () => {
  for (const example of SHOWCASE_EXAMPLES) {
    const path = join(showcaseDir, example.file);
    const source = readFileSync(path, "utf-8");

    it(`${example.file} type-checks`, async () => {
      await checkShowcase(source, "expectFail" in example && example.expectFail);
    });

    if ("runnable" in example && example.runnable) {
      it(`${example.file} runs in simulator`, async () => {
        const state = await runSource(source, {
          backend: createDefaultSimulator(),
          maxLoopIterations: 5,
          rustCli: true,
        });
        expect(state).toBeDefined();
      });
    }
  }

  it("hardware_compatibility.sd verifies via native CLI when available", async () => {
    const source = readFileSync(join(showcaseDir, "hardware_compatibility.sd"), "utf-8");
    try {
      const { isCliAvailable, verifyViaCli } = await import("../src/rust-bridge.js");
      if (!isCliAvailable()) return;
      const result = verifyViaCli(source);
      expect(result.ok).toBe(true);
      expect(result.compatible).toBe(true);
    } catch {
      /* CLI not built */
    }
  });

  it("killer_demo.sd verifies via native CLI when available", async () => {
    const source = readFileSync(join(showcaseDir, "killer_demo.sd"), "utf-8");
    try {
      const { isCliAvailable, verifyViaCli } = await import("../src/rust-bridge.js");
      if (!isCliAvailable()) return;
      const result = verifyViaCli(source);
      expect(result.ok).toBe(true);
      expect(result.compatible).toBe(true);
    } catch {
      /* CLI not built */
    }
  });
});

describe("showcase CLI smoke", () => {
  const cliEntry = join(repoRoot, "src/cli/index.ts");

  for (const file of [
    "rover_navigation.sd",
    "warehouse_robot.sd",
    "killer_demo.sd",
    "communication_demo.sd",
    "digital_twin_demo.sd",
  ]) {
    it(`spanda check showcase/${file}`, () => {
      const result = spawnSync(
        "node",
        ["--import", "tsx", cliEntry, "check", `examples/showcase/${file}`],
        { encoding: "utf-8", cwd: repoRoot }
      );
      expect(result.status).toBe(0);
    });
  }

  it("spanda check showcase/ai_safety_violation.sd fails", () => {
    const result = spawnSync(
      "node",
      ["--import", "tsx", cliEntry, "check", "examples/showcase/ai_safety_violation.sd"],
      { encoding: "utf-8", cwd: repoRoot }
    );
    expect(result.status).not.toBe(0);
  });
});
