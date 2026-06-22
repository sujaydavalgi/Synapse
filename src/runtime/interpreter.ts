/**
 * interpreter module (runtime/interpreter.ts).
 * @module
 */

import type {
  Expr,
  Program,
  RobotDecl,
  SafetyRule,
  SafetyZoneDecl,
  Stmt,
  UnitKind,
} from "../ast/nodes.js";
import { alignForBinary } from "../units/index.js";
import { createSimHal, halMemberFromDecl, type HalBackend } from "../hal/index.js";
import { getSensorDriver, readWithDriver } from "../lib/registry.js";
import {
  createAIModel,
  safeActionFromProposal,
  isActionProposal,
  isSafeAction,
  proposalFromValue,
  type AIModel,
} from "../ai/index.js";
import { createAgentRuntime, executeAgentPlan, type AgentRuntime } from "../ai/Agent.js";
import { MemoryStore } from "../ai/MemoryStore.js";
import { mockAnalyzeFrame, mockCameraFrame } from "../ai/MockAIProvider.js";
import { getSocProfile } from "../soc/index.js";
import { RoutingCommBus, type TransportKind, TlsTransportSession, effectiveTransportPolicy, transportSecurityFromBusFields, resolveBrokerUrl, type SecureCommPolicy } from "../transport/index.js";
import {
  bootstrapProvidersForPackages,
  syncCommBusForOfficialPackages,
  type ProviderRegistry,
} from "../providers/index.js";
import { parseTrustBoundary, boundaryForTransportName } from "../security/trust-boundary.js";
import { SafetyMonitor, createSafetyConfigFromRobot, interpolatePoses } from "../safety/index.js";
import type { SafetyZoneRuntime } from "../safety/index.js";
import {
  SecurityContext,
  createRobotIdentity,
  parseTrustLevel,
  type SecurePolicy,
} from "../security/index.js";
import {
  getNumber,
  getPoseFields,
  getString,
  getTrajectoryWaypoints,
  getVelocityFields,
  poseFromState,
  runtimePose,
  runtimeTrajectory,
  runtimeVelocity,
  velocityFromState,
} from "./values.js";
import { callExternBridge } from "../ffi/subprocess-bridge.js";
import { ConcurrencyRuntime } from "../concurrency.js";
import type { ModuleRegistry } from "../modules/index.js";
import type { ExternFnDecl, ModuleFnDecl, ResourceBudgetDecl, TaskDecl, TaskPriority } from "../foundations.js";
import type { CaptureResult, RegexPattern } from "../regex.js";
import {
  applyGpsPositionFaults,
  connectivityLinkToTransport,
  connectivityPolicyFromDecl,
  faultToConnectivity,
  geofenceContains,
  geofenceFromDecl,
  isModemBearer,
  isLinkImpaired,
  runtimeSimIdentity,
  type ConnectivityPolicyRuntime,
  type GeofenceRuntime,
} from "../connectivity-positioning.js";
import {
  regexCapture,
  regexFind,
  regexMatches,
  regexReplace,
  regexSplit,
} from "../regex.js";
import { ReliabilityRuntime } from "./reliability-runtime.js";
import {
  createMissionRuntime,
  FleetRegistry,
  missionAdvance,
  missionComplete,
  missionCurrentStep,
  missionFail,
  missionPause,
  missionResume,
  missionStart,
  ProgramSafetyZoneRegistry,
  type MissionRuntime,
} from "../robotics-platform.js";
import { tryPublishNav2CmdVel } from "../navigation/index.js";
import { invokeNav2Bridge, invokeSlamBridge } from "../adapter-bridge.js";

export type PoseValue = { x: number; y: number; theta: number; z: number };

export type RuntimeValue =
  | { kind: "number"; value: number; unit: UnitKind }
  | { kind: "bool"; value: boolean }
  | { kind: "string"; value: string }
  | { kind: "void" }
  | { kind: "scan"; nearestDistance: number }
  | { kind: "pose"; x: number; y: number; theta: number; z: number }
  | { kind: "velocity"; linear: number; angular: number }
  | { kind: "trajectory"; waypoints: PoseValue[] }
  | { kind: "transform"; fromFrame: string; toFrame: string; pose: PoseValue }
  | { kind: "object"; typeName: string; fields: Record<string, RuntimeValue> }
  | { kind: "enum"; enumName: string; variant: string; payloads: RuntimeValue[] }
  | { kind: "sensor"; name: string; sensorType: string; library?: string | null; halBinding?: string | null; topic?: string | null }
  | { kind: "actuator"; name: string; actuatorType: string }
  | { kind: "topic"; name: string; messageType: string; topicPath: string }
  | { kind: "service"; name: string; serviceType: string }
  | { kind: "action"; name: string; actionType: string }
  | { kind: "robot" }
  | { kind: "agent"; name: string }
  | { kind: "trait_object"; traitName: string; agent: string }
  | { kind: "twin"; name: string }
  | { kind: "safety_ctx" }
  | { kind: "ai_model"; name: string; modelType: string; provider: string }
  | { kind: "action_proposal"; linear: number; angular: number; source: string; trace: string[]; trusted: false }
  | { kind: "safe_action"; linear: number; angular: number; trusted: true }
  | { kind: "goal"; text: string }
  | { kind: "sensor_fusion"; sensors: string[] }
  | { kind: "mission_control"; runtime: MissionRuntime }
  | { kind: "navigation_control"; goal: string | null }
  | { kind: "slam_control" }
  | { kind: "fleet_control"; registry: FleetRegistry }
  | { kind: "completion"; text: string; model?: string }
  | { kind: "embedding"; dimensions: number; vector: number[] }
  | { kind: "identity"; id: string; publicKey: string }
  | { kind: "secret"; name: string }
  | { kind: "channel"; id: number }
  | { kind: "task_handle"; id: number }
  | { kind: "future"; funcName: string; args: RuntimeValue[]; resolved: RuntimeValue | null }
  | { kind: "regex"; pattern: RegexPattern }
  | { kind: "capture"; result: CaptureResult };

export type MotionCommand =
  | { kind: "drive"; linear: number; angular: number; actuator: string }
  | { kind: "stop"; actuator: string }
  | { kind: "move_to"; x: number; y: number; z: number; actuator: string }
  | { kind: "follow"; waypoints: PoseValue[]; actuator: string }
  | { kind: "grip"; actuator: string }
  | { kind: "release"; actuator: string }
  | { kind: "open"; actuator: string }
  | { kind: "set_thrust"; thrust: number; actuator: string }
  | { kind: "hover"; actuator: string };

export interface RobotBackend {
  readSensor(sensorName: string, sensorType: string, topic?: string | null): RuntimeValue;
  executeMotion(cmd: MotionCommand): void;
  tick(dtMs: number): void;
  getState(): RobotState;
  setEmergencyStop?(active: boolean): void;
  publishTopic?(topicPath: string, messageType: string, value: RuntimeValue): void;
  callService?(serviceName: string, serviceType: string): RuntimeValue;
  sendAction?(actionName: string, actionType: string, goal: RuntimeValue): RuntimeValue;
  getPublishedTopics?(): Array<{ topic: string; messageType: string; value: RuntimeValue }>;
  getHal?(): HalBackend | null;
}

export type RobotState = {
  pose: { x: number; y: number; theta: number; z?: number };
  velocity: { linear: number; angular: number };
  emergencyStop: boolean;
};

export type InterpreterOptions = {
  backend: RobotBackend;
  maxLoopIterations?: number;
  onMotionBlocked?: (reason: string) => void;
  onLog?: (message: string) => void;
  moduleRegistry?: ModuleRegistry;
  recordTrace?: boolean;
  traceSource?: string;
  schedulerClock?: "sim" | "wall";
  secure?: boolean;
  injectSecurityFaults?: boolean;
  /** Official package dependency names from the enclosing project manifest. */
  officialPackages?: readonly string[];
  /** Optional domain provider registry; defaults to package-scoped bootstrap when unset. */
  providerRegistry?: ProviderRegistry;
};

export class Environment {
  private bindings = new Map<string, RuntimeValue>();

  define(name: string, value: RuntimeValue): void {
    // Define.
    //
    // Parameters:
    // - `name` — input value
    // - `value` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = define(name, value);

    this.bindings.set(name, value);
  }

  get(name: string): RuntimeValue | undefined {
    // Get.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = get(name);

    return this.bindings.get(name);
  }

  set(name: string, value: RuntimeValue): void {
    // Set.
    //
    // Parameters:
    // - `name` — input value
    // - `value` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = set(name, value);

    this.bindings.set(name, value);
  }

  remove(name: string): void {
    // Remove.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = remove(name);

    this.bindings.delete(name);
  }

  clone(): Environment {
    // Clone.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Environment.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = clone();

    const env = new Environment();
    for (const [k, v] of this.bindings) env.define(k, v);
    return env;
  }
}

export class Interpreter {
  private env = new Environment();
  private safetyMonitor: SafetyMonitor | null = null;
  private zones: import("../safety/index.js").SafetyZoneRuntime[] = [];
  private hal: HalBackend | null = null;
  private ai_models = new Map<string, AIModel>();
  private agents = new Map<string, AgentRuntime>();
  private agentCapabilities = new Map<string, import("../foundations.js").CapabilityDecl[]>();
  private currentAgent: string | null = null;
  private eventHandlers = new Map<string, Stmt[]>();
  private stateMachines = new Map<
    string,
    { current: string; states: string[]; transitions: Array<{ from: string; to: string }> }
  >();
  private twinRuntime: {
    name: string;
    mirrors: string[];
    replay: boolean;
    shadow: Record<string, RuntimeValue>;
    replayBuffer: Array<Record<string, RuntimeValue>>;
  } | null = null;
  private currentRobot: RobotDecl | null = null;
  private currentProgram: Program | null = null;
  private enumVariants = new Map<string, string[]>();
  private variantOwner = new Map<string, string>();
  private structDefs = new Map<string, Array<{ name: string; typeName: string }>>();
  private agentTraitImpls = new Map<
    string,
    Map<string, { params: import("../foundations.js").TraitParamDecl[]; body: Stmt[] }>
  >();
  private verifyRules: Expr[] = [];
  private fusionSensors: string[] = [];
  private slamEnabled = false;
  private security = new SecurityContext();
  private commBus = new RoutingCommBus();
  private defaultTransport: TransportKind = "local";
  private topicPathToMessageType = new Map<string, string>();
  private moduleFunctions = new Map<string, ModuleFnDecl>();
  private importedFunctions = new Map<string, ModuleFnDecl>();
  private externFunctions = new Map<string, ExternFnDecl>();
  private concurrency = new ConcurrencyRuntime();
  private taskMaxDurationMs = new Map<string, number>();
  private returning = false;
  private returnValue: RuntimeValue = { kind: "void" };
  private reliability = new ReliabilityRuntime();
  private geofences: GeofenceRuntime[] = [];
  private geofenceActive = new Set<string>();
  private connectivityPolicies: ConnectivityPolicyRuntime[] = [];
  private activeConnectivityLink = "wifi";
  private connectivityEventsSeen = new Set<string>();
  private injectedFaults = new Set<string>();
  private gpsAvailable = true;
  private fleets = new FleetRegistry();
  private programSafetyZones = new ProgramSafetyZoneRegistry();
  private providerRegistry: ProviderRegistry;

  constructor(private options: InterpreterOptions) {
    this.providerRegistry =
      options.providerRegistry ??
      bootstrapProvidersForPackages(options.officialPackages ?? []);
  }

  run(program: Program, entryBehavior?: string): RobotState {
    // Run the operation.
    //
    // Parameters:
    // - `program` — input value
    // - `entryBehavior?` — optional input
    //
    // Returns:
    // RobotState.
    //
    // Options:
    // - `entryBehavior?` — optional parameter
    //
    // Example:

    // const result = run(program, entryBehavior?);

    this.currentProgram = program;
    this.loadProgramMetadata(program);
    if (this.providerRegistry.officialPackages().length > 0) {
      this.commBus.attachProviderRegistry(this.providerRegistry);
      syncCommBusForOfficialPackages(this.commBus, this.providerRegistry);
      this.options.onLog?.(
        `providers: ${this.providerRegistry.officialPackages().length} official package(s) active`,
      );
    }
    if (this.options.secure) {
      this.security.enableStrictPermissions();
    }
    if (this.options.injectSecurityFaults) {
      for (const fault of ["InvalidSignature", "ExpiredCertificate", "ReplayAttack"]) {
        this.commBus.injectFault(fault);
        this.security.securityFaultsActive.add(fault);
      }
    }
    for (const robot of program.robots) {
      this.setupRobot(robot);
      if (!entryBehavior && robot.behaviors.length === 0 && robot.tasks.length > 1) {
        this.executeMultiplexedTasks(robot.tasks);
        continue;
      }
      const behaviorName =
        entryBehavior ?? robot.behaviors[0]?.name ?? robot.tasks[0]?.name;
      if (!behaviorName) continue;

      const behavior = robot.behaviors.find((b) => b.name === behaviorName);
      if (behavior) {
        this.executeWithContracts(behavior.body, behavior.requires, behavior.ensures, behavior.invariant);
        this.processSpawnQueue();
        continue;
      }

      const task = robot.tasks.find((t) => t.name === behaviorName);
      if (task) {
        this.executeTaskLoop(task);
      }
    }
    this.processSpawnQueue();
    return this.options.backend.getState();
  }

  runTests(program: Program): void {
    // RunTests.
    //
    // Parameters:
    // - `program` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = runTests(program);

    this.currentProgram = program;
    this.loadProgramMetadata(program);
    for (const test of program.tests) {
      this.options.onLog?.(`test ${test.name}`);
      this.returning = false;
      this.executeBlock(test.body);
      this.processSpawnQueue();
    }
  }

  private loadProgramMetadata(program: Program): void {
    // LoadProgramMetadata.
    //
    // Parameters:
    // - `program` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = loadProgramMetadata(program);
    this.enumVariants.clear();
    this.variantOwner.clear();
    this.structDefs.clear();
    this.moduleFunctions.clear();
    this.importedFunctions.clear();
    this.externFunctions.clear();

    // Process each function.
    for (const func of program.functions) {

      // continue when visibility equals visibility === "public".
      if (func.visibility === "export" || func.visibility === "public") {
        this.moduleFunctions.set(func.name, func);
      }
    }

    // Process each externFunction.
    for (const ext of program.externFunctions) {
      this.externFunctions.set(ext.name, ext);
    }

    // continue when this.options.moduleRegistry.
    if (this.options.moduleRegistry) {

      // Process each import declaration.
      for (const imp of program.imports) {
        const exports = this.options.moduleRegistry.exportsFor(imp.path);

        // continue when exports.
        if (exports) {

          // Iterate over the collection.
          for (const [name, func] of exports.functions) {
            this.importedFunctions.set(name, func);
          }
        }
      }
    }

    // Process each enum.
    for (const enumDecl of program.enums) {
      const variantNames = enumDecl.variants.map((v) => v.name);
      this.enumVariants.set(enumDecl.name, variantNames);

      // Process each variant.
      for (const variant of enumDecl.variants) {
        this.variantOwner.set(variant.name, enumDecl.name);
      }
    }

    // Process each struct.
    for (const structDecl of program.structs) {
      this.structDefs.set(
        structDecl.name,
        structDecl.fields.map((f) => ({ name: f.name, typeName: f.typeName })),
      );
    }
    this.loadConnectivityMetadata(program);
    this.loadRoboticsPlatformMetadata(program);
  }

  private loadRoboticsPlatformMetadata(program: Program): void {
    // Load fleet groupings and program-level safety zone policies.
    //
    // Parameters:
    // - `program` — parsed program
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // loadRoboticsPlatformMetadata(program);

    this.fleets = new FleetRegistry();
    this.programSafetyZones = new ProgramSafetyZoneRegistry();

    for (const fleet of program.fleets) {
      this.fleets.register(fleet.name, [...fleet.members]);
      this.options.onLog?.(`fleet '${fleet.name}': ${fleet.members.length} member(s)`);
    }
    this.env.define("fleet", { kind: "fleet_control", registry: this.fleets });

    for (const zone of program.programSafetyZones) {
      if (zone.maxSpeedMps !== null) {
        this.programSafetyZones.register(zone.name, zone.maxSpeedMps);
        this.options.onLog?.(
          `safety_zone '${zone.name}': max_speed ${zone.maxSpeedMps.toFixed(2)} m/s`,
        );
      }
    }

    for (const cert of program.certifications ?? []) {
      const levelSuffix = cert.level ? ` level ${cert.level}` : "";
      this.options.onLog?.(
        `certify ${cert.standard}${levelSuffix}: metadata recorded (verify-only)`,
      );
    }

    const slamPaths = ["navigation.slam", "navigation.cartographer", "navigation.rtabmap"];
    this.slamEnabled = program.imports.some((imp) => slamPaths.includes(imp.path));
  }

  private loadConnectivityMetadata(program: Program): void {
    // Load geofence zones and connectivity policies from the program AST.
    //
    // Parameters:
    // - `program` — parsed program
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // loadConnectivityMetadata(program);

    this.geofences = program.geofences.map(geofenceFromDecl);
    this.connectivityPolicies = program.connectivityPolicies.map(connectivityPolicyFromDecl);
    this.geofenceActive.clear();
    this.connectivityEventsSeen.clear();
    this.injectedFaults.clear();
    this.gpsAvailable = true;
    const policy = this.connectivityPolicies[0];
    if (policy) {
      this.activeConnectivityLink = policy.preferred || "wifi";
      this.defaultTransport = connectivityLinkToTransport(this.activeConnectivityLink);
    } else {
      this.activeConnectivityLink = "wifi";
    }

    if (program.simulateCompatibility) {
      for (const fault of program.simulateCompatibility.faults) {
        this.injectedFaults.add(fault.faultType);
        this.commBus.injectFault(fault.faultType);
      }
    }
  }

  private currentGpsLatLon(): [number, number] {
    // Read simulated GPS coordinates from robot pose (x=lat, y=lon).
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Tuple of latitude and longitude in degrees.
    //
    // Options:
    // None.
    //
    // Example:
    // const [lat, lon] = currentGpsLatLon();

    const state = this.options.backend.getState();
    const { lat, lon } = applyGpsPositionFaults(
      this.injectedFaults,
      state.pose.x,
      state.pose.y,
      this.reliability.simTimeMs,
    );
    return [lat, lon];
  }

  private runGeofenceTriggers(): void {
    // Dispatch geofence entered/exited handlers when pose crosses a fence boundary.
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
    // runGeofenceTriggers();

    if (this.geofences.length === 0) {
      return;
    }
    const [lat, lon] = this.currentGpsLatLon();
    const entered: string[] = [];
    const exited: string[] = [];

    for (const fence of this.geofences) {
      const inside = geofenceContains(fence, lat, lon);
      const wasInside = this.geofenceActive.has(fence.name);
      if (inside && !wasInside) {
        this.geofenceActive.add(fence.name);
        entered.push(fence.name);
      } else if (!inside && wasInside) {
        this.geofenceActive.delete(fence.name);
        exited.push(fence.name);
      } else if (inside) {
        this.geofenceActive.add(fence.name);
      }
    }

    for (const name of entered) {
      this.dispatchEvent(`geofence:${name}:entered`);
    }
    for (const name of exited) {
      this.dispatchEvent(`geofence:${name}:exited`);
    }
  }

  private runConnectivityMaintenance(): void {
    // Run periodic geofence and failover maintenance during simulation ticks.
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
    // runConnectivityMaintenance();

    this.runGeofenceTriggers();
    this.runConnectivityTriggers();
    this.pollTransportInbound();
  }

  private pollTransportInbound(): void {
    // Poll external transport adapters and verify inbound wire frames.
    const inbound = this.commBus.pollInbound(this.defaultTransport);
    for (const [topicPath, envelope] of inbound) {
      const payload =
        typeof envelope.value === "object" && envelope.value !== null
          ? JSON.stringify(envelope.value)
          : String(envelope.value);
      try {
        this.security.verifyInboundMessage(
          topicPath,
          payload,
          envelope.sourceId,
          this.topicPathToMessageType.get(topicPath) ?? "Unknown",
        );
      } catch (e) {
        this.options.onLog?.(`security: inbound denied on ${topicPath}: ${e}`);
        continue;
      }
      const topicName =
        topicPath.replace(/^\//, "").replace(/\//g, ".") || topicPath;
      this.dispatchMessageTriggers(topicName, topicPath);
    }
  }

  private dispatchMessageTriggers(topicName: string, topicPath: string): void {
    // Dispatch message triggers for an inbound topic path when handlers exist.
    for (const [eventName, handler] of this.eventHandlers) {
      if (eventName === `message:${topicName}` || eventName === `message:${topicPath}`) {
        this.executeBlock(handler);
      }
    }
  }

  private runConnectivityTriggers(): void {
    // Dispatch connectivity triggers from comm faults and GPS availability.
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
    // runConnectivityTriggers();

    for (const fault of this.commBus.activeFaults()) {
      const mapped = faultToConnectivity(fault);
      if (!mapped) continue;
      const key = `fault:${mapped.domain}.${mapped.event}`;
      if (this.connectivityEventsSeen.has(key)) continue;
      this.connectivityEventsSeen.add(key);
      this.applyConnectivityFailover(mapped.domain, mapped.event);
      this.dispatchEvent(`${mapped.domain}.${mapped.event}`);
    }

    const gpsOk = ![...this.injectedFaults].some((f) => f === "GpsFailure" || f === "GPSLost");
    if (this.gpsAvailable && !gpsOk) {
      this.gpsAvailable = false;
      this.applyConnectivityFailover("gps", "lost");
      this.dispatchEvent("gps.lost");
    } else if (!this.gpsAvailable && gpsOk) {
      this.gpsAvailable = true;
      this.dispatchEvent("gps.acquired");
      this.dispatchEvent("gps.fix");
    }
  }

  private activeConnectivityFaults(): Set<string> {
    const faults = new Set(this.injectedFaults);
    for (const fault of this.commBus.activeFaults()) {
      faults.add(fault);
    }
    return faults;
  }

  private activateConnectivityLink(policyName: string, link: string, reason: string): void {
    this.activeConnectivityLink = link;
    this.defaultTransport = connectivityLinkToTransport(link);
    this.commBus.reconnectTransport(this.defaultTransport);
    this.options.onLog?.(
      `connectivity_policy '${policyName}': ${reason} (transport ${this.defaultTransport})`,
    );
  }

  private applyConnectivityFailover(domain: string, event: string): void {
    // Update active link and default transport when a policy failover applies.
    //
    // Parameters:
    // - `domain` — connectivity domain (network, gps, …)
    // - `event` — trigger event name
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // applyConnectivityFailover("network", "disconnected");

    if (domain !== "network" || event !== "disconnected") return;

    const faults = this.activeConnectivityFaults();
    for (const policy of this.connectivityPolicies) {
      const { preferred, fallback, emergency } = policy;

      if (this.activeConnectivityLink === preferred) {
        this.activateConnectivityLink(
          policy.name,
          fallback,
          `failover ${preferred} -> ${fallback}`,
        );
      }

      if (
        emergency &&
        this.activeConnectivityLink !== emergency &&
        isLinkImpaired(this.activeConnectivityLink, faults)
      ) {
        this.activateConnectivityLink(policy.name, emergency, `emergency link ${emergency}`);
      }
    }
  }

  private evalContract(expr: Expr): boolean {
    // EvalContract.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = evalContract(expr);
    const val = this.evalExpr(expr);

    // continue when kind equals "bool".
    if (val.kind === "bool") return val.value;
    throw new RuntimeError("Contract expression must be boolean", 0);
}

  private executeWithContracts(
    body: Stmt[],
    requires: Expr | null,
    ensures: Expr | null,
    invariant: Expr | null,
  ): void {
    // ExecuteWithContracts.
    //
    // Parameters:
    // - `body` — input value
    // - `requires` — input value
    // - `ensures` — input value
    // - `invariant` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = executeWithContracts(body, requires, ensures, invariant);
    if (requires && !this.evalContract(requires)) {
      throw new RuntimeError("requires contract failed", 0);
    }
    this.executeBlock(body);

    // continue when ensures && !this.evalContract(ensures).
    if (ensures && !this.evalContract(ensures)) {
      throw new RuntimeError("ensures contract failed", 0);
    }

    // continue when invariant && !this.evalContract(invariant).
    if (invariant && !this.evalContract(invariant)) {
      throw new RuntimeError("invariant contract failed", 0);
    }
    this.runVerifyRules();
}

  private runVerifyRules(): void {
    // RunVerifyRules.
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

    // const result = runVerifyRules();
    if (this.verifyRules.length === 0) return;

    // Loop with index variable i.
    for (let i = 0; i < this.verifyRules.length; i++) {
      const val = this.evalExpr(this.verifyRules[i]!);

      // continue when kind differs from "bool".
      if (val.kind !== "bool") {
        throw new RuntimeError(`verify rule ${i + 1} must be boolean`, 0);
      }

      // continue when value is falsy.
      if (!val.value) {
        throw new RuntimeError(`verify rule ${i + 1} failed`, 0);
      }
    }
    this.options.onLog?.(`verify: all ${this.verifyRules.length} rule(s) passed`);
}

  private executeTaskLoop(task: TaskDecl): void {
    // ExecuteTaskLoop.
    //
    // Parameters:
    // - `task` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = executeTaskLoop(task);
    const { body, intervalMs, requires, ensures, invariant, name, priority, budget } = task;
    const maxIter = this.options.maxLoopIterations ?? 10;
    this.options.onLog?.(
      `single-task ${name} interval=${intervalMs}ms priority=${priority}`,
    );

    // Loop with index variable i.
    for (let i = 0; i < maxIter; i++) {
      this.options.backend.tick(intervalMs);

      // continue when runScheduledTask is falsy.
      if (!this.runScheduledTask(name, priority, intervalMs, body, requires, ensures, invariant, budget)) {
        break;
      }
      this.runVerifyRules();
      this.updateTwinSnapshot();
      this.runConnectivityMaintenance();
    }
}

  private priorityRank(priority: TaskPriority): number {
    // PriorityRank.
    //
    // Parameters:
    // - `priority` — input value
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = priorityRank(priority);
    switch (priority) {
      case "critical":
        return 0;
      case "high":
        return 1;
      case "normal":
        return 2;
      case "low":
        return 3;
    }
}

  private taskBudgetViolation(
    budget: ResourceBudgetDecl,
    durationMs: number,
    intervalMs: number,
  ): string | null {
    // TaskBudgetViolation.
    //
    // Parameters:
    // - `budget` — input value
    // - `durationMs` — input value
    // - `intervalMs` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = taskBudgetViolation(budget, durationMs, intervalMs);
    const duty = (durationMs / Math.max(intervalMs, 1)) * 100;

    // continue when budget.cpuPctMax != null && duty > budget.cpuPctMax.
    if (budget.cpuPctMax != null && duty > budget.cpuPctMax) return "cpu";

    // continue when budget.batteryPctMax != null && duty > budget.batteryPctMax.
    if (budget.batteryPctMax != null && duty > budget.batteryPctMax) return "battery";
    return null;
}

  private runScheduledTask(
    name: string,
    priority: TaskPriority,
    intervalMs: number,
    body: Stmt[],
    requires: Expr | null,
    ensures: Expr | null,
    invariant: Expr | null,
    budget: ResourceBudgetDecl | null,
  ): boolean {
    // RunScheduledTask.
    //
    // Parameters:
    // - `name` — input value
    // - `priority` — input value
    // - `intervalMs` — input value
    // - `body` — input value
    // - `requires` — input value
    // - `ensures` — input value
    // - `invariant` — input value
    // - `budget` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = runScheduledTask(name, priority, intervalMs, body, requires, ensures, invariant, budget);
    const RUNTIME_TASK_COST_MS = 5;

    // continue when budget.
    if (budget) {
      const prev = this.taskMaxDurationMs.get(name) ?? 0;

      // continue when prev > 0.
      if (prev > 0) {
        const kind = this.taskBudgetViolation(budget, prev, intervalMs);

        // continue when kind.
        if (kind) {
          this.options.onLog?.(`task '${name}': ${kind} budget exceeded — skipping tick`);
          return true;
        }
      }
    }
    const continueRunning = this.executeTaskIteration(body, requires, ensures, invariant, name);
    const durationMs = RUNTIME_TASK_COST_MS;
    this.taskMaxDurationMs.set(name, Math.max(this.taskMaxDurationMs.get(name) ?? 0, durationMs));

    // continue when budget.
    if (budget) {
      const kind = this.taskBudgetViolation(budget, durationMs, intervalMs);

      // continue when kind.
      if (kind) {
        this.options.onLog?.(`task '${name}': ${kind} budget exceeded (${durationMs.toFixed(2)}ms)`);
      }
    }
    this.reliability.touchHeartbeat(name, this.reliability.simTimeMs);
    return continueRunning;
}

  private executeTaskIteration(
    body: Stmt[],
    requires: Expr | null,
    ensures: Expr | null,
    invariant: Expr | null,
    taskName?: string,
  ): boolean {
    // ExecuteTaskIteration.
    //
    // Parameters:
    // - `body` — input value
    // - `requires` — input value
    // - `ensures` — input value
    // - `invariant` — input value
    // - `taskName?` — optional input
    //
    // Returns:
    // true or false.
    //
    // Options:
    // - `taskName?` — optional parameter
    //
    // Example:

    // const result = executeTaskIteration(body, requires, ensures, invariant, taskName?);
    if (requires && !this.evalContract(requires)) {
      const label = taskName ? `task '${taskName}'` : "task";
      this.options.onLog?.(`${label} requires contract failed — skipping iteration`);
      return true;
    }
    this.executeBlock(body);

    // continue when ensures && !this.evalContract(ensures).
    if (ensures && !this.evalContract(ensures)) {
      throw new RuntimeError("task ensures contract failed", 0);
    }

    // continue when invariant && !this.evalContract(invariant).
    if (invariant && !this.evalContract(invariant)) {
      throw new RuntimeError("task invariant contract failed", 0);
    }
    return !this.safetyMonitor?.isEmergencyStop();
}

  private executeMultiplexedTasks(tasks: TaskDecl[]): void {
    // ExecuteMultiplexedTasks.
    //
    // Parameters:
    // - `tasks` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = executeMultiplexedTasks(tasks);
    if (tasks.length === 0) return;
    const schedules = tasks.map((task) => ({
      name: task.name,
      priority: task.priority,
      intervalMs: task.intervalMs,
      nextDueMs: 0,
      body: task.body,
      requires: task.requires,
      ensures: task.ensures,
      invariant: task.invariant,
      budget: task.budget,
    }));
    const baseTick = Math.max(1, Math.min(...schedules.map((task) => task.intervalMs)));
    this.options.onLog?.(
      `scheduler: multiplexing ${schedules.length} task(s) with base tick ${baseTick}ms`,
    );
    const maxIter = this.options.maxLoopIterations ?? 10;
    let simTime = 0;
    const wallMode = this.options.schedulerClock === "wall";
    const wallStart = performance.now();
    let wallNext = wallStart;

    // Loop with index variable i.
    for (let i = 0; i < maxIter; i++) {
      if (wallMode) {
        wallNext += baseTick;
        this.sleepWallMs(wallNext - performance.now());
        simTime = performance.now() - wallStart;
      } else {
        simTime += baseTick;
      }
      this.reliability.simTimeMs = simTime;
      this.options.backend.tick(baseTick);
      const due = schedules
        .filter((schedule) => schedule.nextDueMs <= simTime)
        .sort((a, b) => this.priorityRank(a.priority) - this.priorityRank(b.priority));

      // Iterate over due.
      for (const schedule of due) {
        this.options.onLog?.(`task '${schedule.name}': tick`);

        // continue when value.
        if (
          !this.runScheduledTask(
            schedule.name,
            schedule.priority,
            schedule.intervalMs,
            schedule.body,
            schedule.requires,
            schedule.ensures,
            schedule.invariant,
            schedule.budget,
          )
        ) {
          return;
        }
        schedule.nextDueMs = simTime + schedule.intervalMs;
      }
      this.processSpawnQueue();
      this.runVerifyRules();
      this.updateTwinSnapshot();
      this.reliability.checkWatchdogs(this.reliabilityHost());
      this.runConnectivityMaintenance();
      this.reliability.recordEvent(this.reliabilityHost(), "scheduler_tick", {
        simTimeMs: simTime,
        iteration: i + 1,
      });

      // continue when this.safetyMonitor?.isEmergencyStop().
      if (this.safetyMonitor?.isEmergencyStop()) break;
    }
}

  private refreshTwinShadowFromBackend(): void {
    // RefreshTwinShadowFromBackend.
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

    // const result = refreshTwinShadowFromBackend();
    if (!this.twinRuntime) return;
    const state = this.options.backend.getState();

    // check membership before continuing.
    if (this.twinRuntime.mirrors.includes("pose")) {
      this.twinRuntime.shadow.pose = {
        kind: "pose",
        x: state.pose.x,
        y: state.pose.y,
        theta: state.pose.theta,
        z: state.pose.z ?? 0,
      };
    }

    // check membership before continuing.
    if (this.twinRuntime.mirrors.includes("velocity")) {
      this.twinRuntime.shadow.velocity = {
        kind: "velocity",
        linear: state.velocity.linear,
        angular: state.velocity.angular,
      };
    }
}

  private updateTwinSnapshot(): void {
    // UpdateTwinSnapshot.
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

    // const result = updateTwinSnapshot();
    if (!this.twinRuntime) return;
    this.refreshTwinShadowFromBackend();

    // continue when this.twinRuntime.replay && Object.keys(this.twinRuntime.shadow).length.
    if (this.twinRuntime.replay && Object.keys(this.twinRuntime.shadow).length > 0) {
      this.twinRuntime.replayBuffer.push({ ...this.twinRuntime.shadow });
    }
    const fieldCount = Object.keys(this.twinRuntime.shadow).length;

    // continue when fieldCount > 0.
    if (fieldCount > 0) {
      this.options.onLog?.(
        `twin ${this.twinRuntime.name} mirrored ${fieldCount} field(s), replay frames=${this.twinRuntime.replayBuffer.length}`,
      );
    }
}

  private twinFieldFromExpr(expr: Expr): string {
    // TwinFieldFromExpr.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = twinFieldFromExpr(expr);
    if (expr.kind === "LiteralExpr" && typeof expr.value === "string") return expr.value;

    // continue when kind equals "IdentExpr".
    if (expr.kind === "IdentExpr") return expr.name;
    return getString(this.evalExpr(expr), "");
}

  private evalTwinMethod(
    method: string,
    expr: import("../ast/nodes.js").CallExpr,
  ): RuntimeValue {
    // EvalTwinMethod.
    //
    // Parameters:
    // - `method` — input value
    // - `expr` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = evalTwinMethod(method, expr);
    if (!this.twinRuntime) {
      throw new RuntimeError("No digital twin configured", expr.span.start.line);
    }
    this.refreshTwinShadowFromBackend();
    const twin = this.twinRuntime;

    // continue when method equals "frame count".
    if (method === "frame_count") {
      return { kind: "number", value: twin.replayBuffer.length, unit: "none" };
    }

    // continue when method equals "mirror".
    if (method === "mirror") {
      const fieldArg = expr.namedArgs.find((a) => a.name === "field");
      const field = fieldArg
        ? this.twinFieldFromExpr(fieldArg.value)
        : expr.args[0]
          ? this.twinFieldFromExpr(expr.args[0])
          : "";
      const value = twin.shadow[field];

      // continue when value is falsy.
      if (!value) {
        throw new RuntimeError(`Twin has no mirrored shadow field '${field}'`, expr.span.start.line);
      }
      return value;
    }

    // continue when method equals "replay".
    if (method === "replay") {

      // continue when replay is falsy.
      if (!twin.replay) {
        throw new RuntimeError("Twin replay is disabled — set replay true in twin block", expr.span.start.line);
      }
      const indexArg = expr.namedArgs.find((a) => a.name === "index");
      const index = indexArg
        ? getNumber(this.evalExpr(indexArg.value), 0)
        : expr.args[0]
          ? getNumber(this.evalExpr(expr.args[0]), 0)
          : 0;
      const fieldArg = expr.namedArgs.find((a) => a.name === "field");
      const field = fieldArg
        ? this.twinFieldFromExpr(fieldArg.value)
        : expr.args[1]
          ? this.twinFieldFromExpr(expr.args[1])
          : "";
      const frame = twin.replayBuffer[Math.floor(index)];
      const value = frame?.[field];

      // continue when value is falsy.
      if (!value) {
        throw new RuntimeError(
          `Twin replay frame ${Math.floor(index)} has no field '${field}'`,
          expr.span.start.line,
        );
      }
      return value;
    }

    // check membership before continuing.
    if (twin.mirrors.includes(method)) {
      const value = twin.shadow[method];

      // continue when value is falsy.
      if (!value) {
        throw new RuntimeError(`Twin shadow field '${method}' not yet mirrored`, expr.span.start.line);
      }
      return value;
    }
    return { kind: "void" };
}

  private checkAgentCapability(
    agent: string,
    action: string,
    target: string | undefined,
    line: number,
  ): void {
    // CheckAgentCapability.
    //
    // Parameters:
    // - `agent` — input value
    // - `action` — input value
    // - `target` — input value
    // - `line` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkAgentCapability(agent, action, target, line);
    const caps = this.agentCapabilities.get(agent) ?? [];

    // continue when length equals 0.
    if (caps.length === 0) {
      return;
    }
    const allowed = caps.some(
      (c) => c.action === action && (target === undefined || c.target === target),
    );

    // continue when allowed is falsy.
    if (!allowed) {
      const suffix = target ? `(${target})` : "";
      throw new RuntimeError(`Agent '${agent}' lacks capability ${action}${suffix}`, line);
    }
}

  private dispatchEvent(eventName: string): void {
    // DispatchEvent.
    //
    // Parameters:
    // - `eventName` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = dispatchEvent(eventName);
    const body = this.eventHandlers.get(eventName);

    // continue when body.
    if (body) {
      this.options.onLog?.(`emit ${eventName}`);
      this.executeBlock(body);

    // Handle any remaining cases.
    } else {
      this.options.onLog?.(`emit ${eventName} (no handler)`);
    }
}

  private executeEnter(stateName: string, line: number): void {
    // ExecuteEnter.
    //
    // Parameters:
    // - `stateName` — input value
    // - `line` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = executeEnter(stateName, line);
    let transitioned = false;

    // Iterate over the collection.
    for (const [smName, sm] of this.stateMachines) {

      // continue when includes is falsy.
      if (!sm.states.includes(stateName)) continue;
      const allowed = sm.transitions.some((t) => t.from === sm.current && t.to === stateName);

      // continue when allowed is falsy.
      if (!allowed) continue;
      const previous = sm.current;
      sm.current = stateName;
      this.options.onLog?.(`state_machine ${smName}: ${previous} -> ${stateName}`);
      transitioned = true;
    }

    // continue when transitioned is falsy.
    if (!transitioned) {
      throw new RuntimeError(`No valid transition to state '${stateName}'`, line);
    }
}

  private setupRobot(robot: RobotDecl): void {
    // SetupRobot.
    //
    // Parameters:
    // - `robot` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = setupRobot(robot);
    this.currentRobot = robot;
    this.env = new Environment();
    if (this.fleets.names().length > 0) {
      this.env.define("fleet", { kind: "fleet_control", registry: this.fleets.clone() });
    }
    this.topicPathToMessageType.clear();
    this.zones = [];
    this.eventHandlers.clear();
    this.stateMachines.clear();
    this.twinRuntime = null;
    this.agentCapabilities.clear();
    this.agentTraitImpls.clear();
    this.verifyRules = [];
    this.fusionSensors = [];
    this.security = new SecurityContext();
    this.currentAgent = null;
    this.commBus = new RoutingCommBus();
    this.commBus.attachProviderRegistry(this.providerRegistry);
    if (this.providerRegistry.officialPackages().length > 0) {
      syncCommBusForOfficialPackages(this.commBus, this.providerRegistry);
    }
    for (const fault of this.injectedFaults) {
      this.commBus.injectFault(fault);
    }
    this.defaultTransport = connectivityLinkToTransport(this.activeConnectivityLink);

    // continue when robot.soc.
    if (robot.soc) {
      const profile = getSocProfile(robot.soc.profile);
      this.options.onLog?.(`SoC: ${profile?.name ?? robot.soc.profile} (${profile?.architecture ?? "unknown"})`);
    }
    this.hal = this.options.backend.getHal?.() ?? createSimHal();

    // continue when robot.hal.
    if (robot.hal) {
      const members = robot.hal.members.map(halMemberFromDecl);
      this.hal.configure(members);
      this.options.onLog?.(`HAL configured: ${members.length} bus(es)/pin(s)`);
    }

    // continue when robot.permissions.
    if (robot.permissions) {
      this.security.enableStrictPermissions();
      this.security.capabilities.grantAll(robot.permissions.capabilities);
      this.options.onLog?.(
        `permissions: strict mode, granted ${robot.permissions.capabilities.length} capability(ies)`,
      );
    }

    // continue when robot.trust.
    if (robot.trust) {
      const level = parseTrustLevel(robot.trust.level);

      // continue when level.
      if (level) {
        this.security.trust = level;
        this.options.onLog?.(`trust: level set to ${level}`);
      }
    }

    // Iterate over secrets ?? [].
    for (const secret of robot.secrets ?? []) {
      this.security.secrets.register(secret.name, secret.source);
      this.env.define(secret.name, { kind: "secret", name: secret.name });
      this.options.onLog?.(`secret '${secret.name}': registered`);
    }

    // continue when robot.identity.
    if (robot.identity) {
      const id = robot.identity.fields.find(([k]) => k === "id")?.[1] ?? "unknown";
      const publicKey = robot.identity.fields.find(([k]) => k === "public_key")?.[1] ?? "";
      this.security.identity = createRobotIdentity(id, publicKey, this.security.trust);
      this.env.define("identity", { kind: "identity", id, publicKey });
      this.security.grantIfNotStrict("identity.sign");
      this.security.grantIfNotStrict("identity.verify");
      this.options.onLog?.(`identity: device '${id}' registered`);
    }

    for (const tb of robot.trustBoundaries ?? []) {
      try {
        this.security.trustBoundaries.declare(parseTrustBoundary(tb.name));
        this.options.onLog?.(`trust boundary: ${tb.name}`);
      } catch (e) {
        throw new RuntimeError(String(e), tb.span.start.line);
      }
    }

    // Process each bus after secrets, identity, and trust boundaries are registered.
    for (const bus of robot.buses) {
      this.defaultTransport = bus.transport;
      let busSecurity = transportSecurityFromBusFields(
        bus.encryption,
        bus.authentication,
        bus.integrity,
      );
      if (robot.secureComm) {
        const robotPolicy: SecureCommPolicy = {
          encryption: (robot.secureComm.encryption as SecureCommPolicy["encryption"]) ?? "none",
          authentication:
            (robot.secureComm.authentication as SecureCommPolicy["authentication"]) ?? "none",
          integrity: (robot.secureComm.integrity as SecureCommPolicy["integrity"]) ?? "none",
        };
        busSecurity = effectiveTransportPolicy(robotPolicy, busSecurity);
      }
      for (const secret of robot.secrets ?? []) {
        if (secret.name.includes("cert") && secret.source.source === "file") {
          busSecurity = { ...busSecurity, certPath: secret.source.path };
          this.security.wireCertPath = secret.source.path;
        }
        if (secret.name.includes("key")) {
          busSecurity = { ...busSecurity, keySecret: secret.name };
          if (secret.source.source === "file") {
            busSecurity = { ...busSecurity, keyPath: secret.source.path };
            this.security.wireKeySecret = secret.name;
          }
        }
      }
      const tls = new TlsTransportSession();
      const transportBoundary = boundaryForTransportName(bus.transport);
      this.security.setTransportContext(
        transportBoundary,
        busSecurity.encryption,
        busSecurity.authentication,
        busSecurity.integrity,
      );
      const resolvedBroker = resolveBrokerUrl(bus.brokerUrl);
      this.commBus.configure({
        nodeName: robot.name,
        security: busSecurity,
        tls,
        brokerUrl: resolvedBroker,
      });
      this.options.onLog?.(
        `bus transport: ${bus.transport} (encryption: ${busSecurity.encryption})`,
      );
    }

    // Process each peerRobot.
    for (const peer of robot.peerRobots) {
      this.commBus.registerRobot(peer.name);
    }

    // Process each device.
    for (const device of robot.devices) {
      this.commBus.registerDevice(device.name);
      this.env.define(device.name, {
        kind: "object",
        typeName: "Device",
        fields: {},
      });
    }

    // Process each topic.
    for (const topic of robot.topics) {
      this.defineTopic(topic);
    }

    // Process each service.
    for (const service of robot.services) {
      const serviceType = service.serviceType ?? service.responseType ?? service.name;
      this.env.define(service.name, {
        kind: "service",
        name: service.name,
        serviceType,
      });
    }

    // Process each action.
    for (const action of robot.actions) {
      const actionType = action.actionType ?? action.resultType ?? action.name;
      this.env.define(action.name, {
        kind: "action",
        name: action.name,
        actionType,
      });
    }

    // Process each sensor.
    for (const sensor of robot.sensors) {
      const topic = sensor.binding?.kind === "topic" ? sensor.binding.path : null;
      const halBinding = sensor.binding?.kind === "hal" ? sensor.binding.busName : null;
      this.env.define(sensor.name, {
        kind: "sensor",
        name: sensor.name,
        sensorType: sensor.sensorType,
        library: sensor.library,
        halBinding,
        topic,
      });
    }

    // Process each actuator.
    for (const actuator of robot.actuators) {
      this.env.define(actuator.name, {
        kind: "actuator",
        name: actuator.name,
        actuatorType: actuator.actuatorType,
      });
    }
    this.ai_models.clear();
    this.agents.clear();

    // Iterate over ai models ?? [].
    for (const modelDecl of robot.ai_models ?? []) {
      const model = createAIModel(modelDecl);
      this.ai_models.set(modelDecl.name, model);
      this.env.define(modelDecl.name, model.toRuntimeValue());
      this.options.onLog?.(
        `AI model '${modelDecl.name}': ${modelDecl.modelType} (${model.config.provider}/${model.config.model})`,
      );
    }

    // Iterate over agents ?? [].
    for (const agentDecl of robot.agents ?? []) {
      const memory = agentDecl.memoryKind ? new MemoryStore(agentDecl.memoryKind) : null;
      const agent = createAgentRuntime(agentDecl, memory);
      this.agents.set(agentDecl.name, agent);
      this.agentCapabilities.set(agentDecl.name, agentDecl.capabilities);
      this.commBus.registerAgent(agentDecl.name);
      this.env.define(agentDecl.name, { kind: "agent", name: agentDecl.name });
      this.options.onLog?.(`Agent '${agentDecl.name}': ${agentDecl.goal}`);
    }

    // Iterate over agentChannels ?? [].
    for (const channel of robot.agentChannels ?? []) {
      this.concurrency.registerAgentRoute(channel.fromAgent, channel.toAgent, channel.messageType);
      const typeSuffix = channel.messageType ? ` (${channel.messageType})` : "";
      this.options.onLog?.(
        `agent channel: ${channel.fromAgent} -> ${channel.toAgent}${typeSuffix}`,
      );
    }

    // Iterate over traitImpls ?? [].
    for (const traitImpl of robot.traitImpls ?? []) {
      const agentMethods = this.agentTraitImpls.get(traitImpl.agentName) ?? new Map();

      // Process each method.
      for (const method of traitImpl.methods) {
        agentMethods.set(method.name, { params: method.params, body: method.body });
      }
      this.agentTraitImpls.set(traitImpl.agentName, agentMethods);
    }

    // Iterate over events ?? [].
    for (const event of robot.events ?? []) {
      this.options.onLog?.(`event declared: ${event.name}`);
    }

    // Invoke each registered handler.
    for (const handler of robot.eventHandlers ?? []) {
      this.eventHandlers.set(handler.eventName, handler.body);
      this.options.onLog?.(`handler registered for ${handler.eventName}`);
    }

    // continue when robot.twin.
    if (robot.twin) {
      this.twinRuntime = {
        name: robot.twin.name,
        mirrors: [...robot.twin.mirrors],
        replay: robot.twin.replay,
        shadow: {},
        replayBuffer: [],
      };
      this.env.define(robot.twin.name, { kind: "twin", name: robot.twin.name });
      this.options.onLog?.(
        `twin ${robot.twin.name}: mirrors [${robot.twin.mirrors.join(", ")}], replay=${robot.twin.replay}`,
      );
    }

    // continue when robot.verify.
    if (robot.verify) {
      this.verifyRules = [...robot.verify.rules];
      this.options.onLog?.(`verify: ${robot.verify.rules.length} rule(s) registered`);
    }

    // continue when robot.observe.
    if (robot.observe) {
      this.fusionSensors = [...robot.observe.sensors];
      this.env.define("fusion", { kind: "sensor_fusion", sensors: [...robot.observe.sensors] });
      this.options.onLog?.(
        `observe: fusing ${robot.observe.sensors.length} sensor(s) [${robot.observe.sensors.join(", ")}]`,
      );
    }

    // Initialize mission controller and navigation helpers when declared.
    if (robot.mission) {
      const runtime = createMissionRuntime(
        robot.mission.name,
        [...robot.mission.steps],
        robot.mission.durationHours,
      );
      this.env.define("mission", { kind: "mission_control", runtime });
      this.env.define("navigation", { kind: "navigation_control", goal: null });
      const label = robot.mission.name ?? "mission";
      this.options.onLog?.(
        `mission '${label}': ${robot.mission.steps.length} step(s), duration=${robot.mission.durationHours} h`,
      );
    }

    if (this.slamEnabled) {
      this.env.define("slam", { kind: "slam_control" });
      this.options.onLog?.("slam: adapter enabled (stub localize/map hooks)");
    }

    // Iterate over stateMachines ?? [].
    for (const sm of robot.stateMachines ?? []) {
      const initial = sm.states[0] ?? "unknown";
      this.stateMachines.set(sm.name, {
        current: initial,
        states: [...sm.states],
        transitions: sm.transitions.map((t) => ({ from: t.from, to: t.to })),
      });
      this.options.onLog?.(`state_machine ${sm.name}: initial state ${initial}`);
    }

    // continue when robot.safety.
    if (robot.safety) {
      this.env.define("safety", { kind: "safety_ctx" });
    }
    this.env.define("robot", { kind: "robot" });
    const stopIfRules: Array<(env: Environment) => boolean> = [];
    let maxSpeed = Infinity;

    // continue when robot.safety.
    if (robot.safety) {

      // Process each rule.
      for (const rule of robot.safety.rules) {

        // continue when kind equals "MaxSpeedRule".
        if (rule.kind === "MaxSpeedRule") {
          const val = this.evalExpr(rule.value);

          // continue when kind equals "number".
          if (val.kind === "number") maxSpeed = val.value;

        // Handle any remaining cases.
        } else {
          stopIfRules.push((env) => {
            const saved = this.env;
            this.env = env;
            const result = this.evalExpr(rule.condition);
            this.env = saved;
            return result.kind === "bool" && result.value;
          });
        }
      }

      // Process each zone.
      for (const zone of robot.safety.zones) {
        this.zones.push(this.evalSafetyZone(zone));
      }
    }
    this.safetyMonitor = new SafetyMonitor(
      createSafetyConfigFromRobot(
        maxSpeed,
        stopIfRules,
        this.zones,
        this.programSafetyZones.speedCaps(),
      ),
    );
    this.reliability.loadFromRobot(
      robot,
      this.options.recordTrace ?? false,
      this.options.traceSource,
    );
    if (this.reliability.modes.has("normal")) {
      this.reliability.enterMode("normal", this.reliabilityHost());
    }
    this.runConnectivityMaintenance();
}

  private reliabilityHost() {
    return {
      executeBlock: (stmts: Stmt[]) => this.executeBlock(stmts),
      log: (message: string) => this.options.onLog?.(message),
      getSimTimeMs: () => this.reliability.simTimeMs,
    };
  }

  private evalSafetyZone(zone: SafetyZoneDecl): SafetyZoneRuntime {
    // EvalSafetyZone.
    //
    // Parameters:
    // - `zone` — input value
    //
    // Returns:
    // SafetyZoneRuntime.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = evalSafetyZone(zone);
    const base: SafetyZoneRuntime = {
      name: zone.name,
      shape: zone.shape,
      x: getNumber(this.evalExpr(zone.x)),
      y: getNumber(this.evalExpr(zone.y)),
    };

    // continue when shape equals radius.
    if (zone.shape === "circle" && zone.radius) {
      base.radius = getNumber(this.evalExpr(zone.radius));
    }

    // continue when shape equals height.
    if (zone.shape === "rect" && zone.width && zone.height) {
      base.width = getNumber(this.evalExpr(zone.width));
      base.height = getNumber(this.evalExpr(zone.height));
    }
    return base;
}

  private defineTopic(topic: import("../ast/nodes.js").TopicDecl): void {    // Resolve the filesystem path for the next step.
    const path = topic.topic ?? `/${topic.name}`;

    // continue when topic.secure.
    if (topic.secure) {
      this.security.secureEndpoints.register(path, {
        signed: topic.secure.signed,
        minTrust: topic.secure.minTrust ? parseTrustLevel(topic.secure.minTrust) : null,
        requires: topic.secure.requires,
        encryption: (topic.secure.encryption as SecurePolicy["encryption"]) ?? "none",
        authentication: (topic.secure.authentication as SecurePolicy["authentication"]) ?? "none",
        integrity: (topic.secure.integrity as SecurePolicy["integrity"]) ?? "none",
        trustedSources: topic.secure.trustedSources ?? [],
        rejectUntrusted: topic.secure.rejectUntrusted ?? false,
      });
    }
    this.commBus.subscribe(path, topic.name);
    this.topicPathToMessageType.set(path, topic.messageType);
    this.env.define(topic.name, {
      kind: "topic",
      name: topic.name,
      messageType: topic.messageType,
      topicPath: path,
    });
}

  private executeBlock(stmts: Stmt[]): void {
    // ExecuteBlock.
    //
    // Parameters:
    // - `stmts` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = executeBlock(stmts);
    for (const stmt of stmts) {
      this.executeStmt(stmt);

      // continue when this.returning.
      if (this.returning) break;
    }
}

  private executeBlockWithReturn(stmts: Stmt[]): RuntimeValue {
    // ExecuteBlockWithReturn.
    //
    // Parameters:
    // - `stmts` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = executeBlockWithReturn(stmts);
    this.returning = false;
    this.returnValue = { kind: "void" };

    // Execute each statement in sequence.
    for (const stmt of stmts) {
      this.executeStmt(stmt);

      // continue when this.returning.
      if (this.returning) break;
    }
    return this.returnValue;
}

  private callModuleFunction(func: ModuleFnDecl, args: Expr[]): RuntimeValue {
    // CallModuleFunction.
    //
    // Parameters:
    // - `func` — input value
    // - `args` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = callModuleFunction(func, args);
    if (func.isAsync) {
      const argValues = args.map((arg) => this.evalExpr(arg));
      return { kind: "future", funcName: func.name, args: argValues, resolved: null };
    }
    const saved = this.env.clone();

    // Loop with index variable i.
    for (let i = 0; i < func.params.length; i++) {
      const param = func.params[i];
      const arg = args[i];

      // continue when param && arg.
      if (param && arg) {
        this.env.define(param.name, this.evalExpr(arg));
      }
    }
    const result = this.executeBlockWithReturn(func.body);
    this.env = saved;
    return result;
}

  private executeStmt(stmt: Stmt): void {
    // ExecuteStmt.
    //
    // Parameters:
    // - `stmt` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = executeStmt(stmt);
    switch (stmt.kind) {
      case "VarDecl":

        // continue when stmt.init.
        if (stmt.init) {

          // continue when value.
          if (
            stmt.typeAnnotation?.kind === "trait_object" &&
            stmt.init.kind === "IdentExpr"
          ) {
            this.env.define(stmt.name, {
              kind: "trait_object",
              traitName: stmt.typeAnnotation.traitName,
              agent: stmt.init.name,
            });

          // Handle any remaining cases.
          } else {
            this.env.define(stmt.name, this.evalExpr(stmt.init));
          }

        // Handle any remaining cases.
        } else {
          this.env.define(stmt.name, { kind: "void" });
        }
        break;
      case "IfStmt": {
        const cond = this.evalExpr(stmt.condition);

        // continue when kind equals value.
        if (cond.kind === "bool" && cond.value) {
          this.executeBlock(stmt.thenBranch);

        // Otherwise, continue when stmt.elseBranch.
        } else if (stmt.elseBranch) {
          this.executeBlock(stmt.elseBranch);
        }
        break;
      }
      case "LoopStmt": {
        const maxIter = this.options.maxLoopIterations ?? 10;

        // Loop with index variable i.
        for (let i = 0; i < maxIter; i++) {
          this.options.backend.tick(stmt.intervalMs);
          this.executeBlock(stmt.body);

          // continue when this.safetyMonitor?.isEmergencyStop().
          if (this.safetyMonitor?.isEmergencyStop()) break;
        }
        break;
      }
      case "PublishStmt": {
        const topic = this.env.get(stmt.topicName);
        const value = this.evalExpr(stmt.value);

        if (topic?.kind === "topic") {
          const sourceId = this.currentAgent ?? this.security.identity?.id ?? "robot";
          const payload =
            typeof value === "object" && value !== null ? JSON.stringify(value) : String(value);
          try {
            this.security.preparePublish(topic.topicPath, payload, sourceId, topic.messageType);
          } catch (e) {
            this.options.onLog?.(`security: publish denied on ${topic.topicPath}: ${e}`);
            throw e;
          }
          this.commBus.publish(
            topic.topicPath,
            topic.messageType,
            value,
            this.defaultTransport,
            sourceId,
          );
          this.options.backend.publishTopic?.(topic.topicPath, topic.messageType, value);
          this.options.onLog?.(`publish ${topic.topicPath}`);
        }
        break;
      }
      case "ServiceCallStmt": {
        const service = this.env.get(stmt.serviceName);

        // continue when kind equals "service".
        if (service?.kind === "service") {
          this.commBus.callService(service.serviceType);
          this.options.backend.callService?.(service.name, service.serviceType);
          this.options.onLog?.(`call ${service.name}()`);
        }
        break;
      }
      case "ActionSendStmt": {
        const action = this.env.get(stmt.actionName);
        const goal = this.evalExpr(stmt.goal);

        // continue when kind equals "action".
        if (action?.kind === "action") {
          this.commBus.sendAction(action.actionType);
          this.options.backend.sendAction?.(action.name, action.actionType, goal);
          this.options.onLog?.(`send_goal ${action.name}`);
        }
        break;
      }
      case "SubscribeStmt": {
        const path = stmt.target.includes(".")
          ? `/${stmt.target.replace(".", "/")}`
          : this.env.get(stmt.target)?.kind === "topic"
            ? (this.env.get(stmt.target) as Extract<RuntimeValue, { kind: "topic" }>).topicPath
            : `/${stmt.target}`;
        this.security.authorizeSubscribe(path);
        this.commBus.subscribe(path, stmt.target);
        this.options.onLog?.(`subscribe ${stmt.target}`);
        break;
      }
      case "ExecuteStmt": {
        const action = this.env.get(stmt.actionName);
        const goal = this.evalExpr(stmt.goal);

        // continue when kind equals "action".
        if (action?.kind === "action") {
          this.commBus.sendAction(action.actionType);
          this.options.backend.sendAction?.(action.name, action.actionType, goal);
          this.options.onLog?.(`execute ${action.name}`);
        }
        break;
      }
      case "DiscoverStmt": {
        const results = this.commBus.discover(stmt.target, stmt.filter ?? { capability: null });
        this.options.onLog?.(`discover ${stmt.target}: ${results.join(", ")}`);
        break;
      }
      case "ReceiveStmt": {
        const path = stmt.topicName.includes(".")
          ? `/${stmt.topicName.replace(".", "/")}`
          : this.env.get(stmt.topicName)?.kind === "topic"
            ? (this.env.get(stmt.topicName) as Extract<RuntimeValue, { kind: "topic" }>).topicPath
            : `/${stmt.topicName}`;
        const envelope = this.commBus.receiveEnvelope(path);

        // continue when envelope.
        if (envelope) {
          const payload =
            typeof envelope.value === "object" && envelope.value !== null
              ? JSON.stringify(envelope.value)
              : String(envelope.value);
          this.security.verifyInboundMessage(
            path,
            payload,
            envelope.sourceId,
            this.topicPathToMessageType.get(path) ?? "Unknown",
          );
          this.env.define(stmt.varName, envelope.value);
          this.options.onLog?.(`receive ${stmt.topicName} to ${stmt.varName}`);
        }
        break;
      }
      case "EmergencyStopStmt":
        this.safetyMonitor?.setEmergencyStop(true);
        this.options.backend.setEmergencyStop?.(true);
        this.options.backend.executeMotion({ kind: "stop", actuator: "all" });
        this.options.onLog?.("EMERGENCY STOP triggered");
        break;
      case "ResetEmergencyStopStmt":
        this.safetyMonitor?.reset();
        this.options.backend.setEmergencyStop?.(false);
        this.options.onLog?.("Emergency stop reset");
        break;
      case "EmitStmt":
        this.dispatchEvent(stmt.eventName);
        break;
      case "EnterStmt":
        this.executeEnter(stmt.stateName, stmt.span.start.line);
        break;
      case "EnterModeStmt":
        this.reliability.enterMode(stmt.mode, this.reliabilityHost());
        break;
      case "StopAllActuatorsStmt":
        this.safetyMonitor?.setEmergencyStop(true);
        this.options.backend.setEmergencyStop?.(true);
        this.options.backend.executeMotion({ kind: "stop", actuator: "all" });
        this.options.onLog?.("safety: stop_all_actuators invoked");
        break;
      case "RunPipelineStmt":
        this.reliability.executePipeline(stmt.name, this.reliabilityHost());
        break;
      case "NavigateStmt":
        this.executeNavigateStmt(stmt);
        break;
      case "UseFallbackStmt":
        this.options.onLog?.(`use fallback ${stmt.resource}`);
        break;
      case "RememberStmt": {

        // continue when currentAgent is falsy.
        if (!this.currentAgent) {
          throw new RuntimeError(
            "remember requires active agent context (run inside agent plan)",
            stmt.span.start.line,
          );
        }
        const agent = this.agents.get(this.currentAgent);

        // continue when memory is falsy.
        if (!agent?.memory) {
          throw new RuntimeError(
            "Agent has no memory — declare memory short_term or long_term on the agent",
            stmt.span.start.line,
          );
        }
        agent.memory.remember(stmt.key, this.evalExpr(stmt.value));
        this.options.onLog?.(`remember '${stmt.key}'`);
        break;
      }
      case "ExprStmt":
        this.evalExpr(stmt.expr);
        break;
      case "ReturnStmt":
        this.returnValue = stmt.value ? this.evalExpr(stmt.value) : { kind: "void" };
        this.returning = true;
        break;
      case "SpawnStmt": {
        const { funcName, args } = this.evalSpawnTarget(stmt.callee, stmt.args, stmt.span.start.line);
        this.concurrency.queueFireAndForget(funcName, args);
        this.options.onLog?.(`spawn ${funcName}()`);
        break;
      }
      case "SelectStmt": {

        // Process each arm.
        for (const arm of stmt.arms) {
          const channelVal = this.evalExpr(arm.channel);
          const msg = this.concurrency.tryRecv(channelVal, stmt.span.start.line);

          // continue when msg.
          if (msg) {
            this.env.define("_msg", msg);
            this.executeBlock(arm.body);
            break;
          }
        }
        break;
      }
      case "ParallelStmt": {
        const saved = this.env.clone();
        const pendingHandles: Array<[string | null, number]> = [];
        const results: Record<string, RuntimeValue> = {};
        this.options.onLog?.(`parallel: executing ${stmt.body.length} branch(es) cooperatively`);

        // Iterate over body.
        for (const branch of stmt.body) {
          this.env = saved.clone();

          // continue when kind equals init.
          if (branch.kind === "VarDecl" && branch.init) {
            const val = this.evalExpr(branch.init);

            // continue when kind equals "task handle".
            if (val.kind === "task_handle") {
              pendingHandles.push([branch.name, val.id]);

            // Handle any remaining cases.
            } else {
              results[branch.name] = val;
            }

          // Otherwise, continue when kind equals "ExprStmt".
          } else if (branch.kind === "ExprStmt") {
            const val = this.evalExpr(branch.expr);

            // continue when kind equals "task handle".
            if (val.kind === "task_handle") {
              pendingHandles.push([null, val.id]);
            }

          // Otherwise, continue when kind equals "SpawnStmt".
          } else if (branch.kind === "SpawnStmt") {
            const { funcName, args } = this.evalSpawnTarget(
              branch.callee,
              branch.args,
              branch.span.start.line,
            );
            const handle = this.concurrency.createTaskHandle(funcName, args);

            // continue when kind equals "task handle".
            if (handle.kind === "task_handle") {
              pendingHandles.push([null, handle.id]);
            }

          // Handle any remaining cases.
          } else {
            this.executeStmt(branch);
          }
        }
        this.env = saved;

        // Iterate over the collection.
        for (const [name, id] of pendingHandles) {
          const result = this.resolveTaskHandle(id, stmt.span.start.line);

          // continue when name.
          if (name) results[name] = result;
        }

        // continue when Object.keys(results).length > 0.
        if (Object.keys(results).length > 0) {
          this.env.define("_parallel", {
            kind: "object",
            typeName: "ParallelResults",
            fields: results,
          });
          this.options.onLog?.(`parallel: aggregated ${Object.keys(results).length} result(s)`);
        }
        break;
      }
    }
}

  private evalExpr(expr: Expr): RuntimeValue {
    // EvalExpr.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = evalExpr(expr);
    switch (expr.kind) {
      case "LiteralExpr":

        // continue when value equals "boolean".
        if (typeof expr.value === "boolean") return { kind: "bool", value: expr.value };

        // continue when value equals "number".
        if (typeof expr.value === "number") return { kind: "number", value: expr.value, unit: "none" };

        // continue when value equals "string".
        if (typeof expr.value === "string") return { kind: "string", value: expr.value };
        if (typeof expr.value === "object" && expr.value !== null && "source" in expr.value) {
          return { kind: "regex", pattern: expr.value as RegexPattern };
        }
        return { kind: "void" };
      case "UnitLiteralExpr":
        return { kind: "number", value: expr.value, unit: expr.unit };
      case "IdentExpr": {
        const enumName = this.variantOwner.get(expr.name);

        // continue when enumName.
        if (enumName) {
          return { kind: "enum", enumName, variant: expr.name, payloads: [] };
        }
        const val = this.env.get(expr.name);

        // continue when line is falsy.
        if (!val) throw new RuntimeError(`Undefined variable '${expr.name}'`, expr.span.start.line);
        return val;
      }
      case "BinaryExpr":
        return this.evalBinary(expr.op, this.evalExpr(expr.left), this.evalExpr(expr.right));
      case "UnaryExpr": {
        const operand = this.evalExpr(expr.operand);

        // continue when op equals "not".
        if (expr.op === "not") {
          return { kind: "bool", value: operand.kind === "bool" && !operand.value };
        }

        // continue when op equals kind === "number".
        if (expr.op === "-" && operand.kind === "number") {
          return { kind: "number", value: -operand.value, unit: operand.unit };
        }
        return { kind: "void" };
      }
      case "AwaitExpr":
        return this.resolveFuture(this.evalExpr(expr.operand), expr.span.start.line);
      case "SpawnExpr": {
        const { funcName, args } = this.evalSpawnTarget(expr.callee, expr.args, expr.span.start.line);
        return this.concurrency.createTaskHandle(funcName, args);
      }
      case "MemberExpr":
        return this.evalMember(expr);
      case "CallExpr":
        return this.evalCall(expr);
      case "MatchExpr": {
        const value = this.evalExpr(expr.scrutinee);
        let variant = "";

        // continue when kind equals "enum".
        if (value.kind === "enum") variant = value.variant;

        // Otherwise, continue when kind equals "string".
        else if (value.kind === "string") variant = value.value;

        // Otherwise, continue when kind equals "object".
        else if (value.kind === "object") variant = value.typeName;

        // Process each arm.
        for (const arm of expr.arms) {

          // continue when variant equals variant.
          if (arm.variant === variant) {
            const bindings = arm.bindings ?? [];

            // continue when kind equals "enum".
            if (bindings.length > 0 && value.kind === "enum") {

              // Loop with index variable i.
              for (let i = 0; i < bindings.length; i++) {
                const payload = value.payloads[i];
                const binding = bindings[i];

                // continue when payload && binding) this.env.set(binding, payload.
                if (payload && binding) this.env.set(binding, payload);
              }
            }
            this.executeBlock(arm.body);

            // Process each binding.
            for (const binding of bindings) {
              this.env.remove(binding);
            }
            break;
          }
        }
        return { kind: "void" };
      }
      case "StructLiteralExpr":
        return this.evalStructLiteral(expr);
      case "ServiceCallExpr": {
        const service = this.env.get(expr.serviceName);

        // continue when kind equals "service".
        if (service?.kind === "service") {
          const result = this.commBus.callService(service.serviceType);
          this.options.backend.callService?.(service.name, service.serviceType);
          this.options.onLog?.(`call ${service.name}()`);
          return result;
        }
        return { kind: "void" };
      }
      case "ExecuteExpr": {
        const action = this.env.get(expr.actionName);

        // continue when kind equals "action".
        if (action?.kind === "action") {
          const goal = this.evalExpr(expr.goal);
          const result = this.commBus.sendAction(action.actionType);
          this.options.backend.sendAction?.(action.name, action.actionType, goal);
          this.options.onLog?.(`execute ${action.name}`);
          return result;
        }
        return { kind: "void" };
      }
      case "DiscoverExpr": {
        const results = this.commBus.discover(expr.target, expr.filter ?? { capability: null });
        return {
          kind: "object",
          typeName: "DiscoveryResult",
          fields: {
            count: { kind: "number", value: results.length, unit: "none" },
          },
        };
      }
      default:
        return { kind: "void" };
    }
}

  private evalStructLiteral(expr: import("../ast/nodes.js").StructLiteralExpr): RuntimeValue {
    // EvalStructLiteral.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = evalStructLiteral(expr);
    const values: Record<string, RuntimeValue> = {};

    // Process each field.
    for (const field of expr.fields) {
      values[field.name] = this.evalExpr(field.value);
    }

    // continue when typeName equals "Pose".
    if (expr.typeName === "Pose") {
      const x = values.x?.kind === "number" ? values.x.value : 0;
      const y = values.y?.kind === "number" ? values.y.value : 0;
      const heading = values.heading?.kind === "number"
        ? values.heading.value
        : values.theta?.kind === "number"
          ? values.theta.value
          : 0;
      const z = values.z?.kind === "number" ? values.z.value : 0;
      return { kind: "pose", x, y, theta: heading, z };
    }
    return { kind: "object", typeName: expr.typeName, fields: values };
}

  private evalMember(expr: import("../ast/nodes.js").MemberExpr): RuntimeValue {
    // EvalMember.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = evalMember(expr);
    if (expr.object.kind === "IdentExpr") {
      const variants = this.enumVariants.get(expr.object.name);

      // check membership before continuing.
      if (variants?.includes(expr.property)) {
        return { kind: "enum", enumName: expr.object.name, variant: expr.property, payloads: [] };
      }
    }
    const obj = this.evalExpr(expr.object);

    // continue when kind equals property === "nearest distance".
    if (obj.kind === "scan" && expr.property === "nearest_distance") {
      return { kind: "number", value: obj.nearestDistance, unit: "m" };
    }

    // continue when kind equals "pose".
    if (obj.kind === "pose") {
      const map: Record<string, RuntimeValue> = {
        x: { kind: "number", value: obj.x, unit: "m" },
        y: { kind: "number", value: obj.y, unit: "m" },
        theta: { kind: "number", value: obj.theta, unit: "rad" },
        z: { kind: "number", value: obj.z, unit: "m" },
      };
      return map[expr.property] ?? { kind: "void" };
    }

    // continue when kind equals "velocity".
    if (obj.kind === "velocity") {
      const map: Record<string, RuntimeValue> = {
        linear: { kind: "number", value: obj.linear, unit: "m/s" },
        angular: { kind: "number", value: obj.angular, unit: "rad/s" },
      };
      return map[expr.property] ?? { kind: "void" };
    }

    // continue when kind equals property === "nearest distance".
    if (obj.kind === "sensor" && expr.property === "nearest_distance") {
      const reading = this.readSensorValue(obj);

      // continue when kind equals "scan".
      if (reading.kind === "scan") {
        return { kind: "number", value: reading.nearestDistance, unit: "m" };
      }
    }

    // continue when kind equals "action proposal".
    if (obj.kind === "action_proposal") {

      // continue when property equals "trace".
      if (expr.property === "trace") {
        return {
          kind: "object",
          typeName: "ReasoningTrace",
          fields: {
            source: { kind: "string", value: obj.source },
            steps: { kind: "string", value: obj.trace.join("\n") },
            step_count: { kind: "number", value: obj.trace.length, unit: "none" },
          },
        };
      }
      const map: Record<string, RuntimeValue> = {
        linear: { kind: "number", value: obj.linear, unit: "m/s" },
        angular: { kind: "number", value: obj.angular, unit: "rad/s" },
      };
      return map[expr.property] ?? { kind: "void" };
    }

    // continue when kind equals "safe action".
    if (obj.kind === "safe_action") {
      const map: Record<string, RuntimeValue> = {
        linear: { kind: "number", value: obj.linear, unit: "m/s" },
        angular: { kind: "number", value: obj.angular, unit: "rad/s" },
      };
      return map[expr.property] ?? { kind: "void" };
    }

    // continue when kind equals property === "text".
    if (obj.kind === "goal" && expr.property === "text") {
      return { kind: "string", value: obj.text };
    }

    // continue when kind equals property === "goal".
    if (obj.kind === "agent" && expr.property === "goal") {
      const agent = this.agents.get(obj.name);
      return { kind: "goal", text: agent?.decl.goal ?? "" };
    }

    // continue when kind equals property === "text".
    if (obj.kind === "completion" && expr.property === "text") {
      return { kind: "string", value: obj.text };
    }

    // continue when kind equals "object".
    if (obj.kind === "object") {
      return obj.fields[expr.property] ?? { kind: "void" };
    }
    return { kind: "void" };
}

  private evalStringRegexMethod(
    method: string,
    text: string,
    expr: import("../ast/nodes.js").CallExpr,
  ): RuntimeValue {
    // Dispatch string regex methods against compiled patterns.
    const patternArg = expr.args[0] ? this.evalExpr(expr.args[0]) : { kind: "void" as const };
    if (patternArg.kind !== "regex") {
      throw new RuntimeError("Regex method requires a regex pattern argument", expr.span.start.line);
    }
    const pattern = patternArg.pattern;
    switch (method) {
      case "matches":
        return { kind: "bool", value: regexMatches(pattern, text) };
      case "find": {
        const found = regexFind(pattern, text);
        return found === null ? { kind: "void" } : { kind: "string", value: found };
      }
      case "replace": {
        const replacement = expr.args[1]
          ? getString(this.evalExpr(expr.args[1]))
          : "";
        return { kind: "string", value: regexReplace(pattern, text, replacement) };
      }
      case "split":
        return {
          kind: "object",
          typeName: "StringList",
          fields: Object.fromEntries(
            regexSplit(pattern, text).map((part, index) => [
              String(index),
              { kind: "string" as const, value: part },
            ]),
          ),
        };
      case "capture": {
        const cap = regexCapture(pattern, text);
        if (!cap) {
          return { kind: "void" };
        }
        return { kind: "capture", result: cap };
      }
      default:
        throw new RuntimeError(`Unknown string method '${method}'`, expr.span.start.line);
    }
  }

  private sleepWallMs(ms: number): void {
    if (ms <= 0) {
      return;
    }
    const end = performance.now() + ms;
    while (performance.now() < end) {
      /* spin until wall deadline */
    }
  }

  private evalCall(expr: import("../ast/nodes.js").CallExpr): RuntimeValue {
    // EvalCall.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = evalCall(expr);
    if (expr.callee.kind === "IdentExpr") {
      const calleeName = expr.callee.name;
      const moduleFn =
        this.moduleFunctions.get(calleeName) ?? this.importedFunctions.get(calleeName);

      // continue when moduleFn.
      if (moduleFn) {
        return this.callModuleFunction(moduleFn, expr.args);
      }
      const externFn = this.currentProgram?.externFunctions.find(
        (decl) => decl.name === calleeName,
      );

      // continue when bridge equals bridge === "cpp").
      if (externFn && (externFn.bridge === "python" || externFn.bridge === "cpp")) {
        const args = expr.args.map((arg) => this.evalExpr(arg));
        return callExternBridge(externFn, args);
      }
      const enumName = this.variantOwner.get(calleeName);

      // continue when enumName.
      if (enumName) {
        const payloads = expr.args.map((arg) => this.evalExpr(arg));
        return { kind: "enum", enumName, variant: calleeName, payloads };
      }
      return this.evalBuiltinFunction(calleeName, expr);
    }

    // continue when kind differs from kind !== "IdentExpr".
    if (expr.callee.kind !== "MemberExpr" || expr.callee.object.kind !== "IdentExpr") {
      return { kind: "void" };
    }
    const targetName = expr.callee.object.name;
    const method = expr.callee.property;
    const target = this.env.get(targetName);

    // continue when target is falsy.
    if (!target) {
      throw new RuntimeError(`Undefined '${targetName}'`, expr.span.start.line);
    }

    if (target.kind === "string") {
      return this.evalStringRegexMethod(method, target.value, expr);
    }

    // continue when kind equals "robot" || targetName === "robot".
    if (target.kind === "robot" || targetName === "robot") {
      return this.evalRobotMethod(method, expr);
    }

    // continue when kind equals "twin".
    if (target.kind === "twin") {
      return this.evalTwinMethod(method, expr);
    }

    if (target.kind === "mission_control") {
      return this.evalMissionMethod(target.runtime, method, expr.span.start.line);
    }

    if (target.kind === "navigation_control") {
      return this.evalNavigationMethod(target, method, expr);
    }

    if (target.kind === "slam_control") {
      return this.evalSlamMethod(method, expr.span.start.line);
    }

    if (target.kind === "fleet_control") {
      return this.evalFleetMethod(target.registry, method, expr);
    }

    // continue when kind equals "sensor fusion".
    if (target.kind === "sensor_fusion") {

      // continue when method equals "read".
      if (method === "read") {
        return this.readFusedObservation();
      }
    }

    // continue when kind equals "sensor".
    if (target.kind === "sensor") {

      // continue when method equals "read".
      if (method === "read") {

        // continue when this.currentAgent.
        if (this.currentAgent) {
          this.checkAgentCapability(this.currentAgent, "read", targetName, expr.span.start.line);
        }
        return this.readSensorValue(target);
      }

      // continue when sensorType equals "Camera".
      if (target.sensorType === "Camera") {

        // continue when method equals "frame") return mockCameraFrame.
        if (method === "frame") return mockCameraFrame();

        // continue when method equals "analyze".
        if (method === "analyze") {
          const frame = mockCameraFrame();
          return mockAnalyzeFrame(frame, target.name);
        }
      }
    }

    // continue when kind equals "trait object".
    if (target.kind === "trait_object") {
      const agentName = target.agent;
      const traitImpl = this.agentTraitImpls.get(agentName)?.get(method);

      // continue when traitImpl.
      if (traitImpl) {
        const saved = this.env.clone();

        // Loop with index variable i.
        for (let i = 0; i < traitImpl.params.length; i++) {
          const param = traitImpl.params[i];
          const argVal = expr.args[i] ? this.evalExpr(expr.args[i]) : { kind: "void" as const };
          this.env.define(param.name, argVal);
        }
        this.currentAgent = agentName;

        // Try the operation and handle failures below.
        try {
          this.executeBlock(traitImpl.body);
        } finally {
          this.currentAgent = null;
          this.env = saved;
        }
        this.options.onLog?.(`dyn ${target.traitName}@${agentName}.${method}()`);
        return { kind: "void" };
      }
      throw new RuntimeError(
        `Unknown trait method '${method}' on dyn ${target.traitName}`,
        expr.span.start.line,
      );
    }

    // continue when kind equals "agent".
    if (target.kind === "agent") {
      const traitImpl = this.agentTraitImpls.get(targetName)?.get(method);

      // continue when traitImpl.
      if (traitImpl) {
        const saved = this.env.clone();

        // Loop with index variable i.
        for (let i = 0; i < traitImpl.params.length; i++) {
          const param = traitImpl.params[i];
          const argVal = expr.args[i] ? this.evalExpr(expr.args[i]) : { kind: "void" as const };
          this.env.define(param.name, argVal);
        }
        this.currentAgent = targetName;

        // Try the operation and handle failures below.
        try {
          this.executeBlock(traitImpl.body);
        } finally {
          this.currentAgent = null;
          this.env = saved;
        }
        this.options.onLog?.(`agent ${targetName}.${method}()`);
        return { kind: "void" };
      }

      // continue when method equals "plan".
      if (method === "plan") {
        this.checkAgentCapability(targetName, "plan", undefined, expr.span.start.line);
        const agent = this.agents.get(targetName);

        // continue when agent is falsy.
        if (!agent) {
          throw new RuntimeError(`Unknown agent '${targetName}'`, expr.span.start.line);
        }
        this.currentAgent = targetName;

        // Try the operation and handle failures below.
        try {
          executeAgentPlan(agent, { executeBlock: (stmts) => this.executeBlock(stmts) });
        } finally {
          this.currentAgent = null;
        }
        this.options.onLog?.(`agent ${targetName}.plan()`);
        return { kind: "void" };
      }
    }

    // continue when kind equals "safety ctx" && method === "validate".
    if (target.kind === "safety_ctx" && method === "validate") {
      return this.evalSafetyValidate(expr);
    }
    const aiModel = this.ai_models.get(targetName);

    // continue when kind equals "ai model".
    if (aiModel || target.kind === "ai_model") {
      const model = aiModel ?? this.ai_models.get(targetName);

      // continue when model is falsy.
      if (!model) return { kind: "void" };

      // continue when method equals "reason".
      if (method === "reason") {

        // continue when this.currentAgent.
        if (this.currentAgent) {
          this.checkAgentCapability(this.currentAgent, "propose_motion", undefined, expr.span.start.line);
        }
        const prompt = getString(this.getNamedArgValue(expr, "prompt"));
        const input = this.getNamedArgValue(expr, "input");
        const goalText = this.enrichReasonGoal(this.resolveReasonGoal(expr));
        const result = model.reason(
          prompt,
          input.kind === "void" ? undefined : input,
          goalText,
        );
        this.options.onLog?.(`ai ${targetName}.reason() -> ActionProposal`);
        return result;
      }

      // continue when method equals "summarize".
      if (method === "summarize") {

        // continue when this.currentAgent.
        if (this.currentAgent) {
          this.checkAgentCapability(this.currentAgent, "summarize", undefined, expr.span.start.line);
        }
        const input = this.getNamedArgValue(expr, "input");
        return model.summarize(input.kind === "void" ? undefined : input);
      }

      // continue when method equals "detect".
      if (method === "detect") {

        // continue when this.currentAgent.
        if (this.currentAgent) {
          this.checkAgentCapability(this.currentAgent, "detect", undefined, expr.span.start.line);
        }
        const frame = expr.args[0] ? this.evalExpr(expr.args[0]) : this.getNamedArgValue(expr, "frame");
        return model.detect(frame);
      }

      // continue when method equals "drive".
      if (method === "drive") {
        throw new RuntimeError(
          "Unsafe AI action: LLM cannot drive actuators directly — use safety.validate() then wheels.execute()",
          expr.span.start.line,
        );
      }
    }

    // continue when kind equals "actuator".
    if (target.kind === "actuator") {
      return this.executeActuatorMethod(target.name, target.actuatorType, method, expr);
    }
    return { kind: "void" };
}

  private readSensorValue(target: Extract<RuntimeValue, { kind: "sensor" }>): RuntimeValue {
    // ReadSensorValue.
    //
    // Parameters:
    // - `target` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = readSensorValue(target);
    const state = this.options.backend.getState();
    let reading: RuntimeValue;

    // continue when target.library.
    if (target.library) {
      const driver = getSensorDriver(target.library, target.sensorType);

      // continue when driver.
      if (driver) {
        reading = readWithDriver(driver, {
          hal: this.hal,
          halBinding: target.halBinding ?? null,
          topic: target.topic ?? null,
          simState: { pose: state.pose },
        });
      } else {
        reading = this.options.backend.readSensor(target.name, target.sensorType, target.topic);
      }
    } else {
      reading = this.options.backend.readSensor(target.name, target.sensorType, target.topic);
    }

    if (target.sensorType === "GPS" || target.sensorType === "GNSS") {
      if (reading.kind === "object") {
        const { lat, lon, fixQuality } = applyGpsPositionFaults(
          this.injectedFaults,
          state.pose.x,
          state.pose.y,
          this.reliability.simTimeMs,
        );
        reading = {
          ...reading,
          fields: {
            ...reading.fields,
            lat: { kind: "number", value: lat, unit: "none" },
            lon: { kind: "number", value: lon, unit: "none" },
            ...(reading.fields.fix_quality
              ? { fix_quality: { kind: "number", value: fixQuality, unit: "none" } }
              : {}),
          },
        };
      }
    }
    return reading;
  }

  private readFusedObservation(): RuntimeValue {
    // ReadFusedObservation.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = readFusedObservation();
    const fields: Record<string, RuntimeValue> = {};

    // Process each fusionSensor.
    for (const sensorName of this.fusionSensors) {
      const sensorVal = this.env.get(sensorName);

      // continue when kind differs from "sensor".
      if (!sensorVal || sensorVal.kind !== "sensor") {
        throw new RuntimeError(`Unknown observe sensor '${sensorName}'`, 0);
      }
      fields[sensorName] = this.readSensorValue(sensorVal);
    }
    const state = this.options.backend.getState();
    fields.pose = poseFromState(state.pose);
    fields.count = { kind: "number", value: this.fusionSensors.length, unit: "none" };
    const confidence = this.fusionSensors.length === 0
      ? 0
      : Math.min(this.fusionSensors.length / 4, 1);
    fields.confidence = { kind: "number", value: confidence, unit: "none" };
    fields.state_estimate = {
      kind: "object",
      typeName: "StateEstimate",
      fields: {
        pose: fields.pose,
        confidence: fields.confidence,
      },
    };
    return { kind: "object", typeName: "FusedObservation", fields };
}

  private evalMissionMethod(
    runtime: MissionRuntime,
    method: string,
    line: number,
  ): RuntimeValue {
    // Dispatch mission lifecycle methods on the active mission controller.
    switch (method) {
      case "start":
        missionStart(runtime);
        return { kind: "string", value: runtime.state };
      case "pause":
        missionPause(runtime);
        return { kind: "string", value: runtime.state };
      case "resume":
        missionResume(runtime);
        return { kind: "string", value: runtime.state };
      case "advance":
        return { kind: "string", value: missionAdvance(runtime) };
      case "complete":
        missionComplete(runtime);
        return { kind: "string", value: runtime.state };
      case "fail":
        missionFail(runtime);
        return { kind: "string", value: runtime.state };
      case "state":
        return { kind: "string", value: runtime.state };
      case "step":
        return { kind: "string", value: missionCurrentStep(runtime) };
      default:
        throw new RuntimeError(`Unknown mission method '${method}'`, line);
    }
  }

  private executeNavigateStmt(stmt: import("../ast/nodes.js").NavigateStmt): void {
    // Execute navigate { goal: ... } sugar over navigation.goal/navigate.
    const line = stmt.span.start.line;
    const goalText = getString(this.evalExpr(stmt.goal));
    if (!goalText) {
      throw new RuntimeError("navigate.goal requires a text or numeric expression", line);
    }

    const nav = this.env.get("navigation");
    if (!nav || nav.kind !== "navigation_control") {
      throw new RuntimeError("navigate statement requires a robot with a declared mission", line);
    }
    nav.goal = goalText;

    const linearVal = stmt.linear ? this.evalExpr(stmt.linear) : null;
    const angularVal = stmt.angular ? this.evalExpr(stmt.angular) : null;
    const linearMps =
      linearVal?.kind === "number" ? linearVal.value : 0.2;
    const angularRad =
      angularVal?.kind === "number" ? angularVal.value : 0.0;

    this.options.onLog?.(`navigation: executing goal '${goalText}'`);
    const bridgeOutput = invokeNav2Bridge(goalText);
    if (bridgeOutput) {
      this.options.onLog?.(`navigation: Nav2 bridge output: ${bridgeOutput}`);
    }
    tryPublishNav2CmdVel({
      backend: this.options.backend,
      topicPathToMessageType: this.topicPathToMessageType,
      goal: goalText,
      linearMps,
      angularRadS: angularRad,
      onLog: (message) => this.options.onLog?.(message),
    });
  }

  private evalSlamMethod(method: string, line: number): RuntimeValue {
    // Dispatch SLAM adapter helpers for map and localization stubs.
    if (method === "localize") {
      const state = this.options.backend.getState();
      const bridgeOutput = invokeSlamBridge("localize");
      if (bridgeOutput) {
        this.options.onLog?.(`slam: bridge localize output: ${bridgeOutput}`);
      } else {
        this.options.onLog?.("slam: localize (stub adapter)");
      }
      return {
        kind: "object",
        typeName: "LocalizationEstimate",
        fields: {
          pose: poseFromState(state.pose),
          confidence: { kind: "number", value: 0.85, unit: "none" },
        },
      };
    }
    if (method === "map") {
      const bridgeOutput = invokeSlamBridge("map");
      if (bridgeOutput) {
        this.options.onLog?.(`slam: bridge map output: ${bridgeOutput}`);
      } else {
        this.options.onLog?.("slam: map snapshot (stub adapter)");
      }
      return {
        kind: "object",
        typeName: "OccupancyGrid",
        fields: {
          resolution: { kind: "number", value: 0.05, unit: "m" },
          width: { kind: "number", value: 100, unit: "none" },
        },
      };
    }
    throw new RuntimeError(`Unknown slam method '${method}'`, line);
  }

  private evalNavigationMethod(
    target: { kind: "navigation_control"; goal: string | null },
    method: string,
    expr: import("../ast/nodes.js").CallExpr,
  ): RuntimeValue {
    // Dispatch navigation helpers for goals, paths, and cost maps.
    const line = expr.span.start.line;
    switch (method) {
      case "goal": {
        const textArg = expr.args[0]
          ? this.evalExpr(expr.args[0])
          : this.getNamedArgValue(expr, "text");
        const text = getString(textArg);
        if (!text) {
          throw new RuntimeError("navigation.goal() requires a text argument", line);
        }
        target.goal = text;
        return {
          kind: "object",
          typeName: "NavigationGoal",
          fields: { text: { kind: "string", value: text } },
        };
      }
      case "path": {
        const state = this.options.backend.getState();
        const from = {
          x: state.pose.x,
          y: state.pose.y,
          theta: state.pose.theta,
          z: state.pose.z ?? 0,
        };
        const to = {
          x: state.pose.x + 1,
          y: state.pose.y,
          theta: state.pose.theta,
          z: state.pose.z ?? 0,
        };
        const waypoints = getTrajectoryWaypoints(
          runtimeTrajectory(interpolatePoses(from, to, 2)),
        ) ?? [];
        return {
          kind: "object",
          typeName: "Path",
          fields: {
            waypoints: { kind: "number", value: waypoints.length, unit: "none" },
          },
        };
      }
      case "cost_map":
        return {
          kind: "object",
          typeName: "CostMap",
          fields: {
            resolution: { kind: "number", value: 0.05, unit: "m" },
          },
        };
      case "navigate": {
        this.options.onLog?.(
          `navigation: executing goal '${target.goal ?? "none"}'`,
        );
        const bridgeOutput = invokeNav2Bridge(target.goal ?? "none");
        if (bridgeOutput) {
          this.options.onLog?.(`navigation: Nav2 bridge output: ${bridgeOutput}`);
        }
        tryPublishNav2CmdVel({
          backend: this.options.backend,
          topicPathToMessageType: this.topicPathToMessageType,
          goal: target.goal,
          onLog: (message) => this.options.onLog?.(message),
        });
        return {
          kind: "object",
          typeName: "Trajectory",
          fields: {
            status: { kind: "string", value: "executing" },
          },
        };
      }
      default:
        throw new RuntimeError(`Unknown navigation method '${method}'`, line);
    }
  }

  private evalFleetMethod(
    registry: FleetRegistry,
    method: string,
    expr: import("../ast/nodes.js").CallExpr,
  ): RuntimeValue {
    // Dispatch fleet coordination helpers for member lookup.
    const line = expr.span.start.line;
    switch (method) {
      case "members": {
        const fleetName = expr.args[0]
          ? getString(this.evalExpr(expr.args[0]))
          : "";
        if (!fleetName) {
          throw new RuntimeError("fleet.members() requires a fleet name", line);
        }
        const members = registry.members(fleetName) ?? [];
        return { kind: "number", value: members.length, unit: "none" };
      }
      case "names":
        return { kind: "number", value: registry.names().length, unit: "none" };
      default:
        throw new RuntimeError(`Unknown fleet method '${method}'`, line);
    }
  }

  private evalBuiltinFunction(name: string, expr: import("../ast/nodes.js").CallExpr): RuntimeValue {
    // EvalBuiltinFunction.
    //
    // Parameters:
    // - `name` — input value
    // - `expr` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = evalBuiltinFunction(name, expr);
    switch (name) {
      case "pose":
        return runtimePose(
          this.getNamedArgNumber(expr, "x", 0),
          this.getNamedArgNumber(expr, "y", 0),
          this.getNamedArgNumber(expr, "theta", 0),
          this.getNamedArgNumber(expr, "z", 0),
        );
      case "velocity":
        return runtimeVelocity(
          this.getNamedArgNumber(expr, "linear", 0),
          this.getNamedArgNumber(expr, "angular", 0),
        );
      case "trajectory": {
        const fromVal = this.getNamedArgValue(expr, "from");
        const toVal = this.getNamedArgValue(expr, "to");
        const steps = this.getNamedArgNumber(expr, "steps", 5);
        const from = getPoseFields(fromVal) ?? { x: 0, y: 0, theta: 0, z: 0 };
        const to = getPoseFields(toVal) ?? { x: 0, y: 0, theta: 0, z: 0 };
        return runtimeTrajectory(interpolatePoses(from, to, steps));
      }
      case "transform": {
        const fromFrame = getString(this.getNamedArgValue(expr, "from"), "base");
        const toFrame = getString(this.getNamedArgValue(expr, "to"), "map");
        const pose = getPoseFields(this.getNamedArgValue(expr, "pose")) ?? { x: 0, y: 0, theta: 0, z: 0 };
        return { kind: "transform", fromFrame, toFrame, pose };
      }
      case "goal": {
        const arg0 = expr.args[0];
        const text =
          arg0?.kind === "LiteralExpr" && typeof arg0.value === "string"
            ? arg0.value
            : getString(this.getNamedArgValue(expr, "text"), "");
        return { kind: "goal", text };
      }
      case "recall": {

        // continue when currentAgent is falsy.
        if (!this.currentAgent) {
          throw new RuntimeError(
            "recall() requires active agent context (run inside agent plan)",
            expr.span.start.line,
          );
        }
        const agent = this.agents.get(this.currentAgent);

        // continue when memory is falsy.
        if (!agent?.memory) {
          throw new RuntimeError(
            "Agent has no memory — declare memory short_term or long_term on the agent",
            expr.span.start.line,
          );
        }
        const arg0 = expr.args[0];
        const key =
          arg0?.kind === "LiteralExpr" && typeof arg0.value === "string"
            ? arg0.value
            : getString(this.getNamedArgValue(expr, "key"), "");
        const entry = agent.memory.recall(key);
        return entry ?? { kind: "void" };
      }
      case "assert": {
        const arg0 = expr.args[0];

        // continue when arg0 is falsy.
        if (!arg0) {
          throw new RuntimeError("assert requires a boolean condition", expr.span.start.line);
        }
        const cond = this.evalExpr(arg0);

        // continue when kind differs from "bool".
        if (cond.kind !== "bool") {
          throw new RuntimeError("assert requires a boolean condition", expr.span.start.line);
        }

        // continue when value is falsy.
        if (!cond.value) {
          throw new RuntimeError("Assertion failed", expr.span.start.line);
        }
        return { kind: "void" };
      }
      case "channel":
        return this.concurrency.createChannel();
      case "send": {
        const channel = expr.args[0] ? this.evalExpr(expr.args[0]) : this.getNamedArgValue(expr, "channel");
        const value = expr.args[1] ? this.evalExpr(expr.args[1]) : this.getNamedArgValue(expr, "value");
        this.concurrency.bindChannelType(channel, value, expr.span.start.line);
        this.concurrency.send(channel, value, expr.span.start.line);
        return { kind: "void" };
      }
      case "recv": {
        const channel = expr.args[0] ? this.evalExpr(expr.args[0]) : this.getNamedArgValue(expr, "channel");
        return this.concurrency.tryRecv(channel, expr.span.start.line) ?? { kind: "void" };
      }
      case "join": {
        const handle = expr.args[0] ? this.evalExpr(expr.args[0]) : { kind: "void" as const };

        // continue when kind equals "future".
        if (handle.kind === "future") {
          return this.resolveFuture(handle, expr.span.start.line);
        }

        // continue when kind equals "task handle".
        if (handle.kind === "task_handle") {
          return this.resolveTaskHandle(handle.id, expr.span.start.line);
        }
        throw new RuntimeError("join requires a Future or TaskHandle value", expr.span.start.line);
      }
      case "send_agent": {

        // continue when currentAgent is falsy.
        if (!this.currentAgent) {
          throw new RuntimeError("send_agent requires active agent context", expr.span.start.line);
        }
        const to = expr.args[0]
          ? getString(this.evalExpr(expr.args[0]), "")
          : getString(this.getNamedArgValue(expr, "to"), "");
        const value = expr.args[1] ? this.evalExpr(expr.args[1]) : this.getNamedArgValue(expr, "value");
        this.concurrency.sendAgent(this.currentAgent, to, value, expr.span.start.line);
        this.options.onLog?.(`send_agent ${this.currentAgent} -> ${to}`);
        return { kind: "void" };
      }
      case "recv_agent": {

        // continue when currentAgent is falsy.
        if (!this.currentAgent) {
          throw new RuntimeError("recv_agent requires active agent context", expr.span.start.line);
        }
        return this.concurrency.tryRecvAgent(this.currentAgent) ?? { kind: "void" };
      }
      case "peer_send": {
        const peer = expr.args[0]
          ? getString(this.evalExpr(expr.args[0]), "")
          : getString(this.getNamedArgValue(expr, "peer"), "");
        const topic = expr.args[1]
          ? getString(this.evalExpr(expr.args[1]), "")
          : getString(this.getNamedArgValue(expr, "topic"), "");
        const value = expr.args[2] ? this.evalExpr(expr.args[2]) : this.getNamedArgValue(expr, "value");
        this.commBus.publishPeer(peer, topic, value, this.defaultTransport);
        this.options.onLog?.(`peer_send ${peer}.${topic}`);
        return { kind: "void" };
      }
      default:
        return { kind: "void" };
    }
}

  private evalSpawnTarget(
    callee: Expr,
    args: Expr[],
    line: number,
  ): {
    // EvalSpawnTarget.
    //
    // Parameters:
    // - `callee` — input value
    // - `args` — input value
    // - `line` — input value
    //
    // Returns:
    // .
    //
    // Options:
    // None.
    //
    // Example:

 // const result = evalSpawnTarget(callee, args, line);
 funcName: string; args: RuntimeValue[] } {
    const argValues = args.map((arg) => this.evalExpr(arg));
    if (callee.kind !== "IdentExpr") {
      throw new RuntimeError("spawn requires function name", line);
    }
    return { funcName: callee.name, args: argValues };
  }

  private executeSpawnJob(funcName: string, args: RuntimeValue[], line: number): RuntimeValue {
    // ExecuteSpawnJob.
    //
    // Parameters:
    // - `funcName` — input value
    // - `args` — input value
    // - `line` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = executeSpawnJob(funcName, args, line);
    const func =
      this.moduleFunctions.get(funcName) ?? this.importedFunctions.get(funcName);

    // continue when func is falsy.
    if (!func) {
      throw new RuntimeError(`Unknown spawn target '${funcName}'`, line);
    }
    const saved = this.env.clone();

    // Loop with index variable i.
    for (let i = 0; i < func.params.length; i++) {
      const param = func.params[i];
      const val = args[i];

      // continue when param && val) this.env.define(param.name, val.
      if (param && val) this.env.define(param.name, val);
    }
    const result = this.executeBlockWithReturn(func.body);
    this.env = saved;
    return result;
}

  private resolveTaskHandle(id: number, line: number): RuntimeValue {
    // ResolveTaskHandle.
    //
    // Parameters:
    // - `id` — input value
    // - `line` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = resolveTaskHandle(id, line);
    const handle = this.concurrency.getHandle(id);

    // continue when handle is falsy.
    if (!handle) {
      throw new RuntimeError(`Unknown task handle ${id}`, line);
    }

    // continue when handle.result.
    if (handle.result) return handle.result;
    const result = this.executeSpawnJob(handle.funcName, handle.args, line);
    this.concurrency.setHandleResult(id, result);
    return result;
}

  private resolveFuture(future: RuntimeValue, line: number): RuntimeValue {
    // ResolveFuture.
    //
    // Parameters:
    // - `future` — input value
    // - `line` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = resolveFuture(future, line);
    if (future.kind === "future") {

      // continue when future.resolved.
      if (future.resolved) return future.resolved;
      const result = this.executeSpawnJob(future.funcName, future.args, line);
      return result;
    }
    return future;
}

  private processSpawnQueue(): void {
    // ProcessSpawnQueue.
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

    // const result = processSpawnQueue();
    for (const id of this.concurrency.drainFireAndForgetQueue()) {
      this.resolveTaskHandle(id, 0);
    }
}

  private goalTextFromValue(value: RuntimeValue): string | undefined {
    // GoalTextFromValue.
    //
    // Parameters:
    // - `value` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = goalTextFromValue(value);
    if (value.kind === "goal") return value.text;

    // continue when kind equals "string".
    if (value.kind === "string") return value.value;
    return undefined;
}

  private resolveReasonGoal(expr: import("../ast/nodes.js").CallExpr): string | undefined {    // Compute explicit for the following logic.
    const explicit = this.getNamedArgValue(expr, "goal");

    // continue when kind differs from "void".
    if (explicit.kind !== "void") {
      return this.goalTextFromValue(explicit);
    }

    // continue when this.currentAgent.
    if (this.currentAgent) {
      const agent = this.agents.get(this.currentAgent);
      const text = agent?.decl.goal?.trim();

      // continue when text.
      if (text) return text;
    }
    return undefined;
}

  private enrichReasonGoal(goalText: string | undefined): string | undefined {
    // EnrichReasonGoal.
    //
    // Parameters:
    // - `goalText` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = enrichReasonGoal(goalText);
    if (!this.currentAgent) return goalText;
    const agent = this.agents.get(this.currentAgent);
    const memorySummary = agent?.memory?.summaryForPrompt();

    // continue when memorySummary is falsy.
    if (!memorySummary) return goalText;

    // continue when goalText.
    if (goalText) {
      return `${goalText}\n${memorySummary}`;
    }
    return memorySummary;
}

  private evalSafetyValidate(expr: import("../ast/nodes.js").CallExpr): RuntimeValue {
    // EvalSafetyValidate.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = evalSafetyValidate(expr);
    const arg = expr.args[0] ? this.evalExpr(expr.args[0]) : this.getNamedArgValue(expr, "proposal");
    const proposal = proposalFromValue(arg);

    // continue when proposal is falsy.
    if (!proposal) {
      throw new RuntimeError("safety.validate() expects ActionProposal", expr.span.start.line);
    }
    const state = this.options.backend.getState();
    const result = this.safetyMonitor?.validateActionProposal(
      proposal.linear,
      proposal.angular,
      this.env,
      state.pose,
    );

    // continue when ok is falsy.
    if (!result?.ok) {
      throw new RuntimeError(result?.reason ?? "Safety validation failed for AI action", expr.span.start.line);
    }
    this.options.onLog?.("safety.validate() approved ActionProposal");
    return safeActionFromProposal(result.linear, result.angular);
}

  private evalRobotMethod(method: string, expr: import("../ast/nodes.js").CallExpr): RuntimeValue {
    // EvalRobotMethod.
    //
    // Parameters:
    // - `method` — input value
    // - `expr` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = evalRobotMethod(method, expr);
    const state = this.options.backend.getState();

    // Branch on method.
    switch (method) {
      case "pose":
        return poseFromState(state.pose);
      case "velocity":
        return velocityFromState(state.velocity);
      case "in_zone": {
        const zoneName = expr.args[0] ? getString(this.evalExpr(expr.args[0])) : "";
        const inZone = this.safetyMonitor?.isInZone(zoneName, state.pose) ?? false;
        return { kind: "bool", value: inZone };
      }
      case "in_geofence": {
        const fenceName = expr.args[0] ? getString(this.evalExpr(expr.args[0])) : "";
        const [lat, lon] = this.currentGpsLatLon();
        const inside = this.geofences.some(
          (f) => f.name === fenceName && geofenceContains(f, lat, lon),
        );
        return { kind: "bool", value: inside };
      }
      case "connectivity_link": {
        return { kind: "string", value: this.activeConnectivityLink };
      }
      case "sim_identity": {
        this.security.requireOperation("cellular.sim_identity");
        const modemActive = isModemBearer(this.activeConnectivityLink);
        const outage = this.commBus
          .activeFaults()
          .some((f) => f === "LteOutage" || f === "SatelliteOutage" || f === "NetworkOutage");
        return runtimeSimIdentity(this.activeConnectivityLink, modemActive && !outage);
      }
      default:
        return { kind: "void" };
    }
}

  private executeActuatorMethod(
    name: string,
    actuatorType: string,
    method: string,
    expr: import("../ast/nodes.js").CallExpr,
  ): RuntimeValue {
    // ExecuteActuatorMethod.
    //
    // Parameters:
    // - `name` — input value
    // - `actuatorType` — input value
    // - `method` — input value
    // - `expr` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = executeActuatorMethod(name, actuatorType, method, expr);
    const motionMethods = ["drive", "move_to", "set_thrust", "grip", "release", "open", "hover", "follow"];

    // continue when includes equals "stop".
    if (motionMethods.includes(method) || method === "stop") {

      // continue when checkSafetyBeforeMotion is falsy.
      if (!this.checkSafetyBeforeMotion()) {
        this.options.onMotionBlocked?.("Safety rule triggered — motion blocked");
        this.options.backend.executeMotion({ kind: "stop", actuator: name });
        return { kind: "void" };
      }
    }

    // Branch on method.
    switch (method) {
      case "stop":
        this.options.backend.executeMotion({ kind: "stop", actuator: name });
        break;
      case "drive": {
        const linear = this.getNamedArgNumber(expr, "linear", 0);
        const angular = this.getNamedArgNumber(expr, "angular", 0);
        const pose = this.options.backend.getState().pose;
        const maxSpeed = this.safetyMonitor?.clampSpeedAtPose(linear, pose) ?? linear;
        this.options.backend.executeMotion({ kind: "drive", linear: maxSpeed, angular, actuator: name });
        break;
      }
      case "follow": {
        const pathVal = this.getNamedArgValue(expr, "path");
        const waypoints = getTrajectoryWaypoints(pathVal) ?? [];
        this.options.backend.executeMotion({ kind: "follow", waypoints, actuator: name });
        break;
      }
      case "move_to":
        this.options.backend.executeMotion({
          kind: "move_to",
          x: this.getNamedArgNumber(expr, "x", 0),
          y: this.getNamedArgNumber(expr, "y", 0),
          z: this.getNamedArgNumber(expr, "z", 0),
          actuator: name,
        });
        break;
      case "grip":
        this.options.backend.executeMotion({ kind: "grip", actuator: name });
        break;
      case "release":
        this.options.backend.executeMotion({ kind: "release", actuator: name });
        break;
      case "open":
        this.options.backend.executeMotion({ kind: "open", actuator: name });
        break;
      case "set_thrust":
        this.options.backend.executeMotion({
          kind: "set_thrust",
          thrust: this.getNamedArgNumber(expr, "thrust", 0),
          actuator: name,
        });
        break;
      case "hover":
        this.options.backend.executeMotion({ kind: "hover", actuator: name });
        break;
      case "execute": {

        // continue when this.currentAgent.
        if (this.currentAgent) {
          this.checkAgentCapability(this.currentAgent, "propose_motion", undefined, expr.span.start.line);
        }
        const actionVal = expr.args[0]
          ? this.evalExpr(expr.args[0])
          : this.getNamedArgValue(expr, "action");

        // continue when isSafeAction is falsy.
        if (!isSafeAction(actionVal)) {

          // continue when isActionProposal(actionVal).
          if (isActionProposal(actionVal)) {
            throw new RuntimeError(
              "Unsafe AI action: ActionProposal cannot execute actuators — call safety.validate() first",
              expr.span.start.line,
            );
          }
          throw new RuntimeError(
            "Actuator execute() requires SafeAction from safety.validate()",
            expr.span.start.line,
          );
        }
        const safe = actionVal;

        // continue when checkSafetyBeforeMotion is falsy.
        if (!this.checkSafetyBeforeMotion()) {
          this.options.onMotionBlocked?.("Safety rule triggered — motion blocked");
          this.options.backend.executeMotion({ kind: "stop", actuator: name });
          return { kind: "void" };
        }
        this.options.backend.executeMotion({
          kind: "drive",
          linear: safe.linear,
          angular: safe.angular,
          actuator: name,
        });
        break;
      }
    }
    this.options.onLog?.(`${name}.${method}()`);
    return { kind: "void" };
}

  private getNamedArgValue(expr: import("../ast/nodes.js").CallExpr, name: string): RuntimeValue {    // Compute arg for the following logic.
    const arg = expr.namedArgs.find((a) => a.name === name);
    return arg ? this.evalExpr(arg.value) : { kind: "void" };
}

  private getNamedArgNumber(expr: import("../ast/nodes.js").CallExpr, name: string, defaultVal: number): number {    // Return getNamedArgValue to the caller.
    return getNumber(this.getNamedArgValue(expr, name), defaultVal);
}

  private evalBinary(op: string, left: RuntimeValue, right: RuntimeValue): RuntimeValue {
    // EvalBinary.
    //
    // Parameters:
    // - `op` — input value
    // - `left` — input value
    // - `right` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = evalBinary(op, left, right);
    if (op === "and") {
      return { kind: "bool", value: (left.kind === "bool" && left.value) && (right.kind === "bool" && right.value) };
    }

    // continue when op equals "or".
    if (op === "or") {
      return { kind: "bool", value: (left.kind === "bool" && left.value) || (right.kind === "bool" && right.value) };
    }

    // continue when kind equals kind === "bool".
    if (left.kind === "bool" && right.kind === "bool") {

      // continue when op equals "==".
      if (op === "==") return { kind: "bool", value: left.value === right.value };

      // continue when op equals "!=".
      if (op === "!=") return { kind: "bool", value: left.value !== right.value };
    }

    // continue when kind equals kind === "number".
    if (left.kind === "number" && right.kind === "number") {
      const aligned = alignForBinary(left.value, left.unit, right.value, right.unit);
      const l = aligned?.[0] ?? left.value;
      const r = aligned?.[1] ?? right.value;
      const resultUnit = aligned?.[2] ?? left.unit;

      // Branch on op.
      switch (op) {
        case "+": return { kind: "number", value: l + r, unit: resultUnit };
        case "-": return { kind: "number", value: l - r, unit: resultUnit };
        case "*": return { kind: "number", value: l * r, unit: "none" };
        case "/": return { kind: "number", value: l / r, unit: "none" };
        case "<": return { kind: "bool", value: l < r };
        case "<=": return { kind: "bool", value: l <= r };
        case ">": return { kind: "bool", value: l > r };
        case ">=": return { kind: "bool", value: l >= r };
        case "==": return { kind: "bool", value: l === r };
        case "!=": return { kind: "bool", value: l !== r };
      }
    }
    return { kind: "void" };
}

  private checkSafetyBeforeMotion(): boolean {
    // CheckSafetyBeforeMotion.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkSafetyBeforeMotion();
    const state = this.options.backend.getState();
    const result = this.safetyMonitor?.evaluateBeforeMotion(this.env, state.pose);

    // continue when allowed is falsy.
    if (!result?.allowed) {

      // continue when result?.emergencyStop.
      if (result?.emergencyStop) {
        this.options.backend.setEmergencyStop?.(true);
        this.options.onLog?.(result.reason ?? "Safety blocked motion");
      }
      return false;
    }
    return true;
}
}

export class RuntimeError extends Error {
  constructor(message: string, public line: number) {
    super(message);
    this.name = "RuntimeError";
  }
}
