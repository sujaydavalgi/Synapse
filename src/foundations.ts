/**
 * foundations module (foundations.ts).
 * @module
 */

import type { Expr, Span, SpandaType, Stmt } from "./ast/nodes.js";
import type { RegexPattern } from "./regex.js";

export type Visibility = "private" | "public" | "export";

export type ModuleParamDecl = {
  name: string;
  typeAnn: SpandaType;
  span: Span;
};

export type ModuleFnDecl = {
  kind: "ModuleFnDecl";
  name: string;
  visibility: Visibility;
  typeParams: string[];
  params: ModuleParamDecl[];
  returnType: SpandaType;
  isAsync: boolean;
  body: Stmt[];
  span: Span;
};

export type BridgeKind = "native" | "python" | "cpp";

export type ExternFnDecl = {
  kind: "ExternFnDecl";
  name: string;
  library: string | null;
  bridge: BridgeKind;
  params: ModuleParamDecl[];
  returnType: SpandaType;
  span: Span;
};

export type TestDecl = {
  kind: "TestDecl";
  name: string;
  body: Stmt[];
  span: Span;
};

export type SelectArm = {
  channel: Expr;
  body: Stmt[];
  span: Span;
};

export type FieldDecl = {
  name: string;
  typeName: string;
  span: Span;
};

export type StructDecl = {
  kind: "StructDecl";
  name: string;
  typeParams?: string[];
  fields: FieldDecl[];
  span: Span;
};

export type EnumVariantDecl = {
  name: string;
  fieldTypes: string[];
  span: Span;
};

export type EnumDecl = {
  kind: "EnumDecl";
  name: string;
  variants: EnumVariantDecl[];
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
  bindings?: string[];
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

export type TaskPriority = "critical" | "high" | "normal" | "low";

export type TaskDecl = {
  kind: "TaskDecl";
  name: string;
  priority: TaskPriority;
  intervalMs: number;
  deadlineMs?: number | null;
  jitterMsMax?: number | null;
  isolated?: boolean;
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
  gpuPctMax?: number | null;
  networkMbpsMax: number | null;
  storageMbMax: number | null;
  span: Span;
};

export type PipelineDecl = {
  kind: "PipelineDecl";
  name: string;
  budgetMs: number;
  body: Stmt[];
  span: Span;
};

export type WatchdogDecl = {
  kind: "WatchdogDecl";
  name: string;
  target?: string | null;
  timeoutMs: number;
  body: Stmt[];
  span: Span;
};

export type ModeDecl = {
  kind: "ModeDecl";
  name: string;
  body: Stmt[];
  span: Span;
};

export type RetryDecl = {
  kind: "RetryDecl";
  attempts: number;
  backoffMs: number;
  body: Stmt[];
  fallback: Stmt[];
  span: Span;
};

export type RecoverDecl = {
  kind: "RecoverDecl";
  errorName: string;
  body: Stmt[];
  span: Span;
};

export type ValidateRuleDecl = {
  kind: "ValidateRuleDecl";
  name: string;
  pattern: RegexPattern;
  span: Span;
};

export type SubscribeFilterDecl = {
  field: string;
  pattern: RegexPattern;
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

export type ConnectivityRequirement = "required" | "optional";

export type RequiresConnectivityDecl = {
  kind: "RequiresConnectivityDecl";
  channels: Array<[string, ConnectivityRequirement]>;
  latencyMsMax: number | null;
  bandwidthMbpsMin: number | null;
  packetLossPctMax: number | null;
  span: Span;
};

export type GeofenceDecl = {
  kind: "GeofenceDecl";
  name: string;
  centerLat: number;
  centerLon: number;
  radiusM: number;
  span: Span;
};

export type ConnectivityPolicyDecl = {
  kind: "ConnectivityPolicyDecl";
  name: string;
  preferred: string;
  fallback: string;
  emergency: string | null;
  switchIfLatencyMs: number | null;
  switchIfPacketLossPct: number | null;
  span: Span;
};

export type BluetoothConfigDecl = {
  kind: "BluetoothConfigDecl";
  scanPattern: RegexPattern | null;
  pairMode: string | null;
  span: Span;
};

export type BleServiceDecl = {
  kind: "BleServiceDecl";
  name: string;
  uuid: string;
  span: Span;
};

export type MissionDecl = {
  kind: "MissionDecl";
  name: string | null;
  durationHours: number | null;
  steps: string[];
  span: Span;
};

export type FleetDecl = {
  kind: "FleetDecl";
  name: string;
  members: string[];
  span: Span;
};

export type SwarmPolicy = "round_robin" | "broadcast" | "leader_follow";

export type SwarmDecl = {
  kind: "SwarmDecl";
  name: string;
  fleetName: string;
  policy: SwarmPolicy;
  span: Span;
};

export const SWARM_POLICIES: SwarmPolicy[] = ["round_robin", "broadcast", "leader_follow"];

export function parseSwarmPolicy(name: string): SwarmPolicy | undefined {
  return SWARM_POLICIES.includes(name as SwarmPolicy) ? (name as SwarmPolicy) : undefined;
}

export function validateSwarmFleet(
  swarmName: string,
  fleetName: string,
  fleetNames: string[],
): string | undefined {
  if (fleetNames.includes(fleetName)) return undefined;
  return `swarm '${swarmName}' references unknown fleet '${fleetName}'`;
}

export type ProgramSafetyZoneDecl = {
  kind: "ProgramSafetyZoneDecl";
  name: string;
  maxSpeedMps: number | null;
  span: Span;
};

export type CertificationStandard = "ISO13849" | "IEC61508" | "ISO26262";

export type CertifyDecl = {
  kind: "CertifyDecl";
  standard: CertificationStandard;
  level: string | null;
  span: Span;
};

export const CERTIFICATION_STANDARDS: CertificationStandard[] = [
  "ISO13849",
  "IEC61508",
  "ISO26262",
];

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
  connectivity: string[];
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
  faults: { faultType: string; atOffsetMs?: number; durationMs?: number; span: Span }[];
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

export type IdentityDecl = {
  kind: "IdentityDecl";
  typeName: string;
  fields: [string, string][];
  span: Span;
};

export type AuditDecl = {
  kind: "AuditDecl";
  name: string;
  records: Expr[];
  span: Span;
};

export type ProvenanceDecl = {
  kind: "ProvenanceDecl";
  name: string;
  hashAlgo: string;
  signedBy: string;
  span: Span;
};

export type SignedRecordDecl = {
  kind: "SignedRecordDecl";
  eventName: string;
  signedBy: string;
  span: Span;
};

export type SecretSourceDecl =
  | { source: "env"; var: string }
  | { source: "literal"; value: string }
  | { source: "file"; path: string };

export type SecureCommPolicyDecl = {
  kind: "SecureCommPolicyDecl";
  encryption: string | null;
  authentication: string | null;
  integrity: string | null;
  span: Span;
};

export type TrustBoundaryDecl = {
  kind: "TrustBoundaryDecl";
  name: string;
  span: Span;
};

export type SecretDecl = {
  kind: "SecretDecl";
  name: string;
  source: SecretSourceDecl;
  span: Span;
};

export type TrustDecl = {
  kind: "TrustDecl";
  level: string;
  span: Span;
};

export type PermissionsDecl = {
  kind: "PermissionsDecl";
  capabilities: string[];
  span: Span;
};

export type SecureBlockDecl = {
  signed: boolean;
  minTrust: string | null;
  requires: string[];
  encryption: string | null;
  authentication: string | null;
  integrity: string | null;
  trustedSources: string[];
  rejectUntrusted: boolean;
  span: Span;
};

export function resolveModuleImport(path: string): boolean {
  // ResolveModuleImport.
  //
  // Parameters:
  // - `path` — input value
  //
  // Returns:
  // `true` or `false`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = resolveModuleImport(path);
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
    "navigation.nav2",
    "navigation.cartographer",
    "navigation.rtabmap",
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
    "ledger.mock",
    "provenance.core",
    "identity.core",
    "supply_chain.trace",
    "std.core",
    "std.time",
    "std.units",
    "std.spatial",
    "std.math",
    "std.collections",
    "std.result",
    "std.io",
    "std.log",
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
    "std.security",
    "std.audit",
    "std.crypto",
    "std.network",
  ].includes(path);
}

export function resolveTypeAlias(name: string): string | undefined {
  // ResolveTypeAlias.
  //
  // Parameters:
  // - `name` — input value
  //
  // Returns:
  // `Some` / non-null value on success, otherwise `None` / null.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = resolveTypeAlias(name);
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
