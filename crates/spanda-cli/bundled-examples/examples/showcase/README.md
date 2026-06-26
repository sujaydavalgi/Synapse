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
| `spanda demo continuity` | Mission continuity, takeover, delegation, succession |
| `spanda demo maturity` | Phase A graph, explain, trust, deployment gates |
| `spanda demo trust` | Tamper/trust showcases — package tampering, integrity, spoofing, runtime intrusion |

Set `SPANDA_ROOT` to the repository root if examples are not found.

---

## Trust & tamper showcases

| Directory | Demonstrates |
|-----------|----------------|
| [`gps_spoofing/`](./gps_spoofing/) | GPS spoofing detection and trace plausibility |
| [`package_tampering/`](./package_tampering/) | Suspicious import lowers trust score |
| [`mission_tampering/`](./mission_tampering/) | Mission hash drift vs approved baseline |
| [`runtime_intrusion/`](./runtime_intrusion/) | Unexpected capability usage in traces |
| [`tamper_policy/`](./tamper_policy/) | Declarative tamper response at runtime |
| [`secure_boot/`](./secure_boot/) | `trust.jetson` / `trust.pi` contracts + attestation |
| [`compliance/`](./compliance/) | Defense and medical compliance profile showcases |
| [`fleet_tamper/`](./fleet_tamper/) | Fleet-wide tamper correlation manifest |

One command: `spanda demo trust` · smoke: `./scripts/showcase_smoke.sh`

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
| [`continuity/`](./continuity/) | Mission continuity — checkpoint resume |
| [`takeover/`](./takeover/) | Hot takeover on failure |
| [`delegation/`](./delegation/) | Mission ownership transfer |
| [`swarm_takeover/`](./swarm_takeover/) | Swarm member lost |
| [`fleet_succession/`](./fleet_succession/) | Fleet successor ranking |
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
