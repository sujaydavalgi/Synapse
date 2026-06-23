#!/usr/bin/env bash
# Install the Spanda CLI from source (installs binary `spanda`).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "== Spanda install =="

if ! command -v cargo >/dev/null 2>&1; then
  echo "Rust (cargo) is required. Install from https://rustup.rs" >&2
  exit 1
fi

echo "→ Building and installing spanda CLI…"
cargo install --path crates/spanda-cli --locked --force

if command -v spanda >/dev/null 2>&1; then
  echo "✓ $(spanda --version 2>/dev/null || echo 'spanda installed')"
else
  echo "Add ~/.cargo/bin to your PATH if spanda is not found."
fi

echo ""
echo "Quick start (from repo root):"
echo "  spanda demo rover"
echo "  spanda check examples/showcase/killer_demo.sd"
echo "  spanda verify examples/showcase/hardware_compatibility.sd"
echo ""
echo "Prebuilt packages: docs/installation.md"
