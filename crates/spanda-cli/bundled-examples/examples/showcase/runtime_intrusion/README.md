# Runtime intrusion showcase

Runtime tamper analysis on mission traces — unexpected capability usage and security audit events.

## Commands

```bash
spanda tamper-check examples/showcase/runtime_intrusion/intrusion.trace
spanda diagnose tamper examples/showcase/runtime_intrusion/intrusion.trace
```

The trace records capability denials and should fail runtime tamper-check.

## One command

```bash
spanda demo trust
```

Smoke: `scripts/trust_showcase_smoke.sh` · `scripts/tamper_diagnose_smoke.sh`

See [docs/tamper-detection.md](../../../docs/tamper-detection.md).
