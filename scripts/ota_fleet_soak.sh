#!/usr/bin/env bash
# Production OTA fleet soak — multi-agent version bumps with readiness gates.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

cargo test -p spanda-ota --test fleet_soak --quiet
echo "OTA fleet soak OK"
