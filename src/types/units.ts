import type { UnitKind, SpandaType } from "../ast/nodes.js";
import { allLibrarySensorTypes } from "../lib/registry.js";

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

export function unitsCompatible(a: UnitKind, b: UnitKind): boolean {
  if (a === b) return true;
  if (a === "none" || b === "none") return true;
  if ((a === "deg" && b === "rad") || (a === "rad" && b === "deg")) return true;
  return false;
}

export function resultUnitForBinary(
  op: string,
  left: SpandaType,
  right: SpandaType,
): SpandaType | null {
  if (op === "and" || op === "or") {
    if (left.kind === "bool" && right.kind === "bool") return { kind: "bool" };
    return null;
  }

  if (["<", "<=", ">", ">=", "==", "!="].includes(op)) {
    if (left.kind === "number" && right.kind === "number") {
      if (unitsCompatible(left.unit, right.unit)) return { kind: "bool" };
    }
    if (left.kind === "bool" && right.kind === "bool") return { kind: "bool" };
    if (left.kind === "string" && right.kind === "string") return { kind: "bool" };
    return null;
  }

  if (op === "+" || op === "-") {
    if (left.kind === "number" && right.kind === "number") {
      if (unitsCompatible(left.unit, right.unit)) {
        return { kind: "number", unit: left.unit !== "none" ? left.unit : right.unit };
      }
    }
    return null;
  }

  if (op === "*" || op === "/") {
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
  Camera: { kind: "named", name: "Camera" },
  AltitudeSensor: { kind: "named", name: "AltitudeSensor" },
  ForceTorque: { kind: "named", name: "ForceTorque" },
  ...Object.fromEntries(
    Object.entries(allLibrarySensorTypes()).map(([k, v]) => [k, v.roboType]),
  ),
};

export function getLibraryForSensorType(sensorType: string): string | undefined {
  return allLibrarySensorTypes()[sensorType]?.library;
}

function inferReadReturn(typeName: string): SpandaType {
  if (typeName.includes("Lidar") || typeName.includes("Velodyne") || typeName.includes("Hokuyo") || typeName.includes("Ydlidar") || typeName.includes("RealSense")) {
    return { kind: "scan" };
  }
  if (typeName.includes("BNO") || typeName.includes("LSM9") || typeName.includes("IMU")) {
    return { kind: "named", name: "IMUReading" };
  }
  if (typeName.includes("BMP") || typeName.includes("VL53") || typeName.includes("UWMF")) {
    return { kind: "number", unit: "m" };
  }
  return { kind: "void" };
}

export function mergeLibraryMethods(): void {
  for (const [typeName, info] of Object.entries(allLibrarySensorTypes())) {
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
};

export const ROBOT_METHODS: Record<string, { params: SpandaType[]; returns: SpandaType }> = {
  pose: { params: [], returns: { kind: "pose" } },
  velocity: { params: [], returns: { kind: "velocity" } },
  in_zone: { params: [{ kind: "string" }], returns: { kind: "bool" } },
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
};

export const SCAN_PROPERTIES: Record<string, SpandaType> = {
  nearest_distance: { kind: "number", unit: "m" },
};

export const OBJECT_PROPERTIES: Record<string, Record<string, SpandaType>> = {
  IMUReading: { yaw: { kind: "number", unit: "rad" }, roll: { kind: "number", unit: "rad" }, pitch: { kind: "number", unit: "rad" } },
  ForceTorqueReading: { force: { kind: "number", unit: "none" } },
  GPSReading: { lat: { kind: "number", unit: "none" }, lon: { kind: "number", unit: "none" } },
  NavigationPolicy: {
    linear: { kind: "number", unit: "m/s" },
    angular: { kind: "number", unit: "rad/s" },
  },
  ActionProposal: {
    linear: { kind: "number", unit: "m/s" },
    angular: { kind: "number", unit: "rad/s" },
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
