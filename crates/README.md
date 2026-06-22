# Spanda workspace crates

Rust workspace members under `crates/`. The lean-core refactor (Phases 1–17) split the former monolithic `spanda-core` into focused libraries. **`spanda-core` remains the stable public facade** for external embedders; first-party binaries and bindings import workspace crates directly.

## Dependency rule

```
spanda-cli / spanda-node / spanda-wasm / spanda-dap / spanda-llvm
    → spanda-driver, spanda-hardware, … (no direct spanda-core dep)

spanda-core
    → full workspace graph (re-exports + thin shims)
```

Only `spanda-core` lists `spanda-core` as a dependency target. Everyone else depends on the crate that owns the behavior.

## Layer map

| Layer | Crates | Role |
|-------|--------|------|
| **Apps & bindings** | [`spanda-cli`](spanda-cli/README.md), [`spanda-node`](spanda-node/README.md), [`spanda-wasm`](spanda-wasm/README.md), [`spanda-dap`](spanda-dap/README.md) | CLI, N-API, WASM, DAP server |
| **Public facade** | [`spanda-core`](spanda-core/README.md) | Stable `spanda_core::` API — re-exports + compatibility shims |
| **Compile & run** | [`spanda-driver`](spanda-driver/README.md), [`spanda-lexer`](spanda-lexer/README.md), [`spanda-parser`](spanda-parser/README.md), [`spanda-typecheck`](spanda-typecheck/README.md), [`spanda-sir`](spanda-sir/README.md), [`spanda-error`](spanda-error/README.md) | Lex → parse → type-check → SIR → run |
| **Interpreter** | [`spanda-interpreter`](spanda-interpreter/README.md), [`spanda-runtime-host`](spanda-runtime-host/README.md) | Tree-walking runtime; `RuntimeHost` wiring |
| **Runtime kernel** | [`spanda-runtime`](spanda-runtime/README.md), [`spanda-comm`](spanda-comm/README.md), [`spanda-safety`](spanda-safety/README.md), [`spanda-hal`](spanda-hal/README.md), [`spanda-concurrency`](spanda-concurrency/README.md), [`spanda-debug`](spanda-debug/README.md), [`spanda-ai`](spanda-ai/README.md) | Scheduler, comm bus, safety, HAL, concurrency, debugger, AI registry |
| **Hardware & verify** | [`spanda-hardware`](spanda-hardware/README.md), [`spanda-certify`](spanda-certify/README.md) | `spanda verify`, certification proofs |
| **Fleet & OTA** | [`spanda-fleet`](spanda-fleet/README.md), [`spanda-ota`](spanda-ota/README.md), [`spanda-deploy-http`](spanda-deploy-http/README.md) | Fleet orchestration, rollout agents, HTTP deploy |
| **Connectivity** | [`spanda-connectivity`](spanda-connectivity/README.md), [`spanda-connectivity-runtime`](spanda-connectivity-runtime/README.md) | GPS/Wi-Fi/BLE/cellular types; runtime sim hooks |
| **Transport** | [`spanda-transport`](spanda-transport/README.md), [`spanda-transport-routing`](spanda-transport-routing/README.md), [`spanda-transport-mqtt`](spanda-transport-mqtt/README.md), [`spanda-transport-ros2`](spanda-transport-ros2/README.md), [`spanda-transport-dds`](spanda-transport-dds/README.md), [`spanda-transport-websocket`](spanda-transport-websocket/README.md) | Adapters, `RoutingCommBus`, live bridges |
| **Providers & packages** | [`spanda-providers`](spanda-providers/README.md), [`spanda-package`](spanda-package/README.md), [`spanda-modules`](spanda-modules/README.md), [`spanda-lib-registry`](spanda-lib-registry/README.md) | Official package bootstrap, `spanda.toml`, module loader |
| **Tooling** | [`spanda-format`](spanda-format/README.md), [`spanda-lint`](spanda-lint/README.md), [`spanda-codegen`](spanda-codegen/README.md), [`spanda-docs`](spanda-docs/README.md) | `fmt`, `lint`, codegen metadata, doc generators |
| **FFI & bridge** | [`spanda-bridge`](spanda-bridge/README.md), [`spanda-ffi`](spanda-ffi/README.md), [`spanda-regex-lang`](spanda-regex-lang/README.md) | Python/C++ subprocess bridges, FFI registry, regex AST |
| **Codegen (experimental)** | [`spanda-llvm`](spanda-llvm/README.md), [`spanda-rt`](spanda-rt/README.md) | SIR → LLVM IR → native binary |
| **Shared front-end** | [`spanda-ast`](spanda-ast/README.md) | AST nodes, foundations, comm declarations |
| **Security & audit** | [`spanda-security`](spanda-security/README.md), [`spanda-audit`](spanda-audit/README.md) | Capabilities, crypto, audit records |

Optional (not in default workspace build): `spanda-ros2-rclrs-native` — in-process ROS 2 cdylib.

## Related docs

- [docs/api-documentation.md](../docs/api-documentation.md) — language vs compiler vs JSON API hierarchy
- [docs/api-reference.md](../docs/api-reference.md) — generated Rust/TypeScript symbol index (grouped by layer)
- [docs/lean-core.md](../docs/lean-core.md) — principles and package-first model
- [docs/lean-core-roadmap.md](../docs/lean-core-roadmap.md) — extraction phases 1–17
- [docs/architecture.md](../docs/architecture.md) — compiler pipeline diagrams
- [docs/migration.md](../docs/migration.md) — shim removal and import paths
