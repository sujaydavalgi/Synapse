# spanda-core

**Stable public facade** for the Spanda language implementation. External embedders and legacy integrations should depend on `spanda-core` and import `spanda_core::…`.

## What this crate is

After lean-core Phases 1–17, `spanda-core` is intentionally thin:

- **Re-exports** the compile/run pipeline from [`spanda-driver`](../spanda-driver/README.md)
- **Re-exports** AST, runtime, tooling, fleet, OTA, and security surfaces via `pub use` shims
- **Keeps** a small set of compatibility modules (`transport`, `transport_rclrs`, deploy/fleet shims, `providers` facade)
- **Does not** contain the interpreter body, parser, or transport adapter implementations

First-party apps (`spanda-cli`, `spanda-node`, `spanda-wasm`, `spanda-dap`, `spanda-llvm`) import workspace crates directly and do **not** depend on `spanda-core`.

## Typical imports

```rust
use spanda_core::{check, run, verify_compatibility, RunOptions, SpandaError};
use spanda_core::ast::Program;
use spanda_core::providers::ProviderRegistry;
```

Equivalent direct paths (preferred for in-repo code):

```rust
use spanda_driver::{check, run, verify_compatibility, RunOptions};
use spanda_error::SpandaError;
use spanda_ast::nodes::Program;
use spanda_providers::ProviderRegistry;
```

## Removed modules (Phase 17)

These paths no longer exist on `spanda_core`:

| Removed | Use instead |
|---------|-------------|
| `spanda_core::transport_live` | `spanda_transport_routing::transport_live` |
| `spanda_core::transport_mqtt` | `spanda_transport_mqtt` or `spanda_transport_routing::live_bridges` |
| `spanda_core::transport_dds` | `spanda_transport_dds` or `spanda_transport_routing::live_bridges` |
| `spanda_core::transport_websocket` | `spanda_transport_websocket` or `spanda_transport_routing::live_bridges` |

`spanda_core::transport` still re-exports `RoutingCommBus` and transport adapters for backward compatibility.

## Tests

Integration tests live in `tests/`. `lean_core_shims.rs` guards shim thickness and extraction boundaries.

## Related

- [Workspace crate index](../README.md)
- [lean-core-roadmap.md](../../docs/lean-core-roadmap.md)
