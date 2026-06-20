import type { Expr, Span, Stmt } from "./ast/nodes.js";

export type FieldDecl = {
  name: string;
  typeName: string;
  span: Span;
};

export type StructDecl = {
  kind: "StructDecl";
  name: string;
  fields: FieldDecl[];
  span: Span;
};

export type EnumDecl = {
  kind: "EnumDecl";
  name: string;
  variants: string[];
  span: Span;
};

export type TraitParamDecl = {
  name: string;
  typeName: string;
  span: Span;
};

export type TraitMethodDecl = {
  name: string;
  params: TraitParamDecl[];
  returnType: string;
  span: Span;
};

export type TraitDecl = {
  kind: "TraitDecl";
  name: string;
  methods: TraitMethodDecl[];
  span: Span;
};

export type TraitImplMethodDecl = {
  name: string;
  params: TraitParamDecl[];
  returnType: string;
  body: Stmt[];
  span: Span;
};

export type TraitImplDecl = {
  kind: "TraitImplDecl";
  traitName: string;
  agentName: string;
  methods: TraitImplMethodDecl[];
  span: Span;
};

export type MatchArm = {
  variant: string;
  body: Stmt[];
  span: Span;
};

export type TransitionDecl = {
  from: string;
  to: string;
  span: Span;
};

export type StateMachineDecl = {
  kind: "StateMachineDecl";
  name: string;
  states: string[];
  transitions: TransitionDecl[];
  span: Span;
};

export type TaskDecl = {
  kind: "TaskDecl";
  name: string;
  intervalMs: number;
  requires: Expr | null;
  ensures: Expr | null;
  invariant: Expr | null;
  budget: ResourceBudgetDecl | null;
  body: Stmt[];
  span: Span;
};

export type ResourceBudgetDecl = {
  kind: "ResourceBudgetDecl";
  batteryPctMax: number | null;
  memoryMbMax: number | null;
  cpuPctMax: number | null;
  networkMbpsMax: number | null;
  storageMbMax: number | null;
  span: Span;
};

export type RequiresHardwareDecl = {
  kind: "RequiresHardwareDecl";
  memoryMbMin: number | null;
  storageMbMin: number | null;
  gpuTopsMin: number | null;
  gpuRequired: boolean;
  sensors: string[];
  actuators: string[];
  span: Span;
};

export type RequiresNetworkDecl = {
  kind: "RequiresNetworkDecl";
  bandwidthMbpsMin: number | null;
  latencyMsMax: number | null;
  span: Span;
};

export type MissionDecl = {
  kind: "MissionDecl";
  durationHours: number;
  span: Span;
};

export type HardwareDecl = {
  kind: "HardwareDecl";
  name: string;
  cpu: string | null;
  memoryMb: number | null;
  storageMb: number | null;
  gpuTops: number | null;
  gpuRequired: boolean;
  sensors: string[];
  actuators: string[];
  batteryWh: number | null;
  networkBandwidthMbps: number | null;
  networkLatencyMs: number | null;
  minControlPeriodMs: number | null;
  powerDrawW: number | null;
  span: Span;
};

export type DeployDecl = {
  kind: "DeployDecl";
  robotName: string;
  targets: string[];
  span: Span;
};

export type SimulateCompatibilityDecl = {
  kind: "SimulateCompatibilityDecl";
  faults: { faultType: string; span: Span }[];
  span: Span;
};

export type EventDecl = {
  kind: "EventDecl";
  name: string;
  fields: FieldDecl[];
  span: Span;
};

export type EventHandlerDecl = {
  kind: "EventHandlerDecl";
  eventName: string;
  body: Stmt[];
  span: Span;
};

export type TwinDecl = {
  kind: "TwinDecl";
  name: string;
  mirrors: string[];
  replay: boolean;
  span: Span;
};

export type VerifyDecl = {
  kind: "VerifyDecl";
  rules: Expr[];
  span: Span;
};

export type ObserveDecl = {
  kind: "ObserveDecl";
  sensors: string[];
  span: Span;
};

export type CapabilityDecl = {
  action: string;
  target: string | null;
  span: Span;
};

/** Known code-module import paths (Phase 1 module system). */
export function resolveModuleImport(path: string): boolean {
  return [
    "sensors.lidar",
    "sensors.camera",
    "sensors.imu",
    "motion.drive",
    "motion.arm",
    "navigation.planning",
    "navigation.path_planning",
    "navigation.localize",
    "navigation.slam",
    "safety.validate",
    "ai.reasoning",
    "ai.openai",
    "robotics.ros2",
    "communication.mqtt",
    "vision.opencv",
    "vision.yolo",
    "vision.core",
    "manipulation.grasp",
    "hri.dialogue",
    "twin.sync",
    "sim.gazebo",
    "sim.webots",
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
  ].includes(path);
}

/** Map user-facing type aliases to physical units / builtin types. */
export function resolveTypeAlias(name: string): string | undefined {
  switch (name) {
    case "Distance":
    case "meter":
    case "Meter":
      return "distance";
    case "Angle":
    case "radian":
    case "Radian":
      return "angle";
    case "Path":
      return "path";
    case "Velocity":
      return "velocity";
    case "Pose":
      return "pose";
    default:
      return undefined;
  }
}
