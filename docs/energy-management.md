# Energy Management

Solar, battery storage, EV charging, demand response, and backup power for [Smart Spaces & Ambient Intelligence](./solutions/smart-spaces.md).

**Status:** Experimental (scaffold) · **Package:** `spanda-energy`

---

## Purpose

Verify and orchestrate energy missions while maintaining life-safety and occupant comfort bounds. Spanda does not replace utility SCADA or inverter firmware — it coordinates **readiness-gated** optimization across distributed assets.

---

## Asset types

| Asset | Entity kind | Capabilities |
|-------|-------------|--------------|
| Solar inverter | `device` | `energy_monitoring`, `generation_control` |
| Battery system | `device` | `energy_storage`, `backup_power` |
| EV charger | `device` | `ev_charging`, `load_shed` |
| Utility meter | `device` | `energy_monitoring`, `grid_import` |
| Smart loads | `device` | `load_control` (HVAC, water heater) |

Device tree examples: [smart-space-device-tree.md](./smart-space-device-tree.md#energy-systems)

---

## Mission types

| Mission | Goal | Readiness checks |
|---------|------|------------------|
| Demand response | Shed non-critical loads | Battery SOC, critical load list |
| Peak shave | Limit grid import | Forecast, inverter health |
| EV schedule | Charge off-peak | Occupant departure time, grid price |
| Backup power | Island critical circuits | Battery capacity, transfer switch |
| Solar optimize | Self-consumption max | Inverter online, meter sync |

Example: [examples/solutions/smart-spaces/energy-management/demand_response.sd](../examples/solutions/smart-spaces/energy-management/demand_response.sd)

---

## Continuity

```text
Grid outage
  → Verify battery SOC and critical load panel
  → Shed comfort loads
  → Maintain egress lighting and medical circuits
  → Log assurance evidence
```

Wi-Fi loss does not block local inverter/BMS paths when BACnet/Modbus gateways are on redundant Ethernet.

---

## Assurance evidence

Energy assurance bundles include:

- Pre-mission SOC and load snapshot
- Devices included / excluded from shed
- Grid price signal source and timestamp
- Post-mission energy delta
- Operator approval record (if required)

```bash
spanda verify examples/solutions/smart-spaces/energy-management/demand_response.sd \
  --capabilities --config examples/solutions/smart-spaces/spanda.toml
```

---

## Packages

| Package | Role |
|---------|------|
| `spanda-energy` | Solar, battery, EV, demand-response providers |
| `spanda-modbus` | Inverter / meter Modbus |
| `spanda-mqtt` | Cloud energy APIs |
| `spanda-bacnet` | BMS load control points |

---

## Control Center

Energy panels (experimental Smart Spaces tab):

- Real-time generation / consumption
- Battery SOC and backup reserve
- EV session status
- Demand-response event timeline
- Readiness blockers for optimization missions

---

## Related

- [building-automation.md](./building-automation.md) — HVAC load interaction
- [smart-space-readiness.md](./smart-space-readiness.md) — Pre-optimization gates
- [solutions/smart-spaces.md](./solutions/smart-spaces.md) — Full blueprint
