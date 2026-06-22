# spanda-driver

**Compile and run driver** — owns the high-level pipeline API extracted from `spanda-core` in Phases 9–13.

## Responsibilities

| API | Description |
|-----|-------------|
| `compile` / `check` | Lex → parse → type-check (`spanda-lexer`, `spanda-parser`, `spanda-typecheck`) |
| `run` / `run_program` | Certify gate + FFI defaults + `spanda-interpreter` execution |
| `verify_compatibility` | Hardware/profile verification via `spanda-hardware` |
| `lower_to_sir` | AST → SIR for LLVM and tooling |
| `replay_mission` / `playback_mission` | Mission trace replay helpers |
| `run_debug` | Debugger integration with `spanda-debug` |
| `build_deploy_plan` | Deploy plan extraction (with `spanda-ota` + certify) |
| `debug_session` | Debugger machine (`DebugMachine`, step kinds) |
| `tokenize` | Lexer wrapper with `SpandaError` diagnostics |

## Who depends on this crate

- `spanda-cli`, `spanda-node`, `spanda-wasm`, `spanda-dap`, `spanda-llvm` (first-party)
- `spanda-core` (re-exports public API)

## Example

```rust
use spanda_driver::{check, run, RunOptions};

check(source)?;
let result = run(source, RunOptions::default())?;
```

## Related

- [spanda-interpreter](../spanda-interpreter/README.md) — runtime execution
- [spanda-hardware](../spanda-hardware/README.md) — `verify_compatibility`
