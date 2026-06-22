#!/usr/bin/env bash
# Golden path for live AI provider (OpenAI via Python bridge; mock without API key).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SPANDA="${SPANDA_BIN:-$ROOT/target/release/spanda}"
SOURCE="${ROOT}/examples/ffi_openai_live.sd"
BRIDGE="${ROOT}/scripts/spanda_python_bridge.py"

if [[ ! -x "${SPANDA}" ]]; then
  cargo build -p spanda-cli --release
  SPANDA="${ROOT}/target/release/spanda"
fi

echo "== type-check live AI example =="
"${SPANDA}" check "${SOURCE}"

echo "== Python bridge mock path (no OPENAI_API_KEY) =="
unset OPENAI_API_KEY
RESULT="$(printf '%s\n' '{"fn":"openai_complete","args":["Reply with one word: safe"]}' | python3 "${BRIDGE}")"
echo "${RESULT}" | grep -q 'mock-completion'
echo "${RESULT}" | grep -q '"ok": true'

echo "== spanda run (mock completion) =="
"${SPANDA}" run "${SOURCE}"

echo "Live AI golden path complete."
