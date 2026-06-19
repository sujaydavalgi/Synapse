# Synapse

A safe, readable, strongly typed programming language for robot control, sensors, actuators, motion planning, automation, and simulation.

Synapse is designed for robotics engineers and students who want **deterministic**, **safety-first** robot programs that run in simulation first and can connect to hardware later via a ROS2 adapter.

Source files use the **`.syn`** extension.

## Features

- **Strong typing with physical units** — `m`, `s`, `ms`, `rad`, `m/s`, `rad/s`
- **Robot-centric syntax** — sensors, actuators, safety blocks, and behaviors
- **ROS2-style concepts** — `node`, `topic`, `service`, `action` declarations
- **HAL (Hardware Abstraction Layer)** — `i2c`, `spi`, `gpio`, `pwm`, `uart`, `adc` buses and pins
- **SoC profiles** — Raspberry Pi, ESP32, STM32, Jetson, Arduino with capability validation
- **Sensor libraries** — manufacturer drivers (Velodyne, Hokuyo, Bosch, Intel, YDLIDAR, Adafruit, SparkFun, Waveshare)
- **AI-native agents** — `ai_model`, `agent`, LLM/Vision calls with mandatory safety validation
- **Motion types** — `pose()`, `velocity()`, `trajectory()`, `transform()`
- **Safety zones** — circular and rectangular keep-out regions
- **Emergency stop** — `emergency_stop` and `reset_emergency_stop` statements
- **Deterministic loop scheduling** — `loop every 50ms { ... }`
- **Safety rules** — always evaluated before motion commands
- **Simulation backend** — test without hardware
- **ROS2 adapter interface** — stub ready for future integration

## Quick Start

```bash
npm install
npm test
npm run synapse -- run examples/lidar_avoidance.syn
npm run synapse -- sim examples/differential_drive.syn
```

## HAL, SoC, and Sensor Libraries

```synapse
import bosch.bno055;
import velodyne.vlp16;

robot PiBot {
  soc RaspberryPi4;

  hal {
    i2c imu_bus at 0x68;
    gpio status_led out pin 17;
    uart lidar_uart on "/dev/ttyUSB0" baud 230400;
  }

  sensor imu: BoschBNO055 from bosch.bno055 on imu_bus;
  sensor lidar: VelodyneVLP16 from velodyne.vlp16 on "/velodyne_points";
}
```

### Supported SoC profiles

| Profile | Vendor | Architecture | Key capabilities |
|---------|--------|--------------|------------------|
| `RaspberryPi4` / `RaspberryPi5` | Broadcom | aarch64 | GPIO, I2C, SPI, UART, PWM, WiFi |
| `ESP32` / `ESP32S3` | Espressif | xtensa | GPIO, I2C, SPI, UART, PWM, ADC, WiFi, BLE |
| `STM32F4` | STMicro | Cortex-M4 | GPIO, I2C, SPI, UART, PWM, ADC |
| `JetsonNano` / `JetsonOrin` | NVIDIA | aarch64 | GPIO, CUDA, GPU |
| `ArduinoUno` | Arduino | AVR | GPIO, I2C, SPI, UART, PWM, ADC |

### Sensor libraries (import `vendor.module`)

| Vendor | Modules | Sensors |
|--------|---------|---------|
| Velodyne | `vlp16`, `vlp32` | VLP-16, VLP-32C LiDAR |
| Hokuyo | `ust10`, `utm30` | UST-10LX, UTM-30LX-EW |
| Bosch | `bno055`, `bmp388` | BNO055 IMU, BMP388 barometer |
| Intel | `realsense` | RealSense D435, D455 |
| YDLIDAR | `x4`, `g4` | X4, G4 2D LiDAR |
| Adafruit | `vl53l0x` | VL53L0X ToF distance |
| SparkFun | `lsm9ds1` | LSM9DS1 9-DOF IMU |
| Waveshare | `uwmf` | Ultrasonic distance module |

### AI runtimes (import `vendor.runtime`)

| Vendor | Module | Description |
|--------|--------|-------------|
| ONNX | `onnx.runtime` | ONNX Runtime inference backend |
| TensorFlow | `tflite.runtime` | TensorFlow Lite inference backend |
| NVIDIA | `tensorrt.runtime` | TensorRT inference for Jetson |

## AI-Native Autonomous Systems

Synapse combines **robotics**, **AI agents**, **simulation**, and **safety validation** in one language. AI outputs are treated as **untrusted** by default — an LLM or vision model can propose actions, but only a `SafeAction` from `safety.validate()` may reach actuators.

### Key ideas

- **`ai_model`** — declare LLM, VisionModel, or EmbeddingModel with provider config
- **`agent`** — autonomous planner with `uses`, `tools`, `memory`, `goal`, and `plan` blocks
- **Mock backend first** — deterministic `MockAIProvider` for tests and simulation (no live API keys required)
- **Real providers later** — optional `AIProvider` interface for OpenAI, local models, etc.
- **Safety gate** — `ActionProposal` cannot call `actuator.execute()`; only `SafeAction` can

```synapse
robot Rover {
  sensor lidar: Lidar on "/scan";
  sensor camera: Camera on "/camera";
  actuator wheels: DifferentialDrive;

  ai_model planner: LLM {
    provider: "mock";
    model: "safe-planner";
    temperature: 0.1;
  }

  safety {
    max_speed = 1.0 m/s;
    stop_if lidar.nearest_distance < 0.5 m;
  }

  agent Navigator {
    uses planner;
    tools [lidar, camera, wheels];
    memory short_term;

    goal "Reach destination while avoiding obstacles";

    plan {
      let scene = camera.analyze();
      let proposal = planner.reason(
        prompt: "Create a safe navigation action",
        input: scene
      );

      let action = safety.validate(proposal);
      wheels.execute(action);
    }
  }

  behavior run() {
    loop every 100ms {
      Navigator.plan();
    }
  }
}
```

### AI function calls

```synapse
let proposal = planner.reason(prompt: "Plan safe path", input: scan);
let objects = vision.detect(camera.frame());
let summary = planner.summarize(scan);
```

### Safety rules around AI

Invalid — AI driving actuators directly:

```synapse
planner.drive(wheels);  // compile error
wheels.execute(proposal);  // compile error: requires SafeAction
```

Valid — propose, validate, then execute:

```synapse
let proposal = planner.reason(prompt: "...", input: data);
let action = safety.validate(proposal);
wheels.execute(action);
```

### AI types

| Type | Role |
|------|------|
| `LLM` | Reasoning, summarization |
| `VisionModel` | Object detection |
| `EmbeddingModel` | Vector embeddings |
| `Agent` | Goal-driven planner |
| `ActionProposal` | Untrusted AI motion suggestion |
| `SafeAction` | Safety-approved motion command |
| `Detection` / `Completion` / `Plan` | AI outputs |

### AI examples

- `examples/ai_navigation.syn` — agent + LLM planner with safety validation
- `examples/vision_pick_place.syn` — vision model for pick-and-place
- `examples/llm_robot_assistant.syn` — LLM summarization and reasoning
- `examples/ai_safety_violation.syn` — demonstrates blocked unsafe AI patterns

### Legacy inference runtimes

Import-based ONNX/TFLite/TensorRT backends remain available under `src/ai/registry.ts` for classical model inference workflows.

## Language Overview

```synapse
robot PatrolBot {
  node navigation on "/nav";
  topic cmd_vel: Velocity publish on "/cmd_vel";
  service reset_map: ResetCostmap;
  action go_to: NavigateTo;

  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  safety {
    max_speed = 1.0 m/s;
    zone restricted circle at (1.8 m, 0.0 m) radius 0.4 m;
    stop_if robot.in_zone("restricted");
  }

  behavior patrol() {
    let target = pose(x: 2.5 m, y: 0.0 m, theta: 0.0 rad);
    let path = trajectory(from: robot.pose(), to: target, steps: 8);

    publish cmd_vel with velocity(linear: 0.3 m/s, angular: 0.0 rad/s);
    call reset_map();
    send_goal go_to with target;

    loop every 100ms {
      if robot.in_zone("restricted") {
        emergency_stop;
        wheels.stop();
      } else {
        wheels.follow(path: path);
      }
    }
  }
}
```

## CLI

| Command | Description |
|---------|-------------|
| `synapse run <file.syn>` | Run with simulated backend |
| `synapse sim <file.syn>` | Run simulation with detailed output |
| `synapse check <file.syn>` | Type-check only |

## Architecture: Rust core + TypeScript tooling

Synapse uses a **dual-layer architecture**:

| Layer | Technology | Responsibility |
|-------|------------|----------------|
| **Language core** | Rust (`crates/synapse-core`) | Lexer, parser, type checker, interpreter, safety, AI mock, simulator |
| **Native CLI** | Rust (`crates/synapse-cli`) | `check`, `run`, `sim` with human or `--json` output |
| **Node bindings** | N-API (`crates/synapse-node`) | In-process calls from Node.js |
| **Browser bindings** | WASM (`crates/synapse-wasm`) | Playground and web IDE |
| **Developer UX** | TypeScript + React (`packages/web`, `src/cli`) | CLI wrapper, web playground, tests |

### Build commands

```bash
# Rust core + native CLI
npm run build:rust          # or: cargo build -p synapse-cli --release

# Rust tests (44+ unit/integration tests)
npm run test:rust           # or: cargo test --workspace

# WASM for web playground
npm run build:wasm          # requires wasm-pack

# Web IDE
npm run web:dev             # http://localhost:5173

# TypeScript tests (58 vitest cases; TS interpreter fallback)
npm test
```

The native CLI is at `target/release/synapse`. TypeScript `compile.ts` can delegate to Rust via `compileAsync(source, 'rust-cli')` or `runSource(source, { rustCli: true, ... })`.

API contract (JSON diagnostics, run results): `docs/api-contract.json`

Golden fixtures: `tests/golden/manifest.json`

## Project Structure

```
crates/
  synapse-core/    Rust language implementation
  synapse-cli/     Native synapse binary
  synapse-node/    N-API bindings
  synapse-wasm/    WebAssembly bindings
packages/
  native/          Node.js native module wrapper
  web/             React playground (Monaco-style editor)
src/
  lexer/       Tokenizer
  parser/      Recursive descent parser
  ast/         AST node definitions
  types/       Type checker with unit validation
  runtime/     Tree-walking interpreter
  simulator/   Physics-lite simulation backend
  safety/      Safety rule evaluation
  ai/          Mock AI provider, agents, memory, prompt runtime
  hal/         Hardware abstraction (I2C, SPI, GPIO, PWM, UART, ADC)
  soc/         SoC profiles and HAL validation
  lib/         Manufacturer sensor driver registry
  ros2/        ROS2 adapter stub (future hardware)
  cli/         Command-line interface
examples/      Sample Synapse programs (.syn)
tests/         Lexer, parser, type, safety, interpreter, simulator tests
```

## Core Concepts

| Concept | Description |
|---------|-------------|
| `robot` | Top-level container for a robot definition |
| `node` | ROS2-style node with namespace (`on "/nav"`) |
| `topic` | Typed publish channel (`Velocity`, `Pose`, `Scan`) |
| `service` | Request/response RPC (`ResetCostmap`, `ClearCostmap`) |
| `action` | Long-running goal (`NavigateTo`, `FollowPath`) |
| `sensor` | Input device (Lidar, IMU, GPS, AltitudeSensor, …) |
| `actuator` | Output device (DifferentialDrive, RoboticArm, DroneRotors, …) |
| `pose()` | Position `{ x, y, theta, z }` with units |
| `velocity()` | Motion `{ linear, angular }` with units |
| `trajectory()` | Interpolated path between two poses |
| `transform()` | Coordinate frame transform |
| `safety` / `zone` | Rules and geometric keep-out regions |
| `ai_model` / `agent` | AI models and goal-driven agents |
| `ActionProposal` / `SafeAction` | Untrusted AI output vs safety-approved motion |
| `emergency_stop` | Immediate halt and safety lockout |
| `behavior` | Named control loop or task |
| `loop every Nms` | Deterministic periodic execution |
| `publish` / `call` / `send_goal` | Topic, service, and action usage |

## Examples

- `examples/hello_robot.syn` — minimal robot
- `examples/differential_drive.syn` — wheeled robot motion
- `examples/lidar_avoidance.syn` — obstacle avoidance with safety
- `examples/robotic_arm_pick_place.syn` — arm pick-and-place sequence
- `examples/drone_altitude_hold.syn` — altitude control loop
- `examples/patrol_with_zones.syn` — topics, services, actions, zones, trajectories
- `examples/raspberry_pi_hal.syn` — Raspberry Pi with HAL and Velodyne/Bosch libraries
- `examples/esp32_sensors.syn` — ESP32 with multi-vendor I2C sensors
- `examples/ai_navigation.syn` — AI agent navigation with safety validation
- `examples/vision_pick_place.syn` — vision model pick-and-place
- `examples/llm_robot_assistant.syn` — LLM reasoning assistant
- `examples/ai_safety_violation.syn` — unsafe AI patterns (blocked at compile time)

## Safety Model

Safety rules in the `safety { }` block are evaluated **before every motion command**:

1. **`max_speed = X m/s`** — clamps drive velocity
2. **`stop_if <condition>`** — triggers emergency stop when true

When a safety rule blocks motion, the actuator receives a `stop()` command and the simulation enters emergency-stop state.

## ROS2 Integration (Future)

The `src/ros2/` module defines a `Ros2Adapter` interface mapping Synapse concepts to ROS2 nodes, topics, services, and actions. The current implementation is a stub for development.

## License

Apache-2.0 — see [LICENSE](LICENSE).
