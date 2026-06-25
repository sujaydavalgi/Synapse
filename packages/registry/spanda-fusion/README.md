# spanda-fusion

Official Spanda package for mission assurance: **weighted sensor fusion**.

Lean-core fusion lives in `spanda-runtime` (`fusion.read()`, `spanda state estimate`). This package exposes the same weighting model for imports and provider dispatch.

## Usage

```spanda
import assurance.fusion;

state_estimator RoverState {
    inputs [gps.fix, imu.data, lidar.scan];
    output StateEstimate;
}
```

Query weights programmatically:

```spanda
let w = assurance.fusion.weight_for_sensor("GPS");
let confidence = assurance.fusion.confidence_for_types("GPS,Lidar,IMU");
```

## Spoofing backend

Exports `spoofing_backend_name()` and `spoofing_contract_version()` for GPS/IMU cross-check integrations with `spanda spoof-check`.

See `examples/showcase/assurance/rover.sd` and [state-estimation.md](../../../docs/state-estimation.md).
