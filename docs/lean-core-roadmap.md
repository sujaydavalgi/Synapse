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
