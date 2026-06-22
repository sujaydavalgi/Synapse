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

### Optional follow-up (caller migration)

| Step | Status |
|------|--------|
| Migrate `spanda-cli` / `spanda-node` imports to workspace crates directly | Planned |
| Remove deprecated `spanda_core::transport_*` shims after one release | Planned |
