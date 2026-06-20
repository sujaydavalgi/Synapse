import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { tokenize } from "../src/lexer/index.js";
import { parse } from "../src/parser/index.js";
import { typeCheck } from "../src/types/index.js";

const repoRoot = join(import.meta.dirname, "..");

const p0Module = readFileSync(join(repoRoot, "examples/modules/path_planning.sd"), "utf-8");
const p1Async = `
module maps;

export async fn get_map() -> Pose {
  return pose(x: 0.0 m, y: 0.0 m, theta: 0.0 rad);
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let map = await get_map();
    let _ = map;
    wheels.stop();
  }
}
`;

const p1Concurrency = `
module comm;

export fn ping() -> Int {
  return 1;
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let ch = channel();
    send(ch, 42);
    select {
      recv(ch) => {
        let _ = 0;
      }
    };
    spawn ping();
    wheels.stop();
  }
}
`;

const p3Extern = readFileSync(join(repoRoot, "examples/std/extern.sd"), "utf-8");

describe("P0–P3 TypeScript mirror", () => {
  it("parses module functions and generics", () => {
    const program = parse(tokenize(p0Module));
    expect(program.moduleName).toBe("navigation.path_planning");
    expect(program.functions.length).toBeGreaterThanOrEqual(2);
    expect(program.functions.some((f) => f.name === "plan_path" && f.visibility === "export")).toBe(true);
    expect(program.functions.some((f) => f.typeParams.includes("T"))).toBe(true);
  });

  it("type-checks module exports", () => {
    const program = parse(tokenize(p0Module));
    expect(() => typeCheck(program)).not.toThrow();
  });

  it("parses async/await", () => {
    const program = parse(tokenize(p1Async));
    expect(program.functions[0]?.isAsync).toBe(true);
    const behavior = program.robots[0]?.behaviors[0];
    const body = behavior?.body ?? [];
    const awaitStmt = body.find((s) => s.kind === "VarDecl" && s.init?.kind === "AwaitExpr");
    expect(awaitStmt).toBeDefined();
  });

  it("parses spawn, select, channel builtins", () => {
    const program = parse(tokenize(p1Concurrency));
    const behavior = program.robots[0]?.behaviors[0]?.body ?? [];
    expect(behavior.some((s) => s.kind === "SelectStmt")).toBe(true);
    expect(behavior.some((s) => s.kind === "SpawnStmt")).toBe(true);
    expect(() => typeCheck(program)).not.toThrow();
  });

  it("parses extern fn and test blocks", () => {
    const program = parse(tokenize(p3Extern));
    expect(program.externFunctions.length).toBe(1);
    expect(program.externFunctions[0]?.library).toBe("libc");
    expect(program.externFunctions[0]?.bridge).toBe("native");
    expect(program.functions.some((f) => f.name === "sum_pair")).toBe(true);
    expect(() => typeCheck(program)).not.toThrow();
  });

  it("parses in-language test blocks", () => {
    const source = `
module math;

export fn double(x: Int) -> Int {
  return x;
}

test "double returns input" {
  assert(true);
}
`;
    const program = parse(tokenize(source));
    expect(program.tests.length).toBe(1);
    expect(program.tests[0]?.name).toBe("double returns input");
    expect(() => typeCheck(program)).not.toThrow();
  });
});
