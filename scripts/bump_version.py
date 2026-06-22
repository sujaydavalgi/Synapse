#!/usr/bin/env python3
"""Bump the Spanda workspace semver (major, minor, or patch).

Updates Cargo.toml [workspace.package].version, npm package.json files,
and finalizes CHANGELOG.md [Unreleased] into a dated release section.

Usage:
  python3 scripts/bump_version.py patch
  python3 scripts/bump_version.py minor --dry-run
  python3 scripts/bump_version.py major --github-output "$GITHUB_OUTPUT"
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CARGO_TOML = ROOT / "Cargo.toml"
CHANGELOG = ROOT / "CHANGELOG.md"
NPM_ROOTS = [
    ROOT,
    ROOT / "editor" / "vscode",
]


def read_workspace_version() -> str:
    text = CARGO_TOML.read_text(encoding="utf-8")
    match = re.search(
        r'^\[workspace\.package\]\s*\n(?:[^\[]*\n)*?^version\s*=\s*"([^"]+)"',
        text,
        re.MULTILINE,
    )
    if not match:
        raise SystemExit(f"could not find [workspace.package].version in {CARGO_TOML}")
    return match.group(1)


def bump_semver(current: str, component: str) -> str:
    match = re.match(r"^(\d+)\.(\d+)\.(\d+)(.*)$", current)
    if not match:
        raise SystemExit(f"unsupported version format: {current!r}")
    major, minor, patch, suffix = match.groups()
    if suffix:
        raise SystemExit(
            f"refusing to bump prerelease version {current!r}; finalize prerelease manually"
        )
    major_i, minor_i, patch_i = int(major), int(minor), int(patch)
    if component == "major":
        return f"{major_i + 1}.0.0"
    if component == "minor":
        return f"{major_i}.{minor_i + 1}.0"
    if component == "patch":
        return f"{major_i}.{minor_i}.{patch_i + 1}"
    raise SystemExit(f"unknown bump component: {component!r}")


def write_workspace_version(new_version: str) -> None:
    text = CARGO_TOML.read_text(encoding="utf-8")
    pattern = re.compile(
        r'(\[workspace\.package\]\s*\n(?:[^\[]*\n)*?^version\s*=\s*")([^"]+)(")',
        re.MULTILINE,
    )
    updated, count = pattern.subn(rf"\g<1>{new_version}\g<3>", text, count=1)
    if count != 1:
        raise SystemExit("failed to update [workspace.package].version in Cargo.toml")
    CARGO_TOML.write_text(updated, encoding="utf-8")


def bump_changelog(new_version: str, release_date: str) -> None:
    text = CHANGELOG.read_text(encoding="utf-8")
    pattern = re.compile(r"(## \[Unreleased\]\s*\n)(.*?)(?=\n## \[|\Z)", re.DOTALL)
    match = pattern.search(text)
    if not match:
        raise SystemExit("CHANGELOG.md: missing ## [Unreleased] section")
    unreleased = match.group(2).rstrip()
    if not unreleased:
        unreleased = "\n"
    replacement = f"## [Unreleased]\n\n## [{new_version}] - {release_date}\n{unreleased}\n"
    CHANGELOG.write_text(text[: match.start()] + replacement + text[match.end() :], encoding="utf-8")


def npm_package_json_paths() -> list[Path]:
    paths = [root / "package.json" for root in NPM_ROOTS]
    paths.extend(sorted((ROOT / "packages").glob("*/package.json")))
    return paths


def refresh_npm_versions(new_version: str, dry_run: bool) -> None:
    if dry_run:
        print("would refresh npm lockfiles with npm version")
        return
    for root in NPM_ROOTS:
        cmd = [
            "npm",
            "version",
            new_version,
            "--no-git-tag-version",
            "--allow-same-version",
        ]
        if root == ROOT:
            cmd.append("-ws")
        subprocess.run(cmd, cwd=root, check=True)


def write_github_output(path: str | None, key: str, value: str) -> None:
    if not path:
        return
    with open(path, "a", encoding="utf-8") as handle:
        handle.write(f"{key}={value}\n")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "component",
        choices=("major", "minor", "patch"),
        help="semver component to increment",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="print planned changes without writing files",
    )
    parser.add_argument(
        "--github-output",
        metavar="FILE",
        help="append version=… to a GitHub Actions output file",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    current = read_workspace_version()
    new_version = bump_semver(current, args.component)
    release_date = date.today().isoformat()

    if args.dry_run:
        print(f"{current} -> {new_version} ({args.component})")
        for path in npm_package_json_paths():
            print(f"would set {path.relative_to(ROOT)} version -> {new_version}")
        refresh_npm_versions(new_version, dry_run=True)
        print(f"would update {CARGO_TOML.relative_to(ROOT)}")
        print(f"would update {CHANGELOG.relative_to(ROOT)}")
        return

    write_workspace_version(new_version)
    refresh_npm_versions(new_version, dry_run=False)
    bump_changelog(new_version, release_date)
    write_github_output(args.github_output, "version", new_version)
    print(f"✓ bumped {current} -> {new_version}")
    print(f"  tag: v{new_version}")


if __name__ == "__main__":
    main()
