# Spanda

**The Autonomous Systems Language.** *The pulse of autonomous intelligence.*

Spanda is an AI-native programming language for robotics, autonomous agents, digital twins, and edge systems. Source files use the **`.sd`** extension.

Repository: [github.com/Davalgi/Spanda](https://github.com/Davalgi/Spanda)

---

## Philosophy

Hardware is the body.  
Sensors are the senses.  
AI models are the mind.  
Actuators are the muscles.  
Spanda is the intelligent pulse that transforms perception into action.

**Spanda** is a Sanskrit term meaning *the divine pulse* or *sacred tremor* or *divine vibration*, representing the creative pulsation of absolute consciousness and energy, manifesting as waves of expansion and contraction. It is the universal activity or first stir of awareness that creates and sustains the entire universe.

---

## What is Spanda?

Spanda is a language and runtime for **autonomous systems** — programs where sensors, AI models, actuators, safety rules, and deployment targets are first-class concepts in the source code.

You write a `robot` block with sensors, actuators, safety zones, and agents. The compiler enforces physical units, validates AI proposals before they reach hardware, and checks that your program fits the deployment target before you ship.

```spanda
robot Rover {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  safety {
    max_speed = 1.0 m/s;
    stop_if lidar.nearest_distance < 0.5 m;
  }

  behavior patrol() {
    loop every 100ms {
      wheels.drive(linear: 0.3 m/s, angular: 0.0 rad/s);
    }
  }
}
```

---

## Why Spanda exists

Building autonomous systems today means stitching together Python scripts, C++ drivers, ROS2 nodes, safety monitors, and deployment checklists — with no single language that treats **AI output as untrusted**, **hardware fit as compile-time**, and **safety as mandatory**.

Spanda exists to be that coordination layer: one typed language where perception, planning, safety validation, simulation, and deployment verification live together.

---

## Key differentiators

| Differentiator | What it means |
|----------------|---------------|
| **AI safety gate** | `ActionProposal` from LLMs/vision cannot drive actuators; only `SafeAction` from `safety.validate()` can |
| **Hardware verification** | `deploy Robot to Profile` + `spanda verify` checks sensors, memory, timing, and power before deploy |
| **Physical units** | `1.0 m/s`, `0.5 rad`, `100 ms` — unit algebra enforced at compile time |
| **Robot-native syntax** | Sensors, actuators, topics, services, actions, safety zones, and tasks are language keywords |
| **Deterministic scheduling** | `task every 50ms` with optional resource `budget { }` |
| **Real-time contracts** | `deadline`, `jitter <=`, `priority`, `critical isolated` tasks; latency `pipeline` budgets |
| **Reliability primitives** | Watchdogs, operating `mode` blocks, `recover from`, retry/fallback on faults |
| **Mission trace replay** | `spanda sim --record`, `spanda replay --deterministic` / `--playback` for regression and incident review |
| **First-class regex** | Literals, `Regex` type, string methods, trigger/subscribe filters, `validate` rules |
| **Trigger-driven execution** | Unified `on` / `every` / `when` / `while` handlers for events, topics, safety, state, and AI |
| **Cooperative concurrency** | `spawn`, `join`, `parallel`, channels, and `select` with scheduler telemetry |
| **Simulation built in** | `spanda run` / `spanda sim` — test without hardware |
| **Digital twins** | `twin { mirror pose; replay true; }` for shadow state and replay |
| **Platform packages** | `spanda install` / `update`, provider dispatch, `--trace-providers` | Official registry packages wire to runtime providers |
| **World models** | `world_model { enabled; }` + `fusion.read()` belief hook | Observe → fused observation → belief-gated decisions |
| **Verification & DX** | `spanda verify --health`, traceability matrices, kill switch, health policies | Capability exposure, fleet `require` clauses, typed handler I/O |
| **Live providers (optional)** | OpenAI, Anthropic, ONNX via Python bridge; IoT live bridges | Mock fallback when keys or env flags are unset |
| **Package registry** | Hosted index + `spanda publish` mirror to `registry/packages/` | Ed25519-signed tarballs; override with `SPANDA_REGISTRY_URL` |

Lean-core status: Phases 1–35 complete — [docs/lean-core-roadmap.md](docs/lean-core-roadmap.md)

---

## Trust & feature status

Honest snapshot for evaluators ([full matrix](docs/feature-status.md)):

| Feature | Status |
|---------|--------|
| Parser | **Stable** |
| Type checker | **Stable** |
| Safety validation (`ActionProposal` → `SafeAction`) | **Stable** |
| Hardware verification (`spanda verify`) | **Stable** |
| Simulation (`spanda run` / `spanda sim`) | **Stable** |
| Mission replay (`--record`, `spanda replay`) | **Stable** |
| Package loading (`spanda install`, registry) | **Stable** |
| Connectivity (in-memory + optional live bridges) | **Stable** / live **Experimental** |
| Encryption & secure comm | **Stable** (wire frames); live TLS **Experimental** |
| Health framework | **Stable** |
| Fleet runtime (in-process + HTTP agents) | **Stable** / distributed **Experimental** |
| Debugger (DAP) | **Experimental** |
| LLVM backend | **Experimental** |
| LSP / VS Code extension | **Experimental** |
| Live AI providers (OpenAI, Anthropic, ONNX) | **Experimental** |
| ROS2 adapter | **Experimental** |

Limitations: [docs/known-limitations.md](docs/known-limitations.md)

---

## Quick start (5 minutes)

```bash
# Install (from clone)
./scripts/install.sh
# Or: cargo install --path crates/spanda-cli --locked

# Run flagship demo
spanda demo rover

# Or step by step:
spanda check examples/showcase/killer_demo.sd      # type-check
spanda verify examples/showcase/hardware_compatibility.sd  # hardware fit
spanda sim examples/showcase/killer_demo.sd        # simulate
```

Video script: [docs/demo-script.md](docs/demo-script.md) · Architecture: [docs/diagrams/](docs/diagrams/)

---

## Architecture overview

Spanda uses a **lean-core, package-first** workspace (Phases 1–17 complete). `spanda-core` is the stable public facade; first-party apps import focused workspace crates directly.

```
.sd source → lexer → parser → AST → type checker
                                      ↓
                            hardware verifier (optional)
                                      ↓
                            interpreter + simulator
                                      ↓
                     optional packages (ROS2, MQTT, GPS, …)
                                      ↓
                            SIR → LLVM (experimental)
```

| Layer | Crates | Responsibility |
|-------|--------|----------------|
| **Public facade** | `spanda-core` | Stable `spanda_core::` re-exports + thin shims |
| **Apps** | `spanda-cli`, `spanda-node`, `spanda-wasm`, `spanda-dap` | CLI, bindings, debugger — direct workspace deps |
| **Pipeline** | `spanda-driver`, `spanda-lexer`, `spanda-parser`, `spanda-typecheck`, `spanda-sir` | Compile, check, verify, SIR |
| **Runtime** | `spanda-interpreter`, `spanda-runtime`, `spanda-comm`, `spanda-safety`, … | Execution, scheduling, safety, comm |
| **Transport** | `spanda-transport-routing`, `spanda-transport-*` | Adapters, live bridges, `RoutingCommBus` |
| **Domain** | `spanda-hardware`, `spanda-fleet`, `spanda-ota`, `spanda-certify` | Verify, fleet, rollout, certification |
| **Tooling** | `spanda-format`, `spanda-lint`, `spanda-codegen`, `spanda-docs` | fmt, lint, codegen, docgen |
| **Packages** | `spanda-package`, `spanda-providers` | `spanda.toml`, registry, provider bootstrap |
| **Official packages** | `packages/registry/*` | ROS2, MQTT, GPS, SLAM, vision, fleet, OTA, cloud |
| **Mirror & UX** | `src/`, `packages/lsp`, `packages/web` | TypeScript tests, LSP, web playground |

Crate index: [crates/README.md](crates/README.md) · Deep dive: [docs/lean-core.md](docs/lean-core.md) · [docs/architecture.md](docs/architecture.md)

---

## Example code

### AI agent with safety validation

```spanda
robot Rover {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;

  ai_model planner: LLM { provider: "mock"; model: "safe-planner"; }

  safety { max_speed = 1.0 m/s; }

  agent Navigator {
    uses planner;
    tools [lidar, wheels];
    goal "Navigate safely";

    plan {
      let proposal = planner.reason(prompt: "Plan path", input: lidar.read());
      let action = safety.validate(proposal);
      wheels.execute(action);
    }
  }
}
```

### Hardware deploy verification

```spanda
hardware RoverV1 {
  memory: 4 GB;
  sensors [ Camera, Lidar ];
  actuators [ DifferentialDrive ];
}

robot RoverProgram {
  sensor lidar: Lidar on "/scan";
  actuator wheels: DifferentialDrive;
}

deploy RoverProgram to RoverV1;
```

```bash
spanda verify rover.sd --json
```

### Learn Spanda

**[Tutorials index](docs/tutorials/README.md)** — all learning paths in one place.

| Track | Guide | Time |
|-------|-------|------|
| Plain English | [Spanda for Dummies](docs/spanda-for-dummies/README.md) | ~45 min |
| Hands-on course | [Spanda 101](docs/spanda-101/README.md) | ~3 hours |
| Quickstart | [Getting started](docs/getting-started.md) | ~10 min |

### Examples library

**[examples/README.md](examples/README.md)** — master index: killer demo, learning ladder, topics, packages, CI.

Start with the progressive ladder in [`examples/basics/`](examples/basics/README.md), then integration slices and end-to-end packages:

| Tier | Path | Highlights |
|------|------|------------|
| Basics | `examples/basics/01_minimal_robot.sd` → `11_observe_and_fusion.sd` | Language core from robot blocks to fusion |
| Features | [`examples/features/`](examples/features/README.md) | One file per capability — full coverage index |
| Integration | `examples/integration/` | Triggers, concurrency, verify walkthrough |
| End-to-end | [`examples/end_to_end/`](examples/end_to_end/README.md) | Patrol, warehouse, fleet, replay, real-time workflows |

### Flagship examples (start here)

Three pillars for evaluators — full library has 70+ files; start with these:

| Pillar | Purpose | Command |
|--------|---------|---------|
| **Safety** | Block unsafe AI at compile time | `spanda check examples/showcase/ai_safety_violation.sd` |
| **Verify** | Hardware fit before deploy | `spanda verify examples/showcase/hardware_compatibility.sd --json` |
| **Sim** | Patrol without hardware | `spanda sim examples/showcase/killer_demo.sd` |
| **Platform** | Packages → providers → replay | `cd examples/showcase/autonomous_rover && spanda install && spanda run src/rover.sd --trace-providers` |

5-minute walkthrough: [`docs/killer-demo.md`](docs/killer-demo.md) · Platform demo: [`examples/showcase/autonomous_rover/README.md`](examples/showcase/autonomous_rover/README.md) · Tier 3 CI golden paths: [`docs/tier-3-golden-paths.md`](docs/tier-3-golden-paths.md)

More showcase demos: [`examples/showcase/README.md`](examples/showcase/README.md). Real-time: [`examples/realtime/`](examples/realtime/); regex: [`examples/regex/`](examples/regex/).

---

## Installation

### Quick install (from source)

```bash
git clone https://github.com/Davalgi/Spanda.git
cd Spanda
./scripts/install.sh
spanda demo rover
```

Equivalent: `cargo install --path crates/spanda-cli --locked` (installs the `spanda` binary).

### Prebuilt packages (Linux, macOS, Windows)

Download installable packages from [GitHub Releases](https://github.com/Davalgi/Spanda/releases):

```bash
# Linux / macOS
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/Davalgi/Spanda/releases/download/v0.1.0/spanda-cli-installer.sh | sh
```

Windows: use the `.msi` installer or PowerShell script from the same release page.

Full install guide: [docs/installation.md](docs/installation.md)

### Build from source

#### Prerequisites

- **Node.js** 18+ (for TypeScript tooling and tests)
- **Rust** stable (for native CLI and authoritative runtime)
- **npm** (workspace manager)

#### Clone and build

```bash
git clone https://github.com/Davalgi/Spanda.git
cd Spanda
npm install
npm run build:rust    # builds target/release/spanda
npm run build         # TypeScript mirror (tsc) — must pass in CI
npm test
```

The native CLI is at `target/release/spanda`. Add it to your `PATH` or use `npm run spanda:native -- <command>`.

### Web playground (optional)

```bash
npm run build:wasm
npm run web:dev       # http://localhost:5173
```

---

## CLI commands

| Command | Description |
|---------|-------------|
| `spanda init [name]` | Create a new Spanda project |
| `spanda check <file.sd>` | Type-check |
| `spanda verify <file.sd>` | Hardware compatibility verification |
| `spanda run <file.sd>` | Run with simulated backend |
| `spanda sim <file.sd>` | Run simulation with detailed output |
| `spanda fleet run <file.sd>` | Run multi-robot fleet simulation (in-process) |
| `spanda replay <mission.trace>` | Inspect, verify, or play back a recorded mission trace |
| `spanda demo <rover\|safety\|verify\|fleet\|health>` | One-command showcase demos |
| `spanda test` | Run project tests |
| `spanda fmt <file.sd>` | Format source |
| `spanda lint <file.sd>` | Lint source |
| `spanda doc <file.sd>` | Generate markdown documentation |
| `spanda build` | Build project |
| `spanda install` | Install dependencies |
| `spanda update` | Refresh lockfile and vendored packages |
| `spanda twin export <file.sd> --out <replay.json>` | Export twin replay buffer as JSON |

Verify flags: `--target <Profile>`, `--all-targets`, `--simulate`, `--json`

Run/sim/fleet trace flags: `--trace-scheduler`, `--trace-tasks`, `--trace-triggers`, `--trace-events`, `--trace-providers`, `--trace-realtime`, `--metrics-json`, `--record`, `--wall-clock`, `--replay` (sim)

Replay flags: `--from T+mm:ss`, `--deterministic` (re-run source and verify frame parity), `--playback` (apply recorded state snapshots)

Quick start guide: [docs/getting-started.md](docs/getting-started.md) · Real-time & replay: [docs/realtime.md](docs/realtime.md), [docs/replay.md](docs/replay.md)

---

## Hardware verification

Spanda checks that autonomous programs **fit the deployment target** — sensors, actuators, memory, GPU, timing, network, power, and AI model requirements.

```bash
spanda verify examples/showcase/hardware_compatibility.sd
spanda verify rover.sd --target RoverV1 --all-targets
spanda verify rover.sd --simulate --json
```

Built-in profiles: `RoverV1`, `RoverV2`, `JetsonOrin`, `RaspberryPi5`, `ESP32`.

Full reference: [docs/hardware-compatibility.md](docs/hardware-compatibility.md)

---

## Safety model

Safety rules in the `safety { }` block are evaluated **before every motion command**:

1. **`max_speed = X m/s`** — clamps drive velocity
2. **`zone`** — circular or rectangular keep-out regions
3. **`stop_if <condition>`** — triggers emergency stop when true
4. **`ActionProposal` → `SafeAction`** — AI outputs cannot reach actuators without `safety.validate()`

Invalid (compile error):

```spanda
wheels.execute(proposal);  // requires SafeAction, not ActionProposal
```

Valid:

```spanda
let action = safety.validate(proposal);
wheels.execute(action);
```

---

## Package ecosystem

Spanda includes a package manager for modular robot programs:

```bash
spanda init my_robot
spanda add local_dependency
spanda build
spanda test
```

Manifest format: [docs/spanda-toml.md](docs/spanda-toml.md)  
Package guide: [docs/packages.md](docs/packages.md)

---

## Roadmap

**v0.2.0 (current):** Stable interpreter, safety gate, hardware verify, simulation, package manager, showcase demos, `spanda demo`, verification & DX (Phases 27–35), docs site on GitHub Pages.

**Next (v0.3):** VS Code Marketplace, expanded registry, IDE polish. **v0.4:** LLVM deploy path, distributed fleet. See [docs/roadmap.md](docs/roadmap.md).

Full roadmap: [docs/roadmap.md](docs/roadmap.md)  
Feature status: [docs/feature-status.md](docs/feature-status.md)  
Vision: [docs/vision.md](docs/vision.md)

---

## Contributing

We welcome contributions — bug reports, examples, documentation, and language proposals.

- [CONTRIBUTING.md](CONTRIBUTING.md) — build, test, coding standards
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
- [Issue templates](.github/ISSUE_TEMPLATE/) — bug, feature, language proposal, package proposal

```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
npm test
python3 scripts/normalize_inline_docs.py   # after bulk inline doc edits
```

Rust and TypeScript sources use **inline API documentation** (inside function bodies) and plain-English block comments before logic. See [CONTRIBUTING.md](CONTRIBUTING.md#inline-documentation).

---

## Documentation

| Document | Description |
|----------|-------------|
| [docs/getting-started.md](docs/getting-started.md) | First robot in 10 minutes |
| [docs/health-checks.md](docs/health-checks.md) | Health checks, fleet `require` clauses, policies |
| [docs/kill-switch.md](docs/kill-switch.md) | Kill switch syntax, `remote_signed`, handlers |
| [docs/iot.md](docs/iot.md) | IoT packages, dispatch, live bridge env flags |
| [docs/live-ai-provider.md](docs/live-ai-provider.md) | OpenAI, Anthropic, ONNX live paths |
| [docs/debugging.md](docs/debugging.md) | VS Code DAP — `behavior`, `task every`, `every` triggers |
| [docs/capability-traceability.md](docs/capability-traceability.md) | Capability exposure and traceability matrices |
| [docs/verification-diagnostics.md](docs/verification-diagnostics.md) | `--verification-json` and LSP quick-fixes |
| [docs/typed-handler-io.md](docs/typed-handler-io.md) | Handler return type annotations |
| [docs/testing.md](docs/testing.md) | `expect_compile_error` and test CLI |
| [docs/packages.md](docs/packages.md) | Package manager, `spanda publish`, capabilities |
| [docs/registry.md](docs/registry.md) | Hosted registry, signatures, golden path |
| [docs/killer-demo.md](docs/killer-demo.md) | 5-minute safety + verify + sim walkthrough |
| [docs/known-limitations.md](docs/known-limitations.md) | Honest platform constraints |
| [docs/benchmarks.md](docs/benchmarks.md) | Reproducible timing commands |
| [docs/demo-script.md](docs/demo-script.md) | 3-minute video walkthrough script |
| [docs/diagrams/](docs/diagrams/) | Architecture Mermaid diagrams |
| [docs/realtime.md](docs/realtime.md) | Deadline-aware tasks, wall-clock scheduling |
| [docs/reliability.md](docs/reliability.md) | Pipelines, watchdogs, recovery, operating modes |
| [docs/replay.md](docs/replay.md) | Mission trace record, deterministic replay, playback |
| [docs/regex.md](docs/regex.md) | Regex literals, triggers, subscription filters |
| [docs/triggers.md](docs/triggers.md) | Trigger-driven execution model |
| [docs/concurrency.md](docs/concurrency.md) | Tasks, spawn, channels, fleet CLI |
| [docs/architecture.md](docs/architecture.md) | Compiler pipeline and workspace crate map |
| [docs/lean-core.md](docs/lean-core.md) | Lean-core architecture (Phases 1–17) |
| [crates/README.md](crates/README.md) | Workspace crate index |
| [docs/feature-status.md](docs/feature-status.md) | Stable vs experimental vs planned |
| [docs/product-strategy.md](docs/product-strategy.md) | v0.5 beta priorities and positioning |
| [docs/spanda-language.md](docs/spanda-language.md) | Language reference |
| [docs/spanda-reference.md](docs/spanda-reference.md) | Full language API (JavaDoc + man-style CLI) |
| [docs/api-documentation.md](docs/api-documentation.md) | API doc hierarchy (language → compiler → JSON) |
| [docs/api-reference.md](docs/api-reference.md) | Rust/TypeScript compiler API (grouped by layer) |
| [docs/tutorials/README.md](docs/tutorials/README.md) | All tutorials, walkthroughs, and learning paths |
| [examples/README.md](examples/README.md) | Runnable examples library index |
| [docs/man/](docs/man/) | CLI manual pages |
| [docs/README.md](docs/README.md) | Full documentation index |

---

## License

Apache-2.0 — see [LICENSE](LICENSE).
