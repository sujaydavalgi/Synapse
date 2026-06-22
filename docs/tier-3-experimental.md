# Tier 3 — Experimental foundations (Phase 22)

Product strategy [Tier 3](./product-strategy.md) items are **not** v1 commitments. Phase 22 promotes each area from *deferred* to *experimental* with a documented golden path or minimal runtime.

| Item | Phase 22 status | Golden path / entry point | CI (Phase 23) |
|------|-----------------|---------------------------|---------------|
| **LLVM native codegen** | Experimental | `scripts/llvm_golden_path.sh`, `spanda compile-native` | `llvm-golden-path` job |
| **Blockchain / ledger** | Experimental | `spanda-ledger` scaffold; `scripts/ledger_golden_path.sh` | `ledger-golden-path` job |
| **World models** | Experimental | `world_model { }` parser + fusion hook; `examples/features/world_model_observe.sd` | — |
| **Digital twin cloud sync** | Experimental | `spanda twin export` + `SPANDA_CLOUD_UPLOAD_URL`; `scripts/twin_cloud_golden_path.sh` | `twin-cloud-golden-path` job |
| **Distributed fleet** | Experimental | `spanda fleet orchestrate --remote`, mesh coordinator; `examples/robotics/golden_path_deploy.sh` | `robotics-golden-path` job |
| **MQTT / DDS live transport** | Experimental | `SPANDA_LIVE_MQTT=1`, `--features live-mqtt`; `examples/communication/mqtt_live.sd` | `mqtt-golden-path` job |
| **C++ in-process FFI** | Experimental | `spanda-bridge` `cpp-native` feature; `scripts/cpp_native_golden_path.sh` | `cpp-native-golden-path` job |
| **Self-hosting compiler** | Bootstrap started | `examples/self_host/lexer_keywords.sd`; `scripts/self_host_lexer_golden_path.sh` | `self-host-lexer-golden-path` job |

## Performance (Phase 18 P2) — complete

| ID | Item | Status |
|----|------|--------|
| P2.1 | Slim CLI (`--no-default-features --features slim`) | **Complete** |
| P2.2 | Bridge timeouts (`SPANDA_BRIDGE_TIMEOUT_SECS`) | **Complete** |
| P2.3 | `cargo audit` CI job | **Complete** |

## Observability (Phase 18 P3) — complete

| ID | Item | Status |
|----|------|--------|
| P3.1 | Pipeline benchmark | **Complete** — `cargo test -p spanda-driver pipeline_bench -- --ignored` |

Runtime telemetry (`RuntimeTelemetry`, mission traces, trigger metrics) remains the production observability surface.

## Still future (not experimental)

- LLVM as **primary** deploy path (interpreter stays default)
- Production blockchain adapters (`spanda-ledger-ethereum`, etc.)
- Full world-model / knowledge-graph semantics
- Production twin cloud SaaS
- Full fleet planning / consensus
- OMG DDS middleware (current DDS adapter is UDP JSON shim)
- Full ROS replacement
- Complete self-hosted compiler

**Priority and timeline:** [tier-3-priority-plan.md](./tier-3-priority-plan.md)

## Related

- [phase-18-security-hardening.md](./phase-18-security-hardening.md)
- [lean-core-roadmap.md](./lean-core-roadmap.md) — Phase 22
- [feature-status.md](./feature-status.md)
- [compiler-backend-roadmap.md](./compiler-backend-roadmap.md)
- [future-blockchain-support.md](./future-blockchain-support.md)
