# Spanda Packages

Spanda projects are organized as **packages** — self-contained units with a manifest (`spanda.toml`), source files, dependencies, and declared capabilities.

Spanda uses a **lean-core** architecture: the language core defines types, safety, and [provider interfaces](./provider-interfaces.md). Domain features (ROS2, MQTT, GPS, SLAM, vision, simulation, fleet, cloud) ship as [official packages](./official-packages.md) under `packages/registry/`.

See [How Packages Work](./how-packages-work.md), [How Providers Work](./how-providers-work.md), and [How Runtime Resolution Works](./how-runtime-resolution-works.md) for the full platform integration pipeline.

## Quick start

```bash
spanda init my_robot
cd my_robot
spanda install    # resolve dependencies → spanda.lock
spanda update     # refresh lockfile to latest compatible versions
spanda check      # type-check all sources
spanda build      # compile the project
spanda test       # run tests/
```

## Project layout

```
my_robot/
├── spanda.toml       # package manifest
├── spanda.lock       # resolved dependency lockfile
├── src/
│   └── main.sd       # primary robot program
├── tests/
│   └── smoke.sd      # optional test sources
└── README.md
```

## Creating a package

Run `spanda init` in an empty directory (or pass a name):

```bash
spanda init warehouse_robot --description "Warehouse robot controller"
```

Edit `spanda.toml` to declare dependencies, hardware targets, and capabilities. See [spanda-toml.md](./spanda-toml.md) for the full schema.

## Importing packages

Spanda supports dotted import paths:

```spanda
import navigation.path_planning;
import sensors.lidar;
import ai.openai;
import robotics.ros2;
import std.robotics;
import std.sensors;
```

Package dependencies expose import paths via the registry (see [registry.md](./registry.md)). Standard library namespaces use the `std.*` prefix:

| Namespace | Domain |
|-----------|--------|
| `std.ai` | LLM, vision models, planning |
| `std.robotics` | Motion, agents, goals |
| `std.sensors` | Camera, lidar, IMU types |
| `std.actuators` | Motors, grippers, joints |
| `std.safety` | Hazards, constraints, e-stop |
| `std.communication` | Topics, services, QoS |
| `std.hardware` | Profiles, peripherals |
| `std.sim` | Simulation world/scene |
| `std.twin` | Digital twin sync |
| `std.hri` | Speech, gestures |
| `std.units` | Physical units |
| `std.spatial` | Pose, path, trajectory |
| `std.core` | Result, Option, Error |
| `std.security` | Identity, signatures, trust |
| `std.audit` | Audit logs, provenance records |
| `std.crypto` | Hashing and signing |

See [standard-library.md](./standard-library.md) for the full namespace list.

`ai.openai` is available via the `spanda-openai` package and uses the Python
bridge (`openai_complete`) for live calls when `OPENAI_API_KEY` is set.

`ai.anthropic` (`spanda-anthropic`) and ONNX inference (`spanda-onnx`, `SPANDA_ONNX_MODEL_PATH`) follow the same bridge pattern. See [live-ai-provider.md](./live-ai-provider.md).

## Audit and blockchain packages (optional)

Blockchain is **not** part of the language core. Audit/provenance is built-in; ledger anchoring uses optional packages:

| Package | Purpose |
|---------|---------|
| `spanda-provenance` | Mission provenance helpers |
| `spanda-ledger` | Mock ledger backend (MVP) |
| `spanda-did` | Device decentralized identity |
| `spanda-supply-chain` | Hardware supply-chain traceability |

Example manifest for an audit-enabled robot app:

```toml
[package]
name = "example_robot"
version = "0.1.0"
license = "Apache-2.0"

[dependencies]
spanda-ros2 = "0.1.0"
spanda-provenance = "0.1.0"

[capabilities]
required = [
  "camera.read",
  "lidar.read",
  "network.outbound",
  "audit.write"
]

[safety]
level = "simulation_only"
```

See [audit-provenance.md](./audit-provenance.md) and [future-blockchain-support.md](./future-blockchain-support.md).

Packages declare what permissions they need in `[capabilities]`:

```toml
[capabilities]
uses = ["network.outbound", "camera.read", "lidar.read"]
required = ["motion.propose", "actuator.execute.safe"]
```

The compiler and runtime **warn** when a dependency's capabilities exceed the application's granted permissions. Grant capabilities in your robot program with `can [...]` blocks.

## Hardware compatibility

Declare hardware requirements in `[requires_hardware]`:

```toml
[requires_hardware]
memory = ">=2GB"
gpu = ">=1 TOPS"
sensors = ["Camera", "Lidar"]
```

These integrate with `spanda verify` and the built-in hardware profile registry (`RoverV1`, `JetsonOrin`, etc.). Set deployment targets in `[hardware]`:

```toml
[hardware]
targets = ["RoverV1", "JetsonOrin"]
```

## Package safety levels

Every package has a trust level in `[safety]`:

| Level | Description |
|-------|-------------|
| `experimental` | Unreviewed; default for new packages |
| `simulation_only` | May not control physical actuators |
| `hardware_safe` | Reviewed for real-hardware deployment |
| `certified` | Formally verified / certified for production |

```toml
[safety]
level = "hardware_safe"
requires_review = false
can_control_actuators = true
```

Applications can restrict which safety levels are permitted. Packages at `simulation_only` cannot set `can_control_actuators = true`.

## Dependency types

| Source | Example |
|--------|---------|
| Registry | `spanda-ros2 = "0.1.0"` |
| Local path | `my-lib = { path = "../my-lib" }` |
| Git | `spanda-nav = { git = "https://github.com/spanda/spanda-nav", branch = "main" }` |

Run `spanda install` to resolve all dependencies and write `spanda.lock`.

## Publishing

```bash
spanda publish
```

Validates manifest, capabilities, hardware requirements, safety level, and license. On success:

1. **Local mirror** — copies the bundle to `registry/packages/<name>/<version>/` in the repo (maintainers run `./scripts/build-registry.sh` to refresh the hosted index).
2. **Remote upload** — when `SPANDA_REGISTRY_URL` is set, uploads the tarball to the configured registry base.

See [registry.md](./registry.md) for the hosted index, Ed25519 signatures, and `./scripts/registry_golden_path.sh`.

## CLI reference

| Command | Description |
|---------|-------------|
| `spanda init` | Create a new package |
| `spanda install` | Resolve deps, write lockfile |
| `spanda update` | Refresh lockfile and vendored packages |
| `spanda build` | Compile all `.sd` sources |
| `spanda check` | Type-check project (or single file) |
| `spanda test` | Type-check `tests/` |
| `spanda add <pkg>` | Add a dependency |
| `spanda remove <pkg>` | Remove a dependency |
| `spanda publish` | Validate and publish package (mirrors bundle to `registry/packages/<name>/<version>`) |
| `spanda registry search <q>` | Search hosted index + local registry |

## Examples

See [examples/packages/](../examples/packages/) for:

- `basic_project/` — minimal warehouse robot
- `local_dependency/` — path-based dependency
- `robot_driver_package/` — lidar driver adapter
- `ai_provider_package/` — OpenAI provider
- `ros2_adapter_package/` — ROS 2 integration

## Community packages

Fork official scaffolds (e.g. `spanda-ledger`) for community-maintained integrations. Start from [packages/community/README.md](../packages/community/README.md) and run `./scripts/ledger_golden_path.sh` to validate the ledger scaffold.

See also [community-packages.md](./community-packages.md) for package categories, driver/adapters, and planned framework packages.
