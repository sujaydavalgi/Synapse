# Lean-Core Roadmap

Phased plan to complete the package-first architecture after the initial scaffold.

## Phase 1 — Complete ✓

- Provider trait contracts in `spanda-core/src/providers/`
- `ProviderRegistry` and `bootstrap_default_providers()`
- 20 official package scaffolds under `packages/registry/`
- Compatibility shims documented on legacy core modules
- Architecture docs and migration guide
- TypeScript providers mirror and fleet CLI fix

## Phase 2 — Runtime wiring (in progress)

| Task | Status | Notes |
|------|--------|-------|
| Attach `ProviderRegistry` to `Interpreter` | Done | Auto-bootstrap when unset |
| Resolve official deps from `spanda.toml` | Done | `installed_official_packages()` |
| Load package providers from lockfile at `spanda run` | Planned | Needs resolver hook |
| Route transport calls through registry | Planned | Replace direct adapter construction |
| Register GPS/connectivity from installed packages | Planned | After crate extraction |

## Phase 3 — Crate extraction

Extract optional backends from `spanda-core` into workspace members:

```
crates/spanda-transport-mqtt/   (feature: live-mqtt)
crates/spanda-transport-ros2/   (optional rclrs)
crates/spanda-connectivity/     (GPS, WiFi, BLE, cellular sim)
crates/spanda-fleet/            (orchestrator, agents, mesh)
crates/spanda-ota/              (deploy service, agents)
```

Each crate implements core provider traits and registers via `ProviderRegistry`. Core re-exports shims behind `#[deprecated]` aliases for one release cycle.

## Phase 4 — Compiler split

Break circular `spanda-package` → `spanda-core` dependency:

```
spanda-lexer
spanda-ast
spanda-typecheck   ← spanda-package depends here
spanda-runtime     ← interpreter, scheduler, providers
spanda-core        ← thin facade re-exporting above
```

## Phase 5 — Live package backends

Replace scaffold `.sd` exports with full implementations:

1. `spanda-ros2` — wire rclrs native crate
2. `spanda-mqtt` — move `transport_mqtt.rs`
3. `spanda-gps` — move connectivity positioning drivers
4. `spanda-nav` / `spanda-slam` — promote adapter bridges
5. `spanda-fleet` / `spanda-ota` — move orchestration HTTP surface

## Phase 6 — TypeScript parity

- Mirror `ProviderRegistry` behavior in interpreter
- Share classification table via generated JSON or test golden file
- Route TS transport fallbacks through registry keys

## Known gaps

| Gap | Impact | Mitigation today |
|-----|--------|------------------|
| Domain code still in core | Larger binary | Shims + docs |
| Package scaffolds are stubs | No live vendor I/O | Core shims handle runtime |
| No dynamic `.so` loading | Packages are compile-time | Registry registration API ready |
| Clippy `-D warnings` failures | CI noise | Pre-existing; fix separately |
| `spanda-package` ↔ `spanda-core` cycle | Harder testing | Phase 4 split |

## Success criteria

- [ ] `cargo test --workspace` green
- [ ] `npm test` green (no TS fallback gaps)
- [ ] All 164 examples run without regression
- [ ] Zero protocol-specific code in core except traits + wire types
- [ ] Every official package has live backend or documented stub status

See also: [lean-core.md](./lean-core.md), [migration.md](./migration.md#lean-core-package-first-refactor)
