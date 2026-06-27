#!/usr/bin/env python3
"""Add missing package scaffolds to registry/index.json."""

from __future__ import annotations

import json
import sys
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib  # type: ignore


ROOT = Path(__file__).resolve().parents[1]
SCAFFOLDS = ROOT / "packages" / "registry"
INDEX_PATH = ROOT / "registry" / "index.json"


def category_for(name: str) -> str:
    if "discovery" in name:
        return "hardware"
    if name.startswith("spanda-alert-"):
        return "robotics"
    if any(token in name for token in ("otel", "grafana", "audit", "siem", "security")):
        return "observability"
    return "robotics"


def load_manifest(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def main() -> None:
    entries: list[dict] = json.loads(INDEX_PATH.read_text(encoding="utf-8"))
    by_name = {entry["name"]: entry for entry in entries}
    added = 0

    for scaffold in sorted(SCAFFOLDS.iterdir()):
        manifest_path = scaffold / "spanda.toml"
        if not manifest_path.is_file():
            continue
        manifest = load_manifest(manifest_path)
        package = manifest["package"]
        name = package["name"]
        version = package["version"]
        if name in by_name:
            continue
        adapter = manifest.get("adapter", {})
        provides = adapter.get("provides", [])
        if isinstance(provides, str):
            provides = [provides]
        by_name[name] = {
            "name": name,
            "description": package.get("description", name),
            "versions": [version],
            "category": category_for(name),
            "license": package.get("license", "Apache-2.0"),
            "import_paths": list(provides),
            "version_checksums": {},
            "version_signatures": {},
        }
        added += 1

    if added == 0:
        print("✓ registry index already lists all package scaffolds")
        return

    INDEX_PATH.write_text(
        json.dumps(sorted(by_name.values(), key=lambda entry: entry["name"]), indent=2)
        + "\n",
        encoding="utf-8",
    )
    print(f"✓ added {added} package(s) to {INDEX_PATH.relative_to(ROOT)}")


if __name__ == "__main__":
    try:
        main()
    except Exception as exc:  # noqa: BLE001
        print(f"sync_registry_index.py: {exc}", file=sys.stderr)
        raise SystemExit(1) from exc
