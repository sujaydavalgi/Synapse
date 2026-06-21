# Changelog

All notable changes to Spanda are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Cross-platform installable packages:** cargo-dist release pipeline (Linux/macOS/Windows archives, shell/PowerShell installers, Windows MSI, Homebrew formula); see [docs/installation.md](docs/installation.md)
- Deadline-aware tasks: `deadline`, `jitter <=`, `priority`, `isolated`
- Latency pipelines: `pipeline name budget Nms { … }`
- Watchdogs, operating `mode` blocks, `recover from`, `retry`/`fallback`
- First-class regex: literals, `Regex` type, string methods, triggers, subscribe filters, `validate` rules
- Mission trace replay: `spanda replay`, `--record`, `--trace-realtime`, `--metrics-json`
- Runtime telemetry: `PipelineMetrics`, `WatchdogMetrics`
- Docs: `docs/realtime.md`, `docs/reliability.md`, `docs/watchdogs.md`, `docs/degraded-modes.md`, `docs/replay.md`, `docs/regex.md`
- **Language reference:** `spanda reference`, `docs/spanda-reference.md` (JavaDoc-style `std.*`, builtins, types), `docs/man/` (man-page CLI docs)
- **Compiler API index:** `docs/api-reference.md` (Rust/TypeScript modules and public functions)
- Examples under `examples/realtime/` and `examples/regex/`

### Changed

- Canonical repository moved to [Davalgi/Spanda](https://github.com/Davalgi/Spanda) (transferred from `sujaydavalgi/Spanda`); docs and package metadata URLs updated accordingly
- Runtime now executes watchdogs (task heartbeats), `run_pipeline`, retry/fallback on injected faults, recovery handlers, jitter telemetry, and mission trace recording (`--record` writes `<file>.trace`)
- Operating `mode` blocks execute on enter; topic QoS `deadline` violations are detected at runtime
- `spanda replay --deterministic` re-runs the traced program and verifies frame parity
- TypeScript mirror syncs parse/typecheck for realtime, reliability, regex, and replay features
- Wall-clock RTOS scheduling via `--wall-clock`; frame-by-frame mission playback via `spanda replay --playback`
- Mission traces (v2) embed robot state snapshots for playback without re-running program logic

## [0.1.0-alpha] - 2026-06-20

First public alpha release. Spanda is ready for community evaluation.

### Added

**Language & runtime**
- Spanda language (`.sd`) with robot-centric syntax: sensors, actuators, safety, agents, tasks
- Physical unit type system (`m`, `s`, `rad`, `m/s`, compound units)
- AI-native agents with `ai_model`, `goal`, `memory`, and mock LLM/Vision providers
- `ActionProposal` → `safety.validate()` → `SafeAction` compile-time and runtime gate
- Safety zones, emergency stop, and behavioral `verify { }` assertions
- State machines, events, digital twins with replay buffer
- Communication primitives: `message`, `topic`, `service`, `action`
- Hardware profiles, `deploy` targets, and `requires_hardware` / `requires_network`
- Foundations: `module`, `struct`, `enum`, `trait`, `match`, generics, trait objects
- Deterministic task scheduler (`task every Nms`) with resource budgets
- Sensor fusion via `observe { }` and `fusion.read()`
- SoC/HAL profiles (Raspberry Pi, ESP32, STM32, Jetson, Arduino)
- Manufacturer sensor driver registry (Velodyne, Bosch, Intel, Hokuyo, and others)

**Tooling**
- Native CLI: `check`, `verify`, `run`, `sim`, `fmt`, `lint`, `doc`, `debug`, `ir`
- Hardware verification: `--target`, `--all-targets`, `--simulate`, `--json`
- Package manager: `init`, `build`, `test`, `install`, `add`, `remove`
- TypeScript CLI wrapper with Rust delegation
- Language Server (`@spanda/lsp`): diagnostics, completion, hover, rename, format
- Web playground (`@spanda/web`) with WASM bindings
- Debug Adapter Protocol server (`spanda-dap`)
- Experimental LLVM path: `llvm-ir`, `compile-native`

**Security & audit**
- Capability system, secrets, signed messages, audit records

**Examples**
- 72+ sample `.sd` programs including `examples/showcase/` curated demos
- Package examples under `examples/packages/`

**Documentation**
- README overhaul with positioning and architecture overview
- `docs/getting-started.md`, `docs/architecture.md`, `docs/vision.md`
- `docs/feature-status.md` with v0.1.0-alpha support matrix
- `docs/website-content.md` for future site
- Language reference, type system, hardware compatibility, packages, security docs

**Community**
- `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`
- GitHub issue templates: bug report, feature request, language proposal, package proposal
- CI: Rust tests, TypeScript tests, `cargo fmt`, `cargo clippy`, LSP, WASM, ROS2 rclrs native (Ubuntu 22.04 + Humble)

### Known limitations

- AI providers use mock backends by default; no live API keys shipped
- ROS2 integration requires manual ROS Humble setup (experimental)
- LLVM/native compilation is experimental; interpreter is the primary runtime
- Package publishing uses a local stub registry
- No published VS Code extension (LSP must be configured manually)
- Multi-robot examples run in-process; no distributed fleet runtime
- MQTT/DDS transports are parsed but not live-connected

### Roadmap (post-alpha)

- Production LLVM backend and optimized native binaries
- Published VS Code extension
- Live AI provider plugins (OpenAI, local models, ONNX)
- In-process Python/C++ FFI (PyO3, cxx)
- ROS2 production adapter with zero-config deployment
- Self-hosting compiler
- Digital twin cloud telemetry sync
- Distributed multi-robot orchestration

[0.1.0-alpha]: https://github.com/Davalgi/Spanda/releases/tag/v0.1.0-alpha

## [Unreleased]

Post-alpha improvements on `main` (2026-06-20).

### Added

**Triggers & concurrency**
- Unified trigger execution model: events, messages, timers, conditions (`when`/`while`), state, safety, hardware, AI, verification, and twin triggers
- `TriggerRegistry` with priority ordering, per-tick storm limits, and `TriggerMetrics` telemetry
- CLI trace flags: `--trace-triggers`, `--trace-events` on `run`, `sim`, and `fleet run`
- Cooperative concurrency runtime: `spawn`, `join`, `parallel`, channels, `select`, per-task `budget { }`
- `spanda fleet run` for in-process multi-robot fleet simulation with deploy/peer wiring output
- Runtime telemetry: `TaskMetrics`, `SchedulerMetrics`, `ExecutionMetrics` in `RunResult.metrics`
- TypeScript interpreter parity for concurrency and fleet peer messaging
- `examples/triggers_demo.sd`, `examples/concurrency.sd`, `examples/communication/multi_robot_fleet.sd`
- [docs/triggers.md](docs/triggers.md), [docs/concurrency.md](docs/concurrency.md), [docs/product-strategy.md](docs/product-strategy.md)

**ROS 2**
- Native `spanda-ros2-rclrs-native` cdylib for in-process ROS 2 I/O
- Persistent rclpy ROS2 daemon transport (`SPANDA_ROS2_RCLRS`)
- CI job `ros2-rclrs-native` on Ubuntu 22.04 with ROS Humble

**Developer experience**
- Inline API documentation across all Rust crates and TypeScript sources
- Contextual logic-block comments replacing generic placeholders
- Doc tooling: `scripts/add_inline_docs.py`, `scripts/add_logic_block_docs.py`, `scripts/normalize_inline_docs.py`
- VS Code extension scaffold operationalized (`editor/vscode`) with packaging workflow
- Remote registry tarball caching for offline `spanda install`

### Fixed

- Rust brace indentation after bulk inline doc insertion (`cargo fmt` compliance)
- CI: pin `ros-tooling/setup-ros@v0.7` (invalid `@v2` reference)
- Removed dead empty `if` block in type checker (clippy `needless_ifs`)

### Changed

- [CONTRIBUTING.md](CONTRIBUTING.md) documents inline documentation standards and doc scripts
- [docs/README.md](docs/README.md) indexes triggers, concurrency, and developer doc tooling
- [docs/roadmap.md](docs/roadmap.md) marks triggers and cooperative concurrency as completed
