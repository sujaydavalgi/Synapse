#!/usr/bin/env bash
# ADAS Solution Blueprint Stable tier promotion gate.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SOAK_FILE="${SPANDA_ADAS_FIELD_SOAK_START_FILE:-$ROOT/.spanda/adas-field-soak-start.txt}"
MIN_DAYS="${SPANDA_ADAS_FIELD_SOAK_MIN_DAYS:-30}"
ADAS="$ROOT/examples/solutions/adas"
PROGRAM="$ADAS/src/highway_drive.sd"
CONFIG="$ADAS/spanda.toml"

echo "== ADAS stable promotion gate =="

if [[ "${SPANDA_ADAS_SKIP_SOAK:-0}" != "1" ]]; then
  echo "--- Field soak (min ${MIN_DAYS} days) ---"
  if [[ ! -f "$SOAK_FILE" ]]; then
    echo "missing soak start file: $SOAK_FILE" >&2
    echo "Create with: ./scripts/adas_field_soak_init.sh" >&2
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
  echo "ADAS soak started: $START_DATE (${ELAPSED_DAYS} days elapsed)"
  if (( ELAPSED_DAYS < MIN_DAYS )); then
    echo "ADAS field soak incomplete: need $(( MIN_DAYS - ELAPSED_DAYS )) more day(s)" >&2
    exit 1
  fi
else
  echo "Skipping field soak (SPANDA_ADAS_SKIP_SOAK=1)"
fi

AUDIT_FILE="${SPANDA_ADAS_SECURITY_AUDIT_PREP_FILE:-$ROOT/.spanda/adas-security-audit-prep.json}"
if [[ "${SPANDA_ADAS_SKIP_AUDIT:-0}" != "1" ]]; then
  echo "--- ADAS security audit prep artifact ---"
  if [[ ! -f "$AUDIT_FILE" ]]; then
    echo "missing ADAS audit prep file: $AUDIT_FILE" >&2
    echo "Run: ./scripts/adas_security_audit_prep.sh" >&2
    exit 1
  fi
  python3 -c 'import json,sys; json.load(open(sys.argv[1]))' "$AUDIT_FILE"
  echo "ADAS audit prep artifact present"
else
  echo "Skipping audit prep check (SPANDA_ADAS_SKIP_AUDIT=1)"
fi

echo "--- ADAS blueprint smoke ---"
if [[ "${SPANDA_ADAS_SKIP_SMOKE:-0}" != "1" ]]; then
  "$ROOT/scripts/adas_smoke.sh"
else
  echo "Skipping smoke (SPANDA_ADAS_SKIP_SMOKE=1)"
fi

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

PORT="${SPANDA_ADAS_TEST_PORT:-}"
if [[ -z "$PORT" ]]; then
  PORT=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
fi
BIND="127.0.0.1:${PORT}"

echo "--- Control Center ADAS API probe on ${BIND} ---"
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
  /v1/dashboard \
  /v1/health/summary \
  /v1/assurance/summary \
  /v1/diagnosis/summary \
  /v1/ota/status \
  "/v1/trust/package?name=spanda-gps"
do
  echo "GET ${path}"
  body="$(fetch "$path")"
  echo "$body" | python3 -c 'import json,sys; json.load(sys.stdin)'
done

echo ""
echo "ADAS stable promotion gate passed."
