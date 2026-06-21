/**
 * Experimental swarm coordinator runtime built on fleet declarations and mission controllers.
 * @module
 */

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import type { Program } from "./ast/nodes.js";
import type { SwarmDecl, SwarmPolicy } from "./foundations.js";
import {
  createMissionRuntime,
  missionAdvance,
  missionStart,
  type MissionRuntime,
} from "./robotics-platform.js";
import type { FleetMemberState, PeerDelivery } from "./fleet-orchestrator.js";

export type SwarmState = {
  roundRobinCursor: Record<string, number>;
};

export type SwarmCoordinationReport = {
  swarmName: string;
  fleetName: string;
  policy: SwarmPolicy;
  activeMember: string | null;
  members: FleetMemberState[];
  stepsAdvanced: number;
  coordinationMode: string;
  peerDeliveries: PeerDelivery[];
  roundRobinCursor: number;
};

export type SwarmCoordinationResult = {
  program: string;
  swarms: SwarmCoordinationReport[];
  success: boolean;
};

export function defaultSwarmStatePath(): string {
  return process.env.SPANDA_SWARM_STATE ?? ".spanda/swarm-state.json";
}

export function emptySwarmState(): SwarmState {
  return { roundRobinCursor: {} };
}

export function readSwarmStateFromDisk(path = defaultSwarmStatePath()): SwarmState {
  if (!existsSync(path)) return emptySwarmState();
  try {
    const parsed = JSON.parse(readFileSync(path, "utf-8")) as {
      round_robin_cursor?: Record<string, number>;
      roundRobinCursor?: Record<string, number>;
    };
    return {
      roundRobinCursor: parsed.roundRobinCursor ?? parsed.round_robin_cursor ?? {},
    };
  } catch {
    return emptySwarmState();
  }
}

export function writeSwarmStateToDisk(state: SwarmState, path = defaultSwarmStatePath()): void {
  const abs = resolve(path);
  mkdirSync(dirname(abs), { recursive: true });
  writeFileSync(abs, JSON.stringify({ round_robin_cursor: state.roundRobinCursor }, null, 2));
}

function missionForRobot(robot: Program["robots"][number]): MissionRuntime | null {
  if (!robot.mission) return null;
  return createMissionRuntime(
    robot.mission.name,
    [...robot.mission.steps],
    robot.mission.durationHours,
  );
}

function advanceMember(program: Program, memberName: string): FleetMemberState {
  const robot = program.robots.find((entry) => entry.name === memberName);
  if (!robot) {
    return {
      robotName: memberName,
      missionName: null,
      missionState: "MissingRobot",
      currentStep: "",
      hasPeerLink: false,
    };
  }
  const runtime = missionForRobot(robot);
  if (!runtime) {
    return {
      robotName: memberName,
      missionName: null,
      missionState: "NoMission",
      currentStep: "",
      hasPeerLink: (robot.peerRobots?.length ?? 0) > 0,
    };
  }
  missionStart(runtime);
  const step = missionAdvance(runtime) ?? "";
  const peerHandoffs = (robot.peerRobots ?? []).flatMap((peer) =>
    step ? [`${memberName}->${peer.name}:step=${step}`] : [],
  );
  return {
    robotName: memberName,
    missionName: runtime.name,
    missionState: runtime.state,
    currentStep: step,
    hasPeerLink: (robot.peerRobots?.length ?? 0) > 0,
    peerHandoffs,
  };
}

function leaderFollowDeliveries(
  leader: string,
  step: string,
  members: string[],
): PeerDelivery[] {
  if (!step) return [];
  return members
    .filter((member) => member !== leader)
    .map((follower) => ({
      fromRobot: leader,
      toRobot: follower,
      topic: "mission_step",
      step,
      delivered: true,
    }));
}

function coordinateSwarmGroup(
  program: Program,
  swarm: SwarmDecl,
  cursor: number,
): SwarmCoordinationReport {
  const fleet = program.fleets.find((entry) => entry.name === swarm.fleetName);
  const members = fleet?.members ?? [];
  const memberStates: FleetMemberState[] = [];
  let peerDeliveries: PeerDelivery[] = [];
  let stepsAdvanced = 0;
  let activeMember: string | null = null;
  let nextCursor = cursor;

  if (swarm.policy === "round_robin") {
    if (members.length > 0) {
      const index = cursor % members.length;
      nextCursor = (index + 1) % members.length;
      const memberName = members[index]!;
      activeMember = memberName;
      const state = advanceMember(program, memberName);
      if (state.currentStep) stepsAdvanced = 1;
      memberStates.push(state);
    }
  } else if (swarm.policy === "broadcast") {
    for (const memberName of members) {
      const state = advanceMember(program, memberName);
      if (state.currentStep) stepsAdvanced += 1;
      memberStates.push(state);
    }
  } else {
    const leader = members[0];
    if (leader) {
      activeMember = leader;
      const state = advanceMember(program, leader);
      if (state.currentStep) stepsAdvanced = 1;
      peerDeliveries = leaderFollowDeliveries(leader, state.currentStep, members);
      memberStates.push(state);
    }
  }

  const coordinationMode =
    swarm.policy === "round_robin"
      ? "swarm_round_robin"
      : swarm.policy === "broadcast"
        ? "swarm_broadcast"
        : "swarm_leader_follow";

  return {
    swarmName: swarm.name,
    fleetName: swarm.fleetName,
    policy: swarm.policy,
    activeMember,
    members: memberStates,
    stepsAdvanced,
    coordinationMode,
    peerDeliveries,
    roundRobinCursor: nextCursor,
  };
}

export function coordinateSwarms(
  program: Program,
  programPath: string,
  state: SwarmState,
): SwarmCoordinationResult {
  // Execute one coordination tick for each swarm declaration in the program.
  const reports: SwarmCoordinationReport[] = [];
  for (const swarm of program.swarms ?? []) {
    const fleet = program.fleets.find((entry) => entry.name === swarm.fleetName);
    if (!fleet) {
      reports.push({
        swarmName: swarm.name,
        fleetName: swarm.fleetName,
        policy: swarm.policy,
        activeMember: null,
        members: [{
          robotName: "",
          missionName: null,
          missionState: "MissingFleet",
          currentStep: "",
          hasPeerLink: false,
        }],
        stepsAdvanced: 0,
        coordinationMode: "missing_fleet",
        peerDeliveries: [],
        roundRobinCursor: 0,
      });
      continue;
    }
    const cursor = state.roundRobinCursor[swarm.name] ?? 0;
    const report = coordinateSwarmGroup(program, swarm, cursor);
    state.roundRobinCursor[swarm.name] = report.roundRobinCursor;
    reports.push(report);
  }
  const success = reports.every(
    (report) =>
      report.coordinationMode !== "missing_fleet"
      && report.members.every((member) => member.missionState !== "MissingRobot"),
  );
  return { program: programPath, swarms: reports, success };
}
