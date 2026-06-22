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

---

## Architecture overview

Spanda uses a **lean-core, package-first** architecture. Core provides safety, verification, the type system, runtime hooks, and extension contracts. Robotics, AI, connectivity, simulation, and cloud integrations are added through [official packages](docs/official-packages.md).

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

| Layer | Technology | Responsibility |
|-------|------------|----------------|
| Language core | Rust (`spanda-core`) | Parser, types, safety, provider traits, compile gate, `run(source)` |
| Interpreter | Rust (`spanda-interpreter`) | Tree-walking runtime, `run_program`, simulator, integrated robotics/AI/safety subsystems |
| Official packages | `.sd` + `spanda.toml` | ROS2, MQTT, GPS, SLAM, vision, fleet, OTA, cloud |
| Native CLI | Rust (`spanda-cli`) | `check`, `verify`, `run`, `sim`, package manager |
| Bindings | N-API, WASM | Node and browser integration |
| Developer UX | TypeScript + React | CLI wrapper, LSP, web playground, tests, VS Code extension scaffold (`editor/vscode`) |

Deep dive: [docs/lean-core.md](docs/lean-core.md) · [docs/architecture.md](docs/architecture.md)

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

**Tutorial series:** [Tutorials index](docs/tutorials/README.md) · [For Dummies](docs/spanda-for-dummies/README.md) · [Spanda 101](docs/spanda-101/README.md)

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

5-minute walkthrough: [`docs/killer-demo.md`](docs/killer-demo.md) · Adoption path: [`docs/adoption-path.md`](docs/adoption-path.md) · CI: [`docs/ci-verify.md`](docs/ci-verify.md)

More showcase demos: [`examples/showcase/README.md`](examples/showcase/README.md). Real-time: [`examples/realtime/`](examples/realtime/); regex: [`examples/regex/`](examples/regex/).

---

## Installation

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
| `spanda test` | Run project tests |
| `spanda fmt <file.sd>` | Format source |
| `spanda lint <file.sd>` | Lint source |
| `spanda doc <file.sd>` | Generate markdown documentation |
| `spanda build` | Build project |
| `spanda install` | Install dependencies |

Verify flags: `--target <Profile>`, `--all-targets`, `--simulate`, `--json`

Run/sim/fleet trace flags: `--trace-scheduler`, `--trace-tasks`, `--trace-triggers`, `--trace-events`, `--trace-realtime`, `--metrics-json`, `--record`, `--wall-clock`, `--replay` (sim)

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

**v0.1.0-alpha (current):** Stable interpreter, safety gate, hardware verify, simulation, mock AI, package manager, unified triggers, cooperative concurrency, real-time contracts, mission trace replay, first-class regex, showcase examples.

**Next (v0.5 beta):** LLVM production backend, VS Code marketplace publishing, live AI providers, in-process FFI, distributed multi-robot runtime. See [docs/product-strategy.md](docs/product-strategy.md).

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
| [docs/killer-demo.md](docs/killer-demo.md) | 5-minute safety + verify + sim walkthrough |
| [docs/realtime.md](docs/realtime.md) | Deadline-aware tasks, wall-clock scheduling |
| [docs/reliability.md](docs/reliability.md) | Pipelines, watchdogs, recovery, operating modes |
| [docs/replay.md](docs/replay.md) | Mission trace record, deterministic replay, playback |
| [docs/regex.md](docs/regex.md) | Regex literals, triggers, subscription filters |
| [docs/triggers.md](docs/triggers.md) | Trigger-driven execution model |
| [docs/concurrency.md](docs/concurrency.md) | Tasks, spawn, channels, fleet CLI |
| [docs/architecture.md](docs/architecture.md) | Compiler pipeline and diagrams |
| [docs/feature-status.md](docs/feature-status.md) | Stable vs experimental vs planned |
| [docs/product-strategy.md](docs/product-strategy.md) | v0.5 beta priorities and positioning |
| [docs/spanda-language.md](docs/spanda-language.md) | Language reference |
| [docs/spanda-reference.md](docs/spanda-reference.md) | Full language API (JavaDoc + man-style CLI) |
| [docs/api-reference.md](docs/api-reference.md) | Rust/TypeScript compiler API index |
| [docs/man/](docs/man/) | CLI manual pages |
| [docs/README.md](docs/README.md) | Full documentation index |

---

## License

Apache-2.0 — see [LICENSE](LICENSE).
