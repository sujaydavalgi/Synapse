/** Standard library namespace paths for `import std.*;` */
export const STD_NAMESPACES = new Set([
  "std.time",
  "std.units",
  "std.spatial",
  "std.ai",
  "std.robotics",
  "std.sensors",
  "std.actuators",
  "std.safety",
  "std.communication",
  "std.hardware",
  "std.sim",
  "std.twin",
  "std.hri",
  "std.network",
]);

export function resolveStdImport(path: string): boolean {
  return STD_NAMESPACES.has(path);
}
