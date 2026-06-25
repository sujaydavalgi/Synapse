# Device Tree

The device tree models physical ownership from fleet down to individual sensors and actuators.

## Hierarchy

```
fleet
  robots
    compute / controllers
      devices (sensors, actuators, accessories, connectivity, safety)
```

## TOML structure

```toml
[fleet]
id = "warehouse-fleet-a"

[[fleet.robots]]
id = "rover-001"
model = "RoverV1"
hardware_profile = "RoverV1"

[fleet.robots.compute]
id = "jetson-001"
type = "JetsonOrin"
serial = "JTN-001"

[[fleet.robots.compute.devices]]
id = "gps-001"
type = "GPS"
provider = "spanda-gps"
port = "/dev/ttyUSB0"
capabilities = ["read_location", "read_heading"]

[[fleet.robots.compute.devices]]
id = "drive-controller-001"
type = "DifferentialDrive"
provider = "spanda-canbus"
bus = "can0"
capabilities = ["move", "stop", "emergency_stop"]
```

Place fleet/device definitions in `spanda.devices.toml` or `spanda.fleet.toml` and reference them from `[config]` in `spanda.toml`.

## Device fields

| Field | Description |
|-------|-------------|
| `id` | Unique device identifier |
| `type` | Device class (GPS, Lidar, DifferentialDrive, …) |
| `provider` | Spanda provider package name |
| `port` | Serial/USB port path |
| `bus` | CAN or other bus identifier |
| `mount` | Physical mount location |
| `capabilities` | Capability tokens this device exposes |
| `firmware` / `version` | Firmware metadata (warned when missing) |
| `trusted` | Trust flag for actuator control |
| `identity` | Security identity for networked devices |

## CLI inspection

```bash
spanda device-tree graph
spanda device-tree inspect rover-001
spanda map verify patrol.sd --config spanda.toml
```

## Logical-to-physical mapping

`LogicalPhysicalMap` connects program-level robot/sensor/actuator names to configured physical devices. Sensors are classified by type (GPS, Lidar, Camera, IMU). Actuators include drive units, arms, and motors.

Safety rules require actuators to declare `emergency_stop` in `capabilities` and reject `trusted = false` on actuator devices.

## Hardware profiles

Each robot may set `hardware_profile` (e.g. `RoverV1`, `JetsonOrin`). Validation checks that configured devices match the profile's expected sensors and actuators.
