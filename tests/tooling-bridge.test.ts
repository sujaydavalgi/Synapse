import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import {
  codegenViaCli,
  deployViaCli,
  docViaCli,
  fmtViaCli,
  isCliAvailable,
  lintViaCli,
} from "../src/tooling/index.js";

const repoRoot = join(import.meta.dirname, "..");
const demoSource = readFileSync(join(repoRoot, "examples/std/extern.sd"), "utf-8");

describe("tooling CLI bridge", () => {
  it.skipIf(!isCliAvailable())("formats source via Rust CLI", () => {
    const result = fmtViaCli(demoSource);
    expect(result.ok).toBe(true);
    expect(result.formatted.length).toBeGreaterThan(0);
  });

  it.skipIf(!isCliAvailable())("lints source via Rust CLI", () => {
    const result = lintViaCli(demoSource);
    expect(result.ok).toBe(true);
  });

  it.skipIf(!isCliAvailable())("generates docs via Rust CLI", () => {
    const result = docViaCli(demoSource);
    expect(result.ok).toBe(true);
    expect(result.markdown).toContain("native_bridge");
  });

  it.skipIf(!isCliAvailable())("codegen native output", () => {
    const output = codegenViaCli(demoSource, "native");
    expect(output).toContain("spanda_main");
  });

  it.skipIf(!isCliAvailable())("wasm deploy manifest", () => {
    const manifest = deployViaCli(demoSource);
    expect(manifest).toContain('"target": "wasm"');
  });
});
