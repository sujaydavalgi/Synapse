# Spanda Language Reference (v0.2 foundations)

Spanda programs use the `.sd` extension. Programs are organized around **autonomous systems**, not OOP class hierarchies.

## Modules

```spanda
module navigation;

import sensors.lidar;
import motion.drive;
```

## Structs and type aliases

```spanda
struct Pose {
  x: Distance;
  y: Distance;
  heading: Angle;
}
```

Built-in aliases: `Distance` (meters), `Angle` (radians), `Path` (trajectory).

## Enums and pattern matching

```spanda
enum RobotState {
  Idle,
  Navigating,
  EmergencyStop
}

let state = Idle;              // unqualified variant
let mode = RobotState.Idle;    // qualified variant

match state {
  Idle => wheels.stop();
  Navigating => wheels.drive(linear: 0.3 m/s, angular: 0.0 rad/s);
  EmergencyStop => emergency_stop;
};
```

## Struct literals

Construct typed values at runtime:

```spanda
struct Pose {
  x: Distance;
  y: Distance;
  heading: Angle;
}

let goal = Pose { x: 1.0 m, y: 2.0 m, heading: 0.0 rad };
let _x = goal.x;
```

## Traits and implementations

```spanda
trait Navigator {
  fn plan(goal: Pose) -> Path;
}

robot R {
  agent Nav { tools [wheels]; goal "Navigate"; plan { wheels.stop(); } }

  impl Navigator for Nav {
    fn plan(goal: Pose) -> Path {
      wheels.stop();
    }
  }

  behavior run() {
    Nav.plan(Pose { x: 0.0 m, y: 0.0 m, heading: 0.0 rad });
  }
}
```

Traits define interfaces; bind implementations to agents with `impl Trait for AgentName { ... }` inside a robot block.

## Formatting

The Rust CLI includes a basic formatter:

```bash
spanda fmt program.sd
```

It trims trailing whitespace and normalizes the final newline. The LSP package (`packages/lsp/`) surfaces `spanda check` and `spanda verify` diagnostics in editors.

## Agents, skills, and capabilities

```spanda
agent Navigator {
  uses planner;
  tools [lidar, wheels];
  memory short_term;
  skill path_planning;
  goal "Reach destination safely";
  can [ read(lidar), propose_motion ];

  plan {
    let scan = lidar.read();
    let proposal = planner.reason(prompt: "Plan safe motion", input: scan);
    let action = safety.validate(proposal);
    wheels.execute(action);
  }
}
```

## Deterministic tasks

```spanda
task control_loop every 20ms requires lidar.nearest_distance > 0.4 m {
  budget {
    battery <= 10%;
    memory <= 512 MB;
    cpu <= 20%;
  }
  perceive();
  act();
}
```

Tasks are scheduled with fixed intervals and validated by the type checker. Optional `budget { }` declares per-task resource limits checked at hardware verification time.

## Hardware profiles and deployment

Declare platform capabilities and bind programs to targets:

```spanda
hardware RoverV1 {
  cpu: CortexA78;
  memory: 4 GB;
  sensors [ Camera, Lidar, IMU ];
  actuators [ DifferentialDrive ];
  battery { capacity: 100 Wh; }
  network { bandwidth: 100 Mbps; latency: 20 ms; }
  timing { min_period: 10 ms; }
  resource: 15 W;
}

requires_hardware {
  memory >= 2 GB;
  sensors [ Camera, Lidar ];
}

requires_network {
  bandwidth >= 10 Mbps;
  latency <= 50 ms;
}

robot Rover {
  sensor camera: Camera on "/camera";
  actuator wheels: DifferentialDrive;
  mission { duration: 1 h; }
  behavior run() { wheels.stop(); }
}

deploy Rover to RoverV1;
deploy Rover to [ RoverV1, ESP32 ];
```

Verify before deploy:

```bash
spanda verify program.sd
spanda verify program.sd --target RoverV1 --all-targets --simulate
```

Full reference: [hardware-compatibility.md](./hardware-compatibility.md)

## Simulation compatibility (fault injection)

```spanda
simulate_compatibility {
  fault CameraFailure;
  fault BatteryDegradation;
  fault NetworkOutage;
}
```

Faults modify the target profile during verification (camera/lidar/IMU removal, battery halving, network outage).

## Behavioral verification

Distinct from hardware `spanda verify` — runtime assertions after behavior/task execution:

```spanda
verify {
  robot.velocity().linear <= 2.0 m/s;
}
```

## Goals and memory

```spanda
agent Navigator {
  goal "Reach the dock";
  plan {
    let mission = goal(text: "Reach the dock");
    remember("last_scan", lidar.read());
    let prior = recall("last_scan");
  }
}
```

## Sensor fusion

```spanda
observe {
  lidar;
  camera;
}

behavior fuse() {
  let fused = fusion.read();
}
```

## State machines

```spanda
state_machine Delivery {
  state Idle;
  state Navigate;
  state Deliver;
  transition Idle -> Navigate;
  transition Navigate -> Deliver;
}
```

At runtime, transition with `enter StateName;` inside a behavior or task body. The runtime applies the transition to every state machine that declares a valid edge from its current state to the target.

```spanda
behavior start_delivery() {
  enter Navigate;
}
```

## Contracts

```spanda
behavior move() requires lidar.nearest_distance > 0.5 m ensures true {
  wheels.drive(linear: 0.2 m/s, angular: 0.0 rad/s);
}
```

## Events

```spanda
event ObstacleDetected;

on ObstacleDetected {
  wheels.stop();
}
```

## Digital twins

```spanda
twin RobotTwin {
  mirror pose;
  mirror velocity;
  replay true;
}
```

At runtime, query the twin from task or behavior code:

```spanda
let frames = RobotTwin.frame_count();
let shadow_pose = RobotTwin.pose();
let past_pose = RobotTwin.replay(index: 0, field: pose);
```

`frame_count()` returns the number of buffered replay frames (when `replay true`). Mirrored fields (`pose`, `velocity`, etc.) are readable as methods on the twin name. `replay(index, field)` retrieves a historical snapshot.

## Physical units

`m`, `s`, `ms`, `rad`, `deg`, `m/s`, `Hz` — unit mismatches are compile-time errors.

## Examples

See `examples/` including:

- `hello_world.sd`, `humanoid_assistant.sd`
- `hardware/rover_deploy.sd`, `hardware/full_compat.sd`
- `types/goals.sd`, `types/memory.sd`, `types/verify.sd`, `types/fusion.sd`, `types/multitask.sd`
- `rover_navigation.sd`, `warehouse_robot.sd`, `digital_twin.sd`, `ros2_bridge.sd`
