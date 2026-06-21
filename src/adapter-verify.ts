/**
 * Framework adapter import verification for TypeScript verify fallback.
 * @module
 */

import type { ImportDecl } from "./ast/nodes.js";
import type { CompatItem } from "./rust-bridge.js";

const FRAMEWORK_IMPORT_PACKAGES: Record<string, string> = {
  "robotics.ros2": "spanda-ros2",
  "communication.mqtt": "spanda-mqtt",
  "vision.opencv": "spanda-opencv",
  "vision.yolo": "spanda-yolo",
  "navigation.slam": "spanda-slam",
  "navigation.path_planning": "spanda-nav",
  "navigation.nav2": "spanda-nav2",
  "navigation.cartographer": "spanda-cartographer",
  "navigation.rtabmap": "spanda-rtabmap",
  "vision.detectron": "spanda-detectron",
  "manipulation.grasp": "spanda-manipulation",
  "hri.dialogue": "spanda-hri",
  "twin.sync": "spanda-digital-twin",
  "sim.gazebo": "spanda-sim-gazebo",
  "sim.webots": "spanda-sim-webots",
  "connectivity.ble": "spanda-ble",
  "positioning.gps": "spanda-gps",
  "connectivity.lte": "spanda-lte",
};

export function verifyFrameworkImports(imports: ImportDecl[]): CompatItem[] {
  // Match declared imports against known framework package stubs.
  const items: CompatItem[] = [];
  for (const imp of imports) {
    const pkg = FRAMEWORK_IMPORT_PACKAGES[imp.path];
    if (!pkg) continue;
    items.push({
      category: "adapter",
      message: `Framework import '${imp.path}' maps to ${pkg} — stub adapter (orchestration hook only)`,
      severity: "pass",
      line: imp.span.start.line,
      column: imp.span.start.column,
    });
  }
  return items;
}
