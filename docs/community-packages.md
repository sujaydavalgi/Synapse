# Community Packages

Spanda's package ecosystem supports community-contributed frameworks, drivers, adapters, and libraries for robotics, AI, simulation, and safety.

## Package categories

Every package can declare one or more categories:

| Category | Examples |
|----------|----------|
| `ai` | LLM providers, inference runtimes |
| `robotics` | Control, planning, fleet coordination |
| `vision` | OpenCV, YOLO, depth cameras |
| `navigation` | SLAM, path planning, localization |
| `manipulation` | Grasping, arm control |
| `simulation` | Gazebo, Webots backends |
| `ros2` | ROS 2 bridges and adapters |
| `mqtt` | MQTT pub/sub |
| `hardware` | Board profiles, HAL |
| `sensors` | Lidar, camera, IMU drivers |
| `actuators` | Motor, servo, gripper drivers |
| `digital-twin` | Twin sync and replay |
| `safety` | Constraint checkers, monitors |
| `hri` | Speech, gesture, dialogue |
| `testing` | Test harnesses, mocks |

```toml
categories = ["sensors", "hardware"]
```

## Driver / adapter packages

Community sensor and actuator drivers follow the adapter model:

```toml
[package]
name = "spanda-lidar-rplidar"
version = "0.1.0"
license = "MIT"

[adapter]
provides = ["LidarAdapter", "Topic<LidarScan>"]
requires = ["serial.port", "lidar.read"]

[capabilities]
uses = ["serial.port", "lidar.read"]

[safety]
level = "hardware_safe"
can_control_actuators = false
```

In Spanda source:

```spanda
module spanda_lidar_rplidar;

import sensors.lidar;
import std.sensors;
```

The `provides` list declares symbols exported to consumers; `requires` lists runtime capabilities the driver needs.

## Framework packages

Framework packages wrap external systems (ROS 2, MQTT, Gazebo) and expose Spanda-native import paths:

```spanda
import robotics.ros2;
import communication.mqtt;
import sim.gazebo;
```

Add them as dependencies:

```toml
[dependencies]
spanda-ros2 = "0.1.0"
spanda-sim-gazebo = "0.1.0"
```

## AI provider packages

`spanda-openai` is included as a reference provider package in this repository.
It exports `ai.openai.complete(prompt)` and routes through the Python subprocess
bridge (`scripts/spanda_python_bridge.py`) to call OpenAI when
`OPENAI_API_KEY` is present.

Without an API key, the bridge returns deterministic mock completions so tests
and simulations remain reproducible.

## Safety and trust

Community packages must declare a safety level. Defaults:

- New packages start at `experimental` with `requires_review = true`
- Driver packages that only read sensors should set `can_control_actuators = false`
- Packages controlling actuators on real hardware should target `hardware_safe` or `certified`

Applications gate deployment by allowed safety levels and capability grants.

## License compatibility

Declare your package license in `[package]` and optionally list compatible dependency licenses:

```toml
[package]
license = "Apache-2.0"

license_compat = ["Apache-2.0", "MIT"]
```

Validation warns when dependency licenses may conflict with application policy.

## Contributing a package

1. Run `spanda init my-package`
2. Implement sources under `src/`
3. Declare `[adapter]`, `[capabilities]`, and `[safety]` appropriately
4. Add tests under `tests/`
5. Run `spanda publish` to validate locally
6. (Future) Submit to the public registry

See [examples/packages/](../examples/packages/) for reference implementations.
