#!/usr/bin/env python3
"""Generate docs/spanda-reference.md and docs/man/*.md from the compiler."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REFERENCE_OUT = ROOT / "docs" / "spanda-reference.md"
MAN_DIR = ROOT / "docs" / "man"


def run(cmd: list[str]) -> None:


    """








    Description:








    Run.

















    Inputs:








    cmd: list[str]








    Caller-supplied cmd.

















    Outputs:








    None.

















    Example:








    result = run(cmd)


    """
    print("+", " ".join(cmd))
    subprocess.run(cmd, cwd=ROOT, check=True)


def write_man_index() -> None:


    """








    Description:








    Write man index.

















    Inputs:








    None.

















    Outputs:








    None.

















    Example:








    result = write_man_index()


    """
    rows = []
    for path in sorted(MAN_DIR.glob("spanda*.md")):
        if path.name == "README.md":
            continue
        cmd = path.stem.replace("spanda-", "spanda ", 1)
        rows.append(f"| [{path.stem}](./{path.name}) | `{cmd}` |")
    readme = MAN_DIR / "README.md"
    readme.write_text(
        "# Spanda manual pages\n\n"
        "Man-page style CLI reference (markdown). Regenerate with "
        "`python3 scripts/generate_spanda_reference.py`.\n\n"
        "| Page | Command |\n|------|--------|\n"
        + "\n".join(rows)
        + "\n\nFull language reference: [spanda-reference.md](../spanda-reference.md)\n",
        encoding="utf-8",
    )


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
    run(
        [
            "cargo",
            "run",
            "-p",
            "spanda",
            "--release",
            "--",
            "reference",
            "--out",
            str(REFERENCE_OUT),
            "--man-dir",
            str(MAN_DIR),
        ]
    )
    write_man_index()
    man_count = len(list(MAN_DIR.glob("spanda*.md")))
    print(f"Wrote {REFERENCE_OUT} and {man_count} man pages under {MAN_DIR}")


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as exc:
        sys.exit(exc.returncode)
