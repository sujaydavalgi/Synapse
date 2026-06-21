# Spanda 101

A hands-on tutorial series for learning Spanda from your first robot program through deployment verification.

Each lesson links to a runnable example in the repo. Work through them in order, or jump to a topic you need.

**Prefer plain English first?** See [Spanda for Dummies](../spanda-for-dummies/README.md) (~45 min, no jargon).

**All tutorials:** [Tutorials index](../tutorials/README.md)

**Prerequisites:** [Installation](../installation.md) and a built CLI (`npm run build:rust` → `target/release/spanda`).

---

## Curriculum

| # | Lesson | Example | Time |
|---|--------|---------|------|
| 1 | [Hello, robot](./01-hello-robot.md) | `examples/basics/01_minimal_robot.sd` | 10 min |
| 2 | [Sensors and safety](./02-sensors-and-safety.md) | `examples/basics/02_sensors_and_safety.sd` | 15 min |
| 3 | [Control flow and loops](./03-control-flow.md) | `examples/basics/03_control_flow.sd` | 15 min |
| 4 | [Types, units, and errors](./04-types-and-errors.md) | `examples/basics/04_result_and_option.sd` | 20 min |
| 5 | [Modules and traits](./05-modules-and-traits.md) | `examples/basics/05_traits_and_impl.sd` | 20 min |
| 6 | [AI and the safety gate](./06-ai-and-the-safety-gate.md) | `examples/showcase/rover_navigation.sd` | 25 min |
| 7 | [Tasks, triggers, and concurrency](./07-tasks-triggers-concurrency.md) | `examples/integration/triggers_minimal.sd` | 25 min |
| 8 | [Hardware profiles and verify](./08-hardware-and-verify.md) | `examples/integration/verify_walkthrough.sd` | 20 min |
| 9 | [Packages and tests](./09-packages-and-tests.md) | `examples/basics/07_in_language_tests.sd` | 20 min |
| 10 | [End-to-end patrol](./10-end-to-end-patrol.md) | `examples/end_to_end/safe_patrol/` | 30 min |

**Total:** ~3 hours at a comfortable pace.

---

## After Spanda 101

| Next step | Resource |
|-----------|----------|
| All tutorials | [Tutorials index](../tutorials/README.md) |
| Flagship safety demo | [killer-demo.md](../killer-demo.md) + `examples/showcase/killer_demo.sd` |
| Language reference | [spanda-language.md](../spanda-language.md) |
| Full API index | [spanda-reference.md](../spanda-reference.md) |
| Real-time & replay | [realtime.md](../realtime.md), [replay.md](../replay.md) |
| All examples | [examples/basics/README.md](../../examples/basics/README.md) |

---

## How to use these tutorials

1. Read the lesson.
2. Open the linked `.sd` file in your editor.
3. Run the commands in the **Try it** section.
4. Complete the **Exercise** (optional) before moving on.

All commands assume you are in the repository root and `spanda` is on your `PATH` (or use `./target/release/spanda`).
