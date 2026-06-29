# Smart Space Readiness

Operational go/no-go gates before mode changes, emergency missions, and energy optimization in [Smart Spaces & Ambient Intelligence](./solutions/smart-spaces.md).

**Config:** `examples/solutions/smart-spaces/spanda.readiness.toml` · **Profile:** `smart_space`

---

## Pre-mode checklist

Before night mode, away mode, lockdown, or evacuation missions activate:

| Factor | Weight | Checks |
|--------|--------|--------|
| Gateway availability | 20% | Primary hub online; backup reachable if required |
| Network connectivity | 10% | Wi-Fi / Ethernet / cellular per policy |
| Device health | 15% | Critical devices Healthy in pool |
| Battery levels | 10% | Wireless sensors above minimum SOC |
| Calibration | 5% | Environmental sensors within max age |
| Security status | 15% | Locks, cameras, tamper clear |
| Critical sensors | 15% | Smoke, CO, leak quorum |
| Emergency systems | 10% | Fire panel comms, egress lighting path |

Default `min_score = 85` for comfort missions; `min_score = 95` for evacuation and lockdown.

---

## Example questions

| Question | Blocking dimensions |
|----------|---------------------|
| Can the building safely enter night mode? | Locks secured, leak sensors online, gateway healthy, no open critical alerts |
| Can emergency evacuation operate? | Fire sensors quorum, PA path, exit lighting, robot obstruction clear |
| Can HVAC automation continue if Wi-Fi fails? | Local BACnet/KNX/Thread path verified, backup gateway role assigned |

---

## Required devices (residential profile)

From `spanda.readiness.toml`:

**Required:** primary gateway, front door lock, smoke detector (sleep areas), water leak (basement/kitchen)

**Optional (degraded without):** robot vacuum, solar inverter, EV charger

---

## Required devices (commercial profile)

**Required:** BMS gateway, fire panel interface, elevator recall path (if integrated), perimeter access quorum

**Optional:** service robots, demand-response meter

---

## CLI

```bash
# Full readiness report
spanda readiness examples/solutions/smart-spaces/smart-home/night_mode.sd \
  --profile smart_space \
  --config examples/solutions/smart-spaces/spanda.toml \
  --json

# Building floor rollup
spanda readiness examples/solutions/smart-spaces/smart-building/floor_readiness.sd \
  --profile smart_space \
  --config examples/solutions/smart-spaces/spanda.toml \
  --runtime
```

---

## Mission-specific gates

| Mission | Minimum capabilities | Minimum readiness |
|---------|---------------------|-------------------|
| Night mode | `lighting_control`, `climate_control`, `access_control` | 85 |
| Fire response | `safety_monitoring`, `emergency_notification` | 95 |
| Water leak response | `safety_monitoring` | 90 |
| Demand response | `energy_monitoring`, `load_control` | 85 |
| Patient monitoring | `medical_monitoring` (opt-in) | 90 |
| Building lockdown | `access_control`, `safety_monitoring` | 95 |

---

## Redundancy

| Component | Readiness rule |
|-----------|----------------|
| Gateway | Primary online OR backup promoted via continuity policy |
| Lighting controller | Life-safety path independent of comfort controller |
| BMS | At least one field bus path (BACnet/IP or KNX) |
| Medical wearable | BLE + hub path OR cellular fallback |

---

## Control Center

Smart Spaces readiness panel shows:

- Per-zone readiness score
- Blocking dimensions with device links
- Gateway primary/backup role
- Emergency system test due dates
- Last assurance snapshot before mode change

```bash
spanda control-center serve \
  --config examples/solutions/smart-spaces/spanda.toml \
  --program examples/solutions/smart-spaces/smart-home/night_mode.sd
```

---

## Related

- [readiness.md](./readiness.md) — Platform readiness engine
- [smart-space-security.md](./smart-space-security.md) — Security dimensions
- [mission-continuity.md](./mission-continuity.md) — Failover when readiness degrades mid-mission
