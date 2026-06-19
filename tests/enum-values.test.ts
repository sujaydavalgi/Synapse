import { describe, it, expect } from "vitest";
import { compile, run } from "../src/compile.js";
import { TypeCheckError } from "../src/types/index.js";
import { createDefaultSimulator } from "../src/simulator/index.js";

describe("enum values", () => {
  it("matches on unqualified enum variants", () => {
    const source = `
enum RobotState {
  Idle,
  Navigating,
  EmergencyStop
}

robot Rover {
  actuator wheels: DifferentialDrive;

  behavior run() {
    let state = Idle;
    match state {
      Idle => wheels.stop();
      Navigating => wheels.drive(linear: 0.3 m/s, angular: 0.0 rad/s);
      EmergencyStop => emergency_stop;
    };
  }
}
`;
    expect(() => compile(source)).not.toThrow();
    const { program } = compile(source);
    const sim = createDefaultSimulator();
    expect(() => run(program, { backend: sim, maxLoopIterations: 1 })).not.toThrow();
  });

  it("accepts qualified enum variant references", () => {
    const source = `
enum Mode {
  Idle,
  Active
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let state = Mode.Active;
    match state {
      Idle => wheels.stop();
      Active => wheels.drive(linear: 0.1 m/s, angular: 0.0 rad/s);
    };
  }
}
`;
    expect(() => compile(source)).not.toThrow();
  });

  it("rejects duplicate enum variant names across enums", () => {
    const source = `
enum A { Idle, Go }
enum B { Idle, Stop }
robot R { actuator wheels: DifferentialDrive; }
`;
    try {
      compile(source);
      expect.fail("expected type check to fail");
    } catch (err) {
      expect(err).toBeInstanceOf(TypeCheckError);
      const tcErr = err as TypeCheckError;
      expect(
        tcErr.errors.some((d) => d.message.includes("Enum variant 'Idle' already declared")),
      ).toBe(true);
    }
  });
});
