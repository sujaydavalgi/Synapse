# Hardware Attestation

**Status:** Experimental · **Phase:** Verify, Deploy, Operate · **Priority:** P3.1

Optional live hardware attestation for secure-boot contract imports (`trust.jetson`, `trust.pi`).

## Verify-time HTTP endpoint

Set `SPANDA_ATTESTATION_ENDPOINT` to an HTTP URL that accepts POST JSON:

```json
{
  "contract": "trust.jetson",
  "package": "spanda-trust-jetson",
  "program": "rover.sd"
}
```

Response:

```json
{
  "attested": true,
  "boot_state": "verified",
  "score": 95,
  "detail": "tpm quote ok"
}
```

When configured, `spanda tamper-check` and `spanda integrity` merge live attestation into secure-boot coverage scores.

## TPM / vendor quote backends

When no HTTP endpoint is set, configure a TPM or vendor stub via `SPANDA_TPM_BACKEND`:

| Backend | Purpose |
|---------|---------|
| `mock` | CI/dev stub — always returns verified quote |
| `jetson` | Jetson vendor stub (same as mock with vendor label) |
| `pi` | Raspberry Pi vendor stub |
| `tpm2` | Run `tpm2_createek` / `tpm2_createak` / `tpm2_quote` on PCR0 when tpm2-tools and a TPM are available; falls back to `tpm2_getcap` probe |
| `file` | Read quote JSON from `SPANDA_TPM_QUOTE_PATH` |
| `script` | Run `SPANDA_TPM_SCRIPT`; stdout must be quote JSON |

Quote JSON schema matches the HTTP response above.

Examples:

```bash
# Mock Jetson TPM quote in tamper-check
SPANDA_TPM_BACKEND=jetson spanda tamper-check examples/showcase/secure_boot/rover.sd

# File-backed quote fixture
SPANDA_TPM_BACKEND=file \
SPANDA_TPM_QUOTE_PATH=examples/showcase/secure_boot/fixtures/jetson-tpm-quote.json \
spanda tamper-check examples/showcase/secure_boot/rover.sd

# Script backend (stdout JSON)
SPANDA_TPM_BACKEND=script \
SPANDA_TPM_SCRIPT=examples/showcase/secure_boot/fixtures/emit-tpm-quote.sh \
spanda tamper-check examples/showcase/secure_boot/rover.sd

# Vendor adapter stubs (Jetson / Pi)
SPANDA_TPM_BACKEND=script \
SPANDA_TPM_SCRIPT=examples/showcase/secure_boot/fixtures/jetson-tpm-vendor.sh \
spanda tamper-check examples/showcase/secure_boot/rover.sd

# Host tpm2-tools quote (when TPM available)
SPANDA_TPM_BACKEND=tpm2 spanda tamper-check examples/showcase/secure_boot/rover.sd

# Shell adapter with the same quote workflow
SPANDA_TPM_BACKEND=script \
SPANDA_TPM_SCRIPT=examples/showcase/secure_boot/fixtures/tpm2-quote.sh \
spanda tamper-check examples/showcase/secure_boot/rover.sd
```

Smoke scripts use `scripts/lib/registry_env.sh` to prefer the bundled trust registry (`crates/spanda-cli/bundled-registry`) when `SPANDA_REGISTRY_URL` is unset. `spanda demo trust` configures this automatically.

HTTP takes precedence when both `SPANDA_ATTESTATION_ENDPOINT` and `SPANDA_TPM_BACKEND` are set.

## Deploy agent status

Deploy agents expose attestation fields on `GET /v1/status` when set via environment:

| Variable | Field |
|----------|-------|
| `SPANDA_ATTESTATION_CONTRACT` | `attestation_contract` |
| `SPANDA_ATTESTATION_VERIFIED=1` | `attestation_verified` |
| `SPANDA_BOOT_STATE` | `boot_state` |

`spanda integrity <file.sd> --agent <Robot@Hardware>` compares attestation when present. `spanda drift <file.sd> --agent <Robot@Hardware>` flags missing or failed attestation when the program imports secure-boot contracts. `spanda readiness <file.sd> --agent <Robot@Hardware>` surfaces attestation drift as readiness issues.

## Packages

- `spanda-trust-jetson` — Jetson secure-boot contract stub
- `spanda-trust-pi` — Raspberry Pi secure-boot contract stub

See [trust-framework.md](./trust-framework.md) · [tamper-detection.md](./tamper-detection.md) · [integrity-verification.md](./integrity-verification.md).
