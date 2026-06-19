import { describe, it, expect } from "vitest";
import { compile } from "../src/compile.js";
import { TypeCheckError } from "../src/types/index.js";

const MODULE_STRUCT_ENUM_TRAIT = `
module navigation;

import sensors.lidar;
import motion.drive;

struct Pose {
  x: Distance;
  y: Distance;
  heading: Angle;
}

enum RobotState {
  Idle,
  Navigating,
  EmergencyStop
}

trait Navigator {
  fn plan(goal: Pose) -> Path;
}

robot Rover {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  safety {
    max_speed = 1.0 m/s;
  }

  behavior run() {
    let state = "Idle";
    match state {
      Idle => wheels.stop();
      Navigating => wheels.drive(linear: 0.3 m/s, angular: 0.0 rad/s);
      EmergencyStop => emergency_stop;
    };
  }
}
`;

const AGENT_TASK_STATE_EVENT_TWIN = `
robot DeliveryBot {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  safety {
    max_speed = 1.0 m/s;
    stop_if lidar.nearest_distance < 0.5 m;
  }

  event ObstacleDetected;

  on ObstacleDetected {
    wheels.stop();
  }

  twin DeliveryTwin {
    mirror pose;
    replay true;
  }

  ai_model planner: LLM {
    provider: "mock";
    model: "safe-planner";
    temperature: 0.1;
  }

  agent Navigator {
    uses planner;
    tools [lidar, wheels];
    memory short_term;
    skill path_planning;
    goal "Deliver safely";
    can [ read(lidar), propose_motion ];

    plan {
      let scan = lidar.read();
      let proposal = planner.reason(prompt: "Plan safe motion", input: scan);
      let action = safety.validate(proposal);
      wheels.execute(action);
    }
  }

  state_machine Delivery {
    state Idle;
    state Navigate;
    state Deliver;
    transition Idle -> Navigate;
    transition Navigate -> Deliver;
  }

  task control_loop every 20ms requires lidar.nearest_distance > 0.3 m {
    Navigator.plan();
  }
}
`;

const BEHAVIOR_CONTRACTS = `
robot R {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  behavior move() requires lidar.nearest_distance > 0.5 m ensures true {
    wheels.drive(linear: 0.2 m/s, angular: 0.0 rad/s);
  }
}
`;

describe("foundations", () => {
  it("parses and type-checks module, struct, enum, trait, and match", () => {
    expect(() => compile(MODULE_STRUCT_ENUM_TRAIT)).not.toThrow();
  });

  it("parses and type-checks agent capabilities, task, state machine, event, twin", () => {
    expect(() => compile(AGENT_TASK_STATE_EVENT_TWIN)).not.toThrow();
  });

  it("type-checks behavior contracts", () => {
    expect(() => compile(BEHAVIOR_CONTRACTS)).not.toThrow();
  });

  it("rejects task interval below 1ms", () => {
    expect(() =>
      compile(`
        robot R {
          task fast every 0.5ms { }
        }
      `),
    ).toThrow(TypeCheckError);
  });

  it("rejects twin with no mirror fields", () => {
    expect(() =>
      compile(`
        robot R {
          twin T { replay false; }
        }
      `),
    ).toThrow(TypeCheckError);
  });

  it("rejects unknown module import", () => {
    expect(() =>
      compile(`
        import foo.bar;
        robot R { }
      `),
    ).toThrow(TypeCheckError);
  });

  it("parses enter statement for state machine transitions", () => {
    expect(() =>
      compile(`
        robot Bot {
          state_machine Flow {
            state Idle;
            state Loading;
            transition Idle -> Loading;
          }
          behavior run() {
            enter Loading;
          }
        }
      `),
    ).not.toThrow();
  });
});
