#!/usr/bin/env bash
# Copy offline registry slice into the spanda CLI crate for cargo install demos.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="${ROOT}/crates/spanda-cli/bundled-registry"
mkdir -p "${DEST}/packages"

python3 - "${ROOT}" "${DEST}" <<'PY'
import json
import shutil
import sys
from pathlib import Path

root = Path(sys.argv[1])
dest = Path(sys.argv[2])
src_index = root / "registry" / "index.json"
names = {
    "spanda-trust-jetson",
    "spanda-trust-pi",
    "spanda-gps",
    "spanda-fusion",
    "spanda-matter",
    "spanda-thread",
    "spanda-zwave",
    "spanda-bacnet",
    "spanda-knx",
    "spanda-home-assistant",
    "spanda-energy",
    "spanda-building",
    "spanda-smart-locks",
    "spanda-environment",
}

entries = json.loads(src_index.read_text())
subset = [entry for entry in entries if entry.get("name") in names]
found = {entry["name"] for entry in subset}
if found != names:
    missing = names - found
    raise SystemExit(f"missing bundled packages in index.json: {missing}")

(dest / "index.json").write_text(json.dumps(subset, indent=2) + "\n")

for name in sorted(names):
    src_pkg = root / "registry" / "packages" / name
    dst_pkg = dest / "packages" / name
    if dst_pkg.exists():
        shutil.rmtree(dst_pkg)
    shutil.copytree(src_pkg, dst_pkg)

print(f"✓ Bundled {len(subset)} registry packages to {dest}")
PY
