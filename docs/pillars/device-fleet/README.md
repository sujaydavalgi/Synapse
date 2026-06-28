# Pillar 4 — Device & Fleet Platform

[← Pillars index](../README.md) · [ROADMAP § Pillar 4](../../ROADMAP.md#pillar-4--device--fleet-platform)

Device tree, registry, pool, provisioning, discovery, configuration, health, continuity, swarm, fleet, and OTA.

## Configuration & devices

| Topic | Guide |
|-------|--------|
| Device tree | [device-tree.md](../../device-tree.md) |
| Cascading TOML | [configuration.md](../../configuration.md) · [cascading-config.md](../../cascading-config.md) |
| `spanda.toml` manifest | [spanda-toml.md](../../spanda-toml.md) |
| Config validation | [config-validation.md](../../config-validation.md) |
| Device pool | [device-pool.md](../../device-pool.md) |
| Device quarantine | [device-quarantine.md](../../device-quarantine.md) |

## Health & fleet

| Topic | Guide |
|-------|--------|
| Health checks | [health-checks.md](../../health-checks.md) |
| Fleet health | [fleet-health.md](../../fleet-health.md) |
| Fleet readiness | [fleet-readiness.md](../../fleet-readiness.md) |
| Swarm health | [swarm-health.md](../../swarm-health.md) |
| Distributed fleet | [fleet-distributed.md](../../fleet-distributed.md) |
| Connectivity | [connectivity.md](../../connectivity.md) · [bluetooth.md](../../bluetooth.md) · [cellular.md](../../cellular.md) |
| Geofencing | [geofencing.md](../../geofencing.md) |

## Continuity & OTA

| Topic | Guide |
|-------|--------|
| Mission continuity | [mission-continuity.md](../../mission-continuity.md) |
| OTA (package) | [packages/registry/spanda-ota/](../../../packages/registry/spanda-ota/) |

## Human interaction (device tree)

| Topic | Guide |
|-------|--------|
| Human entity model | [human-interaction.md](../../human-interaction.md) |
| Operator capabilities | [operator-capabilities.md](../../operator-capabilities.md) |
| Human readiness | [human-readiness.md](../../human-readiness.md) |
| HRI roadmap | [human-interaction-spatial-computing-roadmap.md](../../human-interaction-spatial-computing-roadmap.md) |

## Examples

| Directory | Blueprint | Focus |
|-----------|-----------|--------|
| [examples/communication/](../../../examples/communication/) | — | Multi-robot fleet |
| [examples/connectivity/](../../../examples/connectivity/) | — | Offline, failover |
| [examples/end_to_end/warehouse_delivery/](../../../examples/end_to_end/warehouse_delivery/) | Warehouse | AMR logistics |
| [examples/solutions/adas/](../../../examples/solutions/adas/) | ADAS | Automotive device trees |
| [examples/solutions/spatial-computing/](../../../examples/solutions/spatial-computing/) | SAR, HRI | Human/wearable nodes |

## Smoke gates

`scripts/fleet_field_validation.sh` · `scripts/fleet_mesh_tamper_smoke.sh` · `scripts/ota_fleet_execute_smoke.sh` · [scripts/gates/README.md](../../../scripts/gates/README.md)
