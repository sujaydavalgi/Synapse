import { describe, it, expect } from "vitest";
import { compile, run } from "../src/compile.js";
import { TypeCheckError } from "../src/types/index.js";
import { createDefaultSimulator } from "../src/simulator/index.js";

describe("trait impl", () => {
  it("binds agent method to trait impl body", () => {
    const source = `
struct Pose {
  x: Distance;
  y: Distance;
  heading: Angle;
}

trait Navigator {
  fn plan(goal: Pose) -> Path;
}

robot R {
  actuator wheels: DifferentialDrive;

  agent Nav {
    tools [wheels];
    goal "Navigate";
    plan { wheels.stop(); }
  }

  impl Navigator for Nav {
    fn plan(goal: Pose) -> Path {
      wheels.stop();
    }
  }

  behavior run() {
    Nav.plan(Pose { x: 0.0 m, y: 0.0 m, heading: 0.0 rad });
  }
}
`;
    expect(() => compile(source)).not.toThrow();
    const { program } = compile(source);
    const sim = createDefaultSimulator();
    expect(() => run(program, { backend: sim, maxLoopIterations: 1 })).not.toThrow();
  });

  it("rejects trait impl for unknown trait", () => {
    const source = `
robot R {
  actuator wheels: DifferentialDrive;
  agent Nav { tools [wheels]; goal "x"; plan { wheels.stop(); } }
  impl Missing for Nav { fn plan(goal: Pose) -> Path { wheels.stop(); } }
}
`;
    try {
      compile(source);
      expect.fail("expected type check to fail");
    } catch (err) {
      expect(err).toBeInstanceOf(TypeCheckError);
      const tcErr = err as TypeCheckError;
      expect(tcErr.errors.some((d) => d.message.includes("Unknown trait"))).toBe(true);
    }
  });
});
