#!/usr/bin/env bash
# Agriculture Official Solution Blueprint smoke — delegates to solution_blueprints_smoke.sh.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
exec "${ROOT}/scripts/solution_blueprints_smoke.sh"
