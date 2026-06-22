# spanda-runtime

**Runtime kernel** types and traits shared by the interpreter, transport routing, and provider bootstrap.

## Modules

| Area | Contents |
|------|----------|
| `scheduler` | Sim vs wall-clock tick helpers |
| `providers` | Provider trait contracts, `ProviderRegistry` |
| `robot_state` | `RobotState`, `PoseState`, `VelocityState` |
| `value` | `RuntimeValue`, motion commands |
| `environment` | Interpreter variable bindings |
| `error` | `RuntimeError` |
| `host` | `RuntimeHost` trait for domain hook extraction |
| `classification` | Module ownership audit table (lean-core) |
| `robotics` | `MissionRuntime`, `FleetRegistry`, zones |
| `replay`, `telemetry`, `triggers`, `twin`, `events`, `state_machine` | Runtime subsystems (Phase 8 extraction) |

## Bootstrap

Provider bootstrap implementation lives in [`spanda-providers`](../spanda-providers/README.md). `spanda-core::providers` re-exports providers + classification.

`CoreRuntimeHost` lives in [`spanda-runtime-host`](../spanda-runtime-host/README.md).

## Related

- [spanda-interpreter](../spanda-interpreter/README.md)
- [docs/lean-core.md](../../docs/lean-core.md)
