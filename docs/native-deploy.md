# Native deploy (LLVM)

Spanda can emit a **linked native binary** for field deployment when the CLI is built with the `llvm` feature (default). This path complements the interpreter (`spanda run`) and WASM deploy manifest (`spanda deploy --target wasm`).

## Quick start

```bash
cargo build -p spanda --release --features llvm
spanda compile-native examples/showcase/killer_demo.sd
# → target/spanda-native/spanda-program
```

Or via deploy:

```bash
spanda deploy --target native examples/showcase/killer_demo.sd
```

## Commands

| Command | Output |
|---------|--------|
| `spanda llvm-ir <file.sd>` | LLVM IR (`.ll`) for inspection |
| `spanda compile-native <file.sd>` | LLVM IR + linked binary under `target/spanda-native/` |
| `spanda deploy --target native <file.sd>` | Same as `compile-native` with deploy-oriented defaults |

### Flags

| Flag | Purpose |
|------|---------|
| `--out <path>` | Output binary path |
| `--target-triple <triple>` | Cross-compile triple (e.g. `aarch64-unknown-linux-gnu`) |
| `--hal-profile <name>` | HAL profile baked into codegen metadata |

## Requirements

- **clang** on `PATH` (links `libspanda_rt`)
- Programs must lower to SIR successfully (`spanda check` first)

Embedded cross-build example (CI: `llvm-embedded-golden-path`):

```bash
spanda compile-native --target-triple aarch64-unknown-linux-gnu \
  --hal-profile jetson examples/showcase/killer_demo.sd
```

## When to use native vs interpreter

| Runtime | Best for |
|---------|----------|
| `spanda run` / `spanda sim` | Development, triggers, agents, full language surface |
| Native binary | Fixed behaviors, edge nodes with clang toolchain, HAL-tuned builds |

Native codegen covers a **subset** of the language today. Use `spanda check` and [known-limitations.md](./known-limitations.md) before relying on native output in production.

## CI

| Job | Script |
|-----|--------|
| `llvm-golden-path` | `scripts/llvm_golden_path.sh` |
| `llvm-embedded-golden-path` | `scripts/llvm_embedded_golden_path.sh` |

## Related

- [compiler-backend-roadmap.md](./compiler-backend-roadmap.md)
- [llvm-embedded-benchmark.md](./llvm-embedded-benchmark.md)
- [hardware-compatibility.md](./hardware-compatibility.md)
