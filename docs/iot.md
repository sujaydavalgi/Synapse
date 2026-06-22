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
| `spanda-opcua` | OPC-UA (stub) |
| `spanda-modbus` | Modbus (stub) |
| `spanda-canbus` | CAN bus (stub) |
| `spanda-zigbee` | Zigbee (stub) |
| `spanda-lora` | LoRa (stub) |
| `spanda-matter` | Matter (stub) |

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
