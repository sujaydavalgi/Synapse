#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# shellcheck source=lib/registry_env.sh
source "${ROOT}/scripts/lib/registry_env.sh"
ensure_spanda_registry_url "$ROOT"

echo "== attestation integration tests =="
cargo test -p spanda-tamper --test attestation_integration -q
cargo test -p spanda-tamper tpm -q
cargo test -p spanda-config agent_drift_detects_missing_secure_boot_attestation -q
cargo test -p spanda-ota --test agent_attestation agent_status_includes_attestation_from_environment -q
cargo test -p spanda-readiness readiness_passes_verified_agent_attestation -q
cargo test -p spanda-readiness readiness_surfaces_missing_agent_attestation -q

echo "== tpm file backend smoke =="
QUOTE="${ROOT}/examples/showcase/secure_boot/fixtures/jetson-tpm-quote.json"
export SPANDA_TPM_BACKEND=file
export SPANDA_TPM_QUOTE_PATH="${QUOTE}"
TAMPER="$(cargo run -p spanda -q -- tamper-check "${ROOT}/examples/showcase/secure_boot/rover.sd" 2>&1 || true)"
echo "$TAMPER" | grep -q "boot_state=verified"
unset SPANDA_TPM_BACKEND SPANDA_TPM_QUOTE_PATH

echo "== tpm vendor script backend smoke =="
VENDOR_SCRIPT="${ROOT}/examples/showcase/secure_boot/fixtures/jetson-tpm-vendor.sh"
export SPANDA_TPM_BACKEND=script
export SPANDA_TPM_SCRIPT="${VENDOR_SCRIPT}"
TAMPER_VENDOR="$(cargo run -p spanda -q -- tamper-check "${ROOT}/examples/showcase/secure_boot/rover.sd" 2>&1 || true)"
echo "$TAMPER_VENDOR" | grep -q "boot_state=verified"
unset SPANDA_TPM_BACKEND SPANDA_TPM_SCRIPT

echo "== tpm2 backend smoke =="
cargo test -p spanda-tamper tpm2_backend_reports_tooling_status -q

echo "attestation smoke ok"
