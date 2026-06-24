# Showcase — evaluator quick path

Start here when evaluating Spanda as a **professional autonomous-systems platform**.

**5-minute path:** [docs/killer-demo.md](../../docs/killer-demo.md) · **One command:** `spanda demo rover`

---

## One-command demos

| Command | What it runs |
|---------|----------------|
| `spanda demo rover` | Flagship autonomous rover — install, verify, sim, replay |
| `spanda demo safety` | Unsafe AI blocked; safe path passes |
| `spanda demo verify` | Missing Lidar fails; complete robot passes |
| `spanda demo fleet` | Multi-robot fleet simulation |
| `spanda demo health` | Health checks + fault injection |
| `spanda demo readiness` | Operational go/no-go scoring |
| `spanda demo assurance` | Mission assurance CLI suite (`assure`, `anomaly scan`, `state estimate`, …) |
| `spanda demo self-healing` | Recovery policies, heal/recover/sim, fleet recovery |

Set `SPANDA_ROOT` to the repository root if examples are not found.

---

## Showcase directories

| Directory | Demonstrates |
|-----------|----------------|
| [`autonomous_rover/`](./autonomous_rover/) | GPS, MQTT, WiFi, AI planning, safety, verify, replay, audit |
| [`unsafe_ai/`](./unsafe_ai/) | ActionProposal rejected; SafeAction passes |
| [`hardware_verification/`](./hardware_verification/) | Mission needs Lidar; hardware without Lidar fails |
| [`capability_verification/`](./capability_verification/) | Capability exposure, traceability, minimum hardware |
| [`health_monitoring/`](./health_monitoring/) | Robot/sensor health, policies, fault injection |
| [`readiness/`](./readiness/) | Operational readiness scoring |
| [`assurance/`](./assurance/) | Mission assurance declarations and CLI (`spanda demo assurance`) |
| [`assurance/rover.sd`](./assurance/rover.sd) | Flagship assurance program — learned anomaly, state estimation, resilience |
| [`fleet_management/`](./fleet_management/) | Fleet, health requirements, coordination |
| [`replay/`](./replay/) | Record, replay, fault injection |

---

## Three pillars (minimal)

| Pillar | Command |
|--------|---------|
| **Safety** | `spanda check examples/showcase/unsafe_ai/unsafe.sd` (must fail) |
| **Verify** | `spanda verify examples/showcase/hardware_verification/mission_with_lidar.sd` |
| **Sim** | `spanda sim examples/showcase/killer_demo.sd` |

---

## Smoke script

```bash
./scripts/showcase_smoke.sh
```

---

## Supplementary files

| File | Topic |
|------|-------|
| [`killer_demo.sd`](./killer_demo.sd) | Safe hero patrol program |
| [`ai_safety_violation.sd`](./ai_safety_violation.sd) | Legacy single-file unsafe demo |
| [`hardware_compatibility.sd`](./hardware_compatibility.sd) | Deploy + task budgets + faults |
| [`world_model_patrol.sd`](./world_model_patrol.sd) | Observe → fusion → belief |

Browse by capability: [examples/features/README.md](../features/README.md)
