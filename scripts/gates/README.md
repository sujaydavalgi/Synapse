# CI gates & smoke scripts

Index of regression and promotion gates organized by **Platform Pillar** and **Solution Blueprint**.

**Run all examples:** `./scripts/check_all_examples.sh`

---

## By platform pillar

### Pillar 3 — Verification

| Script | Validates |
|--------|-----------|
| `differentiation_smoke.sh` | Mission contracts, explain, audit, coverage |
| `assurance_smoke.sh` | Assurance CLI suite |
| `continuity_smoke.sh` | Takeover, delegation, succession |
| `self_healing_smoke.sh` | Recovery planner, heal/recover |
| `readiness_smoke.sh` | Readiness engine |
| `decision_explain_smoke.sh` | Explain + decision trace |
| `maturity_smoke.sh` | Platform maturity CLIs (graph, drift, chaos, …) |
| `chaos_smoke.sh` | Chaos injection |
| `scorecard_smoke.sh` | Autonomous systems scorecard |
| `readiness_trends_smoke.sh` | Trend analysis |
| `estimate_smoke.sh` | Resource estimation |
| `diff_smoke.sh` | Mission differencing |
| `adr_smoke.sh` | Architecture decision records |
| `generate_smoke.sh` | AI-assisted generate/suggest |

### Pillar 4 — Device & Fleet

| Script | Validates |
|--------|-----------|
| `fleet_field_validation.sh` | Distributed fleet agents |
| `fleet_mesh_tamper_smoke.sh` | Mesh tamper relay |
| `fleet_agent_recovery_smoke.sh` | Agent recovery execute |
| `fleet_tamper_smoke.sh` | Fleet tamper detection |
| `ota_fleet_execute_smoke.sh` | OTA fleet rollout |
| `ota_fleet_soak.sh` | OTA soak |
| `failover_drill_smoke.sh` | Device pool failover |

### Pillar 5 — Security

| Script | Validates |
|--------|-----------|
| `tamper_smoke.sh` | Tamper detection framework |
| `tamper_policy_smoke.sh` | Runtime tamper policy |
| `tamper_diagnose_smoke.sh` | Tamper diagnosis |
| `secure_boot_smoke.sh` | Secure-boot attestation |
| `security_assurance_smoke.sh` | Security assurance rollup |
| `integrity_smoke.sh` | Integrity verification |
| `spoof_smoke.sh` | GPS/sensor spoofing |
| `attestation_smoke.sh` | Hardware attestation |
| `trust_program_smoke.sh` | Program trust scoring |
| `trust_showcase_smoke.sh` | Trust showcase |
| `bundled_trust_smoke.sh` | Bundled trust packages |
| `package_spoofing_smoke.sh` | Package spoofing detection |
| `policy_smoke.sh` | Policy engine verify |
| `policy_runtime_smoke.sh` | Runtime policy |
| `compliance_smoke.sh` | Compliance export |

### Pillar 6 — Operations

| Script | Validates |
|--------|-----------|
| `enterprise_ops_smoke.sh` | Control Center E1–E4 exit criteria |
| `entity_model_smoke.sh` | Unified entity model read + mutation + traceability APIs |
| `control_center_desktop_smoke.sh` | Tauri desktop shell |
| `telemetry_store_golden_path.sh` | Persistent telemetry |
| `device_pool_perf_bench.sh` | 1000-device perf gate |
| `field_soak_gate.sh` | 30-day field pilot wrapper |
| `security_audit_prep.sh` | Audit prep checklist |

### Pillar 7 — Developer

| Script | Validates |
|--------|-----------|
| `showcase_smoke.sh` | Bundled showcase demos |
| `killer_demo_golden_path.sh` | Flagship killer demo |
| `ci_verify_golden_path.sh` | CI verify workflow |
| `gaps_smoke.sh` | Roadmap gap closure |

### Pillar 8 — Packages

| Script | Validates |
|--------|-----------|
| `registry_golden_path.sh` | Registry install/publish |
| `live_ai_golden_path.sh` | OpenAI/Anthropic/ONNX |
| `live_iot_golden_path.sh` | IoT live bridges |
| `mqtt_golden_path.sh` | MQTT golden path |
| `ros2_golden_path.sh` | ROS2 rclpy bridge |
| `llvm_golden_path.sh` | LLVM codegen |
| `llvm_embedded_golden_path.sh` | aarch64 cross-compile |
| `sync_bundled_registry.sh` | CLI bundle sync |

---

## By solution blueprint

| Blueprint | Script | Promotion gate |
|-----------|--------|----------------|
| **ADAS** | `adas_smoke.sh` | `adas_stable_promotion_gate.sh` |
| **ADAS** | `adas_automotive_sensors_smoke.sh` | Live radar/LiDAR/ultrasonic |
| **ADAS** | `adas_field_soak_init.sh` | Field soak prep |
| **ADAS** | `adas_security_audit_prep.sh` | Audit prep |
| **Spatial HRI** | `spatial_computing_smoke.sh` | — |
| **Spatial HRI** | `hri_stable_promotion_gate.sh` | Stable promotion |
| **Spatial HRI** | `hri_field_soak_init.sh` | Field soak prep |
| **Spatial HRI** | `hri_security_audit_prep.sh` | Audit prep |
| **Critical Infrastructure** | `compliance_smoke.sh` | Compliance profiles |
| **Defense** | `secure_boot_smoke.sh`, `tamper_smoke.sh` | — |
| **Research & Education** | `showcase_smoke.sh`, `killer_demo_golden_path.sh` | — |

---

## Stable promotion gates

| Gate | Script | Doc |
|------|--------|-----|
| Enterprise ops Stable | `field_soak_gate.sh` | [field-soak-gate.md](../../docs/field-soak-gate.md) |
| ADAS Stable | `adas_stable_promotion_gate.sh` | [stable-hardening-adas.md](../../docs/stable-hardening-adas.md) |
| HRI Stable | `hri_stable_promotion_gate.sh` | [stable-hardening-human-interaction.md](../../docs/stable-hardening-human-interaction.md) |

---

## Related

- [ROADMAP.md](../../ROADMAP.md) · [docs/pillars/](../../docs/pillars/README.md)
- [tier-3-golden-paths.md](../../docs/tier-3-golden-paths.md)
- [roadmap-codebase-audit-2026-06.md](../../docs/roadmap-codebase-audit-2026-06.md)
