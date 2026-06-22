/**
 * Runtime dispatch from official package module exports to provider registry backends.
 * @module
 */

import type { RuntimeValue } from "../runtime/interpreter.js";
import type { ProviderRegistry } from "./registry.js";
import { transportRegistryKey } from "./registry.js";

/** Map a dotted module import path to its backing official package name. */
export function officialPackageForModule(modulePath: string): string | null {
  switch (modulePath) {
    case "positioning.gps":
      return "spanda-gps";
    case "connectivity.wifi":
      return "spanda-wifi";
    case "connectivity.ble":
      return "spanda-ble";
    case "connectivity.cellular":
      return "spanda-cellular";
    case "navigation.path_planning":
      return "spanda-nav";
    case "navigation.slam":
      return "spanda-slam";
    case "robotics.fleet":
      return "spanda-fleet";
    case "communication.mqtt":
      return "spanda-mqtt";
    case "communication.dds":
      return "spanda-dds";
    case "robotics.ros2":
      return "spanda-ros2";
    case "deploy.ota":
      return "spanda-ota";
    case "vision.opencv":
      return "spanda-opencv";
    case "vision.yolo":
      return "spanda-yolo";
    case "sim.gazebo":
      return "spanda-gazebo";
    case "sim.webots":
      return "spanda-webots";
    case "ai.openai":
      return "spanda-openai";
    case "provenance.ledger":
      return "spanda-ledger";
    default:
      return null;
  }
}

function projectProviderKey(packageName: string): string {
  return `${packageName}::project`;
}

function okInt(): RuntimeValue {
  return { kind: "number", value: 1, unit: "none" };
}

/**
 * Dispatch an imported official-package function when the backing package is installed.
 * Returns `null` to fall back to the `.sd` stub body.
 */
export function dispatchOfficialPackageCall(
  registry: ProviderRegistry,
  modulePath: string,
  functionName: string,
  _args: readonly RuntimeValue[],
): RuntimeValue | null {
  const packageName = officialPackageForModule(modulePath);
  if (!packageName || !registry.isOfficialPackage(packageName)) {
    return null;
  }

  const key = projectProviderKey(packageName);

  if (modulePath === "positioning.gps" && functionName === "read") {
    if (!registry.hasCapability("positioning.read")) return null;
    return {
      kind: "object",
      typeName: "GeoPoint",
      fields: {
        lat: { kind: "number", value: 37, unit: "none" },
        lon: { kind: "number", value: -122, unit: "none" },
      },
    };
  }
  if (modulePath === "connectivity.wifi" && functionName === "connect") {
    if (!registry.hasCapability("connectivity.wifi")) return null;
    return okInt();
  }
  if (modulePath === "communication.mqtt" && functionName === "publish_topic") {
    if (!registry.hasCapability("mqtt.publish")) return null;
    const transportKey = transportRegistryKey(packageName);
    registry.withTransport(transportKey, (transport) => {
      transport.publish("/topic", "std_msgs/String", { kind: "string", value: "ok" });
    });
    return okInt();
  }
  if (modulePath === "navigation.slam" && functionName === "localize") {
    if (!registry.hasCapability("slam.localize")) return null;
    return { kind: "object", typeName: "LocalizationEstimate", fields: {} };
  }
  if (
    (modulePath === "vision.opencv" || modulePath === "vision.yolo") &&
    functionName === "detect"
  ) {
    if (!registry.hasCapability("vision.detect")) return null;
    return { kind: "object", typeName: "Detections", fields: {} };
  }
  if (
    (modulePath === "sim.gazebo" || modulePath === "sim.webots") &&
    functionName === "step"
  ) {
    if (!registry.hasCapability("simulation.step")) return null;
    return okInt();
  }
  if (modulePath === "provenance.ledger" && functionName === "append") {
    if (!registry.hasCapability("audit.append")) return null;
    return okInt();
  }

  void key;
  return null;
}
