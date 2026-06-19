# Spanda Type System

Spanda’s type system supports general-purpose programming and autonomous-systems domains: physical units, spatial math, sensors, AI, agents, safety, digital twins, and distributed robotics.

## Foundation types

| Type | Example |
|------|---------|
| `Int` | `let n: Int = 42;` |
| `Float` | `let x: Float = 0.5;` |
| `Bool` | `let ok: Bool = true;` |
| `String` | `let s: String = "rover";` |
| `Char` | `let c: Char;` |
| `Bytes` | `let buf: Bytes;` |
| `Null` | `let empty: Null;` |

Type annotations are optional when an initializer is present; the checker infers types from expressions.

```spanda
let count: Int = 3;
let label: String = "spanda";
```

## Generic types

Collections and distributed types use angle-bracket generics:

```spanda
let goals: Array<Goal>;
let map: Map<String, Int>;
let stream: Topic<LidarScan>;
let svc: Service<Command, Feedback>;
let nav: Action<Command, Feedback, Path>;
```

The compiler reports arity errors when generic parameters are missing or extra, e.g. `Array` requires exactly one type argument.

## Physical unit types

Unit-aware types prevent mixing incompatible dimensions:

```spanda
let speed: Velocity = 1.5 m/s;
let distance: Distance = 2.0 m;
let timeout: Duration = 500 ms;
```

Invalid operations are rejected at compile time:

```spanda
// ERROR: speed + distance — incompatible physical categories
let bad = speed + distance;
```

Supported unit types include `Distance`, `Velocity`, `Acceleration`, `Angle`, `AngularVelocity`, `Mass`, `Force`, `Power`, `Voltage`, `Current`, `Temperature`, and `Pressure`.

## Time types

```spanda
let timeout: Duration = 500 ms;
let started_at: Timestamp;
```

Namespace: `import std.time;`

## Spatial and robotics types

`Point2D`, `Point3D`, `Vector2D`, `Vector3D`, `Quaternion`, `Pose`, `Transform`, `Trajectory`, `Path`, `Waypoint`, `MotionCommand`, `ControlSignal`, `PIDConfig`.

Namespace: `import std.spatial;`

## Sensor types

`CameraFrame`, `Image`, `DepthImage`, `PointCloud`, `LidarScan`, `GpsFix`, `ImuData`, `AudioFrame`.

Namespace: `import std.sensors;`

## AI types

`LLM`, `VisionModel`, `EmbeddingModel`, `Prompt`, `Completion`, `Embedding`, `Token`, `Context`, `Memory`, `Plan`, `ReasoningTrace`, `Goal`.

Namespace: `import std.ai;`

### AI model hardware config

`ai_model` blocks accept config keys used by hardware verification:

```spanda
ai_model Vision: VisionModel {
  memory_required: 2 GB;
  gpu_required: true;
}
```

| Config key | Verification |
|------------|----------------|
| `memory_required` | Compared to target profile `memory` |
| `gpu_required` | Target must have GPU / `gpu_tops` |

## Agent and autonomy types

`Agent`, `Goal`, `Task`, `Skill`, `Capability`, `Intent`, `ActionProposal`, `SafeAction`.

### ActionProposal vs SafeAction

`ActionProposal` is **untrusted** output from AI planners. It must never reach actuators directly.

```spanda
let proposal: ActionProposal = planner.reason(prompt: "go");
let action: SafeAction = safety.validate(proposal);
wheels.execute(action);   // OK
```

```spanda
wheels.execute(proposal);   // COMPILE ERROR
```

The type checker rejects `ActionProposal` passed to `actuator.execute()`. Only `SafeAction` from `safety.validate()` is permitted.

## Human interaction types

`Command`, `Conversation`, `Speech`, `Gesture`, `Emotion`, `Feedback`.

Namespace: `import std.hri;`

## Safety types

`Risk`, `Hazard`, `SafetyConstraint`, `EmergencyStop`, `SafeAction`.

Namespace: `import std.safety;`

## Digital twin types

`Twin`, `SimulationState`, `Telemetry`, `Replay`, `Fault`, `Scenario`.

Namespace: `import std.twin;`

Fault types for `simulate_compatibility` (verification, not runtime twin API): `CameraFailure`, `LidarFailure`, `ImuFailure`, `BatteryDegradation`, `NetworkOutage`.

## Hardware compatibility types

Declared in programs, not runtime values:

| Construct | Role |
|-----------|------|
| `hardware Profile { }` | Platform capability declaration |
| `deploy Robot to Target` | Deployment binding |
| `requires_hardware { }` | Minimum platform requirements |
| `requires_network { }` | Connectivity requirements |
| `budget { }` | Per-task resource limits |
| `mission { duration }` | Mission length for power estimation |

Verification output types (Rust/JSON): `CompatibilityReport`, `CompatItem`, `CompatibilityMatrix`.

See [hardware-compatibility.md](./hardware-compatibility.md).

## Networking / distributed robotics

`Topic<T>`, `Message<T>`, `Service<Request, Response>`, `Action<Request, Feedback, Result>`, `Endpoint`.

## Advanced autonomous intelligence

`KnowledgeGraph`, `Belief`, `Observation`, `WorldModel`, `Policy`, `Reward`, `StateEstimate`.

## Standard library namespaces

| Module | Domain |
|--------|--------|
| `std.time` | Time and duration |
| `std.units` | Physical units |
| `std.spatial` | Pose, path, transforms |
| `std.ai` | Models and reasoning |
| `std.robotics` | Agents, motion, capabilities |
| `std.sensors` | Perception data |
| `std.safety` | Constraints and safe actions |
| `std.twin` | Simulation and replay |
| `std.hri` | Human–robot interaction |

Import with `import std.units;` then annotate types normally (`Distance`, `Velocity`, …).

## Examples

See `examples/types/` for annotated programs covering each category.

```bash
spanda check examples/types/units.sd
spanda run examples/types/safety.sd
```
