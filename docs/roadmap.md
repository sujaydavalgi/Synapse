# Spanda Roadmap

Version plan organized by **platform area**. Tiers: **Stable** (CI-backed, documented), **Experimental** (usable with caveats), **Future** (planned, not shipped).

Current release line: **v0.4.0**.

Platform overview: [platform-overview.md](./platform-overview.md) · Feature truth table: [feature-status.md](./feature-status.md)

---

## Platform areas at a glance

| Area | Current focus (v0.4) | Next |
|------|----------------------|------|
| [Language](#language) | Stable core; typed handler I/O | Generics polish, self-hosting subset (future) |
| [Runtime](#runtime) | Interpreter LTS; certify gate | Native codegen golden paths |
| [Verification](#verification) | `spanda verify`, capability matrices | 5+ production hardware profiles (v1.0) |
| [Safety](#safety) | ActionProposal → SafeAction stable | Stricter certify / ISO metadata workflows |
| [Simulation](#simulation) | `spanda sim`, twins, fault injection | Deeper package bridges (Gazebo/Webots scaffolds) |
| [Health](#health) | health_check, fleet require | Swarm quorum hardening |
| [Fleet](#fleet) | In-process + HTTP agents | Distributed orchestration polish |
| [Packages](#packages) | 37 official registry packages, publish mirror | Curated remote registry growth |
| [Tooling](#tooling) | CLI, demos, CI golden paths | VS Code Marketplace, LSP polish |
| [Mission assurance](#mission-assurance) | Lean-core analysis + CLI | Package-backed ML anomaly backends |
| [Platform maturity](#platform-maturity) | Design specs + phased plan | P0: graph, explain, gates, package trust |

---

## Platform maturity

**Adoption, trust, and operations** — not new unrelated features. Every item strengthens **Build · Verify · Simulate · Deploy · Operate · Recover**.

Full analysis: [platform-maturity-roadmap.md](./platform-maturity-roadmap.md)

| # | Area | Phase | Priority | Status |
|---|------|-------|----------|--------|
| 1 | AI-assisted development (`generate`, `explain`, `suggest`) | Build, Operate | P0.3 / P3.3 | **Future** |
| 2 | Dependency graph visualization | Build, Operate | P0.1 | **Planned** |
| 3 | Threat modeling | Verify, Deploy | P1.2 | **Planned** |
| 4 | Configuration drift detection | Deploy, Operate | P1.1 | **Planned** |
| 5 | Policy engine | Verify, Operate | P1.5 | **Planned** |
| 6 | Compliance profiles | Verify, Deploy | P2.4 | **Future** |
| 7 | Explainability reports | Operate, Recover | P0.3 / P3.2 | **Planned** |
| 8 | Chaos engineering | Simulate, Recover | P2.1 | **Planned** |
| 9 | Mission resource estimation | Simulate, Deploy | P2.3 | **Planned** |
| 10 | Readiness trend analysis | Operate | P2.2 | **Planned** |
| 11 | Package trust framework | Verify, Build | P0.4 | **Planned** |
| 12 | Architecture decision records | Build | P2.5 | **Planned** |
| 13 | Mission differencing | Build, Verify | P1.3 | **Planned** |
| 14 | Deployment gates | Deploy | P0.2 | **Planned** |
| 15 | Autonomous systems scorecard | Operate | P1.4 | **Planned** |
| 16 | Hack / tamper detection | Verify, Operate, Recover | P3.1 | **Future** |

### Phased delivery

| Phase | Release | Theme | Key deliverables |
|-------|---------|-------|------------------|
| A | v0.5+ (Q3–Q4 2026) | Understand & trust | `spanda graph`, `explain`, `package trust`, deployment gates |
| B | v0.6 (Q1 2027) | Operate & compare | drift, threat model, mission diff, scorecard, policy (verify) |
| C | v0.7 (Q2 2027) | Resilience & planning | chaos, readiness trends, estimate, compliance profiles, ADR |
| D | v1.0 (2027) | Full trust platform | tamper/integrity, explainability traces, AI generate (guardrailed) |

Topic guides: [dependency-graphs.md](./dependency-graphs.md) · [deployment-gates.md](./deployment-gates.md) · [tamper-detection.md](./tamper-detection.md) · [security-assurance.md](./security-assurance.md)

---

## Mission assurance

**Mission-grade autonomous operations** — knowledge models, state estimation, anomaly detection, prognostics, mitigation, resilience, assurance cases.

| Item | Status |
|------|--------|
| `spanda-assurance` crate (static analysis) | **Stable** |
| Language declarations (`knowledge_model`, `state_estimator`, `anomaly_detector`, …) | **Stable** |
| CLI (`assure`, `anomaly scan`, `state estimate`, `prognostics`, `mission verify`, `resilience check`, `mitigation plan`) | **Stable** |
| Runtime `state_estimator` → weighted fusion bindings | **Experimental** |
| Learned anomaly backends (`learned backend`, `spanda-anomaly`) | **Experimental** — runtime `scan_learned` + EMA volatility + optional ONNX (`SPANDA_ANOMALY_ONNX_MODEL_PATH`) |
| Weighted fusion package (`spanda-fusion`, `assurance.fusion`) | **Experimental** — provider dispatch for fusion weights |
| Full ML inference (custom ONNX architectures) | **Future** — beyond 2-feature anomaly scaffold |

See [mission-assurance.md](./mission-assurance.md), [state-estimation.md](./state-estimation.md).

---

## Self-healing & recovery

**Safety-first recovery** — `recovery_policy`, validation gates, knowledge store, runtime dispatch, fleet mesh relay.

| Item | Status |
|------|--------|
| `recovery_policy` syntax + `RecoveryPlanner` | **Stable** |
| CLI (`heal`, `recover`, `recovery-report`, `recovery knowledge`, `sim --inject-failure`) | **Stable** |
| Recovery diagnostics (`spanda check --readiness-json`) | **Stable** |
| Runtime dispatch (modes, speed caps, connectivity, mission pause) | **Experimental** |
| Operator approval (env, Approval topics, mission `requires approval`) | **Experimental** |
| Fleet mesh recovery (`POST /v1/fleet/recovery`, `SPANDA_FLEET_MESH_URL`) | **Experimental** |
| Recovery reassign → continuity mesh relay | **Experimental** |
| Fleet agent assurance recovery (`POST /v1/recovery/execute`, deployed program) | **Experimental** |
| Fleet agent interpreter recovery (`execute_recovery_on_program`, `recovery_engine`) | **Experimental** |
| TypeScript recovery diagnostics (LSP fallback) | **Stable** |
| `spanda demo self-healing` | **Stable** |

See [self-healing.md](./self-healing.md), [recovery-policies.md](./recovery-policies.md).

---

## Mission continuity

**Mission continuity, delegation, takeover, and succession** — checkpoint resume, state transfer, successor ranking, safety-gated takeover.

| Item | Status |
|------|--------|
| Continuity framework (`MissionContinuityManager`, `TakeoverCoordinator`, `SuccessionPlanner`, …) | **Stable** |
| Takeover modes (resume, restart, partial, shadow, hot, cold, human) | **Stable** |
| State transfer (`MissionStateSnapshot`, `MissionContextTransfer`) | **Stable** |
| CLI (`continuity`, `takeover`, `delegate`, `succession`) | **Stable** |
| Continuity diagnostics (`spanda check --readiness-json`) | **Stable** |
| TypeScript continuity diagnostics (LSP fallback) | **Stable** |
| `spanda demo continuity` + showcase examples | **Stable** |
| Official package `spanda-mission-continuity` (`assurance.continuity`) | **Stable** |
| Runtime takeover dispatch on fleet agents | **Experimental** |
| Recovery reassign → continuity mesh relay | **Experimental** |
| Language `continuity_policy` declarations | **Experimental** |

See [mission-continuity.md](./mission-continuity.md).

---

## Language

**Spanda Language (`.sd`)** — syntax, types, robot primitives, units, safety types.

| Item | Status |
|------|--------|
| Lexer, parser, AST, type checker | **Stable** |
| Physical units, `module`/`import`, structs/enums/traits | **Stable** |
| Robot primitives (`robot`, `sensor`, `actuator`, `task`, `agent`) | **Stable** |
| Trigger model (`on`, `every`, `when`, `while`) | **Stable** |
| Typed handler return types | **Stable** |
| Regex literals and validation rules | **Stable** |
| Self-hosting compiler subset | **Future** |
| LLVM as language execution path | **Experimental** — see [compiler-backend-roadmap.md](./compiler-backend-roadmap.md) |

Foundation: Phases 1–35 complete — [lean-core-roadmap.md](./lean-core-roadmap.md)

---

## Runtime

**Spanda Runtime** — interpreter, scheduler, HAL, concurrency, provider dispatch.

| Item | Status |
|------|--------|
| Tree-walking interpreter (primary path) | **Stable** |
| Cooperative concurrency (`spawn`, `join`, `select`) | **Stable** |
| Trigger-driven scheduler + telemetry flags | **Stable** |
| `spanda-certify` runtime gate | **Stable** |
| Real-time contracts (`deadline`, `jitter`, `priority`) | **Stable** |
| Reliability (watchdogs, modes, `recover from`) | **Stable** |
| World model / fusion belief hook | **Experimental** |
| Native binary via LLVM | **Experimental** — `spanda deploy --target native`, [native-deploy.md](./native-deploy.md) |

---

## Verification

**Spanda Verify** — hardware, capability, and behavioral verification.

| Item | Status |
|------|--------|
| `spanda verify` (profiles, `--simulate`, `--json`) | **Stable** |
| `deploy`, `requires_hardware`, hardware profiles | **Stable** |
| Behavioral `verify { }` assertions | **Stable** |
| Capability traceability matrices | **Stable** |
| `spanda check --verification-json` + LSP quick-fixes | **Stable** |
| CI integration guide | **Stable** — [ci-verify.md](./ci-verify.md) |
| Production verify on 5+ profiles | **Future** (v1.0) |
| Hardware adapter trait codegen | **Future** |

---

## Safety

**Spanda Safety** — type-level and runtime safety engine.

| Item | Status |
|------|--------|
| `ActionProposal` → `SafeAction` compile-time gate | **Stable** |
| `safety { }` zones, `max_speed`, `stop_if` | **Stable** |
| Kill switch + `remote_signed` handlers | **Stable** |
| Agent `can[]` capability clarity | **Stable** |
| Certification metadata (`certify`, `spanda certify prove`) | **Experimental** |
| Minimum-hardware safety analysis | **Stable** |

---

## Simulation

**Spanda Sim** and **Spanda Replay** — test and regress without hardware.

| Item | Status |
|------|--------|
| `spanda run` / `spanda sim` (physics-lite) | **Stable** |
| Digital twins (`twin`, mirror, replay buffer) | **Stable** |
| `simulate_compatibility` fault injection | **Stable** |
| Mission trace `--record` | **Stable** |
| `spanda replay` (`--deterministic`, `--playback`) | **Stable** |
| Wall-clock sim mode | **Stable** — [realtime.md](./realtime.md), [replay.md](./replay.md) |
| Twin cloud SaaS | **Future** |
| Full physics (Gazebo/Isaac class) | **Out of scope** — package bridges only |

---

## Health

**Spanda Health** — operational readiness and fleet policies.

| Item | Status |
|------|--------|
| `health_check`, `health_policy` | **Stable** |
| Fleet `require` clauses at runtime | **Stable** |
| `spanda demo health` showcase | **Stable** |
| Operational readiness engine (`spanda readiness`) | **Stable** — [readiness.md](./readiness.md) |
| Mission verification, failure analysis, safety reports | **Stable** — see readiness docs |
| Swarm quorum / mesh health | **Experimental** — [swarm-health.md](./swarm-health.md) |

---

## Fleet

**Spanda Fleet** — multi-robot simulation and distributed coordination.

| Item | Status |
|------|--------|
| `spanda fleet run` (in-process) | **Stable** |
| Fleet orchestrate (round-robin report) | **Stable** |
| HTTP fleet agents + `--remote` | **Experimental** — [fleet-distributed.md](./fleet-distributed.md) |
| Fleet mesh coordinator | **Experimental** |
| OTA deploy plan / rollout / rollback | **Stable** (local state) / remote **Experimental** |
| ROS2 rclpy golden path | **Experimental** — [ros2-golden-path.md](./ros2-golden-path.md) |
| `spanda ros2 check` | **Stable** |

---

## Packages

**Spanda Registry** and **Spanda Providers** — extensibility without bloating the core.

| Item | Status |
|------|--------|
| `spanda install` / `update` / `publish` | **Stable** |
| Hosted registry index (38 packages) | **Stable** — [registry.md](./registry.md) |
| Provider dispatch + `--trace-providers` | **Stable** |
| Official packages (ROS2, MQTT, GPS, vision, …) | **Stable** scaffolds / live **Experimental** |
| Live AI providers (OpenAI, Anthropic, ONNX) | **Experimental** — [live-ai-provider.md](./live-ai-provider.md) |
| Live IoT / MQTT bridges | **Experimental** |
| Blockchain / ledger adapters | **Future** (community packages only) |

---

## Tooling

CLI, LSP, debugger, docs site, and contributor ergonomics.

| Item | Status |
|------|--------|
| Native CLI (`check`, `verify`, `run`, `sim`, `fleet`, `fmt`, `lint`) | **Stable** |
| `cargo install spanda` | **Stable** |
| Bundled `spanda demo {rover,safety,verify,fleet,health,readiness,assurance,self-healing,continuity}` | **Stable** |
| Operations dashboard (`packages/web` Operations view) | **Experimental** |
| mdBook GitHub Pages | **Stable** |
| LSP hover + SafeAction quick-fix | **Stable** |
| VS Code snippets + VSIX CI | **Stable** |
| VS Code Marketplace listing | **Partial** — pending `VSCE_PAT` |
| DAP debugger | **Experimental** — [debugging.md](./debugging.md) |
| WASM web playground | **Experimental** |

---

## Release milestones

### v0.4 — Deploy path (current)

**Theme:** Native binaries, ROS2 polish, distributed fleet docs.

| Item | Status |
|------|--------|
| `spanda deploy --target native` | **Experimental** |
| `spanda compile-native` / LLVM golden paths | **Experimental** |
| `spanda ros2 check` | **Stable** |
| Distributed fleet guide | **Stable** |

### v0.3 — Tooling polish (complete)

**Theme:** IDE, diagnostics, registry, install ergonomics.

| Item | Status |
|------|--------|
| Crate rename → `spanda`, bundled demos | **Stable** |
| Hosted registry (38 packages) | **Stable** |
| LSP + showcase CI smoke | **Stable** |

### v0.2 — Credibility & onboarding (complete)

**Theme:** Trust table, showcase demos, docs site, one-command demos.

| Item | Status |
|------|--------|
| Feature status audit + `spanda demo` | **Stable** |
| mdBook GitHub Pages | **Stable** |

### v1.0 — Production positioning

**Theme:** Trust for field deployment.

| Item | Tier |
|------|------|
| Interpreter + sim as supported LTS runtime | Stable |
| Safety + verify + replay as certified workflows | Stable |
| Native codegen for selected HAL profiles | Experimental → Stable |
| Self-hosting compiler subset | Future (not primary) |
| Blockchain / cryptocurrency adapters | **Out of scope** |
| Advanced swarm intelligence research | **Out of scope** |

---

## Related

- [platform-maturity-roadmap.md](./platform-maturity-roadmap.md) — adoption, trust, operations expansion (16 areas)
- [platform-overview.md](./platform-overview.md)
- [feature-status.md](./feature-status.md)
- [product-strategy.md](./product-strategy.md)
- [compiler-backend-roadmap.md](./compiler-backend-roadmap.md)
- [lean-core-roadmap.md](./lean-core-roadmap.md)
- [roadmap-codebase-audit-2026-06.md](./roadmap-codebase-audit-2026-06.md)
