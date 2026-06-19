# Hardware Compatibility Verification

Spanda is the first autonomous systems language with **built-in hardware compatibility verification**. The compiler answers:

> Will this program safely and correctly execute on this hardware profile before deployment?

Verification runs at **compile time** (and optionally in **simulation mode** with fault injection). It is not a simple runtime guard.

## Quick start

```bash
spanda verify examples/hardware/rover_deploy.sd
spanda verify robot.sd --target RoverV1
spanda verify robot.sd --all-targets
spanda verify robot.sd --simulate
spanda compatibility robot.sd          # alias for verify
```

Human output uses ✓ / ⚠ / ✗ per category. JSON output: `spanda verify --json file.sd`.

## Language constructs

### Hardware profiles

Declare what a physical platform provides:

```spanda
hardware RoverV1 {
  cpu: CortexA78;
  memory: 4 GB;
  storage: 32 GB;
  sensors [ Camera, Lidar, IMU ];
  actuators [ DifferentialDrive ];
  battery { capacity: 100 Wh; }
  network {
    bandwidth: 100 Mbps;
    latency: 20 ms;
  }
  timing { min_period: 10 ms; }
  resource: 15 W;
}
```

Built-in profiles: `RoverV1`, `RoverV2`, `JetsonOrin`, `RaspberryPi5`, `ESP32`. Program-declared `hardware` blocks merge into the profile registry.

### Deployment targets

```spanda
deploy RoverProgram to RoverV1;
deploy RoverProgram to [ RoverV1, ESP32, JetsonOrin ];
```

### Program requirements

```spanda
requires_hardware {
  memory >= 2 GB;
  storage >= 8 GB;
  sensors [ Camera, Lidar ];
  gpu >= 1 TOPS;
}

requires_network {
  bandwidth >= 10 Mbps;
  latency <= 50 ms;
}
```

Requirements may also appear inside a `robot { }` block.

### Resource budgets (tasks)

```spanda
task control_loop every 50ms {
  budget {
    battery <= 15%;
    memory <= 256 MB;
    cpu <= 25%;
    network <= 5 Mbps;
    storage <= 100 MB;
  }
  // task body
}
```

### Mission duration (power)

```spanda
mission { duration: 2 h; }
```

The verifier estimates energy draw (`power_draw_w × duration`) against `battery` capacity and reports errors or low-margin warnings.

### AI model compatibility

```spanda
ai_model Vision: VisionModel {
  memory_required: 512 MB;
  gpu_required: true;
}
```

Checked against target memory and GPU (TOPS / presence).

### Timing verification

Tasks (`task every Nms`) and behavior loops (`loop every Nms`) are compared to the hardware `min_period`. Aggregate CPU load from periodic work is estimated; violations produce errors or warnings.

### Sensor and actuator verification

Every declared `sensor` and `actuator` is matched against the target profile’s `sensors` / `actuators` lists. `observe { }` fused sensors are included.

### Hardware adapter mapping

Logical devices map to builtin adapter traits at verification time:

| Device type | Adapter trait |
|-------------|---------------|
| `Camera` | `CameraAdapter` |
| `Lidar` | `LidarAdapter` |
| `IMU` | `ImuAdapter` |
| `DifferentialDrive` | `MotorAdapter` |
| `RoboticArm` | `ArmAdapter` |

User-declared traits with matching names are recognized; builtins apply when hardware provides the physical device.

### Simulation and fault injection

```spanda
simulate_compatibility {
  fault CameraFailure;
  fault LidarFailure;
  fault BatteryDegradation;
  fault NetworkOutage;
  fault ImuFailure;
}
```

Use `spanda verify --simulate` or declare `simulate_compatibility` in the program. Faults mutate a copy of the target profile before checks run.

### Compatibility matrix

`spanda verify --all-targets` verifies each robot against every known hardware profile and prints a matrix:

```
── Compatibility Matrix ──
  ✓ Rover → RoverV1
  ✗ Rover → ESP32
  ✓ Rover → JetsonOrin
```

## Verification engine

```
Source → Lexer → Parser → TypeChecker → HardwareVerifier → CompatibilityReport
```

Implementation: `crates/spanda-core/src/hardware.rs`

`CompatibilityReport` contains:

- `compatible` — no error-severity items
- `target` — primary deployment target (if any)
- `items` — categorized checks (`sensors`, `actuators`, `memory`, `timing`, `power`, `network`, `ai`, `adapter`, `simulate`, …)
- `matrix` — optional robot × target grid (`--all-targets`)

Severity: `pass`, `warning`, `error`.

## IDE / LSP

`packages/lsp` runs `spanda check` and `spanda verify` on save. Compatibility issues appear as `spanda-compat` diagnostics (errors and warnings with category prefix).

TypeScript tooling can call `verifyViaCli()` from `src/rust-bridge.ts`.

## Examples

- `examples/hardware/rover_deploy.sd` — profile, deploy, sensor/actuator checks
- `examples/hardware/full_compat.sd` — requirements, budgets, mission, multi-target deploy, fault simulation

## Tests

Rust integration: `crates/spanda-core/tests/hardware_compat.rs` (10 cases)  
Rust unit: `hardware::tests` in `hardware.rs`
