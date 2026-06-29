#!/usr/bin/env bash
# Unified Entity Model smoke — read + mutation + traceability endpoints.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
WAREHOUSE_FIXTURE="${ROOT}/crates/spanda-config/tests/fixtures/warehouse"
SMOKE_CONFIG_DIR="$(mktemp -d "${TMPDIR:-/tmp}/spanda-entity-smoke.XXXXXX")"
cp -R "${WAREHOUSE_FIXTURE}/." "${SMOKE_CONFIG_DIR}/"
CONFIG="${SMOKE_CONFIG_DIR}/spanda.toml"
PROGRAM="${ROOT}/examples/showcase/compliance/defense_rover.sd"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

PORT="${SPANDA_ENTITY_SMOKE_PORT:-}"
if [[ -z "$PORT" ]]; then
  PORT=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
fi
BIND="127.0.0.1:${PORT}"
export SPANDA_API_KEY="entity-model-smoke-key"

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
  echo "failed to fetch ${path}" >&2
  return 1
}

post_json() {
  local path="$1"
  local body="$2"
  local auth="${3:-0}"
  local args=(-sf --max-time 30 -X POST -H "Content-Type: application/json" -d "$body")
  if [[ "$auth" == "1" ]]; then
    args+=(-H "Authorization: Bearer ${SPANDA_API_KEY}")
  fi
  curl "${args[@]}" "http://${BIND}${path}"
}

cleanup() {
  kill "$SERVER_PID" 2>/dev/null || true
  rm -rf "$SMOKE_CONFIG_DIR"
}
trap cleanup EXIT

echo "== start control-center on ${BIND} =="
run_spanda control-center serve --bind "$BIND" --config "$CONFIG" --program "$PROGRAM" &
SERVER_PID=$!

echo "== wait for /v1/health =="
fetch /v1/health | grep -q spanda-control-center

echo "== GET /v1/entities =="
fetch /v1/entities | grep -q '"entities"'
fetch /v1/entities | grep -q 'rover-001'

echo "== GET /v1/entities/graph =="
fetch /v1/entities/graph | grep -q '"graph"'

echo "== GET /v1/entities/traceability =="
fetch '/v1/entities/traceability?entity_id=rover-001' | grep -q '"traceability"'

echo "== POST /v1/entities/query =="
post_json /v1/entities/query '{"kind":"robot"}' | grep -q '"result"'

echo "== POST /v1/entities/register (auth) =="
post_json /v1/entities/register '{
  "id": "smoke-bay",
  "entity_type": "calibration_station",
  "display_name": "Smoke Bay",
  "parent_id": "warehouse-a",
  "capabilities": ["calibrate"]
}' 1 | grep -q 'smoke-bay'

echo "== GET /v1/entities/smoke-bay =="
fetch /v1/entities/smoke-bay | grep -q 'smoke-bay'

echo "== POST /v1/entities/smoke-bay/tags =="
post_json /v1/entities/smoke-bay/tags '{"add":["smoke-test"]}' 1 | grep -q 'smoke-test'

echo "== POST /v1/entities/relationships =="
post_json /v1/entities/relationships '{
  "from_id": "rover-001",
  "to_id": "gps-001",
  "kind": "depends_on",
  "label": "entity_smoke"
}' 1 | grep -q 'depends_on'

echo "== POST /v1/entities/sync =="
post_json /v1/entities/sync '{}' 1 | grep -q '"sync"'

echo "== TypeScript SDK entity mutations =="
if command -v npm >/dev/null 2>&1 && [[ -f "${ROOT}/sdk/typescript/package.json" ]]; then
  (
    cd "${ROOT}/sdk/typescript"
    npm run build --silent 2>/dev/null || npm run build
    SPANDA_CONTROL_CENTER_URL="http://${BIND}" \
    SPANDA_API_KEY="${SPANDA_API_KEY}" \
    node --input-type=module -e "
import { SpandaClient } from './dist/index.js';
const c = new SpandaClient();
const entities = await c.listEntities();
if (!entities.some((e) => e.id === 'smoke-bay')) throw new Error('smoke-bay missing after register');
await c.tagEntity('smoke-bay', { add: ['sdk-smoke'] });
const graph = await c.entityGraph();
if (!graph.graph) throw new Error('entity graph missing');
console.log('ts-sdk entity smoke ok');
"
  )
fi

echo "Entity model smoke OK"
