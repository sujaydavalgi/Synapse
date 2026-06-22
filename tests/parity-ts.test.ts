import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { compile } from "../src/compile.js";

const repoRoot = join(import.meta.dirname, "..");

describe("TypeScript parity checks", () => {
  it("supports robot.in_zone in safety rules", () => {
    const source = readFileSync(join(repoRoot, "examples/showcase/warehouse_robot.sd"), "utf-8");
    expect(() => compile(source, "typescript")).not.toThrow();
  });

  it("parses and type-checks showcase communication syntax", () => {
    const source = readFileSync(join(repoRoot, "examples/showcase/communication_demo.sd"), "utf-8");
    expect(() => compile(source, "typescript")).not.toThrow();
  });

  it("parses and type-checks world_model patrol showcase (Rust + TS parity)", () => {
    const source = readFileSync(join(repoRoot, "examples/showcase/world_model_patrol.sd"), "utf-8");
    expect(() => compile(source, "typescript")).not.toThrow();
  });

  it("parses and type-checks world_model_observe feature example", () => {
    const source = readFileSync(join(repoRoot, "examples/features/world_model_observe.sd"), "utf-8");
    expect(() => compile(source, "typescript")).not.toThrow();
  });

  it("parses and type-checks capability verification example (Rust + TS parity)", () => {
    const source = readFileSync(
      join(repoRoot, "examples/hardware/capability_verification.sd"),
      "utf-8",
    );
    expect(() => compile(source, "typescript")).not.toThrow();
  });

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
