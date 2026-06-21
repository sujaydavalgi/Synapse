/**
 * units module (types/units.ts).
 * @module
 */

import type { UnitKind, SpandaType } from "../ast/nodes.js";
import { allLibrarySensorTypes } from "../lib/registry.js";

export { unitsCompatible, unitMatchesNamedType } from "../units/index.js";
import { unitsCompatible, unitCategory, canonicalUnit, type PhysicalCategory } from "../units/index.js";
import { physicalCategory } from "../type-system.js";

export type TypeError = {
  message: string;
  line: number;
  column: number;
};

export class TypeCheckError extends Error {
  constructor(public errors: TypeError[]) {
    super(errors.map((e) => e.message).join("\n"));
    this.name = "TypeCheckError";
  }
}

function physicalTypesCompatible(left: SpandaType, right: SpandaType): boolean {
  // PhysicalTypesCompatible.
  //
  // Parameters:
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

  // const result = physicalTypesCompatible(left, right);
  const catL = physicalCategory(left);
  const catR = physicalCategory(right);
  return catL === catR && catL !== "scalar";
}

function namedTypeDefaultUnit(name: string): UnitKind | undefined {
  // NamedTypeDefaultUnit.
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

  // const result = namedTypeDefaultUnit(name);
  const map: Record<string, PhysicalCategory> = {
    Distance: "distance",
    Duration: "duration",
    Velocity: "velocity",
    Acceleration: "acceleration",
    Angle: "angle",
    AngularVelocity: "angular_velocity",
    Mass: "mass",
    Force: "force",
    Power: "power",
    Voltage: "voltage",
    Current: "current",
    Temperature: "temperature",
    Pressure: "pressure",
    Humidity: "humidity",
    Illuminance: "illuminance",
    Luminance: "luminance",
    Concentration: "concentration",
    SoundLevel: "sound_level",
    MagneticField: "magnetic_field",
    RotationalSpeed: "rotational_speed",
    Torque: "torque",
    Energy: "energy",
    UvIndex: "uv_index",
    Ph: "ph",
    Conductivity: "conductivity",
    ParticulateMatter: "particulate_matter",
    Turbidity: "turbidity",
    Salinity: "salinity",
    Radiation: "radiation",
    SoilMoisture: "soil_moisture",
  };
  const cat = map[name];
  return cat ? canonicalUnit(cat) : undefined;
}

function resultNumberForPhysical(left: SpandaType, right: SpandaType): SpandaType | null {
  // ResultNumberForPhysical.
  //
  // Parameters:
  // - `left` — input value
  // - `right` — input value
  //
  // Returns:
  // `Some` / non-null value on success, otherwise `None` / null.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = resultNumberForPhysical(left, right);
  if (left.kind === "number") return { kind: "number", unit: left.unit };

  // continue when kind equals "number".
  if (right.kind === "number") return { kind: "number", unit: right.unit };

  // continue when kind equals "named".
  if (left.kind === "named") {
    const unit = namedTypeDefaultUnit(left.name);

    // continue when unit.
    if (unit) return { kind: "number", unit };
  }

  // continue when kind equals "named".
  if (right.kind === "named") {
    const unit = namedTypeDefaultUnit(right.name);

    // continue when unit.
    if (unit) return { kind: "number", unit };
  }
  return null;
}

export function resultUnitForBinary(
  op: string,
  left: SpandaType,
  right: SpandaType,
): SpandaType | null {
  // ResultUnitForBinary.
  //
  // Parameters:
  // - `op` — input value
  // - `left` — input value
  // - `right` — input value
  //
  // Returns:
  // `Some` / non-null value on success, otherwise `None` / null.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = resultUnitForBinary(op, left, right);
  if (op === "and" || op === "or") {

    // continue when kind equals kind === "bool".
    if (left.kind === "bool" && right.kind === "bool") return { kind: "bool" };
    return null;
  }

  // check membership before continuing.
  if (["<", "<=", ">", ">=", "==", "!="].includes(op)) {

    // continue when kind equals kind === "number".
    if (left.kind === "number" && right.kind === "number") {

      // continue when unitsCompatible(left.unit, right.unit).
      if (unitsCompatible(left.unit, right.unit)) return { kind: "bool" };
    }

    // continue when kind equals kind === "bool".
    if (left.kind === "bool" && right.kind === "bool") return { kind: "bool" };

    // continue when kind equals kind === "string".
    if (left.kind === "string" && right.kind === "string") return { kind: "bool" };

    // continue when physicalTypesCompatible(left, right).
    if (physicalTypesCompatible(left, right)) return { kind: "bool" };
    return null;
  }

  // continue when op equals "+" || op === "-".
  if (op === "+" || op === "-") {

    // continue when kind equals kind === "number".
    if (left.kind === "number" && right.kind === "number") {

      // continue when unitsCompatible(left.unit, right.unit).
      if (unitsCompatible(left.unit, right.unit)) {
        return { kind: "number", unit: left.unit !== "none" ? left.unit : right.unit };
      }
    }

    // continue when physicalTypesCompatible(left, right).
    if (physicalTypesCompatible(left, right)) {
      return resultNumberForPhysical(left, right);
    }
    return null;
  }

  // continue when op equals "*" || op === "/".
  if (op === "*" || op === "/") {

    // continue when kind equals kind === "number".
    if (left.kind === "number" && right.kind === "number") {
      return { kind: "number", unit: "none" };
    }
    return null;
  }
  return null;
}

export const MESSAGE_TYPES: Record<string, SpandaType> = {
  Velocity: { kind: "velocity" },
  Pose: { kind: "pose" },
  Scan: { kind: "scan" },
  String: { kind: "string" },
};

export const SERVICE_TYPES: Record<string, SpandaType> = {
  ResetCostmap: { kind: "named", name: "ResetCostmap" },
  ClearCostmap: { kind: "named", name: "ClearCostmap" },
  SetPose: { kind: "named", name: "SetPose" },
};

export const ACTION_TYPES: Record<string, SpandaType> = {
  NavigateTo: { kind: "named", name: "NavigateTo" },
  FollowPath: { kind: "named", name: "FollowPath" },
  PickObject: { kind: "named", name: "PickObject" },
};

export const SENSOR_TYPES: Record<string, SpandaType> = {
  Lidar: { kind: "named", name: "Lidar" },
  IMU: { kind: "named", name: "IMU" },
  GPS: { kind: "named", name: "GPS" },
  GNSS: { kind: "named", name: "GNSS" },
  Camera: { kind: "named", name: "Camera" },
  AltitudeSensor: { kind: "named", name: "AltitudeSensor" },
  ForceTorque: { kind: "named", name: "ForceTorque" },
  ...Object.fromEntries(
    Object.entries(allLibrarySensorTypes()).map(([k, v]) => [k, v.roboType]),
  ),
};

export function getLibraryForSensorType(sensorType: string): string | undefined {
  // GetLibraryForSensorType.
  //
  // Parameters:
  // - `sensorType` — input value
  //
  // Returns:
  // `Some` / non-null value on success, otherwise `None` / null.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = getLibraryForSensorType(sensorType);
  return allLibrarySensorTypes()[sensorType]?.library;
}

function inferReadReturn(typeName: string): SpandaType {
  // InferReadReturn.
  //
  // Parameters:
  // - `typeName` — input value
  //
  // Returns:
  // `SpandaType`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = inferReadReturn(typeName);
  if (typeName.includes("Lidar") || typeName.includes("Velodyne") || typeName.includes("Hokuyo") || typeName.includes("Ydlidar") || typeName.includes("Ouster") || typeName.includes("RealSense")) {
    return { kind: "scan" };
  }

  // check membership before continuing.
  if (typeName.includes("BNO") || typeName.includes("LSM9") || typeName.includes("IMU")) {
    return { kind: "named", name: "IMUReading" };
  }

  // check membership before continuing.
  if (typeName.includes("BMP") || typeName.includes("VL53") || typeName.includes("UWMF")) {
    return { kind: "number", unit: "m" };
  }

  // check membership before continuing.
  if (typeName.includes("BME")) {
    return { kind: "number", unit: "rh" };
  }

  // check membership before continuing.
  if (typeName.includes("BH1750") || typeName.includes("Light")) {
    return { kind: "number", unit: "lux" };
  }

  // check membership before continuing.
  if (typeName.includes("VEML") || typeName.includes("UV") || typeName.includes("Si1145")) {
    return { kind: "number", unit: "uvi" };
  }

  // check membership before continuing.
  if (typeName.includes("pH") || typeName.endsWith("PH")) {
    return { kind: "number", unit: "pH" };
  }

  // check membership before continuing.
  if (typeName.includes("EC") || typeName.includes("Conduct")) {
    return { kind: "number", unit: "uS/cm" };
  }

  // check membership before continuing.
  if (typeName.includes("PMS") || typeName.includes("Particulate")) {
    return { kind: "number", unit: "ug/m3" };
  }

  // check membership before continuing.
  if (typeName.includes("Turbid") || typeName.includes("NTU")) {
    return { kind: "number", unit: "NTU" };
  }

  // check membership before continuing.
  if (typeName.includes("Salinity")) {
    return { kind: "number", unit: "ppt" };
  }

  // check membership before continuing.
  if (typeName.includes("Geiger") || typeName.includes("Radiation") || typeName.includes("GMC")) {
    return { kind: "number", unit: "uSv/h" };
  }

  // check membership before continuing.
  if (typeName.includes("Soil") || typeName.includes("VWC") || typeName.includes("Vegetronix")) {
    return { kind: "number", unit: "%VWC" };
  }
  return { kind: "void" };
}

export function mergeLibraryMethods(): void {
  // MergeLibraryMethods.
  //
  // Parameters:
  // None.
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = mergeLibraryMethods();
  for (const [typeName, info] of Object.entries(allLibrarySensorTypes())) {

    // continue when BUILTIN METHODS[typeName] is falsy.
    if (!BUILTIN_METHODS[typeName]) {
      BUILTIN_METHODS[typeName] = {
        read: { params: [], returns: inferReadReturn(info.roboType.name) },
        calibrate: { params: [], returns: { kind: "void" } },
      };
    }
  }
}

export const ACTUATOR_TYPES: Record<string, SpandaType> = {
  DifferentialDrive: { kind: "named", name: "DifferentialDrive" },
  RoboticArm: { kind: "named", name: "RoboticArm" },
  DroneRotors: { kind: "named", name: "DroneRotors" },
  Gripper: { kind: "named", name: "Gripper" },
};

export const AI_MODEL_TYPES: Record<string, SpandaType> = {
  LLM: { kind: "named", name: "LLM" },
  VisionModel: { kind: "named", name: "VisionModel" },
  EmbeddingModel: { kind: "named", name: "EmbeddingModel" },
};

export const AI_VALUE_TYPES: Record<string, SpandaType> = {
  ActionProposal: { kind: "named", name: "ActionProposal" },
  SafeAction: { kind: "named", name: "SafeAction" },
  Completion: { kind: "named", name: "Completion" },
  Detection: { kind: "named", name: "Detection" },
  Classification: { kind: "named", name: "Classification" },
  Plan: { kind: "named", name: "Plan" },
  Agent: { kind: "named", name: "Agent" },
  CameraFrame: { kind: "named", name: "CameraFrame" },
  Memory: { kind: "named", name: "Memory" },
  Prompt: { kind: "string" },
};

export const BUILTIN_FUNCTIONS: Record<
  string,
  { namedParams: Record<string, SpandaType>; returns: SpandaType }
> = {
  pose: {
    namedParams: {
      x: { kind: "number", unit: "m" },
      y: { kind: "number", unit: "m" },
      theta: { kind: "number", unit: "rad" },
      z: { kind: "number", unit: "m" },
    },
    returns: { kind: "pose" },
  },
  velocity: {
    namedParams: {
      linear: { kind: "number", unit: "m/s" },
      angular: { kind: "number", unit: "rad/s" },
    },
    returns: { kind: "velocity" },
  },
  trajectory: {
    namedParams: {
      from: { kind: "pose" },
      to: { kind: "pose" },
      steps: { kind: "number", unit: "none" },
    },
    returns: { kind: "trajectory" },
  },
  transform: {
    namedParams: {
      from: { kind: "string" },
      to: { kind: "string" },
      pose: { kind: "pose" },
    },
    returns: { kind: "transform" },
  },
  goal: {
    namedParams: {
      text: { kind: "string" },
    },
    returns: { kind: "named", name: "Goal" },
  },
  recall: {
    namedParams: {
      key: { kind: "string" },
    },
    returns: { kind: "named", name: "Memory" },
  },
  sha256: {
    namedParams: { data: { kind: "string" } },
    returns: { kind: "named", name: "Hash" },
  },
  sign: {
    namedParams: { data: { kind: "string" }, key: { kind: "string" } },
    returns: { kind: "named", name: "Signature" },
  },
  verify_signature: {
    namedParams: {
      data: { kind: "string" },
      signature: { kind: "string" },
      key: { kind: "string" },
    },
    returns: { kind: "bool" },
  },
  channel: {
    namedParams: {},
    returns: { kind: "named", name: "Channel" },
  },
  send: {
    namedParams: {},
    returns: { kind: "void" },
  },
  recv: {
    namedParams: {},
    returns: { kind: "void" },
  },
  join: {
    namedParams: {},
    returns: { kind: "void" },
  },
  send_agent: {
    namedParams: { to: { kind: "string" }, value: { kind: "void" } },
    returns: { kind: "void" },
  },
  recv_agent: {
    namedParams: {},
    returns: { kind: "void" },
  },
  peer_send: {
    namedParams: {
      peer: { kind: "string" },
      topic: { kind: "string" },
      value: { kind: "void" },
    },
    returns: { kind: "void" },
  },
  serialize: {
    namedParams: { format: { kind: "string" } },
    returns: { kind: "string" },
  },
  deserialize: {
    namedParams: { format: { kind: "string" } },
    returns: { kind: "void" },
  },
  assert: {
    namedParams: {},
    returns: { kind: "void" },
  },
};

export const ROBOT_METHODS: Record<string, { params: SpandaType[]; returns: SpandaType }> = {
  pose: { params: [], returns: { kind: "pose" } },
  velocity: { params: [], returns: { kind: "velocity" } },
  in_zone: { params: [{ kind: "string" }], returns: { kind: "bool" } },
  in_geofence: { params: [{ kind: "string" }], returns: { kind: "bool" } },
  connectivity_link: { params: [], returns: { kind: "string" } },
};

export const BUILTIN_METHODS: Record<
  string,
  Record<string, { params: SpandaType[]; namedParams?: Record<string, SpandaType>; returns: SpandaType }>
> = {
  Lidar: {
    read: { params: [], returns: { kind: "scan" } },
    nearest_distance: { params: [], returns: { kind: "number", unit: "m" } },
  },
  Camera: {
    read: { params: [], returns: { kind: "named", name: "CameraFrame" } },
    analyze: { params: [], returns: { kind: "named", name: "Detection" } },
    frame: { params: [], returns: { kind: "named", name: "CameraFrame" } },
  },
  IMU: {
    read: { params: [], returns: { kind: "named", name: "IMUReading" } },
  },
  AltitudeSensor: {
    read: { params: [], returns: { kind: "number", unit: "m" } },
  },
  ForceTorque: {
    read: { params: [], returns: { kind: "named", name: "ForceTorqueReading" } },
  },
  DifferentialDrive: {
    drive: {
      params: [],
      namedParams: {
        linear: { kind: "number", unit: "m/s" },
        angular: { kind: "number", unit: "rad/s" },
      },
      returns: { kind: "void" },
    },
    move: {
      params: [],
      namedParams: {
        linear: { kind: "number", unit: "m/s" },
        angular: { kind: "number", unit: "rad/s" },
      },
      returns: { kind: "void" },
    },
    execute: {
      params: [{ kind: "named", name: "SafeAction" }],
      returns: { kind: "void" },
    },
    follow: {
      params: [],
      namedParams: {
        path: { kind: "trajectory" },
      },
      returns: { kind: "void" },
    },
    stop: { params: [], returns: { kind: "void" } },
  },
  RoboticArm: {
    move_to: {
      params: [],
      namedParams: {
        x: { kind: "number", unit: "m" },
        y: { kind: "number", unit: "m" },
        z: { kind: "number", unit: "m" },
      },
      returns: { kind: "void" },
    },
    grip: { params: [], returns: { kind: "void" } },
    release: { params: [], returns: { kind: "void" } },
  },
  DroneRotors: {
    set_thrust: {
      params: [],
      namedParams: {
        thrust: { kind: "number", unit: "none" },
      },
      returns: { kind: "void" },
    },
    hover: { params: [], returns: { kind: "void" } },
  },
  Gripper: {
    close: { params: [], returns: { kind: "void" } },
    open: { params: [], returns: { kind: "void" } },
  },
  Scan: {
    nearest_distance: { params: [], returns: { kind: "number", unit: "m" } },
  },
  IMUReading: {
    yaw: { params: [], returns: { kind: "number", unit: "rad" } },
  },
  ForceTorqueReading: {
    force: { params: [], returns: { kind: "number", unit: "none" } },
  },
  LLM: {
    reason: {
      params: [],
      namedParams: {
        prompt: { kind: "string" },
        input: { kind: "scan" },
        goal: { kind: "named", name: "Goal" },
      },
      returns: { kind: "named", name: "ActionProposal" },
    },
    summarize: {
      params: [],
      namedParams: {
        input: { kind: "scan" },
      },
      returns: { kind: "named", name: "Completion" },
    },
    drive: {
      params: [],
      namedParams: {
        linear: { kind: "number", unit: "m/s" },
        angular: { kind: "number", unit: "rad/s" },
      },
      returns: { kind: "void" },
    },
  },
  VisionModel: {
    detect: {
      params: [{ kind: "named", name: "CameraFrame" }],
      returns: { kind: "named", name: "Detection" },
    },
  },
  Agent: {
    plan: { params: [], returns: { kind: "void" } },
  },
  Twin: {
    frame_count: { params: [], returns: { kind: "number", unit: "none" } },
    mirror: {
      params: [],
      namedParams: { field: { kind: "string" } },
      returns: { kind: "pose" },
    },
    replay: {
      params: [],
      namedParams: {
        index: { kind: "number", unit: "none" },
        field: { kind: "string" },
      },
      returns: { kind: "pose" },
    },
    pose: { params: [], returns: { kind: "pose" } },
    velocity: { params: [], returns: { kind: "velocity" } },
  },
  Safety: {
    validate: {
      params: [{ kind: "named", name: "ActionProposal" }],
      returns: { kind: "named", name: "SafeAction" },
    },
  },
  SensorFusion: {
    read: { params: [], returns: { kind: "named", name: "FusedObservation" } },
  },
  AuditLog: {
    record: {
      params: [{ kind: "string" }, { kind: "string" }],
      returns: { kind: "named", name: "RecordId" },
    },
    export: { params: [], returns: { kind: "string" } },
    count: { params: [], returns: { kind: "number", unit: "none" } },
    root_hash: { params: [], returns: { kind: "named", name: "Hash" } },
    create_provenance: {
      params: [{ kind: "string" }, { kind: "named", name: "RecordId" }],
      returns: { kind: "named", name: "ProvenanceRecord" },
    },
  },
  MockLedger: {
    anchor: {
      params: [{ kind: "named", name: "Hash" }],
      returns: { kind: "named", name: "TransactionId" },
    },
    verify: {
      params: [{ kind: "named", name: "Hash" }],
      returns: { kind: "bool" },
    },
  },
  RobotIdentity: {
    id: { params: [], returns: { kind: "string" } },
  },
  String: {
    matches: { params: [{ kind: "regex" }], returns: { kind: "bool" } },
    find: { params: [{ kind: "regex" }], returns: { kind: "string" } },
    replace: {
      params: [{ kind: "regex" }, { kind: "string" }],
      returns: { kind: "string" },
    },
    split: {
      params: [{ kind: "regex" }],
      returns: { kind: "named", name: "StringList" },
    },
    capture: {
      params: [{ kind: "regex" }],
      returns: { kind: "named", name: "Capture" },
    },
  },
};

export const SCAN_PROPERTIES: Record<string, SpandaType> = {
  nearest_distance: { kind: "number", unit: "m" },
};

export const OBJECT_PROPERTIES: Record<string, Record<string, SpandaType>> = {
  IMUReading: { yaw: { kind: "number", unit: "rad" }, roll: { kind: "number", unit: "rad" }, pitch: { kind: "number", unit: "rad" } },
  ForceTorqueReading: { force: { kind: "number", unit: "none" } },
  GPSReading: { lat: { kind: "number", unit: "none" }, lon: { kind: "number", unit: "none" } },
  GpsFix: {
    lat: { kind: "number", unit: "none" },
    lon: { kind: "number", unit: "none" },
    altitude: { kind: "number", unit: "m" },
    fix_quality: { kind: "number", unit: "none" },
  },
  NavigationPolicy: {
    linear: { kind: "number", unit: "m/s" },
    angular: { kind: "number", unit: "rad/s" },
  },
  ActionProposal: {
    linear: { kind: "number", unit: "m/s" },
    angular: { kind: "number", unit: "rad/s" },
    trace: { kind: "named", name: "ReasoningTrace" },
  },
  Goal: {
    text: { kind: "string" },
  },
  Agent: {
    goal: { kind: "named", name: "Goal" },
  },
  SafeAction: {
    linear: { kind: "number", unit: "m/s" },
    angular: { kind: "number", unit: "rad/s" },
  },
  Detection: {
    label: { kind: "string" },
    confidence: { kind: "number", unit: "none" },
    nearest_distance: { kind: "number", unit: "m" },
  },
  Detections: {
    count: { kind: "number", unit: "none" },
    nearest_distance: { kind: "number", unit: "m" },
    label: { kind: "string" },
  },
  Classification: {
    label: { kind: "string" },
    confidence: { kind: "number", unit: "none" },
  },
  Completion: {
    text: { kind: "string" },
  },
  FusedObservation: {
    pose: { kind: "pose" },
    count: { kind: "number", unit: "none" },
  },
};

export const POSE_PROPERTIES: Record<string, SpandaType> = {
  x: { kind: "number", unit: "m" },
  y: { kind: "number", unit: "m" },
  theta: { kind: "number", unit: "rad" },
  z: { kind: "number", unit: "m" },
};

export const VELOCITY_PROPERTIES: Record<string, SpandaType> = {
  linear: { kind: "number", unit: "m/s" },
  angular: { kind: "number", unit: "rad/s" },
};

mergeLibraryMethods();
