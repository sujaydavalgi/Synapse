#!/usr/bin/env bash
# Golden-path deploy + certification workflow for robotics OTA examples.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CERTIFIED="${ROOT}/examples/robotics/certified_deployment.sd"
REMOTE="${ROOT}/examples/robotics/remote_ota_deployment.sd"

echo "== check certified deployment =="
spanda check "${CERTIFIED}"

echo "== verify with strict certify =="
spanda verify "${CERTIFIED}" --all-targets --strict-certify

echo "== certification proof artifact =="
spanda certify prove "${CERTIFIED}" --strict --out /tmp/spanda-certified-proof.json

echo "== deploy plan with certification summary =="
spanda deploy plan "${CERTIFIED}" --version 1.0.0

echo "== dry-run rollout with --require-certify =="
spanda deploy rollout "${CERTIFIED}" --require-certify --dry-run --version 1.0.0

echo "== remote OTA example (plan only) =="
spanda deploy plan "${REMOTE}" --version 1.3.0

echo "Golden path complete."
