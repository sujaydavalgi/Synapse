# Environmental Monitoring — Solution Blueprint

**Status:** Experimental (scaffold) · **Timeline:** Later · **Path:** [examples/solutions/environmental-monitoring/](../../examples/solutions/environmental-monitoring/)

Official Solution Blueprint for distributed sensor networks monitoring air, water, and environmental conditions.

**Full roadmap entry:** [ROADMAP.md § Environmental Monitoring](../../ROADMAP.md#environmental-monitoring)

---

## Purpose

Deploy long-life battery sensor nodes with mesh telemetry, calibration gates, baseline drift detection, and cloud ingest for trend analysis and alerting.

## Platform pillars used

| Pillar | Capabilities |
|--------|--------------|
| Device & Fleet | Device tree, health policies (battery, connectivity), OTA firmware |
| Operations | Telemetry store, OTLP export, alerting, readiness trends |
| Packages & Ecosystem | `spanda-lora`, `spanda-mqtt`, `spanda-cellular`, `spanda-otel-collector` |
| Verification | Calibration readiness, assurance drift on baselines |

## Reference architecture (planned)

```text
Sensor Network
├── Field nodes (LoRa / BLE)
├── Gateway (MQTT / cellular backhaul)
├── Edge readiness (calibration, battery require)
├── Telemetry store (JSONL / SQLite / OTLP)
└── Control Center dashboards + alert rules
```

## Device tree (planned)

| Node | Role |
|------|------|
| `sensor_node` | Air/water/soil sensor |
| `gateway` | Mesh aggregator |
| `cloud_ingest` | Telemetry persistence |

## Packages & providers

| Package | Role |
|---------|------|
| `spanda-lora` | Low-power mesh |
| `spanda-mqtt` | Gateway publish |
| `spanda-cellular` | Remote site backhaul |
| `spanda-otel-collector` | OTLP pipeline |
| `spanda-anomaly` | Baseline drift detection |

## Related docs

- [iot.md](../iot.md)
- [telemetry-store.md](../telemetry-store.md)
- [calibration.md](../calibration.md)
- [health-checks.md](../health-checks.md)
- [anomaly-detection.md](../anomaly-detection.md)

## Example projects

- [examples/solutions/environmental-monitoring/](../../examples/solutions/environmental-monitoring/) — `sensor_mesh.sd`, `gateway_bridge.sd` (CI: `scripts/solution_blueprints_smoke.sh`)

## Simulation & replay

- Fault injection on sensor readings
- Historical trend replay via telemetry sessions

---

**Related blueprints:** [Agriculture](./agriculture.md) · [Critical Infrastructure](./adas.md) (compliance overlap)
