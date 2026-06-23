#!/usr/bin/env bash
# Local compile-stage benchmarks (parse/check/verify/sim).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SPANDA="${SPANDA_BIN:-$ROOT/target/release/spanda}"
FILE="${BENCH_FILE:-$ROOT/examples/showcase/killer_demo.sd}"
RUNS="${BENCH_RUNS:-5}"

if [[ ! -x "${SPANDA}" ]]; then
  echo "Building spanda CLI…"
  cargo build -p spanda-cli --release
  SPANDA="${ROOT}/target/release/spanda"
fi

median_time() {
  local label="$1"
  shift
  local -a samples=()
  local i
  for ((i = 0; i < RUNS; i++)); do
    local t
    t=$(/usr/bin/time -f '%e' "$@" 2>&1 >/dev/null | tail -1)
    samples+=("$t")
  done
  printf '%s\n' "${samples[@]}" | sort -n | awk -v n="$RUNS" -v label="$label" '
    { a[NR]=$1 }
    END {
      if (n % 2) printf "%s median: %.3fs (%d runs)\n", label, a[(n+1)/2], n;
      else printf "%s median: %.3fs (%d runs)\n", label, (a[n/2]+a[n/2+1])/2, n;
    }'
}

echo "== Spanda benchmarks =="
echo "File: ${FILE}"
echo "CLI:  ${SPANDA}"
echo ""

median_time "check" "${SPANDA}" check "${FILE}"
median_time "verify" "${SPANDA}" verify "${FILE}" --json
/usr/bin/time -f "sim wall: %e s (1 run)" "${SPANDA}" sim "${FILE}" 2>&1 >/dev/null

echo ""
echo "Full guide: docs/benchmarks.md"
