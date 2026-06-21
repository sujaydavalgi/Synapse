# End-to-end examples

Complete workflows that combine multiple Spanda capabilities — deploy, verify, AI safety, communication, real-time, twins, and packages.

**Tutorial:** [Spanda 101 Lesson 10](../../docs/spanda-101/10-end-to-end-patrol.md) · **Feature index:** [examples/features/README.md](../features/README.md)

```bash
spanda check examples/end_to_end/warehouse_delivery/src/main.sd
spanda fleet run examples/end_to_end/fleet_coordination.sd
```

---

## Scenarios

| Scenario | Path | What it demonstrates |
|----------|------|----------------------|
| Safe AI patrol (package) | [`safe_patrol/`](./safe_patrol/) | LLM planner, safety gate, hardware, deploy, verify |
| Mission record & replay | [`replay_mission.sd`](./replay_mission.sd) | Twin replay buffer, `--record`, `spanda replay` |
| Warehouse delivery (package) | [`warehouse_delivery/`](./warehouse_delivery/) | Zones, services, actions, state machine, deploy |
| Pick-and-place cell (package) | [`pick_and_place_cell/`](./pick_and_place_cell/) | Vision agent, arm/gripper, periodic behavior |
| Fleet coordination | [`fleet_coordination.sd`](./fleet_coordination.sd) | Multi-robot fleet, network reqs, multi-target deploy |
| Incident response | [`incident_response.sd`](./incident_response.sd) | Twin, recovery, degraded modes, fault triggers |
| Real-time patrol | [`realtime_patrol.sd`](./realtime_patrol.sd) | Deadlines, pipeline, watchdog, mission trace |
| Validated telemetry | [`validated_telemetry.sd`](./validated_telemetry.sd) | Regex `validate` rules + topic publish |
| Concurrent inspection | [`concurrent_inspection.sd`](./concurrent_inspection.sd) | Spawn, parallel, channels, inspection loop |

---

## Suggested order

1. `safe_patrol/` — baseline deploy + AI safety  
2. `warehouse_delivery/` — logistics comms + zones  
3. `realtime_patrol.sd` — reliability contracts  
4. `replay_mission.sd` — record and deterministic replay  
5. `fleet_coordination.sd` — `spanda fleet run`  
6. `incident_response.sd` — failure handling  
7. `pick_and_place_cell/` — manipulation cell  

---

## Commands cheat sheet

```bash
# Package projects
cd examples/end_to_end/safe_patrol && spanda check src/main.sd
spanda verify src/main.sd --target RoverV1

# Single-file workflows
spanda run examples/end_to_end/realtime_patrol.sd --trace-realtime --metrics-json
spanda sim examples/end_to_end/replay_mission.sd --record
spanda replay replay_mission.trace --deterministic

# Fleet
spanda fleet run examples/end_to_end/fleet_coordination.sd --trace-scheduler
```

---

## Adding an end-to-end example

1. Create under `examples/end_to_end/<name>/` (package) or `examples/end_to_end/<name>.sd`
2. Header comment with full command workflow
3. Add row to this README
4. Optional: `tests/golden/manifest.json` + [tutorials index](../../docs/tutorials/README.md)
