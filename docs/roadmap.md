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
- Phase 10: `twin` declarations with mirror/replay runtime
- Phase 11: ROS2-style `topic` / `service` / `action`
- **Hardware compatibility verification**: `hardware`, `deploy`, `requires_hardware`, `requires_network`, task `budget`, `mission`, `simulate_compatibility`, `spanda verify`, compatibility matrix, fault injection
- Tooling: LSP with check + verify diagnostics, `spanda fmt`, `verifyViaCli` TypeScript bridge
- Incremental runtime: enum values, struct literals, trait impl binding
- Phase 14: Example programs (`examples/types/`, `examples/hardware/`)

## In progress

- TypeScript parser mirror for hardware/requirements syntax (verification delegates to Rust CLI)
- Package manager (`spanda pkg`)
- Digital twin live telemetry sync (replay buffer exists)
- Advanced power models (dynamic load from behavior analysis)
- Self-hosted compiler subset

## Planned

- IDE: inline hardware profile picker, deploy target hints
- `spanda verify --all-targets` JSON matrix export for CI dashboards
- Hardware adapter trait code generation
- ROS2 adapter implementation (stub exists)
- Formal verification integration for safety constraints

## Self-hosting compiler path

1. **Bootstrap** — Rust implements full language (current)
2. **Spec stabilization** — grammar + API contract in `docs/`
3. **Spanda subset in Spanda** — rewrite lexer/parser for a minimal `.sd` subset
4. **Incremental migration** — type checker, then code generation, then full runtime
5. **Rust becomes optional** — Spanda self-hosts; Rust retained for WASM/embedded targets

Target: self-hosting type checker by milestone 3; full compiler by milestone 5.
