# Environmental Monitoring — Official Solution Blueprint

Distributed sensor networks for air, water, and environmental conditions — composed from mesh telemetry, calibration gates, and drift-aware operations.

**Profile:** `environmental` · **Status:** Experimental (scaffold) · **Doc:** [docs/solutions/environmental-monitoring.md](../../docs/solutions/environmental-monitoring.md)

---

## Quick start

```bash
cd examples/solutions/environmental-monitoring
spanda check sensor_mesh.sd
spanda verify sensor_mesh.sd --target SensorNodeV1
spanda sim sensor_mesh.sd
spanda readiness sensor_mesh.sd --json
```

---

## Blueprint layout

```
environmental-monitoring/
├── README.md
├── spanda.toml
├── sensor_mesh.sd           # Mesh node sampling with connectivity recovery
└── (planned) gateway_bridge.sd, alert_threshold.sd
```

---

## Platform pillars used

| Pillar | Capabilities |
|--------|--------------|
| Device & Fleet | Device tree, health policies, OTA |
| Operations | Telemetry store, OTLP export, alerting |
| Packages | `spanda-lora`, `spanda-mqtt`, `spanda-cellular`, `spanda-otel-collector` |
| Verification | Calibration readiness, baseline drift |

---

## Related

- [docs/solutions/environmental-monitoring.md](../../docs/solutions/environmental-monitoring.md)
- [docs/telemetry-store.md](../../docs/telemetry-store.md)
- [docs/iot.md](../../docs/iot.md)
