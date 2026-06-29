# Maritime — Solution Blueprint

**Status:** Experimental (scaffold) · **Timeline:** Later · **Path:** [examples/solutions/maritime/](../../examples/solutions/maritime/)

Official Solution Blueprint for autonomous vessels, port logistics, and shore coordination.

**Full roadmap entry:** [ROADMAP.md § Maritime](../../ROADMAP.md#maritime)

---

## Purpose

Operate autonomous or remotely supervised vessels with redundant navigation, corrosion-aware prognostics, GNSS-denied degradation, and shore takeover under high-latency links.

## Platform pillars used

| Pillar | Capabilities |
|--------|--------------|
| Device & Fleet | Fleet mesh, continuity/takeover, OTA, device pool failover |
| Verification | Readiness (pre-departure), assurance (machinery prognostics), recovery (safe harbor) |
| Security | Encrypted comms, tamper detection, RBAC for shore operators |
| Operations | Incident workflow, telemetry trends, Control Center fleet map |
| Packages & Ecosystem | `spanda-gps`, `spanda-radar`, `spanda-cellular`, `spanda-prognostics` |

## Reference architecture (planned)

```text
Vessel Stack
├── Navigation (GPS + radar fusion)
├── Propulsion / steering actuators
├── Machinery health (prognostics)
├── Shore link (SATCOM / cellular)
├── Readiness gates (pre-departure checklist)
├── Degraded mode on GNSS denial
└── Shore takeover via mission continuity
```

## Device tree (planned)

| Node | Role |
|------|------|
| `vessel` | Autonomous or teleoperated platform |
| `payload` | Survey, cargo, or inspection equipment |
| `shore_station` | C2, fleet coordinator |

## Packages & providers

| Package | Role |
|---------|------|
| `spanda-gps` | GNSS positioning |
| `spanda-radar` | Obstacle / collision avoidance |
| `spanda-cellular` | Near-shore link |
| `spanda-prognostics` | Hull/machinery RUL |
| `spanda-fusion` | Multi-sensor state estimation |
| `spanda-mission-continuity` | Shore takeover |

## Related docs

- [positioning.md](../positioning.md)
- [mission-continuity.md](../mission-continuity.md)
- [prognostics.md](../prognostics.md)
- [connectivity.md](../connectivity.md)
- [fleet-distributed.md](../fleet-distributed.md)

## Example projects

- [examples/solutions/maritime/](../../examples/solutions/maritime/) — `harbor_patrol.sd`, `convoy_escort.sd`, `docking_assist.sd` (CI: `scripts/solution_blueprints_smoke.sh`)

## Simulation & replay

- Sea state + GNSS denial scenarios
- Voyage black-box trace for incident reconstruction

---

**Related blueprints:** [Transportation](../ROADMAP.md#transportation) · [Defense](../ROADMAP.md#defense) · [Environmental Monitoring](./environmental-monitoring.md)
