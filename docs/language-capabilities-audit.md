# Spanda Language Capabilities Audit

Principal Language Architect assessment of production-grade language infrastructure.
Audit date: June 2025. Scope: Rust core (`crates/spanda-core`), TypeScript mirror (`src/`), CLI, packages, LSP, WASM.

## Pipeline

```
.sd → lexer → parser → type checker → interpreter
```

Dual backend: Rust authoritative; TypeScript mirrors core language with CLI delegation for verify/hardware.

---

## Summary Matrix

| Capability | Status | Priority gap |
|------------|--------|--------------|
| Modules | Partially Implemented | Cross-file linking, export surface |
| Imports | Partially Implemented | User `.sd` modules, symbol injection |
| Generics | Partially Implemented | Generic functions, user generic types |
| Traits | Implemented | Agent-scoped; no trait objects |
| Enums | Implemented | No associated data variants |
| Pattern matching | Implemented | Enum/Result/Option arms |
| Error handling | Partially Implemented | Result/Option runtime (in progress) |
| Async | Implemented | Cooperative futures on module functions |
| Concurrency | Partially Implemented | Spawn queue, channels, select; not OS threads |
| Package manager | Partially Implemented | No live registry; `spanda test` executes in-language tests |
| Formatter | Implemented | AST pretty-printer with whitespace fallback |
| Linter | Implemented | `spanda lint` style/hygiene rules |
| Testing | Implemented | In-language `test "..." { }` + `spanda test` runner |
| Documentation | Implemented | Strong `docs/` coverage |
| LSP | Partially Implemented | Diagnostics, completion, hover, definition, format, symbols, rename |
| Debugger | Missing | No DAP / breakpoints |
| Serialization | Implemented | `serialize`/`deserialize` builtins (json/yaml/binary) |
| FFI | Partially Implemented | N-API embed; no `extern` in `.sd` |
| WASM | Implemented | Browser embed via `spanda-wasm` |
| Cross compilation | Partially Implemented | Hardware verify, not codegen |

---

## Feature Detail

### Modules — Partially Implemented

**Evidence:** `module navigation;` parses (`parser.rs`), stored in `Program.module_name` (`ast.rs`). Import paths validated against closed registry (`foundations.rs`, `spanda-package/import.rs`).

**Gaps:** `module_name` unused in type checker/runtime; no `export`/`public`/`private`; no cross-file symbol table; each `.sd` file type-checked independently.

**Implementation (this pass):** Top-level `export fn`, visibility modifiers, `ModuleRegistry` for project linking.

### Imports — Partially Implemented

**Evidence:** `import std.robotics;`, sensor `from` bindings, ~40+ builtin paths.

**Gaps:** Imports validate path existence only; do not inject exported symbols from user modules.

**Implementation (this pass):** Import resolves exported functions from linked module registry.

### Generics — Partially Implemented

**Evidence:** `Array<T>`, `Map<K,V>`, `Topic<T>`, etc. with arity checking (`type_system.rs`).

**Gaps:** No `fn first<T>(items: Array<T>) -> T`; no user-defined generic types.

**Implementation (this pass):** Type parameters on module-level functions; `Result<T,E>`, `Option<T>`, `Twin<T>`, `Message<T>` arity.

### Traits — Implemented

**Evidence:** `trait` / `impl Trait for Agent` with compile-time validation and runtime dispatch (`tests/trait_impl.rs`).

**Gap:** Agent-bound only; no default methods or dynamic dispatch.

### Enums — Implemented

**Evidence:** Top-level enums, qualified/unqualified variants, exhaustiveness (`tests/enum_values.rs`).

**Gap:** No tagged variants with payloads.

### Pattern Matching — Implemented

**Evidence:** `match` with exhaustiveness checking (`types.rs`, `runtime.rs`).

**Extension (this pass):** `Ok`/`Err`/`Some`/`None` arms for Result/Option scrutinees.

### Error Handling — Partially Implemented

**Evidence:** Design-by-contract (`requires`/`ensures`/`verify`); `Result`/`Option` as type names in catalog.

**Gaps:** No `Ok`/`Err`/`Some`/`None` constructors or Result algebra at runtime.

**Implementation (this pass):** Result/Option runtime values and match integration.

### Async — Implemented

**Evidence:** `async`/`await` keywords; `ModuleFnDecl.is_async`; `RuntimeValue::Future`; `AwaitExpr` in parser, type checker, and runtime.

**Semantics:** Async module calls return `Future<T>`; `await` resolves synchronously in the interpreter (cooperative, not preemptive).

### Concurrency — Partially Implemented

**Evidence:** Task scheduler (`tests/scheduler.rs`); spawn queue, channels, `select` (`concurrency.rs`, `runtime.rs`).

**Gaps:** Single-threaded cooperative model; `send`/`recv` are builtins, not keywords (select arms use `recv(ch)` syntax).

**Implementation (P1):** `channel()`, `send`, `recv`, `select { recv(ch) => ... }`, `spawn fn();`

### Package Manager — Partially Implemented

**Evidence:** `spanda-package` manifest/lockfile/resolver; CLI `init`, `build`, `test`, `install`, `publish` (stub registry).

**Gaps:** `spanda build` = type-check only; registry publish is stubbed.

**Implementation (P1):** `spanda test` executes in-language test blocks via `run_tests_with_registry`.

### Formatter — Implemented

**Evidence:** `pretty.rs` AST printer; `format_ast()` / `format_source()` in `format.rs`; CLI `spanda fmt [--json]`.

**Gap:** Complex robot blocks with HAL/SOC/comm may preserve source spans instead of full reformat.

### Linter — Implemented

**Evidence:** `lint.rs` with rules for module declaration, whitespace, line length, empty tests/behaviors, unused imports; CLI `spanda lint [--json]`.

**Gap:** No fix suggestions (`--fix`); TypeScript mirror not updated.

### Testing — Implemented

**Evidence:** `test "name" { }` top-level blocks; `assert()` builtin; `run_tests()` / CLI `spanda test` (`lib.rs`, `package.rs`).

**Gap:** TypeScript mirror not yet updated; host Vitest + `cargo test` remain for implementation tests.

### Documentation — Implemented

16+ markdown files, examples, architecture docs, API contract JSON.

### LSP — Partially Implemented

`packages/lsp`: diagnostics, completion, hover, go-to-definition, document formatting, document symbols, rename. Missing workspace symbols and code actions.

### Documentation generator — Implemented

**Evidence:** `docs.rs` + CLI `spanda doc [--json] [--out]`.

### Debugger — Missing

No DAP server or breakpoint protocol.

### Serialization — Implemented

**Evidence:** `serialize(value, format)` / `deserialize(data, format)` builtins (`serialize.rs`); formats `json`, `yaml`, `binary`.

**Gap:** TypeScript mirror lag; no schema/codegen from types.

### FFI — Partially Implemented

N-API (`spanda-node`), native bridge. No Spanda `extern fn` syntax.

### WASM — Implemented

`spanda-wasm` + web playground. Embed check/run, not robot deployment target.

### Cross Compilation — Partially Implemented

Hardware profiles and `spanda verify --target`. Validation only, no backend codegen.

---

## Architecture Assessment

Spanda's strength is **autonomous-systems semantics** (robots, agents, safety, hardware verify, comms, twins). General PL infrastructure is intentionally layered on top without replacing domain constructs.

Recommended evolution order:

1. **P0** — Module exports, generic functions, Result/Option (done)
2. **P1** — Async/await, serialization, in-language tests, spawn/channels (done)
3. **P2** — AST formatter, linter, doc generator, LSP format/symbols/rename (done)
4. **P3** — Debugger DAP, extern FFI, WASM deploy targets, cross-compile codegen

Preserve: safety contracts, hardware verification, comm architecture, audit/provenance.

---

## Deliverables Checklist

| Deliverable | Location |
|-------------|----------|
| Feature gap analysis | This document |
| Architecture assessment | Above + `docs/spanda-architecture.md` |
| Grammar updates | `lexer.rs`, `parser.rs` |
| AST updates | `ast.rs`, `foundations.rs` |
| Type system updates | `types.rs`, `type_system.rs`, `modules.rs` |
| Runtime updates | `runtime.rs`, `concurrency.rs`, `serialize.rs` |
| CLI updates | `spanda-cli` (`spanda test` execution) |
| LSP architecture | `packages/lsp/src/server.ts` (unchanged P0/P1) |
| Debugger architecture | Not yet — see roadmap |
| Documentation | This doc + `docs/spanda-language.md` |
| Example programs | `examples/modules/` |
| Test plan | `crates/spanda-core/tests/modules_result.rs`, `p1_features.rs` |

---

## Test Plan (P0)

- Module declaration + `export fn` type-check and run
- Cross-file import resolves exported symbol
- `private fn` not visible from importing module
- Generic function `first<T>(items: Array<T>) -> T`
- `Result<T,E>` / `Option<T>` construction and exhaustive match
- `cargo test`, `cargo clippy`, `cargo fmt` clean

## Test Plan (P1)

- `serialize` / `deserialize` JSON round-trip for struct values
- `test "..." { assert(...); }` blocks via `run_tests()`
- `export async fn` + `await` in behavior bodies
- `channel`, `send`, `select { recv(ch) => ... }`, `spawn`
- `crates/spanda-core/tests/p1_features.rs`
- `crates/spanda-core/tests/p2_tooling.rs`

## Test Plan (P2)

- AST formatter round-trip parse (`format_ast`)
- `spanda lint` rules (missing-module, trailing-whitespace, empty-test)
- `generate_markdown` doc output
- `cargo test`, `cargo clippy`, `cargo fmt` clean
