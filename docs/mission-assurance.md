# Mission Assurance

Spanda provides NASA-style mission assurance as a **lean-core platform layer** integrated with existing readiness, health, traceability, and safety systems.

## Architecture

```
.sd declarations  →  spanda-assurance (core interfaces + static analysis)
                   →  spanda-readiness (operational readiness hub)
                   →  spanda-capability (health, traceability)
                   →  optional packages (spanda-anomaly, spanda-diagnosis, …)
```

Core defines **interfaces and data models**. Heavy algorithms (ML anomaly detection, advanced prognostics) live in optional packages under `packages/registry/`.

## Language constructs

| Construct | Purpose |
|-----------|---------|
| `knowledge_model` | System model, components, dependencies |
| `state_estimator` | Sensor fusion inputs and estimate type |

At runtime, `state_estimator` wires `SensorFusion` bindings after robot sensors are registered. One estimator also aliases `fusion` for parity with `observe { }` programs.
| `anomaly_detector` | Expected behavior bounds |
| `on anomaly …` | Automated reactions |
| `prognostics` | RUL prediction and degradation warnings |
| `mitigation` | Conditional recovery actions |
| `operating_mode` | Normal / degraded / safe / emergency |
| `mission_plan` | Steps and constraints |
| `resilience_policy` | Fault tolerance strategies |
| `assurance_case` | Links evidence sources |

## CLI

```bash
spanda assure rover.sd
spanda anomaly scan rover.sd
spanda state estimate rover.sd
spanda diagnose mission.trace    # or program .sd
spanda prognostics rover.sd
spanda mission verify mission.sd
spanda resilience check rover.sd
```

All commands support `--json`, `--markdown`, and `--html`.

## Reports

- **Assurance report** — evidence, verification, traceability
- **Anomaly report** — detectors, violations, handler coverage
- **Diagnosis report** — root cause, causal graph, trace timeline
- **Prognostics report** — RUL, degradation warnings
- **Mitigation plan** — recovery actions and mode transitions
- **Mission assurance report** — plan verification + readiness mission checks
- **Resilience report** — policies, recovery, readiness score

## Integration

Mission assurance **composes** with:

- `spanda verify` / hardware verification
- `spanda trace` / traceability matrices
- `spanda health` / health checks (not duplicated)
- `spanda readiness` / fleet readiness
- `spanda audit` / provenance
- Digital twin, replay, kill switch

## Examples

- `examples/assurance/rover_assurance.sd`
- `examples/anomaly/navigation_anomaly.sd`
- `examples/diagnostics/gps_failure.sd`
- `examples/prognostics/battery_degradation.sd`
- `examples/resilience/degraded_mode_recovery.sd`
- `examples/mission/mission_assurance.sd`

## Related docs

- [Knowledge models](knowledge-models.md)
- [State estimation](state-estimation.md)
- [Anomaly detection](anomaly-detection.md)
- [Diagnostics](diagnostics.md)
- [Prognostics](prognostics.md)
- [Resilience](resilience.md)
- [Assurance cases](assurance-cases.md)
- [Readiness](readiness.md)
