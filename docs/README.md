# Spanda Documentation

Spanda is an AI-native autonomous systems programming language. Source files use the `.sd` extension.

## Guides

| Document | Description |
|----------|-------------|
| [../README.md](../README.md) | Project overview, philosophy, quick start, and examples |
| [spanda-architecture.md](./spanda-architecture.md) | Architecture diagram, compiler pipeline, safety model |
| [spanda-language.md](./spanda-language.md) | Language reference for modules, traits, tasks, twins |
| [roadmap.md](./roadmap.md) | Roadmap and self-hosting plan |
| [migration.md](./migration.md) | Migration from legacy syntax and dual-backend notes |
| [test-plan.md](./test-plan.md) | Test coverage plan |
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
  lsp/            @spanda/lsp — Language Server (diagnostics via Rust CLI)
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

## Links

- GitHub: [github.com/sujaydavalgi/Spanda](https://github.com/sujaydavalgi/Spanda)
- Golden tests: [../tests/golden/manifest.json](../tests/golden/manifest.json)
