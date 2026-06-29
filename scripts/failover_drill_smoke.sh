#!/usr/bin/env bash
# Automated failover drill — redundant chain walk-through and recovery action smoke.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

cargo test -p spanda-config --test failover_drill --quiet
echo "Failover drill smoke OK"
