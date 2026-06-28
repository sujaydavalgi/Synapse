#!/usr/bin/env bash
# Smoke remaining platform maturity gaps: vendor TPM SDK, remote AK chain, accreditation export, confidence gates.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# shellcheck source=lib/registry_env.sh
source "${ROOT}/scripts/lib/registry_env.sh"
ensure_spanda_registry_url "$ROOT"

SECURE_BOOT="${ROOT}/examples/showcase/secure_boot/rover.sd"
DEFENSE="${ROOT}/examples/showcase/compliance/defense_rover.sd"
TRUST_STORE="${ROOT}/examples/showcase/secure_boot/fixtures/trust-store"
VENDOR_AK="${ROOT}/examples/showcase/secure_boot/fixtures/vendor-ak-chain.sh"
VENDOR_JETSON="${ROOT}/examples/showcase/secure_boot/fixtures/jetson-tpm-vendor.sh"

chmod +x "${VENDOR_AK}" "${VENDOR_JETSON}" 2>/dev/null || true

echo "== remote attestation unit tests =="
cargo test -p spanda-tamper remote_attestation -q
cargo test -p spanda-tamper attestation::tests::ak_chain_policy -q

echo "== vendor tpm backend smoke =="
export SPANDA_TPM_BACKEND=vendor
export SPANDA_TPM_VENDOR_SDK="${VENDOR_JETSON}"
TAMPER_VENDOR="$(cargo run -p spanda -q -- tamper-check "${SECURE_BOOT}" 2>&1 || true)"
echo "$TAMPER_VENDOR" | grep -q "boot_state=verified"
unset SPANDA_TPM_BACKEND SPANDA_TPM_VENDOR_SDK

echo "== remote ak cert chain smoke =="
export SPANDA_TPM_BACKEND=vendor
export SPANDA_TPM_VENDOR_SDK="${VENDOR_AK}"
export SPANDA_ATTESTATION_TRUST_STORE="${TRUST_STORE}"
TAMPER_AK="$(cargo run -p spanda -q -- tamper-check "${SECURE_BOOT}" 2>&1 || true)"
echo "$TAMPER_AK" | grep -q "ak_chain_verified=true"
unset SPANDA_TPM_BACKEND SPANDA_TPM_VENDOR_SDK SPANDA_ATTESTATION_TRUST_STORE

echo "== compliance accreditation export =="
ACCRED="$(cargo run -p spanda -q -- compliance report "${DEFENSE}" --profile defense 2>&1 || true)"
echo "$ACCRED" | grep -q "template_only"
echo "$ACCRED" | grep -q "Evidence checklist"

echo "== compliance list and iso26262 export =="
LIST="$(cargo run -p spanda -q -- compliance list 2>&1 || true)"
echo "$LIST" | grep -q iso26262
echo "$LIST" | grep -q iso13849
AUTOMOTIVE="${ROOT}/examples/showcase/compliance/automotive_rover.sd"
ACCRED_AUTO="$(cargo run -p spanda -q -- compliance report "${AUTOMOTIVE}" --profile iso26262 2>&1 || true)"
echo "$ACCRED_AUTO" | grep -q "template_only"

echo "== spoofing confidence gate =="
export SPANDA_SPOOFING_MIN_CONFIDENCE=0.99
SPOOF="$(cargo run -p spanda -q -- spoof-check "${ROOT}/examples/showcase/gps_spoofing/spoof.trace" 2>&1 || true)"
echo "$SPOOF" | grep -q "Suppressed low-confidence"
unset SPANDA_SPOOFING_MIN_CONFIDENCE

echo "== spoofing operator confirmation gate =="
SPOOF_OP="$(cargo run -p spanda -q -- spoof-check "${ROOT}/examples/showcase/gps_spoofing/spoof.trace" 2>&1 || true)"
echo "$SPOOF_OP" | grep -q "Operator confirmation required"
unset SPANDA_OPERATOR_APPROVAL

cargo test -p spanda-spoofing confidence -q
cargo test -p spanda-compliance accreditation -q 2>/dev/null || cargo test -p spanda-compliance -q

echo "platform gaps smoke ok"
