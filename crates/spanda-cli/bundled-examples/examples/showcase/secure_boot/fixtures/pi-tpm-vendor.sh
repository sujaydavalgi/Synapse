#!/usr/bin/env bash
# Vendor TPM quote adapter stub for Raspberry Pi secure-boot demos.
set -euo pipefail
detail="pi vendor tpm script stub for ${SPANDA_ATTESTATION_CONTRACT:-trust.pi}"
cat <<EOF
{"attested":true,"boot_state":"verified","score":95,"detail":"${detail}"}
EOF
