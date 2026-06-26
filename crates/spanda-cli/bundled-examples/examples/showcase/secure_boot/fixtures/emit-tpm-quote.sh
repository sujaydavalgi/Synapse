#!/usr/bin/env bash
# Emit a TPM quote JSON payload for SPANDA_TPM_BACKEND=script demos.
set -euo pipefail
cat <<EOF
{"attested":true,"boot_state":"verified","score":97,"detail":"script tpm quote for ${SPANDA_ATTESTATION_CONTRACT:-unknown}"}
EOF
