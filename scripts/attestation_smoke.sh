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

echo "== tpm vendor backend smoke =="
VENDOR_SCRIPT="${ROOT}/examples/showcase/secure_boot/fixtures/jetson-tpm-vendor.sh"
chmod +x "${VENDOR_SCRIPT}" 2>/dev/null || true
export SPANDA_TPM_BACKEND=vendor
export SPANDA_TPM_VENDOR_SDK="${VENDOR_SCRIPT}"
TAMPER_VENDOR="$(cargo run -p spanda -q -- tamper-check "${ROOT}/examples/showcase/secure_boot/rover.sd" 2>&1 || true)"
echo "$TAMPER_VENDOR" | grep -q "boot_state=verified"
unset SPANDA_TPM_BACKEND SPANDA_TPM_VENDOR_SDK

echo "== remote ak cert chain smoke =="
VENDOR_AK="${ROOT}/examples/showcase/secure_boot/fixtures/vendor-ak-chain.sh"
TRUST_STORE="${ROOT}/examples/showcase/secure_boot/fixtures/trust-store"
chmod +x "${VENDOR_AK}" 2>/dev/null || true
export SPANDA_TPM_BACKEND=vendor
export SPANDA_TPM_VENDOR_SDK="${VENDOR_AK}"
export SPANDA_ATTESTATION_TRUST_STORE="${TRUST_STORE}"
TAMPER_AK="$(cargo run -p spanda -q -- tamper-check "${ROOT}/examples/showcase/secure_boot/rover.sd" 2>&1 || true)"
echo "$TAMPER_AK" | grep -q "ak_chain_verified=true"
unset SPANDA_TPM_BACKEND SPANDA_TPM_VENDOR_SDK SPANDA_ATTESTATION_TRUST_STORE

echo "== tpm2 backend smoke =="
cargo test -p spanda-tamper tpm2_backend_reports_tooling_status -q
cargo test -p spanda-tamper tpm2_script_fixture_emits_quote_json -q

echo "== tpm2 quote helpers =="
cargo test -p spanda-tamper normalize_hex_strips_prefixes -q
cargo test -p spanda-tamper extract_pcr_hex_parses_tpm2_pcrread_output -q

echo "== tpm2 quote script fixture smoke =="
TPM2_SCRIPT="${ROOT}/examples/showcase/secure_boot/fixtures/tpm2-quote.sh"
chmod +x "${TPM2_SCRIPT}"
TPM2_JSON="$(SPANDA_ATTESTATION_CONTRACT=trust.jetson SPANDA_ATTESTATION_PACKAGE=spanda-trust-jetson bash "${TPM2_SCRIPT}")"
echo "$TPM2_JSON" | grep -q '"boot_state"'

echo "attestation smoke ok"
