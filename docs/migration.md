# Migration Guide

## From legacy `.syn` files

The `.syn` extension is no longer supported. Rename files to `.sd` and update tooling paths.

## From behavior-only programs

Existing programs continue to work unchanged. New constructs are additive:

| Old pattern | New pattern |
|-------------|-------------|
| `behavior loop()` with `loop every` inside | `task name every 20ms { ... }` |
| Ad-hoc state variables | `enum` + `match` |
| Agent tools only | Add `skill` and `can [ ... ]` for capability clarity |
| Implicit modules | Optional `module name;` + `import path;` |
| Deploy without validation | `hardware` profile + `deploy Robot to Target;` + `spanda verify` |
| Runtime-only safety checks | Add `verify { }` for post-run behavioral assertions |

## Import paths

Library imports (`import bosch.bno055;`) still resolve to sensor drivers.

Code module imports (`import sensors.lidar;`) resolve against the module registry in `foundations.rs`. Add new paths there when splitting source files.

## Breaking changes in v0.2

- New keywords: `module`, `struct`, `enum`, `trait`, `match`, `fn`, `state`, `transition`, `task`, `skill`, `event`, `twin`, `can`, `requires`, `ensures`, `invariant`, `verify`, `observe`, `remember`
- Hardware keywords: `hardware`, `deploy`, `requires_hardware`, `requires_network`, `simulate_compatibility`, `budget`, `mission`, `fault`, `sensors`, `actuators`, `network`, `bandwidth`, `latency`, `timing`, `min_period`, `duration`
- `match` expression arms use `=>` and require `;` after single-statement bodies
- `plan`, `state`, `goal`, `mission`, and other keywords may appear as identifiers in binding positions (parser allows keyword-as-name where unambiguous)
- `to` is a keyword for `deploy Robot to Target` but remains valid as a named argument (`trajectory(from: a, to: b)`)
- `battery` is a keyword in hardware profiles; HAL bindings named `battery` still parse (`adc battery on channel 0`)

## Dual backend note

The **Rust CLI** is canonical for new syntax including hardware verification:

```bash
npm run build:rust
spanda verify program.sd --target RoverV1
```

TypeScript `verifyViaCli()` prefers the native Rust CLI when available. When the CLI is missing, the npm wrapper falls back to the TypeScript parser and `verifyHardwareProgram()` for `hardware`, `deploy`, `requires_*`, geofence, and connectivity checks.

## Adding hardware verification to existing robots

1. Pick or declare a `hardware` profile (or use builtins: `RoverV1`, `ESP32`, …).
2. Add `deploy YourRobot to ProfileName;`.
3. Run `spanda verify your_program.sd`.
4. Optionally add `requires_hardware { }` for minimum platform requirements and `mission { duration: N h; }` for power checks.

See [hardware-compatibility.md](./hardware-compatibility.md).

## Lean-core package-first refactor

Spanda is moving to a **lean-core** architecture. Core keeps language semantics, safety contracts, and provider traits; domain implementations move to official packages under `packages/registry/`.

### What does not change

- All existing CLI commands (`check`, `verify`, `run`, `sim`, `fleet`, `deploy`, etc.)
- Existing `.sd` examples and tests
- Import paths like `robotics.ros2`, `communication.mqtt`, `positioning.gps`
- Parser, type checker, and runtime behavior

### Compatibility shims

Legacy core modules remain until packages fully own implementations:

| Core shim | Target package |
|-----------|----------------|
| `transport_rclrs` | `spanda-ros2` |
| `connectivity_positioning` | `spanda-gps`, `spanda-wifi`, `spanda-ble`, `spanda-cellular` |
| `nav2_adapter` | `spanda-nav` |
| `slam_adapter` | `spanda-slam` |
| `fleet_orchestrator` | `spanda-fleet` |
| `deploy_service` | `spanda-ota` |

**Removed (Phase 17):** `transport_mqtt`, `transport_dds`, `transport_websocket`, and `transport_live` no longer exist under `spanda_core`. Use `spanda_transport_routing` (`transport_live`, `live_bridges`) or the `spanda-transport-*` workspace crates directly.

No action required for existing programs. New projects should add package dependencies explicitly:

```toml
[dependencies]
spanda-ros2 = "0.1"
spanda-mqtt = "0.1"
```

### Provider traits

Packages implement traits in `spanda_core::providers` (`TransportProvider`, `SensorProvider`, etc.). See [provider-interfaces.md](./provider-interfaces.md).

### Further reading

- [lean-core.md](./lean-core.md)
- [official-packages.md](./official-packages.md)
- [security-architecture.md](./security-architecture.md)
