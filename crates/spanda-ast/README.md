# spanda-ast

Spanda **compiler front-end types** — shared AST used by parser, type checker, interpreter, verifier, and SIR lowering.

## Modules

| Module | Contents |
|--------|----------|
| `nodes` | `Program`, `Expr`, `Stmt`, `RobotDecl`, `SpandaType`, … |
| `foundations` | `module`, `struct`, `enum`, `trait`, `extern fn`, `BridgeKind` |
| `comm_decl` | `message`, `topic`, `service`, `action`, peer robots |
| `robotics_decl` | Missions, fleet, safety zones, platform declarations |
| `regex` | `RegexPattern`, capture results |

## Consumers

- [`spanda-parser`](../spanda-parser/README.md) — builds `Program`
- [`spanda-typecheck`](../spanda-typecheck/README.md) — type checking
- [`spanda-hardware`](../spanda-hardware/README.md) — deploy verification
- [`spanda-sir`](../spanda-sir/README.md) — IR lowering
- [`spanda-interpreter`](../spanda-interpreter/README.md) — execution

`spanda_core::ast` is a thin re-export shim for backward compatibility.
