# Spanda Documentation

Spanda is an AI-native autonomous systems programming language. Source files use the `.sd` extension.

## Guides

| Document | Description |
|----------|-------------|
| [../README.md](../README.md) | Project overview, philosophy, quick start, and examples |
| [hardware-compatibility.md](./hardware-compatibility.md) | **Hardware profiles, deploy targets, and compile-time verification** |
| [spanda-architecture.md](./spanda-architecture.md) | Architecture diagram, compiler pipeline, safety model |
| [spanda-language.md](./spanda-language.md) | Language reference for modules, traits, tasks, twins, hardware |
| [spanda-type-system.md](./spanda-type-system.md) | Type system: units, generics, AI/safety types |
| [roadmap.md](./roadmap.md) | Roadmap and self-hosting plan |
| [migration.md](./migration.md) | Migration from legacy syntax and dual-backend notes |
| [test-plan.md](./test-plan.md) | Test coverage plan |
| [api-contract.json](./api-contract.json) | JSON schema for diagnostics, run results, and verify output |

## Repository layout

```
crates/
  spanda-core/    Lexer, parser, type checker, interpreter, safety, AI, simulator, hardware verifier
  spanda-cli/     Native `spanda` binary (`check`, `verify`, `run`, `sim`, `fmt`)
  spanda-node/    Node.js N-API bindings
  spanda-wasm/    WebAssembly bindings for the web playground
packages/
  native/         @spanda/native — Node wrapper for N-API
  web/            @spanda/web — React playground
  lsp/            @spanda/lsp — Language Server (check + verify diagnostics)
src/              TypeScript interpreter, CLI wrapper, rust-bridge, and tests
examples/         Sample `.sd` programs (including examples/hardware/)
tests/            Vitest suite and golden fixtures
```

## CLI

```bash
spanda check examples/rover.sd
spanda verify examples/hardware/rover_deploy.sd
spanda verify robot.sd --target RoverV1 --all-targets --simulate
spanda run examples/rover.sd
spanda sim examples/rover.sd
spanda fmt examples/rover.sd
```

Build the native CLI with `npm run build:rust` (output: `target/release/spanda`).

## Links

- GitHub: [github.com/sujaydavalgi/Spanda](https://github.com/sujaydavalgi/Spanda)
- Golden tests: [../tests/golden/manifest.json](../tests/golden/manifest.json)
