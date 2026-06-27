#!/usr/bin/env bash
# Prepare Spanda Control Center for third-party security audit review.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${ROOT}/.spanda/security-audit-prep.json"
mkdir -p "$(dirname "$OUT")"

echo "== Security audit prep =="
RBAC_OK=false
if cargo test -p spanda-security --lib >/tmp/spanda-security-tests.log 2>&1; then
  grep -q "test result: ok" /tmp/spanda-security-tests.log && RBAC_OK=true
fi

API_POLICY_OK=false
if cargo test -p spanda-api api_policy >/tmp/spanda-api-policy-tests.log 2>&1; then
  grep -q "test result: ok" /tmp/spanda-api-policy-tests.log && API_POLICY_OK=true
fi

CARGO_AUDIT_STATUS="skipped"
if command -v cargo-audit >/dev/null 2>&1; then
  if cargo audit --quiet 2>/dev/null; then
    CARGO_AUDIT_STATUS="pass"
  else
    CARGO_AUDIT_STATUS="findings"
  fi
fi

export ROOT RBAC_OK API_POLICY_OK CARGO_AUDIT_STATUS
python3 - <<'PY' > "$OUT"
import json, os, time
root = os.environ.get("ROOT", ".")
report = {
    "generated_at_ms": int(time.time() * 1000),
    "scope": ["auth", "secrets", "rbac", "mutation_audit", "encrypted_snapshots"],
    "checks": {
        "rbac_tests": os.environ.get("RBAC_OK") == "true",
        "api_policy_tests": os.environ.get("API_POLICY_OK") == "true",
        "cargo_audit": os.environ.get("CARGO_AUDIT_STATUS"),
    },
    "reviewer_packet": [
        "docs/security-audit-third-party.md",
        "packages/registry/spanda-security-audit/README.md",
        "GET /v1/rbac/matrix",
        "GET /v1/audit/mutations/export?format=cef",
    ],
}
print(json.dumps(report, indent=2))
PY

echo "Wrote $OUT"
cat "$OUT"
echo "security-audit-prep ok"
