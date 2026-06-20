import { describe, expect, it } from "vitest";
import { compile, run } from "../src/compile.js";
import { createDefaultSimulator } from "../src/simulator/index.js";
import { isCliAvailable, lintViaCli } from "../src/rust-bridge.js";

function compileAndRun(source: string, maxLoopIterations = 5): string[] {
  const { program } = compile(source);
  const logs: string[] = [];
  run(program, {
    backend: createDefaultSimulator(),
    maxLoopIterations,
    onLog: (msg) => logs.push(msg),
  });
  return logs;
}

describe("TS concurrency extended", () => {
  it("agent mailbox send and recv in plan", () => {
    const source = `
robot R {
  actuator wheels: DifferentialDrive;
  agent Vision {
    goal "see";
    plan {
      send_agent("Planner", 1);
    }
  }
  agent Planner {
    goal "plan";
    plan {
      let msg = recv_agent();
      let _ = msg;
      wheels.stop();
    }
  }
  Vision -> Planner;
  behavior run() {
    Vision.plan();
    Planner.plan();
  }
}
`;
    const logs = compileAndRun(source);
    expect(logs.some((l) => l.includes("send_agent Vision -> Planner"))).toBe(true);
  });

  it("peer_send delivers to subscriber", () => {
    const source = `
robot FleetBot {
  bus local;
  robot RoverA;
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  behavior run() {
    subscribe RoverA.pose;
    peer_send("RoverA", "pose", pose(x: 1.0 m, y: 2.0 m, theta: 0.0 rad));
    receive RoverA.pose to p;
    wheels.stop();
  }
}
`;
    const logs = compileAndRun(source);
    expect(logs.some((l) => l.includes("peer_send RoverA.pose"))).toBe(true);
  });

  it("runtime budget skips over-budget task", () => {
    const source = `
robot R {
  actuator wheels: DifferentialDrive;
  task heavy low every 10ms {
    budget {
      cpu <= 1%;
    }
    wheels.drive(linear: 0.5 m/s, angular: 0.0 rad/s);
  }
  task heavy2 low every 10ms {
    budget {
      cpu <= 1%;
    }
    wheels.drive(linear: 0.5 m/s, angular: 0.0 rad/s);
  }
}
`;
    const logs = compileAndRun(source, 4);
    expect(logs.some((l) => l.includes("budget exceeded"))).toBe(true);
  });

  it.skipIf(!isCliAvailable())("lint warns recv without send", () => {
    const source = `
module m;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let ch = channel();
    select {
      recv(ch) => {
        wheels.stop();
      }
    };
  }
}
`;
    const report = lintViaCli(source);
    expect(report.issues.some((i) => i.rule === "channel-recv-without-send")).toBe(true);
  });
});
