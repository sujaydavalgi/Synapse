# Anomaly Detection

Anomaly detection declares **expected behavior** and wires **automated reactions**.

## Syntax

```spanda
anomaly_detector NavigationAnomaly {
    expected gps.accuracy <= 3 m;
    expected localization.confidence >= 0.85;
}

anomaly_detector NavigationML {
    learned backend assurance.anomaly;
    expected localization.confidence >= 0.80;
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

**Learned backends:** declare `learned backend <module>;` on a detector, or import `assurance.anomaly` to apply the package backend to all detectors. Reports include a `learned` section from `learned_models()` / `spanda anomaly scan`.

## Runtime

During health polling, detectors with `learned backend` invoke the package provider (`assurance.anomaly::scan_learned`) with observed confidence and EMA volatility. Scores above zero add the detector to the anomaly trigger set and fire matching `on anomaly` handlers.

Program-level `state_estimator` declarations register fusion bindings at robot setup. A single estimator aliases `fusion` (same as `observe { }`); named estimators are available as `{Name}.read()`.

## Package

Heavy detection algorithms: **`spanda-anomaly`** (`assurance.anomaly`).

## Example

See `examples/anomaly/navigation_anomaly.sd` and `examples/anomaly/learned_navigation.sd`.
