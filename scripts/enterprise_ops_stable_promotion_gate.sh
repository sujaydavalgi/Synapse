#!/usr/bin/env bash
# Enterprise operations Stable tier promotion gate (implementation + operational checks).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SOAK_FILE="${SPANDA_FIELD_SOAK_START_FILE:-$ROOT/.spanda/field-soak-start.txt}"
MIN_DAYS="${SPANDA_FIELD_SOAK_MIN_DAYS:-30}"
AUDIT_FILE="${SPANDA_SECURITY_AUDIT_PREP_FILE:-$ROOT/.spanda/security-audit-prep.json}"

echo "== Enterprise operations stable promotion gate =="

if [[ "${SPANDA_ENTERPRISE_OPS_SKIP_SOAK:-0}" != "1" ]]; then
  echo "--- Field soak (min ${MIN_DAYS} days) ---"
  if [[ ! -f "$SOAK_FILE" ]]; then
    echo "missing soak start file: $SOAK_FILE" >&2
    echo "Create with: ./scripts/enterprise_ops_field_soak_init.sh" >&2
    exit 1
  fi
  START_DATE="$(tr -d '[:space:]' < "$SOAK_FILE")"
  if date -u -j -f "%Y-%m-%d" "$START_DATE" "+%s" >/dev/null 2>&1; then
    START_EPOCH="$(date -u -j -f "%Y-%m-%d" "$START_DATE" "+%s")"
  else
    START_EPOCH="$(date -u -d "$START_DATE" "+%s")"
  fi
  NOW_EPOCH="$(date -u "+%s")"
  ELAPSED_DAYS=$(( (NOW_EPOCH - START_EPOCH) / 86400 ))
  echo "Soak started: $START_DATE (${ELAPSED_DAYS} days elapsed)"
  if (( ELAPSED_DAYS < MIN_DAYS )); then
    echo "Enterprise ops field soak incomplete: need $(( MIN_DAYS - ELAPSED_DAYS )) more day(s)" >&2
    exit 1
  fi
else
  echo "Skipping field soak (SPANDA_ENTERPRISE_OPS_SKIP_SOAK=1)"
fi

if [[ "${SPANDA_ENTERPRISE_OPS_SKIP_AUDIT:-0}" != "1" ]]; then
  echo "--- Security audit prep artifact ---"
  if [[ ! -f "$AUDIT_FILE" ]]; then
    echo "missing audit prep file: $AUDIT_FILE" >&2
    echo "Run: ./scripts/security_audit_prep.sh" >&2
    exit 1
  fi
  python3 -c 'import json,sys; json.load(open(sys.argv[1]))' "$AUDIT_FILE"
  echo "Security audit prep artifact present (external reviewer sign-off still required)"
else
  echo "Skipping audit prep check (SPANDA_ENTERPRISE_OPS_SKIP_AUDIT=1)"
fi

echo "--- Enterprise ops smoke ---"
if [[ "${SPANDA_ENTERPRISE_OPS_SKIP_SMOKE:-0}" != "1" ]]; then
  "$ROOT/scripts/enterprise_ops_smoke.sh"
else
  echo "Skipping enterprise_ops_smoke (SPANDA_ENTERPRISE_OPS_SKIP_SMOKE=1)"
fi

echo "--- Failover drill smoke ---"
if [[ -x "$ROOT/scripts/failover_drill_smoke.sh" ]]; then
  "$ROOT/scripts/failover_drill_smoke.sh"
fi

echo "--- OTA fleet soak (quick) ---"
if [[ -x "$ROOT/scripts/ota_fleet_soak.sh" ]]; then
  SPANDA_OTA_FLEET_SOAK_QUICK=1 "$ROOT/scripts/ota_fleet_soak.sh"
fi

echo "--- Entity model smoke ---"
if [[ -x "$ROOT/scripts/entity_model_smoke.sh" ]]; then
  "$ROOT/scripts/entity_model_smoke.sh"
fi

echo ""
echo "Enterprise operations stable promotion gate passed."
echo "Remaining operational steps: third-party audit sign-off, production registry releases."
echo "See docs/enterprise-ops-stable-promotion.md"
