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

It trims trailing whitespace and normalizes the final newline. The LSP package (`packages/lsp/`) surfaces `spanda check --json` diagnostics in editors.

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
  perceive();
  act();
}
```

Tasks are scheduled with fixed intervals and validated by the type checker.

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

See `examples/` including `hello_world.sd`, `rover_navigation.sd`, `warehouse_robot.sd`, `drone_patrol.sd`, `humanoid_assistant.sd`, `digital_twin.sd`, and `ros2_bridge.sd`.
