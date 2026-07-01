# Smart Space BMS sidecar bridge

Use an external **building management sidecar** when BACnet/KNX field buses are already aggregated by Home Assistant, OpenHAB, MQTT, or a vendor BMS. Spanda remains the **safety and readiness orchestrator**; the sidecar stays the **device authority**.

**Related:** [smart-space-packages.md](./smart-space-packages.md) · [building-automation.md](./building-automation.md) · [iot.md](./iot.md) · [solutions/smart-spaces.md](./solutions/smart-spaces.md)

---

## When to use a sidecar vs direct field bus

| Approach | Use when |
|----------|----------|
| **Direct BACnet/KNX** (`spanda-bacnet`, `spanda-knx`) | Spanda gateway has L2/L3 access to the BMS network; you need low-latency reads for readiness |
| **Home Assistant sidecar** | Residents/operators already use HA; devices are paired in HA; Spanda runs verified missions on top |
| **MQTT sidecar** | BMS or IoT hub publishes normalized telemetry to MQTT; Spanda consumes via `spanda-mqtt` |
| **REST/OpenHAB** | Vendor exposes HTTP device API; wrap with `SPANDA_*_CMD` or Python bridge handlers |

---

## Architecture

```text
Field devices (BACnet / KNX / Matter / Zigbee)
        │
        ▼
  Sidecar hub (Home Assistant, BMS, MQTT broker)
        │
        ▼
  Spanda provider bridge (package or env cmd)
        │
        ▼
  Readiness · verify · missions · Control Center
```

Layering matches [smart-space-packages.md](./smart-space-packages.md): external hubs own pairing; Spanda packages read state and issue **verified commands** only after readiness gates pass.

---

## Home Assistant sidecar

### Provider config

Enable in `examples/solutions/smart-spaces/spanda.providers.toml`:

```toml
[providers.home_assistant]
package = "spanda-home-assistant"
enabled = true
url = "http://homeassistant.local:8123"
```

### Runtime env

```bash
export SPANDA_LIVE_HOME_ASSISTANT=1
export SPANDA_HOME_ASSISTANT_URL=http://127.0.0.1:8123
export SPANDA_HOME_ASSISTANT_TOKEN=your-long-lived-access-token
export SPANDA_HOME_ASSISTANT_CMD='packages/registry/spanda-home-assistant/scripts/get_state.sh {entity}'
```

Or rely on `home_assistant_get_state` in `spanda_python_bridge.py` when no cmd is set.

Force mock (CI):

```bash
export SPANDA_HOME_ASSISTANT_FORCE_MOCK=1
```

### Map entities to blueprint devices

| Blueprint device | HA entity example |
|------------------|-------------------|
| `leak-basement` | `binary_sensor.basement_leak` |
| `co2-lobby` | `sensor.lobby_co2` |
| `ahu-12` | `climate.floor_12_ahu` |

Twin and readiness config reference logical device ids; provider dispatch resolves live values through the bridge.

---

## MQTT sidecar

When a BMS publishes to MQTT:

1. Enable `spanda-mqtt` in `spanda.providers.toml`.
2. Set `SPANDA_LIVE_MQTT=1` and broker URL (see [iot.md](./iot.md)).
3. Map topics in mission code or provider bootstrap — e.g. `building/tower-demo/ahu-12/present-value`.

Spanda does not ship a universal topic map; each deployment documents topics beside `spanda.devices.toml`.

---

## Direct BACnet/KNX (no sidecar)

For commercial towers without a hub intermediary:

```bash
cargo build -p spanda --features live-building
export SPANDA_LIVE_BACNET=1
export SPANDA_BACNET_NETWORK=10.0.12.50/24
export SPANDA_BACNET_TARGET=10.0.12.100
export SPANDA_ROOT=/path/to/Spanda   # optional — locates registry scripts
```

Read order: `SPANDA_BACNET_CMD` → registry `read_point.py` (with `live-building`) → bacpypes3 Python bridge → mock.

---

## Verification

```bash
./scripts/smart_spaces_bms_sidecar_smoke.sh          # mock CI
SPANDA_LIVE_IOT_HARDWARE=1 ./scripts/smart_spaces_live_iot_smoke.sh  # field buses
SPANDA_BMS_SIDECAR_LIVE=1 ./scripts/smart_spaces_bms_sidecar_smoke.sh # live HA (optional)
```

---

## Operational notes

- Keep life-safety sensors on certified paths; do not route fire panel inputs only through consumer hubs without engineering review.
- Use `spanda readiness` and `spanda verify` before enabling automations that act on sidecar state.
- Document sidecar entity ↔ Spanda device mapping in facility runbooks for auditors.
