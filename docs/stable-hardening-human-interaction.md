# Human Interaction & Spatial Computing — Stable Hardening Checklist

Phases **H1–H4** are shipped at **Experimental** tier with CI smoke (`scripts/spatial_computing_smoke.sh`). This checklist tracks promotion gates before moving the Human Interaction pillar to **Stable** (target: post field soak, v1.0).

**Related:** [human-interaction-spatial-computing-roadmap.md](./human-interaction-spatial-computing-roadmap.md) · [feature-status.md](./feature-status.md) · [control-center.md](./control-center.md) · [field-soak-gate.md](./field-soak-gate.md)

---

## Promotion criteria

| Gate | Requirement | Status |
|------|-------------|--------|
| Blueprint smoke | `scripts/spatial_computing_smoke.sh` green on `main` | **Shipped** |
| Stable gate script | `scripts/hri_stable_promotion_gate.sh` (soak + smoke + API probe) | **Shipped** |
| Human registry | Fleet + flat `[[humans]]` / wearables / AR / VR in `spanda-config` | **Shipped** |
| Operator capabilities | Registry + verify traceability (excluded from robot minimum) | **Shipped** |
| Human readiness | `human_collaboration` profile + CLI | **Shipped** |
| H2 packages | Wearable + spatial session registry stubs + provider traits | **Shipped** (experimental) |
| H3 packages | Voice / gesture / eye + `/v1/hri/sessions` API | **Shipped** (experimental) |
| H4 UI | Embedded HTML Humans tab + `@spanda/web` `ControlCenterPanel` parity | **Shipped** |
| H5 APIs | Team readiness, collaboration graph, hazard zones / context | **Shipped** (experimental) |
| H6 APIs | Human twins, mission approval queue, vendor live backends | **Shipped** (experimental) |
| Health opt-in | `HumanHealthGate` — config + `SPANDA_HUMAN_HEALTH_ENABLED` | **Shipped** |
| OpenAPI | `/v1/humans`, `/wearables`, `/human-health/policy`, HRI sessions documented | **Shipped** (parity CI) |
| Field soak | 30-day HRI pilot without regression | **Pending** — `SPANDA_HRI_FIELD_SOAK_START_FILE` (default `.spanda/hri-field-soak-start.txt`) |
| Security audit | Third-party review of health opt-in + AR session RBAC | **Pending** — run `./scripts/hri_security_audit_prep.sh` then [security-audit-third-party.md](./security-audit-third-party.md) |
| Vendor SDK bindings | Real HoloLens / HealthKit / ARKit backends (optional packages) | **Planned** — stubs sufficient for Stable platform tier |

---

## Running the promotion gate

```bash
# Start 30-day pilot clock (UTC) — one-time; do not reset during pilot
./scripts/hri_field_soak_init.sh

# Generate HRI security audit intake artifact for reviewers
./scripts/hri_security_audit_prep.sh

# After soak period (or CI without soak/audit):
chmod +x scripts/hri_stable_promotion_gate.sh
./scripts/hri_stable_promotion_gate.sh

# CI / local dev without waiting for soak or audit artifact:
SPANDA_HRI_SKIP_SOAK=1 SPANDA_HRI_SKIP_AUDIT=1 ./scripts/hri_stable_promotion_gate.sh
```

The gate runs:

1. Field soak check (unless `SPANDA_HRI_SKIP_SOAK=1`)
2. `scripts/spatial_computing_smoke.sh`
3. HRI unit tests (`spanda-api`, `spanda-security`, `spanda-providers`)
4. Live Control Center probe against the spatial-computing blueprint (`/v1/humans`, `/v1/humans/readiness`, `/v1/wearables`, `/v1/human-health/policy`, `/v1/hri/sessions`, `/v1/hri/collaboration`, `/v1/hri/context`, per-operator readiness)

---

## Control Center UI parity

| Surface | Humans tab | Endpoints |
|---------|------------|-----------|
| Embedded HTML (`spanda-api` static UI) | **Shipped** | Same REST v1 |
| `@spanda/web` `ControlCenterPanel` | **Shipped** | Same REST v1 |
| `@spanda/control-center-desktop` | Inherits web panel | API via `spanda control-center serve` |

Launch for manual verification:

```bash
spanda control-center serve \
  --config examples/solutions/spatial-computing/spanda.toml \
  --program examples/solutions/spatial-computing/warehouse-ar/pick_mission.sd
```

Open **Humans** tab (embedded UI) or `@spanda/web` Control Center view with `VITE_CONTROL_CENTER_URL`.

---

## Remaining before Stable tier label

1. **30-day HRI field soak** — separate clock from enterprise ops ([field-soak-gate.md](./field-soak-gate.md))
2. **Security audit sign-off** — health telemetry opt-in and AR annotation RBAC
3. **Product sign-off** — promote `docs/feature-status.md` Human Interaction row from Experimental → Stable after gates pass

Do **not** rename registry package stubs to Stable until vendor backends exist; platform APIs and blueprint may reach Stable independently.
