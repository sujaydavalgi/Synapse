# Architecture Waiver Burn-Down Plan

Completed incremental refactor plan for Platform Architecture v2.0 — **all production upward waivers eliminated** (Phase 8).

**Parent:** [platform-architecture.md](./platform-architecture.md) · [dependency-rules.md](./dependency-rules.md)

---

## Current baseline (v2.0)

| Category | Waived | CI policy |
|----------|--------|-----------|
| Rust upward dependencies | 0 | Fail on new edges |
| Rust SCC (`ARCH-SCC-001`) | 0 (dissolved) | Fail on any production SCC |
| TypeScript upward imports | 0 | Fail on new edges |
| Blueprint paths | 8 roots | Fail on forbidden artifacts |

Validation: `python3 scripts/validate_architecture.py --check-manifest-sync` and `python3 scripts/validate_blueprints.py`.

---

## Phase 1 — Documentation and enforcement (complete)

- [x] Layer ownership matrix and manifest
- [x] CI regression gates (Rust + TypeScript + manifest sync + blueprints)
- [x] Common `PlatformEvent` envelope in `spanda-audit`
- [x] Waiver tickets (`ARCH-*`, `TS-ARCH-*`, `ARCH-SCC-001`)

---

## Phase 2 — Break the driver ↔ certify cycle (complete)

**Closed:** `ARCH-001`, `ARCH-C01`

| Step | Action | Status |
|------|--------|--------|
| 1 | Move `build_deploy_plan` certification wrapper to `spanda-ota` | Done |
| 2 | Remove `spanda-certify` from `spanda-driver`; runtime gate via `spanda-assurance/certify-runtime` | Done |
| 3 | Merge certification verify items in `spanda-config::verify_with_system_config` | Done |

**Success:** No production `spanda-driver` ↔ `spanda-certify` cycle; dev-test compile path may still use `spanda-driver` from certify tests only.

---

## Phase 3 — Slim interpreter upward edges (done)

**Target:** Reduce `spanda-interpreter` platform-service imports (ARCH-004–ARCH-012)

| Step | Action | Status |
|------|--------|--------|
| 1 | Introduce `RuntimeHooks` trait at runtime layer for optional assurance/telemetry/tamper | Done (`spanda-runtime::hooks`) |
| 2 | Move operational policy + tamper policy runtime to `spanda-runtime`; wire certify hooks from CLI | Done |
| 3 | Remove direct `spanda-assurance`, `spanda-tamper`, `spanda-policy` deps from interpreter | Done — policy/tamper removed in Phase 3a; assurance decoupled via `AssuranceRuntime` trait and `AssuranceBackedRuntime` bridge (Phase 3b) |
| 4 | Decouple config, capability health, telemetry, providers, faults, security, and transport via runtime traits | Done — `TelemetrySink`, `ProviderRuntime`, `FaultRuntime`, `SecurityRuntime`, `CommBusHost`; CLI/fleet bridges (Phase 3c–3d) |

**Closed waivers:** `ARCH-004`–`ARCH-012`, `ARCH-214`, `ARCH-213`. **SCC:** `ARCH-SCC-001`, `ARCH-C02`, `ARCH-C03` dissolved (production graph acyclic).

**Remaining:** None. All interpreter upward edges and platform-service composition edges closed in Phases 3–8.

**Success:** Interpreter depends on core platform + runtime only; services injected at interface layer.

---

## Phase 4 — Dissolve `ARCH-SCC-001` (done)

**Target:** Split the compile-run-verify mesh SCC into acyclic layers

**Resolution:** Production `[dependencies]` graph is already acyclic (`driver` and `readiness` are not in the same SCC). The former `ARCH-SCC-001` waiver was an artifact of counting `[dev-dependencies]` in `validate_architecture.py`. CI now tracks production deps only; `circular_dependency_waivers` is empty.

| Step | Action | Status |
|------|--------|--------|
| 1 | Exclude dev/build deps from architecture graph | Done |
| 2 | Remove `ARCH-SCC-001`, `ARCH-C02`, `ARCH-C03` waivers | Done |
| 3 | Move fleet recovery integration tests to `spanda-fleet`; close `ARCH-213` | Done |

**Success:** `find_scc_cycles` reports no waived SCC on the production graph.

---

## Phase 5 — TypeScript mirror alignment (complete)

**Target:** Remove `TS-ARCH-*` waivers (37 edges at baseline)

| Step | Action | Status |
|------|--------|--------|
| 1 | Split `src/compile.ts` from runtime execution path (`src/cli/run-program.ts`) | Done (Phase 5a) |
| 2 | Inject `TelemetrySink`, `SecurityRuntime`, `ProviderRuntime`, `AdapterRuntime` at CLI boundary | Done (Phase 5a) |
| 3 | Move `types/checker` runtime imports behind `CheckerHost` at compiler layer | Done (Phase 5b) |
| 4 | Split comm/assurance/readiness types to compiler-layer modules; inject deploy/fleet/certify/verify hosts at CLI | Done (Phase 5b) |

**Closed waivers (Phase 5a):** `TS-ARCH-002`–`TS-ARCH-006`, `TS-ARCH-021`–`TS-ARCH-028`.

**Closed waivers (Phase 5b):** `TS-ARCH-001`, `TS-ARCH-007`–`TS-ARCH-020`, `TS-ARCH-029`–`TS-ARCH-037`.

**Success:** Zero TypeScript layer violations without waivers.

---

## Phase 6 — Event model adoption (complete)

**Target:** Subsystems emit `spanda_audit::PlatformEvent` envelopes and persist via `publish_platform_event`.

| Subsystem | Events | Status |
|-----------|--------|--------|
| Entity API mutations | `EntityCreated`, `EntityTagged`, `EntityRelated`, `EntityUpdated` | **Shipped** |
| Readiness | `ReadinessChanged`, `ReadinessGateFailed` | **Shipped** |
| Health | `HealthChanged`, `HealthCheckFailed`, `DegradedModeEntered` | **Shipped** (entity health evaluation) |
| Interpreter missions | `MissionStarted`, `MissionCompleted`, `MissionPaused`, `MissionAborted` | **Shipped** |
| Recovery | `RecoveryTriggered`, `RecoveryCompleted`, `RecoveryFailed` | **Shipped** (runtime recovery execution) |
| Trust | `TrustUpdated`, `TrustGateFailed` | **Shipped** |
| Tamper | `TamperDetected` | **Shipped** |
| Spoofing | `SpoofingDetected` | **Shipped** (spoof-check trace/program analysis) |
| Security | `AuthFailed`, `SecretRotated` | **Shipped** (API RBAC + managed vault rotation) |
| Fleet orchestration | `FleetMemberJoined`, `FleetMemberLeft` | **Shipped** |
| OTA rollouts | `OtaRolloutStarted`, `OtaRolloutCompleted` | **Shipped** |
| Packages | `PackageInstalled`, `PackageVerified`, `PackageRemoved` | **Shipped** (install/verify/remove CLI + provider bootstrap) |
| Telemetry store | Persist all platform events | **Shipped** |

See [event-model.md](./event-model.md).

---

## Phase 7 — Rust upward waiver burn-down (complete)

**Target:** Reduce remaining `ARCH-*` production upward edges (16 at baseline after Phase 7c)

| Step | Action | Status |
|------|--------|--------|
| 1 | Remove `spanda-runtime` from `spanda-error`; lift `RuntimeError` → `SpandaError` in `spanda-runtime` | Done — closed `ARCH-208` |
| 2 | Remove dev-only orphan waivers (`ARCH-013`–`ARCH-014`, `ARCH-018`, `ARCH-212`) and fix corrupted `ARCH-013` YAML | Done |
| 3 | Move `WireCryptoSession` to `spanda-runtime`; decouple `spanda-transport*` from `spanda-security` | Done — closed `ARCH-217`–`ARCH-221` |
| 4 | Decouple `spanda-providers` from audit/telemetry-store via `DeviceTelemetrySink` + inline ledger stub | Done — closed `ARCH-025`–`ARCH-026` |
| 5 | Point formatter/parser/config at compiler-layer types (`ARCH-211`, `ARCH-017`, `ARCH-202`) | Done — closed 3 edges (**19 → 16**) |

## Phase 8 — Final waiver elimination (complete)

**Target:** Close all remaining 16 production upward waivers → 0.

| Step | Action | Tickets closed |
|------|--------|----------------|
| 1 | `spanda-codegen` calls `lexer`→`parser` directly; remove `spanda-driver` dep | `ARCH-201` |
| 2 | `spanda-connectivity-runtime` removes `spanda-hardware`; `HardwareProfile`/`CompatItem` moved to `spanda-connectivity`; validation logic to connectivity-runtime | `ARCH-203` |
| 3 | `spanda-core` removes `spanda-security` prod dep; gut security.rs/security_validate.rs | `ARCH-204` |
| 4 | `spanda-docs` removes `spanda-lib-registry` and `spanda-runtime-host`; `generate_language_reference` accepts `&dyn TypeCheckHost` + libraries slice | `ARCH-205`, `ARCH-206` |
| 5 | `spanda-driver` removes `spanda-hardware`; verify moved to `spanda-core::hardware_verify` | `ARCH-003` |
| 6 | `spanda-driver` removes `spanda-ota`; deploys via `spanda_ota::build_deploy_plan` | `ARCH-207` |
| 7 | `spanda-fleet` removes `spanda-assurance`; uses `platform_assurance_runtime()` OnceLock | `ARCH-022` |
| 8 | `spanda-fleet` removes `spanda-readiness`; uses `readiness_runtime()` OnceLock | `ARCH-021` |
| 9 | `spanda-fleet` removes `spanda-tamper`; fleet tamper mesh uses `fleet_tamper_runtime()` OnceLock | `ARCH-209` |
| 10 | `spanda-fleet` removes `spanda-telemetry-store`; telemetry mesh uses `fleet_telemetry_runtime()` OnceLock | `ARCH-210` |
| 11 | `spanda-llvm` moves `spanda-driver` to dev-deps only | `ARCH-215` |
| 12 | `spanda-ota` removes `spanda-readiness`; uses `readiness_runtime()` OnceLock | `ARCH-024` |
| 13 | `spanda-ota` removes `spanda-telemetry-store`; uses `device_telemetry_sink()` OnceLock | `ARCH-216` |
| 14 | `spanda-runtime-host` removes `spanda-package` and `spanda-security`; uses `import_catalog` and `security_capabilities` from `spanda-typecheck` | `ARCH-019`, `ARCH-020` |

**Result:** 16 → **0** production upward waivers. Architecture manifest cleaned of all orphan entries (`ARCH-002`, `ARCH-015`, `ARCH-016`, `ARCH-023`, `ARCH-027`–`ARCH-036`).

**Success:** Monotonic reduction of Rust upward waivers without new violations.

---

## Waiver review process

1. Open PR that removes a dependency edge or SCC member
2. Delete corresponding waiver from `scripts/architecture-manifest.yaml`
3. Run `scripts/sync_architecture_manifest.sh`
4. Confirm `validate_architecture.py` passes
5. Reference ticket closure in PR description

**Do not** add waivers without architecture review and a ticket ID.

---

## Metrics

Track over time:

```bash
python3 scripts/validate_architecture.py --verbose | rg 'waived'
```

Goal for v2.1: reduce Rust upward waivers by 25% (**achieved**). ~~Goal for v3.0: eliminate `ARCH-SCC-001`.~~ **Done** — production graph has zero SCCs. ~~Goal for v3.1: eliminate all remaining 16 upward waivers.~~ **Done** — 0 production upward waivers as of Phase 8.
