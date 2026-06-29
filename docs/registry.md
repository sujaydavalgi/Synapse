# Spanda Package Registry

Spanda's package registry ships a **hosted index** in this repository (`registry/index.json`). The CLI defaults to:

`https://raw.githubusercontent.com/Davalgi/Spanda/main/registry`

Override with **`SPANDA_REGISTRY_URL`** (supports `https://` and `file://` bases). Entries merge with the local stub registry for search and `spanda install`.

## Searching packages

```bash
spanda registry search ros2
spanda registry search openai
```

## Curated packages (hosted)

All **53** curated packages under `packages/registry/` are published in the hosted index. Tarballs live at `registry/packages/<name>/<version>` with SHA-256 digests and Ed25519 signatures in `registry/index.json` (`version_checksums`, `version_signatures`). Rebuild with `./scripts/build-registry.sh` (runs `scripts/update_registry_checksums.py`, which delegates to `registry-index-maintain`). CI verifies signatures against the trust key in `registry/TRUST_KEY` (hosted signing material: `spanda-hosted-registry-v1` unless `SPANDA_REGISTRY_SIGN_KEY` is set).

| Package | Category | Import paths |
|---------|----------|--------------|
| `spanda-ble` | connectivity | `connectivity.ble` |
| `spanda-cellular` | connectivity | `connectivity.cellular` |
| `spanda-cloud` | cloud | `cloud.remote` |
| `spanda-dds` | communication | `communication.dds` |
| `spanda-fleet` | robotics | `robotics.fleet` |
| `spanda-gazebo` | simulation | `sim.gazebo` |
| `spanda-gps` | positioning | `positioning.gps` |
| `spanda-ledger` | provenance | `provenance.ledger` |
| `spanda-maintenance` | maintenance | `maintenance.health` |
| `spanda-moveit` | manipulation | `manipulation.moveit` |
| `spanda-mqtt` | communication | `communication.mqtt` |
| `spanda-nav` | navigation | `navigation.path_planning` |
| `spanda-openai` | ai | `ai.openai` |
| `spanda-opencv` | vision | `vision.opencv` |
| `spanda-ota` | deploy | `deploy.ota` |
| `spanda-ros2` | ros2 | `robotics.ros2` |
| `spanda-slam` | navigation | `navigation.slam` |
| `spanda-webots` | simulation | `sim.webots` |
| `spanda-wifi` | connectivity | `connectivity.wifi` |
| `spanda-yolo` | vision | `vision.yolo` |
| `spanda-anthropic` | ai | `ai.anthropic` |
| `spanda-onnx` | ai | `ai.onnx` |
| `spanda-iot-core` | iot | `iot.device`, `iot.telemetry`, `iot.command`, `iot.shadow` |
| `spanda-opcua` | iot | `iot.opcua` |
| `spanda-modbus` | iot | `iot.modbus` |
| `spanda-zigbee` | iot | `iot.zigbee` |
| `spanda-lora` | iot | `iot.lora` |
| `spanda-matter` | iot | `iot.matter` |
| `spanda-canbus` | iot | `iot.canbus` |
| `spanda-thread` | iot | `iot.thread` |
| `spanda-zwave` | iot | `iot.zwave` |
| `spanda-bacnet` | iot | `iot.bacnet` |
| `spanda-knx` | iot | `iot.knx` |
| `spanda-home-assistant` | bridge | `bridge.home_assistant` |
| `spanda-energy` | energy | `energy.solar` |
| `spanda-building` | building | `building.entity` |
| `spanda-smart-locks` | access | `access.lock` |
| `spanda-environment` | environment | `environment.aq` |
| `spanda-radar` | sensors | `sensors.radar` |
| `spanda-lidar` | sensors | `sensors.lidar` |
| `spanda-ultrasonic` | sensors | `sensors.ultrasonic` |
| `spanda-automotive-ethernet` | automotive | `automotive.ethernet` |
| `spanda-lin` | automotive | `automotive.lin` |
| `spanda-uds` | automotive | `automotive.uds` |
| `spanda-v2x` | automotive | `automotive.v2x` |

## Mission assurance packages (hosted)

| Package | Category | Import paths |
|---------|----------|--------------|
| `spanda-assurance` | robotics | `assurance.evidence` |
| `spanda-knowledge-model` | robotics | `assurance.knowledge` |
| `spanda-anomaly` | robotics | `assurance.anomaly` |
| `spanda-fusion` | robotics | `assurance.fusion` |
| `spanda-diagnosis` | robotics | `assurance.diagnosis` |
| `spanda-prognostics` | robotics | `assurance.prognostics` |
| `spanda-mission-planning` | robotics | `assurance.mission` |
| `spanda-mission-continuity` | robotics | `assurance.continuity` |
| `spanda-resilience` | robotics | `assurance.resilience` |

Examples: [`examples/showcase/assurance/`](../examples/showcase/assurance/README.md) · Guide: [mission-assurance.md](./mission-assurance.md)

## Local stub packages

| Package | Category | Import paths |
|---------|----------|--------------|
| `spanda-vision` | vision | `vision.core` |
| `spanda-navigation` | navigation | `navigation.path_planning` |
| `spanda-mqtt` | mqtt | `communication.mqtt` |
| `spanda-lidar-rplidar` | sensors | `sensors.lidar.rplidar` |

## Planned framework packages

Specialized adapter packages (not in the hosted index yet) are documented in [official-packages.md](./official-packages.md):

| Package | Description |
|---------|-------------|
| `spanda-nav2` | Nav2 stack adapter |
| `spanda-cartographer` | Cartographer SLAM adapter |
| `spanda-rtabmap` | RTAB-Map SLAM adapter |

## Adding dependencies

From the hosted registry (default):

```bash
spanda add spanda-ros2 --version 0.1.0
spanda add spanda-openai --version 0.1.0
spanda add spanda-onnx --version 0.1.0
spanda install
```

From a local path:

```bash
spanda add my-lib --path ../my-lib
```

From Git:

```bash
spanda add spanda-nav --git https://github.com/spanda/spanda-nav
```

## Dependency resolution

Resolution order:

1. **Local path** — reads `spanda.toml` from the path, locks exact version
2. **Git** — locks URL + branch/tag/rev (no fetch in foundation; metadata only)
3. Registry — selects highest version from hosted index (default) or local stub

Run `spanda install` after changing dependencies to regenerate `spanda.lock`.

## Golden path (CI)

```bash
./scripts/registry_golden_path.sh
```

Uses `file://` registry base, verifies Ed25519 signatures via `registry/TRUST_KEY`, installs `spanda-openai` and `spanda-ros2`, and type-checks the scratch project. CI job: `registry-golden-path`.

## Publishing

```bash
spanda publish
```

Validates manifest, capabilities, hardware requirements, safety level, and license. On success, **`spanda publish` mirrors the bundle** to `registry/packages/<name>/<version>/` in this repository. Maintainers then run `./scripts/build-registry.sh` to refresh tarballs, SHA-256 checksums, and Ed25519 signatures in `registry/index.json`.

When `SPANDA_REGISTRY_URL` is set, publish also uploads to the configured remote base.

## Version constraints

Supported semver operators: exact (`0.1.0`), caret (`^0.1.0`), comparisons (`>=0.1.0, <1.0.0`).

The lockfile pins exact resolved versions for reproducibility.

## Registry integrity environment variables

| Variable | Effect |
|----------|--------|
| `SPANDA_REGISTRY_URL` | Registry base URL (`https://` or `file://`; defaults to hosted GitHub raw index) |
| `SPANDA_REGISTRY_TRUST_KEY` | Public key material for verifying `version_signatures` |
| `SPANDA_REGISTRY_SIGN_KEY` | Private signing material for `spanda publish` / index maintenance |
| `SPANDA_REGISTRY_REQUIRE_CHECKSUM=1` | Fail install/fetch when index checksum metadata is missing |
| `SPANDA_REGISTRY_REQUIRE_SIGNATURE=1` | Fail install/fetch when signatures are missing or invalid |

For **production deployment**, set `SPANDA_REGISTRY_REQUIRE_SIGNATURE=1` and run `spanda deploy gate --policy production` so lockfile registry dependencies are audited against signed checksums in `registry/index.json`. See [deployment-gates.md](./deployment-gates.md).

## Official vs community packages

Packages under `packages/registry/` are **official** (curated catalog + hosted index). Community packages can be published to the same index but are not in the `framework_packages()` allowlist.

Only **registry-provenanced** official names receive built-in provider wiring at runtime. Path or git overrides of official names are allowed for development but do not count as official for providers or production deploy gates. See [packages.md](./packages.md) · [official-packages.md](./official-packages.md).
