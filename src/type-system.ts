/**
 * type system module (type-system.ts).
 * @module
 */

import type { SpandaType, UnitKind } from "./ast/nodes.js";
import { unitCategory, type PhysicalCategory } from "./units/index.js";

export type { PhysicalCategory };

const KNOWN_DOMAIN_TYPES = new Set([
  "Mass", "Force", "Power", "Voltage", "Current", "Temperature", "Pressure",
  "Humidity", "Illuminance", "Luminance", "Concentration", "SoundLevel",
  "MagneticField", "RotationalSpeed", "Torque", "Energy",
  "UvIndex", "Ph", "Conductivity", "ParticulateMatter", "Turbidity",
  "Salinity", "Radiation", "SoilMoisture",
  "Timestamp", "Interval", "Waypoint", "MotionCommand", "ControlSignal", "PIDConfig", "GpsFix",
  "ImuData", "AudioFrame", "Prompt", "Completion", "Embedding", "Token", "Context", "Memory",
  "Plan", "ReasoningTrace", "Agent", "Goal", "Task", "Skill", "Capability", "Intent", "Command",
  "Conversation", "Speech", "Gesture", "Emotion", "Feedback", "Approval", "Risk", "Hazard",
  "Confidence", "Prediction", "Probability",
  "SafetyConstraint", "Twin", "SimulationState", "Telemetry", "Replay", "Fault", "Scenario",
  "KnowledgeGraph", "Belief", "Observation", "WorldModel", "Policy", "Reward", "StateEstimate",
  "SensorFusion", "FusedObservation",
  "LLM", "VisionModel", "EmbeddingModel", "CameraFrame", "Image", "DepthImage", "PointCloud",
  "LidarScan", "Goal",
  "Transport", "QosProfile", "QoS", "Bandwidth", "Latency", "TopicPath",
  "ServiceEndpoint", "MessageEnvelope", "DiscoveryFilter", "NetworkRequirements",
  "Reliability", "HistoryPolicy", "CommBus", "Endpoint",
]);

function genericArity(name: string): number | undefined {
  // GenericArity.
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

  // const result = genericArity(name);
  switch (name) {
    case "Array":
    case "Set":
    case "Queue":
    case "Stack":
    case "Topic":
    case "Message":
    case "Endpoint":
    case "Future":
    case "Option":
      return 1;
    case "Map":
    case "Service":
    case "Tuple":
    case "Result":
      return 2;
    case "Action":
      return 3;
    default:
      return undefined;
  }
}

export function resolveTypeName(name: string): SpandaType {
  // ResolveTypeName.
  //
  // Parameters:
  // - `name` — input value
  //
  // Returns:
  // `SpandaType`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = resolveTypeName(name);
  const short = name.replace(/^std\./, "").split(".").pop() ?? name;

  // Branch on short.
  switch (short) {
    case "Int":
    case "int":
      return { kind: "int" };
    case "Float":
    case "float":
      return { kind: "float" };
    case "Bool":
    case "bool":
      return { kind: "bool" };
    case "String":
    case "string":
      return { kind: "string" };
    case "Regex":
    case "regex":
      return { kind: "regex" };
    case "Char":
    case "char":
      return { kind: "char" };
    case "Bytes":
    case "bytes":
      return { kind: "bytes" };
    case "Null":
    case "null":
      return { kind: "null" };
    case "Void":
    case "void":
      return { kind: "void" };
    case "Time":
      return { kind: "named", name: "Time" };
    case "Duration":
      return { kind: "number", unit: "ms" };
    case "Timestamp":
      return { kind: "named", name: "Timestamp" };
    case "Interval":
      return { kind: "named", name: "Interval" };
    case "Distance":
      return { kind: "number", unit: "m" };
    case "Velocity":
      return { kind: "velocity" };
    case "Acceleration":
      return { kind: "number", unit: "m/s²" };
    case "Angle":
      return { kind: "number", unit: "rad" };
    case "AngularVelocity":
      return { kind: "number", unit: "rad/s" };
    case "Mass":
    case "Force":
    case "Power":
    case "Voltage":
    case "Current":
    case "Temperature":
    case "Pressure":
    case "Humidity":
    case "Illuminance":
    case "Luminance":
    case "Concentration":
    case "SoundLevel":
    case "MagneticField":
    case "RotationalSpeed":
    case "Torque":
    case "Energy":
    case "UvIndex":
    case "Ph":
    case "Conductivity":
    case "ParticulateMatter":
    case "Turbidity":
    case "Salinity":
    case "Radiation":
    case "SoilMoisture":
      return { kind: "named", name: short };
    case "Point2D":
    case "Point3D":
    case "Vector2D":
    case "Vector3D":
    case "Quaternion":
    case "Pose":
      return { kind: "pose" };
    case "Transform":
      return { kind: "transform" };
    case "Trajectory":
    case "Path":
      return { kind: "trajectory" };
    case "Waypoint":
    case "MotionCommand":
    case "ControlSignal":
    case "PIDConfig":
      return { kind: "named", name: short };
    case "CameraFrame":
    case "Image":
    case "DepthImage":
    case "PointCloud":
    case "LidarScan":
    case "Scan":
      return { kind: "scan" };
    case "GpsFix":
    case "ImuData":
    case "AudioFrame":
    case "LLM":
    case "VisionModel":
    case "EmbeddingModel":
    case "Prompt":
    case "Completion":
    case "Embedding":
    case "Token":
    case "Context":
    case "Memory":
    case "Plan":
    case "ReasoningTrace":
    case "Agent":
    case "Goal":
    case "Task":
    case "Skill":
    case "Capability":
    case "Intent":
      return { kind: "named", name: short };
    case "ActionProposal":
      return { kind: "named", name: "ActionProposal" };
    case "SafeAction":
      return { kind: "named", name: "SafeAction" };
    case "Command":
    case "Conversation":
    case "Speech":
    case "Gesture":
    case "Emotion":
    case "Feedback":
    case "Approval":
    case "Confidence":
    case "Prediction":
    case "Probability":
    case "Risk":
    case "Hazard":
    case "SafetyConstraint":
    case "EmergencyStop":
    case "Twin":
    case "SimulationState":
    case "Telemetry":
    case "Replay":
    case "Fault":
    case "Scenario":
    case "KnowledgeGraph":
    case "Belief":
    case "Observation":
    case "WorldModel":
    case "Policy":
    case "Reward":
    case "StateEstimate":
    case "SensorFusion":
    case "FusedObservation":
    case "Transport":
    case "QosProfile":
    case "QoS":
    case "Bandwidth":
    case "Latency":
    case "TopicPath":
    case "ServiceEndpoint":
    case "MessageEnvelope":
    case "DiscoveryFilter":
    case "NetworkRequirements":
    case "Reliability":
    case "HistoryPolicy":
    case "CommBus":
    case "Endpoint":
      return { kind: "named", name: short };
    default:

      // continue when KNOWN DOMAIN TYPES.has(short).
      if (KNOWN_DOMAIN_TYPES.has(short)) {
        return { kind: "named", name: short };
      }
      throw new Error(`Unknown type '${short}'`);
  }
}

export function resolveGenericType(name: string, args: SpandaType[]): SpandaType {
  // ResolveGenericType.
  //
  // Parameters:
  // - `name` — input value
  // - `args` — input value
  //
  // Returns:
  // `SpandaType`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = resolveGenericType(name, args);
  const base = name.split(".").pop() ?? name;
  const expected = genericArity(base);

  // continue when expected equals undefined.
  if (expected === undefined) {
    throw new Error(`Unknown generic type '${base}'`);
  }

  // continue when length differs from expected.
  if (args.length !== expected) {
    throw new Error(`Type '${base}' expects ${expected} type argument(s), got ${args.length}`);
  }
  return { kind: "generic", name: base, typeArgs: args };
}

export function physicalCategory(ty: SpandaType): PhysicalCategory {
  // PhysicalCategory.
  //
  // Parameters:
  // - `ty` — input value
  //
  // Returns:
  // `PhysicalCategory`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = physicalCategory(ty);
  switch (ty.kind) {
    case "int":
    case "float":
      return "scalar";
    case "number":
      return unitCategory(ty.unit);
    case "velocity":
      return "velocity";
    case "pose":
      return "distance";
    case "named":

      // Branch on name.
      switch (ty.name) {
        case "Distance":
          return "distance";
        case "Duration":
        case "Time":
        case "Timestamp":
        case "Interval":
          return "duration";
        case "Velocity":
          return "velocity";
        case "Acceleration":
          return "acceleration";
        case "Angle":
        case "AngularVelocity":
          return "angular_velocity";
        case "Mass":
          return "mass";
        case "Force":
          return "force";
        case "Power":
          return "power";
        case "Voltage":
          return "voltage";
        case "Current":
          return "current";
        case "Temperature":
          return "temperature";
        case "Pressure":
          return "pressure";
        case "Humidity":
          return "humidity";
        case "Illuminance":
          return "illuminance";
        case "Luminance":
          return "luminance";
        case "Concentration":
          return "concentration";
        case "SoundLevel":
          return "sound_level";
        case "MagneticField":
          return "magnetic_field";
        case "RotationalSpeed":
          return "rotational_speed";
        case "Torque":
          return "torque";
        case "Energy":
          return "energy";
        case "UvIndex":
          return "uv_index";
        case "Ph":
          return "ph";
        case "Conductivity":
          return "conductivity";
        case "ParticulateMatter":
          return "particulate_matter";
        case "Turbidity":
          return "turbidity";
        case "Salinity":
          return "salinity";
        case "Radiation":
          return "radiation";
        case "SoilMoisture":
          return "soil_moisture";
        default:
          return "scalar";
      }
    default:
      return "scalar";
  }
}

type BinaryOp = "add" | "sub" | "lt" | "lte" | "gt" | "gte" | "eq" | "neq" | "mul" | "div" | "and" | "or";

const OP_MAP: Record<string, BinaryOp> = {
  "+": "add",
  add: "add",
  "-": "sub",
  sub: "sub",
  "<": "lt",
  lt: "lt",
  "<=": "lte",
  lte: "lte",
  ">": "gt",
  gt: "gt",
  ">=": "gte",
  gte: "gte",
  "==": "eq",
  eq: "eq",
  "!=": "neq",
  neq: "neq",
  "*": "mul",
  mul: "mul",
  "/": "div",
  div: "div",
  and: "and",
  or: "or",
};

export function binaryPhysicalOpAllowed(opLexeme: string, left: SpandaType, right: SpandaType): boolean {
  // BinaryPhysicalOpAllowed.
  //
  // Parameters:
  // - `opLexeme` — input value
  // - `left` — input value
  // - `right` — input value
  //
  // Returns:
  // `true` or `false`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = binaryPhysicalOpAllowed(opLexeme, left, right);
  const op = OP_MAP[opLexeme];

  // continue when op is falsy.
  if (!op) return true;
  const catL = physicalCategory(left);
  const catR = physicalCategory(right);

  // Branch on op.
  switch (op) {
    case "add":
    case "sub":

      // continue when catL equals "scalar" && catR === "scalar".
      if (catL === "scalar" && catR === "scalar") return true;
      return catL === catR && catL !== "scalar";
    case "lt":
    case "lte":
    case "gt":
    case "gte":
    case "eq":
    case "neq":
      return catL === catR;
    case "mul":
    case "div":
      return true;
    case "and":
    case "or":
      return left.kind === "bool" && right.kind === "bool";
    default:
      return true;
  }
}

export function isActionProposalType(ty: SpandaType): boolean {
  // IsActionProposalType.
  //
  // Parameters:
  // - `ty` — input value
  //
  // Returns:
  // `true` or `false`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = isActionProposalType(ty);
  return ty.kind === "named" && ty.name === "ActionProposal";
}

export function isSafeActionType(ty: SpandaType): boolean {
  // IsSafeActionType.
  //
  // Parameters:
  // - `ty` — input value
  //
  // Returns:
  // `true` or `false`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = isSafeActionType(ty);
  return ty.kind === "named" && ty.name === "SafeAction";
}

export function typeKindName(ty: SpandaType): string {
  // TypeKindName.
  //
  // Parameters:
  // - `ty` — input value
  //
  // Returns:
  // Text result.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = typeKindName(ty);
  switch (ty.kind) {
    case "generic":
      return "generic";
    case "enum_variant":
      return "enum_variant";
    case "trait_object":
      return "trait_object";
    case "regex":
      return "regex";
    default:
      return ty.kind;
  }
}
