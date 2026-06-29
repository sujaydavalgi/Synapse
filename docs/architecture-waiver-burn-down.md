# Architecture Waiver Burn-Down Plan

Incremental refactor plan for Platform Architecture v2.0 baseline waivers.

**Parent:** [platform-architecture.md](./platform-architecture.md) · [dependency-rules.md](./dependency-rules.md)

---

## Current baseline (v2.0)

| Category | Waived | CI policy |
|----------|--------|-----------|
| Rust upward dependencies | 40 | Fail on new edges |
| Rust SCC (`ARCH-SCC-001`) | 27 crates | Fail on new SCC members |
| TypeScript upward imports | 37 | Fail on new edges |
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

## Phase 3 — Slim interpreter upward edges

**Target:** Reduce `spanda-interpreter` platform-service imports (ARCH-004–ARCH-012)

| Step | Action |
|------|--------|
| 1 | Introduce `RuntimeHooks` trait at runtime layer for optional assurance/telemetry/tamper |
| 2 | Wire hooks from CLI/API layer, not inside interpreter core |
| 3 | Remove direct `spanda-assurance`, `spanda-tamper`, `spanda-policy` deps from interpreter |

**Success:** Interpreter depends on core platform + runtime only; services injected at interface layer.

---

## Phase 4 — Dissolve `ARCH-SCC-001`

**Target:** Split the 27-crate SCC into acyclic layers

Recommended order:

1. **Config boundary** — entity types and read-only config snapshots passed into runtime; no runtime → config write path in hot loop
2. **Transport leaf** — transport backends depend on traits only; routing depends on backends, not vice versa
3. **Readiness as consumer** — readiness reads entity/registry snapshots; does not pull parser/driver into evaluation core
4. **Verify pipeline** — single orchestration crate at interface layer (`spanda` CLI / `spanda-api`) composes driver + readiness + certify

**Success:** `find_scc_cycles` reports no component containing both `spanda-driver` and `spanda-readiness`.

---

## Phase 5 — TypeScript mirror alignment

**Target:** Remove `TS-ARCH-*` waivers (37 edges)

| Step | Action |
|------|--------|
| 1 | Split `src/compile.ts` from runtime execution path (mirror Rust driver boundary) |
| 2 | Move `types/checker` runtime imports behind type-only barrels at compiler layer |
| 3 | Relocate deploy/fleet modules that import readiness to interface-layer entry points |

**Success:** Zero TypeScript layer violations without waivers.

---

## Phase 6 — Event model adoption

**Target:** Subsystems emit `spanda_audit::PlatformEvent` envelopes

| Subsystem | Events | Status |
|-----------|--------|--------|
| Entity API mutations | `EntityCreated`, `EntityTagged`, `EntityRelated`, `EntityUpdated` | **Shipped** (`spanda-api` → `record_platform_event`) |
| Readiness | `ReadinessChanged` | **Shipped** (`spanda-readiness` → Control Center audit on entity readiness GET) |
| Interpreter | `MissionStarted`, `MissionCompleted` | **Shipped** (when program declares `audit` block) |
| Telemetry store | Persist all platform events | **Shipped** (`record_platform_event` + `TelemetryEvent::Platform`) |

See [event-model.md](./event-model.md).

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

Goal for v2.1: reduce Rust upward waivers by 25%. Goal for v3.0: eliminate `ARCH-SCC-001`.
