<p align="center">
  <img src="assets/image/banner.png" alt="Spanda — The Autonomous Systems Platform" width="640">
</p>

# Spanda

**The Autonomous Systems Platform** — *with a safety-first programming language at its core.*

*Build. Verify. Simulate. Deploy. Operate.*

Spanda is an autonomous systems platform centered on the **Spanda Language** (`.sd` files): typed robot programs, safety gates, hardware verification, simulation, replay, fleet operations, mission assurance, mission continuity, and **38** official packages.

**Spanda focuses on Readiness, Assurance, and Diagnosis for safety-critical autonomous systems.**

Spanda helps answer:

- Can this mission run?
- Can this robot safely perform this mission?
- Does the hardware satisfy the required capabilities?
- Is the system healthy enough to deploy?
- Can it run safely?
- Can it recover?
- Can it continue?
- Can it be trusted?
- Why should this deployment be trusted?
- Why did it behave this way?
- What happened when something failed?
- Who can take over when a robot or fleet member fails mid-mission?
- What evidence supports deployment?

**Signature capabilities:** Safety-Typed AI · Mission Contracts · Readiness Engine · Continuity & Takeover · Trust Framework · Explainability & Audit Trail — see [docs/differentiation-roadmap.md](docs/differentiation-roadmap.md).

Repository: [github.com/Davalgi/Spanda](https://github.com/Davalgi/Spanda)

---

## Table of contents

- [Philosophy](#philosophy)
- [What is Spanda?](#what-is-spanda)
- [Why Spanda?](#why-spanda)
- [Quick start](#quick-start)
- [Try Spanda in 5 Minutes](#try-spanda-in-5-minutes)
- [Flagship Demos](#flagship-demos)
- [Where Should I Start?](#where-should-i-start)
- [What Spanda Is / Is Not](#what-spanda-is--is-not)
- [Feature Status](#feature-status)
- [Spanda Platform Map](#spanda-platform-map)
- [What you get](#what-you-get)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)

---

## Philosophy

Hardware is the body.  
Sensors are the senses.  
AI models are the mind.  
Actuators are the muscles.  
Spanda is the intelligent pulse that transforms perception into action.

**Spanda** (*Pronounced **SPUN-duh** (/ˈspʌndə/)*) is a Sanskrit term meaning *the divine pulse* — the creative vibration of consciousness and energy that manifests as expansion and contraction in all entities, bridging stillness and movement within consciousness; and the first stir of awareness that creates and sustains the universe.


---

## What is Spanda?

Spanda is an **autonomous systems platform** built around the **Spanda Language** — a typed programming language where sensors, AI models, actuators, safety rules, and deployment targets are first-class concepts in source code.

You write a `robot` block with sensors, actuators, safety zones, and agents. The compiler enforces physical units, validates AI proposals before they reach hardware, and checks that your program fits the deployment target before you ship.

```spanda
robot SafePatrol {
  sensor lidar: Lidar;
  actuator wheels: DifferentialDrive;
  ai_model planner: LLM { provider: "mock"; model: "patrol"; }

  safety {
    max_speed = 0.5 m/s;
    stop_if lidar.nearest_distance < 0.5 m;
  }

  behavior patrol() {
    loop every 100ms {
      let proposal = planner.reason(prompt: "Plan motion", input: lidar.read());
      wheels.execute(safety.validate(proposal));
    }
  }
}
```

---

## Why Spanda?

Building autonomous systems today means stitching together Python scripts, C++ drivers, ROS2 nodes, safety monitors, and deployment checklists — with no single platform that treats **AI output as untrusted**, **hardware fit as compile-time**, and **safety as mandatory**.

**Traditional languages focus on:** algorithms, data structures, applications.

**Spanda focuses on:** autonomous systems, safety, hardware awareness, capability verification, simulation, and operational health.

Spanda is the coordination layer where perception, planning, safety validation, simulation, verification, and deployment live together — with the `.sd` language as the expressive core.

---

## Quick start

```bash
# Install (from clone)
git clone https://github.com/Davalgi/Spanda.git
cd Spanda && ./scripts/install.sh
# Or: cargo install --path crates/spanda-cli --locked

spanda demo rover          # flagship platform demo
spanda demo assurance      # mission assurance CLI suite
spanda demo self-healing   # recovery policies, heal/recover/sim
spanda demo continuity     # takeover, delegation, succession

# Or step by step:
spanda check examples/showcase/killer_demo.sd      # type-check
spanda verify examples/showcase/hardware_compatibility.sd  # hardware fit
spanda sim examples/showcase/killer_demo.sd        # simulate

# Documentation
spanda doc examples/showcase/killer_demo.sd        # API docs from .sd source
spanda man verify                                  # CLI man page
```

Install options: [docs/installation.md](docs/installation.md) · First project: [docs/getting-started.md](docs/getting-started.md)

Video script: [docs/demo-script.md](docs/demo-script.md) · Killer demo: [docs/killer-demo.md](docs/killer-demo.md)

---

## Try Spanda in 5 Minutes

One path to evaluate Spanda from a fresh clone:

```bash
git clone https://github.com/Davalgi/Spanda.git
cd Spanda
cargo build --release
./target/release/spanda demo rover
./target/release/spanda demo safety
./target/release/spanda demo verify
```

Optional — readiness, assurance, and diagnosis on showcase examples:

```bash
./target/release/spanda readiness examples/showcase/readiness/rover.sd
./target/release/spanda assure examples/showcase/assurance/rover.sd
./target/release/spanda diagnose examples/showcase/root_cause_analysis/mission.trace
```

Install on `PATH` instead: `./scripts/install.sh` or `cargo install --path crates/spanda-cli --locked` — then use `spanda` without the `./target/release/` prefix. See [docs/installation.md](docs/installation.md).

---

## Flagship Demos

Three primary stories for new visitors. Other demos (`fleet`, `health`, `readiness`, `assurance`, `self-healing`, `continuity`, and more) remain in [Quick start](#quick-start) and [docs/overview/demos-and-examples.md](docs/overview/demos-and-examples.md).

### 1. Safety-Typed AI

**Flow:** `ActionProposal` → Safety Validation → `SafeAction`

```bash
./target/release/spanda demo safety
```

**Expected:** unsafe program fails at compile time; safe program passes `safety.validate()` gate.

**Example:** [examples/showcase/unsafe_ai/](examples/showcase/unsafe_ai/)

### 2. Hardware + Capability Verification

**Flow:** Mission → Capability → Hardware → Provider → Safety Rule

```bash
./target/release/spanda demo verify
```

**Expected:** mission without Lidar fails verification; complete robot passes with JSON report.

**Example:** [examples/showcase/hardware_verification/](examples/showcase/hardware_verification/)

### 3. Readiness / Assurance / Diagnosis

**Questions:** Can the robot run? Why should we trust it? What happened and why?

```bash
./target/release/spanda readiness examples/showcase/readiness/rover.sd
./target/release/spanda assure examples/showcase/assurance/rover.sd
./target/release/spanda diagnose examples/showcase/root_cause_analysis/mission.trace
```

**Expected:** readiness score and go/no-go; assurance report with evidence cases; diagnosis report from mission trace.

**Examples:** [examples/showcase/readiness/](examples/showcase/readiness/) · [examples/showcase/assurance/](examples/showcase/assurance/) · [examples/showcase/root_cause_analysis/](examples/showcase/root_cause_analysis/)

---

## Where Should I Start?

**For developers**

- Try the safety demo: `spanda demo safety`
- Read the language guide: [docs/spanda-language.md](docs/spanda-language.md)

**For robotics engineers**

- Try hardware verification: `spanda demo verify`
- Read capability verification: [docs/capability-traceability.md](docs/capability-traceability.md) · [docs/hardware-compatibility.md](docs/hardware-compatibility.md)

**For safety / reliability engineers**

- Try assurance and readiness: `spanda demo assurance` or the [Readiness / Assurance / Diagnosis](#3-readiness--assurance--diagnosis) commands above
- Try mission continuity: `spanda demo continuity` — takeover, delegation, and succession when robots fail mid-mission
- Read traceability and safety: [docs/mission-assurance.md](docs/mission-assurance.md) · [docs/mission-continuity.md](docs/mission-continuity.md) · [docs/capability-traceability.md](docs/capability-traceability.md) · [docs/safety-reporting.md](docs/safety-reporting.md)

**For Quality Assurance or Test Engineers**

- Try in-language tests and the rover demo: `spanda test examples/basics/07_in_language_tests.sd` and `spanda demo rover` (verify → sim → record/replay)
- Try fault injection and health checks: `spanda demo health`
- Read testing and verification: [docs/testing.md](docs/testing.md) · [docs/replay.md](docs/replay.md) · [docs/ci-verify.md](docs/ci-verify.md) · [docs/mission-verification.md](docs/mission-verification.md)

**For fleet operators**

- Try self-healing and continuity: `spanda demo self-healing` and `spanda demo continuity`
- Read fleet mesh APIs: [docs/fleet-distributed.md](docs/fleet-distributed.md) · [docs/self-healing.md](docs/self-healing.md) · [docs/continuity-policies.md](docs/continuity-policies.md)

**For contributors**

- Read [CONTRIBUTING.md](CONTRIBUTING.md)
- Run tests: `cargo test --workspace && npm test`
- Pick a good first issue on [GitHub Issues](https://github.com/Davalgi/Spanda/issues)

---

## What Spanda Is / Is Not

**Spanda is:**

- an autonomous systems platform
- a safety-first language and runtime
- a verification and assurance layer
- a simulation-first development workflow
- a package / provider ecosystem

**Spanda is not:**

- a replacement for Python
- a replacement for C++
- a replacement for ROS2
- a drone autopilot
- a custom operating system
- a blockchain platform

Spanda **integrates with** existing ecosystems instead of replacing them. See [docs/ffi-and-ecosystem.md](docs/ffi-and-ecosystem.md) and [docs/platform-overview.md](docs/platform-overview.md#what-spanda-is-not).

---

## Feature Status

Compact snapshot — full matrix: [docs/feature-status.md](docs/feature-status.md)

| Feature | Status | Notes |
|---------|--------|-------|
| Spanda Language | **Stable** | `.sd` robot programs, units, safety types |
| Parser | **Stable** | Rust authoritative; TypeScript mirror |
| Type Checker | **Stable** | Physical units, `SafeAction` gate |
| CLI | **Stable** | `check`, `verify`, `run`, `sim`, `demo`, packages |
| Safety-Typed AI | **Stable** | `ActionProposal` → `safety.validate()` → `SafeAction` |
| Hardware Verification | **Stable** | `spanda verify` against hardware profiles |
| Capability Verification | **Stable** | Traceability, grants, minimum-hardware analysis |
| Readiness | **Stable** | Weighted go/no-go scoring |
| Assurance | **Stable** | `spanda assure`, assurance cases, mission assurance CLI |
| Diagnosis | **Stable** | `spanda diagnose` on traces and programs |
| Simulation | **Stable** | `spanda run` / `spanda sim`, physics-lite 2D |
| Replay | **Stable** | Mission trace record, deterministic replay |
| Health | **Stable** | `health_check`, fleet `require`, policies |
| Security / Encryption | **Stable** | Capabilities, audit, AES-GCM wire frames; live TLS optional |
| Package System | **Stable** | `spanda install`, `build`, `test`, hosted index |
| Provider Registry | **Stable** | Official packages + dispatch; local mirror |
| Fleet | **Experimental** | In-process sim stable; distributed HTTP agents experimental |
| IoT | **Experimental** | Live Modbus/OPC-UA env-gated; hub fallback |
| Debugger | **Experimental** | VS Code DAP via `spanda-dap` |
| LLVM | **Experimental** | `spanda ir`, `compile-native` — interpreter is primary runtime |
| WASM | **Experimental** | Browser check/run/verify; limited vs native CLI |
| ROS2 | **Experimental** | rclrs/rclpy bridge; requires ROS Humble setup |
| GitHub Pages / Docs Site | **Experimental** | mdBook under [docs-site/](docs-site/); build with `mdbook build docs-site` |

---

## Signature capabilities

| Capability | Status | Doc |
|------------|--------|-----|
| **Safety-Typed AI** | Stable | [agentic-programming.md](docs/agentic-programming.md) |
| **Readiness Engine** | Stable | [readiness.md](docs/readiness.md) |
| **Continuity & Takeover** | Stable | [mission-continuity.md](docs/mission-continuity.md) |
| **Mission Contracts** | Planned (NOW) | [mission-contracts.md](docs/mission-contracts.md) |
| **Trust Framework** | Planned (NEXT) | [trust-framework.md](docs/trust-framework.md) |
| **Explainability & Audit Trail** | Planned (NOW) | [explainability.md](docs/explainability.md) |

Roadmap: [docs/differentiation-roadmap.md](docs/differentiation-roadmap.md)

---

## Spanda Platform Map

One-line pointers — details in [docs/platform-overview.md](docs/platform-overview.md).

| Component | Summary | Doc |
|-----------|---------|-----|
| **Spanda Language** | Safety-first `.sd` programs with robot, sensor, actuator, and safety primitives | [docs/spanda-language.md](docs/spanda-language.md) |
| **Spanda Runtime** | Interpreter, scheduler, HAL, provider dispatch after compile-time gates | [docs/architecture.md](docs/architecture.md) |
| **Spanda Verify** | Hardware fit, capability traceability, behavioral `verify { }` blocks | [docs/hardware-compatibility.md](docs/hardware-compatibility.md) |
| **Spanda Safety** | `SafeAction` type gate, safety zones, kill switch, emergency stop | [docs/agentic-programming.md](docs/agentic-programming.md) |
| **Spanda Sim** | Simulation and digital twins without physical hardware | [docs/killer-demo.md](docs/killer-demo.md) |
| **Spanda Replay** | Mission trace capture and deterministic playback | [docs/replay.md](docs/replay.md) |
| **Persistent telemetry** | Device/sensor/heartbeat/health store (`--persist-telemetry`); JSONL or SQLite; OTLP export, `push --watch`, session auto-push (`SPANDA_OTLP_AUTO_PUSH`), fleet mesh aggregation (`telemetry fleet-push`); sessions + replay | [docs/telemetry-store.md](docs/telemetry-store.md) |
| **Spanda Health** | Runtime health checks and fleet readiness requirements | [docs/health-checks.md](docs/health-checks.md) |
| **Spanda Assurance** | Knowledge models, anomaly detection, prognostics, assurance cases | [docs/mission-assurance.md](docs/mission-assurance.md) |
| **Mission continuity** | Takeover, delegation, succession, checkpoint resume | [docs/mission-continuity.md](docs/mission-continuity.md) |
| **Spanda Diagnosis** | Root-cause analysis from mission traces and programs | [docs/diagnostics.md](docs/diagnostics.md) |
| **Spanda Registry** | Package index, install, publish, signed tarballs | [docs/registry.md](docs/registry.md) |
| **Spanda Providers** | Official package traits — ROS2, MQTT, vision, fleet, and more | [docs/how-providers-work.md](docs/how-providers-work.md) |

---

## What you get

| | | |
|---|---|---|
| **Language (.sd)** | **Safety validation** | **Hardware verify** |
| **Simulation & replay** | **Mission assurance** | **Packages (38)** |
| **Health & readiness** | **Fleet & OTA** | **ROS2 / IoT / AI bridges** |

More samples: [docs/overview/code-samples.md](docs/overview/code-samples.md) · Demos: [docs/overview/demos-and-examples.md](docs/overview/demos-and-examples.md)

---

## Documentation

### Documentation by topic

| Topic | Guide |
|-------|--------|
| **Getting Started** | [docs/getting-started.md](docs/getting-started.md) |
| **Language Guide** | [docs/spanda-language.md](docs/spanda-language.md) · [docs/language-reference/](docs/language-reference/README.md) |
| **Architecture** | [docs/architecture.md](docs/architecture.md) · [docs/overview/architecture.md](docs/overview/architecture.md) |
| **Safety** | [docs/agentic-programming.md](docs/agentic-programming.md) · [docs/kill-switch.md](docs/kill-switch.md) |
| **Verification** | [docs/hardware-compatibility.md](docs/hardware-compatibility.md) · [docs/ci-verify.md](docs/ci-verify.md) |
| **Readiness** | [docs/readiness.md](docs/readiness.md) |
| **Differentiation** | [docs/differentiation-roadmap.md](docs/differentiation-roadmap.md) · [mission-contracts.md](docs/mission-contracts.md) |
| **Assurance** | [docs/mission-assurance.md](docs/mission-assurance.md) |
| **Mission continuity** | [docs/mission-continuity.md](docs/mission-continuity.md) |
| **Self-healing** | [docs/self-healing.md](docs/self-healing.md) |
| **Diagnosis** | [docs/diagnostics.md](docs/diagnostics.md) · [docs/root-cause-analysis.md](docs/root-cause-analysis.md) |
| **Health** | [docs/health-checks.md](docs/health-checks.md) |
| **Packages** | [docs/packages.md](docs/packages.md) · [docs/how-packages-work.md](docs/how-packages-work.md) |
| **Providers** | [docs/how-providers-work.md](docs/how-providers-work.md) · [docs/official-packages.md](docs/official-packages.md) |
| **Security** | [docs/security.md](docs/security.md) · [docs/security-architecture.md](docs/security-architecture.md) |
| **Examples** | [examples/README.md](examples/README.md) · [docs/overview/demos-and-examples.md](docs/overview/demos-and-examples.md) |
| **Roadmap** | [docs/roadmap.md](docs/roadmap.md) |

Full index: [docs/README.md](docs/README.md)

### Reference

| Resource | Link |
|----------|------|
| **Language reference (topics)** | [docs/language-reference/](docs/language-reference/README.md) — syntax, types, agents, safety, packages, recovery, continuity |
| **Generated language API** | [docs/spanda-reference.md](docs/spanda-reference.md) — keywords, `std.*`, builtins, CLI |
| **CLI man pages** | [docs/man/](docs/man/README.md) — `spanda man <command>` or browse markdown |
| **Rust & TypeScript API** | [docs/api-reference.md](docs/api-reference.md) — crates, modules, types (`cargo doc --workspace --no-deps`) |
| **Documentation site (mdBook)** | [docs-site/](docs-site/) — build with `mdbook build docs-site` |

### Doc commands

```bash
spanda doc src/main.sd                    # JavaDoc-style API docs (markdown)
spanda doc --html src/main.sd --out api.html
spanda doc examples/ --out target/api-docs
spanda man                                # list CLI man pages
spanda man run                            # show spanda-run(1)
spanda reference --out docs/spanda-reference.md --man-dir docs/man
```

### Start here

| Start here | Description |
|------------|-------------|
| **[docs/overview/](docs/overview/README.md)** | Expanded project home — platform, architecture, CLI, library |
| [docs/getting-started.md](docs/getting-started.md) | First robot in 10 minutes |
| [docs/platform-overview.md](docs/platform-overview.md) | Platform components and workflow |
| [docs/tutorials/README.md](docs/tutorials/README.md) | Tutorials and learning paths |
| [examples/README.md](examples/README.md) | Runnable examples library |
| [docs/README.md](docs/README.md) | Full documentation index |

| Topic | Guide |
|-------|--------|
| Feature status | [docs/feature-status.md](docs/feature-status.md) |
| Mission assurance | [docs/mission-assurance.md](docs/mission-assurance.md) |
| Architecture | [docs/overview/architecture.md](docs/overview/architecture.md) (pipeline) · [docs/architecture.md](docs/architecture.md) (full) |
| CLI overview | [docs/overview/cli.md](docs/overview/cli.md) |
| Packages & registry | [docs/overview/packages.md](docs/overview/packages.md) · [docs/packages.md](docs/packages.md) |
| Roadmap | [docs/roadmap.md](docs/roadmap.md) |

### Overview subpages

Full index: [docs/overview/README.md](docs/overview/README.md) — platform structure, components, differentiators, layers, library, packages, web playground, CLI, code samples, demos.

---

## Contributing

[CONTRIBUTING.md](CONTRIBUTING.md) · [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)

```bash
cargo test --workspace && npm test
```

---

## License

Apache-2.0 — see [LICENSE](LICENSE).
