# GPS spoofing showcase

Spoofing detection: GPS vs IMU conflict, impossible jumps, and optional ML alert merge.

## Commands

```bash
spanda spoof-check examples/showcase/gps_spoofing/rover.sd
spanda spoof-check examples/showcase/gps_spoofing/spoof.trace
spanda diagnose tamper examples/showcase/gps_spoofing/spoof.trace
```

Optional ML backend:

```bash
export SPANDA_SPOOFING_ML_ALERTS_PATH=examples/showcase/gps_spoofing/fixtures/ml-alerts.json
spanda spoof-check examples/showcase/gps_spoofing/spoof.trace
```

## One command

```bash
spanda demo spoof
spanda demo trust
```

Smoke: `scripts/spoof_smoke.sh`

See [docs/spoofing-detection.md](../../../docs/spoofing-detection.md).
