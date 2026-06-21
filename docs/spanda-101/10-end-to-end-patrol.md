# Lesson 10 — End-to-end patrol

**Goal:** Combine AI safety, hardware deploy, tasks, and mission replay in one complete workflow.

**Examples:**

- [`examples/end_to_end/safe_patrol/`](../../examples/end_to_end/safe_patrol/)
- [`examples/end_to_end/replay_mission.sd`](../../examples/end_to_end/replay_mission.sd)
- [`examples/showcase/killer_demo.sd`](../../examples/showcase/killer_demo.sd)

---

## The patrol package

`examples/end_to_end/safe_patrol/` is a full project:

```
safe_patrol/
  spanda.toml
  src/main.sd
```

It demonstrates:

- Hardware profile + `deploy`
- AI planner with **`safety.validate()`** gate
- Periodic watchdog task
- Behavioral `verify { }` block

```bash
spanda check examples/end_to_end/safe_patrol/src/main.sd
spanda verify examples/end_to_end/safe_patrol/src/main.sd --target RoverV1
spanda run examples/end_to_end/safe_patrol/src/main.sd
```

---

## Record and replay a mission

Mission traces capture scheduler events and robot state for regression and incident review:

```bash
# Record
spanda sim examples/end_to_end/replay_mission.sd --record

# Inspect
spanda replay replay_mission.trace

# Re-run source and verify frame parity
spanda replay replay_mission.trace --deterministic

# Play back recorded state without re-executing logic
spanda replay replay_mission.trace --playback --from T+00:01
```

See [replay.md](../replay.md). Golden traces can be committed under `examples/` per `.gitignore` rules.

---

## Digital twins (preview)

Twin blocks mirror physical state for simulation and divergence detection:

```spanda
twin Shadow {
  replay true;
  mirror pose;
}
```

Used in `replay_mission.sd` and showcase demos. Full guide: `examples/showcase/digital_twin_demo.sd`.

---

## Capstone checklist

You have completed Spanda 101 when you can:

- [ ] Write a robot with sensors, safety, and behaviors
- [ ] Type-check and run in simulation
- [ ] Route AI output through `safety.validate()`
- [ ] Declare hardware and pass `spanda verify`
- [ ] Structure a package with `spanda.toml` and tests
- [ ] Record and replay a mission trace

---

## What to do next

| Path | Resource |
|------|----------|
| Flagship demo | [killer-demo.md](../killer-demo.md) |
| Real-time contracts | [realtime.md](../realtime.md) |
| Fleet simulation | `examples/communication/multi_robot_fleet.sd` |
| Language deep dive | [spanda-language.md](../spanda-language.md) |
| API lookup | [spanda-reference.md](../spanda-reference.md) |

---

## More end-to-end scenarios

Full catalog: [examples/end_to_end/README.md](../../examples/end_to_end/README.md)

| Scenario | Command |
|----------|---------|
| Warehouse delivery | `spanda verify examples/end_to_end/warehouse_delivery/src/main.sd --target RoverV1` |
| Pick-and-place cell | `spanda run examples/end_to_end/pick_and_place_cell/src/main.sd` |
| Fleet coordination | `spanda fleet run examples/end_to_end/fleet_coordination.sd` |
| Incident response | `spanda verify examples/end_to_end/incident_response.sd --simulate` |
| Real-time patrol | `spanda run examples/end_to_end/realtime_patrol.sd --trace-realtime` |

---

## Final exercise

Build a **`my_patrol`** package that:

1. Uses lidar + differential drive with `stop_if`
2. Includes a mock LLM planner validated through `safety.validate()`
3. Declares `deploy` to `RoverV1` and passes `spanda verify`
4. Records a sim run and replays it with `--deterministic`

Compare your solution to `examples/end_to_end/safe_patrol/`.

---

**Congratulations — you have finished Spanda 101.**

Return to the [curriculum index](./README.md) or explore [all examples](../../examples/basics/README.md).
