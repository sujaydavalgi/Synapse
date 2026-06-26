#!/usr/bin/env bash
# Phase E1–E4 smoke — Control Center API through govern-and-trace endpoints.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
WAREHOUSE_FIXTURE="${ROOT}/crates/spanda-config/tests/fixtures/warehouse"
SMOKE_CONFIG_DIR="$(mktemp -d "${TMPDIR:-/tmp}/spanda-ops-smoke.XXXXXX")"
cp -R "${WAREHOUSE_FIXTURE}/." "${SMOKE_CONFIG_DIR}/"
CONFIG="${SMOKE_CONFIG_DIR}/spanda.toml"
PROGRAM="${ROOT}/examples/showcase/compliance/defense_rover.sd"

if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  run_spanda() { "$SPANDA_BIN" "$@"; }
else
  run_spanda() { cargo run -q -p spanda -- "$@"; }
fi

PORT="${SPANDA_CONTROL_CENTER_TEST_PORT:-}"
if [[ -z "$PORT" ]]; then
  PORT=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
fi
GRPC_PORT="${SPANDA_GRPC_TEST_PORT:-}"
if [[ -z "$GRPC_PORT" ]]; then
  GRPC_PORT=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
fi
BIND="127.0.0.1:${PORT}"
GRPC_BIND="127.0.0.1:${GRPC_PORT}"
export SPANDA_API_KEY="enterprise-ops-smoke-key"

export SPANDA_WS_STREAM_SECONDS=3
echo "== start control-center on ${BIND} + gRPC ${GRPC_BIND} (warehouse config + program) =="
run_spanda control-center serve --bind "$BIND" --grpc-bind "$GRPC_BIND" --config "$CONFIG" --program "$PROGRAM" &
SERVER_PID=$!
sleep 2

cleanup() {
  kill "$SERVER_PID" 2>/dev/null || true
  rm -rf "$SMOKE_CONFIG_DIR"
}
trap cleanup EXIT

chmod +x "${ROOT}/scripts/mock_otlp_traces_collector.py" "${ROOT}/scripts/ws_telemetry_probe.py" 2>/dev/null || true

fetch() {
  local path="$1"
  local attempt=0
  while [[ $attempt -lt 30 ]]; do
    if curl -sf "http://${BIND}${path}"; then
      return 0
    fi
    attempt=$((attempt + 1))
    sleep 0.2
  done
  echo "failed to fetch ${path}" >&2
  return 1
}

echo "== GET /v1/health =="
fetch /v1/health | grep -q spanda-control-center

echo "== GET /v1/dashboard =="
fetch /v1/dashboard | grep -q device_pool

echo "== GET /v1/devices =="
fetch /v1/devices | grep -q '"devices"'

echo "== GET /v1/fleet/agents =="
fetch /v1/fleet/agents | grep -q '"agents"'

echo "== GET /v1/rbac/matrix =="
fetch /v1/rbac/matrix | grep -q Administrator

echo "== POST /v1/alerts/test (authenticated) =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/alerts/test" | grep -q '"ok":true'

echo "== GET /v1/alerts =="
fetch /v1/alerts | grep -q Control

echo "== GET / (Control Center UI) =="
curl -sf "http://${BIND}/" | grep -q "Spanda Control Center"

echo "== GET /v1/devices/lidar-front =="
fetch /v1/devices/lidar-front | grep -q lidar-front

echo "== GET /v1/robots =="
fetch /v1/robots | grep -q rover-001

echo "== GET /v1/device-tree =="
fetch /v1/device-tree | grep -q mapping

echo "== POST /v1/readiness/run =="
curl -sf -X POST "http://${BIND}/v1/readiness/run" | grep -q mission_ready

echo "== GET /v1/failover/chains =="
fetch /v1/failover/chains | grep -q chains

echo "== GET /v1/device-reports =="
fetch /v1/device-reports | grep -q inventory

echo "== POST /v1/devices/discover (multi-transport) =="
SPANDA_DISCOVERY_MDNS_MATCHES="smoke-robot@127.0.0.1" \
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"transports":["mdns"],"timeout_ms":500}' \
  "http://${BIND}/v1/devices/discover" | grep -q '"registered"'

echo "== POST /v1/devices/gps-001/trust =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/devices/gps-001/trust" | grep -q 'device trusted by operator'

echo "== POST /v1/devices/gps-001/assign =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"robot_id":"rover-001"}' \
  "http://${BIND}/v1/devices/gps-001/assign" | grep -q '"ok":true'

echo "== POST /v1/devices/camera-front-001/trust (quarantine approval) =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/devices/camera-front-001/trust" | grep -q '"lifecycle_state":"verified"'

echo "== POST /v1/devices/drive-controller/quarantine =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/devices/drive-controller/quarantine" | grep -q quarantined

echo "== E2 GET /v1/discovery?transport=mdns =="
fetch "/v1/discovery?transport=mdns&timeout_ms=100" | grep -q '"transport":"mdns"'

echo "== E2 GET /v1/health/summary =="
fetch /v1/health/summary | grep -q overall_status

echo "== E2 GET /v1/assurance/summary =="
fetch /v1/assurance/summary | grep -q '"loaded":true'

echo "== E2 POST /v1/provision (expect readiness alert) =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"device_id":"lidar-front","robot_id":"rover-001"}' \
  "http://${BIND}/v1/provision" | grep -q '"ok":false'

echo "== E2 GET /v1/alerts (provisioning failure) =="
fetch /v1/alerts | grep -q readiness_failed

echo "== E2 POST /v1/config/snapshots =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"label":"smoke-baseline"}' \
  "http://${BIND}/v1/config/snapshots" | grep -q '"ok":true'

echo "== E2 GET /v1/config/snapshots =="
SNAPSHOT_JSON=$(fetch /v1/config/snapshots)
echo "$SNAPSHOT_JSON" | grep -q smoke-baseline
BASELINE_ID=$(echo "$SNAPSHOT_JSON" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d["snapshots"][0]["id"])')

echo "== E3 GET /v1/openapi.json =="
fetch /v1/openapi.json | grep -q Spanda

echo "== E3 GET /v1/drift?baseline_id =="
fetch "/v1/drift?baseline_id=${BASELINE_ID}" | grep -q dimensions_checked
export SPANDA_GRPC_BASELINE_ID="${BASELINE_ID}"

echo "== E3 native gRPC (tonic) probe =="
export SPANDA_GRPC_BIND="${GRPC_BIND}"
cargo test -p spanda-api --test grpc_live_probe grpc_live_control_center_endpoints --quiet

echo "== E3 POST /v1/ota/plan (canary dry-run) =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"strategy":"canary","version":"1.2.3","canary_percent":20,"dry_run":true,"assignments":[{"robot_name":"rover-001","hardware":"jetson"}]}' \
  "http://${BIND}/v1/ota/plan" | grep -q '"strategy":"canary"'

echo "== E3 POST /v1/ota/execute (dry-run fleet rollout) =="
curl -sf -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"strategy":"all","version":"1.2.3","dry_run":true,"assignments":[{"robot_name":"rover-001","hardware":"jetson"}]}' \
  "http://${BIND}/v1/ota/execute" | grep -q '"dry_run":true'

echo "== GET /v1/tenant (multi-tenant scope) =="
fetch /v1/tenant | grep -q tenant_id

echo "== GET /v1/observability/backend (OTEL collector config) =="
fetch /v1/observability/backend | grep -q spanda-otel-collector

echo "== E3 GET /v1/version (API policy) =="
fetch /v1/version | grep -q supported_versions

echo "== E3 GET /v1/audit/mutations =="
curl -sf -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/audit/mutations" | grep -q record_count

echo "== E3 live OTA execute (deploy agent) =="
chmod +x "${ROOT}/scripts/ota_fleet_execute_smoke.sh"
"${ROOT}/scripts/ota_fleet_execute_smoke.sh"

echo "== E2 GET /v1/discovery?transport=ble (registry package) =="
fetch "/v1/discovery?transport=ble" | grep -q spanda-discovery-ble

echo "== E2 GET /v1/discovery?transport=usb (registry package) =="
fetch "/v1/discovery?transport=usb" | grep -q spanda-discovery-usb

echo "== E3 GET /v1/trust/package?name=spanda-mqtt =="
fetch "/v1/trust/package?name=spanda-mqtt" | grep -q trust

echo "== E3 GET /v1/sre/summary =="
fetch /v1/sre/summary | grep -q availability_percent

echo "== E3 GET /v1/observability/traces (correlation IDs) =="
curl -sf -H "X-Correlation-ID: smoke-trace-1" "http://${BIND}/v1/health" >/dev/null
fetch /v1/observability/traces | grep -q smoke-trace-1

echo "== E3 POST /v1/rpc (gRPC gateway) =="
curl -sf -X POST \
  -H "Content-Type: application/json" \
  -d '{"method":"spanda.v1.SpandaService/GetHealth"}' \
  "http://${BIND}/v1/rpc" | grep -q spanda-control-center

echo "== E3 Python SDK health =="
PYTHONPATH="${ROOT}/packages/sdk-python/src:${PYTHONPATH:-}" \
  SPANDA_CONTROL_CENTER_URL="http://${BIND}" SPANDA_API_KEY="${SPANDA_API_KEY}" \
  python3 -c "from spanda_sdk import ControlCenterClient; c=ControlCenterClient(); assert c.health()['service']=='spanda-control-center'"

echo "== E4 GET /v1/compliance/export?profile=defense =="
curl -sf -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/compliance/export?profile=defense" | grep -q audit_export_id

echo "== E4 GET /v1/digital-thread/query =="
fetch "/v1/digital-thread/query" | grep -q matched_node_count

echo "== E4 GET /v1/executive/scorecard =="
fetch /v1/executive/scorecard | grep -q overall_score

echo "== E4 GET /v1/reports/export?format=markdown =="
curl -sf -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/reports/export?profile=defense&format=markdown" | grep -q executive

echo "== E4 GET /v1/reports/export?format=pdf =="
curl -sf -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/reports/export?profile=defense&format=pdf" | grep -q body_base64

echo "== OTLP GET /v1/observability/otlp/traces =="
fetch /v1/observability/otlp/traces | grep -q resourceSpans

echo "== OTLP GET /v1/observability/otlp/metrics =="
fetch /v1/observability/otlp/metrics | grep -q resourceMetrics

echo "== OTLP POST /v1/observability/otlp/export (mock Jaeger collector) =="
MOCK_OTLP_PORT=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()')
python3 "${ROOT}/scripts/mock_otlp_traces_collector.py" "${MOCK_OTLP_PORT}" &
MOCK_PID=$!
sleep 0.5
OTLP_RESPONSE=$(curl -s --max-time 10 -X POST \
  -H "Authorization: Bearer ${SPANDA_API_KEY}" \
  "http://${BIND}/v1/observability/otlp/export?endpoint=http://127.0.0.1:${MOCK_OTLP_PORT}/v1/traces")
echo "$OTLP_RESPONSE" | grep -q '"ok":true'
kill "$MOCK_PID" 2>/dev/null || true

echo "== WebSocket /v1/stream/telemetry =="
SPANDA_WS_STREAM_SECONDS=2 \
  python3 "${ROOT}/scripts/ws_telemetry_probe.py" "ws://${BIND}/v1/stream/telemetry" | grep -q '"type":"hello"'

echo "Enterprise operations smoke OK"
