#!/usr/bin/env bash
# Golden path for spanda-ledger community package scaffold.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "== ledger provider dispatch =="
cargo test -p spanda-providers ledger_package_append_dispatches -- --exact --nocapture

echo "== community scaffold files =="
test -f "${ROOT}/packages/community/README.md"
test -f "${ROOT}/packages/registry/spanda-ledger/examples/anchor_audit/spanda.toml"
test -f "${ROOT}/packages/registry/spanda-ledger/examples/anchor_audit/src/main.sd"

echo "Ledger golden path complete."
