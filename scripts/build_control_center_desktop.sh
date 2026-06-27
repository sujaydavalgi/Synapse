#!/usr/bin/env bash
# Build Control Center desktop installers (Tauri) when toolchain is available.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "[control-center-desktop] cargo check (src-tauri)"
if [[ "${SKIP_TAURI_LINUX_CARGO_CHECK:-}" == "1" ]]; then
  echo "[control-center-desktop] skip cargo check (SKIP_TAURI_LINUX_CARGO_CHECK=1; macOS bundle job validates desktop)"
else
  cargo check --manifest-path packages/control-center-desktop/src-tauri/Cargo.toml
fi

if command -v npm >/dev/null 2>&1; then
  echo "[control-center-desktop] npm install (workspace)"
  npm install --workspace=@spanda/control-center-desktop --ignore-scripts 2>/dev/null || npm install
  if [[ "${TAURI_BUILD:-0}" == "1" ]]; then
    echo "[control-center-desktop] tauri build (TAURI_BUILD=1)"
    if [[ -n "${TAURI_UPDATER_PUBKEY:-}" ]]; then
      echo "[control-center-desktop] updater signing pubkey provided (TAURI_UPDATER_ACTIVE=${TAURI_UPDATER_ACTIVE:-true})"
    fi
    npm run build --workspace=@spanda/control-center-desktop
    npm run tauri build --workspace=@spanda/control-center-desktop
    echo "[control-center-desktop] bundle artifacts under src-tauri/target/release/bundle/"
  else
    echo "[control-center-desktop] skip tauri build (set TAURI_BUILD=1 for full installer build)"
  fi
fi

echo "[control-center-desktop] OK"
