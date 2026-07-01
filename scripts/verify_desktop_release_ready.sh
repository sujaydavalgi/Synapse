#!/usr/bin/env bash
# Verify Control Center desktop version sync and compile before tagging desktop-v*.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PKG="${ROOT}/packages/control-center-desktop"

VER_PKG="$(node -p "require('${PKG}/package.json').version")"
VER_CARGO="$(grep '^version' "${PKG}/src-tauri/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')"
VER_TAURI="$(node -p "JSON.parse(require('fs').readFileSync('${PKG}/src-tauri/tauri.conf.json','utf8')).version")"

if [[ "${VER_PKG}" != "${VER_CARGO}" || "${VER_PKG}" != "${VER_TAURI}" ]]; then
  echo "Desktop version mismatch: package.json=${VER_PKG} Cargo.toml=${VER_CARGO} tauri.conf.json=${VER_TAURI}" >&2
  exit 1
fi

echo "== Control Center desktop ${VER_PKG} (manifests synced) =="
"${ROOT}/scripts/control_center_desktop_smoke.sh"
echo "Desktop release readiness verified. Tag and push desktop-v${VER_PKG} to trigger .github/workflows/desktop-release.yml"
