# Spanda Roadmap

## Completed (Rust core)

- Phase 1: `module`, `struct`, `enum`, `trait`, `match`
- Phase 2: Extended `agent` with `skill`, existing robot/sensor/actuator model
- Phase 3: `state_machine` parse + transition validation
- Phase 4: `task every Nms` scheduler in runtime
- Phase 5: `can [ ... ]` capability checking
- Phase 6: AI safety (`ActionProposal` / `SafeAction`) — already enforced
- Phase 7: Physical units — existing unit algebra
- Phase 8: `requires` / `ensures` / `invariant` on behaviors and tasks
- Phase 9: `event` / `on` handlers
- Phase 10: `twin` declarations with mirror/replay metadata
- Phase 11: ROS2-style `topic` / `service` / `action`
- Incremental runtime: `enter`, twin replay API, enum values, struct literals, trait impl binding
- Tooling: LSP package in CI, `spanda fmt` basic formatter
- Phase 14: New example programs

## In progress

- TypeScript mirror for new AST nodes (lexer/parser/types/runtime)
- Language Server Protocol (LSP)
- Formatter and package manager (`spanda pkg`)
- Runtime execution of state-machine transitions tied to behaviors
- Contract enforcement at runtime (not just parse/typecheck)
- Digital twin telemetry sync and replay buffers

## Self-hosting compiler path

1. **Bootstrap** — Rust implements full language (current)
2. **Spec stabilization** — grammar + API contract frozen in `docs/`
3. **Spanda subset in Spanda** — rewrite lexer/parser for a minimal `.sd` subset
4. **Incremental migration** — type checker, then code generation, then full runtime
5. **Rust becomes optional** — Spanda self-hosts; Rust retained for WASM/embedded targets

Target: self-hosting type checker by milestone 3; full compiler by milestone 5.
