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
import { RoutingCommBus, type TransportKind } from "../transport/index.js";
import { SafetyMonitor, createSafetyConfigFromRobot, interpolatePoses } from "../safety/index.js";
import type { SafetyZoneRuntime } from "../safety/index.js";
import {
  SecurityContext,
  createRobotIdentity,
  parseTrustLevel,
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
  | { kind: "completion"; text: string; model?: string }
  | { kind: "embedding"; dimensions: number; vector: number[] }
  | { kind: "identity"; id: string; publicKey: string }
  | { kind: "secret"; name: string }
  | { kind: "channel"; id: number }
  | { kind: "task_handle"; id: number }
  | { kind: "future"; funcName: string; args: RuntimeValue[]; resolved: RuntimeValue | null };

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
};

export class Environment {
  private bindings = new Map<string, RuntimeValue>();

  define(name: string, value: RuntimeValue): void {
    this.bindings.set(name, value);
  }

  get(name: string): RuntimeValue | undefined {
    return this.bindings.get(name);
  }

  set(name: string, value: RuntimeValue): void {
    this.bindings.set(name, value);
  }

  remove(name: string): void {
    this.bindings.delete(name);
  }

  clone(): Environment {
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
  private security = new SecurityContext();
  private commBus = new RoutingCommBus();
  private defaultTransport: TransportKind = "local";
  private moduleFunctions = new Map<string, ModuleFnDecl>();
  private importedFunctions = new Map<string, ModuleFnDecl>();
  private externFunctions = new Map<string, ExternFnDecl>();
  private concurrency = new ConcurrencyRuntime();
  private taskMaxDurationMs = new Map<string, number>();
  private returning = false;
  private returnValue: RuntimeValue = { kind: "void" };

  constructor(private options: InterpreterOptions) {}

  run(program: Program, entryBehavior?: string): RobotState {
    this.currentProgram = program;
    this.loadProgramMetadata(program);
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
    this.enumVariants.clear();
    this.variantOwner.clear();
    this.structDefs.clear();
    this.moduleFunctions.clear();
    this.importedFunctions.clear();
    this.externFunctions.clear();

    for (const func of program.functions) {
      if (func.visibility === "export" || func.visibility === "public") {
        this.moduleFunctions.set(func.name, func);
      }
    }
    for (const ext of program.externFunctions) {
      this.externFunctions.set(ext.name, ext);
    }
    if (this.options.moduleRegistry) {
      for (const imp of program.imports) {
        const exports = this.options.moduleRegistry.exportsFor(imp.path);
        if (exports) {
          for (const [name, func] of exports.functions) {
            this.importedFunctions.set(name, func);
          }
        }
      }
    }

    for (const enumDecl of program.enums) {
      const variantNames = enumDecl.variants.map((v) => v.name);
      this.enumVariants.set(enumDecl.name, variantNames);
      for (const variant of enumDecl.variants) {
        this.variantOwner.set(variant.name, enumDecl.name);
      }
    }
    for (const structDecl of program.structs) {
      this.structDefs.set(
        structDecl.name,
        structDecl.fields.map((f) => ({ name: f.name, typeName: f.typeName })),
      );
    }
  }

  private evalContract(expr: Expr): boolean {
    const val = this.evalExpr(expr);
    if (val.kind === "bool") return val.value;
    throw new RuntimeError("Contract expression must be boolean", 0);
  }

  private executeWithContracts(
    body: Stmt[],
    requires: Expr | null,
    ensures: Expr | null,
    invariant: Expr | null,
  ): void {
    if (requires && !this.evalContract(requires)) {
      throw new RuntimeError("requires contract failed", 0);
    }
    this.executeBlock(body);
    if (ensures && !this.evalContract(ensures)) {
      throw new RuntimeError("ensures contract failed", 0);
    }
    if (invariant && !this.evalContract(invariant)) {
      throw new RuntimeError("invariant contract failed", 0);
    }
    this.runVerifyRules();
  }

  private runVerifyRules(): void {
    if (this.verifyRules.length === 0) return;
    for (let i = 0; i < this.verifyRules.length; i++) {
      const val = this.evalExpr(this.verifyRules[i]!);
      if (val.kind !== "bool") {
        throw new RuntimeError(`verify rule ${i + 1} must be boolean`, 0);
      }
      if (!val.value) {
        throw new RuntimeError(`verify rule ${i + 1} failed`, 0);
      }
    }
    this.options.onLog?.(`verify: all ${this.verifyRules.length} rule(s) passed`);
  }

  private executeTaskLoop(task: TaskDecl): void {
    const { body, intervalMs, requires, ensures, invariant, name, priority, budget } = task;
    const maxIter = this.options.maxLoopIterations ?? 10;
    this.options.onLog?.(
      `single-task ${name} interval=${intervalMs}ms priority=${priority}`,
    );
    for (let i = 0; i < maxIter; i++) {
      this.options.backend.tick(intervalMs);
      if (!this.runScheduledTask(name, priority, intervalMs, body, requires, ensures, invariant, budget)) {
        break;
      }
      this.runVerifyRules();
      this.updateTwinSnapshot();
    }
  }

  private priorityRank(priority: TaskPriority): number {
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
    const duty = (durationMs / Math.max(intervalMs, 1)) * 100;
    if (budget.cpuPctMax != null && duty > budget.cpuPctMax) return "cpu";
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
    const RUNTIME_TASK_COST_MS = 5;
    if (budget) {
      const prev = this.taskMaxDurationMs.get(name) ?? 0;
      if (prev > 0) {
        const kind = this.taskBudgetViolation(budget, prev, intervalMs);
        if (kind) {
          this.options.onLog?.(`task '${name}': ${kind} budget exceeded — skipping tick`);
          return true;
        }
      }
    }
    const continueRunning = this.executeTaskIteration(body, requires, ensures, invariant, name);
    const durationMs = RUNTIME_TASK_COST_MS;
    this.taskMaxDurationMs.set(name, Math.max(this.taskMaxDurationMs.get(name) ?? 0, durationMs));
    if (budget) {
      const kind = this.taskBudgetViolation(budget, durationMs, intervalMs);
      if (kind) {
        this.options.onLog?.(`task '${name}': ${kind} budget exceeded (${durationMs.toFixed(2)}ms)`);
      }
    }
    return continueRunning;
  }

  private executeTaskIteration(
    body: Stmt[],
    requires: Expr | null,
    ensures: Expr | null,
    invariant: Expr | null,
    taskName?: string,
  ): boolean {
    if (requires && !this.evalContract(requires)) {
      const label = taskName ? `task '${taskName}'` : "task";
      this.options.onLog?.(`${label} requires contract failed — skipping iteration`);
      return true;
    }
    this.executeBlock(body);
    if (ensures && !this.evalContract(ensures)) {
      throw new RuntimeError("task ensures contract failed", 0);
    }
    if (invariant && !this.evalContract(invariant)) {
      throw new RuntimeError("task invariant contract failed", 0);
    }
    return !this.safetyMonitor?.isEmergencyStop();
  }

  private executeMultiplexedTasks(tasks: TaskDecl[]): void {
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
    for (let i = 0; i < maxIter; i++) {
      this.options.backend.tick(baseTick);
      simTime += baseTick;
      const due = schedules
        .filter((schedule) => schedule.nextDueMs <= simTime)
        .sort((a, b) => this.priorityRank(a.priority) - this.priorityRank(b.priority));
      for (const schedule of due) {
        this.options.onLog?.(`task '${schedule.name}': tick`);
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
      if (this.safetyMonitor?.isEmergencyStop()) break;
    }
  }

  private refreshTwinShadowFromBackend(): void {
    if (!this.twinRuntime) return;
    const state = this.options.backend.getState();
    if (this.twinRuntime.mirrors.includes("pose")) {
      this.twinRuntime.shadow.pose = {
        kind: "pose",
        x: state.pose.x,
        y: state.pose.y,
        theta: state.pose.theta,
        z: state.pose.z ?? 0,
      };
    }
    if (this.twinRuntime.mirrors.includes("velocity")) {
      this.twinRuntime.shadow.velocity = {
        kind: "velocity",
        linear: state.velocity.linear,
        angular: state.velocity.angular,
      };
    }
  }

  private updateTwinSnapshot(): void {
    if (!this.twinRuntime) return;
    this.refreshTwinShadowFromBackend();
    if (this.twinRuntime.replay && Object.keys(this.twinRuntime.shadow).length > 0) {
      this.twinRuntime.replayBuffer.push({ ...this.twinRuntime.shadow });
    }
    const fieldCount = Object.keys(this.twinRuntime.shadow).length;
    if (fieldCount > 0) {
      this.options.onLog?.(
        `twin ${this.twinRuntime.name} mirrored ${fieldCount} field(s), replay frames=${this.twinRuntime.replayBuffer.length}`,
      );
    }
  }

  private twinFieldFromExpr(expr: Expr): string {
    if (expr.kind === "LiteralExpr" && typeof expr.value === "string") return expr.value;
    if (expr.kind === "IdentExpr") return expr.name;
    return getString(this.evalExpr(expr), "");
  }

  private evalTwinMethod(
    method: string,
    expr: import("../ast/nodes.js").CallExpr,
  ): RuntimeValue {
    if (!this.twinRuntime) {
      throw new RuntimeError("No digital twin configured", expr.span.start.line);
    }
    this.refreshTwinShadowFromBackend();
    const twin = this.twinRuntime;

    if (method === "frame_count") {
      return { kind: "number", value: twin.replayBuffer.length, unit: "none" };
    }
    if (method === "mirror") {
      const fieldArg = expr.namedArgs.find((a) => a.name === "field");
      const field = fieldArg
        ? this.twinFieldFromExpr(fieldArg.value)
        : expr.args[0]
          ? this.twinFieldFromExpr(expr.args[0])
          : "";
      const value = twin.shadow[field];
      if (!value) {
        throw new RuntimeError(`Twin has no mirrored shadow field '${field}'`, expr.span.start.line);
      }
      return value;
    }
    if (method === "replay") {
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
      if (!value) {
        throw new RuntimeError(
          `Twin replay frame ${Math.floor(index)} has no field '${field}'`,
          expr.span.start.line,
        );
      }
      return value;
    }
    if (twin.mirrors.includes(method)) {
      const value = twin.shadow[method];
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
    const caps = this.agentCapabilities.get(agent) ?? [];
    if (caps.length === 0) {
      return;
    }
    const allowed = caps.some(
      (c) => c.action === action && (target === undefined || c.target === target),
    );
    if (!allowed) {
      const suffix = target ? `(${target})` : "";
      throw new RuntimeError(`Agent '${agent}' lacks capability ${action}${suffix}`, line);
    }
  }

  private dispatchEvent(eventName: string): void {
    const body = this.eventHandlers.get(eventName);
    if (body) {
      this.options.onLog?.(`emit ${eventName}`);
      this.executeBlock(body);
    } else {
      this.options.onLog?.(`emit ${eventName} (no handler)`);
    }
  }

  private executeEnter(stateName: string, line: number): void {
    let transitioned = false;
    for (const [smName, sm] of this.stateMachines) {
      if (!sm.states.includes(stateName)) continue;
      const allowed = sm.transitions.some((t) => t.from === sm.current && t.to === stateName);
      if (!allowed) continue;
      const previous = sm.current;
      sm.current = stateName;
      this.options.onLog?.(`state_machine ${smName}: ${previous} -> ${stateName}`);
      transitioned = true;
    }
    if (!transitioned) {
      throw new RuntimeError(`No valid transition to state '${stateName}'`, line);
    }
  }

  private setupRobot(robot: RobotDecl): void {
    this.currentRobot = robot;
    this.env = new Environment();
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
    this.defaultTransport = "local";

    if (robot.soc) {
      const profile = getSocProfile(robot.soc.profile);
      this.options.onLog?.(`SoC: ${profile?.name ?? robot.soc.profile} (${profile?.architecture ?? "unknown"})`);
    }

    this.hal = this.options.backend.getHal?.() ?? createSimHal();
    if (robot.hal) {
      const members = robot.hal.members.map(halMemberFromDecl);
      this.hal.configure(members);
      this.options.onLog?.(`HAL configured: ${members.length} bus(es)/pin(s)`);
    }

    for (const bus of robot.buses) {
      this.defaultTransport = bus.transport;
      this.commBus.configure({ nodeName: robot.name });
      this.options.onLog?.(`bus transport: ${bus.transport}`);
    }
    for (const peer of robot.peerRobots) {
      this.commBus.registerRobot(peer.name);
    }
    for (const device of robot.devices) {
      this.commBus.registerDevice(device.name);
      this.env.define(device.name, {
        kind: "object",
        typeName: "Device",
        fields: {},
      });
    }

    if (robot.permissions) {
      this.security.enableStrictPermissions();
      this.security.capabilities.grantAll(robot.permissions.capabilities);
      this.options.onLog?.(
        `permissions: strict mode, granted ${robot.permissions.capabilities.length} capability(ies)`,
      );
    }

    if (robot.trust) {
      const level = parseTrustLevel(robot.trust.level);
      if (level) {
        this.security.trust = level;
        this.options.onLog?.(`trust: level set to ${level}`);
      }
    }

    for (const secret of robot.secrets ?? []) {
      this.security.secrets.register(secret.name, secret.source);
      this.env.define(secret.name, { kind: "secret", name: secret.name });
      this.options.onLog?.(`secret '${secret.name}': registered`);
    }

    if (robot.identity) {
      const id = robot.identity.fields.find(([k]) => k === "id")?.[1] ?? "unknown";
      const publicKey = robot.identity.fields.find(([k]) => k === "public_key")?.[1] ?? "";
      this.security.identity = createRobotIdentity(id, publicKey, this.security.trust);
      this.env.define("identity", { kind: "identity", id, publicKey });
      this.security.grantIfNotStrict("identity.sign");
      this.security.grantIfNotStrict("identity.verify");
      this.options.onLog?.(`identity: device '${id}' registered`);
    }

    for (const topic of robot.topics) {
      this.defineTopic(topic);
    }

    for (const service of robot.services) {
      const serviceType = service.serviceType ?? service.responseType ?? service.name;
      this.env.define(service.name, {
        kind: "service",
        name: service.name,
        serviceType,
      });
    }

    for (const action of robot.actions) {
      const actionType = action.actionType ?? action.resultType ?? action.name;
      this.env.define(action.name, {
        kind: "action",
        name: action.name,
        actionType,
      });
    }

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

    for (const actuator of robot.actuators) {
      this.env.define(actuator.name, {
        kind: "actuator",
        name: actuator.name,
        actuatorType: actuator.actuatorType,
      });
    }

    this.ai_models.clear();
    this.agents.clear();
    for (const modelDecl of robot.ai_models ?? []) {
      const model = createAIModel(modelDecl);
      this.ai_models.set(modelDecl.name, model);
      this.env.define(modelDecl.name, model.toRuntimeValue());
      this.options.onLog?.(
        `AI model '${modelDecl.name}': ${modelDecl.modelType} (${model.config.provider}/${model.config.model})`,
      );
    }

    for (const agentDecl of robot.agents ?? []) {
      const memory = agentDecl.memoryKind ? new MemoryStore(agentDecl.memoryKind) : null;
      const agent = createAgentRuntime(agentDecl, memory);
      this.agents.set(agentDecl.name, agent);
      this.agentCapabilities.set(agentDecl.name, agentDecl.capabilities);
      this.commBus.registerAgent(agentDecl.name);
      this.env.define(agentDecl.name, { kind: "agent", name: agentDecl.name });
      this.options.onLog?.(`Agent '${agentDecl.name}': ${agentDecl.goal}`);
    }

    for (const channel of robot.agentChannels ?? []) {
      this.concurrency.registerAgentRoute(channel.fromAgent, channel.toAgent, channel.messageType);
      const typeSuffix = channel.messageType ? ` (${channel.messageType})` : "";
      this.options.onLog?.(
        `agent channel: ${channel.fromAgent} -> ${channel.toAgent}${typeSuffix}`,
      );
    }

    for (const traitImpl of robot.traitImpls ?? []) {
      const agentMethods = this.agentTraitImpls.get(traitImpl.agentName) ?? new Map();
      for (const method of traitImpl.methods) {
        agentMethods.set(method.name, { params: method.params, body: method.body });
      }
      this.agentTraitImpls.set(traitImpl.agentName, agentMethods);
    }

    for (const event of robot.events ?? []) {
      this.options.onLog?.(`event declared: ${event.name}`);
    }
    for (const handler of robot.eventHandlers ?? []) {
      this.eventHandlers.set(handler.eventName, handler.body);
      this.options.onLog?.(`handler registered for ${handler.eventName}`);
    }

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

    if (robot.verify) {
      this.verifyRules = [...robot.verify.rules];
      this.options.onLog?.(`verify: ${robot.verify.rules.length} rule(s) registered`);
    }

    if (robot.observe) {
      this.fusionSensors = [...robot.observe.sensors];
      this.env.define("fusion", { kind: "sensor_fusion", sensors: [...robot.observe.sensors] });
      this.options.onLog?.(
        `observe: fusing ${robot.observe.sensors.length} sensor(s) [${robot.observe.sensors.join(", ")}]`,
      );
    }

    for (const sm of robot.stateMachines ?? []) {
      const initial = sm.states[0] ?? "unknown";
      this.stateMachines.set(sm.name, {
        current: initial,
        states: [...sm.states],
        transitions: sm.transitions.map((t) => ({ from: t.from, to: t.to })),
      });
      this.options.onLog?.(`state_machine ${sm.name}: initial state ${initial}`);
    }

    if (robot.safety) {
      this.env.define("safety", { kind: "safety_ctx" });
    }

    this.env.define("robot", { kind: "robot" });

    const stopIfRules: Array<(env: Environment) => boolean> = [];
    let maxSpeed = Infinity;

    if (robot.safety) {
      for (const rule of robot.safety.rules) {
        if (rule.kind === "MaxSpeedRule") {
          const val = this.evalExpr(rule.value);
          if (val.kind === "number") maxSpeed = val.value;
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

      for (const zone of robot.safety.zones) {
        this.zones.push(this.evalSafetyZone(zone));
      }
    }

    this.safetyMonitor = new SafetyMonitor(
      createSafetyConfigFromRobot(maxSpeed, stopIfRules, this.zones),
    );
  }

  private evalSafetyZone(zone: SafetyZoneDecl): SafetyZoneRuntime {
    const base: SafetyZoneRuntime = {
      name: zone.name,
      shape: zone.shape,
      x: getNumber(this.evalExpr(zone.x)),
      y: getNumber(this.evalExpr(zone.y)),
    };
    if (zone.shape === "circle" && zone.radius) {
      base.radius = getNumber(this.evalExpr(zone.radius));
    }
    if (zone.shape === "rect" && zone.width && zone.height) {
      base.width = getNumber(this.evalExpr(zone.width));
      base.height = getNumber(this.evalExpr(zone.height));
    }
    return base;
  }

  private defineTopic(topic: import("../ast/nodes.js").TopicDecl): void {
    const path = topic.topic ?? `/${topic.name}`;
    if (topic.secure) {
      this.security.secureEndpoints.register(path, {
        signed: topic.secure.signed,
        minTrust: topic.secure.minTrust ? parseTrustLevel(topic.secure.minTrust) : null,
        requires: topic.secure.requires,
      });
    }
    this.commBus.subscribe(path, topic.name);
    this.env.define(topic.name, {
      kind: "topic",
      name: topic.name,
      messageType: topic.messageType,
      topicPath: path,
    });
  }

  private executeBlock(stmts: Stmt[]): void {
    for (const stmt of stmts) {
      this.executeStmt(stmt);
      if (this.returning) break;
    }
  }

  private executeBlockWithReturn(stmts: Stmt[]): RuntimeValue {
    this.returning = false;
    this.returnValue = { kind: "void" };
    for (const stmt of stmts) {
      this.executeStmt(stmt);
      if (this.returning) break;
    }
    return this.returnValue;
  }

  private callModuleFunction(func: ModuleFnDecl, args: Expr[]): RuntimeValue {
    if (func.isAsync) {
      const argValues = args.map((arg) => this.evalExpr(arg));
      return { kind: "future", funcName: func.name, args: argValues, resolved: null };
    }
    const saved = this.env.clone();
    for (let i = 0; i < func.params.length; i++) {
      const param = func.params[i];
      const arg = args[i];
      if (param && arg) {
        this.env.define(param.name, this.evalExpr(arg));
      }
    }
    const result = this.executeBlockWithReturn(func.body);
    this.env = saved;
    return result;
  }

  private executeStmt(stmt: Stmt): void {
    switch (stmt.kind) {
      case "VarDecl":
        if (stmt.init) {
          if (
            stmt.typeAnnotation?.kind === "trait_object" &&
            stmt.init.kind === "IdentExpr"
          ) {
            this.env.define(stmt.name, {
              kind: "trait_object",
              traitName: stmt.typeAnnotation.traitName,
              agent: stmt.init.name,
            });
          } else {
            this.env.define(stmt.name, this.evalExpr(stmt.init));
          }
        } else {
          this.env.define(stmt.name, { kind: "void" });
        }
        break;
      case "IfStmt": {
        const cond = this.evalExpr(stmt.condition);
        if (cond.kind === "bool" && cond.value) {
          this.executeBlock(stmt.thenBranch);
        } else if (stmt.elseBranch) {
          this.executeBlock(stmt.elseBranch);
        }
        break;
      }
      case "LoopStmt": {
        const maxIter = this.options.maxLoopIterations ?? 10;
        for (let i = 0; i < maxIter; i++) {
          this.options.backend.tick(stmt.intervalMs);
          this.executeBlock(stmt.body);
          if (this.safetyMonitor?.isEmergencyStop()) break;
        }
        break;
      }
      case "PublishStmt": {
        const topic = this.env.get(stmt.topicName);
        const value = this.evalExpr(stmt.value);
        if (topic?.kind === "topic") {
          this.commBus.publish(
            topic.topicPath,
            topic.messageType,
            value,
            this.defaultTransport,
          );
          this.options.backend.publishTopic?.(topic.topicPath, topic.messageType, value);
          this.options.onLog?.(`publish ${topic.topicPath}`);
        }
        break;
      }
      case "ServiceCallStmt": {
        const service = this.env.get(stmt.serviceName);
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
        this.commBus.subscribe(path, stmt.target);
        this.options.onLog?.(`subscribe ${stmt.target}`);
        break;
      }
      case "ExecuteStmt": {
        const action = this.env.get(stmt.actionName);
        const goal = this.evalExpr(stmt.goal);
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
        const val = this.commBus.receive(path);
        if (val) {
          this.env.define(stmt.varName, val);
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
      case "RememberStmt": {
        if (!this.currentAgent) {
          throw new RuntimeError(
            "remember requires active agent context (run inside agent plan)",
            stmt.span.start.line,
          );
        }
        const agent = this.agents.get(this.currentAgent);
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
        for (const arm of stmt.arms) {
          const channelVal = this.evalExpr(arm.channel);
          const msg = this.concurrency.tryRecv(channelVal, stmt.span.start.line);
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
        for (const branch of stmt.body) {
          this.env = saved.clone();
          if (branch.kind === "VarDecl" && branch.init) {
            const val = this.evalExpr(branch.init);
            if (val.kind === "task_handle") {
              pendingHandles.push([branch.name, val.id]);
            } else {
              results[branch.name] = val;
            }
          } else if (branch.kind === "ExprStmt") {
            const val = this.evalExpr(branch.expr);
            if (val.kind === "task_handle") {
              pendingHandles.push([null, val.id]);
            }
          } else if (branch.kind === "SpawnStmt") {
            const { funcName, args } = this.evalSpawnTarget(
              branch.callee,
              branch.args,
              branch.span.start.line,
            );
            const handle = this.concurrency.createTaskHandle(funcName, args);
            if (handle.kind === "task_handle") {
              pendingHandles.push([null, handle.id]);
            }
          } else {
            this.executeStmt(branch);
          }
        }
        this.env = saved;
        for (const [name, id] of pendingHandles) {
          const result = this.resolveTaskHandle(id, stmt.span.start.line);
          if (name) results[name] = result;
        }
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
    switch (expr.kind) {
      case "LiteralExpr":
        if (typeof expr.value === "boolean") return { kind: "bool", value: expr.value };
        if (typeof expr.value === "number") return { kind: "number", value: expr.value, unit: "none" };
        if (typeof expr.value === "string") return { kind: "string", value: expr.value };
        return { kind: "void" };
      case "UnitLiteralExpr":
        return { kind: "number", value: expr.value, unit: expr.unit };
      case "IdentExpr": {
        const enumName = this.variantOwner.get(expr.name);
        if (enumName) {
          return { kind: "enum", enumName, variant: expr.name, payloads: [] };
        }
        const val = this.env.get(expr.name);
        if (!val) throw new RuntimeError(`Undefined variable '${expr.name}'`, expr.span.start.line);
        return val;
      }
      case "BinaryExpr":
        return this.evalBinary(expr.op, this.evalExpr(expr.left), this.evalExpr(expr.right));
      case "UnaryExpr": {
        const operand = this.evalExpr(expr.operand);
        if (expr.op === "not") {
          return { kind: "bool", value: operand.kind === "bool" && !operand.value };
        }
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
        if (value.kind === "enum") variant = value.variant;
        else if (value.kind === "string") variant = value.value;
        else if (value.kind === "object") variant = value.typeName;
        for (const arm of expr.arms) {
          if (arm.variant === variant) {
            const bindings = arm.bindings ?? [];
            if (bindings.length > 0 && value.kind === "enum") {
              for (let i = 0; i < bindings.length; i++) {
                const payload = value.payloads[i];
                const binding = bindings[i];
                if (payload && binding) this.env.set(binding, payload);
              }
            }
            this.executeBlock(arm.body);
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
    const values: Record<string, RuntimeValue> = {};
    for (const field of expr.fields) {
      values[field.name] = this.evalExpr(field.value);
    }
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
    if (expr.object.kind === "IdentExpr") {
      const variants = this.enumVariants.get(expr.object.name);
      if (variants?.includes(expr.property)) {
        return { kind: "enum", enumName: expr.object.name, variant: expr.property, payloads: [] };
      }
    }

    const obj = this.evalExpr(expr.object);

    if (obj.kind === "scan" && expr.property === "nearest_distance") {
      return { kind: "number", value: obj.nearestDistance, unit: "m" };
    }

    if (obj.kind === "pose") {
      const map: Record<string, RuntimeValue> = {
        x: { kind: "number", value: obj.x, unit: "m" },
        y: { kind: "number", value: obj.y, unit: "m" },
        theta: { kind: "number", value: obj.theta, unit: "rad" },
        z: { kind: "number", value: obj.z, unit: "m" },
      };
      return map[expr.property] ?? { kind: "void" };
    }

    if (obj.kind === "velocity") {
      const map: Record<string, RuntimeValue> = {
        linear: { kind: "number", value: obj.linear, unit: "m/s" },
        angular: { kind: "number", value: obj.angular, unit: "rad/s" },
      };
      return map[expr.property] ?? { kind: "void" };
    }

    if (obj.kind === "sensor" && expr.property === "nearest_distance") {
      const reading = this.readSensorValue(obj);
      if (reading.kind === "scan") {
        return { kind: "number", value: reading.nearestDistance, unit: "m" };
      }
    }

    if (obj.kind === "action_proposal") {
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

    if (obj.kind === "safe_action") {
      const map: Record<string, RuntimeValue> = {
        linear: { kind: "number", value: obj.linear, unit: "m/s" },
        angular: { kind: "number", value: obj.angular, unit: "rad/s" },
      };
      return map[expr.property] ?? { kind: "void" };
    }

    if (obj.kind === "goal" && expr.property === "text") {
      return { kind: "string", value: obj.text };
    }

    if (obj.kind === "agent" && expr.property === "goal") {
      const agent = this.agents.get(obj.name);
      return { kind: "goal", text: agent?.decl.goal ?? "" };
    }

    if (obj.kind === "completion" && expr.property === "text") {
      return { kind: "string", value: obj.text };
    }

    if (obj.kind === "object") {
      return obj.fields[expr.property] ?? { kind: "void" };
    }

    return { kind: "void" };
  }

  private evalCall(expr: import("../ast/nodes.js").CallExpr): RuntimeValue {
    if (expr.callee.kind === "IdentExpr") {
      const calleeName = expr.callee.name;
      const moduleFn =
        this.moduleFunctions.get(calleeName) ?? this.importedFunctions.get(calleeName);
      if (moduleFn) {
        return this.callModuleFunction(moduleFn, expr.args);
      }
      const externFn = this.currentProgram?.externFunctions.find(
        (decl) => decl.name === calleeName,
      );
      if (externFn && (externFn.bridge === "python" || externFn.bridge === "cpp")) {
        const args = expr.args.map((arg) => this.evalExpr(arg));
        return callExternBridge(externFn, args);
      }
      const enumName = this.variantOwner.get(calleeName);
      if (enumName) {
        const payloads = expr.args.map((arg) => this.evalExpr(arg));
        return { kind: "enum", enumName, variant: calleeName, payloads };
      }
      return this.evalBuiltinFunction(calleeName, expr);
    }

    if (expr.callee.kind !== "MemberExpr" || expr.callee.object.kind !== "IdentExpr") {
      return { kind: "void" };
    }

    const targetName = expr.callee.object.name;
    const method = expr.callee.property;
    const target = this.env.get(targetName);
    if (!target) {
      throw new RuntimeError(`Undefined '${targetName}'`, expr.span.start.line);
    }

    if (target.kind === "robot" || targetName === "robot") {
      return this.evalRobotMethod(method, expr);
    }

    if (target.kind === "twin") {
      return this.evalTwinMethod(method, expr);
    }

    if (target.kind === "sensor_fusion") {
      if (method === "read") {
        return this.readFusedObservation();
      }
    }

    if (target.kind === "sensor") {
      if (method === "read") {
        if (this.currentAgent) {
          this.checkAgentCapability(this.currentAgent, "read", targetName, expr.span.start.line);
        }
        return this.readSensorValue(target);
      }
      if (target.sensorType === "Camera") {
        if (method === "frame") return mockCameraFrame();
        if (method === "analyze") {
          const frame = mockCameraFrame();
          return mockAnalyzeFrame(frame, target.name);
        }
      }
    }

    if (target.kind === "trait_object") {
      const agentName = target.agent;
      const traitImpl = this.agentTraitImpls.get(agentName)?.get(method);
      if (traitImpl) {
        const saved = this.env.clone();
        for (let i = 0; i < traitImpl.params.length; i++) {
          const param = traitImpl.params[i];
          const argVal = expr.args[i] ? this.evalExpr(expr.args[i]) : { kind: "void" as const };
          this.env.define(param.name, argVal);
        }
        this.currentAgent = agentName;
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

    if (target.kind === "agent") {
      const traitImpl = this.agentTraitImpls.get(targetName)?.get(method);
      if (traitImpl) {
        const saved = this.env.clone();
        for (let i = 0; i < traitImpl.params.length; i++) {
          const param = traitImpl.params[i];
          const argVal = expr.args[i] ? this.evalExpr(expr.args[i]) : { kind: "void" as const };
          this.env.define(param.name, argVal);
        }
        this.currentAgent = targetName;
        try {
          this.executeBlock(traitImpl.body);
        } finally {
          this.currentAgent = null;
          this.env = saved;
        }
        this.options.onLog?.(`agent ${targetName}.${method}()`);
        return { kind: "void" };
      }
      if (method === "plan") {
        this.checkAgentCapability(targetName, "plan", undefined, expr.span.start.line);
        const agent = this.agents.get(targetName);
        if (!agent) {
          throw new RuntimeError(`Unknown agent '${targetName}'`, expr.span.start.line);
        }
        this.currentAgent = targetName;
        try {
          executeAgentPlan(agent, { executeBlock: (stmts) => this.executeBlock(stmts) });
        } finally {
          this.currentAgent = null;
        }
        this.options.onLog?.(`agent ${targetName}.plan()`);
        return { kind: "void" };
      }
    }

    if (target.kind === "safety_ctx" && method === "validate") {
      return this.evalSafetyValidate(expr);
    }

    const aiModel = this.ai_models.get(targetName);
    if (aiModel || target.kind === "ai_model") {
      const model = aiModel ?? this.ai_models.get(targetName);
      if (!model) return { kind: "void" };
      if (method === "reason") {
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
      if (method === "summarize") {
        if (this.currentAgent) {
          this.checkAgentCapability(this.currentAgent, "summarize", undefined, expr.span.start.line);
        }
        const input = this.getNamedArgValue(expr, "input");
        return model.summarize(input.kind === "void" ? undefined : input);
      }
      if (method === "detect") {
        if (this.currentAgent) {
          this.checkAgentCapability(this.currentAgent, "detect", undefined, expr.span.start.line);
        }
        const frame = expr.args[0] ? this.evalExpr(expr.args[0]) : this.getNamedArgValue(expr, "frame");
        return model.detect(frame);
      }
      if (method === "drive") {
        throw new RuntimeError(
          "Unsafe AI action: LLM cannot drive actuators directly — use safety.validate() then wheels.execute()",
          expr.span.start.line,
        );
      }
    }

    if (target.kind === "actuator") {
      return this.executeActuatorMethod(target.name, target.actuatorType, method, expr);
    }

    return { kind: "void" };
  }

  private readSensorValue(target: Extract<RuntimeValue, { kind: "sensor" }>): RuntimeValue {
    const state = this.options.backend.getState();
    if (target.library) {
      const driver = getSensorDriver(target.library, target.sensorType);
      if (driver) {
        return readWithDriver(driver, {
          hal: this.hal,
          halBinding: target.halBinding ?? null,
          topic: target.topic ?? null,
          simState: { pose: state.pose },
        });
      }
    }
    return this.options.backend.readSensor(target.name, target.sensorType, target.topic);
  }

  private readFusedObservation(): RuntimeValue {
    const fields: Record<string, RuntimeValue> = {};
    for (const sensorName of this.fusionSensors) {
      const sensorVal = this.env.get(sensorName);
      if (!sensorVal || sensorVal.kind !== "sensor") {
        throw new RuntimeError(`Unknown observe sensor '${sensorName}'`, 0);
      }
      fields[sensorName] = this.readSensorValue(sensorVal);
    }
    const state = this.options.backend.getState();
    fields.pose = poseFromState(state.pose);
    fields.count = { kind: "number", value: this.fusionSensors.length, unit: "none" };
    return { kind: "object", typeName: "FusedObservation", fields };
  }

  private evalBuiltinFunction(name: string, expr: import("../ast/nodes.js").CallExpr): RuntimeValue {
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
        if (!this.currentAgent) {
          throw new RuntimeError(
            "recall() requires active agent context (run inside agent plan)",
            expr.span.start.line,
          );
        }
        const agent = this.agents.get(this.currentAgent);
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
        if (!arg0) {
          throw new RuntimeError("assert requires a boolean condition", expr.span.start.line);
        }
        const cond = this.evalExpr(arg0);
        if (cond.kind !== "bool") {
          throw new RuntimeError("assert requires a boolean condition", expr.span.start.line);
        }
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
        if (handle.kind === "future") {
          return this.resolveFuture(handle, expr.span.start.line);
        }
        if (handle.kind === "task_handle") {
          return this.resolveTaskHandle(handle.id, expr.span.start.line);
        }
        throw new RuntimeError("join requires a Future or TaskHandle value", expr.span.start.line);
      }
      case "send_agent": {
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
  ): { funcName: string; args: RuntimeValue[] } {
    const argValues = args.map((arg) => this.evalExpr(arg));
    if (callee.kind !== "IdentExpr") {
      throw new RuntimeError("spawn requires function name", line);
    }
    return { funcName: callee.name, args: argValues };
  }

  private executeSpawnJob(funcName: string, args: RuntimeValue[], line: number): RuntimeValue {
    const func =
      this.moduleFunctions.get(funcName) ?? this.importedFunctions.get(funcName);
    if (!func) {
      throw new RuntimeError(`Unknown spawn target '${funcName}'`, line);
    }
    const saved = this.env.clone();
    for (let i = 0; i < func.params.length; i++) {
      const param = func.params[i];
      const val = args[i];
      if (param && val) this.env.define(param.name, val);
    }
    const result = this.executeBlockWithReturn(func.body);
    this.env = saved;
    return result;
  }

  private resolveTaskHandle(id: number, line: number): RuntimeValue {
    const handle = this.concurrency.getHandle(id);
    if (!handle) {
      throw new RuntimeError(`Unknown task handle ${id}`, line);
    }
    if (handle.result) return handle.result;
    const result = this.executeSpawnJob(handle.funcName, handle.args, line);
    this.concurrency.setHandleResult(id, result);
    return result;
  }

  private resolveFuture(future: RuntimeValue, line: number): RuntimeValue {
    if (future.kind === "future") {
      if (future.resolved) return future.resolved;
      const result = this.executeSpawnJob(future.funcName, future.args, line);
      return result;
    }
    return future;
  }

  private processSpawnQueue(): void {
    for (const id of this.concurrency.drainFireAndForgetQueue()) {
      this.resolveTaskHandle(id, 0);
    }
  }

  private goalTextFromValue(value: RuntimeValue): string | undefined {
    if (value.kind === "goal") return value.text;
    if (value.kind === "string") return value.value;
    return undefined;
  }

  private resolveReasonGoal(expr: import("../ast/nodes.js").CallExpr): string | undefined {
    const explicit = this.getNamedArgValue(expr, "goal");
    if (explicit.kind !== "void") {
      return this.goalTextFromValue(explicit);
    }
    if (this.currentAgent) {
      const agent = this.agents.get(this.currentAgent);
      const text = agent?.decl.goal?.trim();
      if (text) return text;
    }
    return undefined;
  }

  private enrichReasonGoal(goalText: string | undefined): string | undefined {
    if (!this.currentAgent) return goalText;
    const agent = this.agents.get(this.currentAgent);
    const memorySummary = agent?.memory?.summaryForPrompt();
    if (!memorySummary) return goalText;
    if (goalText) {
      return `${goalText}\n${memorySummary}`;
    }
    return memorySummary;
  }

  private evalSafetyValidate(expr: import("../ast/nodes.js").CallExpr): RuntimeValue {
    const arg = expr.args[0] ? this.evalExpr(expr.args[0]) : this.getNamedArgValue(expr, "proposal");
    const proposal = proposalFromValue(arg);
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
    if (!result?.ok) {
      throw new RuntimeError(result?.reason ?? "Safety validation failed for AI action", expr.span.start.line);
    }
    this.options.onLog?.("safety.validate() approved ActionProposal");
    return safeActionFromProposal(result.linear, result.angular);
  }

  private evalRobotMethod(method: string, expr: import("../ast/nodes.js").CallExpr): RuntimeValue {
    const state = this.options.backend.getState();
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
    const motionMethods = ["drive", "move_to", "set_thrust", "grip", "release", "open", "hover", "follow"];
    if (motionMethods.includes(method) || method === "stop") {
      if (!this.checkSafetyBeforeMotion()) {
        this.options.onMotionBlocked?.("Safety rule triggered — motion blocked");
        this.options.backend.executeMotion({ kind: "stop", actuator: name });
        return { kind: "void" };
      }
    }

    switch (method) {
      case "stop":
        this.options.backend.executeMotion({ kind: "stop", actuator: name });
        break;
      case "drive": {
        const linear = this.getNamedArgNumber(expr, "linear", 0);
        const angular = this.getNamedArgNumber(expr, "angular", 0);
        const maxSpeed = this.safetyMonitor?.clampSpeed(linear) ?? linear;
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
        if (this.currentAgent) {
          this.checkAgentCapability(this.currentAgent, "propose_motion", undefined, expr.span.start.line);
        }
        const actionVal = expr.args[0]
          ? this.evalExpr(expr.args[0])
          : this.getNamedArgValue(expr, "action");
        if (!isSafeAction(actionVal)) {
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

  private getNamedArgValue(expr: import("../ast/nodes.js").CallExpr, name: string): RuntimeValue {
    const arg = expr.namedArgs.find((a) => a.name === name);
    return arg ? this.evalExpr(arg.value) : { kind: "void" };
  }

  private getNamedArgNumber(expr: import("../ast/nodes.js").CallExpr, name: string, defaultVal: number): number {
    return getNumber(this.getNamedArgValue(expr, name), defaultVal);
  }

  private evalBinary(op: string, left: RuntimeValue, right: RuntimeValue): RuntimeValue {
    if (op === "and") {
      return { kind: "bool", value: (left.kind === "bool" && left.value) && (right.kind === "bool" && right.value) };
    }
    if (op === "or") {
      return { kind: "bool", value: (left.kind === "bool" && left.value) || (right.kind === "bool" && right.value) };
    }
    if (left.kind === "bool" && right.kind === "bool") {
      if (op === "==") return { kind: "bool", value: left.value === right.value };
      if (op === "!=") return { kind: "bool", value: left.value !== right.value };
    }
    if (left.kind === "number" && right.kind === "number") {
      const aligned = alignForBinary(left.value, left.unit, right.value, right.unit);
      const l = aligned?.[0] ?? left.value;
      const r = aligned?.[1] ?? right.value;
      const resultUnit = aligned?.[2] ?? left.unit;
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
    const state = this.options.backend.getState();
    const result = this.safetyMonitor?.evaluateBeforeMotion(this.env, state.pose);
    if (!result?.allowed) {
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
