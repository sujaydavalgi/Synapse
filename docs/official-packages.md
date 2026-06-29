# Official Packages

First-party Spanda packages live under `packages/registry/`. Each package includes `spanda.toml`, source exports, tests, and a README.

## Connectivity and positioning

| Package | Import path | Description |
|---------|-------------|-------------|
| `spanda-gps` | `positioning.gps` | GPS/GNSS receiver adapters |
| `spanda-wifi` | `connectivity.wifi` | Wi-Fi connectivity |
| `spanda-ble` | `connectivity.ble` | Bluetooth Low Energy |
| `spanda-cellular` | `connectivity.cellular` | LTE/4G/5G cellular |

## Communication and robotics middleware

| Package | Import path | Description |
|---------|-------------|-------------|
| `spanda-mqtt` | `communication.mqtt` | MQTT pub/sub transport |
| `spanda-dds` | `communication.dds` | DDS transport |
| `spanda-ros2` | `robotics.ros2` | ROS 2 integration |

## Navigation, SLAM, manipulation

| Package | Import path | Description |
|---------|-------------|-------------|
| `spanda-nav` | `navigation.path_planning` | Path planning and navigation |
| `spanda-slam` | `navigation.slam` | SLAM localization and mapping |
| `spanda-moveit` | `manipulation.moveit` | MoveIt motion planning |

Specialized adapter packages (examples under `examples/packages/`):

- `spanda-nav2` → `navigation.nav2`
- `spanda-cartographer` → `navigation.cartographer`
- `spanda-rtabmap` → `navigation.rtabmap`

## Vision and AI

| Package | Import path | Description |
|---------|-------------|-------------|
| `spanda-opencv` | `vision.opencv` | OpenCV bindings |
| `spanda-yolo` | `vision.yolo` | YOLO object detection |
| `spanda-openai` | `ai.openai` | OpenAI LLM via Python bridge |

## Simulation

| Package | Import path | Description |
|---------|-------------|-------------|
| `spanda-gazebo` | `sim.gazebo` | Gazebo backend |
| `spanda-webots` | `sim.webots` | Webots backend |

## Automotive sensors and protocols

| Package | Import path | Description |
|---------|-------------|-------------|
| `spanda-radar` | `sensors.radar` | Automotive radar adapters |
| `spanda-lidar` | `sensors.lidar` | Automotive LiDAR adapters |
| `spanda-ultrasonic` | `sensors.ultrasonic` | Ultrasonic parking sensors |
| `spanda-automotive-ethernet` | `automotive.ethernet` | SOME/IP, DoIP, Automotive Ethernet |
| `spanda-lin` | `automotive.lin` | LIN bus protocol |
| `spanda-uds` | `automotive.uds` | UDS/ISO-TP diagnostics |
| `spanda-v2x` | `automotive.v2x` | DSRC / C-V2X communication |

See [solutions/adas.md](solutions/adas.md) for the ADAS Solution Blueprint.

## Smart spaces and building automation

| Package | Import path | Description |
|---------|-------------|-------------|
| `spanda-thread` | `iot.thread` | Thread mesh networking |
| `spanda-zwave` | `iot.zwave` | Z-Wave home automation |
| `spanda-bacnet` | `iot.bacnet` | BACnet building automation |
| `spanda-knx` | `iot.knx` | KNX building control bus |
| `spanda-home-assistant` | `bridge.home_assistant` | Home Assistant bridge |
| `spanda-energy` | `energy.solar` | Solar, battery, and demand response |
| `spanda-building` | `building.entity` | Facility zones and readiness |
| `spanda-smart-locks` | `access.lock` | Smart lock and access control |
| `spanda-environment` | `environment.aq` | Air quality and environmental sensing |

See [solutions/smart-spaces.md](solutions/smart-spaces.md) for the Smart Spaces Solution Blueprint.

Aliases: `spanda-sim-gazebo`, `spanda-sim-webots` (registry metadata).

## Platform services

| Package | Import path | Description |
|---------|-------------|-------------|
| `spanda-fleet` | `robotics.fleet` | Multi-robot fleet orchestration |
| `spanda-ota` | `deploy.ota` | OTA deploy and rollout |
| `spanda-maintenance` | `maintenance.health` | Predictive maintenance |
| `spanda-ledger` | `provenance.ledger` | Audit ledger anchoring |
| `spanda-cloud` | `cloud.remote` | Cloud telemetry and remote commands |

## Mission assurance

| Package | Import path | Description |
|---------|-------------|-------------|
| `spanda-assurance` | `assurance.evidence` | Assurance case and evidence scaffolds |
| `spanda-knowledge-model` | `assurance.knowledge` | System knowledge models |
| `spanda-anomaly` | `assurance.anomaly` | Anomaly detection backends |
| `spanda-diagnosis` | `assurance.diagnosis` | Fault diagnosis and root cause |
| `spanda-prognostics` | `assurance.prognostics` | Prognostics and RUL |
| `spanda-mission-planning` | `assurance.mission` | Mission planning assurance |
| `spanda-mission-continuity` | `assurance.continuity` | Mission continuity and takeover assurance |
| `spanda-resilience` | `assurance.resilience` | Resilience and recovery policies |
| `spanda-fusion` | `assurance.fusion` | Weighted sensor fusion backends |

See [mission-assurance.md](mission-assurance.md) for language constructs and CLI commands.

## Package layout

```
packages/registry/spanda-gps/
├── spanda.toml
├── README.md
├── src/
│   └── positioning_gps.sd
├── tests/
│   └── smoke.sd
└── examples/
```

## Adding a dependency

```toml
# spanda.toml
[dependencies]
spanda-gps = "0.1"
```

```spanda
import positioning.gps;
```

## Registry provenance (provider wiring)

Built-in provider bootstrap uses **provenanced** dependencies only — not every package name in the official catalog.

| How you depend | `bootstrap_providers_for_packages` wires live backends? |
|----------------|--------------------------------------------------------|
| `spanda-gps = "0.1"` (registry) | Yes |
| `spanda-gps = { path = "../../packages/registry/spanda-gps" }` | Yes (canonical monorepo tree) |
| `spanda-gps = { path = "../my-fork" }` | **No** — stub `.sd` only |

Path or git overrides of official names emit an `official_provenance` validation warning. Production deploy gates block them. See [how-packages-work.md](./how-packages-work.md).

## Verify adapter metadata

```bash
spanda verify-adapter packages/registry/spanda-ros2
spanda registry info spanda-mqtt
```

## Status

| Tier | Packages | Runtime behavior |
|------|----------|------------------|
| **Live transport** | `spanda-mqtt`, `spanda-ros2`, `spanda-dds`, `spanda-ble`, `spanda-wifi` | Workspace transport crates registered via `bootstrap_providers_for_packages()`; comm-bus routes through `ProviderRegistry` |
| **Live platform** | `spanda-fleet`, `spanda-ota` | `spanda-fleet` and `spanda-ota` workspace crates; core shims for CLI |
| **Capability grant** | `spanda-nav`, `spanda-slam`, `spanda-ledger`, `spanda-cloud` | Capabilities granted when installed; nav/slam register package stubs in bootstrap |
| **Scaffold** | Remaining packages | Minimal `.sd` exports; smoke tests where present |

Spanda-language package sources remain scaffolds for most domains. Live vendor I/O is implemented in workspace crates and core compatibility shims until each package gains full `*Provider` registration. See [lean-core-roadmap.md](./lean-core-roadmap.md).

## Related docs

- [packages.md](./packages.md) — package manager
- [provider-interfaces.md](./provider-interfaces.md) — trait contracts
- [registry.md](./registry.md) — hosted registry
