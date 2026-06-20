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
| `FfiRegistry` | Partially implemented | Stub handlers; `extern python`/`extern cpp` subprocess bridges |
| N-API / WASM bindings | Partially implemented | `check`, `run`, `verify`, `sir`, `fmt` |
| Python bridge | **Partially implemented** | Subprocess bridge via `scripts/spanda_python_bridge.py`; optional in-process PyO3 |
| C/C++ bridge | **Partially implemented** | Subprocess bridge via build-time C++ helper binary |
| ROS2 bridge | Stubbed | `Ros2AdapterStub` logs calls; no live ROS2 node |
| OpenCV / PyTorch / TensorFlow | Planned | Import paths reserved in std registry |

Real native linking (dlopen, cxx) is **not** implemented yet. **`extern python fn`** calls use a **subprocess JSON bridge** by default when `python3` and `scripts/spanda_python_bridge.py` are available. Build with `--features python-native` on `spanda-core` for an **in-process PyO3** path (same handlers, no subprocess). Set `SPANDA_PYTHON_SUBPROCESS=1` to force subprocess mode even when PyO3 is enabled.

### Subprocess Python bridge (implemented)

Bridge handlers include transport and AI shims (mock when optional deps absent):

- `ros2_publish(topic, data)` — uses **rclpy** when installed, else mock metadata
- `mqtt_publish(topic, payload)` — uses **paho-mqtt** when installed (`MQTT_BROKER` / `MQTT_PORT`), else mock
- `openai_complete(prompt)` — calls OpenAI when `OPENAI_API_KEY` is set, else mock

```bash
# Optional: custom bridge script path
export SPANDA_PYTHON_BRIDGE=/path/to/spanda_python_bridge.py

spanda run examples/ffi_python_extern.sd
```

Register handlers in `scripts/spanda_python_bridge.py`:

```python
HANDLERS = {
    "py_add": lambda a, b: int(a) + int(b),
    "py_version": lambda: 1,
}
```

Spanda program:

```spanda
extern python fn py_add(a: Int, b: Int) -> Int;
let sum = py_add(2, 3);
```

Protocol: Rust sends `{"fn":"py_add","args":[2,3]}` on stdin; Python returns `{"ok":true,"result":5}`.

Calling `extern python fn` without a registered handler fails with `Unknown python extern 'name'`.

### In-process Python bridge (optional `python-native` feature)

When `spanda-core` is built with `--features python-native`, `extern python fn` calls load handlers from the same bridge script in-process via PyO3 (stable ABI `abi3-py310`). Subprocess mode remains the default when the feature is off, or when `SPANDA_PYTHON_SUBPROCESS=1` is set.

On Python versions newer than PyO3's supported range, set `PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1` at build time.

```bash
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 cargo build -p spanda-core --features python-native
spanda run examples/ffi_python_extern.sd
```

### Subprocess C++ bridge (implemented)

`spanda-core` compiles a small C++ helper at build time when a C++ compiler is available (`CXX` or `c++`). Override with:

```bash
export SPANDA_CPP_BRIDGE=/path/to/spanda_cpp_bridge
spanda run examples/ffi_cpp_extern.sd
```

Built-in handlers in `crates/spanda-core/src/bridge/spanda_cpp_bridge.cpp`:

- `cpp_add(a, b)` — numeric sum
- `cpp_echo(x)` — identity
- `cpp_version()` — returns `1`

```spanda
extern cpp fn cpp_add(a: Int, b: Int) -> Int;
let sum = cpp_add(2, 3);
```

Uses the same JSON protocol as the Python bridge. Unknown handlers fail with `Unknown cpp extern 'name'`.

Real static/dynamic linking via cxx/bindgen is **not** implemented yet.

### In-process C++ bridge (optional `cpp-native` feature)

When `spanda-core` is built with `--features cpp-native`, `extern cpp fn` calls the same handler dispatch in-process via a C ABI (`spanda_cpp_bridge_call`). Subprocess mode remains the default when the feature is off, or when `SPANDA_CPP_SUBPROCESS=1` is set.

```bash
cargo build -p spanda-core --features cpp-native
spanda run examples/ffi_cpp_extern.sd
```

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
