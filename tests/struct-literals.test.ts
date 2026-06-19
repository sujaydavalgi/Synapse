import { describe, it, expect } from "vitest";
import { compile, run } from "../src/compile.js";
import { TypeCheckError } from "../src/types/index.js";
import { createDefaultSimulator } from "../src/simulator/index.js";

describe("struct literals", () => {
  it("constructs Pose from struct literal and allows field access", () => {
    const source = `
struct Pose {
  x: Distance;
  y: Distance;
  heading: Angle;
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let goal = Pose { x: 1.0 m, y: 2.0 m, heading: 0.5 rad };
    let _x = goal.x;
    wheels.stop();
  }
}
`;
    expect(() => compile(source)).not.toThrow();
    const { program } = compile(source);
    const sim = createDefaultSimulator();
    expect(() => run(program, { backend: sim, maxLoopIterations: 1 })).not.toThrow();
  });

  it("requires all struct fields in a literal", () => {
    const source = `
struct Pose {
  x: Distance;
  y: Distance;
  heading: Angle;
}
robot R { actuator wheels: DifferentialDrive; behavior run() { let p = Pose { x: 1.0 m }; } }
`;
    try {
      compile(source);
      expect.fail("expected type check to fail");
    } catch (err) {
      expect(err).toBeInstanceOf(TypeCheckError);
      const tcErr = err as TypeCheckError;
      expect(tcErr.errors.some((d) => d.message.includes("Missing struct field"))).toBe(true);
    }
  });
});
