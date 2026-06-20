# Spanda Vision

**Spanda is the Autonomous Systems Language.**

---

## The problem

Autonomous systems — robots, drones, industrial arms, warehouse fleets, humanoid assistants — are built from fragments:

- Python for AI and glue code
- C++ for drivers and real-time control
- ROS2 for communication
- Ad-hoc safety monitors
- Manual deployment checklists
- Simulation environments that diverge from production

No single language treats **perception, intelligence, safety, verification, and deployment** as unified first-class concepts. Teams spend more time integrating than innovating.

---

## The vision

Spanda is the language layer where autonomous systems are **designed, validated, simulated, and deployed** — with AI treated as untrusted input and hardware fit checked before code ships.

```
Sensors → Perception → AI Planning → Safety Gate → Actuators
                ↑                           ↑
           Digital Twin              Hardware Verify
```

We believe the next generation of robotics and autonomous agents needs a language that speaks their domain natively — not a general-purpose language with libraries bolted on.

---

## Focus areas

### AI

AI models are **advisors**, not drivers. LLMs and vision models propose `ActionProposal` values; only `safety.validate()` produces `SafeAction` that may reach actuators. Agents have goals, memory, and tools — but always within safety and capability constraints.

### Robotics

Sensors, actuators, trajectories, safety zones, and deterministic task scheduling are language primitives — not framework imports. Physical units (`m/s`, `rad`, `m`) are enforced at compile time.

### Digital Twins

Every physical robot can have a `twin` that mirrors pose, velocity, and sensor state. Replay buffers enable post-incident analysis and simulation sync. Live cloud twin telemetry is on the roadmap.

### Safety

Safety is not optional middleware. It is woven into the type system (`ActionProposal` vs `SafeAction`), runtime monitor (`stop_if`, zones, emergency stop), and verification (`verify { }` behavioral assertions).

### Verification

Before deployment, Spanda answers: *does this program fit this hardware?* Memory, sensors, timing, power, network, and AI model requirements are checked against hardware profiles via `spanda verify`.

### Human-Machine Interaction

Agents can interact with humans through structured communication primitives — topics, services, actions, and event handlers — with audit trails for accountability.

---

## Positioning

| Spanda is | Spanda is not |
|-----------|---------------|
| The coordination layer for autonomous systems | A replacement for Python, C++, or Rust |
| A safety-first AI orchestration language | A general-purpose application language |
| A deploy-time verification tool | Just another robotics framework |
| Open source and community-driven | A proprietary platform lock-in |

Spanda **orchestrates** existing ecosystems. Train models in Python. Write drivers in C++. Deploy coordination logic in Spanda.

---

## Long-term goals

1. **Production LLVM backend** — compiled native binaries for edge deployment
2. **Self-hosting compiler** — Spanda compiling Spanda
3. **Live ROS2 integration** — zero-config bridge to existing robot stacks
4. **Real AI provider ecosystem** — OpenAI, local models, ONNX, community packages
5. **Published VS Code extension** — full LSP experience out of the box
6. **Distributed fleet runtime** — multi-robot coordination at scale
7. **Digital twin cloud sync** — production telemetry and replay infrastructure

---

## v0.1.0-alpha milestone

The first public release establishes credibility:

- Stable interpreter and type system
- Mandatory AI safety gate
- Hardware compatibility verification
- Simulation backend
- Showcase examples and documentation
- CI/CD with Rust and TypeScript test suites
- Community contribution infrastructure

This is the foundation. The vision scales from a single rover to fleets of autonomous systems — all speaking Spanda.

---

## Join us

Spanda is open source under Apache-2.0. We welcome contributors, roboticists, safety engineers, and AI researchers.

- [CONTRIBUTING.md](../CONTRIBUTING.md)
- [GitHub Issues](https://github.com/sujaydavalgi/Spanda/issues)
- [feature-status.md](./feature-status.md)
