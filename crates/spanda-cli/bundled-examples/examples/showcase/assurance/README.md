# Mission assurance showcase

Mission-grade autonomous operations in one program: knowledge model, state estimation, anomaly detection, prognostics, mitigation, resilience, and assurance evidence.

Uses **`learned backend assurance.anomaly`** for the learned-detector package hook (no `import` required for standalone `.sd` examples). For weighted-fusion package APIs see **`spanda-fusion`** (`assurance.fusion`). Minimal learned-only variant: [learned_navigation.sd](../../anomaly/learned_navigation.sd).

## One command

```bash
spanda demo assurance
```

## Manual path

```bash
spanda check examples/showcase/assurance/rover.sd
spanda assure examples/showcase/assurance/rover.sd --json
spanda anomaly scan examples/showcase/assurance/rover.sd
spanda state estimate examples/showcase/assurance/rover.sd
spanda prognostics examples/showcase/assurance/rover.sd
spanda mission verify examples/showcase/assurance/rover.sd
spanda resilience check examples/showcase/assurance/rover.sd
spanda mitigation plan examples/showcase/assurance/rover.sd
spanda readiness examples/showcase/assurance/rover.sd --target RoverV1 --json
```

Optional ONNX anomaly model: `SPANDA_ANOMALY_ONNX_MODEL_PATH=/path/to/model.onnx` (2-input tensor: observed, volatility).

See [docs/mission-assurance.md](../../../docs/mission-assurance.md).
