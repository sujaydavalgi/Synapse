#!/usr/bin/env bash
# Smart Spaces & Ambient Intelligence promotion gate (experimental scaffold tier).
# Runs blueprint smoke, API unit tests, OpenAPI parity, and Control Center live probe.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SS="$ROOT/examples/solutions/smart-spaces"
PROGRAM="$SS/smart-building/floor_readiness.sd"
CONFIG="$SS/spanda.toml"

echo "== Smart Spaces promotion gate =="

if [[ -z "${SPANDA_BIN:-}" ]]; then
  SPANDA_BIN="$ROOT/target/release/spanda"
  echo "--- Building control-center binary (release) ---"
  cargo build -q -p spanda --release
  export SPANDA_BIN
fi

SOAK_FILE="${SPANDA_SMART_SPACES_FIELD_SOAK_START_FILE:-$ROOT/.spanda/smart-spaces-field-soak-start.txt}"
MIN_DAYS="${SPANDA_SMART_SPACES_FIELD_SOAK_MIN_DAYS:-30}"

if [[ "${SPANDA_SMART_SPACES_SKIP_SOAK:-1}" != "1" ]]; then
  echo "--- Field soak (min ${MIN_DAYS} days) ---"
  if [[ ! -f "$SOAK_FILE" ]]; then
    echo "missing soak start file: $SOAK_FILE" >&2
    echo "Create with: ./scripts/smart_spaces_field_soak_init.sh" >&2
    exit 1
  fi
  START_DATE="$(tr -d '[:space:]' < "$SOAK_FILE")"
  if date -u -j -f "%Y-%m-%d" "$START_DATE" "+%s" >/dev/null 2>&1; then
    START_EPOCH="$(date -u -j -f "%Y-%m-%d" "$START_DATE" "+%s")"
  else
    START_EPOCH="$(date -u -d "$START_DATE" "+%s")"
  fi
  NOW_EPOCH="$(date -u "+%s")"
  ELAPSED_DAYS=$(( (NOW_EPOCH - START_EPOCH) / 86400 ))
  echo "Smart Spaces soak started: $START_DATE (${ELAPSED_DAYS} days elapsed)"
  if (( ELAPSED_DAYS < MIN_DAYS )); then
    echo "Smart Spaces field soak incomplete: need $(( MIN_DAYS - ELAPSED_DAYS )) more day(s)" >&2
    exit 1
  fi
else
  echo "Skipping field soak (SPANDA_SMART_SPACES_SKIP_SOAK=1)"
fi

AUDIT_FILE="${SPANDA_SMART_SPACES_SECURITY_AUDIT_PREP_FILE:-$ROOT/.spanda/smart-spaces-security-audit-prep.json}"
SELF_AUDIT_FILE="${SPANDA_SMART_SPACES_SECURITY_SELF_AUDIT_FILE:-$ROOT/.spanda/smart-spaces-security-self-audit.json}"
if [[ "${SPANDA_SMART_SPACES_SKIP_AUDIT:-1}" != "1" ]]; then
  echo "--- Smart Spaces security audit prep artifact ---"
  if [[ ! -f "$AUDIT_FILE" ]]; then
    echo "missing audit prep file: $AUDIT_FILE" >&2
    echo "Run: ./scripts/smart_spaces_security_audit_prep.sh" >&2
    exit 1
  fi
  python3 -c 'import json,sys; json.load(open(sys.argv[1]))' "$AUDIT_FILE"
  echo "Smart Spaces audit prep artifact present"
  if [[ ! -f "$SELF_AUDIT_FILE" ]]; then
    echo "missing self-audit file: $SELF_AUDIT_FILE" >&2
    echo "Run: ./scripts/smart_spaces_security_self_audit.sh" >&2
    exit 1
  fi
  python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); sys.exit(0 if all(c.get("passed") for c in d.get("checks",[])) else 1)' "$SELF_AUDIT_FILE"
  echo "Smart Spaces security self-audit checks passed"
  if [[ "${SPANDA_SMART_SPACES_AUDIT_SIGNED_OFF:-0}" != "1" ]]; then
    if ! python3 -c 'import json,sys; sys.exit(0 if json.load(open(sys.argv[1])).get("signed_off") else 1)' "$SELF_AUDIT_FILE" 2>/dev/null; then
      echo "Smart Spaces third-party security sign-off required (set SPANDA_SMART_SPACES_AUDIT_SIGNED_OFF=1 after human review)" >&2
      exit 1
    fi
  fi
else
  echo "Skipping audit prep check (SPANDA_SMART_SPACES_SKIP_AUDIT=1)"
fi

echo "--- Smart Spaces blueprint smoke ---"
if [[ "${SPANDA_SMART_SPACES_SKIP_SMOKE:-0}" != "1" ]]; then
  "$ROOT/scripts/smart_spaces_smoke.sh"
else
  echo "Skipping smoke (SPANDA_SMART_SPACES_SKIP_SMOKE=1)"
fi

echo "--- Smart Spaces live IoT smoke ---"
"$ROOT/scripts/smart_spaces_live_iot_smoke.sh"

echo "--- Smart Spaces BMS sidecar smoke ---"
"$ROOT/scripts/smart_spaces_bms_sidecar_smoke.sh"

echo "--- Smart Spaces live-building registry test ---"
cargo test -p spanda-providers --features live-building live_bacnet_registry_script_reads_mock_stdout -q

echo "--- Smart Spaces API unit tests ---"
cargo test -p spanda-api smart_spaces -q
cargo test -p spanda-api openapi_documents_all_rest -q

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

PORT="${SPANDA_SMART_SPACES_TEST_PORT:-}"
if [[ -z "$PORT" ]]; then
  PORT=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
fi
BIND="127.0.0.1:${PORT}"

echo "--- Control Center Smart Spaces API probe on ${BIND} ---"
run_spanda control-center serve --bind "$BIND" --config "$CONFIG" --program "$PROGRAM" &
SERVER_PID=$!
sleep 2

cleanup() {
  kill "$SERVER_PID" 2>/dev/null || true
}
trap cleanup EXIT

fetch() {
  local path="$1"
  local attempt=0
  while [[ $attempt -lt 30 ]]; do
    if curl -sf --max-time 15 "http://${BIND}${path}"; then
      return 0
    fi
    attempt=$((attempt + 1))
    sleep 0.2
  done
  echo "failed to fetch http://${BIND}${path}" >&2
  return 1
}

for path in \
  /v1/facilities \
  "/v1/facilities/tower-demo/readiness" \
  "/v1/facilities/tower-demo/health" \
  "/v1/facilities/tower-demo/security" \
  "/v1/facilities/tower-demo/floor-map" \
  "/v1/zones/floor-12/occupancy" \
  "/v1/zones/room-lobby/environment" \
  /v1/energy/systems \
  "/v1/energy/systems/battery-001" \
  /v1/emergency/status \
  /v1/smart-spaces/summary \
  "/v1/smart-spaces/devices?facility_id=tower-demo"
do
  echo "GET ${path}"
  body="$(fetch "$path")"
  echo "$body" | python3 -c 'import json,sys; json.load(sys.stdin)'
done

echo ""
echo "Smart Spaces promotion gate passed."
