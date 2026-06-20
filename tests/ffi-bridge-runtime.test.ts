import { describe, it, expect } from "vitest";
import { compile, run } from "../src/compile.js";
import { createDefaultSimulator } from "../src/simulator/index.js";
import {
  cppBridgeBinaryPath,
  pythonBridgeScriptPath,
} from "../src/ffi/subprocess-bridge.js";
import { spawnSync } from "node:child_process";

function pythonAvailable(): boolean {
  for (const cmd of ["python3", "python"]) {
    const result = spawnSync(cmd, ["-c", "import sys"], { stdio: "ignore" });
    if (result.status === 0) return true;
  }
  return false;
}

function runSource(source: string): void {
  const { program } = compile(source);
  run(program, { backend: createDefaultSimulator(), maxLoopIterations: 1 });
}

describe("FFI subprocess bridge runtime", () => {
  it("runs extern python fn via subprocess when bridge is available", () => {
    if (!pythonAvailable() || !pythonBridgeScriptPath()) return;
    const source = `
extern python fn py_add(a: Int, b: Int) -> Int;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let sum = py_add(3, 4);
    let _ = sum;
    wheels.stop();
  }
}
`;
    expect(() => runSource(source)).not.toThrow();
  });

  it("runs extern cpp fn via subprocess when binary is available", () => {
    if (!cppBridgeBinaryPath()) return;
    const source = `
extern cpp fn cpp_add(a: Int, b: Int) -> Int;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let sum = cpp_add(3, 4);
    let _ = sum;
    wheels.stop();
  }
}
`;
    expect(() => runSource(source)).not.toThrow();
  });

  it("errors clearly for unknown python extern", () => {
    if (!pythonAvailable() || !pythonBridgeScriptPath()) return;
    const source = `
extern python fn py_missing() -> Int;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let _ = py_missing();
    wheels.stop();
  }
}
`;
    expect(() => runSource(source)).toThrow(/Unknown python extern/);
  });
});
