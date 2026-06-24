/**
 * Span lookup for readiness diagnostics (TypeScript mirror of spanda-readiness spans).
 * @module
 */

import type { Program } from "./ast/nodes.js";

type Spanned = { span: { start: { line: number; column: number } } };

function atSpan(node?: Spanned): { line: number; column: number } {
  return node ? { line: node.span.start.line, column: node.span.start.column } : { line: 1, column: 1 };
}

function deploySpan(program: Program): { line: number; column: number } | undefined {
  const deploy = program.deployments?.[0];
  if (deploy && deploy.kind === "DeployDecl") {
    return atSpan(deploy);
  }
  return undefined;
}

function firstRobotSpan(program: Program): { line: number; column: number } {
  return atSpan(program.robots?.[0]);
}

function missionSpan(program: Program): Spanned | undefined {
  const mission = program.robots?.find((r) => r.mission)?.mission;
  return mission ?? undefined;
}

function firstRobotSafetySpan(program: Program): Spanned | undefined {
  for (const robot of program.robots ?? []) {
    if (robot.safety) return robot.safety;
  }
  return undefined;
}

/** Resolve a display line/column for a readiness issue factor. */
export function lineColumnForFactor(program: Program, factor: string): { line: number; column: number } {
  // Map readiness factors to AST spans, matching crates/spanda-readiness/src/spans.rs.
  //
  // Parameters:
  // - `program` — parsed `.sd` program
  // - `factor` — readiness factor label from a report issue
  //
  // Returns:
  // Best-effort source location for IDE diagnostics.
  //
  // Options:
  // None.
  //
  // Example:
  // const span = lineColumnForFactor(program, "Health");

  switch (factor) {
    case "Hardware":
    case "Battery":
    case "Connectivity":
    case "Storage":
    case "Compute":
    case "Packages":
    case "Providers":
      return deploySpan(program) ?? firstRobotSpan(program);
    case "Health": {
      const health = program.healthChecks?.[0];
      return health ? atSpan(health) : firstRobotSpan(program);
    }
    case "Capabilities":
    case "Mission Requirements": {
      const mission = missionSpan(program);
      return mission ? atSpan(mission) : firstRobotSpan(program);
    }
    case "Safety": {
      const safety = firstRobotSafetySpan(program);
      return safety ? atSpan(safety) : firstRobotSpan(program);
    }
    case "Fleet": {
      const fleet = program.fleets?.[0];
      return fleet ? atSpan(fleet) : { line: 1, column: 1 };
    }
    default:
      return { line: 1, column: 1 };
  }
}
