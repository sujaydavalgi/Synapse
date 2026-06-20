# Changelog

All notable changes to Spanda are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
- CI: Rust tests, TypeScript tests, `cargo fmt`, `cargo clippy`, LSP, WASM, ROS2 native

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

[0.1.0-alpha]: https://github.com/sujaydavalgi/Spanda/releases/tag/v0.1.0-alpha
