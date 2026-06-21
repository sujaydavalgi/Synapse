/**
 * TypeScript hardware verification fallback when the native CLI is unavailable.
 * @module
 */

import type { Program } from "./ast/nodes.js";
import type { CompatItem, VerifyResult } from "./rust-bridge.js";
import {
  validateConnectivityPolicy,
  validateGeofence,
  verifyRequiresConnectivity,
} from "./connectivity-positioning.js";

export type VerifyHardwareTsOptions = {
  target?: string;
};

export function verifyHardwareProgram(
  program: Program,
  options: VerifyHardwareTsOptions = {},
): VerifyResult {
  // Run connectivity-focused hardware verification in TypeScript.
  //
  // Parameters:
  // - `program` — parsed Spanda program
  // - `options` — optional deploy target override
  //
  // Returns:
  // Verify result compatible with the native CLI JSON shape.
  //
  // Options:
  // - `options.target` — hardware profile name
  //
  // Example:
  // const result = verifyHardwareProgram(program, { target: "RoverV2" });

  const targetName =
    options.target ??
    program.deployments[0]?.targets[0] ??
    program.hardwareProfiles[0]?.name ??
    "unknown";
  const profile = program.hardwareProfiles.find((p) => p.name === targetName);
  const items: CompatItem[] = [];

  for (const geofence of program.geofences) {
    items.push(...validateGeofence(geofence));
  }
  for (const policy of program.connectivityPolicies) {
    items.push(...validateConnectivityPolicy(policy));
  }

  const req =
    program.requiresConnectivity ??
    program.robots.find((r) => r.requiresConnectivity)?.requiresConnectivity ??
    null;
  if (req && profile) {
    items.push(...verifyRequiresConnectivity(req, profile));
  } else if (req && !profile) {
    items.push({
      category: "connectivity",
      message: `Hardware profile '${targetName}' not found for connectivity verify`,
      severity: "error",
      line: req.span.start.line,
      column: req.span.start.column,
    });
  }

  if (items.length === 0) {
    items.push({
      category: "connectivity",
      message: "No connectivity requirements or geofences to verify (TS fallback)",
      severity: "pass",
      line: 1,
      column: 1,
    });
  }

  const ok = !items.some((i) => i.severity === "error");
  return {
    ok,
    compatible: ok,
    target: targetName,
    items,
  };
}
