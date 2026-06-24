# Anomaly Detection

Anomaly detection declares **expected behavior** and wires **automated reactions**.

## Syntax

```spanda
anomaly_detector NavigationAnomaly {
    expected gps.accuracy <= 3 m;
    expected localization.confidence >= 0.85;
}

on anomaly NavigationAnomaly severity High {
    diagnose root_cause;
    enter degraded_mode;
    audit.record("navigation_anomaly");
}
```

## Core types

| Type | Role |
|------|------|
| `Anomaly` | Detected deviation with severity |
| `AnomalyDetector` | Static expected-behavior model |
| `ExpectedBehaviorModel` | Declared thresholds |
| `LearnedBehaviorModel` | Optional ML backend (package) |
| `AnomalySeverity` | Low / Medium / High / Critical |

## CLI

```bash
spanda anomaly scan rover.sd [--json]
```

Integrates with existing **health checks** — failed health checks surface as anomalies without duplicating health evaluation.

**Learned backends:** import `assurance.anomaly` (or another anomaly package) to mark detectors as ML-backed via `learned_models()` static analysis.

## Runtime

Program-level `state_estimator` declarations register fusion bindings at robot setup. A single estimator aliases `fusion` (same as `observe { }`); named estimators are available as `{Name}.read()`.

## Package

Heavy detection algorithms: **`spanda-anomaly`** (`assurance.anomaly`).

## Example

See `examples/anomaly/navigation_anomaly.sd`.
