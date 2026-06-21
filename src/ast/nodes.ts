/**
 * nodes module (ast/nodes.ts).
 * @module
 */

import type {
  AgentChannelDecl,
  BusDecl,
  DiscoverFilter,
  DiscoverTarget,
  MessageDecl,
  PeerRobotDecl,
  DeviceDecl,
  QosDecl,
  TopicRole,
  TransportKind,
  TwinSyncDecl,
} from "../comm/index.js";
import type {
  CapabilityDecl,
  EnumDecl,
  EventDecl,
  EventHandlerDecl,
  MatchArm,
  StateMachineDecl,
  StructDecl,
  TaskDecl,
  MissionDecl,
  TraitDecl,
  TraitImplDecl,
  TwinDecl,
  VerifyDecl,
  ObserveDecl,
} from "../foundations.js";

export type {
  AgentChannelDecl,
  BusDecl,
  DiscoverFilter,
  DiscoverTarget,
  MessageDecl,
  PeerRobotDecl,
  DeviceDecl,
  QosDecl,
  TopicRole,
  TransportKind,
  TwinSyncDecl,
} from "../comm/index.js";

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
  | "mm"
  | "cm"
  | "km"
  | "ft"
  | "in"
  | "s"
  | "ms"
  | "us"
  | "min"
  | "h"
  | "m/s"
  | "km/h"
  | "mph"
  | "m/s²"
  | "g"
  | "rad"
  | "deg"
  | "rad/s"
  | "deg/s"
  | "kg"
  | "gram"
  | "lb"
  | "N"
  | "kN"
  | "W"
  | "kW"
  | "MW"
  | "V"
  | "mV"
  | "kV"
  | "A"
  | "mA"
  | "celsius"
  | "fahrenheit"
  | "kelvin"
  | "Pa"
  | "kPa"
  | "bar"
  | "psi"
  | "mbar"
  | "Hz"
  | "kHz"
  | "MHz"
  | "rh"
  | "%RH"
  | "lux"
  | "lx"
  | "cd/m²"
  | "nit"
  | "ppm"
  | "ppb"
  | "dB"
  | "dBA"
  | "uT"
  | "gauss"
  | "rpm"
  | "N·m"
  | "Nm"
  | "J"
  | "Wh"
  | "kWh"
  | "uvi"
  | "pH"
  | "uS/cm"
  | "mS/cm"
  | "S/m"
  | "ug/m3"
  | "µg/m³"
  | "NTU"
  | "FNU"
  | "ppt"
  | "psu"
  | "uSv/h"
  | "mSv/h"
  | "%VWC"
  | "vwc";

export type SpandaType =
  | { kind: "void" }
  | { kind: "int" }
  | { kind: "float" }
  | { kind: "bool" }
  | { kind: "number"; unit: UnitKind }
  | { kind: "string" }
  | { kind: "char" }
  | { kind: "bytes" }
  | { kind: "null" }
  | { kind: "named"; name: string }
  | { kind: "generic"; name: string; typeArgs: SpandaType[] }
  | { kind: "scan" }
  | { kind: "pose" }
  | { kind: "velocity" }
  | { kind: "trajectory" }
  | { kind: "transform" }
  | { kind: "enum_variant"; enumName: string; variant: string }
  | { kind: "trait_object"; traitName: string }
  | { kind: "regex" };

export type Program = {
  kind: "Program";
  moduleName: string | null;
  imports: ImportDecl[];
  functions: import("../foundations.js").ModuleFnDecl[];
  tests: import("../foundations.js").TestDecl[];
  externFunctions: import("../foundations.js").ExternFnDecl[];
  structs: StructDecl[];
  enums: EnumDecl[];
  traits: TraitDecl[];
  hardwareProfiles: import("../foundations.js").HardwareDecl[];
  deployments: import("../foundations.js").DeployDecl[];
  requiresHardware: import("../foundations.js").RequiresHardwareDecl | null;
  requiresNetwork: import("../foundations.js").RequiresNetworkDecl | null;
  requiresConnectivity: import("../foundations.js").RequiresConnectivityDecl | null;
  geofences: import("../foundations.js").GeofenceDecl[];
  fleets: import("../foundations.js").FleetDecl[];
  swarms: import("../foundations.js").SwarmDecl[];
  programSafetyZones: import("../foundations.js").ProgramSafetyZoneDecl[];
  certifications: import("../foundations.js").CertifyDecl[];
  connectivityPolicies: import("../foundations.js").ConnectivityPolicyDecl[];
  bleServices: import("../foundations.js").BleServiceDecl[];
  simulateCompatibility: import("../foundations.js").SimulateCompatibilityDecl | null;
  messages: MessageDecl[];
  validateRules: import("../foundations.js").ValidateRuleDecl[];
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
  pipelines: import("../foundations.js").PipelineDecl[];
  watchdogs: import("../foundations.js").WatchdogDecl[];
  modes: import("../foundations.js").ModeDecl[];
  retries: import("../foundations.js").RetryDecl[];
  recovers: import("../foundations.js").RecoverDecl[];
  mission: MissionDecl | null;
  stateMachines: StateMachineDecl[];
  events: EventDecl[];
  eventHandlers: EventHandlerDecl[];
  twin: TwinDecl | null;
  verify: VerifyDecl | null;
  observe: ObserveDecl | null;
  identity: import("../foundations.js").IdentityDecl | null;
  audit: import("../foundations.js").AuditDecl | null;
  provenance: import("../foundations.js").ProvenanceDecl | null;
  signedRecords: import("../foundations.js").SignedRecordDecl[];
  secrets: import("../foundations.js").SecretDecl[];
  trust: import("../foundations.js").TrustDecl | null;
  permissions: import("../foundations.js").PermissionsDecl | null;
  requiresConnectivity: import("../foundations.js").RequiresConnectivityDecl | null;
  bluetooth: import("../foundations.js").BluetoothConfigDecl | null;
  traitImpls: TraitImplDecl[];
  buses: BusDecl[];
  peerRobots: PeerRobotDecl[];
  devices: DeviceDecl[];
  agentChannels: AgentChannelDecl[];
  twinSync: TwinSyncDecl | null;
  secureComm: import("../foundations.js").SecureCommPolicyDecl | null;
  trustBoundaries: import("../foundations.js").TrustBoundaryDecl[];
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
  topic: string | null;
  role: TopicRole;
  qos: QosDecl | null;
  transport: TransportKind | null;
  secure: import("../foundations.js").SecureBlockDecl | null;
  span: Span;
};

export type ServiceDecl = {
  kind: "ServiceDecl";
  name: string;
  serviceType: string | null;
  requestType: string | null;
  responseType: string | null;
  secure: import("../foundations.js").SecureBlockDecl | null;
  span: Span;
};

export type ActionDecl = {
  kind: "ActionDecl";
  name: string;
  actionType: string | null;
  requestType: string | null;
  feedbackType: string | null;
  resultType: string | null;
  secure: import("../foundations.js").SecureBlockDecl | null;
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
  | SpawnStmt
  | SelectStmt
  | ParallelStmt
  | PublishStmt
  | ServiceCallStmt
  | ActionSendStmt
  | EmergencyStopStmt
  | ResetEmergencyStopStmt
  | EmitStmt
  | EnterStmt
  | EnterModeStmt
  | StopAllActuatorsStmt
  | RunPipelineStmt
  | NavigateStmt
  | UseFallbackStmt
  | RememberStmt
  | SubscribeStmt
  | ExecuteStmt
  | DiscoverStmt
  | ReceiveStmt;

export type SpawnStmt = {
  kind: "SpawnStmt";
  callee: Expr;
  args: Expr[];
  span: Span;
};

export type ParallelStmt = {
  kind: "ParallelStmt";
  body: Stmt[];
  span: Span;
};

export type SelectStmt = {
  kind: "SelectStmt";
  arms: import("../foundations.js").SelectArm[];
  span: Span;
};

export type RememberStmt = {
  kind: "RememberStmt";
  key: string;
  value: Expr;
  span: Span;
};

export type SubscribeStmt = {
  kind: "SubscribeStmt";
  target: string;
  filter: import("../foundations.js").SubscribeFilterDecl | null;
  span: Span;
};

export type EnterModeStmt = {
  kind: "EnterModeStmt";
  mode: string;
  span: Span;
};

export type StopAllActuatorsStmt = {
  kind: "StopAllActuatorsStmt";
  span: Span;
};

export type RunPipelineStmt = {
  kind: "RunPipelineStmt";
  name: string;
  span: Span;
};

export type NavigateStmt = {
  kind: "NavigateStmt";
  goal: Expr;
  linear: Expr | null;
  angular: Expr | null;
  span: Span;
};

export type UseFallbackStmt = {
  kind: "UseFallbackStmt";
  resource: string;
  span: Span;
};

export type ExecuteStmt = {
  kind: "ExecuteStmt";
  actionName: string;
  goal: Expr;
  span: Span;
};

export type DiscoverStmt = {
  kind: "DiscoverStmt";
  target: DiscoverTarget;
  filter: DiscoverFilter | null;
  span: Span;
};

export type ReceiveStmt = {
  kind: "ReceiveStmt";
  topicName: string;
  varName: string;
  span: Span;
};

export type VarDecl = {
  kind: "VarDecl";
  name: string;
  typeAnnotation: SpandaType | null;
  init: Expr | null;
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
  | AwaitExpr
  | SpawnExpr
  | CallExpr
  | MemberExpr
  | MatchExpr
  | StructLiteralExpr
  | ServiceCallExpr
  | ExecuteExpr
  | DiscoverExpr;

export type SpawnExpr = {
  kind: "SpawnExpr";
  callee: Expr;
  args: Expr[];
  span: Span;
};

export type AwaitExpr = {
  kind: "AwaitExpr";
  operand: Expr;
  span: Span;
};

export type LiteralExpr = {
  kind: "LiteralExpr";
  value: number | string | boolean | null | import("../regex.js").RegexPattern;
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

export type ServiceCallExpr = {
  kind: "ServiceCallExpr";
  serviceName: string;
  span: Span;
};

export type ExecuteExpr = {
  kind: "ExecuteExpr";
  actionName: string;
  goal: Expr;
  span: Span;
};

export type DiscoverExpr = {
  kind: "DiscoverExpr";
  target: DiscoverTarget;
  filter: DiscoverFilter | null;
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
