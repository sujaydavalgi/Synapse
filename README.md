# Spanda Programming Language

**The pulse of autonomous intelligence.**

Repository: [github.com/sujaydavalgi/Spanda](https://github.com/sujaydavalgi/Spanda)

Spanda is an AI-native autonomous systems programming language for robotics, agents, human-machine interaction, digital twins, simulation, and edge intelligence.

## Philosophy

Hardware is the body.  
Sensors are the senses.  
AI models are the mind.  
Actuators are the muscles.  
Spanda is the intelligent pulse that transforms perception into action.

Source files use the **`.sd`** extension.

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
- **Hardware compatibility verification** — compile-time deploy validation against hardware profiles
- **Autonomous primitives** — `goal`, `memory`, `verify { }`, `observe { }`, multi-task scheduling
- **Foundations** — `module`, `struct`, `enum`, `trait`, `match`, `task`, `state_machine`, `twin`, `event`
- **ROS2 adapter interface** — stub ready for future integration

## Quick Start

```bash
git clone https://github.com/sujaydavalgi/Spanda.git
cd Spanda
npm install
npm test
npm run build
npm run lint
npm run spanda -- run examples/rover.sd
npm run spanda -- sim examples/rover.sd
npm run spanda -- check examples/rover.sd
npm run spanda:native -- verify examples/hardware/rover_deploy.sd
```

Native CLI (after `npm run build:rust`):

```bash
spanda check examples/rover.sd
spanda verify examples/hardware/rover_deploy.sd --target RoverV1
spanda verify robot.sd --all-targets
spanda run examples/rover.sd
spanda sim examples/rover.sd
spanda fmt examples/rover.sd
```

## HAL, SoC, and Sensor Libraries

```spanda
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

Spanda combines **robotics**, **AI agents**, **simulation**, and **safety validation** in one language. AI outputs are treated as **untrusted** by default — an LLM or vision model can propose actions, but only a `SafeAction` from `safety.validate()` may reach actuators.

### Key ideas

- **`ai_model`** — declare LLM, VisionModel, or EmbeddingModel with provider config
- **`agent`** — autonomous planner with `uses`, `tools`, `memory`, `goal`, and `plan` blocks
- **Mock backend first** — deterministic `MockAIProvider` for tests and simulation (no live API keys required)
- **Real providers later** — optional `AIProvider` interface for OpenAI, local models, etc.
- **Safety gate** — `ActionProposal` cannot call `actuator.execute()`; only `SafeAction` can

```spanda
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

```spanda
let proposal = planner.reason(prompt: "Plan safe path", input: scan);
let objects = vision.detect(camera.frame());
let summary = planner.summarize(scan);
```

### Safety rules around AI

Invalid — AI driving actuators directly:

```spanda
planner.drive(wheels);  // compile error
wheels.execute(proposal);  // compile error: requires SafeAction
```

Valid — propose, validate, then execute:

```spanda
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

- `examples/ai_navigation.sd` — agent + LLM planner with safety validation
- `examples/vision_pick_place.sd` — vision model for pick-and-place
- `examples/llm_robot_assistant.sd` — LLM summarization and reasoning
- `examples/ai_safety_violation.sd` — demonstrates blocked unsafe AI patterns

### Legacy inference runtimes

Import-based ONNX/TFLite/TensorRT backends remain available under `src/ai/registry.ts` for classical model inference workflows.

## Hardware Compatibility Verification

Spanda verifies that autonomous programs **fit the deployment target before they run on hardware** — sensors, actuators, memory, GPU, timing, network, power, and AI model requirements.

```spanda
hardware RoverV1 {
  cpu: CortexA78;
  memory: 4 GB;
  sensors [ Camera, Lidar, IMU ];
  actuators [ DifferentialDrive ];
  battery { capacity: 100 Wh; }
  timing { min_period: 10 ms; }
}

requires_hardware {
  memory >= 2 GB;
  sensors [ Camera, Lidar ];
}

robot RoverProgram {
  sensor camera: Camera on "/camera";
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  ai_model Vision: VisionModel {
    memory_required: 512 MB;
    gpu_required: false;
  }

  mission { duration: 1 h; }

  task control every 50ms {
    budget { cpu <= 25%; memory <= 256 MB; }
    wheels.drive(linear: 0.2 m/s, angular: 0.0 rad/s);
  }
}

deploy RoverProgram to RoverV1;
```

```bash
spanda verify rover.sd
spanda verify rover.sd --target RoverV1
spanda verify rover.sd --all-targets    # compatibility matrix
spanda verify rover.sd --simulate     # fault injection
```

Built-in profiles: `RoverV1`, `RoverV2`, `JetsonOrin`, `RaspberryPi5`, `ESP32`.

Full reference: [docs/hardware-compatibility.md](docs/hardware-compatibility.md)

### Hardware examples

- `examples/hardware/rover_deploy.sd` — deploy to custom or builtin profile
- `examples/hardware/full_compat.sd` — requirements, budgets, multi-target, simulation

## Language Overview

```spanda
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
| `spanda check <file.sd>` | Type-check only |
| `spanda verify <file.sd>` | Hardware compatibility verification |
| `spanda verify --target <Profile>` | Verify against a specific hardware profile |
| `spanda verify --all-targets` | Generate robot × profile compatibility matrix |
| `spanda verify --simulate` | Run with fault injection scenarios |
| `spanda compatibility <file.sd>` | Alias for `verify` |
| `spanda run <file.sd>` | Run with simulated backend |
| `spanda sim <file.sd>` | Run simulation with detailed output |
| `spanda fmt <file.sd>` | Format source file |

All commands support `--json` for machine-readable output.

## Architecture: Rust core + TypeScript tooling

Spanda uses a **dual-layer architecture**:

| Layer | Technology | Responsibility |
|-------|------------|----------------|
| **Language core** | Rust (`crates/spanda-core`) | Lexer, parser, type checker, interpreter, safety, AI, simulator, **hardware verifier** |
| **Native CLI** | Rust (`crates/spanda-cli`) | `check`, `verify`, `run`, `sim`, `fmt` with human or `--json` output |
| **Node bindings** | N-API (`crates/spanda-node`) | In-process calls from Node.js |
| **Browser bindings** | WASM (`crates/spanda-wasm`) | Playground and web IDE |
| **Language Server** | TypeScript (`packages/lsp`) | Type-check + hardware compatibility diagnostics |
| **Developer UX** | TypeScript + React (`packages/web`, `src/cli`) | CLI wrapper, web playground, tests |

### Build commands

```bash
# Rust core + native CLI
npm run build:rust          # or: cargo build -p spanda-cli --release

# Rust tests (115+ unit/integration tests)
npm run test:rust           # or: cargo test --workspace

# TypeScript tests (121 vitest cases)
npm test

# WASM for web playground
npm run build:wasm          # requires wasm-pack

# Web IDE
npm run web:dev             # http://localhost:5173

# TypeScript tests (58 vitest cases; TS interpreter fallback)
npm test
```

The native CLI is at `target/release/spanda`. TypeScript `compile.ts` can delegate to Rust via `compileAsync(source, 'rust-cli')` or `runSource(source, { rustCli: true, ... })`. Hardware verification uses `verifyViaCli()` from `src/rust-bridge.ts`.

API contract (JSON diagnostics, run results): `docs/api-contract.json`

Golden fixtures: `tests/golden/manifest.json`

Documentation index: `docs/README.md`

## Project Structure

```
crates/
  spanda-core/    Rust language implementation
  spanda-cli/     Native spanda binary
  spanda-node/    N-API bindings
  spanda-wasm/    WebAssembly bindings
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
  hardware/    Hardware profiles and compatibility verification (Rust); types in foundations.ts
  lib/         Manufacturer sensor driver registry
  ros2/        ROS2 adapter stub (future hardware)
  cli/         Command-line interface
examples/      Sample Spanda programs (.sd)
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
| `task every Nms` | Deterministic periodic task with optional `budget { }` |
| `loop every Nms` | Deterministic periodic execution inside behaviors |
| `hardware` / `deploy` | Hardware profile and deployment target binding |
| `requires_hardware` / `requires_network` | Program-level deployment requirements |
| `mission { duration }` | Mission duration for power budgeting |
| `simulate_compatibility` | Fault injection scenarios for verification |
| `verify { }` | Behavioral assertions checked after execution |
| `observe { }` / `fusion.read()` | Multi-sensor fusion |
| `goal` / `remember` / `recall` | Agent goals and memory store |
| `publish` / `call` / `send_goal` | Topic, service, and action usage |

## Examples

- `examples/rover.sd` — minimal robot
- `examples/differential_drive.sd` — wheeled robot motion
- `examples/lidar_avoidance.sd` — obstacle avoidance with safety
- `examples/robotic_arm_pick_place.sd` — arm pick-and-place sequence
- `examples/drone_altitude_hold.sd` — altitude control loop
- `examples/patrol_with_zones.sd` — topics, services, actions, zones, trajectories
- `examples/warehouse_logistics.sd` — rect zones, transforms, FollowPath, ClearCostmap, SetPose
- `examples/pick_object_action.sd` — PickObject action with arm and force sensing
- `examples/outdoor_navigation.sd` — IMU heading fused with lidar avoidance
- `examples/jetson_inspection.sd` — Jetson SoC with vision agent inspection loop
- `examples/stm32_motor_control.sd` — STM32 HAL with PWM, GPIO, ADC, and IMU
- `examples/raspberry_pi_hal.sd` — Raspberry Pi with HAL and Velodyne/Bosch libraries
- `examples/esp32_sensors.sd` — ESP32 with multi-vendor I2C sensors
- `examples/ai_navigation.sd` — AI agent navigation with safety validation
- `examples/vision_pick_place.sd` — vision model pick-and-place
- `examples/llm_robot_assistant.sd` — LLM reasoning assistant
- `examples/ai_safety_violation.sd` — unsafe AI patterns (blocked at compile time)
- `examples/hardware/rover_deploy.sd` — hardware profile + deploy verification
- `examples/hardware/full_compat.sd` — full compatibility feature showcase
- `examples/types/goals.sd` — `Goal` type and agent goal injection
- `examples/types/memory.sd` — `remember` / `recall`
- `examples/types/verify.sd` — behavioral `verify { }` block
- `examples/types/multitask.sd` — multi-task tick multiplexer
- `examples/types/fusion.sd` — `observe` and sensor fusion
- `examples/humanoid_assistant.sd` — humanoid agent example
- `examples/hello_world.sd` — minimal program

## Safety Model

Safety rules in the `safety { }` block are evaluated **before every motion command**:

1. **`max_speed = X m/s`** — clamps drive velocity
2. **`stop_if <condition>`** — triggers emergency stop when true

When a safety rule blocks motion, the actuator receives a `stop()` command and the simulation enters emergency-stop state.

## ROS2 Integration (Future)

The `src/ros2/` module defines a `Ros2Adapter` interface mapping Spanda concepts to ROS2 nodes, topics, services, and actions. The current implementation is a stub for development.

## License

Apache-2.0 — see [LICENSE](LICENSE).
