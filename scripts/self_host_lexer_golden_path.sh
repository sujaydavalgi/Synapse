#!/usr/bin/env bash
# Golden path for self-host lexer milestone (Rust parity + Spanda bootstrap example).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ -n "${SPANDA:-}" ]]; then
  :
elif [[ -x "${ROOT}/target/release/spanda" ]]; then
  SPANDA="${ROOT}/target/release/spanda"
else
  SPANDA="spanda"
fi

echo "== Rust lexer parity test =="
cargo test -p spanda-lexer self_host_bootstrap_sample_tokenizes -- --exact --nocapture

echo "== Spanda bootstrap examples =="
"${SPANDA}" check "${ROOT}/examples/self_host/word_tokenizer.sd"
"${SPANDA}" check "${ROOT}/examples/self_host/lexer_keywords.sd"
"${SPANDA}" run "${ROOT}/examples/self_host/lexer_keywords.sd"

echo "Self-host lexer golden path complete."
