# Test Coverage Plan

## Rust (`cargo test --workspace`)

| Area | Tests | Location |
|------|-------|----------|
| Lexer | unit suffixes, keywords, `%` | `lexer.rs` |
| Parser | robot, HAL, AI, foundations, hardware | `parser.rs`, `tests/foundations.rs` |
| Type checker | units, safety, capabilities | `types.rs`, `tests/type_system.rs` |
| Runtime | match, tasks, interpreter, contracts | `runtime.rs`, `tests/runtime_hardening.rs` |
| Hardware verify | sensors, timing, power, faults, matrix | `hardware.rs`, `tests/hardware_compat.rs` |
| Scheduler | multi-task multiplex | `tests/scheduler.rs` |
| Fusion | observe + fusion | `tests/fusion.rs` |
| Twin replay | mirror, replay frames | `tests/twin_replay.rs` |
| Integration | all `examples/*.sd` compile + run | `tests/integration.rs` |
| Continuity | runtime takeover, checkpoints, CLI JSON, auto-trigger | `crates/spanda-interpreter/tests/continuity_runtime.rs`, `crates/spanda-cli/tests/continuity_cli.rs`, `crates/spanda-assurance/src/continuity_checkpoint.rs` |
| Swarm continuity | member-lost handoff + mesh relay | `crates/spanda-fleet/src/swarm_continuity.rs`, `crates/spanda-fleet/tests/mesh_integration.rs` |
| Self-healing runtime | auto-trigger, approval retry, mesh relay | `crates/spanda-interpreter/tests/recovery_runtime.rs`, `scripts/self_healing_smoke.sh` |
| Fleet field validation | multi-process agents + mesh orchestrate | `scripts/fleet_field_validation.sh` |
| gRPC Control Center | tonic (60 RPCs — full REST parity except `/v1/rpc`) | `crates/spanda-api/tests/grpc_tests.rs`, `grpc_live_probe.rs` |
| API rate limit + versioning | `SPANDA_API_RATE_LIMIT_PER_MINUTE`, `GET /v1/version`, `X-Spanda-Api-Version` | `crates/spanda-api/tests/api_policy_tests.rs` |
| OpenAPI REST parity | `GET /v1/openapi.json` documents all `/v1/*` routes | `crates/spanda-api/tests/openapi_parity_tests.rs` |
| Live OTA execute | `POST /v1/ota/execute` against deploy agent | `crates/spanda-api/tests/ota_execute_live.rs`, `scripts/ota_fleet_execute_smoke.sh` |
| OTA fleet soak | Multi-agent version bumps + canary progression | `crates/spanda-ota/tests/fleet_soak.rs`, `scripts/ota_fleet_soak.sh` |
| Failover drill | Redundant chain selection + recovery actions | `crates/spanda-config/tests/failover_drill.rs`, `scripts/failover_drill_smoke.sh` |
| Remote CLI parity | `spanda control-center` routes vs OpenAPI registry | `crates/spanda-cli/tests/control_center_openapi_parity.rs` |
| Discovery registry runtime | `spanda-discovery-mdns` package wrap | `crates/spanda-config/src/discovery_registry.rs` |
| OTLP metrics (Control Center) | `GET /v1/observability/otlp/metrics`, `POST /v1/observability/otlp/export-metrics` | `crates/spanda-ops/src/otlp_metrics.rs`, `scripts/enterprise_ops_smoke.sh` |
| Fleet agent interpreter recovery | HTTP deploy + `/v1/recovery/execute` | `scripts/fleet_agent_recovery_smoke.sh`, `crates/spanda-fleet/tests/mesh_integration.rs` |
| Operational drift (full) | program + agent dimensions | `crates/spanda-config/src/operational_drift.rs` |
| Platform maturity | graph, drift, gates, trust, tamper, compliance | crate tests under `spanda-graph`, `spanda-config`, `spanda-readiness`, `spanda-trust`, `spanda-tamper`, `spanda-compliance` |
| Enterprise ops API | Control Center handlers, device pool | `crates/spanda-api/tests/` |
| Negative | `ai_safety_violation.sd` fails | `tests/integration.rs` |

**Current count:** ~115+ Rust unit/integration tests (workspace total grows with platform crates).

## TypeScript (`npm test`)

| Area | Status |
|------|--------|
| Lexer, parser, typechecker | Passing |
| Foundations + phases 4–7 | enum, struct literal, trait impl, twin replay |
| Runtime hardening | contracts, capabilities, verify |
| Golden (Rust CLI) | `tests/golden/rust.test.ts` |
| LSP diagnostics | `tests/lsp.test.ts` via `spanda check` + `spanda verify` |
| Mission continuity mirror | `tests/mission-continuity.test.ts`, `tests/continuity-diagnostics.test.ts` |

**Current count:** 121+ vitest tests.

## CLI smoke scripts

| Script | Coverage |
|--------|----------|
| `scripts/showcase_smoke.sh` | Bundled demos (continuity, maturity, enterprise ops, policy, trust, …) |
| `scripts/continuity_smoke.sh` | Continuity CLI + demo |
| `scripts/policy_smoke.sh` | Verify-time policy |
| `scripts/policy_runtime_smoke.sh` | Runtime `--enforce-policy` |
| `scripts/maturity_smoke.sh` | Graph, explain, trust, deploy gate |
| `scripts/enterprise_ops_smoke.sh` | Control Center E1–E4 API surface (compliance catalog, report schedules, discovery TLS, audit prep) |
| `scripts/field_soak_gate.sh` | 30-day field pilot gate before Stable promotion |
| `scripts/spatial_computing_smoke.sh` | Spatial Computing blueprint (human registry, readiness, examples) |
| `scripts/hri_stable_promotion_gate.sh` | HRI Stable promotion (soak + audit prep + spatial smoke + Control Center HRI API probe) |
| `scripts/adas_smoke.sh` | ADAS Solution Blueprint (verify, readiness, replay, compliance, examples) |
| `scripts/adas_stable_promotion_gate.sh` | ADAS Stable promotion (soak + audit prep + smoke + Control Center ADAS API probe) |
| `scripts/adas_automotive_sensors_smoke.sh` | Automotive sensor hub + live `SPANDA_*_CMD` bridge tests |
| `scripts/hri_field_soak_init.sh` | Start 30-day HRI field soak clock |
| `scripts/hri_security_audit_prep.sh` | HRI security audit intake artifact |
| `scripts/security_audit_prep.sh` | Third-party audit intake artifact |
| `scripts/verify_sdk_publish_ready.sh` | PyPI + npm pack readiness (no publish) |

## CLI verification

```bash
cargo test -p spanda-core --test hardware_compat
spanda verify examples/hardware/rover_deploy.sd
spanda verify examples/hardware/full_compat.sd   # expect incompatible (ESP32 in matrix)
spanda readiness examples/showcase/policy/warehouse.sd --policy WarehousePolicy
```

## CI

`.github/workflows/ci.yml`: TypeScript tests, Rust tests, WASM + web build, enterprise ops smoke (when enabled).

## Acceptance criteria per feature

Each feature merges when:

- Rust unit + integration tests pass
- New examples in `examples/` compile; hardware examples verify as expected
- Relevant `docs/` updated
- Golden manifest updated for stable fixtures (when applicable)
- Smoke script added or extended for user-visible CLI paths

## Future tests

1. Verify JSON output schema conformance (`api-contract.json`)
2. LSP verify diagnostic golden files
3. Per-fault simulation coverage matrix
4. Cross-profile deploy matrix CI job (`--all-targets` on main examples)
5. Multi-process fleet agent field validation — **shipped** (`scripts/fleet_field_validation.sh`)
