# Spanda Tutorials — Index

One place to find every learning path: quick starts, structured courses, topic guides, demos, and runnable examples.

**New here?** Pick a track below — you can switch anytime.

| I want… | Start here | Time |
|---------|------------|------|
| Plain English, no jargon | [Spanda for Dummies](../spanda-for-dummies/README.md) | ~45 min read |
| Structured lessons + exercises | [Spanda 101](../spanda-101/README.md) | ~3 hours |
| First robot in 10 minutes | [Getting started](../getting-started.md) | ~10 min |
| Impress someone in a meeting | [Killer demo](../killer-demo.md) | ~5 min |

---

## 1. Beginner tracks

Curated paths for your first programs.

| Tutorial | Format | Best for |
|----------|--------|----------|
| [Spanda for Dummies](../spanda-for-dummies/README.md) | 7 short chapters, glossary, cheat sheet | Casual readers, managers, first week |
| [Spanda 101](../spanda-101/README.md) | 10 lessons with exercises | Developers learning systematically |
| [Getting started](../getting-started.md) | Single quickstart doc | Install → first project → core commands |
| [Installation](../installation.md) | Platform installers & PATH | Before any hands-on tutorial |

### Spanda for Dummies (chapters)

| # | Chapter |
|---|---------|
| 1 | [What is Spanda, anyway?](../spanda-for-dummies/01-what-is-spanda.md) |
| 2 | [Your first five minutes](../spanda-for-dummies/02-five-minutes.md) |
| 3 | [Anatomy of a robot program](../spanda-for-dummies/03-robot-anatomy.md) |
| 4 | [AI without the scary parts](../spanda-for-dummies/04-ai-made-simple.md) |
| 5 | [Ten commands cheat sheet](../spanda-for-dummies/05-commands-cheat-sheet.md) |
| 6 | [Common mistakes](../spanda-for-dummies/06-common-mistakes.md) |
| 7 | [Glossary](../spanda-for-dummies/07-glossary.md) |

### Spanda 101 (lessons)

| # | Lesson | Example |
|---|--------|---------|
| 1 | [Hello, robot](../spanda-101/01-hello-robot.md) | `examples/basics/01_minimal_robot.sd` |
| 2 | [Sensors and safety](../spanda-101/02-sensors-and-safety.md) | `02_sensors_and_safety.sd` |
| 3 | [Control flow](../spanda-101/03-control-flow.md) | `03_control_flow.sd` |
| 4 | [Types and errors](../spanda-101/04-types-and-errors.md) | `04_result_and_option.sd` |
| 5 | [Modules and traits](../spanda-101/05-modules-and-traits.md) | `05_traits_and_impl.sd` |
| 6 | [AI and the safety gate](../spanda-101/06-ai-and-the-safety-gate.md) | `showcase/rover_navigation.sd` |
| 7 | [Tasks, triggers, concurrency](../spanda-101/07-tasks-triggers-concurrency.md) | `integration/triggers_minimal.sd` |
| 8 | [Hardware and verify](../spanda-101/08-hardware-and-verify.md) | `integration/verify_walkthrough.sd` |
| 9 | [Packages and tests](../spanda-101/09-packages-and-tests.md) | `basics/07_in_language_tests.sd` |
| 10 | [End-to-end patrol](../spanda-101/10-end-to-end-patrol.md) | `end_to_end/safe_patrol/` |

---

## 2. Walkthroughs & flagship demos

Scripted flows for talks, evals, and “show me what Spanda does.”

| Walkthrough | What you demonstrate |
|-------------|----------------------|
| [Killer demo](../killer-demo.md) | Compile-time AI safety, `spanda verify`, simulation |
| [Killer demo source](../../examples/showcase/killer_demo.sd) | Runnable flagship program |
| [AI safety violation](../../examples/showcase/ai_safety_violation.sd) | Intentional compile error (`ActionProposal`) |
| [Hardware compatibility](../../examples/showcase/hardware_compatibility.sd) | Deploy + verify report |
| [Release announcement v0.1.0-alpha](../release-announcement-v0.1.0-alpha.md) | Launch copy and positioning |

---

## 3. Topic tutorials

Deep dives on one capability. Read after a beginner track.

| Topic | Guide | Example directory |
|-------|-------|-------------------|
| Language syntax | [spanda-language.md](../spanda-language.md) | `examples/types/`, `examples/basics/` |
| Triggers | [triggers.md](../triggers.md) | `examples/triggers_demo.sd` |
| Concurrency | [concurrency.md](../concurrency.md) | `examples/concurrency.sd` |
| Real-time | [realtime.md](../realtime.md) | `examples/realtime/` |
| Reliability | [reliability.md](../reliability.md) | `examples/realtime/watchdog.sd`, `recovery.sd` |
| Watchdogs | [watchdogs.md](../watchdogs.md) | `examples/realtime/watchdog.sd` |
| Degraded modes | [degraded-modes.md](../degraded-modes.md) | `examples/realtime/degraded_mode.sd` |
| Mission replay | [replay.md](../replay.md) | `examples/end_to_end/replay_mission.sd` |
| Regex | [regex.md](../regex.md) | `examples/regex/` |
| Hardware & deploy | [hardware-compatibility.md](../hardware-compatibility.md) | `examples/hardware/` |
| Packages | [packages.md](../packages.md) | `examples/packages/` |
| Standard library | [standard-library.md](../standard-library.md) | `examples/std/` |
| Type system | [spanda-type-system.md](../spanda-type-system.md) | `examples/types/` |
| FFI & ROS2 | [ffi-and-ecosystem.md](../ffi-and-ecosystem.md) | `examples/ffi_*.sd`, `ros2_bridge.sd` |
| Architecture | [architecture.md](../architecture.md) | — |
| Migration | [migration.md](../migration.md) | — |

---

## 4. Example libraries (learn by reading code)

Runnable `.sd` programs grouped by skill level. All paths relative to [`examples/`](../../examples/).

### Feature coverage map

**[examples/features/README.md](../../examples/features/README.md)** — master index mapping **every capability** to a runnable file (language core, AI, triggers, hardware, regex, FFI, and more).

### Progressive ladder

| Tier | Directory | Index |
|------|-----------|-------|
| Basics | `examples/basics/` | [README](../../examples/basics/README.md) |
| Integration | `examples/integration/` | triggers, concurrency, verify |
| Features | `examples/features/` | one file per capability |
| End-to-end | `examples/end_to_end/` | [README](../../examples/end_to_end/README.md) — patrol, warehouse, fleet, replay, … |

### Curated & domain demos

| Directory | Focus |
|-----------|--------|
| [`showcase/`](../../examples/showcase/) | v0.1.0-alpha flagship demos |
| [`features/`](../../examples/features/) | One file per capability (dyn trait, join, zones, QoS, …) |
| [`realtime/`](../../examples/realtime/) | Deadlines, pipelines, watchdogs, replay |
| [`regex/`](../../examples/regex/) | Pattern triggers and validation |
| [`communication/`](../../examples/communication/) | Topics, services, fleet |
| [`hardware/`](../../examples/hardware/) | Deploy and compatibility |
| [`modules/`](../../examples/modules/) | Cross-file imports |
| [`types/`](../../examples/types/) | Type-system snippets |
| [`std/`](../../examples/std/) | Standard library samples |
| [`packages/`](../../examples/packages/) | `spanda.toml` project layouts |

### Application demos (root `examples/`)

Noteworthy single-file programs: `lidar_avoidance.sd`, `ai_navigation.sd`, `humanoid_assistant.sd`, `multi_robot_fleet.sd` (under `communication/`), `warehouse_logistics.sd`, platform HAL demos (`esp32_sensors.sd`, `jetson_inspection.sd`, etc.).

---

## 5. Reference docs (lookup, not lessons)

Use these when you know *what* you need; use tutorials when you're learning *how*.

| Doc | Use when |
|-----|----------|
| [spanda-reference.md](../spanda-reference.md) | Keyword, builtin, CLI lookup |
| [api-reference.md](../api-reference.md) | Rust/TypeScript compiler API |
| [man/](../man/) | Man-page CLI reference |
| [feature-status.md](../feature-status.md) | What is stable vs experimental |
| [api-contract.json](../api-contract.json) | JSON schemas for tool integration |

---

## 6. Suggested learning paths

### Path A — “I have an afternoon”

1. [Spanda for Dummies](../spanda-for-dummies/README.md) chapters 1–3  
2. Run `examples/basics/01` through `03`  
3. [Killer demo](../killer-demo.md)

### Path B — “I want to ship a patrol bot”

1. [Getting started](../getting-started.md)  
2. [Spanda 101](../spanda-101/README.md) lessons 1–6, 8, 10  
3. `examples/end_to_end/safe_patrol/`  
4. [hardware-compatibility.md](../hardware-compatibility.md)

### Path C — “I need real-time + replay”

1. [Spanda 101](../spanda-101/README.md) lessons 1–3, 7  
2. [realtime.md](../realtime.md) + `examples/realtime/`  
3. [replay.md](../replay.md) + `examples/end_to_end/replay_mission.sd`

### Path D — “I'm evaluating for my team”

1. [vision.md](../vision.md)  
2. [Killer demo](../killer-demo.md)  
3. [feature-status.md](../feature-status.md)  
4. [product-strategy.md](../product-strategy.md)

---

## 7. Contributing tutorials

Adding a new tutorial? See [CONTRIBUTING.md](../../CONTRIBUTING.md):

- Beginner content → `docs/spanda-for-dummies/` or `docs/spanda-101/`
- Topic guide → `docs/<topic>.md` + runnable example
- Golden CI → `tests/golden/manifest.json`
- Update **this index** when you add a new learning resource

---

## Quick commands (all tutorials)

```bash
spanda check examples/basics/01_minimal_robot.sd
spanda run examples/basics/02_sensors_and_safety.sd
spanda verify examples/integration/verify_walkthrough.sd --target RoverV1
spanda test examples/basics/07_in_language_tests.sd
```

Build CLI: `npm run build:rust` → `target/release/spanda`
