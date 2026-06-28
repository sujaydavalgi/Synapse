#!/usr/bin/env bash
# Prepare ADAS security audit artifacts (ISO 26262 readiness + vehicle ECU paths).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${ROOT}/.spanda/adas-security-audit-prep.json"
ADAS="$ROOT/examples/solutions/adas"
mkdir -p "$(dirname "$OUT")"

echo "== ADAS security audit prep =="

SMOKE_OK=false
if ./scripts/adas_smoke.sh >/tmp/adas-smoke-audit.log 2>&1; then
  SMOKE_OK=true
fi

GATE_OK=false
if SPANDA_ADAS_SKIP_SOAK=1 SPANDA_ADAS_SKIP_AUDIT=1 ./scripts/adas_stable_promotion_gate.sh >/tmp/adas-gate-audit.log 2>&1; then
  GATE_OK=true
fi

export ROOT SMOKE_OK GATE_OK
python3 - <<'PY' > "$OUT"
import json, os, time
report = {
    "generated_at_ms": int(time.time() * 1000),
    "scope": [
        "iso26262_readiness_gates",
        "secure_comm_vehicle_ecu",
        "canbus_provider_tamper_policy",
        "ota_certify_before_rollout",
        "mission_trace_assurance_export",
    ],
    "checks": {
        "adas_smoke": os.environ.get("SMOKE_OK") == "true",
        "promotion_gate_smoke": os.environ.get("GATE_OK") == "true",
    },
    "reviewer_packet": [
        "docs/stable-hardening-adas.md",
        "docs/adas-security.md",
        "docs/adas-readiness.md",
        "examples/solutions/adas/spanda.security.toml",
        "spanda verify src/highway_drive.sd --profile iso26262",
        "spanda readiness src/highway_drive.sd --profile iso26262",
        "GET /v1/trust/package?name=spanda-gps",
    ],
}
print(json.dumps(report, indent=2))
PY

echo "Wrote $OUT"
cat "$OUT"
echo "adas-security-audit-prep ok"
