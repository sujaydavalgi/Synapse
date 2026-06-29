# Enterprise Operations — Stable Promotion Runbook

Operational checklist for promoting enterprise operations pillars from **Experimental** to **Stable** in `docs/feature-status.md`.

**Implementation status:** All per-pillar hardening items in [stable-hardening-enterprise-ops.md](./stable-hardening-enterprise-ops.md) are **shipped** in code and CI.

---

## Automated gate

```bash
chmod +x scripts/enterprise_ops_field_soak_init.sh
chmod +x scripts/enterprise_ops_stable_promotion_gate.sh
chmod +x scripts/security_audit_prep.sh

# One-time: start 30-day soak clock
./scripts/enterprise_ops_field_soak_init.sh

# Generate audit prep packet for reviewers
./scripts/security_audit_prep.sh

# After soak + prep artifact exist (and external audit signed off):
./scripts/enterprise_ops_stable_promotion_gate.sh
```

### CI (implementation checks only)

The `enterprise-ops-promotion-gate` job runs the promotion gate with soak and audit checks skipped:

```bash
SPANDA_ENTERPRISE_OPS_SKIP_SOAK=1 \
SPANDA_ENTERPRISE_OPS_SKIP_AUDIT=1 \
./scripts/enterprise_ops_stable_promotion_gate.sh
```

---

## Environment variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `SPANDA_FIELD_SOAK_START_FILE` | `.spanda/field-soak-start.txt` | UTC soak start (`YYYY-MM-DD`) |
| `SPANDA_FIELD_SOAK_MIN_DAYS` | `30` | Minimum elapsed days |
| `SPANDA_SECURITY_AUDIT_PREP_FILE` | `.spanda/security-audit-prep.json` | Audit prep artifact |
| `SPANDA_ENTERPRISE_OPS_SKIP_SOAK` | `0` | Skip soak elapsed check |
| `SPANDA_ENTERPRISE_OPS_SKIP_AUDIT` | `0` | Skip audit prep file check |
| `SPANDA_ENTERPRISE_OPS_SKIP_SMOKE` | `0` | Skip `enterprise_ops_smoke.sh` |

---

## What the gate runs

1. **Field soak** — 30-day clock ([field-soak-gate.md](./field-soak-gate.md))
2. **Security audit prep** — local artifact from [security_audit_prep.sh](../scripts/security_audit_prep.sh); **external reviewer sign-off** still required ([security-audit-third-party.md](./security-audit-third-party.md))
3. **`enterprise_ops_smoke.sh`** — E1–E4 Control Center API surface
4. **`failover_drill_smoke.sh`** — device pool failover drill
5. **`ota_fleet_soak.sh`** — quick OTA fleet soak (`SPANDA_OTA_FLEET_SOAK_QUICK=1`)

---

## Remaining human gates (not automatable)

| Gate | Action |
|------|--------|
| Third-party audit | Engage reviewer using `security-audit-prep.json` packet; record sign-off in change management |
| Production releases | Publish PyPI/npm/desktop tags per [sdk-publishing.md](./sdk-publishing.md) and [desktop-release-runbook.md](./desktop-release-runbook.md) |
| Feature status | Update `docs/feature-status.md` enterprise operations rows to **Stable** after soak + audit + releases |

---

## Related

- [stable-hardening-enterprise-ops.md](./stable-hardening-enterprise-ops.md) — per-pillar checklist
- [enterprise-operations-roadmap.md](./enterprise-operations-roadmap.md) — E1–E4 scope
- [control-center.md](./control-center.md) — API reference
