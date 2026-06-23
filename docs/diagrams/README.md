# Architecture diagrams

Visual overview of the Spanda compile and runtime pipeline.

## Language pipeline

```mermaid
flowchart TD
    SD[".sd source + spanda.toml"]
    DRIVER["spanda-driver"]
    LEX["Lexer"]
    PAR["Parser"]
    AST["AST"]
    TC["Type checker\n+ units + safety + capabilities"]
    HV["Hardware verification\n+ capability / health gates"]
    CERT["spanda-certify gate"]
    RT["Interpreter +\nSimulator"]
    PKG["Provider registry\n← official packages"]
    SIR["SIR → LLVM\n(experimental)"]

    SD --> DRIVER --> LEX --> PAR --> AST --> TC
    TC --> HV
    TC --> CERT --> RT
    PKG --> RT
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
