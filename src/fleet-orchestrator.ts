/**
 * Fleet orchestration beyond in-process fleet run.
 * @module
 */

import type { Program } from "./ast/nodes.js";
import {
  createMissionRuntime,
  missionAdvance,
  missionStart,
  type MissionRuntime,
} from "./robotics-platform.js";

export type FleetMemberState = {
  robotName: string;
  missionName: string | null;
  missionState: string;
  currentStep: string;
  hasPeerLink: boolean;
};

export type FleetOrchestrationReport = {
  fleetName: string;
  members: FleetMemberState[];
  coordinationMode: string;
  stepsAdvanced: number;
};

export type FleetOrchestrationResult = {
  program: string;
  fleets: FleetOrchestrationReport[];
  success: boolean;
};

function missionForRobot(robot: Program["robots"][number]): MissionRuntime | null {
  if (!robot.mission) return null;
  return createMissionRuntime(
    robot.mission.name,
    [...robot.mission.steps],
    robot.mission.durationHours,
  );
}

export function orchestrateFleets(program: Program, programPath: string): FleetOrchestrationResult {
  // Coordinate declared fleet groups using each member robot's mission controller.
  const reports: FleetOrchestrationReport[] = [];

  for (const fleet of program.fleets) {
    const members: FleetMemberState[] = [];
    let stepsAdvanced = 0;

    for (const memberName of fleet.members) {
      const robot = program.robots.find((r) => r.name === memberName);
      if (!robot) {
        members.push({
          robotName: memberName,
          missionName: null,
          missionState: "MissingRobot",
          currentStep: "",
          hasPeerLink: false,
        });
        continue;
      }

      const runtime = missionForRobot(robot);
      if (runtime) {
        missionStart(runtime);
        const step = missionAdvance(runtime);
        if (step) stepsAdvanced += 1;
        members.push({
          robotName: memberName,
          missionName: runtime.name,
          missionState: runtime.state,
          currentStep: step,
          hasPeerLink: (robot.peerRobots?.length ?? 0) > 0,
        });
      } else {
        members.push({
          robotName: memberName,
          missionName: null,
          missionState: "NoMission",
          currentStep: "",
          hasPeerLink: (robot.peerRobots?.length ?? 0) > 0,
        });
      }
    }

    reports.push({
      fleetName: fleet.name,
      members,
      coordinationMode: "round_robin_mission",
      stepsAdvanced,
    });
  }

  const success = reports.every((r) =>
    r.members.every((m) => m.missionState !== "MissingRobot"),
  );

  return { program: programPath, fleets: reports, success };
}
