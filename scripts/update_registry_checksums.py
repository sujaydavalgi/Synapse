#!/usr/bin/env python3
"""Refresh SHA-256 checksums and Ed25519 signatures in registry/index.json.

Delegates to `cargo run -p spanda-package --bin registry-index-maintain`.
Pass `--verify` to check the committed index without writing.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


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
    cmd = [
        "cargo",
        "run",
        "-q",
        "-p",
        "spanda-package",
        "--bin",
        "registry-index-maintain",
        "--",
    ]
    if "--verify" in sys.argv[1:]:
        cmd.append("--verify")
    subprocess.run(cmd, cwd=ROOT, check=True)


if __name__ == "__main__":
    main()
