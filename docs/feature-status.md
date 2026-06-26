# Feature Status

Honest snapshot of **Spanda Platform** capabilities as of **v0.4.0**. The Spanda Language (`.sd`) is one component; this matrix covers verification, simulation, fleet, packages, and tooling as well. Use this document to understand what is production-ready, experimental, planned, or deprecated.

Platform overview: [platform-overview.md](./platform-overview.md)

**Stubbed** = syntax or API exists without full external integration.  
**Broken** = known to fail or incomplete in current builds.

---

## v0.4.0 — Deploy & tooling (current)

| Area | Status |
|------|--------|
| **Native deploy** | `spanda deploy --target native`, `compile-native`, LLVM IR — Experimental (clang required) |
| **ROS 2 interop** | `spanda ros2 check`, rclpy bridge with `SPANDA_ROS2_LIVE=1` — Experimental |
| **Distributed fleet** | `fleet orchestrate --remote`, agent registry — Experimental |
| **CLI install** | `cargo install --path crates/spanda-cli` → binary `spanda` — Stable |
| **Bundled demos** | `spanda demo` without full clone — Stable |
| **Cascading configuration** | `spanda config`, `spanda drift`, `spanda device discover|inspect`, `spanda network scan`, `spanda device-tree`, `spanda map verify`; `DeviceRegistry` + network identity validation; config and agent drift (`--baseline`, `--agent`); `--config` on run/verify/readiness/replay/assurance — Experimental |

## v0.2.0 — Officially Supported

### Supported (stable for public evaluation)

| Area | Capabilities |
|------|----------------|
| **Language core** | Lexer, parser, AST, type checker, physical units, `module`/`import`, structs/enums/traits, `match`, `Result`/`Option`, `test` blocks |
| **AI agents** | `ai_model`, `agent`, `goal`, `memory`, mock LLM/Vision providers, `ActionProposal` → `safety.validate()` → `SafeAction` |
| **Robotics primitives** | `robot`, `sensor`, `actuator`, `behavior`, `task every Nms`, state machines, events |
| **Hardware profiles** | `hardware`, `deploy`, `requires_hardware`, `requires_network`, SoC/HAL validation |
| **Compatibility verification** | `spanda verify`, `--target`, `--all-targets`, `--simulate`, `--json` |
| **Simulation** | `spanda run` / `spanda sim`, physics-lite 2D backend, lidar/arm/drone models |
| **Communication** | `message`, `topic`, `service`, `action`, `publish`/`call`/`send_goal`, in-memory transport |
| **Safety validation** | Safety zones, `max_speed`, `stop_if`, emergency stop, compile-time `SafeAction` gate |
| **Trigger-driven execution** | Unified `on` / `every` / `when` / `while`; event, message, timer, condition, state, safety, hardware, AI, verification, twin |
| **Cooperative concurrency** | `spawn`, `join`, `parallel`, channels, `select`, per-task `budget { }`; TypeScript mirror parity |
| **Fleet simulation** | `spanda fleet run` — in-process multi-robot with deploy/peer wiring |
| **Swarm coordinator (experimental)** | `swarm { fleet; policy; }` + `spanda swarm coordinate` — round-robin cursors in `.spanda/swarm-state.json`; `--mesh-url` relays peer/leader-follow steps via fleet mesh |
| **Robotics platform** | `mission`, `fleet`, `safety_zone`, `certify`; navigation/fusion runtime; Nav2 adapter hook |
| **OTA deploy CLI** | `spanda deploy plan|rollout|rollback|status` — local rollout state (`.spanda/deploy-state.json`) |
| **Remote OTA agents** | `spanda deploy agent start|register|list` + `deploy rollout --remote` — HTTP agent on devices; `--require-certify` on agent and rollout |
| **Fleet orchestration** | `spanda fleet orchestrate` — round-robin mission coordination report; `--remote` relays peer steps via HTTP fleet agents |
| **Fleet peer agents** | `spanda fleet agent start|register|list` — on-device peer relay server (`.spanda/fleet-agents.json`) |
| **Fleet mesh coordinator** | `spanda fleet mesh start` + `fleet orchestrate --mesh-url` — centralized multi-host peer relay |
| **Adapter package verify** | `spanda verify-adapter` — validate `[adapter]` provides/requires against registry metadata |
| **Tooling** | Native CLI (`check`, `verify`, `run`, `sim`, `fleet`, `deploy`, `fmt`, `lint`, `doc`), package manager (`init`, `build`, `test`, `install`), **prebuilt installable packages** (Linux/macOS/Windows via GitHub Releases) |
| **Showcase demos** | `spanda demo {rover,safety,verify,fleet,health,readiness,assurance,self-healing,continuity}`; `examples/showcase/*` |
| **Security / audit** | Capabilities, secrets, signed messages, audit records |
| **Secure communication** | `secure_comm`, encrypted buses, trusted-source publish/receive enforcement, AES-GCM wire frames, TLS session + rustls PEM validation, `spanda security check|audit`, TS runtime parity |
| **Digital twins** | `twin`, mirror fields, replay buffer, `twin sync` telemetry |
| **Real-time contracts** | `deadline`, `jitter <=`, `priority`, `critical isolated` on tasks; latency `pipeline` budgets |
| **Reliability runtime** | Watchdogs, operating `mode` blocks, `recover from`, retry/fallback; topic QoS deadline detection |
| **Runtime fault detection** | `heartbeat`, `memory_watch`, `resource_watch`, `restart_policy`, `on runtime crash`; CLI `spanda fault scan|report`, `spanda runtime health|diagnose`, `spanda replay --show-faults`; mission trace fault frames |
| **Mission trace replay** | `spanda sim --record`, `spanda replay`, `--deterministic`, `--playback`, `--wall-clock` |
| **Persistent telemetry** | `--persist-telemetry`, `SPANDA_TELEMETRY_STORE=1`, `spanda telemetry` — JSONL or SQLite; OTLP `push`/`serve`, `fleet-push` mesh aggregation, sessions + replay |
| **First-class regex** | Literals, `Regex` type, string methods, trigger/subscribe filters, `validate` rules |
| **Lean-core workspace** | 50+ focused Rust crates; `spanda-core` facade; CLI/bindings use workspace deps directly ([crates/README.md](../crates/README.md)) |
| **Verification & DX** | `spanda-capability` — traceability, minimum-hardware, health analysis; `spanda-readiness` — operational readiness, mission verification, safety reports; `spanda check --verification-json`; LSP verification diagnostics and quick-fixes |
| **Health & kill switch** | `health_check`, `health_policy`, fleet `require` runtime; `kill_switch`, `remote_signed`, `on kill_switch` handlers |
| **Self-healing & recovery** | `recovery_policy`, recovery planner, validation gates, audit/traceability; runtime dispatch (modes, speed caps, fleet mesh relay, reassign → continuity mesh); CLI `heal`, `recover`, `recovery-report`, `recovery knowledge`, `sim --inject-failure`; fleet agent interpreter + assurance recovery; mission operator approval gating |
| **Mission continuity** | Checkpoint resume, state transfer, succession ranking, takeover/delegation; CLI `continuity`, `takeover`, `delegate`, `succession`; `continuity_policy`; diagnostics in `spanda check --readiness-json`; `spanda demo continuity`; official `spanda-mission-continuity` package |
| **Differentiation NOW** | `spanda contract verify`, `spanda explain`, `spanda audit decisions`, `spanda safety-coverage`, `spanda recovery-coverage`; `spanda demo differentiation` |
| **Mission assurance** | `knowledge_model`, `state_estimator`, `anomaly_detector`, `on anomaly`, `prognostics`, `mitigation`, `resilience_policy`, `assurance_case`; CLI `assure`, `anomaly scan`, `diagnose`, `state estimate`, `prognostics`, `mission verify`, `resilience check`, `mitigation plan`; `spanda demo assurance` |
| **Weighted sensor fusion** | `observe { }`, `state_estimator`, `fusion.read()` with type-weighted confidence; `spanda-fusion` package |
| **Learned anomaly runtime** | `learned backend assurance.anomaly`; EMA volatility; optional ONNX (`SPANDA_ANOMALY_ONNX_MODEL_PATH`) |
| **Typed handler I/O** | Return types on behavior, task, trigger, event, and agent plan handlers (Rust + TS mirror) |

### Experimental (usable with caveats)

| Area | Capabilities | Caveats |
|------|--------------|---------|
| **Digital twins (live sync)** | Twin mirror + replay; optional HTTP upload via `SPANDA_CLOUD_UPLOAD_URL` | No production twin cloud SaaS |
| **Replay** | `replay true`, frame buffer, mission traces | In-process only; v2 traces embed state snapshots for `--playback` |
| **Advanced verification** | Fault injection, compatibility matrix | Matrix may report stub targets |
| **Multi-agent systems** | Agent-to-agent comm, fleet peer messaging | In-process mesh + HTTP fleet agent relay (`fleet orchestrate --remote` / `--mesh-url`) |
| **OTA rollout** | Deploy plan/rollout/rollback/status | Local state file + HTTP deploy agents; `--require-certify` blocks uncertified rollouts |
| **Certification metadata** | `certify ISO13849 { level PLd; }` | Verify-only metadata; `--strict-certify` / `--enforce-certify`; `spanda certify prove`; deploy plan proof summary |
| **Nav2 / SLAM packages** | Registry adapter stubs + example packages | External Nav2/Gazebo/OpenCV not bundled; optional `SPANDA_NAV2_CMD` / `SPANDA_SLAM_CMD` bridges |
| **ROS2 adapter** | Native `rclrs` cdylib, rclpy daemon, CLI bridge | Requires ROS Humble; not default transport |
| **LLVM / native codegen** | `spanda ir`, `llvm-ir`, `compile-native`; `scripts/llvm_golden_path.sh` | Early stage; not primary execution path |
| **FFI** | `extern python`/`extern cpp` subprocess bridges; optional `cpp-native` in-process | PyO3 path is Tier 2 adoption unlock |
| **World models** | `world_model { }` block parser; `fusion.read()` → belief hook; `world_model.update` / `belief` / `export`; Rust + TS typecheck parity | Minimal belief buffer; see [world_model_patrol.sd](../examples/showcase/world_model_patrol.sd) |
| **Ledger / provenance** | `spanda-ledger` provider → `MockLedgerBackend` | Mock chain only; no production blockchain adapters |
| **MQTT / DDS live** | `SPANDA_LIVE_MQTT=1`, `--features live-mqtt`; CI `mqtt-golden-path` | DDS is UDP JSON shim, not full DDS middleware |
| **Self-hosting bootstrap** | `examples/self_host/lexer_keywords.sd`; Rust parity tests | Rust compiler remains authoritative |
| **LSP** | Diagnostics, completion, hover, rename, verification quick-fixes | Requires built native CLI; VS Code extension with bundled LSP; continuity/recovery policy quick-fixes; CI builds VSIX |
| **DAP debugger** | Breakpoints, step over/in/out, `every` trigger entry | VS Code + `spanda-dap`; tested in `phase34_gaps.rs` / `phase35_gaps.rs` |
| **WASM / web playground** | Browser check/run/verify | Limited surface vs native CLI |
| **Live AI providers** | OpenAI, Anthropic, ONNX via Python bridge | Requires API keys or `SPANDA_ONNX_MODEL_PATH`; mock fallback by default |
| **Live IoT bridges** | Modbus TCP, OPC-UA, zigbee, lora, matter, canbus | Env-gated (`SPANDA_LIVE_*=1`); in-memory hub fallback |
| **Package publish** | `spanda publish`, registry search, mirror to `registry/packages/` | Remote upload via `SPANDA_REGISTRY_URL`; hosted index lists **40** packages after `build-registry.sh` |

### Planned (v0.5 beta and beyond)

| Area | Description |
|------|-------------|
| **Differentiation (NEXT)** | What-If Analysis, Mission Risk Analysis, Readiness Forecasting, Trust Graph, Scorecards |
| **Differentiation (LATER)** | Digital Mission Twin, Certification Packs, Mission Time Travel, Human/Robot Teaming, Autonomous Governance |
| **Platform maturity (Phase A)** | `spanda graph`, `spanda deploy gate`, `spanda explain` (with `--config`/`--baseline`), `spanda trust` (package + program) — **Experimental**; see [platform-maturity-roadmap.md](./platform-maturity-roadmap.md) |
| **Platform maturity (Phase B)** | Threat model, mission diff, scorecard (`spanda score`), policy engine (`spanda verify --policy`, `readiness --policy`, `deploy gate --operational-policy`, runtime `--enforce-policy`) — **Experimental** |
| **Platform maturity (Phase C)** | Chaos, readiness trends, resource estimation, compliance profiles, ADR (`spanda adr`) — **Experimental** |
| **Platform maturity (Phase D)** | Verify-time tamper/integrity, composite program trust, secure-boot attestation (vendor TPM + remote AK chain), compliance accreditation export, decision explain, runtime policy, AI generate/suggest, spoof-check with confidence gates, security assurance, tamper_policy runtime — **Experimental** |
| **Platform maturity (Phase C–D)** | Readiness trends, resource estimation, compliance profiles, ADR, tamper check |
| **Enterprise operations (E1–E4, experimental)** | Control Center (`spanda control-center serve`, embedded UI, `ControlCenterPanel` in `@spanda/web`, Tauri `@spanda/control-center-desktop` scaffold), REST v1 (`spanda-api`), Device Pool lifecycle (assign/trust/quarantine/retire, failover chains, recovery integration), host-backed discovery + pool ingest, RBAC v1 (`SPANDA_API_KEY`), `ManagedSecretVault`, alerting core (`spanda-ops`), provisioning/snapshots/discovery (E2), operational drift/OTA/trust/SRE/operator APIs + Python SDK + WebSocket telemetry + OTLP trace export to Jaeger (E3), compliance export/digital thread/executive scorecard/PDF reports (E4); see [enterprise-operations-roadmap.md](./enterprise-operations-roadmap.md), [control-center.md](./control-center.md) |
| **Enterprise operations (NEXT)** | tonic gRPC expansion (9 RPCs), Tauri installer CI (`TAURI_BUILD=1` on macOS), registry-backed discovery packages, vendor alert channel packages, OTLP metrics export, full gRPC CLI parity |
| **LLVM backend (production primary)** | Optimized native binaries replacing interpreter as default deploy path |
| **Self-hosting compiler (full)** | Complete Spanda-authored compiler pipeline |
| **ROS2 production adapter** | First-class, zero-config ROS2 deployment |
| **VS Code Marketplace publish** | VSIX CI + local install ready; marketplace listing pending maintainer `VSCE_PAT` |
| **Production blockchain** | `spanda-ledger-ethereum` and related chain adapters |
| **Full world models** | Knowledge graphs, beliefs, policies beyond minimal runtime |
| **Twin cloud SaaS** | Managed digital-twin backend (golden-path upload script exists) |

See [tier-3-experimental.md](./tier-3-experimental.md) and [tier-3-golden-paths.md](./tier-3-golden-paths.md).

### Deprecated

| Feature | Replacement | Notes |
|---------|-------------|-------|
| Legacy inference-only AI paths | `ai_model` + `agent` | Import-based ONNX/TFLite remain for classical workflows |
| TypeScript-only verification | Native `spanda verify` | TS mirror validates deploy syntax; Rust CLI is authoritative |
| `spanda_core::transport_live` | `spanda_transport_routing::transport_live` | Removed Phase 17 |
| `spanda_core::transport_mqtt` / `transport_dds` / `transport_websocket` / `transport_live` | `spanda-transport-*` or `spanda_transport_routing::live_bridges` | Removed Phase 17 |
| `spanda_core::transport` / `transport_wire` / `transport_security` / `transport_rclrs` | `spanda-transport-routing`, `spanda-transport`, `spanda-transport-ros2` | Removed Phase 19 |

---

## Feature matrix

### Language core

| Feature | Status | Notes |
|---------|--------|-------|
| Lexer / parser / AST | **Stable** | Rust authoritative; TS mirror |
| Type checker + units | **Stable** | Physical unit algebra enforced |
| modules / imports | **Stable** | `spanda install` vendor support |
| structs / enums / traits | **Stable** | Generic struct literals; enum payloads |
| generics | **Experimental** | Module fn + struct type params |
| trait objects (`dyn Trait`) | **Stable** | Rust + TS mirror |
| match / Result / Option | **Stable** | |
| async / await | **Stable** | Cooperative single-threaded |
| spawn / select / channels | **Stable** | Cooperative concurrency with TS mirror |
| triggers (`on` / `every` / `when`) | **Stable** | Unified `TriggerRegistry`; see `docs/triggers.md` |
| test blocks | **Stable** | Rust runtime + TS `runTests()` |
| `extern fn` / FFI | **Experimental** | Subprocess bridges; optional in-process |
| Spanda IR (SIR) | **Stable** | JSON export via `spanda ir` |
| Codegen / LLVM | **Experimental** | HAL profiles; conditional codegen |

### Autonomous systems

| Feature | Status | Notes |
|---------|--------|-------|
| robot / sensor / actuator | **Stable** | |
| agent / goal / task / skill | **Stable** | Mock AI backend |
| ActionProposal → SafeAction | **Stable** | Compile + runtime |
| safety zones / emergency stop | **Stable** | |
| deterministic scheduler | **Stable** | `task every Nms` |
| deadline / jitter / priority | **Stable** | Compile-time validation; runtime telemetry |
| pipelines / watchdogs / modes | **Stable** | See `docs/reliability.md`, `docs/watchdogs.md`, `docs/degraded-modes.md` |
| mission trace replay | **Stable** | `--record`, `spanda replay --deterministic` / `--playback` |
| persistent telemetry store | **Stable** | `--persist-telemetry`, `spanda telemetry`; JSONL (default) or SQLite; OTLP export/push/serve, `fleet-push`, sessions — [telemetry-store.md](./telemetry-store.md) |
| regex literals / filters | **Stable** | See `docs/regex.md` |
| state machine / events | **Stable** | |
| twin / replay | **Experimental** | Replay buffer; live sync simulated |
| observe / fusion | **Stable** | Weighted fusion by sensor type; `state_estimator` runtime bindings |
| mission assurance (static + CLI) | **Stable** | `spanda-assurance` crate; 9 official packages (includes `spanda-mission-continuity`) |
| self-healing & recovery (static + CLI) | **Stable** | Recovery planner, validation gates, audit, knowledge store |
| mission continuity (static + CLI + diagnostics) | **Stable** | `spanda-assurance` continuity module; CLI `continuity`, `takeover`, `delegate`, `succession`; `continuity:*` diagnostics in check JSON and LSP |
| mission continuity runtime dispatch | **Stable** | Interpreter mode-specific takeover, durable checkpoints, auto-trigger on health faults, fleet agent `/v1/continuity/execute`, mesh relay, swarm `--failed` handoff |
| self-healing runtime dispatch | **Stable** | Auto-trigger on health faults, approval polling/retry, fleet mesh relay with failure events, mission approval gating; `scripts/fleet_field_validation.sh` |
| fleet agent interpreter recovery | **Stable** | `POST /v1/recovery/execute` with `recovery_engine: interpreter`; `scripts/fleet_agent_recovery_smoke.sh` |
| recovery diagnostics (CLI + LSP) | **Stable** | `spanda check --readiness-json` merges `recovery:*` categories; TS mirror in `scripts/lsp-readiness.mts` |
| continuity diagnostics (CLI + LSP) | **Stable** | `spanda check --readiness-json` merges `continuity:*` categories including `continuity:mission`; TS mirror in `src/continuity-diagnostics.ts` |
| learned anomaly backends | **Experimental** | Runtime `scan_learned`; ONNX optional |
| verify { } behavioral assertions | **Stable** | |
| hardware / deploy | **Stable** | Rust verify CLI |

### Tooling

| Feature | Status | Notes |
|---------|--------|-------|
| Native CLI (full) | **Stable** | check, verify, run, sim, replay, fleet, fmt, lint, doc, man, reference, package |
| Prebuilt packages | **Stable** | Linux/macOS/Windows archives, shell/PowerShell installers, Windows MSI, Homebrew formula; see [installation.md](./installation.md) |
| TypeScript CLI | **Stable** | Delegates to Rust when built |
| Formatter / linter / docgen | **Stable** | `///` doc comments in `.sd`; `spanda doc` (markdown/HTML/JSON); `spanda man`; [man pages](./man/README.md) |
| LSP | **Experimental** | VS Code extension scaffold; CI builds VSIX on push |
| DAP debugger | **Experimental** | VS Code + `spanda-dap`; `every` trigger entry (Phase 35) |
| N-API | **Experimental** | check, run, verify, sir, fmt |
| WASM | **Experimental** | check, run, verify, sir, fmt |

### Ecosystem / FFI

| Feature | Status | Notes |
|---------|--------|-------|
| python.* / cpp.* imports | **Experimental** | Subprocess bridges |
| ROS2 adapter | **Experimental** | Native rclrs cdylib; CI job on Ubuntu 22.04 + Humble |
| Transport adapters | **Experimental** | In-memory + optional rclrs/rclpy |
| Package manager | **Stable** | Hosted index + local mirror; `spanda publish` copies to `registry/packages/` |
| LLVM / native codegen | **Experimental** | `compile-native` early stage |

### Enterprise operations (20 pillars)

| Pillar | Status | Key surfaces |
|--------|--------|--------------|
| **Control Center** | **Experimental** | `spanda control-center serve`, `ControlCenterPanel`, Tauri desktop scaffold |
| **Device Pool** | **Experimental** | Lifecycle states, assign/trust/quarantine/retire, failover chains |
| **Device Discovery** | **Experimental** | Subnet, mDNS, BLE, USB, CAN, MQTT, ROS2 host probes; pool ingest |
| **Provisioning** | **Experimental** | `POST /v1/provision`, discover → ready workflow |
| **Configuration Management** | **Experimental** | Snapshots, diff, resolve; approval **Planned** |
| **RBAC** | **Experimental** | 7 roles, `SPANDA_API_KEY`, `/v1/rbac/matrix` |
| **Secret Management** | **Experimental** | `ManagedSecretVault`, rotation metadata |
| **Telemetry** | **Experimental** | Health/readiness/mission signals; trend analysis; forecasting **Planned** |
| **Alerting** | **Experimental** | Webhook + email core; Slack/Teams/PagerDuty packages **Planned** |
| **Configuration Drift** | **Experimental** | Full operational drift API (`detect_operational_drift_full`); seven dimensions via Control Center `GET /v1/drift` |
| **OTA & Rollback** | **Experimental** | Canary, blue/green, phased dry-run; production fleet automation **Planned** |
| **Package Trust** | **Experimental** | `spanda trust`, `/v1/trust/package`, trust score |
| **SDKs** | **Experimental** | Python SDK, REST v1, WebSocket; tonic gRPC (9 RPCs); CLI reference **Stable** |
| **Operator Workflows** | **Experimental** | Mission approve, takeover, quarantine, recovery approval |
| **SRE** | **Experimental** | `/v1/sre/summary`; incident workflow UI **Planned** |
| **Reporting** | **Experimental** | HTML, Markdown, JSON, PDF, CSV exports |
| **Compliance** | **Experimental** | Evidence packs, `GET /v1/compliance/export` |
| **APIs** | **Experimental** | REST v1 + OpenAPI; JSON-RPC gateway; native gRPC (tonic) **Experimental** — 28 RPCs on `--grpc-bind` |
| **Observability** | **Experimental** | OTLP trace export, correlation IDs, WebSocket telemetry; Control Center OTLP metrics preview/export |
| **Digital Thread** | **Experimental** | `GET /v1/digital-thread/query`; full graph UI **Planned** |

See [enterprise-operations-roadmap.md](./enterprise-operations-roadmap.md) · [control-center.md](./control-center.md) · [device-pool.md](./device-pool.md)

---

## Known limitations (v0.4.0)

- AI providers use **mock backends** by default; set `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, or `SPANDA_ONNX_MODEL_PATH` for live calls (`SPANDA_LIVE_AI=0` forces mock).
- ROS2 integration requires manual ROS Humble setup and is not the default simulator transport.
- Native compilation via LLVM is **experimental**; the tree-walking interpreter is the primary runtime.
- `spanda publish` mirrors bundles to `registry/packages/` in-repo; remote upload requires `SPANDA_REGISTRY_URL`. Run `./scripts/build-registry.sh` to refresh the hosted index after adding scaffolds under `packages/registry/`.
- VS Code extension builds in CI (`verify_vscode_vsix.sh`); **Marketplace publish** pending maintainer `VSCE_PAT`.
- Multi-robot fleet examples run in a single process by default; distributed orchestration uses HTTP fleet agents and an optional fleet mesh coordinator (`spanda fleet mesh start`, `--mesh-url` on orchestrate/swarm).

---

## Broken / stubbed (honest audit)

| Item | Category | Detail |
|------|----------|--------|
| Global package registry | Hosted + mirror | Default `SPANDA_REGISTRY_URL` points at repo index; `spanda publish` mirrors to `registry/packages/` |
| Live OpenAI / Anthropic / ONNX | Optional live path | `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, or `SPANDA_ONNX_MODEL_PATH`; Python bridge; mock fallback |
| Live Modbus / OPC-UA IoT | Optional live hardware | `SPANDA_LIVE_MODBUS=1` / `SPANDA_LIVE_OPCUA=1`; `--features live-iot` for native Modbus TCP |
| IoT protocol bridges (zigbee/lora/matter/canbus) | Live + hub fallback | `SPANDA_LIVE_ZIGBEE=1` etc.; in-memory hub + `package_dispatch`; `./scripts/live_iot_golden_path.sh` |
| Kill switch remote_signed | Runtime + verify enforced | Requires `kill_switch_signature` JSON when `remote_signed` is set; verify reports **error** without signed comm |
| MQTT / DDS / WebSocket live transport | Production wire + optional live brokers | AES-256-GCM wire frames; live MQTT/WebSocket/DDS via `--features live-transport` + `SPANDA_LIVE_MQTT=1` / `SPANDA_LIVE_WEBSOCKET=1` / `SPANDA_LIVE_DDS=1`; TypeScript mirrors the same env flags |
| Secure comm live crypto | Production wire | AES-256-GCM for transport frames and `EncryptedMessage` payloads; session material from robot secrets |
| Full native binary deploy | Experimental | `spanda deploy --target native`, `compile-native` (clang + llvm feature) |
| Blockchain / ledger cloud | Stubbed | Audit records local; see `future-blockchain-support.md` |

Nothing in the **Supported** list above is known broken in CI (`cargo test --workspace`, `npm test`, `cargo fmt`, `cargo clippy`, ROS2 rclrs native on Ubuntu 22.04).

---

## Architecture summary

```
.sd source
  → lexer → parser → AST
  → type checker (+ units, safety, capabilities)
  → [optional] hardware verifier (deploy targets)
  → interpreter + simulator
  → [optional] SIR → LLVM (experimental)
```

| Crate | Role |
|-------|------|
| `spanda-core` | Language implementation (authoritative) |
| `spanda-cli` | Native `spanda` binary |
| `spanda-package` | Package manager |
| `spanda-audit` / `spanda-security` | Audit and security |
| `spanda-llvm` / `spanda-rt` | Experimental native codegen |
| `spanda-node` / `spanda-wasm` | Bindings |
| `spanda-dap` | Debug adapter |
| `@spanda/lsp` / `@spanda/web` | LSP and web playground |

See [architecture.md](./architecture.md) for diagrams.

---

## Related docs

- [README.md](../README.md) — project overview
- [getting-started.md](./getting-started.md) — first robot in 10 minutes
- [installation.md](./installation.md) — prebuilt packages and source install
- [triggers.md](./triggers.md) — trigger-driven execution
- [concurrency.md](./concurrency.md) — tasks, spawn, channels, fleet CLI
- [realtime.md](./realtime.md) — deadline-aware tasks and wall-clock scheduling
- [reliability.md](./reliability.md) — pipelines, watchdogs, recovery
- [replay.md](./replay.md) — mission trace record and replay
- [telemetry-store.md](./telemetry-store.md) — persistent device/sensor/heartbeat storage
- [regex.md](./regex.md) — first-class regex
- [vision.md](./vision.md) — long-term positioning
- [product-strategy.md](./product-strategy.md) — v0.5 beta priorities
- [ffi-and-ecosystem.md](./ffi-and-ecosystem.md) — Python/C++/ROS2 interop
- [compiler-backend-roadmap.md](./compiler-backend-roadmap.md) — LLVM evolution
- [health-checks.md](./health-checks.md) — health checks and fleet requirements
- [kill-switch.md](./kill-switch.md) — kill switch syntax and handlers
- [capability-traceability.md](./capability-traceability.md) — traceability matrices
- [verification-diagnostics.md](./verification-diagnostics.md) — `--verification-json` and LSP quick-fixes
- [typed-handler-io.md](./typed-handler-io.md) — handler return type annotations
- [testing.md](./testing.md) — `expect_compile_error` and test CLI
- [iot.md](./iot.md) — IoT packages and live bridges
- [live-ai-provider.md](./live-ai-provider.md) — OpenAI, Anthropic, ONNX
- [debugging.md](./debugging.md) — VS Code DAP workflow
- [registry.md](./registry.md) — hosted registry and publish mirror
- [packages.md](./packages.md) — package manager
