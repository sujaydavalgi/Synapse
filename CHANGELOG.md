# Changelog

All notable changes to Spanda are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-06-22

### Added

- **Native deploy path:** `spanda deploy --target native` links LLVM binaries (same pipeline as `compile-native`); guide [native-deploy.md](docs/native-deploy.md).
- **ROS 2 polish:** `spanda ros2 check [--json]` validates `ROS_DISTRO`, rclpy, and bridge script before live transport.
- **Distributed fleet docs:** [fleet-distributed.md](docs/fleet-distributed.md) for `--remote` orchestration, agent registry, and OTA rollout.
- **CI:** `live-iot-golden-path` job runs `scripts/live_iot_golden_path.sh`.

## [0.3.0] - 2026-06-22

### Added

- **Install ergonomics:** crate renamed to `spanda` (`cargo install --path crates/spanda-cli` installs binary `spanda`); `spanda --version`; bundled showcase examples ship in the crate for `spanda demo` without a full clone; `scripts/sync_bundled_examples.sh`.
- **Productization (credibility & demos):** `spanda demo {rover,safety,verify,fleet,health}`; showcase directories under `examples/showcase/`; `scripts/install.sh`, `scripts/benchmark.sh`, `scripts/showcase_smoke.sh`; docs [benchmarks.md](docs/benchmarks.md), [known-limitations.md](docs/known-limitations.md), [demo-script.md](docs/demo-script.md), [diagrams/](docs/diagrams/); README trust table; improved `SafeAction` type errors with hints; VS Code snippets; CI `showcase-smoke` job.
- **LSP v0.3 polish:** keyword hover for `ActionProposal`, `SafeAction`, `safety.validate`, `deploy`, `health_check`, `kill_switch`; SafeAction quick-fix code action.

### Changed

- CI and golden-path scripts use `cargo build -p spanda` (package rename from `spanda-cli`).

### Fixed

- **Multi-robot fleet runtime:** interpreter now setup+executes each robot in isolation so `spanda fleet run` works when the last robot lacks member actuators (e.g. coordinator-only programs).

## [0.2.0] - 2026-06-23
### Added

- **Documentation audit:** new guides [verification-diagnostics.md](docs/verification-diagnostics.md), [typed-handler-io.md](docs/typed-handler-io.md), [testing.md](docs/testing.md); expanded [fleet-health.md](docs/fleet-health.md), [swarm-health.md](docs/swarm-health.md), [kill-switch.md](docs/kill-switch.md), [capability-traceability.md](docs/capability-traceability.md), [agentic-programming.md](docs/agentic-programming.md); example indexes under [examples/hardware/README.md](examples/hardware/README.md), [examples/iot/README.md](examples/iot/README.md).
- **Examples (Phase 27–35 coverage):** `features/kill_switch.sd`, `fleet_health_require.sd`, `typed_handler_returns.sd`, `agent_can_deny.sd`, `live_openai.sd`, `live_anthropic.sd`, `live_onnx.sd`, `security/remote_signed_kill_switch.sd`, `integration/debugger_every.sd`, `basics/12_compile_fail_tests.sd`, `iot/modbus_dispatch/`, `packages/publish_mirror_project/`.

### Added

- **Verification & DX (Phase 28):** `expect_compile_error { }` blocks in test bodies — validated at test-run time; module function return type enforcement in the typechecker; TypeScript parser mirror for Phase 27 syntax (`kill_switch`, `health_check`, `health_policy`, `requires_capability`, hardware components, robot `uses hardware` / `exposes capabilities`); IoT protocol package stubs (`spanda-opcua`, `spanda-modbus`, `spanda-zigbee`, `spanda-lora`, `spanda-matter`, `spanda-canbus`); integration tests in `p1_features.rs` and `tests/capability-parser.test.ts`.
- **Verification & DX (Phase 29):** span-aware verification diagnostics for capability, traceability, minimum-hardware, health, and kill-switch checks; `spanda check --verification-json`; LSP integration; runtime health evaluation wired to `HardwareMonitor`; kill-switch and health-fault sim integration tests.
- **Verification & DX (Phase 30):** `suggested_fix` hints on verification diagnostics; LSP quick-fix code actions; continuous runtime health polling during trigger maintenance; debugger pause events for kill switch (`kill_switch_activated`) and critical health (`health_critical`).
- **Verification & DX (Phase 31):** runtime `health_policy` enforcement; behavior `-> Type` return validation; agent plan `SafeAction` return checks; IoT package dispatch stubs; agent capability audit logging; DAP output events for health/kill-switch.
- **Verification & DX (Phase 32):** in-memory IoT hub; task return types; agent `can[]` default-deny; VS Code VSIX verify script.
- **Verification & DX (Phase 33):** trigger handler `-> Type` return validation; live Modbus TCP and OPC-UA bridge IoT paths; live OpenAI provider for `ai_model` when `OPENAI_API_KEY` is set.
- **Verification & DX (Phase 34):** event handler I/O verification; kill switch `remote_signed` runtime enforcement and `on kill_switch` handlers; VS Code extension CI; IoT protocol dispatch stubs; live Anthropic provider; fleet/swarm health runtime coordination.
- **Verification & DX (Phase 35):** TypeScript build parity; live IoT bridges for zigbee/lora/matter/canbus; fleet health `require` runtime; ONNX provider; registry mirror publish; kill-switch verify errors; debugger `every` entry.
- **Verification & DX (Phase 27):** `spanda-capability` crate — capability registry, hardware/robot capability inference, traceability matrices, minimum-hardware safety checks, health-check analysis; CLI commands `spanda trace {hardware|capabilities|health}`, `spanda health robot`, `spanda hardware capabilities`, `spanda robot capabilities`, `spanda safety check --capabilities`; verify flags `--traceability`, `--capabilities`, `--health`, `--minimum-capabilities`; hardened `spanda test` with file paths, `--json`, `--filter`, `--compile-fail`; language syntax for `kill_switch`, `health_check`, `health_policy`, `requires_capability`, `uses hardware`, `exposes capabilities`; sim flags `--trigger-kill-switch`, `--inject-health-faults`; IoT provider contracts in `spanda-runtime`; `spanda-iot-core` package stub; mdBook site at `docs-site/` + GitHub Pages workflow; guides for kill switch, health, capabilities, traceability, IoT, agentic programming, debugger. (`dispatch_official_package_call`); connectivity provider stubs for Wi-Fi/BLE/cellular; `--trace-providers` observability flag; `spanda update` command; flagship demo at `examples/showcase/autonomous_rover/`; guides [how-packages-work.md](docs/how-packages-work.md), [how-providers-work.md](docs/how-providers-work.md), [how-runtime-resolution-works.md](docs/how-runtime-resolution-works.md).
- **Platform integration (phase 2):** transitive dependency resolution; SLAM/vision/simulation provider dispatch; `provider_call` mission-trace frames; aligned provider capabilities in validation/security; TS `package_dispatch.ts` mirror; project-aware module registry for check/build/test/run/verify.
- **Phase 23 CI golden paths:** fleet `--remote` in `golden_path_deploy.sh`; live MQTT Mosquitto job (`scripts/mqtt_golden_path.sh`, `examples/communication/mqtt_live.sd`); twin cloud upload job (`scripts/twin_cloud_golden_path.sh`); LLVM job (`scripts/llvm_golden_path.sh`); `live-mqtt` / `live-transport` Cargo features on `spanda-cli`.
- **Phase 23 completion:** `world_model { }` parser + observe→fusion belief hook; ledger community scaffold (`packages/community/`, `scripts/ledger_golden_path.sh`); cpp-native and self-host lexer golden paths; Phase 23 marked complete in roadmap.
- **Phase 26 (in progress):** P1 adoption golden paths — `ci_verify_golden_path.sh`, `python_native_golden_path.sh`; `python-native` feature on `spanda-cli`; Phase 26 on roadmap. — `killer_demo_golden_path.sh`, `live_ai_golden_path.sh`, `ros2_golden_path.sh`, `registry_golden_path.sh`; CI jobs; [ros2_cmd_vel_ping.sd](examples/communication/ros2_cmd_vel_ping.sd); VS Code `vsce publish` on release when `VSCE_PAT` is set; P0 status table in [tier-3-priority-plan.md](docs/tier-3-priority-plan.md).
- **CI fix:** `cargo fmt` drift; TypeScript build parity (lexer tokens, compile stub, package dispatch); release workflow `VSCE_PAT` guard without invalid `secrets` in `if`.
- **Phase 24 (complete):** `world_model_patrol.sd` showcase; `fleet_field_trial.sd` three-agent layout; [tier-3-golden-paths.md](docs/tier-3-golden-paths.md) index; `world-model-golden-path` CI; typechecker support for `world_model.belief()` / `update()` / `export()`; [mqtt-nav2-reference-architecture.md](docs/mqtt-nav2-reference-architecture.md); [llvm-embedded-benchmark.md](docs/llvm-embedded-benchmark.md) + `llvm-embedded-golden-path` CI; TS mirror parser/checker parity for `world_model { }`.

- **Tier 3 priority plan:** [tier-3-priority-plan.md](docs/tier-3-priority-plan.md) documents P0–P4 ordering (v0.5 beta → Phase 23 hardening → v1.0 optional → post-v1.0 production); Phase 23 planned in [lean-core-roadmap.md](docs/lean-core-roadmap.md).

- **Phase 22 Tier 3 experimental:** world-model runtime (`world_model.update`/`belief`/`export`), ledger provider wired to `MockLedgerBackend`, cloud upload via `SPANDA_CLOUD_UPLOAD_URL`, LLVM golden path script, self-host bootstrap example, and [tier-3-experimental.md](docs/tier-3-experimental.md).
- **Phase 18 P2/P3 closure:** performance (slim CLI, bridge timeouts, `cargo audit`) and observability (pipeline benchmark) marked complete in docs.

- **Phase 21 hosted registry signing:** `registry-index-maintain` binary updates checksums and Ed25519 `version_signatures` in `registry/index.json`; CI verifies against `registry/TRUST_KEY`; `scripts/update_registry_checksums.py` delegates to the Rust tool.
- **Phase 21 embedder slimming:** optional `certify` and `bridge` features on `spanda-core` (`default-features = false` omits certification and FFI shims; `full` remains default).

- **Automated version bumps:** `scripts/bump_version.py` bumps `Cargo.toml`, npm packages, and finalizes `CHANGELOG.md`. **Auto release** runs after CI on `main` when a merged PR has `release:major`, `release:minor`, or `release:patch`; **Bump version** (manual Actions workflow) is available for ad-hoc releases. Both push `v*` tags that trigger cargo-dist **Release** builds.

- **Phase 18 security hardening:** registry tarball SHA-256 verification and tar-slip-safe extraction in `spanda-package`; deploy/fleet/mesh agents require `--token` on non-loopback binds; bridge subprocess timeouts; `cargo audit` CI job; slim CLI build (`--no-default-features --features slim`); pipeline benchmark test; [phase-18-security-hardening.md](docs/phase-18-security-hardening.md).
- **Phase 18b signed registry:** Ed25519 `version_signatures` on publish/install via `SPANDA_REGISTRY_SIGN_KEY` / `SPANDA_REGISTRY_TRUST_KEY`.
- **Phase 19 transport shim removal:** dropped `spanda_core::transport*` modules; `spanda-core` no longer depends on transport adapter crates directly.
- **Phase 20 test distribution + embedder features:** OTA, fleet, provider, and certify integration tests moved to owning crates; `spanda-core` exposes optional `ota` / `fleet` features (`default = ["full"]`; `--no-default-features` for minimal embedder builds).

- **Interpreter architecture docs:** [architecture.md](docs/architecture.md) documents the modular `spanda-interpreter` runtime tree, one-way `spanda-core` → `spanda-interpreter` dependency, and `CoreRuntimeHost` wiring (see [lean-core-roadmap.md](docs/lean-core-roadmap.md)).
- **Hosted registry (20 packages):** `registry/index.json` and tarballs for all official packages under `packages/registry/`; `./scripts/build-registry.sh` auto-discovers package scaffolds; [registry.md](docs/registry.md) curated table updated.
- **Killer demo:** flagship program at `examples/showcase/killer_demo.sd` with walkthrough in [killer-demo.md](docs/killer-demo.md) (check → verify → sim narrative).
- **Hosted registry tests:** `crates/spanda-package/tests/hosted_registry.rs` guards 20-package index, tarballs, and `file://` fetch; `killer_demo.sd` added to golden manifest.
- **Interpreter Phase 8 (partial):** moved `triggers`, `telemetry`, `replay`, `twin`, `events`, `state_machine`, `reliability_runtime`, and `serialize` into `spanda-runtime` with thin `spanda-core` shims; interpreter runtime routes imports through workspace crates; `RuntimeError` implements `Display`.
- **`spanda-comm` crate:** extracts `CommBus`, `InMemoryCommBus`, comm safety chain, and bandwidth helpers from `spanda-core` with a thin shim; interpreter runtime imports `spanda_comm::CommBus` directly.
- **`spanda-safety` crate:** extracts `SafetyMonitor`, zones, `Pose2d`, and motion validation from `spanda-core` with a thin shim; interpreter runtime imports `spanda_safety` directly.
- **`spanda-hal` crate:** extracts HAL simulation backend, SoC profiles, and hardware health monitoring from `spanda-core` with thin shims; interpreter runtime imports `spanda_hal` directly.
- **`spanda-transport` routing:** moves wire-frame encode/decode into `spanda-transport` and `RoutingCommBus` into new `spanda-transport-routing` (avoids adapter-backend cycle); thin `transport` / `transport_wire` shims; interpreter runtime imports `spanda_transport_routing::RoutingCommBus` directly.
- **`spanda-error` crate:** extracts `SpandaError` and diagnostic helpers from `spanda-core`; interpreter runtime imports `spanda_error::SpandaError` directly; `RunOptions` / `RunResult` remain in core.
- **`spanda-ai` crate:** extracts AI model registry, agent runtime, memory store, and mock inference helpers from `spanda-core` with a thin shim; interpreter runtime imports `spanda_ai` directly.
- **`spanda-providers` crate:** extracts official package bootstrap, stubs, and transport adapter wiring; interpreter imports `spanda_providers` for registry bootstrap and comm-bus sync.
- **`spanda-concurrency` crate:** extracts cooperative channels, spawn handles, and select; thin core shim; interpreter imports `spanda_concurrency` directly.
- **`spanda-debug` crate:** extracts debugger controller, breakpoints, and `stmt_line`; interpreter imports `spanda_debug` directly.
- **Phase 8 routing complete:** `spanda-regex-lang`, `spanda-lib-registry`, `spanda-connectivity-runtime`, `spanda-runtime-host`, and `spanda-ffi` extracted with thin shims; interpreter runtime has zero `crate::` imports.
- **Phase 11 SIR extraction:** `spanda-sir` crate owns AST lowering to typed intermediate representation; `spanda-core` re-exports via thin shim.
- **Phase 10 run pipeline:** `spanda-certify` and `spanda-bridge` extracted; `spanda-driver::run` owns compile + certify gate + FFI defaults + interpreter execution; `spanda-core` re-exports the public API.
- **Phase 12 tooling extraction:** `spanda-hardware` (full verify + adapter verify + connectivity validators), `spanda-format`, `spanda-lint`, `spanda-codegen`, `spanda-modules`, and `spanda-docs` extracted with thin core shims; `spanda-security::validate` owns static security audit; `spanda-driver::debug_session` owns the debugger machine; `spanda-fleet::swarm_coordinator` owns swarm coordination; connectivity-runtime re-exports hardware validators to preserve the public API.
- **Phase 13 facade slim-down:** `spanda-driver` now owns verify, SIR lowering, replay/playback, debug run, deploy plan (with certify proof), and type-check host wiring; `spanda-ota::plan` extracts deploy assignments from the AST; reliability validators live in `spanda-typecheck`; `spanda-core` re-exports the public API without local pipeline bodies.
- **Phase 14 shim consolidation:** `transport_live` RuntimeValue hooks live in `spanda-transport-routing`; lexer `tokenize` error mapping and FFI bridge alias moved to `spanda-driver` / `spanda-bridge`; `providers/` collapsed to a single facade module.
- **Phase 15 caller migration:** `spanda-cli` and `spanda-node` import workspace crates directly (`spanda-driver`, OTA/fleet/deploy-http, tooling crates); MQTT/DDS/WebSocket `RuntimeValue` live bridges consolidated in `spanda-transport-routing::live_bridges` with thin core shims retained for API stability.
- **Phase 16 caller migration:** `spanda-llvm`, `spanda-wasm`, and `spanda-dap` no longer depend on `spanda-core`; only the `spanda-core` facade crate itself pulls the full workspace graph for external API stability.
- **Phase 17 transport shim removal:** removed `spanda_core::transport_mqtt`, `transport_dds`, `transport_websocket`, and `transport_live`; use `spanda-transport-routing` or `spanda-transport-*` workspace crates directly.
- **Documentation refresh:** rewritten lean-core guide, workspace crate index, per-crate READMEs; **tutorials & examples hub** (`examples/README.md`, `examples/packages/README.md`, updated tutorial indexes and learning paths); **API doc hierarchy** ([api-documentation.md](docs/api-documentation.md), grouped [api-reference.md](docs/api-reference.md) with facade→crate mapping).

### Changed core `mission` (named steps + lifecycle), program-level `fleet`, `safety_zone`, and `certify` metadata (optional `level` block), extended `observe`/`fusion.read()` with `confidence` and `state_estimate`, `std.navigation` / `std.fusion` / `std.slam` namespaces, navigation runtime helpers; program-level safety zone speed caps (motion allowed in cap zones); TypeScript parser/type-checker and interpreter parity; Nav2 golden-path publish on `navigation.navigate()` when `/cmd_vel` is declared; **OTA deploy CLI** (`spanda deploy plan|rollout|rollback|status` with canary/staged strategies); **fleet orchestrator** (`spanda fleet orchestrate`); verify warning when deploy targets lack certification metadata; examples in `examples/robotics/`; tests in `crates/spanda-core/tests/` and `tests/robotics-platform.test.ts`
- **Robotics TS CLI parity:** TypeScript mirrors for OTA deploy service and fleet orchestrator; `spanda deploy plan|rollout|rollback|status` and `spanda fleet orchestrate` in the Node CLI without requiring the Rust binary; hardware verify warns on deploy-without-`certify` in the TS fallback
- **Robotics navigation sugar + adapter verify:** `navigate { goal: ... }` statement sugar (Rust + TS); `spanda verify` reports framework adapter mappings for imports like `navigation.nav2`; registry entries for `spanda-nav2`, `spanda-cartographer`, and `spanda-rtabmap`; example package `examples/packages/nav2_adapter_package/`
- **Remote OTA deploy agents:** HTTP deploy agent server (`spanda deploy agent start`), agent registry (`.spanda/deploy-agents.json`), `spanda deploy rollout|rollback --remote`; SLAM stub runtime (`slam.localize()` / `slam.map()`); fleet orchestrator peer-aware mode
- **OTA artifact integrity + HTTPS agents:** deploy plans include SHA-256 `program_hash`; remote rollouts send hash to agents; optional `--require-hash` on agents; HTTPS agent URLs and `--tls-cert` / `--tls-key` for deploy agents (Rust rustls + Node https); fleet peer handoff messages during orchestration; SLAM adapter example packages (`cartographer_adapter_package`, `rtabmap_adapter_package`); `examples/robotics/fleet_peer_missions.sd`
- **Signed OTA bundles + fleet mesh delivery:** Ed25519-signed deploy artifact bundles (`--sign-key`, `--bundle-out`); agents verify signatures with `--require-signature --trust-key`; fleet orchestrator delivers peer mission steps over the in-process comm mesh (`peer_mesh_mission`)
- **Distributed fleet agents + strict certify verify:** HTTP fleet peer relay agents (`spanda fleet agent start|register|list`, `.spanda/fleet-agents.json`); `spanda fleet orchestrate --remote` relays peer mission steps to registered agents (`distributed_peer_mesh`); `spanda verify --strict-certify` treats missing deploy certification, ISO13849 level gaps, and deployed-robot mission/safety metadata as errors; adapter registry metadata for Nav2/Cartographer/RTabMap packages
- **Fleet mesh coordinator + runtime certify gate + adapter production hooks:** `spanda fleet mesh start` centralizes multi-host peer relay (`--mesh-url`); fleet agents forward peer deliveries to downstream robots; `spanda run --enforce-certify` / `SPANDA_ENFORCE_CERTIFY=1` blocks uncertified deploy programs at runtime; `spanda verify-adapter` validates package `[adapter]` sections; optional `SPANDA_NAV2_CMD` / `SPANDA_SLAM_CMD` subprocess bridges for production Nav2/SLAM backends
- **Certification proof artifacts + TS adapter bridge parity:** `spanda certify prove [--strict] [--out proof.json]` emits structured checklist reports with `program_hash`; TypeScript interpreter invokes Nav2/SLAM subprocess bridges and enforces `--enforce-certify` on run; reference bridge scripts in `examples/adapters/`
- **Deploy certification gate:** deploy plans embed certification proof summaries; `spanda deploy rollout --require-certify` blocks OTA when strict proof fails; deploy agents accept `--require-certify` to reject rollouts missing strict proof in the payload; TypeScript deploy-service and `verify-adapter` Node fallback mirror Rust behavior
- **Swarm coordinator (experimental):** program-level `swarm` declarations with `round_robin`, `broadcast`, and `leader_follow` policies; `spanda swarm coordinate` runtime with persistent round-robin cursors in `.spanda/swarm-state.json`; TypeScript parser/checker/coordinator parity
- **Robotics golden path script:** `examples/robotics/golden_path_deploy.sh` now covers certify, deploy, verify-adapter, fleet orchestrate, and swarm coordinate
- **Swarm mesh relay:** `spanda swarm coordinate --mesh-url` relays leader-follow peer deliveries through the fleet mesh coordinator; CI `robotics-golden-path` job runs the golden-path script against the release CLI
- **Swarm peer mesh parity:** round_robin and broadcast policies collect peer-link deliveries for mesh relay; golden path covers mesh fleet/swarm, remote OTA dry-run, and Nav2/SLAM adapter bridge fixtures

### Fixed

- **Fleet mesh CLI routing:** `spanda fleet mesh start` now receives the correct subcommand args (was treating `mesh` as the subcommand and exiting with usage)
- **Fleet mesh registry reload:** mesh coordinator reloads `SPANDA_FLEET_AGENTS` on each relay request instead of snapshotting at startup; fleet agents honor the same env for downstream forwarding
- **Swarm mesh peer delivery:** round_robin/broadcast include peer-link deliveries; leader_follow avoids duplicate peer/member handoffs
- **Secure communication:** optional encrypted communication across buses, topics, services, and actions — `secure_comm` policy, `trust_boundary` declarations, `secrets` blocks (env/file), extended `secure { }` blocks with encryption/authentication/trusted sources, `EncryptedMessage`/`VerifiedMessage` types (AES-256-GCM), production transport wire frames with `source_id`, `spanda security check|audit`, `--secure` and `--inject-security-faults` CLI flags; docs in `docs/secure-communication.md`, `docs/identity.md`, `docs/secrets.md`, `docs/trust-boundaries.md`; examples in `examples/security/`
- **Secure comm TS parity:** TypeScript `RoutingCommBus` wire encryption, `secure_comm` configure fail-fast, inbound `source_id`, trust-boundary registry, static `security check|audit`, and integration tests in `tests/security-comm.test.ts`
- **Live MQTT (optional):** `live-mqtt` Cargo feature with rumqttc bridge; enable with `SPANDA_LIVE_MQTT=1`
- **Live WebSocket + DDS (optional):** `live-websocket` / `live-dds` Cargo features (or `live-transport` bundle); enable with `SPANDA_LIVE_WEBSOCKET=1` / `SPANDA_LIVE_DDS=1`
- **mTLS handshake (optional):** rustls client handshake when mutual auth + cert/key files + TLS broker URL; `SPANDA_MTLS_REQUIRED=1` fails hard; TypeScript mirror with `SPANDA_MTLS_HANDSHAKE=1`
- **Runtime trust-boundary enforcement:** publish/receive validates declared boundaries against transport-mapped crossing rules
- **Bus broker URL:** `url:` field on `bus { }` blocks and `SPANDA_BROKER_URL` env fallback for live transport and mTLS

### Fixed

- **Secure comm parser/runtime:** `secure_topic.publish` / `actuator.execute.safe` capability parsing, timed `fault … at T+10s` offsets, inbound trusted-source checks on receive/poll, TypeScript parser mirror for `secure_comm`, `trust_boundary`, `secrets`, bus blocks, and full `secure { }` fields
- **Example regression:** repaired 20 skipped `.sd` examples (regex, security, robotics, packages, hardware/modules); `scripts/check_all_examples.sh` resolves relative `SPANDA_BIN` from repo root for package checks — **162 pass, 2 expected-fail, 0 skips**
- **Lean-core transport shims:** ROS2/MQTT live bridge logic moved from `spanda-core/src/transport_live.rs` into `spanda-transport-ros2` and `spanda-transport-mqtt`; core retains a thin `RuntimeValue` shim with `lean_core_shims` guard tests
- **Lean-core transport adapters:** `TransportAdapter` implementations moved from `spanda-core/src/transport.rs` into `spanda-transport-{ros2,mqtt,dds,websocket}`; ROS2 rclrs consolidated in `spanda-transport-ros2`; Nav2/SLAM subprocess bridge moved to `spanda-connectivity::adapter_bridge`; unused TLS deps removed from `spanda-core` (TLS remains in `spanda-transport` and deploy crates)
- **Lean-core provider kernel:** `ProviderRegistry` and provider trait contracts moved to `spanda-runtime`; new `spanda-transport` crate for adapter traits and wire security; `spanda-interpreter` staging crate; fleet orchestration moved to `spanda-fleet`
- **Lean-core connectivity runtime split:** moved geofence math, connectivity/fault trigger mapping, GPS drift/spoof simulation, and link impairment checks from `spanda-core::connectivity_positioning` to `spanda-connectivity::runtime_sim`; core keeps compatibility wrappers for AST/runtime value conversions
- **Interpreter extraction staging:** expanded `spanda-runtime::RuntimeHost` with connectivity/geofence/GPS-fault hooks and routed `spanda-core::runtime` trigger/failover/geofence callsites through host methods to reduce direct core coupling
- **Interpreter host injection:** `Interpreter` now stores an injectable `RuntimeHost` (`InterpreterOptions::runtime_host`); remaining GPS reading and SIM identity paths route through host hooks; `spanda-interpreter` re-exports `RuntimeHost`
- **Interpreter module split:** connectivity trigger, geofence, and failover logic extracted from `runtime.rs` into `runtime_connectivity.rs` as a staging step toward `spanda-interpreter`
- **Interpreter submodule extraction:** navigation/SLAM (`runtime_navigation.rs`), robot methods (`runtime_robot.rs`), and trigger dispatch (`runtime_triggers.rs`) split out of `runtime.rs` with lean_core guard tests
- **Interpreter robotics/sensors/twin split:** AI/mission/fleet/safety (`runtime_robotics.rs`), sensor fusion (`runtime_sensors.rs`), and digital twin (`runtime_twin.rs`) extracted from `runtime.rs` (~580 lines); `runtime.rs` down to ~7670 lines
- **Interpreter builtins/audit/actuators split:** builtin dispatch (`runtime_builtins.rs`), audit/ledger (`runtime_audit.rs`), actuator motion (`runtime_actuators.rs`), and shared helpers (`runtime_helpers.rs`) extracted; `runtime.rs` down to ~6640 lines
- **Interpreter eval cluster split:** expression evaluation, member/call dispatch, regex methods, and binary operators moved to `runtime_eval.rs`; `runtime.rs` down to ~5750 lines
- **Interpreter spawn/async split:** module calls, future resolution, spawn targets, and task-handle queue processing moved to `runtime_spawn.rs`; `runtime.rs` down to ~5480 lines
- **Interpreter execution split:** statement execution (`runtime_execute.rs`), scheduling/contracts (`runtime_scheduler.rs`), robot setup (`runtime_setup.rs`), reliability (`runtime_reliability.rs`), declarations (`runtime_declarations.rs`), program/trigger glue (`runtime_program.rs`), and security helpers (`runtime_security.rs`) extracted; orchestrator down to ~1790 lines
- **Interpreter sources in `spanda-interpreter`:** all `runtime_*.rs` modules and `orchestrator.rs` now live under `crates/spanda-interpreter/src/runtime/`; `spanda-core/src/runtime.rs` is a thin `#[path]` include shim; smoke tests moved to `runtime_smoke.rs`

### Changed

- **Dependency security:** `cargo update` bumps `log` 0.4.33 and `quote` 1.0.46; npm upgrades `vitest` to 3.2.6 (critical Dependabot) with `vite` override to 6.4.3 (`npm audit` clean); removed unused TLS crates from `spanda-core` AES-256-GCM wire frames (`spanda/wire/v1:`), `TransportWireFrame` with `source_id`, TLS session negotiation from cert/key secrets, rustls PEM validation when cert files exist, broker URL TLS scheme auto-upgrade (`mqtts://`, `wss://`), session-key derivation from robot secrets for `EncryptedMessage`, and production wire crypto (replacing mock-session stubs)
- **Python native bridge runtime:** upgraded optional `pyo3` from 0.23 to 0.29 and migrated bridge GIL entrypoint to `Python::attach`; fixed embedded Python runner script syntax for native bridge tests
- **MQTT TLS dependency chain:** `spanda-transport-mqtt` now uses `rumqttc` 0.25.1 with `default-features = false` + `use-native-tls`, removing the old `rustls-webpki <0.103.13` path from the MQTT transport dependency graph
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

- AI providers use mock backends by default; set `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, or `SPANDA_ONNX_MODEL_PATH` for live calls
- ROS2 integration requires manual ROS Humble setup (experimental)
- LLVM/native compilation is experimental; interpreter is the primary runtime
- `spanda publish` mirrors to `registry/packages/`; hosted index lists 20 curated packages until `./scripts/build-registry.sh` is run
- VS Code extension VSIX builds in CI; Marketplace publish pending maintainer `VSCE_PAT`
- Multi-robot examples run in-process by default; distributed orchestration uses HTTP fleet agents

### Roadmap (post-alpha)

- VS Code Marketplace publish
- Production LLVM backend and optimized native binaries
- In-process Python/C++ FFI (PyO3, cxx) as primary path
- ROS2 production adapter with zero-config deployment
- Self-hosting compiler
- Digital twin cloud SaaS backend
- Distributed multi-robot orchestration at scale

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
