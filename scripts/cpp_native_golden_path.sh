#!/usr/bin/env bash
# Golden path for in-process C++ FFI (`cpp-native` feature).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE="${ROOT}/examples/ffi_cpp_extern.sd"

if ! command -v c++ >/dev/null 2>&1 && ! command -v g++ >/dev/null 2>&1; then
  echo "C++ compiler not found; skip cpp-native golden path" >&2
  exit 0
fi

echo "== build spanda-cli with cpp-native =="
cargo build -p spanda-cli --release --features cpp-native
SPANDA="${ROOT}/target/release/spanda"

echo "== check ffi_cpp_extern example =="
"${SPANDA}" check "${SOURCE}"

echo "== run with in-process C++ bridge =="
unset SPANDA_CPP_SUBPROCESS
"${SPANDA}" run "${SOURCE}"

echo "== native bridge unit test =="
cargo test -p spanda-bridge native_cpp_add_when_available -- --exact --nocapture

echo "cpp-native golden path complete."
