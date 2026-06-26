# Mission tampering showcase

Integrity verification detects mission hash drift since an approved baseline.

## Commands

```bash
spanda integrity examples/showcase/mission_tampering/approved.sd \
  --baseline examples/showcase/mission_tampering/approved.sd

spanda integrity examples/showcase/mission_tampering/modified.sd \
  --baseline examples/showcase/mission_tampering/approved.sd
```

The modified program changes patrol speed; integrity compare against the approved baseline should fail.

## One command

```bash
spanda demo trust
```

Smoke: `scripts/trust_showcase_smoke.sh`

See [docs/integrity-verification.md](../../../docs/integrity-verification.md).
