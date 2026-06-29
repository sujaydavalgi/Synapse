# Spanda Documentation

<p align="center">
  <img src="../assets/image/low_res_logo.png" alt="Spanda logo" width="240">
</p>

**Spanda is an Autonomous Systems Platform with a safety-first programming language at its core.** Source files use the `.sd` extension.

*Pronounced **SPUN-duh** (/ˈspʌndə/)* — Sanskrit for *the divine pulse*; see [overview/philosophy.md](./overview/philosophy.md).

**Start here:** [platform-overview.md](./platform-overview.md) — platform vs language, component map, and workflow.

**Product roadmap:** [../ROADMAP.md](../ROADMAP.md) — Platform Pillars, Official Solution Blueprints, ownership model, timeline.

**Platform pillars (navigation hubs):** [pillars/README.md](./pillars/README.md)

**Project home:** [../README.md](../README.md) — quick start and links to overview subpages.

**Expanded overview:** [overview/README.md](./overview/README.md) — flagship demos, audience paths, platform map, feature snapshot, philosophy, differentiators, CLI, code samples.

## Tutorials

**[Tutorials index](./tutorials/README.md)** — all learning paths: For Dummies, Spanda 101, topic guides, walkthroughs, and example libraries.

## Guides

| Document | Description |
|----------|-------------|
| [platform-overview.md](./platform-overview.md) | **Spanda Platform — components, workflow, platform vs language** |
| [platform-positioning-migration.md](./platform-positioning-migration.md) | **Messaging migration — taglines, GitHub metadata, branding** |
| [tutorials/README.md](./tutorials/README.md) | **Master index — all tutorials, walkthroughs, and example paths** |
| [../examples/README.md](../examples/README.md) | **Runnable examples library — ladder, topics, CI** |
| [../README.md](../README.md) | Project landing page (links to overview subpages) |
| [overview/README.md](./overview/README.md) | **Project overview subpages** (platform, architecture, CLI, library, packages, demos) |
| [getting-started.md](./getting-started.md) | **First robot in 10 minutes** |
| [spanda-for-dummies/README.md](./spanda-for-dummies/README.md) | **Spanda for Dummies — plain-English no-jargon guide** |
| [spanda-101/README.md](./spanda-101/README.md) | **Spanda 101 — 10-lesson tutorial series (basics → end-to-end)** |
| [installation.md](./installation.md) | **Prebuilt packages for Linux, macOS, and Windows** |
| [architecture.md](./architecture.md) | **Compiler pipeline with diagrams** |
| [platform-architecture.md](./platform-architecture.md) | **Platform Architecture v2.0 — layers, governance, validation** |
| [layered-architecture.md](./layered-architecture.md) | **Official layer stack and crate mapping** |
| [dependency-rules.md](./dependency-rules.md) | **Dependency rules, waiver process, anti-patterns** |
| [module-ownership.md](./module-ownership.md) | **Module ownership matrix** |
| [platform-services.md](./platform-services.md) | **Platform service boundaries** |
| [event-model.md](./event-model.md) | **Common event model** |
| [design-principles.md](./design-principles.md) | **Architectural design principles** |
| [architecture-waiver-burn-down.md](./architecture-waiver-burn-down.md) | **Completed waiver burn-down (Phases 1–8; 0 waivers)** |
| [lean-core.md](./lean-core.md) | **Lean-core workspace architecture (Phases 1–17)** |
| [crates/README.md](../crates/README.md) | **Workspace crate index and dependency rules** |
| [lean-core-roadmap.md](./lean-core-roadmap.md) | **Phased plan — Phases 1–35 complete; crate extraction and verification/DX** |
| [phase-18-security-hardening.md](./phase-18-security-hardening.md) | **Post–Phase 17 security/stability/performance hardening** |
| [tier-3-experimental.md](./tier-3-experimental.md) | **Tier 3 experimental foundations (Phase 22–23)** |
| [tier-3-golden-paths.md](./tier-3-golden-paths.md) | **Tier 3 CI golden paths — scripts, jobs, feature flags** |
| [tier-3-priority-plan.md](./tier-3-priority-plan.md) | **Priority plan: beta → experimental hardening → v1.0 → production Tier 3** |
| [provider-interfaces.md](./provider-interfaces.md) | **Provider trait contracts for packages** |
| [official-packages.md](./official-packages.md) | **Official package catalog** |
| [how-packages-work.md](./how-packages-work.md) | **Package loading pipeline, provenance, and CLI workflow** |
| [configuration.md](./configuration.md) | **Cascading TOML configuration and ResolvedSystemConfig** |
| [entity-model.md](./entity-model.md) | **Unified Entity Model — foundational platform abstraction** |
| [entity-overview.md](./entity-overview.md) | **Entity model overview and documentation map** |
| [entity-apis.md](./entity-apis.md) | **Entity REST and gRPC API reference** |
| [entity-sdk.md](./entity-sdk.md) | **Entity SDK — Rust, TypeScript, Python** |
| [entity-model-stable-promotion.md](./entity-model-stable-promotion.md) | **Entity model Experimental → Stable promotion** |
| [entity-registry.md](./entity-registry.md) | **Entity registry — inventory, lookup, filtering** |
| [entity-graph.md](./entity-graph.md) | **Entity graph — traversal, impact, visualization** |
| [entity-relationships.md](./entity-relationships.md) | **Entity relationship taxonomy** |
| [entity-query-language.md](./entity-query-language.md) | **Entity query language — REST and JSON filters** |
| [entity-verification.md](./entity-verification.md) | **Entity verification — unified verify(entity) engine** |
| [entity-readiness.md](./entity-readiness.md) | **Entity readiness evaluation** |
| [entity-health.md](./entity-health.md) | **Entity health diagnostics** |
| [entity-trust.md](./entity-trust.md) | **Entity trust evaluation** |
| [entity-best-practices.md](./entity-best-practices.md) | **Entity model best practices** |
| [entity-migration-guide.md](./entity-migration-guide.md) | **Entity model migration guide** |
| [entity-integration-report.md](./entity-integration-report.md) | **Entity model integration report (Phase 2)** |
| [cascading-config.md](./cascading-config.md) | **Layered config overrides (base → environment → deployment → robot)** |
| [device-tree.md](./device-tree.md) | **Fleet/device hierarchy and device identity in TOML** |
| [config-validation.md](./config-validation.md) | **Configuration validation rules and CLI** |
| [spanda-toml.md](./spanda-toml.md) | **`spanda.toml` manifest reference (package + system config)** |
| [how-providers-work.md](./how-providers-work.md) | **Provider registry, traits, and dispatch** |
| [how-runtime-resolution-works.md](./how-runtime-resolution-works.md) | **Runtime resolution from imports to providers** |
| [security-architecture.md](./security-architecture.md) | **Security contracts vs package backends** |
| [triggers.md](./triggers.md) | **Unified trigger-driven execution** (`on`, `every`, `when`, safety, state, AI) |
| [concurrency.md](./concurrency.md) | **Tasks, spawn, channels, fleet CLI, and runtime telemetry** |
| [realtime.md](./realtime.md) | **Deadline-aware tasks, jitter bounds, wall-clock scheduling** |
| [reliability.md](./reliability.md) | **Pipelines, watchdogs, recovery, retry/fallback, operating modes** |
| [watchdogs.md](./watchdogs.md) | Task heartbeats and timeout handling |
| [runtime-fault-detection.md](./runtime-fault-detection.md) | **Runtime fault detection: crashes, reboots, memory leaks, resource pressure** |
| [crash-detection.md](./crash-detection.md) | Process/provider crash detection and recovery |
| [reboot-detection.md](./reboot-detection.md) | Unexpected reboot detection and post-reboot diagnostics |
| [memory-leak-detection.md](./memory-leak-detection.md) | Memory growth monitoring and leak events |
| [runtime-health.md](./runtime-health.md) | Runtime health status and CLI reporting |
| [degraded-modes.md](./degraded-modes.md) | Operating `mode` blocks and graceful degradation |
| [replay.md](./replay.md) | **Mission trace record, deterministic replay, frame playback** |
| [regex.md](./regex.md) | **First-class regex literals, triggers, and validation rules** |
| [vision.md](./vision.md) | Long-term vision and positioning |
| [product-strategy.md](./product-strategy.md) | **Product strategy, priorities, v0.5 beta scope, killer demo** |
| [killer-demo.md](./killer-demo.md) | **Flagship demo: safety-typed AI, verify, and sim (5 min)** |
| [adoption-path.md](./adoption-path.md) | **One-sprint adoption: wrap Python + ROS2, CI, one extern call** |
| [ci-verify.md](./ci-verify.md) | **`spanda verify` in GitHub Actions and GitLab CI (`--json`)** |
| [ros2-golden-path.md](./ros2-golden-path.md) | **ROS2 interop golden path (rclpy bridge, `/cmd_vel` / `/scan`)** |
| [mqtt-nav2-reference-architecture.md](./mqtt-nav2-reference-architecture.md) | **MQTT + Nav2 + ROS2 reference stack for field robots** |
| [llvm-embedded-benchmark.md](./llvm-embedded-benchmark.md) | **LLVM aarch64 cross-compile slice (Jetson / Pi)** |
| [live-ai-provider.md](./live-ai-provider.md) | **Live OpenAI, Anthropic, and ONNX paths via Python bridge** |
| [debugging.md](./debugging.md) | **Debug `behavior`, `task every`, and `every` triggers in VS Code (DAP)** |
| [health-checks.md](./health-checks.md) | **Health checks, fleet `require` clauses, and policies** |
| [readiness.md](./readiness.md) | **Operational readiness engine and weighted go/no-go scoring** |
| [mission-assurance.md](./mission-assurance.md) | **Mission assurance domains, CLI, packages, and examples** |
| [state-estimation.md](./state-estimation.md) | **State estimators, weighted fusion, and `spanda state estimate`** |
| [anomaly-detection.md](./anomaly-detection.md) | **Anomaly detectors, learned backends, ONNX inference** |
| [knowledge-models.md](./knowledge-models.md) | **System knowledge models and dependency graphs** |
| [diagnostics.md](./diagnostics.md) | **Fault diagnosis and `spanda diagnose`** |
| [prognostics.md](./prognostics.md) | **Prognostics, RUL, and degradation warnings** |
| [resilience.md](./resilience.md) | **Resilience policies and recovery** |
| [self-healing.md](./self-healing.md) | **Self-healing framework, recovery workflow, fleet mesh** |
| [self-correction.md](./self-correction.md) | **Self-correction actions and safety gates** |
| [recovery-planning.md](./recovery-planning.md) | **Recovery planner and failure classification** |
| [recovery-assurance.md](./recovery-assurance.md) | **Recovery evidence and assurance integration** |
| [recovery-policies.md](./recovery-policies.md) | **`recovery_policy` syntax and operating modes** |
| [continuity-policies.md](./continuity-policies.md) | **`continuity_policy` syntax, takeover modes, and fleet handoff** |
| [mission-continuity.md](./mission-continuity.md) | **Mission continuity, takeover, delegation, and succession** |
| [assurance-cases.md](./assurance-cases.md) | **Assurance cases and evidence linking** |
| [mission-verification.md](./mission-verification.md) | **Mission achievability and approval verification** |
| [failure-analysis.md](./failure-analysis.md) | **Component failure impact and mitigations** |
| [safety-reporting.md](./safety-reporting.md) | **Deployable safety case reports (JSON/Markdown/HTML)** |
| [fleet-readiness.md](./fleet-readiness.md) | **Fleet readiness and multi-robot verification** |
| [root-cause-analysis.md](./root-cause-analysis.md) | **Replay-based failure diagnosis** |
| [safety-auditor.md](./safety-auditor.md) | **Autonomous safety auditor** |
| [kill-switch.md](./kill-switch.md) | **Kill switch syntax, `remote_signed`, and `on kill_switch` handlers** |
| [iot.md](./iot.md) | **IoT provider contracts, package dispatch, live bridge env flags** |
| [telemetry-store.md](./telemetry-store.md) | **Persistent device/sensor/heartbeat storage (`spanda telemetry`)** |
| [capability-traceability.md](./capability-traceability.md) | **Capability exposure and traceability matrices** |
| [verification-diagnostics.md](./verification-diagnostics.md) | **`spanda check --verification-json`, LSP quick-fixes, kill-switch severity** |
| [typed-handler-io.md](./typed-handler-io.md) | **Return types on behavior, task, trigger, event, and agent plan handlers** |
| [testing.md](./testing.md) | **`expect_compile_error`, `spanda test --json`, compile-fail tests** |
| [agentic-programming.md](./agentic-programming.md) | **Safety-gated agents, `can[]`, audit hooks** |
| [fleet-health.md](./fleet-health.md) | **Fleet health `require` clauses and runtime evaluation** |
| [swarm-health.md](./swarm-health.md) | **Swarm quorum and mesh health checks** |
| [minimum-hardware-safety.md](./minimum-hardware-safety.md) | **Minimum-hardware safety analysis** |
| [hardware-capabilities.md](./hardware-capabilities.md) | **Hardware capability exposure** |
| [robot-capabilities.md](./robot-capabilities.md) | **Robot `exposes capabilities` and mission grants** |
| [hardware-traceability.md](./hardware-traceability.md) | **Hardware-to-code traceability mapping** |
| [packages.md](./packages.md) | **Package manager, `spanda publish`, capabilities** |
| [registry.md](./registry.md) | **Hosted package registry, `spanda publish` mirror, and `spanda install`** |
| [feature-status.md](./feature-status.md) | **v0.4 support matrix** |
| [roadmap-codebase-audit-2026-06.md](./roadmap-codebase-audit-2026-06.md) | **June 2026 roadmap vs codebase gap audit and closure tracking** |
| [hardware-compatibility.md](./hardware-compatibility.md) | **Hardware profiles, deploy targets, and compile-time verification** |
| [positioning.md](./positioning.md) | **GPS/GNSS types, sensors, and simulation faults** |
| [connectivity.md](./connectivity.md) | **Wi-Fi, LTE, failover policies, and offline modes** |
| [geofencing.md](./geofencing.md) | **WGS84 geofences and safety triggers** |
| [bluetooth.md](./bluetooth.md) | **Bluetooth discovery, pairing, and BLE services** |
| [cellular.md](./cellular.md) | **LTE/4G/5G hardware and roaming** |
| [spanda-architecture.md](./spanda-architecture.md) | Architecture diagram, compiler pipeline, safety model |
| [spanda-language.md](./spanda-language.md) | Language reference for modules, traits, tasks, twins, hardware |
| [language-reference/](./language-reference/README.md) | **Language reference topics** — syntax, types, agents, safety, packages |
| [spanda-reference.md](./spanda-reference.md) | **Spanda language API** — keywords, `std.*`, builtins, CLI (JavaDoc + man style) |
| [api-documentation.md](./api-documentation.md) | **API doc hierarchy** — language vs compiler vs JSON contract |
| [api-reference.md](./api-reference.md) | **Rust/TypeScript compiler API** — crates grouped by lean-core layer |
| [control-center-api.md](./control-center-api.md) | **Control Center REST v1** — SDK program ops, entity registry, auth |
| [sdk.md](./sdk.md) | **Official SDKs** — Rust, Python, TypeScript overview |
| [sdk-rust.md](./sdk-rust.md) | Rust SDK (`spanda-sdk`) |
| [sdk-python.md](./sdk-python.md) | Python SDK (`spanda-sdk`) |
| [sdk-typescript.md](./sdk-typescript.md) | TypeScript SDK (`@davalgi-spanda/sdk`) |
| [sdk-publishing.md](./sdk-publishing.md) | **Publishing SDKs** — PyPI/npm tokens, GitHub secrets, release tags |
| [api-contract.json](./api-contract.json) | JSON schema for diagnostics, run results, and verify output |
| [standard-library.md](./standard-library.md) | Standard library overview and design |
| [robotics-platform.md](./robotics-platform.md) | **Robotics platform: missions, fleet, safety zones, navigation, fusion, package strategy** |
| [spanda-type-system.md](./spanda-type-system.md) | Type system: units, generics, AI/safety types |
| [man/](./man/) | Man-page style CLI reference |
| [roadmap.md](./roadmap.md) | Redirect to [ROADMAP.md](../ROADMAP.md) — product roadmap |
| [roadmap-migration.md](./roadmap-migration.md) | **Roadmap restructure migration notes** |
| [repository-organization.md](./repository-organization.md) | **Docs/examples/packages layout recommendations** |
| [../ROADMAP.md](../ROADMAP.md) | **Canonical product roadmap** — pillars, blueprints, timeline |
| [differentiation-roadmap.md](./differentiation-roadmap.md) | **Signature capabilities — mission contracts, explainability, coverage, risk (15 areas)** |
| [platform-maturity-roadmap.md](./platform-maturity-roadmap.md) | **Platform maturity expansion — adoption, trust, operations (16 areas)** |
| [enterprise-operations-roadmap.md](./enterprise-operations-roadmap.md) | **Enterprise operations — Control Center, Device Pool, RBAC, APIs, observability (20 pillars)** |
| [stable-hardening-enterprise-ops.md](./stable-hardening-enterprise-ops.md) | **Experimental → Stable promotion checklist for enterprise operations** |
| [enterprise-ops-stable-promotion.md](./enterprise-ops-stable-promotion.md) | **Stable promotion runbook** (soak, audit prep, promotion gate) |
| [stable-hardening-human-interaction.md](./stable-hardening-human-interaction.md) | **Experimental → Stable promotion checklist for Human Interaction (H1–H4)** |
| [stable-hardening-adas.md](./stable-hardening-adas.md) | **Experimental → Stable promotion checklist for ADAS Solution Blueprint** |
| [field-soak-gate.md](./field-soak-gate.md) | **30-day field pilot gate before Stable promotion** |
| [security-audit-third-party.md](./security-audit-third-party.md) | **Third-party security audit scope and prep workflow** |
| [desktop-release-runbook.md](./desktop-release-runbook.md) | **Signed/notarized desktop builds and auto-update** |
| [control-center.md](./control-center.md) | **Control Center — `spanda control-center serve`, REST v1, Tauri desktop (`@spanda/control-center-desktop`)** |
| [mission-contracts.md](./mission-contracts.md) | **Mission Contracts — guarantees, constraints, assumptions** |
| [decision-audit-trail.md](./decision-audit-trail.md) | **Decision audit trail — mission → decision → evidence → action** |
| [safety-coverage.md](./safety-coverage.md) | **Safety scenario coverage reporting** |
| [recovery-coverage.md](./recovery-coverage.md) | **Recovery path coverage reporting** |
| [what-if-analysis.md](./what-if-analysis.md) | **What-if failure scenario analysis** |
| [mission-risk-analysis.md](./mission-risk-analysis.md) | **Mission deployment risk scoring** |
| [digital-mission-twin.md](./digital-mission-twin.md) | **Digital mission twin — progress, health, risks** |
| [certification-packs.md](./certification-packs.md) | **Deployment evidence bundles** |
| [mission-time-travel.md](./mission-time-travel.md) | **Replay mission state at any timestamp** |
| [human-robot-teaming.md](./human-robot-teaming.md) | **Human approval, escalation, override paths** |
| [dependency-graphs.md](./dependency-graphs.md) | **Dependency graph visualization (`spanda graph`)** |
| [deployment-gates.md](./deployment-gates.md) | **Deployment gates — block unsafe rollout (production provenance + signatures)** |
| [explainability.md](./explainability.md) | **`spanda explain` — code, missions, failures, decisions** |
| [drift-detection.md](./drift-detection.md) | **Configuration drift — expected vs actual** |
| [threat-modeling.md](./threat-modeling.md) | **Pre-deploy threat modeling** |
| [mission-diff.md](./mission-diff.md) | **Mission differencing and change-impact** |
| [policy-engine.md](./policy-engine.md) | **Declarative operational policies** |
| [compliance-profiles.md](./compliance-profiles.md) | **Industry verification profile templates** |
| [solutions/README.md](./solutions/README.md) | **Official Solution Blueprints index** |
| [solutions/warehouse.md](./solutions/warehouse.md) | **Warehouse Automation blueprint** |
| [solutions/smart-factory.md](./solutions/smart-factory.md) | **Smart Factory blueprint** |
| [solutions/defense.md](./solutions/defense.md) | **Defense blueprint** |
| [solutions/environmental-monitoring.md](./solutions/environmental-monitoring.md) | **Environmental monitoring blueprint (planned)** |
| [solutions/maritime.md](./solutions/maritime.md) | **Maritime blueprint (planned)** |
| [pillars/README.md](./pillars/README.md) | **Platform Pillars — navigation hubs** |
| [solutions/adas.md](./solutions/adas.md) | **ADAS & Autonomous Driving blueprint** |
| [human-interaction-spatial-computing-roadmap.md](./human-interaction-spatial-computing-roadmap.md) | **Human Interaction, wearables, AR/VR/XR roadmap (H1–H4)** |
| [solutions/spatial-computing.md](./solutions/spatial-computing.md) | **Spatial Computing & Human-Robot Collaboration blueprint** |
| [solutions/smart-spaces.md](./solutions/smart-spaces.md) | **Smart Spaces & Ambient Intelligence blueprint** |
| [building-automation.md](./building-automation.md) | **Building automation and BMS integration** |
| [ambient-intelligence.md](./ambient-intelligence.md) | **Context-aware ambient intelligence** |
| [energy-management.md](./energy-management.md) | **Smart space energy management** |
| [smart-space-readiness.md](./smart-space-readiness.md) | **Smart space readiness gates** |
| [smart-space-security.md](./smart-space-security.md) | **Smart space security and access** |
| [smart-space-device-tree.md](./smart-space-device-tree.md) | **Smart space device tree examples** |
| [smart-space-packages.md](./smart-space-packages.md) | **Smart space packages and integration strategy** |
| [human-interaction.md](./human-interaction.md) | **Human entity model and device tree** |
| [operator-capabilities.md](./operator-capabilities.md) | **Operator capability verification** |
| [wearables.md](./wearables.md) | **Wearable device registry and packages** |
| [spatial-computing.md](./spatial-computing.md) | **Spatial anchors, workspaces, overlays** |
| [ar-vr-xr.md](./ar-vr-xr.md) | **AR, VR, and mixed reality workflows** |
| [hri.md](./hri.md) | **Human-robot interaction abstractions** |
| [human-readiness.md](./human-readiness.md) | **Operator, team, and mission readiness** |
| [remote-expert.md](./remote-expert.md) | **Remote expert guided repair workflows** |
| [hri-packages.md](./hri-packages.md) | **HRI and spatial computing optional packages** |
| [automotive-device-tree.md](./automotive-device-tree.md) | **Automotive device hierarchy and capability mapping** |
| [adas-readiness.md](./adas-readiness.md) | **ADAS pre-drive readiness gates** |
| [demo-plan-adas.md](./demo-plan-adas.md) | **ADAS blueprint demo plan** |
| [chaos-engineering.md](./chaos-engineering.md) | **Chaos injection and resilience validation** |
| [resource-estimation.md](./resource-estimation.md) | **Mission resource estimation** |
| [readiness-trends.md](./readiness-trends.md) | **Readiness trend analysis and forecasting** |
| [package-trust.md](./package-trust.md) | **Package trust scoring** |
| [scorecards.md](./scorecards.md) | **Autonomous systems scorecard** |
| [tamper-detection.md](./tamper-detection.md) | **Hack / tamper detection framework** |
| [integrity-verification.md](./integrity-verification.md) | **Configuration and artifact integrity** |
| [trust-framework.md](./trust-framework.md) | **Composite trust scoring** |
| [hardware-attestation.md](./hardware-attestation.md) | **Secure-boot contracts and live attestation** |
| [spoofing-detection.md](./spoofing-detection.md) | **GPS and sensor spoofing detection** |
| [security-assurance.md](./security-assurance.md) | **Security assurance rollup reports** |
| [compiler-backend-roadmap.md](./compiler-backend-roadmap.md) | **LLVM / native codegen evolution** |
| [ffi-and-ecosystem.md](./ffi-and-ecosystem.md) | **Python/C++/ROS2 interoperability strategy** |
| [migration.md](./migration.md) | Migration from legacy syntax and dual-backend notes |
| [test-plan.md](./test-plan.md) | Test coverage plan |

## Repository layout

```
crates/                     Rust workspace — see crates/README.md for full index
  spanda-core/              Public facade (re-exports, hardware_verify, thin shims)
  spanda-driver/            compile, check, run, SIR, replay, debug
  spanda-connectivity/      Connectivity catalogs and hardware profile foundation types
  spanda-cli/               Native `spanda` binary (crate package name: `spanda`)
  spanda-interpreter/       Tree-walking runtime (~21 modules under src/runtime/)
  spanda-parser/            Parser (lexer → AST)
  spanda-ast/               AST nodes and foundation types
  spanda-typecheck/         Type checker and units
  spanda-hardware/          Builtin hardware profile catalog (re-exports connectivity types)
  spanda-transport-routing/ RoutingCommBus, transport_live, live_bridges
  spanda-fleet/             Fleet orchestration, agents, mesh, swarm
  spanda-ota/               OTA deploy, rollout, remote agents
  spanda-package/           Package manager (no spanda-core dependency)
  spanda-providers/         Official package bootstrap
  spanda-llvm/              Experimental LLVM codegen
  spanda-node/              Node.js N-API bindings
  spanda-wasm/              WebAssembly bindings
  spanda-dap/               Debug Adapter Protocol server
  …                         + comm, safety, hal, format, lint, certify, bridge, …

packages/
  native/                   @spanda/native — Node wrapper for N-API
  web/                      @davalgi-spanda/web — React playground
  lsp/                      @spanda/lsp — Language Server
  registry/                 38 official .sd packages (spanda-gps, spanda-ros2, spanda-onnx, spanda-fusion, …)

src/                        TypeScript mirror (tests, CLI wrapper, providers)
editor/vscode/              VS Code extension scaffold
examples/                   Sample .sd programs (basics/, features/, showcase/, …)
tests/                      Vitest suite and golden fixtures
docs/                       Guides, architecture, API reference
scripts/                    Doc tooling, example regression, ROS2 bridge helpers
```

**Dependency rule:** Only `spanda-core` pulls the full facade graph. `spanda-cli`, `spanda-node`, `spanda-wasm`, `spanda-dap`, and `spanda-llvm` import workspace crates directly.

**Removed from `spanda-core` (Phase 17):** `transport_live`, `transport_mqtt`, `transport_dds`, `transport_websocket` — use `spanda-transport-routing` or `spanda-transport-*` crates.

## CLI

```bash
spanda check examples/rover.sd
spanda verify examples/hardware/rover_deploy.sd
spanda verify robot.sd --target RoverV1 --all-targets --simulate
spanda run examples/rover.sd
spanda sim examples/rover.sd --replay --record
spanda replay mission.trace --deterministic
spanda replay mission.trace --playback --from T+00:30
spanda fleet run examples/communication/multi_robot_fleet.sd
spanda fmt examples/rover.sd
spanda reference --out docs/spanda-reference.md --man-dir docs/man
```

Trace and telemetry flags for `run`, `sim`, and `fleet run`:

```bash
spanda run robot.sd --trace-scheduler --trace-tasks --trace-triggers --trace-events
spanda run robot.sd --trace-realtime --metrics-json
spanda sim robot.sd --record --trace-realtime
spanda sim robot.sd --wall-clock
```

## Install

Install prebuilt packages for Linux, macOS, and Windows from [GitHub Releases](https://github.com/Davalgi/Spanda/releases), or build from source. See [installation.md](./installation.md) for shell/MSI/PowerShell installers, platform archives, and maintainer packaging notes.

```bash
# Linux / macOS (replace v0.1.0 with your release tag)
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/Davalgi/Spanda/releases/download/v0.1.0/spanda-cli-installer.sh | sh
```

Contributors can build the native CLI with `npm run build:rust` (output: `target/release/spanda`).

## Developer documentation

Rust (`crates/`), TypeScript (`src/`, `packages/`), and Python (`scripts/`) use structured inline API docs. See **[coding-standards.md](./coding-standards.md)** and **[documentation-coverage.md](./documentation-coverage.md)**.

Tooling lives in `scripts/`:

- `validate_documentation.py` — audit coverage and regenerate the coverage report
- `add_structured_api_docs.py` — generate structured API doc blocks
- `fix_structured_doc_gaps.py` — fix empty Inputs and legacy single-line comments
- `repair_doc_param_typos.py` — repair truncated parameter names in generated docs
- `add_inline_docs.py` — legacy API doc generator
- `add_logic_block_docs.py` — generate contextual block comments
- `normalize_inline_docs.py` — fix spacing and indentation (run after bulk edits)
- `generate_api_reference.py` — regenerate [api-reference.md](./api-reference.md) from source (see [api-documentation.md](./api-documentation.md))
- `generate_spanda_reference.py` — regenerate [spanda-reference.md](./spanda-reference.md) and [man/](./man/)

See [../CONTRIBUTING.md](../CONTRIBUTING.md#inline-documentation) for contributor workflow.

## Links

- GitHub: [github.com/Davalgi/Spanda](https://github.com/Davalgi/Spanda)
- Golden tests: [../tests/golden/manifest.json](../tests/golden/manifest.json)
