/**
 * index module (safety/index.ts).
 * @module
 */

import type { Environment, RobotState } from "../runtime/interpreter.js";

export type SafetyZoneRuntime = {
  name: string;
  shape: "circle" | "rect";
  x: number;
  y: number;
  radius?: number;
  width?: number;
  height?: number;
};

export type SafetyEvaluation = {
  allowed: boolean;
  reason?: string;
  emergencyStop: boolean;
};

export type SafetyConfig = {
  maxSpeed: number;
  stopIfRules: Array<(env: Environment) => boolean>;
  zones: SafetyZoneRuntime[];
  zoneSpeedCaps: Map<string, number>;
};

export class SafetyMonitor {
  private emergencyStop = false;

  constructor(private config: SafetyConfig) {}

  evaluateBeforeMotion(env: Environment, pose: { x: number; y: number }): SafetyEvaluation {
    // EvaluateBeforeMotion.
    //
    // Parameters:
    // - `env` — input value
    // - `pose` — input value
    //
    // Returns:
    // SafetyEvaluation.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = evaluateBeforeMotion(env, pose);

    const peek = this.peekBeforeMotion(env, pose);
    if (!peek.allowed && peek.emergencyStop) {
      this.emergencyStop = true;
    }
    return peek;
  }

  peekBeforeMotion(env: Environment, pose: { x: number; y: number }): SafetyEvaluation {
    // PeekBeforeMotion.
    //
    // Parameters:
    // - `env` — input value
    // - `pose` — input value
    //
    // Returns:
    // SafetyEvaluation.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = peekBeforeMotion(env, pose);

    if (this.emergencyStop) {
      return { allowed: false, reason: "Emergency stop active", emergencyStop: true };
    }

    for (const rule of this.config.stopIfRules) {
      if (rule(env)) {
        return {
          allowed: false,
          reason: "stop_if safety rule triggered",
          emergencyStop: true,
        };
      }
    }

    for (const zone of this.config.zones) {
      if (this.isPointInZone(pose.x, pose.y, zone)) {
        // Allow motion inside zones that only declare a program speed cap.
        if (this.config.zoneSpeedCaps.has(zone.name)) {
          continue;
        }
        return {
          allowed: false,
          reason: `Robot entered safety zone '${zone.name}'`,
          emergencyStop: true,
        };
      }
    }

    return { allowed: true, emergencyStop: false };
  }

  validateActionProposal(
    linear: number,
    angular: number,
    env: Environment,
    pose: { x: number; y: number },
  ): {
    // ValidateActionProposal.
    //
    // Parameters:
    // - `linear` — input value
    // - `angular` — input value
    // - `env` — input value
    // - `pose` — input value
    //
    // Returns:
    // .
    //
    // Options:
    // None.
    //
    // Example:

 // const result = validateActionProposal(linear, angular, env, pose);
 ok: true; linear: number; angular: number } | { ok: false; reason: string } {
    const peek = this.peekBeforeMotion(env, pose);
    if (!peek.allowed) {
      return { ok: false, reason: peek.reason ?? "Safety validation failed" };
    }
    return { ok: true, linear: this.clampSpeedAtPose(linear, pose), angular };
  }

  effectiveMaxSpeed(pose: { x: number; y: number }): number {
    // Compute the active speed cap from global max and program zone policies.
    let cap = this.config.maxSpeed;
    for (const zone of this.config.zones) {
      if (this.isPointInZone(pose.x, pose.y, zone)) {
        const zoneCap = this.config.zoneSpeedCaps.get(zone.name);
        if (zoneCap !== undefined) {
          cap = Math.min(cap, zoneCap);
        }
      }
    }
    return cap;
  }

  clampSpeedAtPose(requested: number, pose: { x: number; y: number }): number {
    // Clamp requested linear speed to the effective cap at the current pose.
    const sign = requested === 0 ? 1 : Math.sign(requested);
    return Math.min(Math.abs(requested), this.effectiveMaxSpeed(pose)) * sign;
  }

  isInZone(zoneName: string, pose: { x: number; y: number }): boolean {
    // IsInZone.
    //
    // Parameters:
    // - `zoneName` — input value
    // - `pose` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = isInZone(zoneName, pose);

    const zone = this.config.zones.find((z) => z.name === zoneName);
    if (!zone) return false;
    return this.isPointInZone(pose.x, pose.y, zone);
  }

  clampSpeed(requested: number): number {
    // ClampSpeed.
    //
    // Parameters:
    // - `requested` — input value
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = clampSpeed(requested);

    return Math.min(Math.abs(requested), this.config.maxSpeed) * Math.sign(requested || 1);
  }

  isEmergencyStop(): boolean {
    // IsEmergencyStop.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = isEmergencyStop();

    return this.emergencyStop;
  }

  setEmergencyStop(active: boolean): void {
    // SetEmergencyStop.
    //
    // Parameters:
    // - `active` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = setEmergencyStop(active);

    this.emergencyStop = active;
  }

  reset(): void {
    // Reset the value.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = reset();

    this.emergencyStop = false;
  }

  private isPointInZone(x: number, y: number, zone: SafetyZoneRuntime): boolean {    // continue when shape equals radius !== undefined.
    if (zone.shape === "circle" && zone.radius !== undefined) {
      const dx = x - zone.x;
      const dy = y - zone.y;
      return Math.sqrt(dx * dx + dy * dy) <= zone.radius;
    }

    // continue when shape equals height !== undefined.
    if (zone.shape === "rect" && zone.width !== undefined && zone.height !== undefined) {
      return x >= zone.x && x <= zone.x + zone.width && y >= zone.y && y <= zone.y + zone.height;
    }
    return false;
}
}

export function createSafetyConfigFromRobot(
  maxSpeed: number,
  stopIfRules: Array<(env: Environment) => boolean>,
  zones: SafetyZoneRuntime[] = [],
  zoneSpeedCaps: Map<string, number> = new Map(),
): SafetyConfig {
  // Build a safety monitor configuration from robot rules and program zone caps.
  //
  // Parameters:
  // - `maxSpeed` — global maximum linear speed
  // - `stopIfRules` — runtime stop-if predicates
  // - `zones` — geometric safety zones on the robot
  // - `zoneSpeedCaps` — program-level caps keyed by zone name
  //
  // Returns:
  // Safety configuration for `SafetyMonitor`.
  //
  // Options:
  // None.
  //
  // Example:
  // createSafetyConfigFromRobot(1.0, [], zones, caps);

  return { maxSpeed, stopIfRules, zones, zoneSpeedCaps };
}

export function applyEmergencyStop(state: RobotState): RobotState {
  // ApplyEmergencyStop.
  //
  // Parameters:
  // - `state` — input value
  //
  // Returns:
  // `RobotState`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = applyEmergencyStop(state);
  return {
    ...state,
    emergencyStop: true,
    velocity: { linear: 0, angular: 0 },
  };
}

export function interpolatePoses(
  from: { x: number; y: number; theta: number; z?: number },
  to: { x: number; y: number; theta: number; z?: number },
  steps: number,
): Array<{
  // InterpolatePoses.
  //
  // Parameters:
  // - `from` — optional input
  // - `to` — optional input
  // - `steps` — input value
  //
  // Returns:
  // `Array<`.
  //
  // Options:
  // - `from` — optional parameter
  // - `to` — optional parameter
  //
  // Example:

 // const result = interpolatePoses(from, to, steps);
 x: number; y: number; theta: number; z: number }> {
  const count = Math.max(2, Math.floor(steps));
  const waypoints: Array<{ x: number; y: number; theta: number; z: number }> = [];
  for (let i = 0; i < count; i++) {
    const t = i / (count - 1);
    waypoints.push({
      x: from.x + (to.x - from.x) * t,
      y: from.y + (to.y - from.y) * t,
      theta: from.theta + (to.theta - from.theta) * t,
      z: (from.z ?? 0) + ((to.z ?? 0) - (from.z ?? 0)) * t,
    });
  }
  return waypoints;
}
