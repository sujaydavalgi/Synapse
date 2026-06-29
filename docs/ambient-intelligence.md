# Ambient Intelligence

Context-aware, safety-verified orchestration for intelligent environments — part of the [Smart Spaces & Ambient Intelligence](./solutions/smart-spaces.md) blueprint.

**Status:** Experimental (scaffold)

Ambient intelligence here means: **sense context → verify readiness → orchestrate missions → produce evidence** — not opaque black-box automation.

---

## Definition

| Term | Spanda interpretation |
|------|----------------------|
| Ambient | Environment-wide signals (occupancy, AQ, energy, presence) |
| Intelligence | Verified missions + optional AI agents within capability bounds |
| Context | Entity graph: who is where, what devices are healthy, what mode applies |

Spanda adds **verification** and **trust** layers that typical ambient-AI stacks omit.

---

## Context stack

```text
Sensors & wearables
  → Entity graph (building, zone, occupant, device)
  → Readiness engine (can we act safely?)
  → Mission planner (.sd programs + optional AI agents)
  → Providers (Matter, BACnet, MQTT, robots)
  → Assurance evidence + Control Center
```

---

## Context signals

| Signal | Sources | Missions influenced |
|--------|---------|---------------------|
| Occupancy | PIR, mmWave, BLE, cameras | HVAC setback, lighting, cleaning |
| Presence | Wearables, mobile app | Health alerts, personalized climate |
| Air quality | CO₂, VOC, particulate | Ventilation boost, notify operator |
| Energy price | Utility API, `spanda-energy` | Demand response, EV defer |
| Safety | Smoke, CO, leak | Emergency override all comfort missions |
| Trust | Lock tamper, camera attestation | Block access missions |

---

## Adaptive workflows

### Occupancy-driven climate

Office zone unoccupied for 30 minutes → readiness checks BACnet path → setback HVAC → record assurance snapshot.

Example: `examples/solutions/smart-spaces/smart-office/occupancy_climate.sd`

### Presence-aware lighting

Occupant enters room → verify gateway + dimmer health → raise lights → log access in assurance bundle.

### Health-adjacent ambient (opt-in)

Wearable fall signal → verify responder availability → emergency mission → Connected Healthcare bridge.

Example: `examples/solutions/smart-spaces/hospital-at-home/patient_monitoring.sd`

---

## AI agents

AI agents (via `spanda-openai`, `spanda-onnx`, or custom packages) may **propose** mission parameter changes. They do **not** bypass:

- `requires capabilities` verification
- Readiness score thresholds
- Human approval for lockdown, medical, or life-safety overrides
- `kill_switch` and `health_policy` enforcement

---

## Privacy

- Occupant and health dimensions follow [human-readiness.md](./human-readiness.md) opt-in policy.
- Camera and microphone streams require explicit capability grants in device tree.
- Assurance bundles redact PII per deployment `spanda.security.toml`.

---

## Simulation

Test ambient scenarios without live devices:

```bash
spanda sim examples/solutions/smart-spaces/smart-office/occupancy_climate.sd
```

Fault injection: sudden occupancy spike during night mode, AQ threshold breach, gateway loss mid-mission.

---

## Related

- [building-automation.md](./building-automation.md) — BMS and device control
- [human-interaction.md](./human-interaction.md) — Operator and occupant entities
- [solutions/spatial-computing.md](./solutions/spatial-computing.md) — AR / wearable context
