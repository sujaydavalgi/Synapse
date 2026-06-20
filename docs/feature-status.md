# Feature Status

Honest snapshot of Spanda capabilities as of **v0.1.0-alpha**. Use this document to understand what is production-ready, experimental, planned, or deprecated.

**Stubbed** = syntax or API exists without full external integration.  
**Broken** = known to fail or incomplete in current builds.

---

## v0.1.0-alpha — Officially Supported

### Supported (stable for public evaluation)

| Area | Capabilities |
|------|----------------|
| **Language core** | Lexer, parser, AST, type checker, physical units, `module`/`import`, structs/enums/traits, `match`, `Result`/`Option`, `test` blocks |
| **AI agents** | `ai_model`, `agent`, `goal`, `memory`, mock LLM/Vision providers, `ActionProposal` → `safety.validate()` → `SafeAction` |
| **Robotics primitives** | `robot`, `sensor`, `actuator`, `behavior`, `task every Nms`, state machines, events |
| **Hardware profiles** | `hardware`, `deploy`, `requires_hardware`, `requires_network`, SoC/HAL validation |
| **Compatibility verification** | `spanda verify`, `--target`, `--all-targets`, `--simulate`, `--json` |
| **Simulation** | `spanda run` / `spanda sim`, physics-lite 2D backend, lidar/arm/drone models |
| **Communication** | `message`, `topic`, `service`, `action`, `publish`/`call`/`send_goal`, in-memory transport |
| **Safety validation** | Safety zones, `max_speed`, `stop_if`, emergency stop, compile-time `SafeAction` gate |
| **Tooling** | Native CLI (`check`, `verify`, `run`, `sim`, `fmt`, `lint`, `doc`), package manager (`init`, `build`, `test`, `install`) |
| **Security / audit** | Capabilities, secrets, signed messages, audit records |
| **Digital twins** | `twin`, mirror fields, replay buffer, `twin sync` telemetry |

### Experimental (usable with caveats)

| Area | Capabilities | Caveats |
|------|--------------|---------|
| **Digital twins (live sync)** | Twin mirror + replay | External telemetry sync is simulated; no production twin cloud |
| **Replay** | `replay true`, frame buffer | In-process only |
| **Advanced verification** | Fault injection, compatibility matrix | Matrix may report stub targets |
| **Multi-agent systems** | Agent-to-agent comm examples | No distributed runtime |
| **ROS2 adapter** | Native `rclrs` cdylib, rclpy daemon, CLI bridge | Requires ROS Humble; not default transport |
| **LLVM / native codegen** | `spanda ir`, `llvm-ir`, `compile-native` | Early stage; not primary execution path |
| **FFI** | `extern python`/`extern cpp` subprocess bridges | No in-process PyO3/cxx by default |
| **LSP** | Diagnostics, completion, hover, rename | Requires built native CLI; VS Code extension scaffold available in `editor/vscode` |
| **WASM / web playground** | Browser check/run/verify | Limited surface vs native CLI |
| **Package publish** | `spanda publish`, registry search | Local/stub registry only |

### Planned (not in v0.1.0-alpha)

| Area | Description |
|------|-------------|
| **LLVM backend (production)** | Optimized native binaries as primary deploy path |
| **Self-hosting compiler** | Spanda compiler written in Spanda |
| **ROS2 production adapter** | First-class, zero-config ROS2 deployment |
| **Live AI providers** | OpenAI, local models, ONNX inference plugins |
| **VS Code extension** | Publishable extension package with LSP wiring (`editor/vscode`) |
| **In-process FFI** | PyO3 / cxx linking for Python and C++ |
| **Distributed multi-robot** | Fleet coordination runtime |

### Deprecated

| Feature | Replacement | Notes |
|---------|-------------|-------|
| Legacy inference-only AI paths | `ai_model` + `agent` | Import-based ONNX/TFLite remain for classical workflows |
| TypeScript-only verification | Native `spanda verify` | TS mirror validates deploy syntax; Rust CLI is authoritative |

---

## Feature matrix

### Language core

| Feature | Status | Notes |
|---------|--------|-------|
| Lexer / parser / AST | **Stable** | Rust authoritative; TS mirror |
| Type checker + units | **Stable** | Physical unit algebra enforced |
| modules / imports | **Stable** | `spanda install` vendor support |
| structs / enums / traits | **Stable** | Generic struct literals; enum payloads |
| generics | **Experimental** | Module fn + struct type params |
| trait objects (`dyn Trait`) | **Stable** | Rust + TS mirror |
| match / Result / Option | **Stable** | |
| async / await | **Stable** | Cooperative single-threaded |
| spawn / select / channels | **Experimental** | Cooperative concurrency |
| test blocks | **Stable** | Rust runtime + TS `runTests()` |
| `extern fn` / FFI | **Experimental** | Subprocess bridges; optional in-process |
| Spanda IR (SIR) | **Stable** | JSON export via `spanda ir` |
| Codegen / LLVM | **Experimental** | HAL profiles; conditional codegen |

### Autonomous systems

| Feature | Status | Notes |
|---------|--------|-------|
| robot / sensor / actuator | **Stable** | |
| agent / goal / task / skill | **Stable** | Mock AI backend |
| ActionProposal → SafeAction | **Stable** | Compile + runtime |
| safety zones / emergency stop | **Stable** | |
| deterministic scheduler | **Stable** | `task every Nms` |
| state machine / events | **Stable** | |
| twin / replay | **Experimental** | Replay buffer; live sync simulated |
| observe / fusion | **Stable** | |
| verify { } behavioral assertions | **Stable** | |
| hardware / deploy | **Stable** | Rust verify CLI |

### Tooling

| Feature | Status | Notes |
|---------|--------|-------|
| Native CLI (full) | **Stable** | check, verify, run, fmt, lint, doc, package |
| TypeScript CLI | **Stable** | Delegates to Rust when built |
| Formatter / linter / docgen | **Stable** | Rust |
| LSP | **Experimental** | VS Code extension scaffold exists; marketplace publish pending |
| DAP debugger | **Experimental** | Per-frame locals, step-in/out |
| N-API | **Experimental** | check, run, verify, sir, fmt |
| WASM | **Experimental** | check, run, verify, sir, fmt |

### Ecosystem / FFI

| Feature | Status | Notes |
|---------|--------|-------|
| python.* / cpp.* imports | **Experimental** | Subprocess bridges |
| ROS2 adapter | **Experimental** | Native rclrs cdylib; requires ROS Humble |
| Transport adapters | **Experimental** | In-memory + optional rclrs/rclpy |
| Package manager | **Stable** | Local registry; publish stub |
| LLVM / native codegen | **Experimental** | `compile-native` early stage |

---

## Known limitations (v0.1.0-alpha)

- AI providers use **mock backends** by default; no live API keys required or shipped.
- ROS2 integration requires manual ROS Humble setup and is not the default simulator transport.
- Native compilation via LLVM is **experimental**; the tree-walking interpreter is the primary runtime.
- Package publishing targets a **local stub registry**, not crates.io or npm-style global registry.
- VS Code extension is currently repo-hosted (`editor/vscode`), not marketplace-published yet.
- Multi-robot fleet examples run in a single process; no distributed orchestration.

---

## Broken / stubbed (honest audit)

| Item | Category | Detail |
|------|----------|--------|
| Global package registry | Stubbed | `spanda publish` writes to local `~/.spanda/registry` |
| Live OpenAI / cloud AI | Stubbed | Provider interface exists; mock only in default build |
| MQTT / DDS live transport | Stubbed | Annotations parsed; simulator logs only |
| Full native binary deploy | Stubbed | `spanda codegen` emits skeleton output |
| Blockchain / ledger cloud | Stubbed | Audit records local; see `future-blockchain-support.md` |

Nothing in the **Supported** list above is known broken in CI (`cargo test --workspace`, `npm test`).

---

## Architecture summary

```
.sd source
  → lexer → parser → AST
  → type checker (+ units, safety, capabilities)
  → [optional] hardware verifier (deploy targets)
  → interpreter + simulator
  → [optional] SIR → LLVM (experimental)
```

| Crate | Role |
|-------|------|
| `spanda-core` | Language implementation (authoritative) |
| `spanda-cli` | Native `spanda` binary |
| `spanda-package` | Package manager |
| `spanda-audit` / `spanda-security` | Audit and security |
| `spanda-llvm` / `spanda-rt` | Experimental native codegen |
| `spanda-node` / `spanda-wasm` | Bindings |
| `spanda-dap` | Debug adapter |
| `@spanda/lsp` / `@spanda/web` | LSP and web playground |

See [architecture.md](./architecture.md) for diagrams.

---

## Related docs

- [README.md](../README.md) — project overview
- [getting-started.md](./getting-started.md) — first robot in 10 minutes
- [vision.md](./vision.md) — long-term positioning
- [ffi-and-ecosystem.md](./ffi-and-ecosystem.md) — Python/C++/ROS2 interop
- [compiler-backend-roadmap.md](./compiler-backend-roadmap.md) — LLVM evolution
