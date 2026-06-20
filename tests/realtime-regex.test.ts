import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { compile } from "../src/compile.js";
import {
  validateTaskTiming,
  validatePipeline,
  validateWatchdog,
  validateRecover,
} from "../src/reliability.js";
import { compileRegex, regexMatches, regexFromLexeme } from "../src/regex.js";
import {
  createMissionTrace,
  recordTraceFrame,
  verifyTraces,
  parseReplayOffset,
} from "../src/replay.js";

const repoRoot = join(import.meta.dirname, "..");
const span = {
  start: { line: 1, column: 1, offset: 0 },
  end: { line: 1, column: 1, offset: 0 },
};

describe("reliability validation", () => {
  it("rejects deadline greater than period", () => {
    const diags = validateTaskTiming({
      kind: "TaskDecl",
      name: "t",
      priority: "normal",
      intervalMs: 5,
      deadlineMs: 10,
      requires: null,
      ensures: null,
      invariant: null,
      budget: null,
      body: [],
      span,
    });
    expect(diags.some((d) => d.message.includes("deadline"))).toBe(true);
  });

  it("rejects invalid pipeline budget", () => {
    const diags = validatePipeline({
      kind: "PipelineDecl",
      name: "p",
      budgetMs: 0,
      body: [],
      span,
    });
    expect(diags.length).toBeGreaterThan(0);
  });

  it("rejects watchdog with unknown target", () => {
    const diags = validateWatchdog(
      {
        kind: "WatchdogDecl",
        name: "w",
        target: "missing",
        timeoutMs: 10,
        body: [],
        span,
      },
      ["other"],
    );
    expect(diags.some((d) => d.message.includes("target task"))).toBe(true);
  });

  it("requires safe action for RuntimeError recovery", () => {
    const diags = validateRecover({
      kind: "RecoverDecl",
      errorName: "RuntimeError",
      body: [{ kind: "EmergencyStopStmt", span }],
      span,
    });
    expect(diags.length).toBeGreaterThan(0);
  });
});

describe("regex module", () => {
  it("compiles and matches patterns", () => {
    const pattern = regexFromLexeme("/^robot-[0-9]+$/", span);
    compileRegex(pattern);
    expect(regexMatches(pattern, "robot-123")).toBe(true);
    expect(regexMatches(pattern, "bad")).toBe(false);
  });

  it("rejects unsupported flags", () => {
    const pattern = regexFromLexeme("/abc/x", span);
    expect(() => compileRegex(pattern)).toThrow(/Invalid regex flag/);
  });
});

describe("replay module", () => {
  it("verifies matching traces", () => {
    const expected = createMissionTrace("demo.sd");
    recordTraceFrame(expected, 0, "tick", {});
    const actual = createMissionTrace("demo.sd");
    recordTraceFrame(actual, 0, "tick", {});
    const report = verifyTraces(expected, actual, 0);
    expect(report.ok).toBe(true);
  });

  it("parses replay offsets", () => {
    expect(parseReplayOffset("1500")).toBe(1500);
    expect(parseReplayOffset("T+01:30")).toBe(90000);
  });
});

describe("realtime and regex example compile checks", () => {
  for (const dir of ["realtime", "regex"] as const) {
    const examplesDir = join(repoRoot, "examples", dir);
    for (const file of readdirSync(examplesDir).filter((name) => name.endsWith(".sd"))) {
      it(`compiles examples/${dir}/${file}`, () => {
        const source = readFileSync(join(examplesDir, file), "utf-8");
        expect(() => compile(source, "typescript")).not.toThrow();
      });
    }
  }
});
