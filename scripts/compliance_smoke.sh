#!/usr/bin/env bash
# Smoke compliance profile verification.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
WAREHOUSE="${ROOT}/examples/showcase/policy/warehouse.sd"
DEFENSE="${ROOT}/examples/showcase/compliance/defense_rover.sd"
SECURE_BOOT="${ROOT}/examples/showcase/secure_boot/rover.sd"

# shellcheck source=lib/registry_env.sh
source "${ROOT}/scripts/lib/registry_env.sh"
ensure_spanda_registry_url "$ROOT"
cargo build -p spanda -q

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

echo "== verify warehouse profile =="
run_spanda verify "$WAREHOUSE" --profile warehouse >/dev/null

echo "== verify warehouse profile json =="
run_spanda verify "$WAREHOUSE" --profile warehouse --json >/dev/null

echo "== readiness warehouse profile =="
run_spanda readiness "$WAREHOUSE" --profile warehouse >/dev/null

echo "== defense showcase profile passes =="
run_spanda verify "$DEFENSE" --profile defense >/dev/null

echo "== medical showcase profile passes =="
MEDICAL_FILE="${ROOT}/examples/showcase/compliance/medical_rover.sd"
run_spanda verify "$MEDICAL_FILE" --profile medical >/dev/null

echo "== deploy gate secure_boot on defense showcase =="
GATE="$(run_spanda deploy gate "$DEFENSE" 2>&1 || true)"
echo "$GATE" | grep -q "secure_boot"

echo "== secure boot import typechecks =="
run_spanda check "$SECURE_BOOT" >/dev/null

echo "== medical profile flags missing secure boot on warehouse =="
MEDICAL="$(run_spanda verify "$WAREHOUSE" --profile medical 2>&1 || true)"
echo "$MEDICAL" | grep -q "requires_secure_boot"

cargo test -p spanda-compliance defense_showcase_passes_profile -q
cargo test -p spanda-compliance medical_showcase_passes_profile -q
cargo test -p spanda-compliance defense_profile_requires_secure_boot_contract -q
cargo test -p spanda-compliance secure_boot_showcase_satisfies_secure_boot_requirement -q

echo "Compliance smoke OK"
