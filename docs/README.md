# Spanda Documentation

Spanda is an AI-native autonomous systems programming language. Source files use the `.sd` extension.

## Guides

| Document | Description |
|----------|-------------|
| [../README.md](../README.md) | Project overview, philosophy, quick start, and examples |
| [getting-started.md](./getting-started.md) | **First robot in 10 minutes** |
| [architecture.md](./architecture.md) | **Compiler pipeline with diagrams** |
| [triggers.md](./triggers.md) | **Unified trigger-driven execution** (`on`, `every`, `when`, safety, state, AI) |
| [concurrency.md](./concurrency.md) | **Tasks, spawn, channels, fleet CLI, and runtime telemetry** |
| [realtime.md](./realtime.md) | **Deadline-aware tasks, jitter bounds, wall-clock scheduling** |
| [reliability.md](./reliability.md) | **Pipelines, watchdogs, recovery, retry/fallback, operating modes** |
| [watchdogs.md](./watchdogs.md) | Task heartbeats and timeout handling |
| [degraded-modes.md](./degraded-modes.md) | Operating `mode` blocks and graceful degradation |
| [replay.md](./replay.md) | **Mission trace record, deterministic replay, frame playback** |
| [regex.md](./regex.md) | **First-class regex literals, triggers, and validation rules** |
| [vision.md](./vision.md) | Long-term vision and positioning |
| [product-strategy.md](./product-strategy.md) | **Product strategy, priorities, v0.5 beta scope, killer demo** |
| [killer-demo.md](./killer-demo.md) | **Flagship demo: safety-typed AI, verify, and sim (5 min)** |
| [feature-status.md](./feature-status.md) | **v0.1.0-alpha support matrix** |
| [release-announcement-v0.1.0-alpha.md](./release-announcement-v0.1.0-alpha.md) | Announcement copy for launch channels |
| [hardware-compatibility.md](./hardware-compatibility.md) | **Hardware profiles, deploy targets, and compile-time verification** |
| [spanda-architecture.md](./spanda-architecture.md) | Architecture diagram, compiler pipeline, safety model |
| [spanda-language.md](./spanda-language.md) | Language reference for modules, traits, tasks, twins, hardware |
| [spanda-reference.md](./spanda-reference.md) | **Full language API reference** (keywords, `std.*`, builtins, man-style CLI) |
| [api-reference.md](./api-reference.md) | Rust/TypeScript compiler API index (modules, types, functions) |
| [standard-library.md](./standard-library.md) | Standard library overview and design |
| [spanda-type-system.md](./spanda-type-system.md) | Type system: units, generics, AI/safety types |
| [man/](./man/) | Man-page style CLI reference |
| [roadmap.md](./roadmap.md) | Roadmap and self-hosting plan |
| [compiler-backend-roadmap.md](./compiler-backend-roadmap.md) | **LLVM / native codegen evolution** |
| [ffi-and-ecosystem.md](./ffi-and-ecosystem.md) | **Python/C++/ROS2 interoperability strategy** |
| [migration.md](./migration.md) | Migration from legacy syntax and dual-backend notes |
| [test-plan.md](./test-plan.md) | Test coverage plan |
| [api-contract.json](./api-contract.json) | JSON schema for diagnostics, run results, and verify output |

## Repository layout

```
crates/
  spanda-core/              Lexer, parser, type checker, interpreter, triggers, concurrency, safety, AI, simulator
  spanda-cli/               Native `spanda` binary (`check`, `verify`, `run`, `sim`, `fleet`, `fmt`)
  spanda-package/           Package manager
  spanda-audit/             Audit records and backends
  spanda-security/          Capabilities, secrets, signed messages
  spanda-ros2-rclrs-native/ Native ROS 2 rclrs cdylib for in-process transport
  spanda-node/              Node.js N-API bindings
  spanda-wasm/              WebAssembly bindings for the web playground
  spanda-dap/               Debug Adapter Protocol server
  spanda-llvm/              Experimental LLVM codegen
  spanda-rt/                Runtime support for native codegen
packages/
  native/                   @spanda/native — Node wrapper for N-API
  web/                      @spanda/web — React playground
  lsp/                      @spanda/lsp — Language Server (check + verify diagnostics)
src/                        TypeScript interpreter, CLI wrapper, rust-bridge, and tests
editor/vscode/              First-party VS Code extension scaffold
scripts/                    Inline doc tooling, ROS2 daemon, Python bridge helpers
examples/                   Sample `.sd` programs (showcase/, realtime/, regex/, hardware/, communication/)
tests/                      Vitest suite and golden fixtures
```

## CLI

```bash
spanda check examples/rover.sd
spanda verify examples/hardware/rover_deploy.sd
spanda verify robot.sd --target RoverV1 --all-targets --simulate
spanda run examples/rover.sd
spanda sim examples/rover.sd --replay --record
spanda replay mission.trace --deterministic
spanda replay mission.trace --playback --from T+00:30
spanda fleet run examples/communication/multi_robot_fleet.sd
spanda fmt examples/rover.sd
spanda reference --out docs/spanda-reference.md --man-dir docs/man
```

Trace and telemetry flags for `run`, `sim`, and `fleet run`:

```bash
spanda run robot.sd --trace-scheduler --trace-tasks --trace-triggers --trace-events
spanda run robot.sd --trace-realtime --metrics-json
spanda sim robot.sd --record --trace-realtime
spanda sim robot.sd --wall-clock
```

Build the native CLI with `npm run build:rust` (output: `target/release/spanda`).

## Developer documentation

Rust (`crates/`) and TypeScript (`src/`, `packages/`) use inline API docs inside function bodies plus plain-English block comments before logic blocks. Tooling lives in `scripts/`:

- `add_inline_docs.py` — generate API doc blocks
- `add_logic_block_docs.py` — generate contextual block comments
- `normalize_inline_docs.py` — fix spacing and indentation (run after bulk edits)
- `generate_api_reference.py` — regenerate [api-reference.md](./api-reference.md) from source
- `generate_spanda_reference.py` — regenerate [spanda-reference.md](./spanda-reference.md) and [man/](./man/)

See [../CONTRIBUTING.md](../CONTRIBUTING.md#inline-documentation) for the full standard.

## Links

- GitHub: [github.com/sujaydavalgi/Spanda](https://github.com/sujaydavalgi/Spanda)
- Golden tests: [../tests/golden/manifest.json](../tests/golden/manifest.json)
