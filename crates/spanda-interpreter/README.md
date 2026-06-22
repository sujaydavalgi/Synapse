# spanda-interpreter

**Tree-walking interpreter** and simulator — executes typed Spanda programs.

## Layout

Runtime lives under `src/runtime/` (~21 modules):

- `orchestrator.rs` — main execution loop
- `runtime_eval.rs`, `runtime_execute.rs` — expression and statement evaluation
- `runtime_scheduler.rs`, `runtime_triggers.rs` — tasks and triggers
- `runtime_robot.rs`, `runtime_sensors.rs`, `runtime_navigation.rs` — robot surface
- `simulator.rs` — physics-lite 2D backend
- Domain hooks delegate to workspace crates (`spanda-safety`, `spanda-comm`, `spanda-hal`, …)

## Public API

- `run_program` — execute a parsed `Program`
- `Interpreter`, `InterpreterOptions`, `RobotBackend`, `SimRobotBackend`
- Re-exported by `spanda-driver::run` and `spanda_core::run`

## Dependencies

Imports workspace crates directly (no `spanda-core`). `CoreRuntimeHost` in [`spanda-runtime-host`](../spanda-runtime-host/README.md) implements `RuntimeHost` for connectivity, fleet, and transport wiring.

## Related

- [spanda-driver](../spanda-driver/README.md) — compile + certify + run entry
- [docs/architecture.md](../../docs/architecture.md) — runtime diagrams
