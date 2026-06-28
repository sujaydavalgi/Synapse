# Wearables

Wearables are **device-tree nodes** registered in the device pool with provider-backed telemetry. Spanda does not embed wearable vendor SDKs in core — optional packages bridge to platform APIs (HealthKit, Wear OS, industrial BLE vests, etc.).

**Related:** [human-interaction.md](./human-interaction.md) · [hri-packages.md](./hri-packages.md) · [human-readiness.md](./human-readiness.md)

---

## Supported wearable types

| Type | Examples | Typical capabilities |
|------|----------|---------------------|
| Smart watch | Apple Watch, Wear OS | `heart_rate`, `connectivity_status`, `battery_level` |
| Fitness band | Fitbit-class | `heart_rate`, `step_count`, `fatigue_hint` |
| Smart helmet | Industrial hard hat + sensors | `impact_detection`, `connectivity_status` |
| Smart glasses | Non-AR smart glasses | `camera_stream`, `connectivity_status` |
| Body camera | Law enforcement / field service | `video_stream`, `gps_location` |
| Smart vest | Industrial safety vest | `fall_detection`, `proximity_alert` |
| Heart rate sensor | Chest strap, medical | `heart_rate`, `stress_index` |
| EEG device | Research / fatigue monitoring | `eeg_signal` (opt-in) |
| EMG device | Exoskeleton / rehab | `emg_signal` (opt-in) |
| Industrial wearable | PPE with BLE tags | `geofence_alert`, `gas_detection` |
| Medical wearable | Patient monitor | `vitals`, `fall_detection` |
| Fall detection device | Pendant, wristband | `fall_detection`, `sos_alert` |

---

## Device tree declaration

```toml
[[fleet.wearables]]
id = "vest-001"
type = "SmartVest"
provider = "spanda-industrial-wearables"
human_id = "operator-001"
capabilities = ["fall_detection", "proximity_alert", "battery_level"]
protocol = "ble"
endpoint = "ble://AA:BB:CC:DD:EE:01"
trust_level = "verified"
```

Flat registry (`[[devices]]`) is also supported with `human_id` binding.

---

## Discovery

Wearables are discovered through existing discovery transports:

- **BLE** — `spanda-discovery-ble` + `spanda-ble`
- **Wi-Fi** — `spanda-discovery-wifi`
- **USB** — body cams, EEG dongles via `spanda-discovery-usb`

```bash
spanda device discover --transport ble
spanda control-center devices discover --transport ble
```

Discovered wearables ingest into the device pool with `type = Wearable` and optional `human_id` assignment.

---

## Readiness integration

Wearable dimensions in human readiness profiles:

| Dimension | Source |
|-----------|--------|
| Connectivity | Last telemetry timestamp |
| Battery | `battery_level` capability |
| Sensor health | Provider health check |
| Assignment | `human_id` matches mission operator |

Optional health dimensions (heart rate, fatigue, stress) require `SPANDA_HUMAN_HEALTH_ENABLED=1` — see [human-readiness.md](./human-readiness.md).

---

## Reference packages

| Package | Devices | Status |
|---------|---------|--------|
| `spanda-smartwatch` | Smart watches | **Planned** |
| `spanda-industrial-wearables` | Vests, helmets, gas tags | **Planned** |
| `spanda-bodycam` | Body cameras | **Planned** |

Full list: [hri-packages.md](./hri-packages.md).

---

## Example

`examples/solutions/spatial-computing/wearable-health/` — optional health monitoring with privacy controls for Connected Healthcare deployments.
