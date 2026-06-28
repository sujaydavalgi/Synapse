# Human Readiness

The **Readiness Engine** extends to human operators, teams, and collaborative missions — same weighted scoring model as robot readiness, with optional health dimensions gated by deployment policy.

**Related:** [readiness.md](./readiness.md) · [operator-capabilities.md](./operator-capabilities.md) · [wearables.md](./wearables.md)

---

## Readiness types

| Type | Scope | Output |
|------|-------|--------|
| **Operator readiness** | Single human | Go/no-go for assignment |
| **Team readiness** | Human group on a mission | Rollup score + blocking members |
| **Mission readiness** | Full collaborative mission | Human + robot + wearable + authorization |

---

## Operator dimensions

| Dimension | Weight (default) | Source |
|-----------|------------------|--------|
| Certification validity | 25% | Human entity `certifications` |
| Capability coverage | 20% | `capabilities` vs mission `requires_capability` |
| Availability | 15% | `availability` field |
| Trust level | 15% | `trust_level` |
| Location / zone | 10% | Assignment zone vs mission zone |
| Permissions | 10% | RBAC matrix |
| Wearable connectivity | 5% | Linked wearable last-seen |
| Health status (optional) | 0% default | Wearable telemetry when enabled |

---

## Team readiness

Team readiness aggregates operator scores:

```bash
spanda readiness src/collaborative_mission.sd --profile human_collaboration --config spanda.toml --json
```

Blocking rules (configurable in `spanda.readiness.toml`):

- Any supervisor-capable member required for `approve_mission`
- Minimum team size for SAR missions
- All members must pass certification dimension

---

## Mission readiness

Mission readiness composes:

1. **Robot readiness** — existing engine
2. **Operator readiness** — human dimensions
3. **Wearable readiness** — connectivity, battery
4. **Mission authorization** — approval queue state
5. **Trust** — composite program + operator trust

```toml
[readiness.profiles.human_collaboration]
min_score = 85
require_supervisor_approval = true
health_enabled = false  # set true only with privacy policy

[readiness.profiles.human_collaboration.weights]
certification = 0.25
capability = 0.20
availability = 0.15
trust = 0.15
location = 0.10
permissions = 0.10
wearable_connectivity = 0.05
```

---

## Optional health dimensions

When `SPANDA_HUMAN_HEALTH_ENABLED=1` and `health_enabled = true` in profile:

| Signal | Wearable capability | Alert |
|--------|---------------------|-------|
| Heart rate | `heart_rate` | Threshold breach |
| Stress | `stress_index` | Supervisor notify |
| Fatigue | `fatigue_hint` | Mission pause recommendation |
| Temperature | `temperature` | Medical workflow |
| Fall detection | `fall_detection` | Emergency response |
| Battery | `battery_level` | Wearable swap |

**Privacy controls:**

- Health data never stored in core by default — packages stream aggregates only
- RBAC `health:read` permission required
- Audit log on every health query
- Connected Healthcare blueprint defines retention policy

---

## CLI

```bash
spanda readiness mission.sd --profile human_collaboration --config spanda.toml
spanda readiness mission.sd --profile human_collaboration --json
spanda control-center readiness run  # includes human rollup (planned API)
```

---

## Control Center

**Operator Readiness** panel shows per-operator and team rollup with dimension breakdown. Failed dimensions link to certification renewal and wearable diagnostics.

See [control-center.md](./control-center.md#human-interaction-dashboard).

---

## ADAS parallel

Driver readiness in the ADAS blueprint ([adas-readiness.md](./adas-readiness.md)) is the automotive specialization of operator readiness — `driver_monitoring` capability and `driver_takeover` continuity.

Human collaboration profiles generalize this pattern for warehouse, healthcare, SAR, and field service.
