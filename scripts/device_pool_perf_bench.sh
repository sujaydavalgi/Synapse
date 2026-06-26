#!/usr/bin/env bash
# Device pool list/summary performance gate for fleet-scale inventory.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo test -p spanda-config --test device_pool_scale device_pool_lists_1000_devices_under_budget -- --nocapture
echo "Device pool 1000-device perf gate passed."
