/**
 * Positioning and wireless connectivity helpers for parse, verify, and runtime.
 * @module
 */

import type {
  BleServiceDecl,
  ConnectivityPolicyDecl,
  GeofenceDecl,
} from "./foundations.js";

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

export type { BleServiceDecl };
