# FFI and Ecosystem Interoperability

Spanda is designed to **orchestrate** existing robotics and AI ecosystems — not replace Python, C++, or ROS2 overnight. The language provides typed boundaries, safety validation, and hardware verification at the Spanda layer while delegating heavy computation to mature libraries.

## Design principles

1. **Orchestrate, don't rewrite** — Spanda programs coordinate perception, planning, safety, and actuation. Inference, SLAM, and low-level drivers stay in Python/C++ where the ecosystem is strongest.
2. **Untrusted by default** — Outputs from external AI libraries are `ActionProposal` values until `safety.validate()` produces a `SafeAction`.
3. **Explicit boundaries** — Every foreign call is declared with `extern fn` and typed at the Spanda boundary.
4. **Capability-gated** — Agent `can [...]` and package capabilities control which foreign symbols a program may link.
5. **Verify before deploy** — Hardware compatibility checks run regardless of whether actuation goes through Spanda or FFI.

## Current status

| Layer | Status | Notes |
|-------|--------|-------|
| `extern fn` syntax | Implemented | Parsed and type-checked in Rust core |
| `FfiRegistry` | Partially implemented | Stub handlers; `extern python`/`extern cpp` fail at runtime until linked |
| N-API / WASM bindings | Partially implemented | `check` and `run` only |
| Python bridge | Planned | See syntax below |
| C/C++ bridge | Planned | See syntax below |
| ROS2 bridge | Stubbed | `Ros2AdapterStub` logs calls; no live ROS2 node |
| OpenCV / PyTorch / TensorFlow | Planned | Import paths reserved in std registry |

Real native linking (dlopen, PyO3, cxx) is **not** implemented yet. Calling `extern fn` at runtime without a registered stub produces a clear runtime error.

## Planned import syntax

Future modules will map ecosystem namespaces to bridge backends:

```spanda
import python.torch;
import python.opencv;
import cpp.ros2;
import cpp.pcl;
```

Bridge packages declare capabilities in `spanda.toml`:

```toml
[package]
name = "ros2-bridge"
capabilities = ["comm.ros2.publish", "comm.ros2.subscribe"]
safety_level = "certified"
```

## Planned extern declarations

Foreign functions are declared in Spanda and implemented by bridge shims:

```spanda
extern python fn detect_objects(frame: CameraFrame) -> Array<Detection>;

extern cpp fn publish_ros_topic(topic: String, msg: Message);

extern cpp fn run_slam(scan: Scan) -> Pose;
```

At compile time, the type checker validates signatures. At link time, the bridge resolves symbols to:

- **Python** — embedded interpreter or subprocess RPC (PyO3 / IPC)
- **C/C++** — static or dynamic link via cxx / bindgen
- **ROS2** — rclcpp/rclpy adapter implementing Spanda `topic` / `service` / `action` mappings

## ROS2 mapping (planned)

| Spanda | ROS2 |
|--------|------|
| `node navigation on "/nav"` | `rclcpp::Node` with namespace |
| `topic cmd_vel: Velocity publish` | `rclcpp::Publisher` |
| `subscribe scan` | `rclcpp::Subscription` |
| `service reset_map` | `rclcpp::Service` |
| `action go_to` | `rclcpp_action::Server` |
| `publish cmd_vel with ...` | `publish()` on typed adapter |

The existing `examples/ros2_bridge.sd` and `examples/packages/ros2_adapter_package/` demonstrate the intended surface; transport adapters today log to the simulator only.

## AI / ML mapping (planned)

| Spanda | External |
|--------|----------|
| `ai_model planner: LLM` | OpenAI / local llama.cpp via Python bridge |
| `vision.detect(frame)` | PyTorch / ONNX Runtime via `import onnx.runtime` |
| `ActionProposal` | Raw model output — never reaches actuators directly |

Import registry entries (`onnx.runtime`, `tflite.runtime`, `tensorrt.runtime`, `openvino.runtime`) exist for metadata and future linking.

## Safety at the boundary

```spanda
let proposal = detect_objects(camera.frame());  // ActionProposal or typed Detection[]
let action = safety.validate(proposal);         // SafeAction
wheels.execute(action);                         // OK
```

Direct actuator calls with foreign outputs remain a **compile error** unless the value is explicitly validated.

## Deployment model

```
┌─────────────────────────────────────────┐
│  Spanda program (.sd)                   │
│  safety · verify · scheduler · twin     │
└──────────────┬──────────────────────────┘
               │ extern fn / bridge imports
┌──────────────▼──────────────────────────┐
│  Bridge layer (future)                  │
│  Python · C++ · ROS2 · CUDA            │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│  Vendor SDKs · PyTorch · Gazebo · …    │
└─────────────────────────────────────────┘
```

## Related documents

- [compiler-backend-roadmap.md](./compiler-backend-roadmap.md) — native code generation path
- [packages.md](./packages.md) — package capabilities and trust levels
- [security.md](./security.md) — capabilities, secrets, signed messages
- [hardware-compatibility.md](./hardware-compatibility.md) — deploy verification before FFI-heavy workloads
