# Spanda Configuration

Spanda uses **TOML as the primary human-authored configuration format** for autonomous systems. Machine-generated configs and API interchange may use **JSON**; both formats load through the same resolver.

## Architecture

```
spanda.toml
    ↓
ConfigResolver (spanda-config)
    ↓
ResolvedSystemConfig
    ↓
Package Loader / Provider Registry
    ↓
Hardware + Capability Verification
    ↓
Readiness / Assurance / Diagnosis
    ↓
Runtime / Simulator
```

Runtime, verifier, readiness, assurance, health, recovery, and package loading consume **`ResolvedSystemConfig`** via `spanda-config` integration helpers — not raw TOML/JSON files.

### Auto-resolution

Commands that accept a `.sd` file automatically resolve config from the nearest `spanda.toml`:

- `spanda run` / `spanda sim` / `spanda fleet run` — `--config` optional; attaches config to `RunOptions`
- `spanda verify` — merges config validation into compatibility report
- `spanda readiness` — uses fleet hardware profile, readiness weights, robot alignment
- `spanda replay` — resolves config from trace source for deterministic replay and playback
- `spanda assure` / `spanda diagnose` / `spanda mission verify` / `spanda recovery-coverage` — apply `[assurance]`, `[mission]`, `[recovery]` thresholds
- `spanda heal` / `spanda recover` — validate config before evaluation

Use `--config <path/to/spanda.toml>` to point at a non-default manifest.

## Root manifest

`spanda.toml` at the project root references domain-specific fragments:

```toml
[project]
name = "Warehouse Patrol"
version = "0.1.0"
language = "0.2"

[config]
hardware = "spanda.hardware.toml"
devices = "spanda.devices.toml"
providers = "spanda.providers.toml"
fleet = "spanda.fleet.toml"
security = "spanda.security.toml"
health = "spanda.health.toml"
readiness = "spanda.readiness.toml"
assurance = "spanda.assurance.toml"
recovery = "spanda.recovery.toml"
mission = "spanda.mission.toml"
```

The existing `[package]` section for package management remains supported. When `[project]` is absent, the resolver derives project metadata from `[package]`.

## Cascading overrides

Layer environment, deployment, and robot-specific settings with `[extends]`:

```toml
[extends]
base = "configs/base.toml"
environment = "configs/warehouse-a.toml"
deployment = "configs/production.toml"
robot = "configs/rover-001.toml"
```

Later layers override earlier layers. Control array behavior per section:

```toml
[merge]
fleet = "merge_by_id"
tags = "append"
```

Strategies: `replace` (default), `append`, `merge_by_id`.

## Device identity

Declare flat `[[devices]]` records (or extend fleet `[[fleet.robots.compute.devices]]`) with network and bus identity fields:

```toml
[[devices]]
id = "camera-front-001"
type = "Camera"
logical_name = "front_camera"
ip = "192.168.1.42"
mac = "AA:BB:CC:DD:EE:FF"
serial = "CAM-12345"
provider = "spanda-vision"
protocol = "rtsp"
endpoint = "rtsp://192.168.1.42/stream"
capabilities = ["capture_image", "stream_video"]
trust_level = "verified"
security_identity = "camera-front-001"
robot_id = "rover-001"
```

Supported identity fields include: `logical_name`, `serial`, `mac`/`mac_address`, `ip`/`ip_address`, `hostname`, `dns_name`, `mdns_name`, `endpoint`/`endpoint_url`, `protocol`, `port`, `bus`, `can_id`, `usb_path`, `pci_path`, `bluetooth_address`, `ble_uuid`, `cellular_imei`, `sim_iccid`, `gps_device_id`, `firmware_version`, `hardware_revision`, `security_identity`, `certificate_fingerprint`, `trust_level`, `redundant_group`, `failover_priority`.

Reference fragments via `[config] network_devices = "spanda.network-devices.toml"` (merged into `[[devices]]` with `merge_by_id`).

## CLI

| Command | Purpose |
|---------|---------|
| `spanda config resolve` | Print merged configuration |
| `spanda config validate` | Run validation rules |
| `spanda config graph` | Show config dependency graph |
| `spanda config diff <a> <b>` | Diff two config files |
| `spanda config drift --baseline <dir>` | Semantic drift vs approved baseline |
| `spanda drift <file.sd> [--agent Robot@HW]` | Program + agent drift vs deploy/fleet agents |
| `spanda config report` | Full configuration report |
| `spanda config report --network` | Network/device identity report only |
| `spanda device discover` | List configured devices; optional `--subnet` scan |
| `spanda device inspect <id>` | Inspect one device identity record |
| `spanda device-tree inspect <robot>` | Inspect one robot's hierarchy |
| `spanda device-tree graph` | Print device hierarchy |
| `spanda network scan --subnet CIDR` | TCP probe hosts on a subnet |
| `spanda map verify <file.sd>` | Verify logical-to-physical mapping |
| `spanda readiness <file.sd> --config spanda.toml` | Readiness with config validation |
| `spanda readiness <file.sd> --baseline <dir>` | Readiness with baseline drift checks |

Add `--json` to any command for machine-readable output. Use `--config <path>` to point at a non-default manifest location.

## Integration

| Subsystem | Config access |
|-----------|---------------|
| Hardware verification | `ResolvedSystemConfig::device_tree`, hardware profiles |
| Capability verification | Device `capabilities`, provider registry |
| Readiness | `--config` loads and validates before evaluation |
| Device registry | `[[devices]]` identity records merged into `DeviceRegistry` |
| Assurance / diagnosis | `assurance`, `mission`, `recovery` sections + traceability rows |
| Health framework | `health` section and per-robot policies |
| Provider registry | `providers` fragment + package dependencies |
| Security | `security.devices.*` identities and trust flags |

## Reports

`spanda config report` generates:

- Resolved configuration summary
- Device hierarchy
- Logical-to-physical mapping counts
- Capability mapping per device
- Health policy coverage
- Trust/security identity mapping

## See also

- [spanda.toml reference](spanda-toml.md)
- [Device tree](device-tree.md)
- [Cascading config](cascading-config.md)
- [Config validation](config-validation.md)
