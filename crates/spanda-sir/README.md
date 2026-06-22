# spanda-sir

**Spanda IR (SIR)** — typed intermediate representation between AST and backends (LLVM, codegen metadata).

## Contents

- `lower_program` / AST lowering helpers
- `SirProgram`, `SirStmt`, `SirBehavior`, extern and module function metadata
- Used by `spanda-driver::lower_to_sir` and [`spanda-llvm`](../spanda-llvm/README.md)

## Example

```rust
use spanda_driver::lower_to_sir;
use spanda_llvm::emit_module_ir;

let sir = lower_to_sir(source)?;
let ir = emit_module_ir(&sir);
```

## Related

- [docs/compiler-backend-roadmap.md](../../docs/compiler-backend-roadmap.md)
