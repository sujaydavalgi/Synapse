# Spanda Architecture

Spanda is an **AI-native autonomous systems programming language**. The implementation uses a dual-layer architecture: a canonical **Rust core** and a **TypeScript mirror** for developer tooling and tests.

## System diagram

```mermaid
flowchart TB
  subgraph sources ["Source (.sd)"]
    MOD[module / import]
    TYPES[struct / enum / trait]
    ROBOT[robot / agent / task]
  end

  subgraph rust ["Rust Core (spanda-core)"]
    LEX[lexer.rs]
    PAR[parser.rs]
    AST[ast.rs + foundations.rs]
    TC[types.rs]
    RT[runtime.rs]
    SAF[safety.rs]
    AI[ai.rs]
    HAL[hal.rs + soc.rs + lib_registry.rs]
    SM[state machines + tasks]
    SIM[simulator.rs]
  end

  subgraph bindings ["Bindings"]
    CLI[spanda-cli]
    NODE[spanda-node]
    WASM[spanda-wasm]
  end

  subgraph ux ["Developer UX"]
    TS[src/ TypeScript mirror]
    WEB[packages/web playground]
    LSP[LSP — planned]
  end

  sources --> LEX --> PAR --> AST --> TC --> RT
  RT --> SAF & AI & HAL & SIM
  RT --> CLI & NODE & WASM --> WEB
  TC -.->|mirror in progress| TS
```

## Language layers

| Layer | Purpose | Status |
|-------|---------|--------|
| **Foundations** | `module`, `struct`, `enum`, `trait`, `match` | Implemented (Rust) |
| **Autonomous primitives** | `robot`, `sensor`, `actuator`, `agent`, `skill`, `goal`, `memory` | Implemented |
| **Scheduling** | `task every Nms`, `behavior`, contracts (`requires` / `ensures`) | Implemented (Rust) |
| **State machines** | `state_machine`, `state`, `transition` | Parsed, validated, logged |
| **Capabilities** | `can [ read(lidar), propose_motion ]` | Type-checked |
| **Events** | `event`, `on Event { }` | Parsed + runtime handlers |
| **Digital twins** | `twin { mirror pose; replay true; }` | Parsed + validated |
| **Safety** | `ActionProposal` → `safety.validate` → `SafeAction` | Enforced at compile + run time |
| **ROS2 surface** | `node`, `topic`, `service`, `action` | Implemented |

## Compiler pipeline

1. **Lex** — tokenize keywords, units, `->`, `=>`
2. **Parse** — build AST (`Program`, `RobotDecl`, foundations)
3. **Type-check** — units, capabilities, state machines, AI safety, SoC/HAL
4. **Run** — interpreter + simulator; tasks scheduled deterministically

## Safety model

AI outputs are **untrusted**. The only allowed motion path:

```spanda
let proposal = planner.reason(...);
let action = safety.validate(proposal);
wheels.execute(action);
```

Direct `planner.drive(...)` or `wheels.execute(proposal)` is rejected by the type checker.

## Self-hosting roadmap

See [roadmap.md](./roadmap.md). Phase 0–2 are in progress in Rust; TypeScript mirror and LSP follow.
