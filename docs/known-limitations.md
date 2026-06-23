# Known limitations

Honest constraints for **v0.2.0** evaluators. For capability tiers see [feature-status.md](./feature-status.md).

## Runtime and execution

- The **tree-walking interpreter** is the primary execution path. LLVM native codegen is experimental (`spanda compile-native`, `spanda llvm-ir`).
- Simulation is **physics-lite 2D** — suitable for logic and safety testing, not high-fidelity Gazebo-class physics.
- Multi-robot **fleet** examples default to **in-process** simulation. Distributed orchestration uses HTTP fleet agents and optional mesh coordinator — not a production fleet OS.
- `spanda fleet run` on multi-robot programs may hit actuator scoping limits in the interpreter; prefer `spanda verify --health` and per-robot `spanda sim` for stable demos.

## AI and providers

- AI models use **mock backends** by default. Live OpenAI, Anthropic, and ONNX require API keys or model paths and optional feature flags (`SPANDA_LIVE_AI=1`).
- Provider packages wire through an in-process registry; there is no managed cloud inference service.

## Connectivity and IoT

- In-memory transport is the default. Live MQTT, WebSocket, DDS, Modbus, and OPC-UA require env flags and often `--features live-*` builds.
- DDS support is a **UDP JSON shim**, not OMG DDS middleware.

## Packages and registry

- `spanda publish` mirrors bundles to `registry/packages/` in-repo. Remote upload needs `SPANDA_REGISTRY_URL`.
- The hosted index lists **curated packages**; community expansion is ongoing.

## Verification and certification

- `certify ISO13849 { … }` is **verify-time metadata** — not a formal certification body sign-off.
- Capability traceability and minimum-hardware checks are **static analysis** plus runtime health hooks — not IEC 61508 tooling.

## Tooling

- **LSP** and **DAP** work with a built native CLI; VS Code extension builds in CI. **Marketplace publish** pending maintainer credentials.
- **WASM playground** covers check/run/verify — smaller surface than native CLI.

## Security

- Encryption and signed messages are implemented for wire frames and audit records. **No production HSM or PKI integration** is bundled.
- `remote_signed` kill switch requires configured signature material — verify reports errors when missing.

## Replay and twins

- Mission traces are **local files** (`--record` → `spanda replay`). No managed trace cloud.
- Digital twin live sync can upload to a URL via `SPANDA_CLOUD_UPLOAD_URL` — no twin SaaS product.

## Platform

- ROS2 adapter requires **ROS Humble** and manual setup on Linux.
- Windows support is via MSI/prebuilt CLI; some golden paths are Linux/macOS only in CI.

## Not planned (by design)

Spanda intentionally does **not** target: blockchain production adapters, cryptocurrency integrations, advanced swarm intelligence research, self-hosting compiler as default, or custom database backends as core product scope.

## Reporting issues

If behavior differs from this document, file an issue with `spanda --version`, OS, and the smallest `.sd` reproducer.
