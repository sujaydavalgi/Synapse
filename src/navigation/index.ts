/**
 * Navigation helpers and Nav2 golden-path bridge hooks.
 * @module
 */

import type { RobotBackend, RuntimeValue } from "../runtime/interpreter.js";
import { runtimeVelocity } from "../runtime/values.js";

export type Nav2BridgeContext = {
  backend: RobotBackend;
  topicPathToMessageType: Map<string, string>;
  goal: string | null;
  linearMps?: number;
  angularRadS?: number;
  onLog?: (message: string) => void;
};

const DEFAULT_CMD_VEL_TOPIC = "/cmd_vel";

export function tryPublishNav2CmdVel(ctx: Nav2BridgeContext): boolean {
  // Publish a velocity command on `/cmd_vel` when the robot declares that topic.
  //
  // Parameters:
  // - `ctx` — backend, topic map, goal text, and optional velocity overrides
  //
  // Returns:
  // true when a publish was attempted on a declared `/cmd_vel` topic.
  //
  // Options:
  // - `linearMps` — defaults to 0.2 m/s
  // - `angularRadS` — defaults to 0.0 rad/s
  //
  // Example:
  // tryPublishNav2CmdVel({ backend, topicPathToMessageType, goal: "Dock" });

  const messageType = ctx.topicPathToMessageType.get(DEFAULT_CMD_VEL_TOPIC);
  if (!messageType) {
    return false;
  }

  const linear = ctx.linearMps ?? 0.2;
  const angular = ctx.angularRadS ?? 0.0;
  const velocity = runtimeVelocity(linear, angular);
  ctx.backend.publishTopic?.(DEFAULT_CMD_VEL_TOPIC, messageType, velocity);
  ctx.onLog?.(
    `navigation: Nav2 bridge publish ${DEFAULT_CMD_VEL_TOPIC} goal='${ctx.goal ?? "none"}' (${linear} m/s)`,
  );
  return true;
}
