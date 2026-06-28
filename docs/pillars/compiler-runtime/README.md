# Pillar 2 — Compiler & Runtime

[← Pillars index](../README.md) · [ROADMAP § Pillar 2](../../ROADMAP.md#pillar-2--compiler--runtime)

Parse, typecheck, execute, and optionally codegen programs; load packages and dispatch providers.

## Core guides

| Topic | Guide |
|-------|--------|
| Architecture | [architecture.md](../../architecture.md) |
| Spanda architecture diagram | [spanda-architecture.md](../../spanda-architecture.md) |
| Lean core | [lean-core.md](../../lean-core.md) |
| Native deploy / LLVM | [native-deploy.md](../../native-deploy.md) · [compiler-backend-roadmap.md](../../compiler-backend-roadmap.md) |
| Runtime resolution | [how-runtime-resolution-works.md](../../how-runtime-resolution-works.md) |
| Reliability | [reliability.md](../../reliability.md) |
| Runtime faults | [runtime-fault-detection.md](../../runtime-fault-detection.md) |
| Certification gate | `spanda-certify` · [man/spanda-certify.md](../../man/spanda-certify.md) |

## Packages & providers (runtime wiring)

| Topic | Guide |
|-------|--------|
| Package loading | [how-packages-work.md](../../how-packages-work.md) |
| Provider registry | [how-providers-work.md](../../how-providers-work.md) |
| Provider interfaces | [provider-interfaces.md](../../provider-interfaces.md) |

## Crates

| Crate | Role |
|-------|------|
| `spanda-parser`, `spanda-lexer`, `spanda-ast`, `spanda-typecheck` | Frontend |
| `spanda-interpreter`, `spanda-runtime` | Execution |
| `spanda-driver`, `spanda-cli` | CLI pipeline |
| `spanda-llvm`, `spanda-wasm`, `spanda-node` | Alternate targets |

Full index: [crates/README.md](../../../crates/README.md)

## Examples

| Directory | Focus |
|-----------|--------|
| [examples/realtime/](../../../examples/realtime/) | Deadlines, watchdogs |
| [examples/features/world_model_belief.sd](../../../examples/features/world_model_belief.sd) | Fusion belief hook |
| [examples/digital_twin.sd](../../../examples/digital_twin.sd) | Twin runtime |
