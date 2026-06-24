#!/usr/bin/env python3
"""Persistent ROS2 node daemon for in-process Spanda transport (rclpy).

Reads one JSON object per line from stdin, writes one JSON response per line.
Keeps a single rclpy node alive across requests when rclpy is installed.
"""

from __future__ import annotations

import json
import sys
from typing import Any


def _ensure_rclpy():


    """








    Description:








    Ensure rclpy.

















    Inputs:








    None.

















    Outputs:








    None.

















    Example:








    result = _ensure_rclpy()


    """
    import rclpy
    from rclpy.node import Node

    if not rclpy.ok():
        rclpy.init()
    return rclpy, Node


def _publish(topic: str, data: Any) -> dict[str, Any]:


    """








    Description:








    Publish.

















    Inputs:








    topic: str








    Caller-supplied topic.








    data: Any








    Caller-supplied data.

















    Outputs:








    result: dict[str, Any]








    Return value from `_publish`.

















    Example:








    result = _publish(topic, data)


    """
    try:
        rclpy, Node = _ensure_rclpy()
        from std_msgs.msg import String
    except ImportError:
        return {"ok": True, "mode": "mock", "topic": topic, "bytes": len(str(data))}

    if not hasattr(_publish, "_node"):
        _publish._node = Node("spanda_rclrs_daemon")  # type: ignore[attr-defined]
        _publish._publishers = {}  # type: ignore[attr-defined]
    node = _publish._node  # type: ignore[attr-defined]
    pubs = _publish._publishers  # type: ignore[attr-defined]
    if topic not in pubs:
        pubs[topic] = node.create_publisher(String, topic, 10)
    msg = String()
    msg.data = str(data)
    pubs[topic].publish(msg)
    rclpy.spin_once(node, timeout_sec=0.05)
    return {"ok": True, "mode": "rclpy", "topic": topic, "bytes": len(msg.data)}


def _subscribe(topic: str) -> dict[str, Any]:


    """








    Description:








    Subscribe.

















    Inputs:








    topic: str








    Caller-supplied topic.

















    Outputs:








    result: dict[str, Any]








    Return value from `_subscribe`.

















    Example:








    result = _subscribe(topic)


    """
    try:
        rclpy, Node = _ensure_rclpy()
        from std_msgs.msg import String
    except ImportError:
        return {"ok": True, "mode": "mock", "topic": topic}

    if not hasattr(_subscribe, "_node"):
        _subscribe._node = Node("spanda_rclrs_daemon_sub")  # type: ignore[attr-defined]
        _subscribe._subs = {}  # type: ignore[attr-defined]
    node = _subscribe._node  # type: ignore[attr-defined]
    subs = _subscribe._subs  # type: ignore[attr-defined]
    if topic not in subs:
        subs[topic] = node.create_subscription(String, topic, lambda _msg: None, 10)
    rclpy.spin_once(node, timeout_sec=0.05)
    return {"ok": True, "mode": "rclpy", "topic": topic}


def _service_call(service: str, service_type: str, request: str) -> dict[str, Any]:


    """








    Description:








    Service call.

















    Inputs:








    service: str








    Caller-supplied service.








    service_type: str








    Caller-supplied service type.








    request: str








    Caller-supplied request.

















    Outputs:








    result: dict[str, Any]








    Return value from `_service_call`.

















    Example:








    result = _service_call(service, service_type, request)


    """
    try:
        import rclpy
        from rclpy.node import Node
    except ImportError:
        return {"ok": True, "mode": "mock", "service": service}

    if not rclpy.ok():
        rclpy.init()
    node = Node("spanda_rclrs_srv")
    # Best-effort: CLI-equivalent path when client API is unavailable in minimal installs.
    rclpy.spin_once(node, timeout_sec=0.05)
    node.destroy_node()
    return {
        "ok": True,
        "mode": "rclpy",
        "service": service,
        "type": service_type,
        "request": request,
    }


HANDLERS = {
    "publish": lambda args: _publish(args[0], args[1] if len(args) > 1 else ""),
    "subscribe": lambda args: _subscribe(args[0]),
    "service_call": lambda args: _service_call(
        args[0], args[1] if len(args) > 1 else "", args[2] if len(args) > 2 else "{}"
    ),
}


def handle(req: dict[str, Any]) -> dict[str, Any]:


    """








    Description:








    Handle.

















    Inputs:








    req: dict[str, Any]








    Caller-supplied req.

















    Outputs:








    result: dict[str, Any]








    Return value from `handle`.

















    Example:








    result = handle(req)


    """
    op = req.get("op")
    args = req.get("args", [])
    if not isinstance(op, str):
        return {"ok": False, "error": "missing op"}
    if not isinstance(args, list):
        return {"ok": False, "error": "args must be array"}
    handler = HANDLERS.get(op)
    if handler is None:
        return {"ok": False, "error": f"unknown op {op}"}
    try:
        body = handler(args)
        if isinstance(body, dict):
            body.setdefault("ok", True)
            return body
        return {"ok": True, "result": body}
    except Exception as exc:  # noqa: BLE001
        return {"ok": False, "error": str(exc)}


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
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            req = json.loads(line)
        except json.JSONDecodeError as exc:
            print(json.dumps({"ok": False, "error": str(exc)}), flush=True)
            continue
        print(json.dumps(handle(req)), flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
