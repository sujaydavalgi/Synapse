# Test Coverage Plan

## Rust (`cargo test --workspace`)

| Area | Tests | Location |
|------|-------|----------|
| Lexer | unit suffixes, keywords, `%` | `lexer.rs` |
| Parser | robot, HAL, AI, foundations, hardware | `parser.rs`, `tests/foundations.rs` |
| Type checker | units, safety, capabilities | `types.rs`, `tests/type_system.rs` |
| Runtime | match, tasks, interpreter, contracts | `runtime.rs`, `tests/runtime_hardening.rs` |
| Hardware verify | sensors, timing, power, faults, matrix | `hardware.rs`, `tests/hardware_compat.rs` |
| Scheduler | multi-task multiplex | `tests/scheduler.rs` |
| Fusion | observe + fusion | `tests/fusion.rs` |
| Twin replay | mirror, replay frames | `tests/twin_replay.rs` |
| Integration | all `examples/*.sd` compile + run | `tests/integration.rs` |
| Negative | `ai_safety_violation.sd` fails | `tests/integration.rs` |

**Current count:** ~115 Rust tests (52 unit + 63 integration).

## TypeScript (`npm test`)

| Area | Status |
|------|--------|
| Lexer, parser, typechecker | Passing |
| Foundations + phases 4–7 | enum, struct literal, trait impl, twin replay |
| Runtime hardening | contracts, capabilities, verify |
| Golden (Rust CLI) | `tests/golden/rust.test.ts` |
| LSP diagnostics | `tests/lsp.test.ts` via `spanda check` + `spanda verify` |

**Current count:** 121 vitest tests.

## CLI verification

```bash
cargo test -p spanda-core --test hardware_compat
spanda verify examples/hardware/rover_deploy.sd
spanda verify examples/hardware/full_compat.sd   # expect incompatible (ESP32 in matrix)
```

## CI

`.github/workflows/ci.yml`: TypeScript tests, Rust tests, WASM + web build.

## Acceptance criteria per feature

Each feature merges when:

- Rust unit + integration tests pass
- New examples in `examples/` compile; hardware examples verify as expected
- Relevant `docs/` updated
- Golden manifest updated for stable fixtures (when applicable)

## Future tests

1. Verify JSON output schema conformance (`api-contract.json`)
2. LSP verify diagnostic golden files
3. Per-fault simulation coverage matrix
4. Cross-profile deploy matrix CI job (`--all-targets` on main examples)
