# Smart Space Device Tree

Entity hierarchy examples for [Smart Spaces & Ambient Intelligence](./solutions/smart-spaces.md).

**Canonical config:** `examples/solutions/smart-spaces/spanda.devices.toml` · **Platform guide:** [device-tree.md](./device-tree.md)

---

## Hierarchy

```text
facility (building / campus)
├── zones (floors, rooms, wards, lines)
│   ├── devices (sensors, actuators, locks, HVAC)
│   ├── robots (vacuum, service, inspection)
│   └── humans (occupants, visitors, operators)
├── gateways (protocol bridges)
├── wearables (linked to humans)
├── energy_systems (solar, battery, EV, meter)
└── control_center
```

All nodes use the [Unified Entity Model](./entity-model.md) — `EntityRecord` with `entity_kind` of `facility`, `zone`, `device`, `robot`, `human`, or `wearable`.

---

## Residential (smart home)

```toml
[[facilities]]
id = "home-001"
name = "Oak Street Residence"
entity_kind = "facility"
type = "single_family_home"

[[facilities.zones]]
id = "floor-main"
name = "Main Floor"
type = "floor"

[[facilities.zones]]
id = "room-living"
name = "Living Room"
parent = "floor-main"
type = "room"

[[facilities.gateways]]
id = "matter-hub-001"
type = "MatterHub"
provider = "spanda-matter"
capabilities = ["lighting_control", "climate_control", "access_control"]

[[facilities.zones.devices]]
id = "thermostat-001"
zone = "room-living"
type = "Thermostat"
provider = "spanda-matter"
capabilities = ["climate_control"]

[[facilities.zones.devices]]
id = "smoke-bedroom"
zone = "room-bedroom"
type = "SmokeDetector"
provider = "spanda-matter"
capabilities = ["safety_monitoring"]

[[facilities.zones.devices]]
id = "leak-basement"
zone = "zone-basement"
type = "WaterLeakSensor"
provider = "spanda-zigbee"
capabilities = ["safety_monitoring"]

[[facilities.robots]]
id = "vacuum-001"
type = "RobotVacuum"
provider = "spanda-mqtt"
capabilities = ["robot_assistance", "occupancy_detection"]
```

---

## Commercial (smart building)

```toml
[[facilities]]
id = "tower-a"
name = "Central Tower"
entity_kind = "facility"
type = "commercial_building"

[[facilities.zones]]
id = "floor-12"
name = "Floor 12"
type = "floor"

[[facilities.gateways]]
id = "bacnet-gw-primary"
type = "BACnetGateway"
provider = "spanda-bacnet"
role = "primary"
capabilities = ["climate_control", "environmental_monitoring"]

[[facilities.gateways]]
id = "bacnet-gw-backup"
type = "BACnetGateway"
provider = "spanda-bacnet"
role = "backup"
failover_from = "bacnet-gw-primary"

[[facilities.zones.devices]]
id = "ahu-12"
zone = "floor-12"
type = "HVAC"
provider = "spanda-bacnet"
capabilities = ["climate_control"]

[[facilities.zones.devices]]
id = "co2-lobby"
zone = "zone-lobby"
type = "CO2Sensor"
provider = "spanda-environment"
capabilities = ["environmental_monitoring"]
```

---

## Hospital-at-home

```toml
[[facilities.zones]]
id = "patient-room"
name = "Patient Room"
type = "room"
health_zone = true

[[facilities.humans]]
id = "patient-001"
role = "occupant"
health_opt_in = true
capabilities = ["receive_care"]

[[facilities.wearables]]
id = "watch-patient-001"
type = "SmartWatch"
provider = "spanda-smartwatch"
human_id = "patient-001"
capabilities = ["medical_monitoring", "fall_detection"]

[[facilities.zones.devices]]
id = "bedside-monitor"
zone = "patient-room"
type = "MedicalDevice"
provider = "spanda-mqtt"
capabilities = ["medical_monitoring", "vital_signs"]
```

Bridges [Connected Healthcare](../ROADMAP.md#connected-healthcare) wearable-health example.

---

## Energy systems

```toml
[[facilities.energy_systems]]
id = "solar-array-001"
type = "SolarInverter"
provider = "spanda-energy"
capabilities = ["energy_monitoring", "generation_control"]

[[facilities.energy_systems]]
id = "battery-001"
type = "BatteryStorage"
provider = "spanda-energy"
capabilities = ["energy_storage", "backup_power"]

[[facilities.energy_systems]]
id = "ev-charger-001"
type = "EVCharger"
provider = "spanda-energy"
capabilities = ["ev_charging", "load_shed"]

[[facilities.energy_systems]]
id = "grid-meter-001"
type = "UtilityMeter"
provider = "spanda-modbus"
capabilities = ["energy_monitoring", "grid_import"]
```

---

## Digital twins

```toml
[[twins]]
id = "building-twin-tower-a"
entity_id = "tower-a"
entity_type = "facility"
mirror = ["readiness", "occupancy", "energy", "environment", "emergency_status"]
replay = true

[[twins]]
id = "energy-twin-tower-a"
entity_id = "tower-a"
entity_type = "energy"
mirror = ["generation", "storage_soc", "grid_import", "critical_loads"]
replay = true
```

---

## Continuity links

```toml
[[continuity_pairs]]
primary = "matter-hub-001"
backup = "matter-hub-backup"
on_failure = "promote_backup"
missions = ["night_mode", "leak_response"]

[[continuity_pairs]]
primary = "lighting-ctrl-a"
backup = "emergency-lighting-panel"
on_failure = "life_safety_override"
missions = ["fire_response", "evacuation"]
```

---

## Full fixture

See [examples/solutions/smart-spaces/spanda.devices.toml](../examples/solutions/smart-spaces/spanda.devices.toml) for the complete blueprint device tree with all entity types referenced in the solution blueprint.

---

## Related

- [entity-model.md](./entity-model.md) — Unified Entity Model
- [building-automation.md](./building-automation.md) — BMS integration
- [automotive-device-tree.md](./automotive-device-tree.md) — Parallel pattern (ADAS)
