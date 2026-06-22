#!/usr/bin/env bash
# Golden path for hosted registry install (file:// base + signature verify).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SPANDA="${SPANDA_BIN:-$ROOT/target/release/spanda}"
PROJECT="$(mktemp -d "${TMPDIR:-/tmp}/spanda-registry-golden.XXXXXX")"
TRUST_KEY="${ROOT}/registry/TRUST_KEY"

cleanup() {
  rm -rf "${PROJECT}"
}
trap cleanup EXIT

if [[ ! -x "${SPANDA}" ]]; then
  cargo build -p spanda-cli --release
  SPANDA="${ROOT}/target/release/spanda"
fi

export SPANDA_REGISTRY_URL="file://${ROOT}/registry"
export SPANDA_REGISTRY_TRUST_KEY="$(tr -d '\n' <"${TRUST_KEY}")"

mkdir -p "${PROJECT}/src"
cat >"${PROJECT}/spanda.toml" <<'EOF'
[package]
name = "registry_golden"
version = "0.1.0"
description = "Registry golden path smoke project"
license = "Apache-2.0"

[dependencies]
spanda-openai = "0.1.0"
spanda-ros2 = "0.1.0"
EOF
echo 'module registry_golden;' >"${PROJECT}/src/main.sd"

echo "== registry search =="
"${SPANDA}" registry search openai | grep -q spanda-openai
"${SPANDA}" registry info spanda-ros2 | grep -q spanda-ros2

echo "== install curated packages =="
"${SPANDA}" install --project "${PROJECT}"
test -d "${PROJECT}/.spanda/packages/spanda-openai"
test -d "${PROJECT}/.spanda/packages/spanda-ros2"
test -f "${PROJECT}/spanda.lock"

echo "== check project with installed imports =="
"${SPANDA}" check "${PROJECT}/src/main.sd"

echo "Registry golden path complete."
