# Changelog

All notable changes to Spanda are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Robotics platform:** core `mission` (named steps + lifecycle), program-level `fleet`, `safety_zone`, and `certify` metadata (optional `level` block), extended `observe`/`fusion.read()` with `confidence` and `state_estimate`, `std.navigation` / `std.fusion` / `std.slam` namespaces, navigation runtime helpers; program-level safety zone speed caps (motion allowed in cap zones); TypeScript parser/type-checker and interpreter parity; Nav2 golden-path publish on `navigation.navigate()` when `/cmd_vel` is declared; **OTA deploy CLI** (`spanda deploy plan|rollout|rollback|status` with canary/staged strategies); **fleet orchestrator** (`spanda fleet orchestrate`); verify warning when deploy targets lack certification metadata; examples in `examples/robotics/`; tests in `crates/spanda-core/tests/` and `tests/robotics-platform.test.ts`
- **Robotics TS CLI parity:** TypeScript mirrors for OTA deploy service and fleet orchestrator; `spanda deploy plan|rollout|rollback|status` and `spanda fleet orchestrate` in the Node CLI without requiring the Rust binary; hardware verify warns on deploy-without-`certify` in the TS fallback
- **Robotics navigation sugar + adapter verify:** `navigate { goal: ... }` statement sugar (Rust + TS); `spanda verify` reports framework adapter mappings for imports like `navigation.nav2`; registry entries for `spanda-nav2`, `spanda-cartographer`, and `spanda-rtabmap`; example package `examples/packages/nav2_adapter_package/`
- **Remote OTA deploy agents:** HTTP deploy agent server (`spanda deploy agent start`), agent registry (`.spanda/deploy-agents.json`), `spanda deploy rollout|rollback --remote`; SLAM stub runtime (`slam.localize()` / `slam.map()`); fleet orchestrator peer-aware mode
- **OTA artifact integrity + HTTPS agents:** deploy plans include SHA-256 `program_hash`; remote rollouts send hash to agents; optional `--require-hash` on agents; HTTPS agent URLs and `--tls-cert` / `--tls-key` for deploy agents (Rust rustls + Node https); fleet peer handoff messages during orchestration; SLAM adapter example packages (`cartographer_adapter_package`, `rtabmap_adapter_package`); `examples/robotics/fleet_peer_missions.sd`
- **Signed OTA bundles + fleet mesh delivery:** Ed25519-signed deploy artifact bundles (`--sign-key`, `--bundle-out`); agents verify signatures with `--require-signature --trust-key`; fleet orchestrator delivers peer mission steps over the in-process comm mesh (`peer_mesh_mission`)
- **Distributed fleet agents + strict certify verify:** HTTP fleet peer relay agents (`spanda fleet agent start|register|list`, `.spanda/fleet-agents.json`); `spanda fleet orchestrate --remote` relays peer mission steps to registered agents (`distributed_peer_mesh`); `spanda verify --strict-certify` treats missing deploy certification, ISO13849 level gaps, and deployed-robot mission/safety metadata as errors; adapter registry metadata for Nav2/Cartographer/RTabMap packages
- **Fleet mesh coordinator + runtime certify gate + adapter production hooks:** `spanda fleet mesh start` centralizes multi-host peer relay (`--mesh-url`); fleet agents forward peer deliveries to downstream robots; `spanda run --enforce-certify` / `SPANDA_ENFORCE_CERTIFY=1` blocks uncertified deploy programs at runtime; `spanda verify-adapter` validates package `[adapter]` sections; optional `SPANDA_NAV2_CMD` / `SPANDA_SLAM_CMD` subprocess bridges for production Nav2/SLAM backends
- **Certification proof artifacts + TS adapter bridge parity:** `spanda certify prove [--strict] [--out proof.json]` emits structured checklist reports with `program_hash`; TypeScript interpreter invokes Nav2/SLAM subprocess bridges and enforces `--enforce-certify` on run; reference bridge scripts in `examples/adapters/`
- **Secure communication:** optional encrypted communication across buses, topics, services, and actions — `secure_comm` policy, `trust_boundary` declarations, `secrets` blocks (env/file), extended `secure { }` blocks with encryption/authentication/trusted sources, `EncryptedMessage`/`VerifiedMessage` types (AES-256-GCM), production transport wire frames with `source_id`, `spanda security check|audit`, `--secure` and `--inject-security-faults` CLI flags; docs in `docs/secure-communication.md`, `docs/identity.md`, `docs/secrets.md`, `docs/trust-boundaries.md`; examples in `examples/security/`
- **Secure comm TS parity:** TypeScript `RoutingCommBus` wire encryption, `secure_comm` configure fail-fast, inbound `source_id`, trust-boundary registry, static `security check|audit`, and integration tests in `tests/security-comm.test.ts`
- **Live MQTT (optional):** `live-mqtt` Cargo feature with rumqttc bridge; enable with `SPANDA_LIVE_MQTT=1`
- **Live WebSocket + DDS (optional):** `live-websocket` / `live-dds` Cargo features (or `live-transport` bundle); enable with `SPANDA_LIVE_WEBSOCKET=1` / `SPANDA_LIVE_DDS=1`
- **mTLS handshake (optional):** rustls client handshake when mutual auth + cert/key files + TLS broker URL; `SPANDA_MTLS_REQUIRED=1` fails hard; TypeScript mirror with `SPANDA_MTLS_HANDSHAKE=1`
- **Runtime trust-boundary enforcement:** publish/receive validates declared boundaries against transport-mapped crossing rules
- **Bus broker URL:** `url:` field on `bus { }` blocks and `SPANDA_BROKER_URL` env fallback for live transport and mTLS

### Fixed

- **Secure comm parser/runtime:** `secure_topic.publish` / `actuator.execute.safe` capability parsing, timed `fault … at T+10s` offsets, inbound trusted-source checks on receive/poll, TypeScript parser mirror for `secure_comm`, `trust_boundary`, `secrets`, bus blocks, and full `secure { }` fields

### Changed

- **Transport security productised:** AES-256-GCM wire frames (`spanda/wire/v1:`), `TransportWireFrame` with `source_id`, TLS session negotiation from cert/key secrets, rustls PEM validation when cert files exist, broker URL TLS scheme auto-upgrade (`mqtts://`, `wss://`), session-key derivation from robot secrets for `EncryptedMessage`, and production wire crypto (replacing mock-session stubs)
- **VS Code marketplace readiness:** bundled LSP in extension VSIX, deploy-target autocomplete, verify picker command, Spanda debug type (`editor/vscode/`)
- **Hosted package registry:** `registry/index.json` + `spanda-openai` / `spanda-ros2` tarballs; default `SPANDA_REGISTRY_URL`
- **Live AI provider:** OpenAI via Python bridge — `docs/live-ai-provider.md`, `examples/ffi_openai_live.sd`
- **Twin replay JSON export:** `spanda twin export` and `--twin-export` on run/sim
- **Web playground:** killer demo preset as default (`packages/web/`)
- **Debug workflow:** `docs/debugging.md` — step through `task every` in VS Code
- **Adoption docs:** `docs/adoption-path.md` (one-sprint Python + ROS2 wrap), `docs/ci-verify.md` (GitHub Actions / GitLab + `--json`), `docs/ros2-golden-path.md` (rclpy bridge golden path)
- **Flagship showcase index:** `examples/showcase/README.md` — three evaluator entry points (safety, verify, sim); README trimmed to match
- **End-to-end examples:** warehouse delivery, pick-and-place cell, fleet coordination, incident response, real-time patrol, validated telemetry, concurrent inspection (`examples/end_to_end/`)
- **Feature examples:** `examples/features/` (16 focused demos) plus coverage index mapping every capability to a runnable file
- **Tutorials index:** master catalog at `docs/tutorials/README.md` (all learning paths, topic guides, examples)
- **Spanda for Dummies:** plain-English guide in `docs/spanda-for-dummies/` (cheat sheet, glossary, common mistakes)
- **Spanda 101:** ten-lesson tutorial series in `docs/spanda-101/` (hello robot through end-to-end patrol)
- **Examples ladder:** `examples/basics/` (11 progressive tutorials), `examples/integration/`, and `examples/end_to_end/` (safe patrol package + replay mission)
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
- **GPS/GNSS positioning and wireless connectivity:** `requires_connectivity`, hardware `connectivity [ … ]`, WGS84 `geofence`, `connectivity_policy`, Bluetooth/BLE blocks, connectivity triggers (`on gps.lost`, `on network.disconnected`, `on gps.spoofed`), `std.positioning` / `std.connectivity` namespaces; TypeScript parser/runtime mirror with TS verify fallback and transport rebinding on failover; u-blox NEO-M8N UART GNSS stub in `lib_registry`; docs in `docs/positioning.md`, `docs/connectivity.md`, `docs/geofencing.md`, `docs/bluetooth.md`, `docs/cellular.md`; examples in `examples/connectivity/`
- **GPS fault simulation at runtime:** `GpsSpoofing` offsets coordinates and degrades fix quality; `GpsDrift` accumulates positional drift over sim time; applied to GPS sensor reads and geofence checks in Rust and TypeScript; triggers `on gps.spoofed` and `on gps.drift`
- **TypeScript hardware verify parity:** builtin profile registry, sensor/actuator/network/connectivity checks, timing and mission validation, resource budget, deploy resolution, AI model memory/GPU checks, adapter mapping, topic bandwidth estimation, and `simulate_compatibility` fault injection when Rust CLI is unavailable
- **Transport reconnect on connectivity failover:** active transport adapter connects and resubscribes topic paths when `connectivity_policy` switches links; inactive stub adapters disconnect
- **Cellular SIM identity:** `SimIdentity` type and `robot.sim_identity()` return ICCID/carrier/eSIM/attested fields; gated by `cellular.connect` under strict permissions
- **Satellite emergency backhaul:** `Satellite` connectivity token maps to websocket transport; `SatelliteOutage` fault and `emergency: satellite` failover policies; example in `examples/connectivity/satellite_backup.sd`
- **Cascade failover:** when fallback link is fault-impaired (`NetworkOutage`, `LteOutage`, etc.), runtime escalates to `emergency` link in the same step
- **Documentation sync:** migration and getting-started guides updated for TypeScript hardware verify fallback

### Changed

- `.gitignore` allows committed golden mission traces under `examples/` and `tests/golden/` while ignoring other runtime `.trace` files
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
