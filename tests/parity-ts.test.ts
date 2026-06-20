import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
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
});
