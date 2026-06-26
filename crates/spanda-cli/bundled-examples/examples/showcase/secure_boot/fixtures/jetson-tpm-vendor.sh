#!/usr/bin/env bash
# Vendor TPM quote adapter stub for Jetson secure-boot demos.
# Use with: SPANDA_TPM_BACKEND=script SPANDA_TPM_SCRIPT=examples/showcase/secure_boot/fixtures/jetson-tpm-vendor.sh
set -euo pipefail
if command -v tpm2_quote >/dev/null 2>&1; then
  detail="jetson tpm2_quote available (vendor integration stub)"
else
  detail="jetson vendor tpm script stub for ${SPANDA_ATTESTATION_CONTRACT:-trust.jetson}"
fi
cat <<EOF
{"attested":true,"boot_state":"verified","score":96,"detail":"${detail}"}
EOF
