/**
 * Runtime dispatch from official package module exports to provider registry backends.
 * @module
 */

import type { RuntimeValue } from "../runtime/interpreter.js";
import { notifyProviderCall } from "../runtime/provider-observer.js";
import type { ProviderRegistry } from "./registry.js";
import { transportRegistryKey } from "./registry.js";

/** Map a dotted module import path to its backing official package name. */
export function officialPackageForModule(modulePath: string): string | null {
  // Description:
  //     OfficialPackageForModule.
  //
  // Inputs:
  //     modulePath: string
  //         Caller-supplied modulePath.
  //
  // Outputs:
  //     result: string | null
  //         Return value from `officialPackageForModule`.
  //
  // Example:

  //     const result = officialPackageForModule(modulePath);

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
    case "assurance.evidence":
      return "spanda-assurance";
    case "assurance.knowledge":
      return "spanda-knowledge-model";
    case "assurance.anomaly":
      return "spanda-anomaly";
    case "assurance.diagnosis":
      return "spanda-diagnosis";
    case "assurance.prognostics":
      return "spanda-prognostics";
    case "assurance.mission":
      return "spanda-mission-planning";
    case "assurance.continuity":
      return "spanda-mission-continuity";
    case "assurance.resilience":
      return "spanda-resilience";
    default:
      return null;
  }
}

function projectProviderKey(packageName: string): string {
  // Description:
  //     ProjectProviderKey.
  //
  // Inputs:
  //     packageName: string
  //         Caller-supplied packageName.
  //
  // Outputs:
  //     result: string
  //         Return value from `projectProviderKey`.
  //
  // Example:

  //     const result = projectProviderKey(packageName);

  return `${packageName}::project`;
}

function okInt(): RuntimeValue {
  // Description:
  //     OkInt.
  //
  // Inputs:
  //     None.
  //
  // Outputs:
  //     result: RuntimeValue
  //         Return value from `okInt`.
  //
  // Example:

  //     const result = okInt();

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
  // Description:
  //     DispatchOfficialPackageCall.
  //
  // Inputs:
  //     registry: ProviderRegistry
  //         Caller-supplied registry.
  //     modulePath: string
  //         Caller-supplied modulePath.
  //     functionName: string
  //         Caller-supplied functionName.
  //     _args: readonly RuntimeValue[]
  //         Caller-supplied args.
  //
  // Outputs:
  //     result: RuntimeValue | null
  //         Return value from `dispatchOfficialPackageCall`.
  //
  // Example:

  //     const result = dispatchOfficialPackageCall(registry, modulePath, functionName, _args);

  const packageName = officialPackageForModule(modulePath);
  if (!packageName || !registry.isOfficialPackage(packageName)) {
    return null;
  }

  const started = performance.now();
  const finish = (failed: boolean): void => {
    notifyProviderCall(modulePath, packageName, performance.now() - started, failed);
  };

  // TODO: Wire project-scoped provider registry lookup via key for dispatch paths that need per-project backends.
  const key = projectProviderKey(packageName);
  void key;

  if (modulePath === "positioning.gps" && functionName === "read") {
    if (!registry.hasCapability("positioning.read")) return null;
    finish(false);
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
    finish(false);
    return okInt();
  }
  if (modulePath === "communication.mqtt" && functionName === "publish_topic") {
    if (!registry.hasCapability("mqtt.publish")) return null;
    const transportKey = transportRegistryKey(packageName);
    registry.withTransport(transportKey, (transport) => {
      transport.publish("/topic", "std_msgs/String", { kind: "string", value: "ok" });
    });
    finish(false);
    return okInt();
  }
  if (modulePath === "navigation.slam" && functionName === "localize") {
    if (!registry.hasCapability("slam.localize")) return null;
    finish(false);
    return { kind: "object", typeName: "LocalizationEstimate", fields: {} };
  }
  if (
    (modulePath === "vision.opencv" || modulePath === "vision.yolo") &&
    functionName === "detect"
  ) {
    if (!registry.hasCapability("vision.detect")) return null;
    finish(false);
    return { kind: "object", typeName: "Detections", fields: {} };
  }
  if (
    (modulePath === "sim.gazebo" || modulePath === "sim.webots") &&
    functionName === "step"
  ) {
    if (!registry.hasCapability("simulation.step")) return null;
    finish(false);
    return okInt();
  }
  if (modulePath === "provenance.ledger" && functionName === "append") {
    if (!registry.hasCapability("audit.append")) return null;
    finish(false);
    return okInt();
  }

  return null;
}
