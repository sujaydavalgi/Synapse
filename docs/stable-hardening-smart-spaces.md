# Smart Spaces & Ambient Intelligence — Stable Hardening Checklist

The Smart Spaces Solution Blueprint is shipped at **Stable** tier with CI smoke (`scripts/smart_spaces_smoke.sh`). **Promoted 2026-07-02** after `smart_spaces_promotion_gate.sh`.

**Related:** [solutions/smart-spaces.md](./solutions/smart-spaces.md) · [feature-status.md](./feature-status.md) · [control-center.md](./control-center.md#smart-spaces-dashboard) · [field-soak-gate.md](./field-soak-gate.md)

---

## Promotion criteria

| Gate | Requirement | Status |
|------|-------------|--------|
| Blueprint smoke | `scripts/smart_spaces_smoke.sh` green on `main` | **Shipped** |
| Scaffold promotion gate | `scripts/smart_spaces_promotion_gate.sh` (smoke + API + Control Center probe) | **Shipped** |
| Readiness profile | `spanda readiness --profile smart_space` on blueprint apps | **Shipped** |
| Control Center REST | `/v1/facilities`, readiness, occupancy, energy, emergency, summary | **Shipped** |
| Control Center UI | Smart Spaces tab (buildings, gateways, zones, energy, robots, wearables, trust chart, continuity) | **Shipped** |
| OpenAPI parity | `openapi_parity_tests` documents all Smart Spaces routes | **Shipped** |
| Registry packages | Nine optional packages + provider dispatch stubs | **Shipped** (experimental) |
| Grafana dashboard | `control-center-smart-spaces.json` template | **Shipped** |
| Golden traces | Emergency / mode-change deterministic replay | **Shipped** — fire, gateway failover, power island, water leak fixtures |
| Bundled offline registry | Smart Spaces packages in `bundled-registry` | **Shipped** |
| Live building I/O | BACnet/KNX/Thread/Z-Wave/HA env bridges + bacpypes3/xknx + `live-building` registry scripts | **Shipped** (experimental) — `SPANDA_LIVE_*`, package `read_*.sh` / `get_state.sh`, `scripts/smart_spaces_live_iot_smoke.sh` |
| BMS sidecar bridge | Home Assistant REST + sidecar patterns (MQTT) | **Shipped** (experimental) — [smart-space-bms-bridge.md](./smart-space-bms-bridge.md), `scripts/smart_spaces_bms_sidecar_smoke.sh` |
| Field soak | 30-day smart-building pilot without regression | **Pending** (operational) — `./scripts/smart_spaces_stable_init.sh` |
| Security audit | Third-party review of life-safety and access-control paths | **Pending** (operational) — automated self-audit **shipped** |
| Extended panels API | devices, health, security, environment, floor-map, energy detail | **Shipped** |
| gRPC extended panels | devices, health, security, floor-map, environment, energy detail, gateways | **Shipped** — proto **1.0.5**, **96 RPCs** |
| CI promotion gate | `smart-spaces-promotion-gate` job (API + OpenAPI + live probe) | **Shipped** |
| Blueprint certify metadata | `certify ISO13849` + robot `safety` on all six apps | **Shipped** |
| Fleet orchestrator robots | `fleet.robots` for all blueprint orchestrators | **Shipped** |

---

## Running the promotion gate

```bash
# Start 30-day pilot clock (UTC) — one-time
./scripts/smart_spaces_field_soak_init.sh

# Generate Smart Spaces security audit intake artifact
./scripts/smart_spaces_security_audit_prep.sh

# Scaffold gate (soak/audit skipped by default for experimental tier):
chmod +x scripts/smart_spaces_promotion_gate.sh
./scripts/smart_spaces_promotion_gate.sh

# Full gate after soak and audit artifact:
SPANDA_SMART_SPACES_SKIP_SOAK=0 SPANDA_SMART_SPACES_SKIP_AUDIT=0 ./scripts/smart_spaces_promotion_gate.sh

# CI after smart-spaces-smoke (skip duplicate smoke):
SPANDA_SMART_SPACES_SKIP_SOAK=1 SPANDA_SMART_SPACES_SKIP_AUDIT=1 SPANDA_SMART_SPACES_SKIP_SMOKE=1 ./scripts/smart_spaces_promotion_gate.sh
```

The gate runs:

1. Field soak check (unless `SPANDA_SMART_SPACES_SKIP_SOAK=1`, default **skip**)
2. Security audit prep artifact (unless `SPANDA_SMART_SPACES_SKIP_AUDIT=1`, default **skip**)
3. `scripts/smart_spaces_smoke.sh`
4. `scripts/smart_spaces_live_iot_smoke.sh` and `scripts/smart_spaces_bms_sidecar_smoke.sh`
5. `cargo test -p spanda-providers --features live-building` (BACnet registry script)
6. `cargo test -p spanda-api smart_spaces` and OpenAPI parity
7. Live Control Center probe (`/v1/facilities`, readiness, occupancy, energy, emergency, summary)

---

## Promotion status (2026-07-02)

**Promoted to Stable** in `docs/feature-status.md` and [ROADMAP.md](../ROADMAP.md).

### Ongoing organizational gates

| Gate | Status |
|------|--------|
| 30-day field soak | **Pending** — `./scripts/smart_spaces_field_soak_init.sh` |
| Third-party security audit sign-off | **Pending** — `./scripts/smart_spaces_security_audit_prep.sh` |
| Site-specific protocol tuning | Operational — `bacpypes3` / `xknx` adapters shipped |
