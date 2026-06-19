# Test Coverage Plan

## Rust (`cargo test --workspace`)

| Area | Tests | Location |
|------|-------|----------|
| Lexer | unit suffixes, keywords | `lexer.rs` |
| Parser | robot, HAL, AI, foundations | `parser.rs`, `tests/foundations.rs` |
| Type checker | units, safety, capabilities | `types.rs` |
| Runtime | match, tasks, interpreter | `runtime.rs` |
| Integration | all `examples/*.sd` compile + run | `tests/integration.rs` |
| Negative | `ai_safety_violation.sd` fails | `tests/integration.rs` |

## TypeScript (`npm test`)

| Area | Status |
|------|--------|
| Legacy syntax | 67 tests passing |
| Foundations + phases 4–7 | 98 tests passing (enum, struct literal, trait impl, twin replay) |
| Golden (Rust CLI) | `tests/golden/rust.test.ts` — extend manifest |
| LSP diagnostics | `tests/lsp.test.ts` via `spanda check --json` |

## CI

`.github/workflows/ci.yml`: TypeScript tests, Rust tests, WASM + web build.

## Next tests to add

1. Match exhaustiveness warnings (enum arms)
2. Capability denial at runtime when agent exceeds `can`
3. Task interval validation (< 1ms rejection)
4. State-machine invalid transition rejection
5. Contract runtime failure (`requires` false → abort)
6. Twin mirror field validation against robot state schema
7. LSP diagnostic golden files

## Acceptance criteria per phase

Each phase merges when:

- Rust unit + integration tests pass
- New examples in `examples/` compile and run
- `docs/spanda-language.md` updated
- Golden manifest updated for stable fixtures
