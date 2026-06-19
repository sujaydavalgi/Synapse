import type {
  CapabilityDecl,
  EnumDecl,
  EventDecl,
  EventHandlerDecl,
  MatchArm,
  StateMachineDecl,
  StructDecl,
  TaskDecl,
  TraitDecl,
  TraitImplDecl,
  TwinDecl,
} from "../foundations.js";

export type SourceLocation = {
  line: number;
  column: number;
  offset: number;
};

export type Span = {
  start: SourceLocation;
  end: SourceLocation;
};

export type UnitKind =
  | "none"
  | "m"
  | "s"
  | "ms"
  | "rad"
  | "m/s"
  | "m/s²"
  | "rad/s"
  | "deg"
  | "Hz";

export type SpandaType =
  | { kind: "void" }
  | { kind: "bool" }
  | { kind: "number"; unit: UnitKind }
  | { kind: "string" }
  | { kind: "named"; name: string }
  | { kind: "scan" }
  | { kind: "pose" }
  | { kind: "velocity" }
  | { kind: "trajectory" }
  | { kind: "transform" }
  | { kind: "enum_variant"; enumName: string; variant: string };

export type Program = {
  kind: "Program";
  moduleName: string | null;
  imports: ImportDecl[];
  structs: StructDecl[];
  enums: EnumDecl[];
  traits: TraitDecl[];
  robots: RobotDecl[];
  span: Span;
};

export type ImportDecl = {
  kind: "ImportDecl";
  path: string;
  span: Span;
};

export type RobotDecl = {
  kind: "RobotDecl";
  name: string;
  soc: SocDecl | null;
  hal: HalBlock | null;
  nodes: NodeDecl[];
  topics: TopicDecl[];
  services: ServiceDecl[];
  actions: ActionDecl[];
  sensors: SensorDecl[];
  actuators: ActuatorDecl[];
  safety: SafetyBlock | null;
  ai_models: AiModelDecl[];
  agents: AgentDecl[];
  behaviors: BehaviorDecl[];
  tasks: TaskDecl[];
  stateMachines: StateMachineDecl[];
  events: EventDecl[];
  eventHandlers: EventHandlerDecl[];
  twin: TwinDecl | null;
  traitImpls: TraitImplDecl[];
  span: Span;
};

export type SocDecl = {
  kind: "SocDecl";
  profile: string;
  span: Span;
};

export type HalBlock = {
  kind: "HalBlock";
  members: HalMemberDecl[];
  span: Span;
};

export type HalMemberDecl =
  | HalI2cDecl
  | HalSpiDecl
  | HalGpioDecl
  | HalPwmDecl
  | HalUartDecl
  | HalAdcDecl;

export type HalI2cDecl = {
  kind: "HalI2cDecl";
  name: string;
  address: number;
  span: Span;
};

export type HalSpiDecl = {
  kind: "HalSpiDecl";
  name: string;
  bus: number;
  csPin: number | null;
  span: Span;
};

export type HalGpioDecl = {
  kind: "HalGpioDecl";
  name: string;
  direction: "in" | "out";
  pin: number;
  span: Span;
};

export type HalPwmDecl = {
  kind: "HalPwmDecl";
  name: string;
  pin: number;
  frequencyHz: number;
  span: Span;
};

export type HalUartDecl = {
  kind: "HalUartDecl";
  name: string;
  device: string;
  baud: number;
  span: Span;
};

export type HalAdcDecl = {
  kind: "HalAdcDecl";
  name: string;
  channel: number;
  span: Span;
};

export type NodeDecl = {
  kind: "NodeDecl";
  name: string;
  namespace: string | null;
  span: Span;
};

export type TopicDecl = {
  kind: "TopicDecl";
  name: string;
  messageType: string;
  topic: string;
  span: Span;
};

export type ServiceDecl = {
  kind: "ServiceDecl";
  name: string;
  serviceType: string;
  span: Span;
};

export type ActionDecl = {
  kind: "ActionDecl";
  name: string;
  actionType: string;
  span: Span;
};

export type SensorDecl = {
  kind: "SensorDecl";
  name: string;
  sensorType: string;
  library: string | null;
  binding: SensorBinding | null;
  span: Span;
};

export type SensorBinding =
  | { kind: "topic"; path: string }
  | { kind: "hal"; busName: string };

export type ActuatorDecl = {
  kind: "ActuatorDecl";
  name: string;
  actuatorType: string;
  span: Span;
};

export type SafetyBlock = {
  kind: "SafetyBlock";
  rules: SafetyRule[];
  zones: SafetyZoneDecl[];
  span: Span;
};

export type AiConfigEntry = {
  key: string;
  value: string | number | boolean;
  span: Span;
};

export type AiModelDecl = {
  kind: "AiModelDecl";
  name: string;
  modelType: string;
  config: AiConfigEntry[];
  span: Span;
};

export type AgentDecl = {
  kind: "AgentDecl";
  name: string;
  usesAi: string[];
  memoryKind: "short_term" | "long_term" | null;
  tools: string[];
  skills: string[];
  capabilities: CapabilityDecl[];
  goal: string;
  planBody: Stmt[];
  span: Span;
};

export type SafetyRule =
  | {
      kind: "MaxSpeedRule";
      name: string;
      value: Expr;
      unit: UnitKind;
      span: Span;
    }
  | {
      kind: "StopIfRule";
      condition: Expr;
      span: Span;
    };

export type SafetyZoneDecl = {
  kind: "SafetyZoneDecl";
  name: string;
  shape: "circle" | "rect";
  x: Expr;
  y: Expr;
  radius: Expr | null;
  width: Expr | null;
  height: Expr | null;
  span: Span;
};

export type BehaviorDecl = {
  kind: "BehaviorDecl";
  name: string;
  requires: Expr | null;
  ensures: Expr | null;
  invariant: Expr | null;
  body: Stmt[];
  span: Span;
};

export type Stmt =
  | VarDecl
  | IfStmt
  | LoopStmt
  | ExprStmt
  | ReturnStmt
  | PublishStmt
  | ServiceCallStmt
  | ActionSendStmt
  | EmergencyStopStmt
  | ResetEmergencyStopStmt
  | EmitStmt
  | EnterStmt;

export type VarDecl = {
  kind: "VarDecl";
  name: string;
  init: Expr;
  span: Span;
};

export type IfStmt = {
  kind: "IfStmt";
  condition: Expr;
  thenBranch: Stmt[];
  elseBranch: Stmt[] | null;
  span: Span;
};

export type LoopStmt = {
  kind: "LoopStmt";
  intervalMs: number;
  body: Stmt[];
  span: Span;
};

export type ExprStmt = {
  kind: "ExprStmt";
  expr: Expr;
  span: Span;
};

export type ReturnStmt = {
  kind: "ReturnStmt";
  value: Expr | null;
  span: Span;
};

export type PublishStmt = {
  kind: "PublishStmt";
  topicName: string;
  value: Expr;
  span: Span;
};

export type ServiceCallStmt = {
  kind: "ServiceCallStmt";
  serviceName: string;
  span: Span;
};

export type ActionSendStmt = {
  kind: "ActionSendStmt";
  actionName: string;
  goal: Expr;
  span: Span;
};

export type EmergencyStopStmt = {
  kind: "EmergencyStopStmt";
  span: Span;
};

export type ResetEmergencyStopStmt = {
  kind: "ResetEmergencyStopStmt";
  span: Span;
};

export type EmitStmt = {
  kind: "EmitStmt";
  eventName: string;
  span: Span;
};

export type EnterStmt = {
  kind: "EnterStmt";
  stateName: string;
  span: Span;
};

export type Expr =
  | LiteralExpr
  | UnitLiteralExpr
  | IdentExpr
  | BinaryExpr
  | UnaryExpr
  | CallExpr
  | MemberExpr
  | MatchExpr
  | StructLiteralExpr;

export type LiteralExpr = {
  kind: "LiteralExpr";
  value: number | string | boolean | null;
  span: Span;
};

export type UnitLiteralExpr = {
  kind: "UnitLiteralExpr";
  value: number;
  unit: UnitKind;
  span: Span;
};

export type IdentExpr = {
  kind: "IdentExpr";
  name: string;
  span: Span;
};

export type BinaryExpr = {
  kind: "BinaryExpr";
  op: BinaryOp;
  left: Expr;
  right: Expr;
  span: Span;
};

export type UnaryExpr = {
  kind: "UnaryExpr";
  op: UnaryOp;
  operand: Expr;
  span: Span;
};

export type CallExpr = {
  kind: "CallExpr";
  callee: Expr;
  args: Expr[];
  namedArgs: NamedArg[];
  span: Span;
};

export type NamedArg = {
  name: string;
  value: Expr;
  span: Span;
};

export type MemberExpr = {
  kind: "MemberExpr";
  object: Expr;
  property: string;
  span: Span;
};

export type MatchExpr = {
  kind: "MatchExpr";
  scrutinee: Expr;
  arms: MatchArm[];
  span: Span;
};

export type StructFieldInit = {
  name: string;
  value: Expr;
  span: Span;
};

export type StructLiteralExpr = {
  kind: "StructLiteralExpr";
  typeName: string;
  fields: StructFieldInit[];
  span: Span;
};

export type BinaryOp =
  | "+"
  | "-"
  | "*"
  | "/"
  | "<"
  | "<="
  | ">"
  | ">="
  | "=="
  | "!="
  | "and"
  | "or";

export type UnaryOp = "-" | "not";

export const MESSAGE_TYPES = ["Velocity", "Pose", "Scan", "String"] as const;
export const SERVICE_TYPES = ["ResetCostmap", "ClearCostmap", "SetPose"] as const;
export const ACTION_TYPES = ["NavigateTo", "FollowPath", "PickObject"] as const;
