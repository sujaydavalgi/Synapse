#!/usr/bin/env python3
"""Spanda Python FFI bridge (subprocess protocol v1).

Reads JSON from stdin:
  {"fn": "py_add", "args": [1, 2]}

Writes JSON to stdout:
  {"ok": true, "result": 3}
  {"ok": false, "error": "..."}

Register handlers below or extend this module for project-specific bindings.
"""

from __future__ import annotations

import json
import sys
from typing import Any, Callable

Handler = Callable[..., Any]

HANDLERS: dict[str, Handler] = {
    "py_add": lambda a, b: int(a) + int(b),
    "py_echo": lambda x: x,
    "py_version": lambda: 1,
}


def main() -> int:
    try:
        req = json.load(sys.stdin)
    except json.JSONDecodeError as exc:
        print(json.dumps({"ok": False, "error": f"Invalid JSON request: {exc}"}))
        return 0

    fn = req.get("fn")
    args = req.get("args", [])
    if not isinstance(fn, str):
        print(json.dumps({"ok": False, "error": "Missing fn string in request"}))
        return 0
    if not isinstance(args, list):
        print(json.dumps({"ok": False, "error": "args must be a JSON array"}))
        return 0

    handler = HANDLERS.get(fn)
    if handler is None:
        print(json.dumps({"ok": False, "error": f"Unknown python extern '{fn}'"}))
        return 0

    try:
        result = handler(*args)
    except Exception as exc:  # noqa: BLE001 — bridge boundary
        print(json.dumps({"ok": False, "error": str(exc)}))
        return 0

    print(json.dumps({"ok": True, "result": result}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
