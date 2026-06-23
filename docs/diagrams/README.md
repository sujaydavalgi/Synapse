# Architecture diagrams

Visual overview of the Spanda compile and runtime pipeline.

## Language pipeline

```mermaid
flowchart TD
    SD[".sd source"]
    LEX["Lexer"]
    PAR["Parser"]
    AST["AST"]
    TC["Type checker\n+ units + safety"]
    HV["Hardware verification\n(optional)"]
    RT["Interpreter +\nSimulator"]
    SIR["SIR → LLVM\n(experimental)"]

    SD --> LEX --> PAR --> AST --> TC
    TC --> HV
    HV --> RT
    TC --> RT
    TC -.-> SIR
```

## Provider architecture

```mermaid
flowchart TD
    PROG["Program"]
    REG["Provider registry"]
    PKG["Official packages\n(GPS, MQTT, …)"]
    HW["Hardware backends"]
    SIM["Simulation stubs"]

    PROG --> REG
    PKG --> REG
    REG --> HW
    REG --> SIM
```

## Package architecture

```mermaid
flowchart TD
    TOML["spanda.toml"]
    LOAD["Package loader\ninstall / lockfile"]
    PROV["Provider bootstrap"]
    RUN["Runtime dispatch"]

    TOML --> LOAD --> PROV --> RUN
```

See also [architecture.md](../architecture.md) and [lean-core.md](../lean-core.md).
