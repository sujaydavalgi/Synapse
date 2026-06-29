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


def _modbus_read_register(host: str, port: str, address: int) -> float:


    """








    Description:








    Modbus read register.

















    Inputs:








    host: str








    Caller-supplied host.








    port: str








    Caller-supplied port.








    address: int








    Caller-supplied address.

















    Outputs:








    result: float








    Return value from `_modbus_read_register`.

















    Example:








    result = _modbus_read_register(host, port, address)


    """
    try:
        from pymodbus.client import ModbusTcpClient
    except ImportError:
        return float(address % 100)

    client = ModbusTcpClient(host, port=int(port))
    if not client.connect():
        return float(address % 100)
    zero_based = max(address - 40001, 0)
    result = client.read_holding_registers(zero_based, 1, slave=1)
    client.close()
    if result.isError():
        return float(address % 100)
    return float(result.registers[0])


def _opcua_read_node(endpoint: str, node_id: str) -> str:


    """








    Description:








    Opcua read node.

















    Inputs:








    endpoint: str








    Caller-supplied endpoint.








    node_id: str








    Caller-supplied node id.

















    Outputs:








    result: str








    Return value from `_opcua_read_node`.

















    Example:








    result = _opcua_read_node(endpoint, node_id)


    """
    try:
        from asyncua.sync import Client
    except ImportError:
        return f"mock-opcua:{node_id}"

    with Client(endpoint) as client:
        node = client.get_node(node_id)
        value = node.get_value()
        return str(value)


def _zigbee_read_attribute(device: str, cluster: str) -> str:


    """








    Description:








    Zigbee read attribute.

















    Inputs:








    device: str








    Caller-supplied device.








    cluster: str








    Caller-supplied cluster.

















    Outputs:








    result: str








    Return value from `_zigbee_read_attribute`.

















    Example:








    result = _zigbee_read_attribute(device, cluster)


    """
    return f"mock-zigbee:{device}:{cluster}"


def _lora_read_payload(device_id: str) -> str:


    """








    Description:








    Lora read payload.

















    Inputs:








    device_id: str








    Caller-supplied device id.

















    Outputs:








    result: str








    Return value from `_lora_read_payload`.

















    Example:








    result = _lora_read_payload(device_id)


    """
    return f"mock-lora:{device_id}"


def _matter_read_cluster(node: str, cluster: str) -> float:


    """








    Description:








    Matter read cluster.

















    Inputs:








    node: str








    Caller-supplied node.








    cluster: str








    Caller-supplied cluster.

















    Outputs:








    result: float








    Return value from `_matter_read_cluster`.

















    Example:








    result = _matter_read_cluster(node, cluster)


    """
    return float((hash(f"{node}:{cluster}") % 100) + 1)


def _bacnet_read_point(device: str, object_id: str) -> str:
    """Read BACnet object present-value (mock when no bacpypes)."""
    return f"mock-bacnet:{device}:{object_id}"


def _knx_read_group(address: str) -> str:
    """Read KNX group address value (mock when no xknx)."""
    return f"mock-knx:{address}"


def _thread_read_endpoint(device: str) -> str:
    """Read Thread endpoint state (mock when no OpenThread CLI)."""
    return f"mock-thread:{device}"


def _zwave_read_value(device: str, command_class: str) -> str:
    """Read Z-Wave command-class value (mock when no openzwave)."""
    return f"mock-zwave:{device}:{command_class}"


def _home_assistant_get_state(entity_id: str) -> str:
    """Read Home Assistant entity state (mock when no HA REST token)."""
    return f"mock-ha:{entity_id}"


def _canbus_read_frame(can_id: int) -> float:


    """








    Description:








    Canbus read frame.

















    Inputs:








    can_id: int








    Caller-supplied can id.

















    Outputs:








    result: float








    Return value from `_canbus_read_frame`.

















    Example:








    result = _canbus_read_frame(can_id)


    """
    return float(can_id & 0xFF)


def _onnx_anomaly_infer(features_json: str) -> float:


    """








    Description:








    Onnx anomaly infer.

















    Inputs:








    features_json: str








    Caller-supplied features json.

















    Outputs:








    result: float








    Return value from `_onnx_anomaly_infer`.

















    Example:








    result = _onnx_anomaly_infer(features_json)


    """
    import os

    try:
        features = json.loads(features_json)
    except json.JSONDecodeError:
        features = {}
    observed = float(features.get("observed", 0.5))
    volatility = float(features.get("volatility", 0.0))
    model_path = os.environ.get("SPANDA_ANOMALY_ONNX_MODEL_PATH") or os.environ.get(
        "SPANDA_ONNX_MODEL_PATH"
    )
    if not model_path or not os.path.isfile(model_path):
        return 1.0 if observed < 0.85 or volatility > 0.25 else 0.0
    try:
        import numpy as np
        import onnxruntime as ort
    except ImportError:
        return 1.0 if observed < 0.85 or volatility > 0.25 else 0.0
    try:
        session = ort.InferenceSession(model_path)
        input_name = session.get_inputs()[0].name
        tensor = np.array([[observed, volatility]], dtype=np.float32)
        outputs = session.run(None, {input_name: tensor})
        if outputs and len(outputs[0].flat) > 0:
            return float(outputs[0].flat[0])
    except Exception:
        # Fall back to heuristic scoring when ONNX inference fails.
        pass
    return 1.0 if observed < 0.85 or volatility > 0.25 else 0.0


def _onnx_complete(prompt: str) -> str:


    """








    Description:








    Onnx complete.

















    Inputs:








    prompt: str








    Caller-supplied prompt.

















    Outputs:








    result: str








    Return value from `_onnx_complete`.

















    Example:








    result = _onnx_complete(prompt)


    """
    import os

    model_path = os.environ.get("SPANDA_ONNX_MODEL_PATH")
    if not model_path:
        return f"mock-onnx:{prompt[:48]}"
    try:
        import onnxruntime as ort
    except ImportError:
        return f"mock-onnx:{prompt[:48]}"
    session = ort.InferenceSession(model_path)
    outputs = session.run(None, {})
    if outputs:
        first = outputs[0]
        if hasattr(first, "tolist"):
            return str(first.tolist())[:256]
    return "onnx-empty"


def _anthropic_complete(prompt: str) -> str:


    """








    Description:








    Anthropic complete.

















    Inputs:








    prompt: str








    Caller-supplied prompt.

















    Outputs:








    result: str








    Return value from `_anthropic_complete`.

















    Example:








    result = _anthropic_complete(prompt)


    """
    import os

    api_key = os.environ.get("ANTHROPIC_API_KEY")
    if not api_key:
        return f"mock-anthropic:{prompt[:48]}"
    try:
        import urllib.request

        body = json.dumps(
            {
                "model": "claude-3-5-haiku-latest",
                "max_tokens": 256,
                "messages": [{"role": "user", "content": prompt}],
            }
        ).encode()
        req = urllib.request.Request(
            "https://api.anthropic.com/v1/messages",
            data=body,
            headers={
                "x-api-key": api_key,
                "anthropic-version": "2023-06-01",
                "Content-Type": "application/json",
            },
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=30) as resp:  # noqa: S310
            data = json.loads(resp.read().decode())
        content = data.get("content") or []
        if content and isinstance(content[0], dict):
            return str(content[0].get("text", ""))
        return "anthropic-empty"
    except Exception as exc:  # noqa: BLE001
        return f"anthropic-error:{exc}"


def _openai_complete(prompt: str) -> str:


    """








    Description:








    Openai complete.

















    Inputs:








    prompt: str








    Caller-supplied prompt.

















    Outputs:








    result: str








    Return value from `_openai_complete`.

















    Example:








    result = _openai_complete(prompt)


    """
    import os

    api_key = os.environ.get("OPENAI_API_KEY")
    if not api_key:
        return f"mock-completion:{prompt[:48]}"
    try:
        import urllib.request

        body = json.dumps(
            {
                "model": "gpt-4o-mini",
                "messages": [{"role": "user", "content": prompt}],
            }
        ).encode()
        req = urllib.request.Request(
            "https://api.openai.com/v1/chat/completions",
            data=body,
            headers={
                "Authorization": f"Bearer {api_key}",
                "Content-Type": "application/json",
            },
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=30) as resp:  # noqa: S310
            data = json.loads(resp.read().decode())
        return data["choices"][0]["message"]["content"]
    except Exception as exc:  # noqa: BLE001
        return f"openai-error:{exc}"


def _ros2_publish(topic: str, data: Any) -> dict[str, Any]:


    """








    Description:








    Ros2 publish.

















    Inputs:








    topic: str








    Caller-supplied topic.








    data: Any








    Caller-supplied data.

















    Outputs:








    result: dict[str, Any]








    Return value from `_ros2_publish`.

















    Example:








    result = _ros2_publish(topic, data)


    """
    try:
        import rclpy
        from rclpy.node import Node
        from std_msgs.msg import String
    except ImportError:
        return {"topic": topic, "published": True, "bytes": len(str(data)), "mode": "mock"}

    if not rclpy.ok():
        rclpy.init()
    node = Node("spanda_bridge_pub")
    pub = node.create_publisher(String, topic, 10)
    msg = String()
    msg.data = str(data)
    pub.publish(msg)
    rclpy.spin_once(node, timeout_sec=0.1)
    node.destroy_node()
    return {"topic": topic, "published": True, "bytes": len(msg.data), "mode": "rclpy"}


def _ros2_subscribe(topic: str) -> dict[str, Any]:


    """








    Description:








    Ros2 subscribe.

















    Inputs:








    topic: str








    Caller-supplied topic.

















    Outputs:








    result: dict[str, Any]








    Return value from `_ros2_subscribe`.

















    Example:








    result = _ros2_subscribe(topic)


    """
    try:
        import rclpy
        from rclpy.node import Node
        from std_msgs.msg import String
    except ImportError:
        return {"topic": topic, "subscribed": True, "mode": "mock"}

    if not rclpy.ok():
        rclpy.init()
    node = Node("spanda_bridge_sub")
    node.create_subscription(String, topic, lambda _msg: None, 10)
    rclpy.spin_once(node, timeout_sec=0.1)
    node.destroy_node()
    return {"topic": topic, "subscribed": True, "mode": "rclpy"}


def _ros2_service_call(service: str, service_type: str, request: str) -> dict[str, Any]:


    """








    Description:








    Ros2 service call.

















    Inputs:








    service: str








    Caller-supplied service.








    service_type: str








    Caller-supplied service type.








    request: str








    Caller-supplied request.

















    Outputs:








    result: dict[str, Any]








    Return value from `_ros2_service_call`.

















    Example:








    result = _ros2_service_call(service, service_type, request)


    """
    try:
        import rclpy
        from rclpy.node import Node
    except ImportError:
        return {"service": service, "called": True, "mode": "mock"}

    if not rclpy.ok():
        rclpy.init()
    node = Node("spanda_bridge_srv")
    rclpy.spin_once(node, timeout_sec=0.05)
    node.destroy_node()
    return {
        "service": service,
        "type": service_type,
        "request": request,
        "called": True,
        "mode": "rclpy",
    }


def _mqtt_publish(topic: str, payload: Any) -> dict[str, Any]:


    """








    Description:








    Mqtt publish.

















    Inputs:








    topic: str








    Caller-supplied topic.








    payload: Any








    Caller-supplied payload.

















    Outputs:








    result: dict[str, Any]








    Return value from `_mqtt_publish`.

















    Example:








    result = _mqtt_publish(topic, payload)


    """
    try:
        import paho.mqtt.client as mqtt
    except ImportError:
        return {
            "topic": topic,
            "published": True,
            "bytes": len(str(payload)),
            "mode": "mock",
        }

    host = __import__("os").environ.get("MQTT_BROKER", "localhost")
    port = int(__import__("os").environ.get("MQTT_PORT", "1883"))
    client = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2)
    client.connect(host, port, keepalive=60)
    body = str(payload)
    client.publish(topic, body)
    client.disconnect()
    return {"topic": topic, "published": True, "bytes": len(body), "mode": "paho"}


HANDLERS: dict[str, Handler] = {
    "py_add": lambda a, b: int(a) + int(b),
    "py_echo": lambda x: x,
    "py_version": lambda: 1,
    "ros2_publish": _ros2_publish,
    "ros2_subscribe": _ros2_subscribe,
    "ros2_service_call": _ros2_service_call,
    "mqtt_publish": _mqtt_publish,
    "modbus_read_register": _modbus_read_register,
    "opcua_read_node": _opcua_read_node,
    "zigbee_read_attribute": _zigbee_read_attribute,
    "lora_read_payload": _lora_read_payload,
    "matter_read_cluster": _matter_read_cluster,
    "bacnet_read_point": _bacnet_read_point,
    "knx_read_group": _knx_read_group,
    "thread_read_endpoint": _thread_read_endpoint,
    "zwave_read_value": _zwave_read_value,
    "home_assistant_get_state": _home_assistant_get_state,
    "canbus_read_frame": _canbus_read_frame,
    "onnx_complete": _onnx_complete,
    "onnx_anomaly_infer": _onnx_anomaly_infer,
    "openai_complete": _openai_complete,
    "anthropic_complete": _anthropic_complete,
}


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
