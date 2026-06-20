# Compiler Backend Roadmap

This document describes how the Spanda compiler evolves from the current tree-walking interpreter toward native binaries via LLVM.

## Current pipeline (implemented)

```
Spanda source (.sd)
  → lexer
  → parser
  → AST
  → semantic / type checker
  → interpreter / simulator / hardware verifier
```

The **authoritative implementation** lives in Rust (`crates/spanda-core`). A TypeScript mirror (`src/`) supports tests, the web playground, and CLI delegation when the native binary is built.

Supporting outputs today:

| Output | Command | Status |
|--------|---------|--------|
| Diagnostics | `spanda check` | Implemented |
| Compatibility report | `spanda verify` | Implemented |
| Interpreted run | `spanda run` / `spanda sim` | Implemented |
| Formatted source | `spanda fmt` | Implemented |
| Markdown docs | `spanda doc` | Implemented |
| Skeleton codegen | `spanda codegen --target native\|wasm\|esp32` | Stubbed (SIR-aware template C/WASM/ESP32, not a real compiler) |
| WASM manifest | `spanda deploy --target wasm` | Partially implemented |

## Future pipeline (planned)

```
Spanda source (.sd)
  → Spanda IR (SIR)
  → LLVM IR
  → native binary / WASM module
```

### Milestone 1 — Spanda IR (SIR) ✓ (extended)

- Lower typed AST to SIR preserving module functions, extern bridge kinds, imports, behavior metadata (requires/ensures/invariant flags, task names), and robot names.
- **`spanda ir [--json] file.sd`** emits SIR for codegen planning and CI inspection.
- SIR is the contract between frontend and backends; the interpreter still executes AST directly.

### Milestone 2 — LLVM backend ✓ (extended)

- **`spanda llvm-ir file.sd`** emits LLVM IR from SIR with `libspanda_rt` declarations and calls for supported statements (actuator drive/stop, publish with string payloads, loop every, emergency stop, integer returns).
- **`spanda compile-native [--out <binary>] file.sd`** links LLVM IR with `libspanda_rt` via clang when available.
- `crates/spanda-rt` exposes the C ABI (`spanda_rt_drive`, `spanda_rt_stop`, `spanda_rt_publish`, `spanda_rt_loop_delay_ms`, …).
- `crates/spanda-llvm` lowers SIR statement bodies where supported; loops, match, and comm remain planned.
- Robot scheduler, safety monitor, and comm routing remain in **`libspanda_rt`** (interpreter-backed for now).
- `-O2` builds for deployment; `-O0` + debug info for DAP debugging.

### Milestone 3 — Cross-compilation

- Target triples and HAL profiles drive conditional compilation (Jetson CUDA vs ESP32 bare-metal).

## Target platforms

| Platform | Architecture | Priority | Notes |
|----------|--------------|----------|-------|
| Linux x86_64 | amd64 | P1 | Dev machines, CI, cloud agents |
| macOS | aarch64 / x86_64 | P1 | Developer workstations |
| Windows | x86_64 | P2 | Simulation and tooling |
| Linux ARM64 | aarch64 | P1 | Raspberry Pi 5, custom SBCs |
| NVIDIA Jetson | aarch64 + CUDA | P1 | Vision / AI edge workloads |
| Raspberry Pi | aarch64 | P2 | Low-cost robotics |
| ESP32 | xtensa / riscv | P3 | Microcontroller targets via existing codegen stub |
| RISC-V | rv64gc | P3 | Open hardware platforms |
| WASM | wasm32 | P2 | Browser playground, serverless agents |

## What stays interpreted (for now)

These subsystems depend on dynamic robot graphs and are expensive to compile prematurely:

- Full hardware compatibility matrix (`spanda verify --all-targets`)
- Digital twin replay buffers
- Mock AI providers and agent planning loops
- Transport adapter discovery (ROS2/MQTT/DDS stubs)

They will remain in `libspanda_rt` even after LLVM codegen ships.

## Self-hosting path

See [roadmap.md](./roadmap.md). Bootstrap order:

1. Rust implements full language (**current**)
2. Spec stabilization — grammar + [api-contract.json](./api-contract.json)
3. Spanda subset rewritten in Spanda (lexer/parser for minimal `.sd`)
4. Incremental migration — type checker, then SIR, then LLVM
5. Rust optional for embedded/WASM targets

## Debugging

| Stage | Tool | Status |
|-------|------|--------|
| Interpreter breakpoints | `spanda debug --break N` | Partially implemented |
| DAP server | `crates/spanda-dap` | Partially implemented |
| LLVM debug symbols | — | Planned |

## Related documents

- [ffi-and-ecosystem.md](./ffi-and-ecosystem.md) — foreign library linking strategy
- [spanda-architecture.md](./spanda-architecture.md) — current layer diagram
- [migration.md](./migration.md) — dual-backend (Rust + TypeScript) notes
