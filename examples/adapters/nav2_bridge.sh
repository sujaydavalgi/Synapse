#!/usr/bin/env bash
# Reference Nav2 bridge for SPANDA_NAV2_CMD — replace with your Nav2 action client.
set -euo pipefail
goal="${1:-unknown}"
echo "nav2-bridge: accepted goal=${goal}"
