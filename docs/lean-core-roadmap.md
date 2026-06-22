# Lean-Core Roadmap

Phased plan to complete the package-first architecture.

## Phase 1 — Complete ✓

- Provider trait contracts in `spanda-runtime/src/providers/` (bootstrap shims in `spanda-core`)
- `ProviderRegistry` and `bootstrap_default_providers()`
- 20 official package scaffolds under `packages/registry/`
- Compatibility shims documented on legacy core modules
- Architecture docs and migration guide
- TypeScript providers mirror and fleet CLI fix

## Phase 2 — Complete ✓

Runtime wiring: `ProviderRegistry` on interpreter, lockfile/manifest official deps, comm-bus sync, package-scoped bootstrap.

## Phase 3 — Complete ✓

Transport, fleet, OTA, connectivity, and deploy-http crates extracted with core shims and registry-backed comm-bus routing.

## Phase 4 — Complete ✓ (kernel)

Compiler/runtime kernel extracted; interpreter remains the composition root in `spanda-core`:

| Crate | Status |
|-------|--------|
| `spanda-hardware` | Done — breaks `spanda-package` → `spanda-core` cycle |
| `spanda-ast` | Done |
| `spanda-lexer` | Done |
| `spanda-typecheck` | Done — `TypeCheckHost` + `CoreTypeCheckHost` |
| `spanda-transport` | Done — `TransportAdapter` trait, wire security/TLS, stub state |
| `spanda-runtime` | Done — scheduler, `ProviderRegistry`, provider traits, robotics state, `RuntimeValue`, `Environment`, `RuntimeError`, `RuntimeHost` |
| `spanda-interpreter` | **Done** — owns `src/runtime/` module tree (21 files, ~10.7k lines), `run_program`, and simulator; compiles standalone |
| `spanda-core` | Facade — lexer/parser/typecheck, domain shims, bootstrap, `run(source)` compile gate, and one-way re-export of `spanda_interpreter::runtime` |

The `Interpreter` is split across 21 modules under `crates/spanda-interpreter/src/runtime/` (orchestrator + eval/execute/scheduler/setup/… child files). `spanda-core` depends one-way on `spanda-interpreter` and re-exports `spanda_core::runtime` from `spanda_interpreter::runtime`. Auxiliary domain modules (`ai`, `safety`, `transport`, …) live in workspace crates that the interpreter imports directly.

## Phase 5 — Complete ✓ (bootstrap wiring)

All 20 official packages register capabilities; transport, positioning, navigation, SLAM, fleet, ledger, cloud, maintenance, vision, and simulation packages register `*Provider` stubs when installed. Spanda-language `.sd` exports remain scaffolds; live I/O is in workspace crates + core shims. See [official-packages.md](./official-packages.md).

## Phase 6 — Complete ✓

TypeScript parity: `bootstrapProvidersForPackages()`, registry-backed `RoutingCommBus`, interpreter `officialPackages` / `providerRegistry`, full classification table, `tests/providers-comm.test.ts`.

## Technical debt addressed

| Item | Status |
|------|--------|
| `cargo clippy --workspace -D warnings` | Green |
| `cargo test --workspace` | Green |
| `npm test` | Green |
| Example regression (`scripts/check_all_examples.sh`) | 162 pass, 2 expected-fail, 0 skips |
| `lean_core_cycle` cargo tree guard | Done |
| Clippy / visibility / API hygiene | Fixed across hardware, fleet, ota, core, cli |
| Transport `TransportAdapter` impls | Moved to `spanda-transport-{ros2,mqtt,dds,websocket}`; `lean_core_shims` guards `transport.rs` |
| Nav2/SLAM adapter bridge | Moved from `spanda-core` to `spanda-connectivity::adapter_bridge` |
| Connectivity runtime helpers | Geofence math + GPS fault simulation + link impairment moved to `spanda-connectivity::runtime_sim` (core keeps AST/runtime conversion shims) |
| ROS2 rclrs transport | Consolidated in `spanda-transport-ros2` (`rclrs.rs`); removed `transport_rclrs*.rs` from core |
| Fleet orchestration | Moved to `spanda-fleet::orchestrator`; core shim re-exports |
| `ProviderRegistry` / traits | Moved to `spanda-runtime`; bootstrap stays in core |

## Phase 7 — Complete ✓ (shim deprecation)

Protocol-specific I/O and adapter bodies live in workspace transport/fleet/connectivity crates. `spanda-core` retains only `RoutingCommBus`, wire encode/decode, bootstrap, package stubs, the `Interpreter` orchestration root, and thin `pub use` shims.

### Example repairs — Complete ✓

All 20 previously skipped examples now pass `spanda check`. The manifest retains only two `expect-fail` negative tests (`ai_safety_violation.sd`).

## Success criteria

- [x] `cargo test --workspace` green
- [x] `npm test` green
- [x] Example regression script in CI (162 + 2 negative tests)
- [x] `spanda-package` does not depend on `spanda-core`
- [x] Every official package has bootstrap registration or documented stub status
- [x] Zero protocol-specific code in core except traits + wire types (`spanda-transport-*` adapters, `spanda-connectivity` bridges, `spanda-fleet` orchestration; core routing + shims only)

See also: [lean-core.md](./lean-core.md), [migration.md](./migration.md#lean-core-package-first-refactor)

## Phase 8 — Complete ✓ (interpreter standalone compile)

Goal: `cargo build -p spanda-interpreter` compiles the runtime tree natively; `spanda-core` depends on `spanda-interpreter` (one-way) and drops the `#[path]` shim.

| Step | Status |
|------|--------|
| Route runtime imports through workspace crates (`spanda-ast`, `spanda-runtime`, …) instead of `crate::` | **Complete** |
| Native `cargo build -p spanda-interpreter` | **Complete** |
| Move `run()` / `run_program()` from `spanda-core` into `spanda-interpreter` | **Complete** (`run_program`, `run_tests_with_registry`; `run(source)` stays in core for compile) |
| `spanda-core` one-way dependency on `spanda-interpreter` | **Complete** |
| Add `spanda-interpreter` integration tests (not only re-exports) | **Complete** (`tests/native_smoke.rs`) |
| Drop `#[path]` shim in `spanda-core` | **Complete** — `runtime.rs` re-exports `spanda_interpreter::runtime` |
| Update `lean_core_shims` for native re-export | **Complete** (`runtime_shim_reexports_spanda_interpreter`) |

## Phase 9 — Complete ✓ (compile pipeline extraction)

Goal: extract parser and compile driver so `spanda-driver` owns the lexer → parser → type-check pipeline; `spanda-core` keeps `run(source)` (certify + FFI) as facade.

| Step | Status |
|------|--------|
| Extract `spanda-parser` (~8k LOC) | **Complete** |
| Move `CoreTypeCheckHost` to `spanda-runtime-host` | **Complete** |
| Create `spanda-driver` (`compile`, `check`, `CompileResult`) | **Complete** |
| Thin `spanda-core` shims for parser, compile, type-check host | **Complete** |
| TypeScript `compile.ts` documents Rust parity | **Complete** |
| Move `run(source)` into `spanda-driver` or `spanda-interpreter` | **Complete** — `spanda-driver::run` with certify + FFI defaults |
| Extract `spanda-certify` and `spanda-bridge` | **Complete** |

### Remaining `spanda-core` bodies (shrink candidates)

| Module | ~LOC | Notes |
|--------|------|-------|
| `sir.rs` | — | **Extracted** to `spanda-sir` |
| `hardware.rs` / `adapter_verify.rs` | — | **Extracted** to `spanda-hardware` (connectivity validators in `connectivity_validate`) |
| `pretty.rs` / `format.rs` | — | **Extracted** to `spanda-format` |
| `debug_session.rs` | — | **Extracted** to `spanda-driver::debug_session`; `spanda-debug` keeps controller + `stmt_line` |
| `language_reference.rs` / `docs.rs` | — | **Extracted** to `spanda-docs` |
| `lint.rs` | — | **Extracted** to `spanda-lint` |
| `codegen.rs` | — | **Extracted** to `spanda-codegen` |
| `modules.rs` | — | **Extracted** to `spanda-modules` |
| `security_validate.rs` | — | **Extracted** to `spanda-security::validate` |
| `swarm_coordinator.rs` | — | **Extracted** to `spanda-fleet::swarm_coordinator` |
| `certify_*` | — | **Extracted** to `spanda-certify` |
| `ffi.rs` / `bridge/` | — | **Extracted** to `spanda-bridge` + `spanda-ffi` |
| 40+ thin shims | ≤25 each | `pub use spanda_*` re-exports — keep until callers migrate |

## Phase 12 — Complete ✓ (tooling and verify extraction)

Goal: move formatter, linter, codegen metadata, module loader, hardware verify, docs generators, and swarm coordination out of `spanda-core` while preserving the public API via thin shims.

| Step | Status |
|------|--------|
| Extract `spanda-hardware` verify + adapter verify | **Complete** |
| Break `spanda-hardware` ↔ `spanda-connectivity-runtime` cycle | **Complete** — validators live in `spanda-hardware::connectivity_validate` |
| Extract `spanda-format`, `spanda-lint`, `spanda-codegen`, `spanda-modules` | **Complete** |
| Move `debug_session` to `spanda-driver` (break driver ↔ debug cycle) | **Complete** |
| Extract `spanda-docs` (program docs + language reference) | **Complete** |
| Move `swarm_coordinator` to `spanda-fleet` | **Complete** |
| `lean_core_shims` guards for Phase 12 extractions | **Complete** |

## Phase 13 — Complete ✓ (facade slim-down)

Goal: move remaining non-shim bodies into workspace crates; `spanda-driver` owns the high-level pipeline API; `spanda-core` is a thin facade.

| Step | Status |
|------|--------|
| Move `build_deploy_plan` AST extraction to `spanda-ota` (+ certify wrapper in `spanda-driver`) | **Complete** |
| Move reliability validators to `spanda-typecheck::reliability_validation` | **Complete** |
| Move type-check host wiring to `spanda-driver::type_check` | **Complete** |
| Move `verify_compatibility`, `lower_to_sir`, replay/debug helpers to `spanda-driver` | **Complete** |
| Move `validate_certification_standard` to `spanda-runtime-host` | **Complete** |
| Guard compatibility shims and Phase 13 extractions in `lean_core_shims` | **Complete** |

### `spanda-core` after Phase 13

| Area | Location |
|------|----------|
| Compile / verify / run / SIR / replay / debug | `spanda-driver` (re-exported by core) |
| OTA plan extraction | `spanda-ota::plan` + `spanda-driver::deploy_plan` |
| Type-check host wiring | `spanda-driver::type_check` |
| Reliability validators | `spanda-typecheck` |
| Provider bootstrap | `spanda-core::providers` (intentional composition root) |
| Transport / fleet / deploy agent shims | Thin `pub use` re-exports |

## Phase 14 — Complete ✓ (shim consolidation)

Goal: move the last small glue bodies out of `spanda-core` and collapse multi-file shim directories.

| Step | Status |
|------|--------|
| Move `transport_live` RuntimeValue helpers to `spanda-transport-routing` | **Complete** |
| Move `tokenize` SpandaError wrapper to `spanda-driver` | **Complete** |
| Move `new_with_core_bridges` alias to `spanda-bridge` | **Complete** |
| Collapse `providers/` subdirectory into single `providers.rs` facade | **Complete** |
| `lean_core_shims` guards for Phase 14 | **Complete** |

## Phase 15 — Complete ✓ (caller migration)

Goal: point first-party binaries and bindings at workspace crates directly instead of routing through `spanda-core` shims.

| Step | Status |
|------|--------|
| Migrate `spanda-cli` imports to workspace crates (`spanda-driver`, `spanda-ota`, `spanda-fleet`, tooling crates) | **Complete** |
| Migrate `spanda-node` imports to `spanda-driver`, `spanda-format`, `spanda-hardware`, `spanda-error` | **Complete** |
| Move MQTT/DDS/WebSocket `RuntimeValue` live bridges to `spanda-transport-routing::live_bridges` | **Complete** |
| `spanda-core` transport shims remain as thin `pub use` re-exports for external API stability | **Complete** |

## Phase 16 — Complete ✓ (remaining caller migration)

Goal: migrate the last first-party crates that still depended on `spanda-core` directly.

| Step | Status |
|------|--------|
| Migrate `spanda-llvm` to `spanda-sir`, `spanda-ast`, and `spanda-driver` | **Complete** |
| Migrate `spanda-wasm` to workspace driver/format/hardware/error crates | **Complete** |
| Migrate `spanda-dap` to `spanda-driver` and `spanda-debug` | **Complete** |
| Only `spanda-core` itself remains a direct consumer of the full facade graph | **Complete** |

### Optional follow-up

| Step | Status |
|------|--------|
| Remove deprecated `spanda_core::transport_*` shims after one release | **Complete** (see Phase 17) |

## Phase 17 — Complete ✓ (transport shim removal)

Goal: drop redundant `spanda_core::transport_{mqtt,dds,websocket,live}` modules now that callers use workspace crates.

| Step | Status |
|------|--------|
| Delete `transport_mqtt`, `transport_dds`, `transport_websocket`, `transport_live` from `spanda-core` | **Complete** |
| Mark removed modules `Deprecated` in classification tables (Rust + TS) | **Complete** |
| Update migration docs and shim guard tests | **Complete** |
| Keep `transport`, `transport_rclrs`, `transport_wire`, `transport_security` shims | **Complete** |
| Documentation ground-up refresh (`crates/README.md`, per-crate READMEs, lean-core.md) | **Complete** |

## Phase 18 — Complete ✓ (security & hardening)

Goal: address post–Phase 17 audit findings without changing the lean-core dependency graph.

| Step | Status |
|------|--------|
| Registry tarball SHA-256 + safe extraction (`spanda-package`) | **Complete** |
| Agent auth required on non-loopback bind | **Complete** |
| Bridge subprocess timeouts (`SPANDA_BRIDGE_TIMEOUT_SECS`) | **Complete** |
| `cargo audit` in CI | **Complete** |
| Pipeline benchmark (`spanda-driver`) | **Complete** |
| Slim CLI feature (`--no-default-features` + `--features slim`) | **Complete** |
| Panic audit (`runtime_twin`, agent helpers) | **Complete** |
| Phase 19 shim sunset plan (`transport*` remaining shims) | **Complete** |

See [phase-18-security-hardening.md](./phase-18-security-hardening.md).

## Phase 19 — Complete ✓ (transport shim removal)

Goal: remove the last `spanda_core::transport*` facade modules and slim `spanda-core` dependencies.

| Step | Status |
|------|--------|
| Delete `transport`, `transport_wire`, `transport_security`, `transport_rclrs` from `spanda-core` | **Complete** |
| Drop direct `spanda-transport-*` deps from `spanda-core` Cargo.toml | **Complete** |
| Migrate tests to workspace crate imports | **Complete** |
| Mark modules `Deprecated` in classification tables (Rust + TS) | **Complete** |
| Move `transport_rclrs` tests to `spanda-transport-ros2` | **Complete** |

## Phase 20 — Complete ✓ (test distribution + embedder features)

Goal: move domain integration tests into owning crates and optional `spanda-core` feature bundles for minimal embedders.

| Step | Status |
|------|--------|
| Move provider tests to `spanda-providers` | **Complete** |
| Move OTA/deploy tests to `spanda-ota` | **Complete** |
| Move fleet/swarm tests to `spanda-fleet` | **Complete** |
| Move certify tests to `spanda-certify` | **Complete** |
| Optional `ota` / `fleet` features on `spanda-core` (`--no-default-features` = minimal) | **Complete** |
| CI builds minimal `spanda-core` without fleet/OTA | **Complete** |

Embedders that only need compile/run/check can depend on `spanda-core` with `default-features = false`. Enable `features = ["ota"]`, `["fleet"]`, `["certify"]`, `["bridge"]`, or `["full"]` as needed.

## Phase 21 — Complete ✓ (hosted registry signing + embedder slimming)

Goal: sign curated hosted registry tarballs in CI and make certification / FFI shims optional on `spanda-core`.

| Step | Status |
|------|--------|
| `registry-index-maintain` refreshes checksums + Ed25519 `version_signatures` | **Complete** |
| CI verifies hosted registry signatures against `registry/TRUST_KEY` | **Complete** |
| Optional `certify` / `bridge` features on `spanda-core` | **Complete** |

## Phase 22 — Complete ✓ (Tier 3 experimental + P2/P3 closure)

Goal: promote deferred product-strategy items to experimental with minimal runtimes and golden paths; mark Phase 18 performance and observability tasks complete.

| Step | Status |
|------|--------|
| `world_model` runtime (`update`, `belief`, `export`) | **Complete** |
| `spanda-ledger` provider → `MockLedgerBackend` | **Complete** |
| `spanda-cloud` HTTP upload via `SPANDA_CLOUD_UPLOAD_URL` | **Complete** |
| LLVM golden path script (`scripts/llvm_golden_path.sh`) | **Complete** |
| Self-host bootstrap example (`examples/self_host/`) | **Complete** |
| [tier-3-experimental.md](./tier-3-experimental.md) index | **Complete** |
| Phase 18 P2 performance + P3 observability marked complete | **Complete** |

See [tier-3-experimental.md](./tier-3-experimental.md).

## Phase 23 — Complete ✓ (platform integration + experimental Tier 3 hardening)

Goal: complete package→provider→runtime wiring, transitive dependencies, replay observability, and CI golden paths for experimental Tier 3 items.

| Step | Status |
|------|--------|
| Runtime provider dispatch (`package_dispatch.rs`) for GPS, connectivity, MQTT, nav, fleet, SLAM, vision, simulation | **Complete** |
| Transitive dependency resolution in `spanda-package` resolver | **Complete** |
| Provider capability alignment in validation + security | **Complete** |
| `provider_call` frames in mission trace (`--record`) | **Complete** |
| `spanda update`, `--trace-providers`, flagship `autonomous_rover` demo | **Complete** |
| Project-aware `check`/`build`/`run`/`verify` module registry | **Complete** |
| TS mirror `package_dispatch.ts` | **Complete** |
| Fleet multi-host CI + golden path docs | **Complete** |
| MQTT live Mosquitto CI | **Complete** |
| Twin cloud export CI | **Complete** |
| `cpp-native` golden path CI | **Complete** |
| LLVM golden path in CI | **Complete** |
| Ledger community package scaffold | **Complete** |
| World model parser + fusion hook | **Complete** |
| Self-host lexer milestone | **Complete** |

See [tier-3-priority-plan.md](./tier-3-priority-plan.md) for full P0–P4 ordering and release windows.

## Phase 24 — Complete ✓ (v1.0 optional Tier 3 showcases)

Goal: ship evaluator-ready showcases and golden paths for P3 promotion criteria — world-model decisions, multi-agent fleet trials, twin incident workflow, and live MQTT in reference architectures.

| Step | Status |
|------|--------|
| `world_model_patrol.sd` showcase + `world_model_golden_path.sh` CI | **Complete** |
| Three-agent `fleet_field_trial.sd` in robotics golden path | **Complete** |
| [tier-3-golden-paths.md](./tier-3-golden-paths.md) index | **Complete** |
| Twin replay in incident workflow docs | **Complete** |
| MQTT + Nav2 reference architecture doc | **Complete** |
| LLVM Jetson/Pi benchmark slice | **Complete** |
| Rust + TS world_model parity tests | **Complete** |

See [tier-3-golden-paths.md](./tier-3-golden-paths.md) and [tier-3-priority-plan.md](./tier-3-priority-plan.md) § P3.

## Phase 25 — In progress (v0.5 beta P0 golden paths)

Goal: close v0.5 beta blockers with CI-backed golden paths for killer demo, live AI, ROS2 rclpy bridge, and hosted registry install.

| Step | Status |
|------|--------|
| `killer_demo_golden_path.sh` + CI job | **Complete** |
| `live_ai_golden_path.sh` + CI job | **Complete** |
| `ros2_cmd_vel_ping.sd` + `ros2_golden_path.sh` CI | **Complete** |
| `registry_golden_path.sh` (file:// + signatures) CI | **Complete** |
| VS Code extension marketplace publish | **Partial** — release workflow publishes when `VSCE_PAT` secret is set |

Phase 25 P0 golden paths are **complete** except marketplace publish (requires maintainer `VSCE_PAT`).
| P0 acceptance tracking in [tier-3-priority-plan.md](./tier-3-priority-plan.md) | **Complete** |

See [killer-demo.md](./killer-demo.md), [live-ai-provider.md](./live-ai-provider.md), [ros2-golden-path.md](./ros2-golden-path.md), [registry.md](./registry.md).

## Phase 26 — Complete ✓ (P1 adoption enablers)

Goal: ship CI-backed golden paths and polish for v0.5 beta adoption — verify in CI, PyO3 FFI, LSP deploy-target completion.

| Step | Status |
|------|--------|
| `ci_verify_golden_path.sh` + CI job | **Complete** |
| `python_native_golden_path.sh` + `python-native` CLI feature | **Complete** |
| Hardware profile autocomplete in LSP/VS Code | **Complete** (existing) |
| [ci-verify.md](./ci-verify.md) + [adoption-path.md](./adoption-path.md) cross-links | **Complete** |
| Showcase trim to 3 pillars | **Complete** ([showcase/README.md](../examples/showcase/README.md)) |

## Phase 27 — Complete ✓ (verification, traceability, health, DX)

Goal: ship test framework hardening, hardware/robot capability exposure, traceability matrices, kill switch, health checks, IoT contracts, and GitHub Pages docs without bloating core.

| Step | Status |
|------|--------|
| `spanda-capability` crate (registry, traceability, minimum-hardware, health) | **Complete** |
| Language syntax: `kill_switch`, `health_check`, `health_policy`, `requires_capability`, `uses hardware`, `exposes capabilities` | **Complete** |
| CLI: `trace`, `health`, `hardware capabilities`, `robot capabilities`, `safety check --capabilities` | **Complete** |
| Hardened `spanda test` (file paths, `--json`, `--filter`, `--compile-fail`) | **Complete** |
| Sim: `--trigger-kill-switch`, `--inject-health-faults` | **Complete** |
| IoT provider traits + `spanda-iot-core` stub | **Complete** |
| mdBook `docs-site/` + `.github/workflows/pages.yml` | **Complete** |
| Example: `examples/hardware/capability_verification.sd` | **Complete** |
| Docs: kill-switch, health, capabilities, traceability, IoT, agentic, debugger | **Complete** |

See [capability-traceability.md](./capability-traceability.md), [health-checks.md](./health-checks.md), [kill-switch.md](./kill-switch.md).

## Phase 28 — Complete ✓ (compile-fail tests, return types, TS parity, IoT scaffolds)

Goal: close verification/DX gaps from Phase 27 audit — in-language compile-fail tests, module return-type enforcement, TypeScript parser mirror, and IoT protocol package stubs.

| Step | Status |
|------|--------|
| `expect_compile_error { }` in test blocks (parser, typecheck skip, test runner validation) | **Complete** |
| Module function return type validation in typechecker | **Complete** |
| TypeScript parser mirror for Phase 27 syntax + `expect_compile_error` | **Complete** |
| IoT protocol package stubs (`spanda-opcua`, `spanda-modbus`, `spanda-zigbee`, `spanda-lora`, `spanda-matter`, `spanda-canbus`) | **Complete** |
| Integration tests in `crates/spanda-core/tests/p1_features.rs` | **Complete** |
| `tests/capability-parser.test.ts` | **Complete** |

## Phase 29 — Complete ✓ (LSP verification diagnostics, runtime health, kill-switch tests)

Goal: wire capability/traceability/health analysis into the IDE, connect runtime health to `HardwareMonitor`, and add kill-switch integration coverage.

| Step | Status |
|------|--------|
| `collect_verification_diagnostics` with source spans (`spanda-capability`) | **Complete** |
| CLI `spanda check --verification-json` | **Complete** |
| LSP diagnostics for capability/traceability/health/kill-switch | **Complete** |
| Runtime health evaluation via `evaluate_runtime_health` + `HardwareMonitor` | **Complete** |
| Integration tests (`phase29_verification.rs`, diagnostics unit tests) | **Complete** |

## Phase 30 — Complete ✓ (continuous health polling, verification quick-fixes, debug events)

Goal: poll runtime health during trigger maintenance, surface suggested fixes in verification diagnostics and LSP code actions, and emit debugger pause events for kill switch and critical health.

| Step | Status |
|------|--------|
| `suggested_fix` on `VerificationDiagnostic` (kill-switch, health, capability, minimum-hardware) | **Complete** |
| LSP quick-fix code actions from `--verification-json` cache | **Complete** |
| Continuous health polling in `run_trigger_maintenance` + startup inject path | **Complete** |
| `record_debug_event` for kill switch and critical health (`health_critical`, `kill_switch_activated`) | **Complete** |
| Integration test: continuous health poll during `every` trigger loop | **Complete** |

## Phase 31 — Complete ✓ (gap closure: health policy runtime, typed I/O, IoT dispatch, agent audit, DAP output)

Goal: close partial gaps from the 14-item verification/DX audit — runtime health_policy enforcement, behavior/agent I/O typing, IoT package dispatch, agent capability audit hooks, and DAP output for health events.

| Step | Status |
|------|--------|
| `health_policy` reactions store `Vec<Stmt>` and execute at runtime on status transitions | **Complete** |
| Behavior `-> Type` return type validation in typechecker | **Complete** |
| Agent plan `SafeAction` return enforcement when `propose_motion` / `execute` granted | **Complete** |
| IoT package bootstrap + `package_dispatch` for `spanda-iot-core`, modbus, opcua | **Complete** |
| Agent capability grant/deny audit logging | **Complete** |
| DAP `output` events for health/kill-switch debug pauses | **Complete** |
| Integration tests (`phase31_gaps.rs`, IoT dispatch test) | **Complete** |

## Phase 32 — Complete ✓ (IoT hub, task I/O, agent can[] enforcement, VSIX verify)

Goal: close remaining partial gaps — in-memory IoT state, task return types, default-deny for empty `can[]`, VS Code VSIX build verification.

| Step | Status |
|------|--------|
| In-memory `IotHub` with device/telemetry/shadow/modbus state | **Complete** |
| IoT dispatch wired to hub (`register`, `publish`, `read_register`, …) | **Complete** |
| Task `-> Type` return validation | **Complete** |
| Agent `can[]` default-deny for `execute` / `propose_motion` | **Complete** |
| `scripts/verify_vscode_vsix.sh` build smoke test | **Complete** |
| Integration tests (`phase32_gaps.rs`, IoT hub tests) | **Complete** |

## Phase 33 — Complete ✓ (trigger I/O, live IoT, live AI)

Goal: close remaining partial gaps — trigger handler return types, live Modbus/OPC-UA hardware paths, and live OpenAI provider for `ai_model`.

| Step | Status |
|------|--------|
| Trigger `-> Type` return validation (`every`, `when`, `while`, `on`) | **Complete** |
| Live Modbus TCP (`SPANDA_LIVE_MODBUS=1`, `--features live-iot`) + Python bridge fallback | **Complete** |
| Live OPC-UA bridge reads (`SPANDA_LIVE_OPCUA=1`, Python asyncua path) | **Complete** |
| Live OpenAI provider for `provider: "openai"` when `OPENAI_API_KEY` set | **Complete** |
| Integration tests (`phase33_gaps.rs`) + `scripts/live_iot_golden_path.sh` | **Complete** |

## Phase 34 — Complete ✓ (I/O verification, kill switch signing, debugger CI, IoT protocols, Anthropic, fleet/swarm health)

Goal: fully close the six remaining partial areas from the 14-item verification/DX audit.

| Step | Status |
|------|--------|
| Event handler `-> Type` return validation (Rust + TypeScript mirror) | **Complete** |
| `on kill_switch Name { }` trigger syntax + typecheck + runtime dispatch | **Complete** |
| `remote_signed` kill switch runtime signature enforcement (`kill_switch_signature` in `RunOptions`) | **Complete** |
| VS Code extension CI workflow (`.github/workflows/vscode-extension-ci.yml`) | **Complete** |
| IoT protocol dispatch for zigbee/lora/matter/canbus + package `.sd` stubs | **Complete** |
| Live Anthropic provider (`ANTHROPIC_API_KEY`, `spanda-anthropic` package) | **Complete** |
| Fleet-target health refinement via `apply_fleet_health_checks` + swarm coordination hooks | **Complete** |
| Integration tests (`phase34_gaps.rs`) | **Complete** |

## Phase 35 — Complete ✓ (beta hardening: TS build, IoT live bridges, fleet requirements, ONNX, registry mirror)

Goal: close remaining gaps except VS Code Marketplace publish — TS build parity, live IoT protocol bridges, fleet health requirements, ONNX provider, registry mirror publish, kill-switch verify severity, debugger `every` entry.

| Step | Status |
|------|--------|
| TypeScript `tsc` build green (`compile.ts` stub, `void` types, foundations import) | **Complete** |
| Live IoT env gates + Python bridge for zigbee/lora/matter/canbus | **Complete** |
| Extended `scripts/live_iot_golden_path.sh` | **Complete** |
| Fleet `require` clause parsing + runtime evaluation (`at_least N%`) | **Complete** |
| `remote_signed` kill switch verification diagnostic upgraded to **error** | **Complete** |
| Debugger entry from `every` trigger bodies | **Complete** |
| ONNX provider (`SPANDA_ONNX_MODEL_PATH`, `spanda-onnx` package) | **Complete** |
| `spanda publish` mirrors bundle to `registry/packages/` | **Complete** |
| Integration tests (`phase35_gaps.rs`) | **Complete** |

