# Spanda Product Strategy

Strategic analysis for Spanda as an autonomous systems language. This document defines positioning, priorities, and release scope. It complements [vision.md](./vision.md) (aspiration) and [feature-status.md](./feature-status.md) (implementation truth).

**Last updated:** 2026-06-20 (post v0.1.0-alpha)

---

## Executive summary

Spanda should not compete as a general-purpose language, Python replacement, Rust replacement, or ROS replacement. It wins as **the coordination and verification layer** where:

1. AI output is **typed as untrusted** (`ActionProposal` → `SafeAction`)
2. Hardware fit is **checked before deploy** (`spanda verify`)
3. Safety is **mandatory in the language**, not bolted on

**Official positioning:** *Spanda is the Autonomous Systems Language.* *The pulse of autonomous intelligence.*

**Sharpened tagline:** *Design autonomous systems. Verify they fit the hardware. Block unsafe AI before it reaches actuators.*

### Philosophy

Hardware is the body.  
Sensors are the senses.  
AI models are the mind.  
Actuators are the muscles.  
Spanda is the intelligent pulse that transforms perception into action.

---

## Five pillars (core identity)

These are non-negotiable differentiators. Protect them in every release decision.

| # | Pillar | What it means |
|---|--------|---------------|
| 1 | **Safety-Typed AI** | `ActionProposal` cannot reach actuators; only `SafeAction` from `safety.validate()` can. Enforced at compile time and runtime. |
| 2 | **Hardware Compatibility Verification** | `spanda verify` checks memory, sensors, actuators, timing, battery, network, and AI model requirements against hardware profiles. |
| 3 | **Simulation-First Development** | `spanda check` → `spanda verify` → `spanda sim` as the default dev loop before hardware exists. |
| 4 | **Autonomous Systems Primitives** | `robot`, `sensor`, `actuator`, `task`, `agent`, `safety` are language keywords — not framework imports. |
| 5 | **Safe Deployment Validation** | `deploy`, `requires_hardware`, `verify { }`, mission duration, and task budgets encode the ship checklist in source. |

---

## Competitive landscape

### Where Spanda wins

| Capability | vs Python/Rust/C++ | vs ROS2/Dora-RS | vs Agentic AI frameworks |
|------------|-------------------|-----------------|--------------------------|
| AI safety as types | Convention only | Not addressed | Runtime filters only |
| Pre-deploy hardware verify | Manual CI/scripts | URDF + launch files | Not addressed |
| Units in robot syntax | Libraries | Not native | Not addressed |
| Unified design→verify→sim | Fragmented toolchain | Runtime graph only | No hardware/deploy model |

### Where Spanda does not win (and should not try)

| Area | Incumbent | Spanda stance |
|------|-----------|---------------|
| ML training | Python/PyTorch | Orchestrate via `extern python` |
| Real-time drivers | C++/Rust | Call via FFI; don't rewrite |
| Communication at scale | ROS2 | Bridge, don't replace |
| Physics simulation | Gazebo/Isaac/Mujoco | Physics-lite sim for safety testing only |
| Package ecosystem | PyPI/crates.io/ROS | Build incrementally; local stub today |
| Native performance | Rust/C++ | Interpreter first; LLVM deferred |

### Differentiation ratings

| Feature | Rating | Notes |
|---------|--------|-------|
| `ActionProposal` / `SafeAction` | **Highly differentiated** | No mainstream language enforces this natively |
| `spanda verify` | **Highly differentiated** | Closest analog is manual checklists, not a language primitive |
| Physical units | Somewhat unique | Libraries exist elsewhere; integration with robot syntax is the value |
| Robot-native syntax | Somewhat unique | ROS2/Dora are runtime graphs, not unified typed languages |
| `verify { }` assertions | Somewhat unique | Between unit tests and formal methods |
| Built-in simulation | Already common | Adequate for alpha; not a Gazebo replacement |
| Communication model | Already common | ROS2 owns production transport |
| AI agents / goals / memory | Already common | Structural value today; mock backends only |
| Blockchain / provenance | Already common | Optional; stub only — remove from core narrative |
| World models | Not implemented | Type names only; no runtime |
| LLVM / self-hosting | Already common | Premature for current adoption stage |

---

## Feature classification

Every major feature is classified for prioritization. See [feature-status.md](./feature-status.md) for implementation status.

| Feature | Classification |
|---------|----------------|
| ActionProposal / SafeAction | **Core Identity** |
| Safety validation (`safety { }`, zones, `stop_if`) | **Core Identity** |
| Physical units | **Core Identity** |
| Hardware profiles + `deploy` | **Core Identity** |
| Compatibility verification (`spanda verify`) | **Core Identity** |
| Autonomous primitives (robot/sensor/actuator/task) | **Core Identity** |
| Battery estimation | Important |
| Timing verification | Important |
| Simulation | Important |
| `verify { }` behavioral assertions | Important |
| AI agents (goal, memory, skill) | Important |
| Communication model | Important |
| Deployment targets | Core Identity |
| observe / fusion | Important |
| Package ecosystem | Important |
| Security (capabilities, signed comm) | Important |
| Python integration | Important |
| ROS2 integration | Important |
| Digital twins (local replay) | Nice To Have |
| Replay buffer | Nice To Have |
| Provenance / audit (local) | Nice To Have |
| C++ integration | Nice To Have |
| Concurrency (spawn/select) | Nice To Have |
| Generics / async / traits | Nice To Have |
| Digital twin cloud sync | Future |
| Multi-agent distributed runtime | Future |
| World models | Future |
| LLVM production backend | Future |
| Self-hosting compiler | Future |
| Blockchain support | Future (community packages only) |

---

## Focus tiers

### Tier 1 — Primary focus (max 5)

1. Safety-Typed AI
2. Hardware Compatibility Verification
3. Autonomous Systems Primitives
4. Simulation-First Development
5. Safe Deployment Validation

### Tier 2 — Secondary

- Python FFI (in-process PyO3 as adoption unlock)
- ROS2 bridge (one documented golden path)
- Communication primitives (in-memory + ROS adapter)
- Package manager (local → remote registry)
- LSP + published VS Code extension
- Lean security (capabilities; not a full IAM platform)

### Tier 3 — Experimental (Phase 22)

Promoted from deferred to experimental with minimal runtimes and golden paths. See [tier-3-experimental.md](./tier-3-experimental.md).

- LLVM native codegen (`spanda compile-native`, `scripts/llvm_golden_path.sh`)
- Blockchain / ledger (`spanda-ledger` → `MockLedgerBackend`)
- World models (`world_model.update` / `belief` runtime)
- Digital twin cloud upload (`SPANDA_CLOUD_UPLOAD_URL`)
- Distributed fleet (HTTP agents, mesh — partial)
- MQTT / DDS live transport (env-gated live bridges)
- C++ in-process FFI (`cpp-native` feature)
- Self-hosting bootstrap (`examples/self_host/`)

**Still future:** LLVM as primary path, production chain adapters, full world-model semantics, twin cloud SaaS, full fleet planning, OMG DDS, ROS replacement, complete self-hosted compiler.

---

## Release roadmap

### v1 vision

A robotics team writes a `.sd` program, runs `spanda verify` against their hardware profile, simulates it, connects one real AI provider and one real transport (Python + ROS2), and deploys coordination logic to edge hardware — with unsafe AI blocked at compile time.

### v0.1.0-alpha — shipped

| Must have | Status |
|-----------|--------|
| Interpreter + type checker | Done |
| ActionProposal → SafeAction | Done |
| `spanda verify` | Done |
| Physics-lite sim | Done |
| Showcase examples + CI | Done |

Optional (present): mock AI, experimental LLVM, stub registry.  
Not needed: blockchain, self-hosting, fleet runtime.

### v0.5 beta — target Q4 2026

| Must have | |
|-----------|---|
| Published VS Code extension with LSP | |
| One live AI path (OpenAI or local ONNX via Python bridge) | |
| One documented ROS2 golden path | |
| `spanda verify` CI integration guide | |
| Curated killer demo (`examples/showcase/killer_demo.sd`) | |
| Package install from remote registry (small, curated) | |

Optional: in-process PyO3, digital twin export, DAP debugger.  
Not needed: LLVM as primary, blockchain, world models.

### v1.0 — target 2027

| Must have | |
|-----------|---|
| Production-quality verify on 5+ hardware profiles | |
| Real deploy to one edge target (Jetson or Pi) | |
| Python + ROS2 interop documented and tested in CI | |
| Stable language subset with migration guide | |
| Security capabilities in production comm examples | |

Optional: LLVM for hot paths, twin replay export, fleet examples.  
Not needed: self-hosting, full blockchain suite, Gazebo parity.

---

## v0.5 beta — prioritized work items

Use these as GitHub issue titles. Order reflects dependency and adoption impact.

### P0 — Blockers for beta credibility

| # | Issue | Acceptance criteria |
|---|-------|---------------------|
| 1 | **Publish VS Code extension to marketplace** | Install from marketplace; LSP diagnostics for `check` and `verify` work out of the box |
| 2 | **Curate killer demo program** | Single `.sd` file: compile-time AI block + verify pass + sim with emergency stop; documented in `docs/killer-demo.md` |
| 3 | **One live AI provider path** | `extern python` or package calls real OpenAI/ONNX when configured; mock fallback when not |
| 4 | **One ROS2 golden path** | [ros2-golden-path.md](./ros2-golden-path.md) — rclpy bridge; `/cmd_vel` or `/scan` manual validation |
| 5 | **Remote package registry (minimal)** | `spanda install` fetches from a hosted registry; at least 2 curated packages (`spanda-openai`, `spanda-ros2`) |

### P1 — Adoption enablers

| # | Issue | Acceptance criteria |
|---|-------|---------------------|
| 6 | **CI integration guide for `spanda verify`** | [ci-verify.md](./ci-verify.md) — GitHub Actions + GitLab; `--json` parsed in CI |
| 7 | **In-process Python FFI (PyO3) as default when built** | Document build flags; subprocess remains fallback — see [ffi-and-ecosystem.md](./ffi-and-ecosystem.md) |
| 8 | **Hardware profile picker in LSP/VS Code** | Deploy target hints or autocomplete for built-in profiles |
| 9 | **Trim showcase to 3 flagship examples** | [examples/showcase/README.md](../examples/showcase/README.md) — safety, verify, sim |
| 10 | **Adoption quickstart: wrap existing Python stack** | [adoption-path.md](./adoption-path.md) — 1-sprint integration guide |

### P2 — Nice to have for beta

| # | Issue | Acceptance criteria |
|---|-------|---------------------|
| 11 | Digital twin replay export (JSON) | Export replay buffer for post-incident review |
| 12 | DAP debugger polish | Step through `task every` loops in VS Code |
| 13 | Web playground: killer demo preset | WASM run of killer demo without local Rust build |

### Explicitly out of scope for v0.5

- Blockchain / ledger packages
- Self-hosting compiler milestones
- LLVM `compile-native` as primary runtime
- World model runtime
- Digital twin cloud sync
- Distributed fleet orchestration
- MQTT/DDS live transport
- New general-purpose language features (heavy generics, advanced async)

---

## Developer adoption

### Current workflow

```
Python (AI, glue) + ROS2 (comm) + C++ (drivers) + manual safety monitors + deploy checklists
```

### Spanda value

```
Spanda (.sd) — coordination, safety gate, verify
  ↳ extern python — PyTorch, OpenCV, existing nodes
  ↳ ROS2 bridge — existing drivers unchanged
```

### Smallest adoption path (one sprint)

```bash
spanda check my_robot.sd
spanda verify my_robot.sd --target JetsonOrin
spanda sim my_robot.sd
```

Week 1: CI with `check` + `verify`.  
Week 2: one `extern python` call to existing model.  
Week 3: ROS2 bridge for a single topic.

**Guides:** [adoption-path.md](./adoption-path.md) · [ci-verify.md](./ci-verify.md) · [ros2-golden-path.md](./ros2-golden-path.md)

**Positioning:** Spanda is a 2–5k LOC coordination layer above existing stacks, not a rewrite.

---

## Killer demo: "The Unsafe Planner"

**Duration:** under 5 minutes  
**Message:** Unsafe AI is blocked at compile time; hardware fit is verified; safe execution is simulated.

| Step | Action | Audience sees |
|------|--------|---------------|
| 1 | Show `.sd` with `ai_model`, `safety { }`, `deploy` | Readable robot program |
| 2 | `spanda check` (unsafe version) | Compile error: `ActionProposal` cannot reach actuators |
| 3 | Add `safety.validate(proposal)` | `check` passes |
| 4 | `spanda verify --json` | Memory, sensors, timing, battery report |
| 5 | `spanda sim` | Robot drives; `stop_if` triggers near obstacle |
| 6 | `simulate_compatibility { fault ... }` | Verify warns before deploy |

**Implementation:** merge `examples/showcase/ai_safety_violation.sd` and `examples/showcase/hardware_compatibility.sd` into `examples/showcase/killer_demo.sd`; add `docs/killer-demo.md`.

Do **not** show blockchain, world models, LLVM, or fleet comm in this demo.

---

## Do not implement (next 6 months)

| Feature | Reason |
|---------|--------|
| Blockchain / crypto ledger packages | Stub only; distracts from safety story |
| Self-hosting compiler | No adoption value before v1 |
| Production LLVM backend | Verify + sim loop not proven in field |
| World models runtime | No implementation exists |
| Digital twin cloud sync | Local replay sufficient |
| Distributed fleet runtime | Single-process examples enough |
| MQTT/DDS live adapters | ROS2 first |
| Full ROS replacement | Guaranteed failure vs incumbents |
| Advanced power models | Heuristic battery check is enough |
| Expanding GP language surface | Rust/Go win on GP; Spanda wins on domain |

**Exception:** maintain existing LLVM/security/audit code if tests pass — but allocate no new engineering until Tier 1 is adoption-proven.

---

## Long-term vision

Spanda becomes the **standard coordination layer** for autonomous systems: the `.sd` file between Python AI pipelines and ROS2/C++ hardware, with verify-as-CI and safety-typed AI as industry expectation.

Not a general-purpose language. Not a ROS fork. The deploy-verified, safety-gated orchestration language for robots, drones, and edge agents.

---

## Related documents

- [vision.md](./vision.md) — aspirational positioning
- [feature-status.md](./feature-status.md) — honest implementation snapshot
- [roadmap.md](./roadmap.md) — engineering milestones
- [hardware-compatibility.md](./hardware-compatibility.md) — verify feature deep dive
- [ffi-and-ecosystem.md](./ffi-and-ecosystem.md) — Python/C++/ROS2 interop
- [future-blockchain-support.md](./future-blockchain-support.md) — optional, deferred
