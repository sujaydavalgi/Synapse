import { describe, it, expect } from "vitest";
import { compile, run } from "../src/compile.js";
import { createDefaultSimulator } from "../src/simulator/index.js";

describe("twin replay API", () => {
  it("accumulates replay frames during task loops", () => {
    const source = `
      robot R {
        actuator wheels: DifferentialDrive;
        twin Shadow { mirror pose; replay true; }
        task sync every 50ms {
          wheels.drive(linear: 0.2 m/s, angular: 0.0 rad/s);
          let frames = Shadow.frame_count();
          if frames < 1 { wheels.stop(); }
        }
      }
    `;
    const { program } = compile(source);
    const logs: string[] = [];
    run(program, {
      backend: createDefaultSimulator(),
      maxLoopIterations: 3,
      onLog: (msg) => logs.push(msg),
    });
    expect(logs.some((l) => l.includes("replay frames=3"))).toBe(true);
  });

  it("returns current shadow pose via twin.pose()", () => {
    const source = `
      robot R {
        actuator wheels: DifferentialDrive;
        twin Shadow { mirror pose; replay true; }
        task sync every 50ms {
          wheels.drive(linear: 0.3 m/s, angular: 0.0 rad/s);
          let shadow_pose = Shadow.pose();
          let _x = shadow_pose.x;
        }
      }
    `;
    const { program } = compile(source);
    expect(() =>
      run(program, { backend: createDefaultSimulator(), maxLoopIterations: 2 }),
    ).not.toThrow();
  });

  it("reads historical frames with twin.replay()", () => {
    const source = `
      robot R {
        actuator wheels: DifferentialDrive;
        twin Shadow { mirror pose; replay true; }
        task sync every 50ms {
          wheels.drive(linear: 0.3 m/s, angular: 0.0 rad/s);
          if Shadow.frame_count() >= 2 {
            let first = Shadow.replay(index: 0, field: pose);
            let _x = first.x;
          }
        }
      }
    `;
    const { program } = compile(source);
    expect(() =>
      run(program, { backend: createDefaultSimulator(), maxLoopIterations: 3 }),
    ).not.toThrow();
  });
});
