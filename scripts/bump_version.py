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
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CARGO_TOML = ROOT / "Cargo.toml"
CHANGELOG = ROOT / "CHANGELOG.md"
NPM_ROOTS = [
    ROOT,
    ROOT / "editor" / "vscode",
]


def _workspace_package_body(text: str) -> str | None:


    """








    Description:








    Workspace package body.

















    Inputs:








    text: str








    Caller-supplied text.

















    Outputs:








    result: str | None








    Return value from `_workspace_package_body`.

















    Example:








    result = _workspace_package_body(text)


    """
    lines = text.splitlines()
    in_section = False
    body: list[str] = []
    for line in lines:
        stripped = line.strip()
        if stripped == "[workspace.package]":
            in_section = True
            body = []
            continue
        if in_section:
            if stripped.startswith("[") and stripped.endswith("]"):
                break
            body.append(line)
    if not in_section:
        return None
    return "\n".join(body)


def read_workspace_version() -> str:


    """








    Description:








    Read workspace version.

















    Inputs:








    None.

















    Outputs:








    result: str








    Return value from `read_workspace_version`.

















    Example:








    result = read_workspace_version()


    """
    text = CARGO_TOML.read_text(encoding="utf-8")
    body = _workspace_package_body(text)
    if body is None:
        raise SystemExit(f"could not find [workspace.package] in {CARGO_TOML}")
    for line in body.splitlines():
        match = re.match(r'^version\s*=\s*"([^"]+)"\s*$', line)
        if match:
            return match.group(1)
    raise SystemExit(f"could not find [workspace.package].version in {CARGO_TOML}")


def bump_semver(current: str, component: str) -> str:


    """








    Description:








    Bump semver.

















    Inputs:








    current: str








    Caller-supplied current.








    component: str








    Caller-supplied component.

















    Outputs:








    result: str








    Return value from `bump_semver`.

















    Example:








    result = bump_semver(current, component)


    """
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


    """








    Description:








    Write workspace version.

















    Inputs:








    new_version: str








    Caller-supplied new version.

















    Outputs:








    None.

















    Example:








    result = write_workspace_version(new_version)


    """
    text = CARGO_TOML.read_text(encoding="utf-8")
    lines = text.splitlines(keepends=True)
    in_section = False
    replaced = False
    for index, line in enumerate(lines):
        stripped = line.strip()
        if stripped == "[workspace.package]":
            in_section = True
            continue
        if in_section:
            if stripped.startswith("[") and stripped.endswith("]"):
                break
            match = re.match(r'^(\s*version\s*=\s*")([^"]+)("\s*)$', line)
            if match:
                lines[index] = f'{match.group(1)}{new_version}{match.group(3)}'
                if not lines[index].endswith("\n"):
                    lines[index] += "\n"
                replaced = True
                break
    if not replaced:
        raise SystemExit("failed to update [workspace.package].version in Cargo.toml")
    CARGO_TOML.write_text("".join(lines), encoding="utf-8")


def _unreleased_span(text: str) -> tuple[int, int]:


    """








    Description:








    Unreleased span.

















    Inputs:








    text: str








    Caller-supplied text.

















    Outputs:








    result: tuple[int, int]








    Return value from `_unreleased_span`.

















    Example:








    result = _unreleased_span(text)


    """
    header = "## [Unreleased]"
    start = text.find(header)
    if start == -1:
        raise SystemExit("CHANGELOG.md: missing ## [Unreleased] section")
    body_start = start + len(header)
    if body_start < len(text) and text[body_start] == "\n":
        body_start += 1
    next_section = text.find("\n## [", body_start)
    end = len(text) if next_section == -1 else next_section
    return start, end


def read_unreleased_section(text: str) -> str:


    """








    Description:








    Read unreleased section.

















    Inputs:








    text: str








    Caller-supplied text.

















    Outputs:








    result: str








    Return value from `read_unreleased_section`.

















    Example:








    result = read_unreleased_section(text)


    """
    _, end = _unreleased_span(text)
    header = "## [Unreleased]"
    start = text.find(header)
    body_start = start + len(header)
    if body_start < len(text) and text[body_start] == "\n":
        body_start += 1
    return text[body_start:end]


def unreleased_has_content(text: str) -> bool:


    """








    Description:








    Unreleased has content.

















    Inputs:








    text: str








    Caller-supplied text.

















    Outputs:








    result: bool








    Return value from `unreleased_has_content`.

















    Example:








    result = unreleased_has_content(text)


    """
    body = read_unreleased_section(text).strip()
    return bool(body)


def bump_changelog(new_version: str, release_date: str, *, allow_empty: bool) -> None:


    """








    Description:








    Bump changelog.

















    Inputs:








    new_version: str








    Caller-supplied new version.








    release_date: str








    Caller-supplied release date.








    *: input value








    Caller-supplied *.








    allow_empty: bool








    Caller-supplied allow empty.

















    Outputs:








    None.

















    Example:








    result = bump_changelog(new_version, release_date, *, allow_empty)


    """
    text = CHANGELOG.read_text(encoding="utf-8")
    unreleased = read_unreleased_section(text).rstrip()
    if not unreleased.strip() and not allow_empty:
        raise SystemExit(
            "CHANGELOG.md: ## [Unreleased] is empty; add release notes or pass --allow-empty-changelog"
        )
    if not unreleased:
        unreleased = "\n"
    span_start, span_end = _unreleased_span(text)
    replacement = f"## [Unreleased]\n\n## [{new_version}] - {release_date}\n{unreleased}\n"
    CHANGELOG.write_text(text[:span_start] + replacement + text[span_end:], encoding="utf-8")


def npm_package_json_paths() -> list[Path]:


    """








    Description:








    Npm package json paths.

















    Inputs:








    None.

















    Outputs:








    result: list[Path]








    Return value from `npm_package_json_paths`.

















    Example:








    result = npm_package_json_paths()


    """
    paths = [root / "package.json" for root in NPM_ROOTS]
    paths.extend(sorted((ROOT / "packages").glob("*/package.json")))
    return paths


def refresh_npm_versions(new_version: str, dry_run: bool) -> None:


    """








    Description:








    Refresh npm versions.

















    Inputs:








    new_version: str








    Caller-supplied new version.








    dry_run: bool








    Caller-supplied dry run.

















    Outputs:








    None.

















    Example:








    result = refresh_npm_versions(new_version, dry_run)


    """
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


    """








    Description:








    Write github output.

















    Inputs:








    path: str | None








    Caller-supplied path.








    key: str








    Caller-supplied key.








    value: str








    Caller-supplied value.

















    Outputs:








    None.

















    Example:








    result = write_github_output(path, key, value)


    """
    if not path:
        return
    with open(path, "a", encoding="utf-8") as handle:
        handle.write(f"{key}={value}\n")


def parse_args() -> argparse.Namespace:


    """








    Description:








    Parse args.

















    Inputs:








    None.

















    Outputs:








    result: argparse.Namespace








    Return value from `parse_args`.

















    Example:








    result = parse_args()


    """
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
    parser.add_argument(
        "--allow-empty-changelog",
        action="store_true",
        help="allow releasing when ## [Unreleased] has no entries",
    )
    return parser.parse_args()


def main() -> None:


    """








    Description:








    Main.

















    Inputs:








    None.

















    Outputs:








    None.

















    Example:








    result = main()


    """
    args = parse_args()
    current = read_workspace_version()
    new_version = bump_semver(current, args.component)
    release_date = date.today().isoformat()

    changelog_text = CHANGELOG.read_text(encoding="utf-8")
    if not unreleased_has_content(changelog_text) and not args.allow_empty_changelog:
        raise SystemExit(
            "CHANGELOG.md: ## [Unreleased] is empty; add release notes or pass --allow-empty-changelog"
        )

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
    bump_changelog(new_version, release_date, allow_empty=args.allow_empty_changelog)
    write_github_output(args.github_output, "version", new_version)
    print(f"✓ bumped {current} -> {new_version}")
    print(f"  tag: v{new_version}")


if __name__ == "__main__":
    main()
