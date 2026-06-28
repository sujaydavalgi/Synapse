# ADAS & Autonomous Driving — Stable Hardening Checklist

The ADAS Solution Blueprint is shipped at **Experimental** tier with CI smoke (`scripts/adas_smoke.sh`). This checklist tracks promotion gates before moving the blueprint to **Stable**.

**Related:** [solutions/adas.md](./solutions/adas.md) · [feature-status.md](./feature-status.md) · [demo-plan-adas.md](./demo-plan-adas.md) · [field-soak-gate.md](./field-soak-gate.md)

---

## Promotion criteria

| Gate | Requirement | Status |
|------|-------------|--------|
| Blueprint smoke | `scripts/adas_smoke.sh` green on `main` | **Shipped** |
| Stable gate script | `scripts/adas_stable_promotion_gate.sh` (soak + smoke + Control Center probe) | **Shipped** |
| ISO 26262 profile | `spanda verify` + `readiness --profile iso26262` on `highway_drive.sd` | **Shipped** |
| Golden traces | `behavior_tick` + `scheduler_tick` deterministic replay | **Shipped** |
| Automotive packages | `spanda-radar`, `spanda-lidar`, `spanda-ultrasonic`, protocol stubs | **Shipped** (experimental) |
| ROS 2 automotive bridge | `ros2_automotive/automotive_nav.sd` + `spanda-ros2` | **Shipped** (experimental) |
| Grafana ADAS dashboard | `control-center-adas.json` template | **Shipped** |
| Control Center ADAS tab | Embedded UI + dashboard/health/assurance endpoints | **Shipped** |
| Field soak | 30-day ADAS pilot without regression | **Pending** — `.spanda/adas-field-soak-start.txt` |
| Security audit | Third-party review of ISO 26262 readiness gates + CAN/OTA paths | **Pending** — `./scripts/adas_security_audit_prep.sh` |
| Live vehicle I/O | Radar/LiDAR/ultrasonic env bridges (`SPANDA_LIVE_*`, `SPANDA_*_CMD`) | **Shipped** (experimental) — `./scripts/adas_automotive_sensors_smoke.sh` |

---

## Running the promotion gate

```bash
# Start 30-day pilot clock (UTC) — one-time
./scripts/adas_field_soak_init.sh

# Generate ADAS security audit intake artifact
./scripts/adas_security_audit_prep.sh

# After soak period (or CI without soak/audit):
chmod +x scripts/adas_stable_promotion_gate.sh
./scripts/adas_stable_promotion_gate.sh

# CI / local dev without waiting for soak or audit artifact:
SPANDA_ADAS_SKIP_SOAK=1 SPANDA_ADAS_SKIP_AUDIT=1 ./scripts/adas_stable_promotion_gate.sh

# CI after `adas-smoke` (skip duplicate smoke):
SPANDA_ADAS_SKIP_SOAK=1 SPANDA_ADAS_SKIP_AUDIT=1 SPANDA_ADAS_SKIP_SMOKE=1 ./scripts/adas_stable_promotion_gate.sh
```

The gate runs:

1. Field soak check (unless `SPANDA_ADAS_SKIP_SOAK=1`)
2. Security audit prep artifact check (unless `SPANDA_ADAS_SKIP_AUDIT=1`)
3. `scripts/adas_smoke.sh`
4. Live Control Center probe against the ADAS blueprint (`/v1/dashboard`, `/v1/health/summary`, `/v1/assurance/summary`, `/v1/diagnosis/summary`, `/v1/ota/status`, `/v1/trust/package`)

---

## Remaining before Stable tier label

1. **30-day ADAS field soak** — separate clock from enterprise ops and HRI ([field-soak-gate.md](./field-soak-gate.md))
2. **Security audit sign-off** — ISO 26262 readiness enforcement, secure comm, tamper policy on vehicle ECUs
3. **Optional vendor SDK bindings** — production radar/LiDAR firmware adapters beyond env-bridge stubs
