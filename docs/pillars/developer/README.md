# Pillar 7 — Developer Platform

[← Pillars index](../README.md) · [ROADMAP § Pillar 7](../../ROADMAP.md#pillar-7--developer-platform)

CLI, REST/gRPC APIs, SDKs, documentation, editor integration, CI/CD, and onboarding.

## Getting started

| Topic | Guide |
|-------|--------|
| First robot | [getting-started.md](../../getting-started.md) |
| Installation | [installation.md](../../installation.md) |
| Spanda 101 | [spanda-101/](../../spanda-101/README.md) |
| Spanda for Dummies | [spanda-for-dummies/](../../spanda-for-dummies/README.md) |
| Adoption path | [adoption-path.md](../../adoption-path.md) |
| Killer demo | [killer-demo.md](../../killer-demo.md) |

## CLI & APIs

| Topic | Guide |
|-------|--------|
| CLI man pages | [man/](../../man/README.md) |
| Control Center API | [control-center.md](../../control-center.md) |
| Python SDK | [packages/sdk-python/](../../../packages/sdk-python/) |
| API contract JSON | [api-contract.json](../../api-contract.json) |
| Rust/TS compiler API | [api-reference.md](../../api-reference.md) |

## CI & golden paths

| Topic | Guide |
|-------|--------|
| CI verify | [ci-verify.md](../../ci-verify.md) |
| Tier 3 golden paths | [tier-3-golden-paths.md](../../tier-3-golden-paths.md) |
| ROS2 golden path | [ros2-golden-path.md](../../ros2-golden-path.md) |
| Live AI golden path | [live-ai-provider.md](../../live-ai-provider.md) |
| Product strategy | [product-strategy.md](../../product-strategy.md) |

## Editor & web

| Topic | Guide |
|-------|--------|
| VS Code extension | [editor/vscode/](../../../editor/vscode/README.md) |
| Web playground | [packages/web/](../../../packages/web/) |
| WASM build | `scripts/build-wasm.sh` |

## Bundled demos

```bash
spanda demo rover          # flagship
spanda demo assurance      # mission assurance
spanda demo self-healing   # recovery
spanda demo continuity     # takeover
spanda demo differentiation
spanda demo adas
```

## Examples index

[examples/README.md](../../../examples/README.md) · [examples/showcase/README.md](../../../examples/showcase/README.md)

## Smoke gates

`scripts/showcase_smoke.sh` · `scripts/killer_demo_golden_path.sh` · `scripts/ci_verify_golden_path.sh` · [scripts/gates/README.md](../../../scripts/gates/README.md)
