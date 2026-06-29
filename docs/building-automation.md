# Building Automation

Building management, HVAC, lighting, access control, and life-safety orchestration for the [Smart Spaces & Ambient Intelligence](./solutions/smart-spaces.md) blueprint.

**Status:** Experimental (scaffold) · **Profile:** `smart_space`

Spanda does **not** replace BACnet/KNX BMS products or home automation hubs. It verifies readiness, orchestrates missions, and produces assurance evidence across those systems.

---

## Scope

| Layer | Spanda role | Typical external system |
|-------|-------------|-------------------------|
| Device control | Mission triggers via providers | Matter hub, BMS controller |
| Scheduling | Readiness-gated mode changes | Home Assistant automations (optional bridge) |
| Life safety | Verify + escalate + evidence | Fire panel, UL-listed devices |
| Access | Trust-verified lock missions | Smart lock vendor cloud |
| Energy | Optimize within readiness bounds | Utility / inverter APIs |

---

## Reference architectures

### Smart home

```text
Home (facility)
├── Gateway (Matter / Zigbee hub)
│   ├── Thermostat → climate_control
│   ├── Locks → access_control
│   ├── Smoke / leak sensors → safety_monitoring
│   └── Lights → lighting_control
├── Occupants (human entities)
├── Robot vacuum (robot)
└── Energy meter + solar (optional)
```

Example: [examples/solutions/smart-spaces/smart-home/](../examples/solutions/smart-spaces/smart-home/)

### Smart office

```text
Office floor (zone)
├── BACnet gateway → HVAC zones
├── Occupancy sensors → climate_control
├── Access readers → access_control
└── Cleaning robot → robot_assistance
```

Example: [examples/solutions/smart-spaces/smart-office/](../examples/solutions/smart-spaces/smart-office/)

### Smart building

```text
Commercial tower (facility)
├── Floors (zones)
│   ├── Rooms
│   └── Redundant gateways (primary + backup)
├── Central BMS (BACnet / KNX)
├── Emergency systems (fire, PA, egress lighting)
└── Control Center NOC operator
```

Example: [examples/solutions/smart-spaces/smart-building/](../examples/solutions/smart-spaces/smart-building/)

---

## Protocol integration

| Building type | Primary packages |
|---------------|------------------|
| Residential | `spanda-matter`, `spanda-zigbee`, `spanda-thread`, `spanda-zwave` |
| Commercial | `spanda-bacnet`, `spanda-knx`, `spanda-modbus` |
| Mixed / retrofit | `spanda-home-assistant`, `spanda-mqtt` |

Provider pattern: packages implement `iot.*` import paths; missions use `requires capabilities` — see [iot.md](./iot.md).

---

## Mode missions

| Mode | Preconditions (readiness) | Actions (via providers) |
|------|---------------------------|-------------------------|
| Night mode | Locks secure, leak sensors online, gateway healthy | Dim lights, setback HVAC, arm perimeter |
| Away mode | Occupancy clear, windows closed | Reduce HVAC, enable security recording |
| Lockdown | Operator approval, cameras online | Deny entry, record all access attempts |
| Maintenance | BMS path verified, local override available | Isolate zones, notify occupants |

Readiness profile: [smart-space-readiness.md](./smart-space-readiness.md)

---

## Life safety

Life-safety devices remain on certified hardware paths. Spanda:

1. Verifies sensor quorum and battery levels before trusting automation.
2. Escalates fire / CO / leak events to emergency missions.
3. Records assurance evidence for inspection and insurance.
4. Never blocks UL-listed panel functions — orchestrates *around* them.

Emergency examples: [examples/solutions/smart-spaces/emergency-response/](../examples/solutions/smart-spaces/emergency-response/)

---

## Digital twin

Building automation twins mirror:

- Zone setpoints and actuals
- Device online / degraded state
- Gateway redundancy role (primary / standby)
- Last readiness score per floor

See [digital-twin.md](./digital-twin.md) and `spanda.devices.toml` twins section in the smart-spaces blueprint.

---

## Related

- [ambient-intelligence.md](./ambient-intelligence.md) — Context-aware adaptation
- [smart-space-device-tree.md](./smart-space-device-tree.md) — Entity hierarchy
- [smart-space-packages.md](./smart-space-packages.md) — Optional packages
