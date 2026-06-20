# Spanda Language Capabilities Audit (Archived)

This document is retained for historical context.

- Original audit date: **June 2025**
- Current release target: **v0.1.0-alpha**
- Canonical status source: [`docs/feature-status.md`](./feature-status.md)

## Why this file is archived

The original audit predates major implementation progress (traits, enum payloads,
debugger, FFI bridge behavior, test infrastructure, and release engineering).
Several historical entries in the old version no longer reflected repository
reality and created confusion during release evaluation.

## Current source of truth

For all release decisions, roadmap discussions, and community communication, use:

1. [`docs/feature-status.md`](./feature-status.md) for Stable / Experimental / Planned / Deprecated status
2. [`docs/architecture.md`](./architecture.md) for up-to-date compiler/runtime diagrams
3. [`CHANGELOG.md`](../CHANGELOG.md) for release-level deltas

## v0.1.0-alpha alignment snapshot

| Area | Current status |
|------|----------------|
| Language core | Stable |
| AI safety gate | Stable |
| Hardware verification | Stable |
| Simulation runtime | Stable |
| Communication primitives | Stable |
| LSP + VS Code extension | Experimental |
| LLVM backend | Experimental |
| ROS2 production adapter | Planned |

---

If future architectural audits are needed, create a new dated file
(`language-capabilities-audit-YYYY-MM.md`) and keep `feature-status.md`
as the release-facing canonical matrix.
