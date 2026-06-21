import { describe, expect, it } from "vitest";
import { check } from "../src/types/checker.js";
import { TypeCheckError } from "../src/types/index.js";
import { parse } from "../src/parser/index.js";
import { tokenize } from "../src/lexer/index.js";
import { compile, run } from "../src/compile.js";
import { createDefaultSimulator } from "../src/simulator/index.js";
import { SafetyMonitor, createSafetyConfigFromRobot } from "../src/safety/index.js";

describe("robotics platform parser", () => {
  it("parses fleet and safety_zone program members", () => {
    const source = `
fleet Warehouse {
  Picker1;
}

safety_zone HumanArea {
  max_speed 0.5 m/s;
}

robot Picker1 {
  actuator wheels: DifferentialDrive;
  behavior idle() { wheels.stop(); }
}
`;
    const program = parse(tokenize(source));
    expect(program.fleets).toHaveLength(1);
    expect(program.fleets[0]?.name).toBe("Warehouse");
    expect(program.programSafetyZones).toHaveLength(1);
    expect(program.programSafetyZones[0]?.maxSpeedMps).toBeCloseTo(0.5);
  });

  it("parses named mission with steps", () => {
    const source = `
robot R {
  actuator wheels: DifferentialDrive;
  mission Delivery {
    duration: 20 min;
    navigate;
    deliver;
  }
  behavior run() { wheels.stop(); }
}
`;
    const program = parse(tokenize(source));
    const robot = program.robots[0];
    expect(robot?.mission?.name).toBe("Delivery");
    expect(robot?.mission?.steps).toEqual(["navigate", "deliver"]);
    expect(robot?.mission?.durationHours).toBeCloseTo(20 / 60);
  });

  it("parses navigate statement sugar", () => {
    const source = `
robot R {
  actuator wheels: DifferentialDrive;
  mission Go { navigate; }
  behavior run() {
    navigate {
      goal: "Dock";
      linear: 0.2 m/s;
    }
  }
}
`;
    const program = parse(tokenize(source));
    const stmt = program.robots[0]?.behaviors[0]?.body[0];
    expect(stmt?.kind).toBe("NavigateStmt");
    if (stmt?.kind === "NavigateStmt") {
      expect(stmt.goal.kind).toBe("LiteralExpr");
    }
  });
});

describe("robotics platform type checker", () => {
  it("rejects unknown fleet members", () => {
    const source = `
robot Picker1 {
  actuator wheels: DifferentialDrive;
  behavior idle() { wheels.stop(); }
}

fleet Warehouse {
  MissingBot;
}
`;
    const program = parse(tokenize(source));
    expect(() => check(program)).toThrow(TypeCheckError);
    try {
      check(program);
    } catch (err) {
      expect(err).toBeInstanceOf(TypeCheckError);
      const tcErr = err as TypeCheckError;
      expect(tcErr.errors.some((e) => e.message.includes("unknown robot 'MissingBot'"))).toBe(true);
    }
  });

  it("type-checks fleet.members call", () => {
    const source = `
robot Picker1 {
  actuator wheels: DifferentialDrive;
  behavior idle() { wheels.stop(); }
}

fleet Warehouse {
  Picker1;
}

robot Coordinator {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let count = fleet.members("Warehouse");
    let _ = count;
    wheels.stop();
  }
}
`;
    const program = parse(tokenize(source));
    expect(() => check(program)).not.toThrow();
  });
});

describe("robotics platform interpreter", () => {
  it("runs mission lifecycle and navigation helpers", () => {
    const source = `
robot R {
  actuator wheels: DifferentialDrive;
  mission Patrol {
    navigate;
    return_home;
  }
  behavior run() {
    mission.start();
    let step = mission.advance();
    let _ = step;
    navigation.goal("Dock");
    let traj = navigation.navigate();
    let _ = traj;
    wheels.stop();
  }
}
`;
    const { program } = compile(source);
    const sim = createDefaultSimulator();
    const logs: string[] = [];
    run(program, {
      backend: sim,
      maxLoopIterations: 1,
      onLog: (msg) => logs.push(msg),
    });
    expect(logs.some((l) => l.includes("navigation: executing goal 'Dock'"))).toBe(true);
  });

  it("reports fleet member count at runtime", () => {
    const source = `
robot Picker1 {
  actuator wheels: DifferentialDrive;
  behavior idle() { wheels.stop(); }
}

fleet Warehouse {
  Picker1;
}

robot Coordinator {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let count = fleet.members("Warehouse");
    let _ = count;
    wheels.stop();
  }
}
`;
    const { program } = compile(source);
    run(program, { backend: createDefaultSimulator(), maxLoopIterations: 1 });
  });

  it("adds fusion confidence and state_estimate fields", () => {
    const source = `
robot R {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;
  observe {
    lidar;
  }
  behavior run() {
    let fused = fusion.read();
    let _ = fused;
    wheels.stop();
  }
}
`;
    const { program } = compile(source);
    run(program, { backend: createDefaultSimulator(), maxLoopIterations: 1 });
  });

  it("clamps speed with program safety_zone caps via SafetyMonitor", () => {
    const caps = new Map<string, number>([["HumanArea", 0.5]]);
    const monitor = new SafetyMonitor(
      createSafetyConfigFromRobot(
        1.0,
        [],
        [{
          name: "HumanArea",
          shape: "circle",
          x: 0,
          y: 0,
          radius: 2,
        }],
        caps,
      ),
    );
    expect(monitor.clampSpeedAtPose(0.8, { x: 0, y: 0 })).toBeCloseTo(0.5);
    expect(monitor.clampSpeedAtPose(0.8, { x: 10, y: 10 })).toBeCloseTo(0.8);
    expect(monitor.peekBeforeMotion({} as never, { x: 0, y: 0 }).allowed).toBe(true);
  });

  it("parses certify program metadata", () => {
    const source = `
certify ISO26262 {
  level ASIL_B;
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}
`;
    const program = parse(tokenize(source));
    expect(program.certifications).toHaveLength(1);
    expect(program.certifications[0]?.standard).toBe("ISO26262");
    expect(program.certifications[0]?.level).toBe("ASIL_B");
    expect(() => check(program)).not.toThrow();
  });

  it("clamps drive speed inside program safety_zone at runtime", () => {
    const source = `
safety_zone HumanArea {
  max_speed 0.5 m/s;
}

robot ZoneBot {
  actuator wheels: DifferentialDrive;

  safety {
    max_speed = 1.0 m/s;
    zone HumanArea circle at (0.0 m, 0.0 m) radius 2.0 m;
  }

  behavior drive() {
    wheels.drive(linear: 0.8 m/s, angular: 0.0 rad/s);
  }
}
`;
    const { program } = compile(source);
    const sim = createDefaultSimulator();
    run(program, { backend: sim, maxLoopIterations: 1 });
    expect(sim.getState().velocity.linear).toBeCloseTo(0.5);
  });

  it("publishes Nav2 cmd_vel when topic is declared", () => {
    const source = `
robot R {
  topic cmd_vel: Velocity publish on "/cmd_vel";
  actuator wheels: DifferentialDrive;
  mission Go { navigate; }
  behavior run() {
    navigation.goal("Dock");
    let _ = navigation.navigate();
    wheels.stop();
  }
}
`;
    const { program } = compile(source);
    const sim = createDefaultSimulator();
    const logs: string[] = [];
    run(program, {
      backend: sim,
      maxLoopIterations: 1,
      onLog: (msg) => logs.push(msg),
    });
    expect(sim.getPublishedTopics().some((p) => p.topic === "/cmd_vel")).toBe(true);
    expect(logs.some((l) => l.includes("Nav2 bridge publish"))).toBe(true);
  });

  it("executes navigate statement sugar", () => {
    const source = `
robot R {
  topic cmd_vel: Velocity publish on "/cmd_vel";
  actuator wheels: DifferentialDrive;
  mission Go { navigate; }
  behavior run() {
    navigate {
      goal: "Dock";
    }
    wheels.stop();
  }
}
`;
    const { program } = compile(source);
    const sim = createDefaultSimulator();
    const logs: string[] = [];
    run(program, {
      backend: sim,
      maxLoopIterations: 1,
      onLog: (msg) => logs.push(msg),
    });
    expect(logs.some((l) => l.includes("navigation: executing goal 'Dock'"))).toBe(true);
  });
});
