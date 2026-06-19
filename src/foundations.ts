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
  body: Stmt[];
  span: Span;
};

export type EventDecl = {
  kind: "EventDecl";
  name: string;
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
    "navigation.localize",
    "safety.validate",
    "ai.reasoning",
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
