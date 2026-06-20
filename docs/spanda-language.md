# Spanda Language Reference (v0.2 foundations)

Spanda programs use the `.sd` extension. Programs are organized around **autonomous systems**, not OOP class hierarchies.

> **API reference:** [spanda-reference.md](./spanda-reference.md) lists every keyword, `std.*` type, built-in function/method (with signatures), and CLI command in JavaDoc / man-page form. Generate per-module docs with `spanda doc file.sd`.

## Modules

```spanda
module navigation;

import navigation.path_planning;
import std.robotics;
```

Dotted module names (`navigation.path_planning`) identify compilation units in multi-file projects. Use **`export`**, **`public`**, or **`private`** on module-level functions:

```spanda
module navigation.path_planning;

export fn plan_path(from: Pose, to: Pose) -> Path {
  return trajectory(from: from, to: to, steps: 8);
}

private fn internal_helper() -> Path { ... }
```

Imported modules inject **exported** symbols into the importer's scope. Cross-file linking uses `ModuleRegistry` (see `compile_with_registry` / `RunOptions.module_registry`).

Generic module functions:

```spanda
export fn identity<T>(value: T) -> T {
  return value;
}
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

## Result and Option

`Result<T, E>` and `Option<T>` are first-class generic types. Construct and match them without exceptions:

```spanda
export fn navigate() -> Result<Path, NavError> {
  return Err(Blocked);
}

match navigate() {
  Ok => wheels.stop();
  Err => emergency_stop;
};

let scan: Option<Scan> = None();
match scan {
  Some => process(scan);
  None => wheels.stop();
};
```

## Async and await

Module functions may be declared `async`. Calls return `Future<T>`; use `await` inside behaviors, tasks, or other async functions:

```spanda
module maps;

export async fn get_map() -> Pose {
  return pose(x: 0.0 m, y: 0.0 m, theta: 0.0 rad);
}

robot R {
  behavior run() {
    let map = await get_map();
    let _ = map;
  }
}
```

## Concurrency

Cooperative concurrency primitives for background work and message passing:

```spanda
module comm;

export fn ping() -> Int {
  return 1;
}

robot R {
  behavior run() {
    let ch = channel();
    send(ch, 42);
    select {
      recv(ch) => wheels.stop();
    };
    spawn ping();
  }
}
```

- `channel()` — create a typed channel handle
- `send(ch, value)` / `recv(ch)` — non-blocking send and receive builtins
- `select { recv(ch) => ... }` — run the first arm whose channel has a message
- `spawn callee(args);` — queue a module function call on the spawn queue (processed after behaviors and tests)
- `join(handle)` — resolve a `Future<T>` or `TaskHandle<T>`
- `parallel { ... }` — cooperative concurrent orchestration with `_parallel` results

Full reference: [concurrency.md](./concurrency.md)

## Serialization

Serialize and deserialize runtime values for telemetry, logging, and IPC:

```spanda
let data = serialize(pose, "json");
let restored = deserialize(data, "json");
```

Supported formats: `"json"`, `"yaml"`, `"binary"`.

## In-language tests

Top-level test blocks run with `spanda test` or `run_tests()`:

```spanda
module math;

export fn double(x: Int) -> Int {
  return x;
}

test "double returns input" {
  assert(true);
}
```

`assert(condition)` is a builtin; failed assertions fail the test run.

## Foreign functions (FFI)

Declare native bindings the runtime resolves through `FfiRegistry`:

```spanda
extern "libc" fn stub_add(a: Int, b: Int) -> Int;

export fn sum_pair(a: Int, b: Int) -> Int {
  return stub_add(a, b);
}
```

Built-in stub bindings include `stub_echo` and `stub_add` for testing.

## Code generation and deployment

Cross-target stubs (validation + emit only; no full native compiler yet):

```bash
spanda codegen program.sd --target native
spanda codegen program.sd --target wasm --out out.wat
spanda codegen program.sd --target esp32 --out robot.ino
spanda deploy program.sd --target wasm --out deploy.json
```

## Debugging

Set breakpoints by line and run under the debug controller:

```bash
spanda debug program.sd --break 12
```

For editor integration, use the DAP adapter:

```bash
spanda-dap program.sd   # stdio Debug Adapter Protocol
```

## Formatting

The Rust CLI includes an AST-aware formatter:

```bash
spanda fmt program.sd
spanda fmt --json program.sd   # returns formatted source without writing
```

It normalizes indentation (2 spaces), spacing around types/operators, and block structure. Unparseable files fall back to whitespace normalization.

## Linting

Style and hygiene checks beyond type-checking:

```bash
spanda lint program.sd
spanda lint --json program.sd
```

Rules include `missing-module`, `trailing-whitespace`, `line-length`, `empty-test`, `empty-behavior`, and `unused-import`.

## Documentation generation

Generate Markdown API docs from module exports:

```bash
spanda doc program.sd
spanda doc program.sd --out docs/api.md
spanda doc --json program.sd
```

## In-language tests

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

## Events and triggers

Events are the simplest trigger form. The unified trigger model also supports timers, conditions, topics, state transitions, safety, hardware faults, AI outcomes, and twin divergence — see [triggers.md](./triggers.md).

```spanda
event ObstacleDetected;

on ObstacleDetected {
  wheels.stop();
}

every 100ms {
  publish_pose();
}

when lidar.nearest_distance < 1.0 m {
  slow_down();
}
```

Trace trigger execution at runtime:

```bash
spanda run robot.sd --trace-triggers --trace-events
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

- `hello_world.sd`, `humanoid_assistant.sd`, `triggers_demo.sd`, `concurrency.sd`
- `hardware/rover_deploy.sd`, `hardware/full_compat.sd`
- `communication/multi_robot_fleet.sd`
- `types/goals.sd`, `types/memory.sd`, `types/verify.sd`, `types/fusion.sd`, `types/multitask.sd`
- `examples/modules/` — cross-file exports and imports
- `crates/spanda-core/tests/p1_features.rs` — async, serialize, tests, concurrency
