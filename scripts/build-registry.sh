#!/usr/bin/env bash
# Build curated registry tarballs into registry/packages/<name>/<version>.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

shopt -s nullglob
for src in "$ROOT/packages/registry"/*/; do
  name=$(basename "$src")
  if [[ ! -f "$src/spanda.toml" ]]; then
    echo "missing $src/spanda.toml" >&2
    exit 1
  fi
  version=$(grep '^version' "$src/spanda.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')
  dest_dir="$ROOT/registry/packages/$name"
  mkdir -p "$dest_dir"
  tmp=$(mktemp -d)
  tar -czf "$tmp/$name-$version.tar.gz" -C "$src" .
  cp "$tmp/$name-$version.tar.gz" "$dest_dir/$version"
  rm -rf "$tmp"
  echo "✓ registry/packages/$name/$version"
done

python3 scripts/sync_registry_index.py
echo "✓ Registry index: registry/index.json"
python3 scripts/update_registry_checksums.py
