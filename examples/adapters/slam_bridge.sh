#!/usr/bin/env bash
# Reference SLAM bridge for SPANDA_SLAM_CMD — replace with Cartographer/RTAB-Map hooks.
set -euo pipefail
op="${1:-localize}"
echo "slam-bridge: operation=${op} ok"
