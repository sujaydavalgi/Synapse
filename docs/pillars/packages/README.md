# Pillar 8 — Packages & Ecosystem

[← Pillars index](../README.md) · [ROADMAP § Pillar 8](../../ROADMAP.md#pillar-8--packages--ecosystem)

**Spanda Registry** and **Spanda Providers** — extensibility without bloating the core.

## Registry & workflow

| Topic | Guide |
|-------|--------|
| Package manager | [packages.md](../../packages.md) |
| Hosted registry | [registry.md](../../registry.md) |
| Official catalog | [official-packages.md](../../official-packages.md) |
| How packages work | [how-packages-work.md](../../how-packages-work.md) |
| Provider dispatch | [how-providers-work.md](../../how-providers-work.md) |
| Provider interfaces | [provider-interfaces.md](../../provider-interfaces.md) |
| Community packages | [community-packages.md](../../community-packages.md) |
| Package trust | [package-trust.md](../../package-trust.md) |

## Package sources (three paths)

| Path | Role |
|------|------|
| [packages/registry/](../../../packages/registry/) | **Authoritative** — 38 official package sources |
| [registry/](../../../registry/) | Publish mirror / hosted index tarballs |
| [crates/spanda-cli/bundled-registry/](../../../crates/spanda-cli/bundled-registry/) | CLI offline bundle snapshot |

Sync scripts: `scripts/build-registry.sh` · `scripts/sync_bundled_registry.sh`

## Protocol & domain packages (sample)

| Domain | Packages |
|--------|----------|
| ROS2 / robotics | `spanda-ros2`, `spanda-nav`, `spanda-moveit`, `spanda-slam` |
| IoT / transport | `spanda-mqtt`, `spanda-matter`, `spanda-opcua`, `spanda-ble` |
| Automotive | `spanda-canbus`, `spanda-v2x`, `spanda-radar`, `spanda-lidar` |
| Vision / AI | `spanda-opencv`, `spanda-onnx`, `spanda-openai` |
| Assurance | `spanda-assurance`, `spanda-anomaly`, `spanda-fusion`, `spanda-prognostics` |
| HRI / spatial | `spanda-openxr`, `spanda-arkit`, `spanda-voice`, `spanda-gesture` |
| Ops | `spanda-ota`, `spanda-otel-collector`, `spanda-alert-*` |

## Examples

| Directory | Focus |
|-----------|--------|
| [examples/packages/](../../../examples/packages/) | Adapter project layouts |
| [examples/packages/publish_mirror_project/](../../../examples/packages/publish_mirror_project/) | Publish workflow |
| [packages/registry/*/tests/smoke.sd](../../../packages/registry/) | Per-package smoke |

## FFI & interop

[ffi-and-ecosystem.md](../../ffi-and-ecosystem.md) · [ros2-golden-path.md](../../ros2-golden-path.md) · [live-ai-provider.md](../../live-ai-provider.md)

## Smoke gates

`scripts/registry_golden_path.sh` · `scripts/live_ai_golden_path.sh` · `scripts/mqtt_golden_path.sh` · `scripts/bundled_trust_smoke.sh` · [scripts/gates/README.md](../../../scripts/gates/README.md)
