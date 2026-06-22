# Spanda Roadmap

## Completed (Rust core)

- Phase 1: `module`, `struct`, `enum`, `trait`, `match`
- Phase 2: Extended `agent` with `skill`, existing robot/sensor/actuator model
- Phase 3: `state_machine` parse + transition validation + `enter`
- Phase 4: Autonomous primitives — `goal`, `memory` (`remember`/`recall`), behavioral `verify { }`, multi-task scheduler, `observe`/`fusion`
- Phase 5: `can [ ... ]` capability checking (compile + runtime)
- Phase 6: AI safety (`ActionProposal` / `SafeAction`) — enforced
- Phase 7: Physical units — unit algebra
- Phase 8: `requires` / `ensures` / `invariant` on behaviors and tasks (runtime enforcement)
- Phase 9: `event` / `on` handlers
- **Unified triggers**: `every`, `when`, `while`, message/safety/state/AI/verification/twin handlers, `TriggerRegistry`, storm limits, metrics
- **Cooperative concurrency**: `spawn`, `join`, `parallel`, channels, `select`, per-task `budget { }`, runtime telemetry
- **Fleet CLI**: `spanda fleet run` for in-process multi-robot simulation
- **Real-time contracts**: deadline/jitter/priority tasks, latency pipelines, wall-clock scheduling (`--wall-clock`)
- **Reliability runtime**: watchdogs, operating modes, recovery handlers, retry/fallback, topic QoS deadlines
- **Mission trace replay**: `--record`, `spanda replay --deterministic` / `--playback`, v2 state snapshots
- **First-class regex**: literals, `Regex` type, trigger/subscribe filters, validation rules
- Phase 10: `twin` declarations with mirror/replay runtime
- Phase 11: ROS2-style `topic` / `service` / `action`
- **Hardware compatibility verification**: `hardware`, `deploy`, `requires_hardware`, `requires_network`, task `budget`, `mission`, `simulate_compatibility`, `spanda verify`, compatibility matrix, fault injection
- Tooling: LSP with check + verify diagnostics, `spanda fmt`, full TypeScript CLI parity
- FFI bridge import registry (`python.*`, `cpp.*`) — type-check only
- N-API / WASM `verify` bindings
- Incremental runtime: enum values, struct literals, trait impl binding
- Phase 14: Example programs (`examples/types/`, `examples/hardware/`)
- TypeScript parser mirror for hardware/deploy/requirements syntax
- **Verification & DX (Phase 27):** `spanda-capability` crate; traceability matrices; hardware/robot capability exposure; minimum-hardware safety; health checks; kill switch syntax; hardened `spanda test`; IoT provider contracts; mdBook docs site
- **Verification & DX (Phase 28):** `expect_compile_error` test blocks; module return type validation; TypeScript parser/typechecker parity for Phase 27 syntax; IoT protocol package stubs
- **Verification & DX (Phase 29):** LSP verification diagnostics; runtime health wired to `HardwareMonitor`; `spanda check --verification-json`; kill-switch integration tests
- **Verification & DX (Phase 30):** verification quick-fix hints; LSP code actions; continuous health polling in trigger loop; debug pause events for kill switch and critical health
- **Verification & DX (Phase 31):** runtime health_policy reactions; behavior return types; agent SafeAction plan returns; IoT package dispatch; agent capability audit hooks
- **Verification & DX (Phase 32):** in-memory IoT hub; task return types; agent can[] enforcement; VSIX verify script

## In progress

- Package manager publish to live registry
- Digital twin live telemetry sync (replay buffer exists)
- Advanced power models (dynamic load from behavior analysis)
- Self-hosted compiler subset
- Real Python/C++/ROS2 bridge linking (see `docs/ffi-and-ecosystem.md`)
- Inline documentation coverage across remaining community packages

## Planned

- IDE: inline hardware profile picker, deploy target hints
- Hardware adapter trait code generation
- ROS2 adapter live node integration (stub exists)
- Formal verification integration for safety constraints
- Spanda IR + LLVM native backend (see `docs/compiler-backend-roadmap.md`)

## Self-hosting compiler path

1. **Bootstrap** — Rust implements full language (current)
2. **Spec stabilization** — grammar + API contract in `docs/`
3. **Spanda subset in Spanda** — rewrite lexer/parser for a minimal `.sd` subset
4. **Incremental migration** — type checker, then code generation, then full runtime
5. **Rust becomes optional** — Spanda self-hosts; Rust retained for WASM/embedded targets

Target: self-hosting type checker by milestone 3; full compiler by milestone 5.
