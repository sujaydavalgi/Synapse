#!/usr/bin/env bash
# SDK smoke — program-level REST endpoints and official SDK clients.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PROGRAM="${ROOT}/examples/showcase/compliance/defense_rover.sd"
if [[ ! -f "$PROGRAM" ]]; then
  PROGRAM="${ROOT}/examples/robotics/rover.sd"
fi

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

PORT="${SPANDA_SDK_TEST_PORT:-}"
if [[ -z "$PORT" ]]; then
  PORT=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
fi
GRPC_PORT="${SPANDA_SDK_GRPC_PORT:-}"
if [[ -z "$GRPC_PORT" ]]; then
  GRPC_PORT=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
fi
BIND="127.0.0.1:${PORT}"
GRPC_BIND="127.0.0.1:${GRPC_PORT}"

echo "== start control-center on ${BIND} (program: ${PROGRAM}) =="
run_spanda control-center serve \
  --bind "$BIND" \
  --grpc-bind "$GRPC_BIND" \
  --program "$PROGRAM" &
SERVER_PID=$!
sleep 2

cleanup() {
  kill "$SERVER_PID" 2>/dev/null || true
}
trap cleanup EXIT

fetch() {
  curl -sf --max-time 15 "http://${BIND}${1}"
}

post_json() {
  local path="$1"
  local body="$2"
  curl -sf --max-time 30 -X POST \
    -H "Content-Type: application/json" \
    -d "$body" \
    "http://${BIND}${path}"
}

echo "== POST /v1/programs/readiness =="
post_json /v1/programs/readiness "{\"file\":\"${PROGRAM}\"}" | grep -q '"report"'

echo "== GET /v1/entities =="
fetch /v1/entities | grep -q '"entities"'

echo "== POST /v1/rpc EvaluateProgramReadiness =="
curl -sf -X POST \
  -H "Content-Type: application/json" \
  -d "{\"method\":\"spanda.v1.ControlCenter/EvaluateProgramReadiness\",\"params\":{\"body_json\":\"{\\\"file\\\":\\\"${PROGRAM}\\\"}\"}}" \
  "http://${BIND}/v1/rpc" | grep -q '"result"'

echo "== Python SpandaClient (sdk/python) =="
PYTHONPATH="${ROOT}/sdk/python:${PYTHONPATH:-}" \
  SPANDA_CONTROL_CENTER_URL="http://${BIND}" \
  python3 -c "
from spanda import SpandaClient
c = SpandaClient()
assert c.health_check()['service'] == 'spanda-control-center'
entities = c.list_entities()
assert 'entities' in entities
"

echo "== TypeScript @spanda/sdk (compiled client) =="
if [[ -d "${ROOT}/sdk/typescript/node_modules" ]]; then
  npm run build --prefix "${ROOT}/sdk/typescript" --silent
  node --input-type=module -e "
import { SpandaClient } from './sdk/typescript/dist/index.js';
const c = SpandaClient.local();
c.baseUrl = 'http://${BIND}';
const health = await c.healthCheck();
if (!health.service) throw new Error('missing service');
console.log('ts sdk ok');
"
else
  echo "skip TypeScript (run npm ci --prefix sdk/typescript first)"
fi

echo "SDK smoke OK"
