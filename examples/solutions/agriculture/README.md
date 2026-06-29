# Agriculture — Official Solution Blueprint

Autonomous field robots, precision row following, and crop monitoring — composed from GPS, geofencing, health policies, and recovery without core language extensions.

**Profile:** `agriculture` · **Status:** Experimental (scaffold) · **Doc:** [docs/solutions/agriculture.md](../../docs/solutions/agriculture.md)

---

## Quick start

```bash
cd examples/solutions/agriculture
spanda check field_patrol.sd
spanda verify field_patrol.sd --target FieldRobotV1
spanda sim field_patrol.sd
spanda readiness field_patrol.sd --json
```

---

## Blueprint layout

```
agriculture/
├── README.md
├── spanda.toml
├── field_patrol.sd          # Row patrol with geofence + connectivity recovery
└── (planned) spray_mission.sd, harvest_convoy.sd
```

---

## Platform pillars used

| Pillar | Capabilities |
|--------|--------------|
| Device & Fleet | Device tree, health policies, OTA |
| Verification | Readiness, recovery policies, verify |
| Packages | `spanda-gps`, `spanda-cellular`, `spanda-lora`, `spanda-opencv` |
| Operations | Fleet map, readiness trends |

---

## Related

- ADAS agricultural vehicle variant: [../adas/applications/agricultural/](../adas/applications/agricultural/)
- Architecture: [docs/solutions/agriculture.md](../../docs/solutions/agriculture.md)
- Geofencing: [docs/geofencing.md](../../docs/geofencing.md)
