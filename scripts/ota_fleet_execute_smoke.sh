#!/usr/bin/env bash
# Live OTA fleet execute — Control Center POST /v1/ota/execute against a test deploy agent.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

cargo test -p spanda-api --test ota_execute_live ota_execute_live_rollout_updates_agent -q
echo "OTA fleet execute live smoke OK"
