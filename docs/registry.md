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

All 20 official packages under `packages/registry/` are published in the hosted index. Tarballs live at `registry/packages/<name>/<version>` with SHA-256 digests and Ed25519 signatures in `registry/index.json` (`version_checksums`, `version_signatures`). Rebuild with `./scripts/build-registry.sh` (runs `scripts/update_registry_checksums.py`, which delegates to `registry-index-maintain`). CI verifies signatures against the trust key in `registry/TRUST_KEY` (hosted signing material: `spanda-hosted-registry-v1` unless `SPANDA_REGISTRY_SIGN_KEY` is set).

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

From registry (local stub):

```bash
spanda add spanda-ros2 --version 0.1.0
spanda add spanda-openai --version 0.1.0
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

## Publishing (foundation)

```bash
spanda publish
```

Validates manifest, capabilities, hardware requirements, safety level, and license before marking the package publish-ready. Maintainers run `./scripts/build-registry.sh` and commit tarballs under `registry/packages/`.

## Version constraints

Supported semver operators: exact (`0.1.0`), caret (`^0.1.0`), comparisons (`>=0.1.0, <1.0.0`).

The lockfile pins exact resolved versions for reproducibility.
