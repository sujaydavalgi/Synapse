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

Runtime, verifier, readiness, assurance, health, and package loading should consume **`ResolvedSystemConfig`**, not raw TOML/JSON files.

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

## CLI

| Command | Purpose |
|---------|---------|
| `spanda config resolve` | Print merged configuration |
| `spanda config validate` | Run validation rules |
| `spanda config graph` | Show config dependency graph |
| `spanda config diff <a> <b>` | Diff two config files |
| `spanda config report` | Full configuration report |
| `spanda device-tree inspect <robot>` | Inspect one robot's hierarchy |
| `spanda device-tree graph` | Print device hierarchy |
| `spanda map verify <file.sd>` | Verify logical-to-physical mapping |
| `spanda readiness <file.sd> --config spanda.toml` | Readiness with config validation |

Add `--json` to any command for machine-readable output. Use `--config <path>` to point at a non-default manifest location.

## Integration

| Subsystem | Config access |
|-----------|---------------|
| Hardware verification | `ResolvedSystemConfig::device_tree`, hardware profiles |
| Capability verification | Device `capabilities`, provider registry |
| Readiness | `--config` loads and validates before evaluation |
| Assurance / diagnosis | `assurance`, `mission`, `recovery` sections |
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
