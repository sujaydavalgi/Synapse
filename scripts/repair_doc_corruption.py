#!/usr/bin/env python3
"""Repair syntax corruption introduced by malformed inline doc insertion."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

CORRUPT_CODE = re.compile(
    r"^(?P<head>.+?)(?P<suffix>\)|\w+|>|\])\`\.$"
)
CORRUPT_IMPL = re.compile(
    r"^(?P<head>(?:pub\s+)?(?:impl|struct|enum|trait|fn)\b.+?)\`\.$"
)
ORPHAN_DOC = re.compile(
    r"^\s*//\s*(?:Options:|Example:|Parameters:|Returns:)\s*$"
)


def repair_text(text: str) -> tuple[str, int]:


    """








    Description:








    Repair text.

















    Inputs:








    text: str








    Caller-supplied text.

















    Outputs:








    result: tuple[str, int]








    Return value from `repair_text`.

















    Example:








    result = repair_text(text)


    """
    fixes = 0
    lines = text.splitlines(keepends=True)
    out: list[str] = []
    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.rstrip("\n")

        m = CORRUPT_CODE.match(stripped) or CORRUPT_IMPL.match(stripped)
        if m and not stripped.lstrip().startswith("//"):
            indent = re.match(r"^(\s*)", line).group(1)
            head = m.group("head")
            out.append(f"{indent}{head} {{\n")
            fixes += 1
            i += 1
            # Drop orphaned trailing doc fragment until real body starts.
            while i < len(lines):
                nxt = lines[i]
                ns = nxt.strip()
                if ns.startswith("//"):
                    i += 1
                    continue
                if ns == "}":
                    i += 1
                    break
                break
            continue

        out.append(line)
        i += 1

    text = "".join(out)

    # Remove duplicate impl/trait headers created by bad insertion.
    dedup_pattern = re.compile(
        r"(?ms)^(?P<header>(?:impl|pub trait)\b[^\n]+\{\n)"
        r"(?:\s*//[^\n]*\n)*"
        r"\s*\}\n\n"
        r"(?P=header)"
    )
    while True:
        new_text, n = dedup_pattern.subn(r"\1", text)
        if n == 0:
            break
        fixes += n
        text = new_text

    # Collapse duplicated blank lines inside doc blocks from pre-existing work.
    text = re.sub(r"(// Example:\n)\s*// Example:\n", r"\1", text)

    return text, fixes


def main() -> int:


    """








    Description:








    Main.

















    Inputs:








    None.

















    Outputs:








    result: int








    Return value from `main`.

















    Example:








    result = main()


    """
    total_fixes = 0
    changed = 0
    for path in sorted((ROOT / "crates").rglob("*.rs")):
        if "target" in path.parts:
            continue
        original = path.read_text(encoding="utf-8")
        if "`." not in original and "// Example:\n        // Example:" not in original:
            continue
        repaired, fixes = repair_text(original)
        if fixes and repaired != original:
            path.write_text(repaired, encoding="utf-8")
            changed += 1
            total_fixes += fixes
            print(f"repaired {path.relative_to(ROOT)} ({fixes} fixes)")

    print(f"\nDone. Repaired {changed} files, {total_fixes} fixes.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
