# IoT Support (Package-First)

IoT integrations live in official packages. Core defines generic contracts; packages implement protocols.

## Core contracts

| Trait | Purpose |
|-------|---------|
| `IoTDeviceProvider` | Device lifecycle and identity |
| `TelemetryProvider` | Sensor reading ingestion |
| `CommandProvider` | Remote command dispatch |
| `DeviceShadowProvider` | Desired/reported state sync |

## Core types

`IoTDevice`, `DeviceShadow`, `Telemetry`, `Command`, `SensorReading`, `ActuatorCommand`

## Official packages

| Package | Protocol |
|---------|----------|
| `spanda-iot-core` | Base contracts and types |
| `spanda-mqtt` | MQTT pub/sub |
| `spanda-ble` | Bluetooth LE |
| `spanda-wifi` | WiFi connectivity |
| `spanda-cellular` | LTE/cellular |
| `spanda-opcua` | OPC-UA (stub + live bridge) |
| `spanda-modbus` | Modbus (stub + live TCP) |
| `spanda-canbus` | CAN bus (stub + live bridge) |
| `spanda-zigbee` | Zigbee (stub + live bridge) |
| `spanda-lora` | LoRa (stub + live bridge) |
| `spanda-matter` | Matter (stub + live bridge) |
| `spanda-thread` | Thread mesh (stub) |
| `spanda-zwave` | Z-Wave (stub) |
| `spanda-bacnet` | BACnet building automation (bacpypes3 + env bridge) |
| `spanda-knx` | KNX building bus (xknx + env bridge) |
| `spanda-home-assistant` | Home Assistant bridge (REST + mock) |
| `spanda-energy` | Solar, battery, and demand-response (stub) |
| `spanda-building` | Facility zones and readiness orchestration (stub) |
| `spanda-smart-locks` | Smart lock and access control (stub) |
| `spanda-environment` | Air quality and environmental sensing (stub) |

See [solutions/smart-spaces.md](solutions/smart-spaces.md) for the Smart Spaces Solution Blueprint.

## Example

```spanda
device TemperatureSensor {
    protocol: mqtt;
    topic: "/factory/temp";
}
```

Install packages via `spanda add spanda-mqtt`.

**Example:** [`examples/iot/modbus_dispatch.sd`](../examples/iot/modbus_dispatch.sd) · Golden path: `./scripts/live_iot_golden_path.sh`

## Runtime dispatch

When official IoT packages are installed, module imports dispatch through `package_dispatch`:

| Module | Function | Package |
|--------|----------|---------|
| `iot.device` | `register` | `spanda-iot-core` |
| `iot.telemetry` | `publish` | `spanda-iot-core` |
| `iot.modbus` | `read_register` | `spanda-modbus` |
| `iot.opcua` | `read_node` | `spanda-opcua` |
| `iot.zigbee` | `read_attribute` | `spanda-zigbee` |
| `iot.lora` | `read_payload` | `spanda-lora` |
| `iot.matter` | `read_cluster` | `spanda-matter` |
| `iot.canbus` | `read_frame` | `spanda-canbus` |
| `iot.thread` | `read_endpoint` | `spanda-thread` |
| `iot.zwave` | `read_value` | `spanda-zwave` |
| `iot.bacnet` | `read_point` | `spanda-bacnet` |
| `iot.knx` | `read_group_address` | `spanda-knx` |
| `bridge.home_assistant` | `get_state` | `spanda-home-assistant` |
| `energy.solar` | `read_generation` | `spanda-energy` |
| `building.entity` | `facility_readiness` | `spanda-building` |
| `access.lock` | `lock_state` | `spanda-smart-locks` |
| `environment.aq` | `read_aq` | `spanda-environment` |

## Live hardware (optional)

Enable live reads with environment flags (build with `--features live-iot` on `spanda-cli` for native Modbus TCP):

| Variable | Purpose |
|----------|---------|
| `SPANDA_LIVE_MODBUS=1` | Read holding registers from Modbus TCP hardware |
| `SPANDA_MODBUS_HOST` | Modbus host (default `127.0.0.1`) |
| `SPANDA_MODBUS_PORT` | Modbus port (default `502`) |
| `SPANDA_MODBUS_UNIT` | Modbus unit/slave id (default `1`) |
| `SPANDA_LIVE_OPCUA=1` | Read nodes via Python bridge (`asyncua`) |
| `SPANDA_LIVE_ZIGBEE=1` | Read Zigbee attributes via Python bridge |
| `SPANDA_LIVE_LORA=1` | Read LoRa payloads via Python bridge |
| `SPANDA_LIVE_MATTER=1` | Read Matter clusters via Python bridge |
| `SPANDA_LIVE_CANBUS=1` | Read CAN frames via Python bridge |
| `SPANDA_LIVE_BACNET=1` | Read BACnet points via `SPANDA_BACNET_CMD` or Python bridge |
| `SPANDA_BACNET_CMD` | Shell template for BACnet reads (`{device}`, `{object_id}`) |
| `SPANDA_BACNET_NETWORK` | Local BACnet/IP bind (bacpypes3), e.g. `192.168.1.50/24` |
| `SPANDA_BACNET_TARGET` | Remote BACnet device IP for bacpypes3 reads |
| `SPANDA_BACNET_OBJECT` | Default object id when `object_id` is a property name |
| `SPANDA_BACNET_FORCE_MOCK` | Force mock BACnet reads (`1` for CI) |
| `SPANDA_LIVE_KNX=1` | Read KNX group addresses via `SPANDA_KNX_CMD` or Python bridge |
| `SPANDA_KNX_CMD` | Shell template for KNX reads (`{address}`) |
| `SPANDA_KNX_GATEWAY` | KNX/IP gateway IP for xknx reads |
| `SPANDA_KNX_VALUE_TYPE` | Optional xknx decode hint (`temperature`, etc.) |
| `SPANDA_KNX_FORCE_MOCK` | Force mock KNX reads (`1` for CI) |
| `SPANDA_LIVE_THREAD=1` | Read Thread endpoints via `SPANDA_THREAD_CMD` or Python bridge |
| `SPANDA_THREAD_CMD` | Shell template for Thread reads (`{device}`) |
| `SPANDA_LIVE_ZWAVE=1` | Read Z-Wave values via `SPANDA_ZWAVE_CMD` or Python bridge |
| `SPANDA_ZWAVE_CMD` | Shell template for Z-Wave reads (`{device}`, `{object_id}`) |
| `SPANDA_LIVE_HOME_ASSISTANT=1` | Read HA entity state via `SPANDA_HOME_ASSISTANT_CMD`, registry script, or Python bridge |
| `SPANDA_HOME_ASSISTANT_CMD` | Shell template for HA reads (`{entity}`) |
| `SPANDA_HOME_ASSISTANT_URL` | Home Assistant base URL for REST reads (Python bridge) |
| `SPANDA_HOME_ASSISTANT_TOKEN` | Long-lived HA access token |
| `SPANDA_HOME_ASSISTANT_FORCE_MOCK` | Force mock HA reads (`1` for CI) |
| `SPANDA_HOME_ASSISTANT_READ_SCRIPT` | Override path to `get_state.py` registry script |
| `SPANDA_ROOT` | Repo root for locating registry BACnet/KNX/HA scripts |
| `SPANDA_BACNET_READ_SCRIPT` | Override path to BACnet `read_point.py` |
| `SPANDA_KNX_READ_SCRIPT` | Override path to KNX `read_group.py` |

Build with `--features live-building` on `spanda-cli` to enable registry BACnet/KNX script dispatch before the Python bridge.

Golden path (mock fallback without hardware): `./scripts/live_iot_golden_path.sh`

Smart Spaces building I/O smoke: `./scripts/smart_spaces_live_iot_smoke.sh`

BMS sidecar (Home Assistant / MQTT patterns): `./scripts/smart_spaces_bms_sidecar_smoke.sh` — see [smart-space-bms-bridge.md](smart-space-bms-bridge.md)

## Persistent storage

Device telemetry published through `iot.telemetry.publish` is mirrored to the local append-only store when persistence is enabled (`--persist-telemetry` or `SPANDA_TELEMETRY_STORE=1`). Sensor reads, task heartbeats, and device liveness (`heartbeat`/`liveness`/`alive`/`ping` metrics, device registration, fleet agent health) are recorded the same way.

See [telemetry-store.md](./telemetry-store.md) for file layout and `spanda telemetry` CLI commands.
