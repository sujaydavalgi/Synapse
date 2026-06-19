# Spanda Documentation

Spanda is an AI-native autonomous systems programming language. Source files use the `.sd` extension.

## Guides

| Document | Description |
|----------|-------------|
| [../README.md](../README.md) | Project overview, philosophy, quick start, and examples |
| [api-contract.json](./api-contract.json) | JSON schema for diagnostics and run results between Rust core and TypeScript tooling |

## Repository layout

```
crates/
  spanda-core/    Lexer, parser, type checker, interpreter, safety, AI, simulator
  spanda-cli/     Native `spanda` binary (`check`, `run`, `sim`)
  spanda-node/    Node.js N-API bindings
  spanda-wasm/    WebAssembly bindings for the web playground
packages/
  native/         @spanda/native — Node wrapper for N-API
  web/            @spanda/web — React playground
src/              TypeScript interpreter, CLI wrapper, and tests
examples/         Sample `.sd` programs
tests/            Vitest suite and golden fixtures
```

## CLI

```bash
spanda check examples/rover.sd
spanda run examples/rover.sd
spanda sim examples/rover.sd
```

Build the native CLI with `npm run build:rust` (output: `target/release/spanda`).

## Source extensions

| Extension | Status |
|-----------|--------|
| `.sd` | Primary Spanda source format |
| `.syn` | Deprecated legacy alias (accepted with a warning) |

## Links

- GitHub: [github.com/sujaydavalgi/Spanda](https://github.com/sujaydavalgi/Spanda)
- Golden tests: [../tests/golden/manifest.json](../tests/golden/manifest.json)
