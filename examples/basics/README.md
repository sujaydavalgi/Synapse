# Basics → end-to-end examples

Progressive Spanda examples from a minimal robot to full deployment workflows.

**Guided tutorials:** [Tutorials index](../../docs/tutorials/README.md) · [Spanda for Dummies](../../docs/spanda-for-dummies/README.md) · [Spanda 101](../../docs/spanda-101/README.md)

## Tier 1 — Language basics (`basics/`)

| # | File | Topics |
|---|------|--------|
| 01 | [01_minimal_robot.sd](./01_minimal_robot.sd) | `robot`, `actuator`, `behavior` |
| 02 | [02_sensors_and_safety.sd](./02_sensors_and_safety.sd) | Sensors, `safety { }`, `stop_if` |
| 03 | [03_control_flow.sd](./03_control_flow.sd) | `if`/`else`, `match`, `loop every` |
| 04 | [04_result_and_option.sd](./04_result_and_option.sd) | `Result<T,E>`, `Option<T>`, `match Ok/Err/Some/None` |
| 05 | [05_traits_and_impl.sd](./05_traits_and_impl.sd) | `trait`, `agent`, `impl Trait for Agent` |
| 06 | [06_serialize_telemetry.sd](./06_serialize_telemetry.sd) | `serialize` / `deserialize` for IPC |
| 07 | [07_in_language_tests.sd](./07_in_language_tests.sd) | `test "..." { assert(...) }` — run with `spanda test` |
| 08 | [08_async_await.sd](./08_async_await.sd) | `async fn`, `await` in behaviors |
| 09 | [09_behavior_contracts.sd](./09_behavior_contracts.sd) | `requires` / `ensures` on behaviors |
| 10 | [10_state_machine.sd](./10_state_machine.sd) | `state_machine`, `enter StateName` |
| 11 | [11_observe_and_fusion.sd](./11_observe_and_fusion.sd) | `observe { }`, `fusion.read()` |

Try each file:

```bash
spanda check examples/basics/01_minimal_robot.sd
spanda run examples/basics/02_sensors_and_safety.sd
spanda test examples/basics/07_in_language_tests.sd
```

## Tier 2 — Integration slices (`integration/`)

| File | Topics |
|------|--------|
| [triggers_minimal.sd](../integration/triggers_minimal.sd) | Event triggers + periodic task |
| [concurrency_minimal.sd](../integration/concurrency_minimal.sd) | `channel`, `spawn`, `select` |
| [verify_walkthrough.sd](../integration/verify_walkthrough.sd) | `hardware`, `deploy`, `verify { }` |

## Tier 3 — End-to-end scenarios (`end_to_end/`)

See [end_to_end/README.md](../end_to_end/README.md) for the full catalog.

| Path | Topics |
|------|--------|
| [safe_patrol/](../end_to_end/safe_patrol/) | AI safety gate + deploy package |
| [warehouse_delivery/](../end_to_end/warehouse_delivery/) | Zones, services, state machine, verify |
| [pick_and_place_cell/](../end_to_end/pick_and_place_cell/) | Vision agent + arm/gripper cell |
| [fleet_coordination.sd](../end_to_end/fleet_coordination.sd) | Multi-robot fleet + network reqs |
| [incident_response.sd](../end_to_end/incident_response.sd) | Twin, recovery, degraded modes |
| [realtime_patrol.sd](../end_to_end/realtime_patrol.sd) | Deadlines, pipeline, watchdog |
| [replay_mission.sd](../end_to_end/replay_mission.sd) | Record → replay with twin |

## Existing topic directories

| Directory | Focus |
|-----------|--------|
| `showcase/` | Curated v0.1.0-alpha demos |
| `realtime/` | Deadlines, watchdogs, degraded modes |
| `regex/` | Pattern triggers and validation |
| `communication/` | Topics, services, fleet |
| `hardware/` | Deploy and compatibility |
| `modules/` | Cross-file imports |
| `types/` | Type-system snippets |
| `std/` | Standard library samples |
| `packages/` | Package manifest layouts |

Next step after basics: [Spanda for Dummies](../../docs/spanda-for-dummies/README.md), [Spanda 101](../../docs/spanda-101/README.md), [getting-started.md](../../docs/getting-started.md), and [showcase/killer_demo.sd](../showcase/killer_demo.sd).
