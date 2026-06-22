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

## Example

```spanda
device TemperatureSensor {
    protocol: mqtt;
    topic: "/factory/temp";
}
```

Install packages via `spanda add spanda-mqtt`.

## Agent capability enforcement

When an agent declares `can [ ... ]`, runtime enforces the list. An empty `can []` denies high-risk actions (`execute`, `propose_motion`) by default. Capability grant/deny events are written to the audit trail when configured.

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

Golden path (mock fallback without hardware): `./scripts/live_iot_golden_path.sh`
