#!/usr/bin/env bash
# Build the VS Code extension VSIX without publishing (CI / maintainer smoke test).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
if [[ ! -d node_modules ]]; then
  npm ci
fi
if [[ ! -d editor/vscode/node_modules ]]; then
  npm install --prefix editor/vscode
fi
./scripts/bundle-vscode-extension.sh
cd editor/vscode
npm run package
test -f spanda-vscode-0.1.0.vsix
echo "✓ VS Code VSIX built: editor/vscode/spanda-vscode-0.1.0.vsix"
