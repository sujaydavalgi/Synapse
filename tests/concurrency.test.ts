import { describe, expect, it } from "vitest";
import { compile, run } from "../src/compile.js";
import { createDefaultSimulator } from "../src/simulator/index.js";

function compileAndRun(source: string): string[] {
  const { program } = compile(source);
  const logs: string[] = [];
  run(program, {
    backend: createDefaultSimulator(),
    maxLoopIterations: 5,
    onLog: (msg) => logs.push(msg),
  });
  return logs;
}

describe("TS concurrency runtime", () => {
  it("runs spawn channel select", () => {
    const source = `
module comm;
export fn ping() -> Int { return 1; }

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let ch = channel();
    send(ch, 42);
    select {
      recv(ch) => {
        wheels.stop();
      }
    };
    spawn ping();
  }
}
`;
    const logs = compileAndRun(source);
    expect(logs.some((l) => l.includes("spawn ping"))).toBe(true);
  });

  it("runs parallel block with spawn handles", () => {
    const source = `
module comm;
export fn perception() -> Int { return 1; }
export fn planning() -> Int { return 2; }

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    parallel {
      let a = spawn perception();
      let b = spawn planning();
    };
    let results = _parallel;
    let _ = results;
    wheels.stop();
  }
}
`;
    const logs = compileAndRun(source);
    expect(logs.some((l) => l.includes("parallel: executing"))).toBe(true);
    expect(logs.some((l) => l.includes("parallel: aggregated"))).toBe(true);
  });

  it("parses task priority without every", () => {
    const source = `
robot R {
  actuator wheels: DifferentialDrive;
  task SafetyMonitor critical {
    wheels.stop();
  }
}
`;
    const { program } = compile(source);
    expect(program.robots[0]?.tasks[0]?.priority).toBe("critical");
    expect(program.robots[0]?.tasks[0]?.intervalMs).toBe(10);
  });
});
