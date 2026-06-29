# Official Solution Blueprints

Spanda ships **Official Solution Blueprints** — reference architectures built entirely on existing platform capabilities. Each blueprint demonstrates how to compose **Platform Pillars**, packages, and providers for a specific industry **without** bloating the core.

**Full index with cross-references:** [ROADMAP.md § Official Solution Blueprints](../ROADMAP.md#official-solution-blueprints)

---

## Blueprint catalog

| Blueprint | Status | Timeline | Path |
|-----------|--------|----------|------|
| **Warehouse Automation** | Experimental | Now | [warehouse.md](./warehouse.md) · [examples/end_to_end/warehouse_delivery/](../../examples/end_to_end/warehouse_delivery/) |
| **Search & Rescue** | Experimental | Next | [examples/solutions/spatial-computing/search-and-rescue-ar/](../../examples/solutions/spatial-computing/search-and-rescue-ar/) |
| **Connected Healthcare** | Experimental | Next | [examples/solutions/spatial-computing/wearable-health/](../../examples/solutions/spatial-computing/wearable-health/) |
| **ADAS & Autonomous Driving** | Experimental | Now | [examples/solutions/adas/](../../examples/solutions/adas/) · [adas.md](./adas.md) |
| **Smart Factory** | Experimental | Now | [smart-factory.md](./smart-factory.md) · [examples/end_to_end/pick_and_place_cell/](../../examples/end_to_end/pick_and_place_cell/) |
| **Agriculture** | Experimental (scaffold) | Later | [agriculture.md](./agriculture.md) · [examples/solutions/agriculture/](../../examples/solutions/agriculture/) |
| **Critical Infrastructure** | Experimental | Next | [examples/showcase/compliance/](../../examples/showcase/compliance/) |
| **Environmental Monitoring** | Experimental (scaffold) | Later | [environmental-monitoring.md](./environmental-monitoring.md) · [examples/solutions/environmental-monitoring/](../../examples/solutions/environmental-monitoring/) |
| **Maritime** | Experimental (scaffold) | Later | [maritime.md](./maritime.md) · [examples/solutions/maritime/](../../examples/solutions/maritime/) |
| **Transportation** | Experimental | Now | [examples/solutions/adas/applications/](../../examples/solutions/adas/applications/) |
| **Space** | Research | Long Term | — |
| **Defense** | Experimental | Next | [defense.md](./defense.md) · [examples/showcase/secure_boot/](../../examples/showcase/secure_boot/) |
| **Research & Education** | Stable | Now | [examples/showcase/autonomous_rover/](../../examples/showcase/autonomous_rover/) |
| **Spatial Computing & HRI** | Experimental | Next | [examples/solutions/spatial-computing/](../../examples/solutions/spatial-computing/) · [spatial-computing.md](./spatial-computing.md) |
| **Smart Spaces & Ambient Intelligence** | Experimental (scaffold) | Next | [smart-spaces.md](./smart-spaces.md) · [examples/solutions/smart-spaces/](../../examples/solutions/smart-spaces/) |

**Also:** Compliance profiles showcase — [examples/showcase/compliance/](../../examples/showcase/compliance/)

**Scaffold CI:** `./scripts/solution_blueprints_smoke.sh` (agriculture, environmental monitoring, maritime) · `./scripts/smart_spaces_smoke.sh` (smart spaces)

---

## Blueprint template

Each blueprint documents (see [ROADMAP.md](../ROADMAP.md) for full entries):

- Purpose
- Reference Architecture
- Device Tree
- Packages & Providers
- Mission Examples
- Health Policies · Readiness · Assurance · Recovery
- Control Center integration
- Example Projects · Documentation
- Simulation · Replay
- **Platform Pillars used** (cross-reference — no duplicate implementations)

---

## ADAS & Autonomous Driving

Safety-first intelligent vehicle workflows — lane keeping, adaptive cruise, emergency braking, sensor recovery, driver takeover, and highway pilot.

- **Architecture:** [adas.md](./adas.md)
- **Device tree:** [automotive-device-tree.md](../automotive-device-tree.md)
- **Readiness:** [adas-readiness.md](../adas-readiness.md)
- **Assurance:** [adas-assurance.md](../adas-assurance.md)
- **Security:** [adas-security.md](../adas-security.md)
- **Replay:** [adas-replay.md](../adas-replay.md)
- **Pillars used:** Verification · Device & Fleet · Security · Operations · Packages

```bash
spanda demo adas
./scripts/adas_smoke.sh
```

See also: [compliance-profiles.md](../compliance-profiles.md) (ISO 26262) · [mission-continuity.md](../mission-continuity.md) · [control-center.md](../control-center.md)

---

## Spatial Computing & Human-Robot Collaboration

Human–robot collaboration, wearables, AR/VR/XR, and collaborative autonomy — composes Device Registry, Capability Framework, Readiness, Continuity, Trust, and Control Center without core language extensions.

- **Architecture:** [spatial-computing.md](./spatial-computing.md)
- **Roadmap:** [human-interaction-spatial-computing-roadmap.md](../human-interaction-spatial-computing-roadmap.md)
- **Human entity:** [human-interaction.md](../human-interaction.md)
- **Operator capabilities:** [operator-capabilities.md](../operator-capabilities.md)
- **Human readiness:** [human-readiness.md](../human-readiness.md)
- **Packages:** [hri-packages.md](../hri-packages.md)
- **Pillars used:** Device & Fleet · Operations · Verification · Developer · Packages

```bash
cd examples/solutions/spatial-computing && spanda check warehouse-ar/pick_mission.sd
```

---

## Smart Spaces & Ambient Intelligence

Safety-first orchestration for intelligent environments — IoT, robots, wearables, connected healthcare, and energy — without competing with Home Assistant or home automation hubs.

- **Architecture:** [smart-spaces.md](./smart-spaces.md)
- **Building automation:** [building-automation.md](../building-automation.md)
- **Ambient intelligence:** [ambient-intelligence.md](../ambient-intelligence.md)
- **Energy:** [energy-management.md](../energy-management.md)
- **Readiness:** [smart-space-readiness.md](../smart-space-readiness.md)
- **Security:** [smart-space-security.md](../smart-space-security.md)
- **Packages:** [smart-space-packages.md](../smart-space-packages.md)
- **Pillars used:** Verification · Device & Fleet · Operations · Security · Packages

```bash
cd examples/solutions/smart-spaces && spanda check smart-home/night_mode.sd
./scripts/smart_spaces_smoke.sh
```

---

## Warehouse Automation

Autonomous logistics — AMRs, pick-and-place, fleet coordination.

- **Examples:** [warehouse_delivery/](../../examples/end_to_end/warehouse_delivery/) · [warehouse_robot.sd](../../examples/warehouse_robot.sd)
- **Continuity walkthrough:** [tutorials/continuity-walkthrough.md](../tutorials/continuity-walkthrough.md)
- **Pillars used:** Device & Fleet · Verification · Operations · Developer

---

## Website

[solutions.html](../../website/solutions.html) · [roadmap.html](../../website/roadmap.html)
