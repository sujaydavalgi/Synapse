# Package tampering showcase

Verify-time trust scoring when a suspicious third-party import is added after approval.

## Commands

```bash
spanda tamper-check examples/showcase/package_tampering/approved.sd
spanda tamper-check examples/showcase/package_tampering/tampered.sd
```

The tampered variant adds `import untrusted.vendor_payload` and should report a lower trust score than the approved baseline.

## One command

```bash
spanda demo trust
```

Smoke: `scripts/trust_showcase_smoke.sh`

See [docs/tamper-detection.md](../../../docs/tamper-detection.md) · [docs/package-trust.md](../../../docs/package-trust.md).
