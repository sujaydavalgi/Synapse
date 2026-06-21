/**
 * Positioning and wireless connectivity helpers for parse, verify, and runtime.
 * @module
 */

import type {
  BleServiceDecl,
  ConnectivityPolicyDecl,
  GeofenceDecl,
  HardwareDecl,
  RequiresConnectivityDecl,
} from "./foundations.js";
import type { TransportKind } from "./comm/index.js";
import type { CompatItem } from "./rust-bridge.js";

export type ConnectivityRequirement = "required" | "optional";

export type GeofenceRuntime = {
  name: string;
  centerLat: number;
  centerLon: number;
  radiusM: number;
};

export type ConnectivityPolicyRuntime = {
  name: string;
  preferred: string;
  fallback: string;
  emergency: string | null;
  switchIfLatencyMs: number | null;
  switchIfPacketLossPct: number | null;
};

export function haversineM(lat1: number, lon1: number, lat2: number, lon2: number): number {
  // Great-circle distance in meters between two WGS84 coordinates.
  //
  // Parameters:
  // - `lat1`, `lon1` — first point in degrees
  // - `lat2`, `lon2` — second point in degrees
  //
  // Returns:
  // Distance in meters.
  //
  // Options:
  // None.
  //
  // Example:
  // const d = haversineM(30.0, -97.0, 30.1, -97.1);

  const r = 6_371_000;
  const dLat = ((lat2 - lat1) * Math.PI) / 180;
  const dLon = ((lon2 - lon1) * Math.PI) / 180;
  const a =
    Math.sin(dLat / 2) ** 2 +
    Math.cos((lat1 * Math.PI) / 180) *
      Math.cos((lat2 * Math.PI) / 180) *
      Math.sin(dLon / 2) ** 2;

  return 2 * r * Math.asin(Math.sqrt(a));
}

export function geofenceContains(fence: GeofenceRuntime, lat: number, lon: number): boolean {
  // Return true when the coordinate lies inside the geofence circle.
  //
  // Parameters:
  // - `fence` — runtime geofence definition
  // - `lat`, `lon` — probe coordinate in degrees
  //
  // Returns:
  // Whether the point is inside the fence radius.
  //
  // Options:
  // None.
  //
  // Example:
  // const inside = geofenceContains(fence, 30.2672, -97.7431);

  return haversineM(lat, lon, fence.centerLat, fence.centerLon) <= fence.radiusM;
}

export function geofenceFromDecl(decl: GeofenceDecl): GeofenceRuntime {
  // Build runtime geofence state from an AST declaration.
  //
  // Parameters:
  // - `decl` — parsed geofence block
  //
  // Returns:
  // Runtime geofence circle.
  //
  // Options:
  // None.
  //
  // Example:
  // const fence = geofenceFromDecl(decl);

  return {
    name: decl.name,
    centerLat: decl.centerLat,
    centerLon: decl.centerLon,
    radiusM: decl.radiusM,
  };
}

export function connectivityPolicyFromDecl(
  decl: ConnectivityPolicyDecl,
): ConnectivityPolicyRuntime {
  // Build runtime failover policy from an AST declaration.
  //
  // Parameters:
  // - `decl` — parsed connectivity_policy block
  //
  // Returns:
  // Runtime policy used by the interpreter.
  //
  // Options:
  // None.
  //
  // Example:
  // const policy = connectivityPolicyFromDecl(decl);

  return {
    name: decl.name,
    preferred: decl.preferred,
    fallback: decl.fallback,
    emergency: decl.emergency,
    switchIfLatencyMs: decl.switchIfLatencyMs,
    switchIfPacketLossPct: decl.switchIfPacketLossPct,
  };
}

export function connectivityCapabilities(): string[] {
  // List security capabilities for positioning and connectivity.
  //
  // Parameters:
  // None.
  //
  // Returns:
  // Capability identifier strings.
  //
  // Options:
  // None.
  //
  // Example:
  // const caps = connectivityCapabilities();

  return [
    "gps.read",
    "network.status",
    "wifi.connect",
    "bluetooth.scan",
    "bluetooth.pair",
    "cellular.connect",
    "network.failover",
  ];
}

export function positioningSensorTypes(): string[] {
  // Return built-in positioning sensor type names.
  //
  // Parameters:
  // None.
  //
  // Returns:
  // Sensor type identifiers.
  //
  // Options:
  // None.
  //
  // Example:
  // const types = positioningSensorTypes();

  return ["GPS", "GNSS"];
}

export function connectivityLinkTypes(): string[] {
  // Return built-in wireless link type names for hardware profiles.
  //
  // Parameters:
  // None.
  //
  // Returns:
  // Connectivity type identifiers.
  //
  // Options:
  // None.
  //
  // Example:
  // const links = connectivityLinkTypes();

  return ["WiFi", "WiFi6", "Bluetooth", "Bluetooth5", "LTE", "5G", "GPS", "Satellite"];
}

export function connectivityFaultNames(): string[] {
  // Return simulation fault names for positioning and connectivity.
  //
  // Parameters:
  // None.
  //
  // Returns:
  // Fault identifier strings.
  //
  // Options:
  // None.
  //
  // Example:
  // const faults = connectivityFaultNames();

  return [
    "GPSLost",
    "GpsFailure",
    "GpsDrift",
    "GpsSpoofing",
    "NetworkOutage",
    "NetworkLatencySpike",
    "WeakWifi",
    "LteOutage",
    "FiveGHandoff",
    "BluetoothDisconnect",
    "PacketLoss",
    "LatencySpike",
  ];
}

export function connectivityKeyToProfileTokens(key: string): string[] {
  // Map requires_connectivity channel keys to hardware profile tokens.
  //
  // Parameters:
  // - `key` — channel name from requires_connectivity
  //
  // Returns:
  // Matching hardware connectivity identifiers.
  //
  // Options:
  // None.
  //
  // Example:
  // const tokens = connectivityKeyToProfileTokens("wifi");

  switch (key) {
    case "gps":
      return ["GPS"];
    case "gnss":
      return ["GNSS", "GPS"];
    case "wifi":
      return ["WiFi", "WiFi6"];
    case "bluetooth":
      return ["Bluetooth", "Bluetooth5", "BLE"];
    case "cellular":
      return ["LTE", "FourG", "4G", "FiveG", "5G"];
    case "ethernet":
      return ["Ethernet"];
    case "mesh":
      return ["Mesh"];
    case "satellite":
      return ["Satellite"];
    default:
      return [];
  }
}

export function faultToConnectivity(
  fault: string,
): { domain: string; event: string } | null {
  // Map a simulation fault to a connectivity trigger domain and event.
  //
  // Parameters:
  // - `fault` — fault name from simulate_compatibility or comm bus
  //
  // Returns:
  // Trigger pair, or null when the fault is unrelated.
  //
  // Options:
  // None.
  //
  // Example:
  // const evt = faultToConnectivity("NetworkOutage");

  switch (fault) {
    case "NetworkOutage":
    case "LteOutage":
    case "WeakWifi":
      return { domain: "network", event: "disconnected" };
    case "BluetoothDisconnect":
      return { domain: "bluetooth", event: "device_disconnected" };
    case "FiveGHandoff":
      return { domain: "cellular", event: "roaming" };
    default:
      return null;
  }
}

export function connectivityLinkToTransport(link: string): TransportKind {
  // Map an active connectivity link to the default transport kind.
  //
  // Parameters:
  // - `link` — link name from connectivity_policy or robot.connectivity_link()
  //
  // Returns:
  // Transport kind used for comm publish/subscribe.
  //
  // Options:
  // None.
  //
  // Example:
  // const transport = connectivityLinkToTransport("wifi");

  switch (link.toLowerCase()) {
    case "wifi":
      return "mqtt";
    case "cellular":
    case "lte":
    case "4g":
    case "fiveg":
    case "5g":
      return "dds";
    case "bluetooth":
    case "ble":
      return "websocket";
    case "ethernet":
      return "ros2";
    case "network":
      return "sim";
    default:
      return "sim";
  }
}

function compatItem(
  category: string,
  message: string,
  severity: CompatItem["severity"],
  line: number,
  column: number,
): CompatItem {
  return { category, message, severity, line, column };
}

export function verifyRequiresConnectivity(
  req: RequiresConnectivityDecl,
  profile: HardwareDecl,
): CompatItem[] {
  // Verify requires_connectivity against a hardware profile (TypeScript fallback).
  //
  // Parameters:
  // - `req` — parsed requires_connectivity block
  // - `profile` — deploy target hardware profile
  //
  // Returns:
  // Compatibility check items.
  //
  // Options:
  // None.
  //
  // Example:
  // const items = verifyRequiresConnectivity(req, profile);

  const items: CompatItem[] = [];
  const line = req.span.start.line;
  const column = req.span.start.column;
  const profileSet = new Set(profile.connectivity ?? []);

  for (const [key, level] of req.channels) {
    if (level !== "required") continue;
    const tokens = connectivityKeyToProfileTokens(key);
    if (tokens.length === 0) {
      items.push(
        compatItem(
          "connectivity",
          `Unknown connectivity key '${key}' in requires_connectivity`,
          "warning",
          line,
          column,
        ),
      );
      continue;
    }
    const satisfied = tokens.some((t) => profileSet.has(t));
    if (satisfied) {
      items.push(
        compatItem(
          "connectivity",
          `Required connectivity '${key}' present on '${profile.name}'`,
          "pass",
          line,
          column,
        ),
      );
    } else {
      items.push(
        compatItem(
          "connectivity",
          `Required connectivity '${key}' not on '${profile.name}' [${[...profileSet].join(", ")}]`,
          "error",
          line,
          column,
        ),
      );
    }
  }

  if (req.bandwidthMbpsMin != null) {
    const minBw = req.bandwidthMbpsMin;
    const bw = profile.networkBandwidthMbps;
    if (bw == null) {
      items.push(
        compatItem(
          "connectivity",
          "Target bandwidth unknown — cannot verify connectivity bandwidth requirement",
          "warning",
          line,
          column,
        ),
      );
    } else if (bw >= minBw) {
      items.push(
        compatItem(
          "connectivity",
          `Bandwidth ${bw} Mbps meets connectivity requirement >= ${minBw} Mbps`,
          "pass",
          line,
          column,
        ),
      );
    } else {
      items.push(
        compatItem(
          "connectivity",
          `Connectivity bandwidth requirement ${minBw} Mbps exceeds target ${bw} Mbps`,
          "error",
          line,
          column,
        ),
      );
    }
  }

  if (req.latencyMsMax != null) {
    const maxLat = req.latencyMsMax;
    const lat = profile.networkLatencyMs;
    if (lat == null) {
      items.push(
        compatItem(
          "connectivity",
          "Target latency unknown — cannot verify connectivity latency requirement",
          "warning",
          line,
          column,
        ),
      );
    } else if (lat <= maxLat) {
      items.push(
        compatItem(
          "connectivity",
          `Latency ${lat} ms meets connectivity requirement <= ${maxLat} ms`,
          "pass",
          line,
          column,
        ),
      );
    } else {
      items.push(
        compatItem(
          "connectivity",
          `Connectivity latency requirement ${maxLat} ms exceeded by target ${lat} ms`,
          "error",
          line,
          column,
        ),
      );
    }
  }

  if (req.packetLossPctMax != null) {
    items.push(
      compatItem(
        "connectivity",
        "Target packet loss unknown — cannot verify packet_loss requirement (TS verify)",
        "warning",
        line,
        column,
      ),
    );
  }

  return items;
}

export function validateGeofence(geofence: GeofenceDecl): CompatItem[] {
  // Validate geofence WGS84 geometry.
  //
  // Parameters:
  // - `geofence` — parsed geofence declaration
  //
  // Returns:
  // Compatibility check items.
  //
  // Options:
  // None.
  //
  // Example:
  // const items = validateGeofence(geofence);

  const line = geofence.span.start.line;
  const column = geofence.span.start.column;
  if (geofence.centerLat < -90 || geofence.centerLat > 90) {
    return [
      compatItem(
        "geofence",
        `Geofence '${geofence.name}' center latitude ${geofence.centerLat} out of range [-90, 90]`,
        "error",
        line,
        column,
      ),
    ];
  }
  if (geofence.centerLon < -180 || geofence.centerLon > 180) {
    return [
      compatItem(
        "geofence",
        `Geofence '${geofence.name}' center longitude ${geofence.centerLon} out of range [-180, 180]`,
        "error",
        line,
        column,
      ),
    ];
  }
  if (geofence.radiusM <= 0) {
    return [
      compatItem(
        "geofence",
        `Geofence '${geofence.name}' radius must be positive`,
        "error",
        line,
        column,
      ),
    ];
  }
  return [
    compatItem(
      "geofence",
      `Geofence '${geofence.name}' geometry valid`,
      "pass",
      line,
      column,
    ),
  ];
}

export function validateConnectivityPolicy(policy: ConnectivityPolicyDecl): CompatItem[] {
  // Validate connectivity failover policy declarations.
  //
  // Parameters:
  // - `policy` — parsed connectivity_policy block
  //
  // Returns:
  // Compatibility check items.
  //
  // Options:
  // None.
  //
  // Example:
  // const items = validateConnectivityPolicy(policy);

  const line = policy.span.start.line;
  const column = policy.span.start.column;
  const items: CompatItem[] = [
    compatItem(
      "connectivity_policy",
      `Connectivity policy '${policy.name}' parsed: preferred=${policy.preferred}, fallback=${policy.fallback}`,
      "pass",
      line,
      column,
    ),
  ];
  if (policy.preferred === policy.fallback) {
    items.push(
      compatItem(
        "connectivity_policy",
        `Policy '${policy.name}' preferred and fallback are the same link`,
        "warning",
        line,
        column,
      ),
    );
  }
  if (policy.emergency && (policy.emergency === policy.preferred || policy.emergency === policy.fallback)) {
    items.push(
      compatItem(
        "connectivity_policy",
        `Policy '${policy.name}' emergency link duplicates preferred or fallback`,
        "warning",
        line,
        column,
      ),
    );
  }
  return items;
}

export type { BleServiceDecl };
