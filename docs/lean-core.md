# Lean-Core Architecture

Spanda uses a **lean-core, package-first** architecture. The language core defines contracts, safety rules, the type system, runtime hooks, and verification interfaces. Domain-specific integrations — GPS drivers, Wi-Fi, ROS2, SLAM, vision models, simulators, fleet orchestration, cloud backends — live in **optional official packages**.

## Principles

| Principle | Meaning |
|-----------|---------|
| **Core = contracts** | Lexer, parser, AST, type checker, safety gate, scheduler, default simulator |
| **Packages = implementations** | Vendor protocols, hardware drivers, AI runtimes, simulation backends |
| **No feature loss** | Moved modules keep compatibility shims until migration is complete |
| **Explicit capabilities** | Packages declare required capabilities in `spanda.toml` |
| **Provider traits** | Packages implement core traits (`SensorProvider`, `TransportProvider`, etc.) |

## What stays in core

- Lexer, parser, AST, diagnostics
- Type checker and unit algebra
- Module/import system and package loader (`spanda-package`)
- Capability model and safety model (`ActionProposal` / `SafeAction`)
- Hardware verification **interfaces** (profile matching, not vendor drivers)
- Communication **interfaces** (topics, services, actions, encrypted message types)
- Trigger/task runtime and scheduler interfaces
- Identity/trust model (`spanda-security`)
- Audit/provenance interfaces (`spanda-audit`)
- Generic **provider traits** (`crates/spanda-core/src/providers/`)
- Default in-memory simulator

## What moves to packages

| Domain | Official package(s) |
|--------|---------------------|
| GPS/GNSS | `spanda-gps` |
| Wi-Fi | `spanda-wifi` |
| BLE | `spanda-ble` |
| Cellular/LTE/5G | `spanda-cellular` |
| MQTT | `spanda-mqtt` |
| DDS | `spanda-dds` |
| ROS2 | `spanda-ros2` |
| SLAM | `spanda-slam` |
| Navigation | `spanda-nav` |
| OpenCV | `spanda-opencv` |
| YOLO | `spanda-yolo` |
| MoveIt | `spanda-moveit` |
| Gazebo | `spanda-gazebo` |
| Webots | `spanda-webots` |
| Fleet | `spanda-fleet` |
| OTA deploy | `spanda-ota` |
| Maintenance | `spanda-maintenance` |
| Ledger | `spanda-ledger` |
| Cloud | `spanda-cloud` |

## Module classification

Core modules are tagged in `providers/classification.rs`:

| Ownership | Description |
|-----------|-------------|
| `Core` | Language and platform kernel |
| `StandardLibrary` | `std.*` type definitions without vendor code |
| `OfficialPackage` | First-party package under `packages/registry/` |
| `CompatibilityShim` | Legacy core module; target package named |
| `Deprecated` | Scheduled for removal after migration |

Run `spanda registry info spanda-gps` (or any official package) for manifest metadata.

## Workspace layout

```
crates/
  spanda-core/       Language + runtime kernel + provider traits
  spanda-cli/        Native CLI
  spanda-package/    Package manager (spanda.toml)
  spanda-security/   Identity, crypto, capabilities
  spanda-audit/      Audit and ledger interfaces
  spanda-llvm/       Experimental native codegen
  spanda-rt/         Native runtime ABI
  spanda-node/       Node.js bindings
  spanda-wasm/       WASM bindings
  spanda-dap/        Debug adapter

packages/registry/   Official .sd packages (spanda-gps, spanda-ros2, …)
examples/            Runnable .sd demos
docs/                Architecture and migration guides
```

## Compatibility shims

Legacy core modules (`connectivity_positioning`, `nav2_adapter`, `transport_rclrs`, etc.) remain functional where still present. Transport live bridges (`transport_mqtt`, `transport_dds`, `transport_websocket`, `transport_live`) were removed from `spanda-core`; use `spanda-transport-routing` or official transport packages. See [migration.md](./migration.md#lean-core-package-first-refactor).

## Related docs

- [provider-interfaces.md](./provider-interfaces.md) — trait contracts
- [official-packages.md](./official-packages.md) — package catalog
- [architecture.md](./architecture.md) — full system diagram
- [packages.md](./packages.md) — package manager usage
