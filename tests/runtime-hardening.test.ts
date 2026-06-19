import { describe, it, expect } from "vitest";
import { compile, run } from "../src/compile.js";
import { TypeCheckError } from "../src/types/index.js";
import { RuntimeError } from "../src/runtime/interpreter.js";
import { createDefaultSimulator } from "../src/simulator/index.js";

describe("runtime hardening", () => {
  it("rejects unknown twin mirror field at typecheck", () => {
    expect(() =>
      compile(`
        robot R {
          twin T {
            mirror telemetry;
            replay false;
          }
        }
      `),
    ).toThrow(TypeCheckError);
  });

  it("rejects non-exhaustive match against declared enum", () => {
    expect(() =>
      compile(`
        enum Mode {
          Idle,
          Active,
          Stop
        }
        robot R {
          actuator wheels: DifferentialDrive;
          behavior run() {
            let mode = "Idle";
            match mode {
              Idle => wheels.stop();
            };
          }
        }
      `),
    ).toThrow(TypeCheckError);
  });

  it("denies agent sensor read without read capability", () => {
    const source = `
      robot R {
        sensor lidar: Lidar on "/scan";
        actuator wheels: DifferentialDrive;
        safety { max_speed = 1.0 m/s; }
        ai_model planner: LLM {
          provider: "mock";
          model: "test";
          temperature: 0.1;
        }
        agent NoRead {
          uses planner;
          tools [lidar, wheels];
          goal "test";
          can [ propose_motion ];
          plan {
            let scan = lidar.read();
            let proposal = planner.reason(prompt: "go", input: scan);
            let action = safety.validate(proposal);
            wheels.execute(action);
          }
        }
        behavior run() { NoRead.plan(); }
      }
    `;
    const { program } = compile(source);
    const sim = createDefaultSimulator();
    expect(() =>
      run(program, { backend: sim, maxLoopIterations: 1 }),
    ).toThrow(/lacks capability read\(lidar\)/);
  });

  it("denies agent actuator execute without propose_motion capability", () => {
    const source = `
      robot R {
        sensor lidar: Lidar on "/scan";
        actuator wheels: DifferentialDrive;
        safety { max_speed = 1.0 m/s; }
        ai_model planner: LLM {
          provider: "mock";
          model: "test";
          temperature: 0.1;
        }
        agent NoMotion {
          uses planner;
          tools [lidar, wheels];
          goal "test";
          can [ read(lidar) ];
          plan {
            let scan = lidar.read();
            let proposal = planner.reason(prompt: "go", input: scan);
            let action = safety.validate(proposal);
            wheels.execute(action);
          }
        }
        behavior run() { NoMotion.plan(); }
      }
    `;
    const { program } = compile(source);
    const sim = createDefaultSimulator();
    expect(() =>
      run(program, { backend: sim, maxLoopIterations: 1 }),
    ).toThrow(/lacks capability propose_motion/);
  });

  it("aborts behavior when requires contract fails", () => {
    const source = `
      robot R {
        sensor lidar: Lidar on "/scan";
        actuator wheels: DifferentialDrive;
        behavior move() requires lidar.nearest_distance > 100.0 m {
          wheels.drive(linear: 0.2 m/s, angular: 0.0 rad/s);
        }
      }
    `;
    const { program } = compile(source);
    const sim = createDefaultSimulator();
    expect(() =>
      run(program, { backend: sim, maxLoopIterations: 1 }),
    ).toThrow(RuntimeError);
  });

  it("aborts behavior when ensures contract fails", () => {
    const source = `
      robot R {
        sensor lidar: Lidar on "/scan";
        actuator wheels: DifferentialDrive;
        behavior move() requires true ensures lidar.nearest_distance > 100.0 m {
          wheels.drive(linear: 0.2 m/s, angular: 0.0 rad/s);
        }
      }
    `;
    const { program } = compile(source);
    const sim = createDefaultSimulator();
    expect(() => run(program, { backend: sim, maxLoopIterations: 1 })).toThrow(/ensures contract failed/);
  });

  it("skips task iterations when requires contract fails", () => {
    const source = `
      robot R {
        sensor lidar: Lidar on "/scan";
        actuator wheels: DifferentialDrive;
        task tick every 20ms requires lidar.nearest_distance > 100.0 m {
          wheels.drive(linear: 0.5 m/s, angular: 0.0 rad/s);
        }
      }
    `;
    const { program } = compile(source);
    const sim = createDefaultSimulator();
    const logs: string[] = [];
    const state = run(program, {
      backend: sim,
      maxLoopIterations: 3,
      onLog: (msg) => logs.push(msg),
    });
    expect(logs.some((l) => l.includes("task requires contract failed"))).toBe(true);
    expect(state.velocity.linear).toBe(0);
  });

  it("aborts task when ensures contract fails", () => {
    const source = `
      robot R {
        sensor lidar: Lidar on "/scan";
        actuator wheels: DifferentialDrive;
        task tick every 20ms requires true ensures lidar.nearest_distance > 100.0 m {
          wheels.stop();
        }
      }
    `;
    const { program } = compile(source);
    const sim = createDefaultSimulator();
    expect(() => run(program, { backend: sim, maxLoopIterations: 1 })).toThrow(/task ensures contract failed/);
  });
});
