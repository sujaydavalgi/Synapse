# Secure boot showcase

Secure-boot contract imports (`trust.jetson`, `trust.pi`) with registry package trust and optional live TPM attestation.

## Commands

```bash
spanda tamper-check examples/showcase/secure_boot/rover.sd
spanda verify examples/showcase/secure_boot/rover.sd
```

`spanda demo trust` sets `SPANDA_REGISTRY_URL` to the bundled trust registry automatically. For manual runs from a full clone:

```bash
export SPANDA_REGISTRY_URL="file://$(pwd)/registry"
```

TPM stub backends:

```bash
SPANDA_TPM_BACKEND=file \
SPANDA_TPM_QUOTE_PATH=examples/showcase/secure_boot/fixtures/jetson-tpm-quote.json \
spanda tamper-check examples/showcase/secure_boot/rover.sd

# tpm2-tools PCR quote (when TPM available)
SPANDA_TPM_BACKEND=tpm2 spanda tamper-check examples/showcase/secure_boot/rover.sd
SPANDA_TPM_BACKEND=script \
SPANDA_TPM_SCRIPT=examples/showcase/secure_boot/fixtures/tpm2-quote.sh \
spanda tamper-check examples/showcase/secure_boot/rover.sd
```

## One command

```bash
spanda demo trust
```

Smoke: `scripts/secure_boot_smoke.sh` · `scripts/attestation_smoke.sh`

See [docs/hardware-attestation.md](../../../docs/hardware-attestation.md).
