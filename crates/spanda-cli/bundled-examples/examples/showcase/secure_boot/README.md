# Secure boot showcase

Secure-boot contract imports (`trust.jetson`, `trust.pi`) with registry package trust and optional live TPM attestation.

## Commands

```bash
export SPANDA_REGISTRY_URL="file://$(pwd)/registry"
spanda tamper-check examples/showcase/secure_boot/rover.sd
spanda verify examples/showcase/secure_boot/rover.sd
```

TPM stub backends:

```bash
SPANDA_TPM_BACKEND=file \
SPANDA_TPM_QUOTE_PATH=examples/showcase/secure_boot/fixtures/jetson-tpm-quote.json \
spanda tamper-check examples/showcase/secure_boot/rover.sd
```

## One command

```bash
spanda demo trust
```

Smoke: `scripts/secure_boot_smoke.sh` · `scripts/attestation_smoke.sh`

See [docs/hardware-attestation.md](../../../docs/hardware-attestation.md).
