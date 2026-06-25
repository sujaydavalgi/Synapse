#!/usr/bin/env bash
# Smoke tests for showcase demos (CI + local).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SPANDA="${SPANDA_BIN:-$ROOT/target/release/spanda}"

if [[ ! -x "${SPANDA}" ]]; then
  cargo build -p spanda --release
  SPANDA="${ROOT}/target/release/spanda"
fi

export SPANDA_BIN="${SPANDA}"
export SPANDA_ROOT="${ROOT}"

echo "== showcase smoke: safety =="
"${SPANDA}" demo safety

echo "== showcase smoke: verify =="
"${SPANDA}" demo verify

echo "== showcase smoke: health =="
"${SPANDA}" demo health

echo "== showcase smoke: fleet =="
"${SPANDA}" demo fleet

echo "== showcase smoke: self-healing =="
chmod +x scripts/self_healing_smoke.sh
./scripts/self_healing_smoke.sh

echo "== showcase smoke: continuity =="
chmod +x scripts/continuity_smoke.sh
./scripts/continuity_smoke.sh

echo "== showcase smoke: capability =="
"${SPANDA}" check examples/showcase/capability_verification/rover.sd
"${SPANDA}" verify examples/showcase/capability_verification/rover.sd --capabilities

echo "== showcase smoke: replay =="
"${SPANDA}" check examples/showcase/replay/mission.sd
"${SPANDA}" sim examples/showcase/replay/mission.sd

echo "== showcase smoke: killer path =="
chmod +x scripts/killer_demo_golden_path.sh
./scripts/killer_demo_golden_path.sh

echo "== showcase smoke: differentiation =="
chmod +x scripts/differentiation_smoke.sh
./scripts/differentiation_smoke.sh

echo "== showcase smoke: maturity =="
chmod +x scripts/maturity_smoke.sh
./scripts/maturity_smoke.sh

echo "== showcase smoke: mission diff =="
chmod +x scripts/diff_smoke.sh
./scripts/diff_smoke.sh

echo "== showcase smoke: scorecard =="
chmod +x scripts/scorecard_smoke.sh
./scripts/scorecard_smoke.sh

echo "== showcase smoke: policy =="
chmod +x scripts/policy_smoke.sh
./scripts/policy_smoke.sh

echo "== showcase smoke: chaos =="
chmod +x scripts/chaos_smoke.sh
./scripts/chaos_smoke.sh

echo "== showcase smoke: readiness trends =="
chmod +x scripts/readiness_trends_smoke.sh
./scripts/readiness_trends_smoke.sh

echo "== showcase smoke: estimate =="
chmod +x scripts/estimate_smoke.sh
./scripts/estimate_smoke.sh

echo "== showcase smoke: compliance =="
chmod +x scripts/compliance_smoke.sh
./scripts/compliance_smoke.sh

echo "== showcase smoke: adr =="
chmod +x scripts/adr_smoke.sh
./scripts/adr_smoke.sh

echo "== showcase smoke: tamper =="
chmod +x scripts/tamper_smoke.sh
./scripts/tamper_smoke.sh

echo "== showcase smoke: integrity =="
chmod +x scripts/integrity_smoke.sh
./scripts/integrity_smoke.sh

echo "== showcase smoke: decision explain =="
chmod +x scripts/decision_explain_smoke.sh
./scripts/decision_explain_smoke.sh

echo "== showcase smoke: policy runtime =="
chmod +x scripts/policy_runtime_smoke.sh
./scripts/policy_runtime_smoke.sh

echo "== showcase smoke: generate =="
chmod +x scripts/generate_smoke.sh
./scripts/generate_smoke.sh

echo "== showcase smoke: spoof =="
chmod +x scripts/spoof_smoke.sh
./scripts/spoof_smoke.sh

echo "== showcase smoke: package spoofing backends =="
chmod +x scripts/package_spoofing_smoke.sh
./scripts/package_spoofing_smoke.sh

echo "== showcase smoke: trust demos =="
chmod +x scripts/trust_showcase_smoke.sh
./scripts/trust_showcase_smoke.sh

echo "== showcase smoke: tamper diagnose =="
chmod +x scripts/tamper_diagnose_smoke.sh
./scripts/tamper_diagnose_smoke.sh

echo "== showcase smoke: fleet tamper =="
chmod +x scripts/fleet_tamper_smoke.sh
./scripts/fleet_tamper_smoke.sh

echo "== showcase smoke: security assurance =="
chmod +x scripts/security_assurance_smoke.sh
./scripts/security_assurance_smoke.sh

echo "== showcase smoke: tamper policy =="
chmod +x scripts/tamper_policy_smoke.sh
./scripts/tamper_policy_smoke.sh

echo "== showcase smoke: fleet mesh tamper =="
chmod +x scripts/fleet_mesh_tamper_smoke.sh
./scripts/fleet_mesh_tamper_smoke.sh

echo "Showcase smoke tests passed."
