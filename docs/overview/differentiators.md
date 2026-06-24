# What makes Spanda different

[← Overview](./README.md)

## Core pillars

1. **Safety-typed AI** — `ActionProposal` from LLMs and vision cannot drive actuators; only `SafeAction` from `safety.validate()` can. Enforced at compile time and runtime.

2. **Hardware verification** — `deploy Robot to Profile` and `spanda verify` check sensors, memory, timing, power, and network before deployment.

3. **Capability verification** — Expose, grant, and trace robot capabilities; verify the system can perform the mission, not just compile.

4. **Simulation + replay** — `spanda sim` before hardware exists; `spanda replay` for regression and incident review.

5. **Health-aware runtime** — `health_check`, fleet `require` clauses, and policies during operation.

6. **Package-based extensibility** — Lean core; official packages (ROS2, MQTT, GPS, vision, fleet, mission assurance) via the provider registry.

## Capability matrix

| Differentiator | What it means |
|----------------|---------------|
| **AI safety gate** | `ActionProposal` cannot drive actuators; only `SafeAction` from `safety.validate()` |
| **Hardware verification** | `deploy` + `spanda verify` — sensors, memory, timing, power |
| **Physical units** | `1.0 m/s`, `0.5 rad`, `100 ms` — unit algebra at compile time |
| **Robot-native syntax** | Sensors, actuators, topics, safety zones, tasks as keywords |
| **Deterministic scheduling** | `task every 50ms` with optional `budget { }` |
| **Real-time contracts** | `deadline`, `jitter <=`, `priority`, `critical isolated`; `pipeline` budgets |
| **Reliability primitives** | Watchdogs, `mode` blocks, `recover from`, retry/fallback |
| **Mission trace replay** | `--record`, `spanda replay --deterministic` / `--playback` |
| **First-class regex** | Literals, triggers, subscription filters, `validate` rules |
| **Trigger-driven execution** | `on` / `every` / `when` / `while` for events, safety, AI |
| **Cooperative concurrency** | `spawn`, `join`, `parallel`, channels, `select` |
| **Simulation built in** | `spanda run` / `spanda sim` without hardware |
| **Digital twins** | `twin { mirror …; replay true; }` |
| **Platform packages** | **37** hosted packages; `spanda install`, provider dispatch |
| **Mission assurance** | `knowledge_model`, `state_estimator`, `anomaly_detector`, …; `spanda demo assurance` |
| **Weighted sensor fusion** | `observe { }`, `state_estimator`, `fusion.read()`; `spanda-fusion` package |
| **Learned anomaly detection** | `learned backend assurance.anomaly`; optional ONNX |
| **World models** | `world_model { }` + `fusion.read()` belief hook |
| **Verification & DX** | Traceability, kill switch, health policies, typed handler I/O |
| **Live providers (optional)** | OpenAI, Anthropic, ONNX; IoT live bridges; mock fallback |
| **Package registry** | Ed25519-signed tarballs; `SPANDA_REGISTRY_URL` override |

Honest status tiers: [feature-status.md](../feature-status.md) · Lean-core phases: [lean-core-roadmap.md](../lean-core-roadmap.md)
