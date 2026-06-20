# Spanda Standard Library

The Spanda standard library is organized as modular **`std.*` namespaces**. Each namespace registers types and import paths; runtime behavior is provided by the interpreter and domain modules (`safety`, `simulator`, `comm`, etc.).

**Full API reference:** [spanda-reference.md](./spanda-reference.md) (generated from the type checker — all `std.*` types, global functions, and built-in methods with signatures).

## Design principles

- **Modular** — import only what a program needs: `import std.robotics;`
- **Not monolithic** — there is no single `std` package; namespaces are independent
- **Blockchain-free** — ledger and on-chain features live in optional community packages, not in `std.*`

## Namespaces

| Namespace | Purpose | Key types |
|-----------|---------|-----------|
| `std.core` | Foundation types | `Result`, `Option`, `Error` |
| `std.time` | Time and scheduling | `Time`, `Duration`, `Timestamp`, `Interval` |
| `std.units` | Physical units | `Distance`, `Velocity`, `Mass`, `Temperature`, … |
| `std.spatial` | Geometry | `Pose`, `Transform`, `Vector3D`, `Path`, `Trajectory` |
| `std.math` | Scalar math | `Float`, `Int` |
| `std.collections` | Containers | `Array`, `Map`, `Set`, `Queue`, `Stack` |
| `std.result` | Error handling | `Result`, `Option`, `Error` |
| `std.io` | I/O abstractions | `File`, `Reader`, `Writer`, `Bytes` |
| `std.log` | Logging | `Logger`, `LogLevel` |
| `std.ai` | AI models | `LLM`, `VisionModel`, `Prompt`, `Completion`, `ReasoningTrace` |
| `std.robotics` | Robot graph | `Robot`, `Sensor`, `Actuator`, `MotionCommand`, `ActionProposal`, `SafeAction` |
| `std.sensors` | Sensor payloads | `LidarScan`, `CameraFrame`, `ImuData`, … |
| `std.actuators` | Actuator types | `Motor`, `Servo`, `Gripper`, `DriveUnit`, … |
| `std.safety` | Safety types | `Risk`, `Hazard`, `SafetyConstraint`, `EmergencyStop` |
| `std.communication` | Messaging | `Message`, `Topic`, `Service`, `Action`, `Event`, `Bus` |
| `std.hardware` | Deploy targets | `HardwareProfile`, `CompatibilityReport` |
| `std.sim` | Simulation | `Simulator`, `Scenario`, `Fault`, `Replay` |
| `std.twin` | Digital twin | `Twin`, `SimulationState`, `Telemetry` |
| `std.hri` | Human interaction | `Command`, `Conversation`, `Intent`, `Feedback`, `Approval` |
| `std.security` | Identity & trust | `Identity`, `RobotIdentity`, `Signature`, `Permission`, `TrustLevel` |
| `std.audit` | Audit types | `AuditEvent`, `AuditLog`, `ProvenanceRecord`, `MissionRecord` |
| `std.crypto` | Cryptography | `Hash`, `sha256()`, `sign()`, `verify_signature()` |

## Usage

```spanda
import std.units;
import std.spatial;
import std.robotics;

robot Demo {
  behavior run() {
    let d: Distance = 1.0 m;
    let p = pose(x: 0.0, y: 0.0, theta: 0.0);
  }
}
```

## Examples

See `examples/std/` for runnable programs covering time, units, spatial, robotics, AI, communication, audit, provenance, device identity, and mock ledger anchoring.

## Community packages

Vendor and domain libraries (ROS 2, OpenCV, YOLO, SLAM, ledger backends) ship as **optional packages** — see [packages.md](./packages.md) and [registry.md](./registry.md).
