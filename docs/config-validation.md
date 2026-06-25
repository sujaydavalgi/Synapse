# Configuration Validation

`spanda config validate` runs structural and safety checks on the **resolved** configuration (after all layers and fragments are merged).

## Rules

| Code | Severity | Check |
|------|----------|-------|
| `fleet.missing` | warning | No `[fleet]` section |
| `fleet.empty` | warning | Fleet has no robots |
| `robot.no_compute` | error | Robot without compute node |
| `hardware.profile_unknown` | warning | Unknown hardware profile name |
| `hardware.profile_sensor_gap` | warning | Profile expects sensor not present |
| `hardware.profile_actuator_gap` | warning | Profile expects actuator not present |
| `provider.unknown` | error | Provider not in registry or config |
| `provider.missing` | warning | Device without provider |
| `device.port_conflict` | error | Same port assigned twice |
| `device.bus_conflict` | error | Same bus assigned twice |
| `compute.duplicate_serial` | error | Duplicate compute serial number |
| `device.firmware_missing` | warning | No firmware/version metadata |
| `safety.no_emergency_stop` | error | Actuator missing `emergency_stop` |
| `security.untrusted_actuator` | error | Untrusted device controls actuator |
| `security.identity_missing` | warning | Networked device without identity |
| `mapping.gap` | error/warning | Logical-to-physical mapping issue |

## Provider validation

Providers are validated against:

1. Declared `[providers]` entries in config fragments
2. Built-in framework packages (`spanda-gps`, `spanda-lidar`, `spanda-canbus`, …)

## Package validation

Dependencies declared in merged config are collected for package loading. Full package resolution still flows through `spanda install` and the lockfile.

## Readiness integration

When running readiness with `--config`:

```bash
spanda readiness patrol.sd --config spanda.toml
```

Config validation runs first. Errors block readiness evaluation. The hardware profile from the first configured robot is used as the default `--target` when not specified.

## Health and security

- Robots without a `[health.robots.<id>]` policy appear in the config report under missing health policies.
- Remote/networked devices should have `[security.devices.<id>]` identity entries.

## Exit codes

`spanda config validate` exits non-zero when any **error**-severity finding is present. Warnings do not fail validation by default but appear in reports.
