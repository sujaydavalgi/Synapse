#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$ROOT"

echo "== offline fleet tamper manifest =="
./scripts/fleet_tamper_smoke.sh

echo "== fleet mesh tamper ingest + correlation =="
cargo test -p spanda-fleet mesh_coordinator_correlates_ingested_tamper_traces -q

echo "fleet mesh tamper smoke ok"
