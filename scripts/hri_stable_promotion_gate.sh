#!/usr/bin/env bash
# Human Interaction & Spatial Computing Stable tier promotion gate.
# Runs blueprint smoke, HRI API unit tests, and Control Center humans endpoints.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SOAK_FILE="${SPANDA_HRI_FIELD_SOAK_START_FILE:-$ROOT/.spanda/hri-field-soak-start.txt}"
MIN_DAYS="${SPANDA_HRI_FIELD_SOAK_MIN_DAYS:-30}"
SC="$ROOT/examples/solutions/spatial-computing"
PROGRAM="$SC/warehouse-ar/pick_mission.sd"
CONFIG="$SC/spanda.toml"

echo "== HRI stable promotion gate =="

if [[ "${SPANDA_HRI_SKIP_SOAK:-0}" != "1" ]]; then
  echo "--- Field soak (min ${MIN_DAYS} days) ---"
  if [[ ! -f "$SOAK_FILE" ]]; then
    echo "missing soak start file: $SOAK_FILE" >&2
    echo "Create with: date -u +%Y-%m-%d > $SOAK_FILE" >&2
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
  echo "HRI soak started: $START_DATE (${ELAPSED_DAYS} days elapsed)"
  if (( ELAPSED_DAYS < MIN_DAYS )); then
    echo "HRI field soak incomplete: need $(( MIN_DAYS - ELAPSED_DAYS )) more day(s)" >&2
    exit 1
  fi
else
  echo "Skipping field soak (SPANDA_HRI_SKIP_SOAK=1)"
fi

echo "--- Spatial computing blueprint smoke ---"
"$ROOT/scripts/spatial_computing_smoke.sh"

echo "--- HRI unit tests ---"
cargo test -p spanda-api humans -q
cargo test -p spanda-api hri_session -q
cargo test -p spanda-security --test human_health_tests -q
cargo test -p spanda-providers hri_ -q
cargo test -p spanda-api openapi_documents_all_rest -q

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

PORT="${SPANDA_HRI_TEST_PORT:-}"
if [[ -z "$PORT" ]]; then
  PORT=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
fi
BIND="127.0.0.1:${PORT}"

echo "--- Control Center HRI API probe on ${BIND} ---"
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
  /v1/humans \
  /v1/humans/readiness \
  /v1/wearables \
  /v1/human-health/policy \
  /v1/hri/sessions \
  /v1/hri/collaboration \
  /v1/hri/context \
  "/v1/humans/operator-001/readiness"
do
  echo "GET ${path}"
  body="$(fetch "$path")"
  echo "$body" | python3 -c 'import json,sys; json.load(sys.stdin)'
done

echo ""
echo "HRI stable promotion gate passed."
