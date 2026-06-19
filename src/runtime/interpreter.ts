import type {
  Expr,
  Program,
  RobotDecl,
  SafetyRule,
  SafetyZoneDecl,
  Stmt,
  UnitKind,
} from "../ast/nodes.js";
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
import { SafetyMonitor, createSafetyConfigFromRobot, interpolatePoses } from "../safety/index.js";
import type { SafetyZoneRuntime } from "../safety/index.js";
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
  | { kind: "enum"; enumName: string; variant: string }
  | { kind: "sensor"; name: string; sensorType: string; library?: string | null; halBinding?: string | null; topic?: string | null }
  | { kind: "actuator"; name: string; actuatorType: string }
  | { kind: "topic"; name: string; messageType: string; topicPath: string }
  | { kind: "service"; name: string; serviceType: string }
  | { kind: "action"; name: string; actionType: string }
  | { kind: "robot" }
  | { kind: "agent"; name: string }
  | { kind: "twin"; name: string }
  | { kind: "safety_ctx" }
  | { kind: "ai_model"; name: string; modelType: string; provider: string }
  | { kind: "action_proposal"; linear: number; angular: number; source: string; trusted: false }
  | { kind: "safe_action"; linear: number; angular: number; trusted: true }
  | { kind: "completion"; text: string; model?: string }
  | { kind: "embedding"; dimensions: number; vector: number[] };

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

  constructor(private options: InterpreterOptions) {}

  run(program: Program, entryBehavior?: string): RobotState {
    this.currentProgram = program;
    this.loadProgramMetadata(program);
    for (const robot of program.robots) {
      this.setupRobot(robot);
      const behaviorName =
        entryBehavior ?? robot.behaviors[0]?.name ?? robot.tasks[0]?.name;
      if (!behaviorName) continue;

      const behavior = robot.behaviors.find((b) => b.name === behaviorName);
      if (behavior) {
        this.executeWithContracts(behavior.body, behavior.requires, behavior.ensures, behavior.invariant);
        continue;
      }

      const task = robot.tasks.find((t) => t.name === behaviorName);
      if (task) {
        this.executeTaskLoop(task.body, task.intervalMs, task.requires, task.ensures, task.invariant);
      }
    }
    return this.options.backend.getState();
  }

  private loadProgramMetadata(program: Program): void {
    this.enumVariants.clear();
    this.variantOwner.clear();
    this.structDefs.clear();
    for (const enumDecl of program.enums) {
      this.enumVariants.set(enumDecl.name, [...enumDecl.variants]);
      for (const variant of enumDecl.variants) {
        this.variantOwner.set(variant, enumDecl.name);
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
  }

  private executeTaskLoop(
    body: Stmt[],
    intervalMs: number,
    requires: Expr | null,
    ensures: Expr | null,
    invariant: Expr | null,
  ): void {
    const maxIter = this.options.maxLoopIterations ?? 10;
    for (let i = 0; i < maxIter; i++) {
      this.options.backend.tick(intervalMs);
      if (requires && !this.evalContract(requires)) {
        this.options.onLog?.("task requires contract failed — skipping iteration");
        continue;
      }
      this.executeBlock(body);
      if (ensures && !this.evalContract(ensures)) {
        throw new RuntimeError("task ensures contract failed", 0);
      }
      if (invariant && !this.evalContract(invariant)) {
        throw new RuntimeError("task invariant contract failed", 0);
      }
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
    this.currentAgent = null;

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

    for (const topic of robot.topics) {
      this.env.define(topic.name, {
        kind: "topic",
        name: topic.name,
        messageType: topic.messageType,
        topicPath: topic.topic,
      });
    }

    for (const service of robot.services) {
      this.env.define(service.name, {
        kind: "service",
        name: service.name,
        serviceType: service.serviceType,
      });
    }

    for (const action of robot.actions) {
      this.env.define(action.name, {
        kind: "action",
        name: action.name,
        actionType: action.actionType,
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
      this.env.define(agentDecl.name, { kind: "agent", name: agentDecl.name });
      this.options.onLog?.(`Agent '${agentDecl.name}': ${agentDecl.goal}`);
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

  private executeBlock(stmts: Stmt[]): void {
    for (const stmt of stmts) {
      this.executeStmt(stmt);
    }
  }

  private executeStmt(stmt: Stmt): void {
    switch (stmt.kind) {
      case "VarDecl":
        this.env.define(stmt.name, this.evalExpr(stmt.init));
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
          this.options.backend.publishTopic?.(topic.topicPath, topic.messageType, value);
          this.options.onLog?.(`publish ${topic.topicPath}`);
        }
        break;
      }
      case "ServiceCallStmt": {
        const service = this.env.get(stmt.serviceName);
        if (service?.kind === "service") {
          this.options.backend.callService?.(service.name, service.serviceType);
          this.options.onLog?.(`call ${service.name}()`);
        }
        break;
      }
      case "ActionSendStmt": {
        const action = this.env.get(stmt.actionName);
        const goal = this.evalExpr(stmt.goal);
        if (action?.kind === "action") {
          this.options.backend.sendAction?.(action.name, action.actionType, goal);
          this.options.onLog?.(`send_goal ${action.name}`);
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
      case "ExprStmt":
        this.evalExpr(stmt.expr);
        break;
      case "ReturnStmt":
        break;
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
          return { kind: "enum", enumName, variant: expr.name };
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
            this.executeBlock(arm.body);
            break;
          }
        }
        return { kind: "void" };
      }
      case "StructLiteralExpr":
        return this.evalStructLiteral(expr);
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
        return { kind: "enum", enumName: expr.object.name, variant: expr.property };
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

    if (obj.kind === "action_proposal" || obj.kind === "safe_action") {
      const map: Record<string, RuntimeValue> = {
        linear: { kind: "number", value: obj.linear, unit: "m/s" },
        angular: { kind: "number", value: obj.angular, unit: "rad/s" },
      };
      return map[expr.property] ?? { kind: "void" };
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
      return this.evalBuiltinFunction(expr.callee.name, expr);
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
        const prompt = getString(this.getNamedArgValue(expr, "prompt"));
        const input = this.getNamedArgValue(expr, "input");
        const result = model.reason(prompt, input.kind === "void" ? undefined : input);
        this.options.onLog?.(`ai ${targetName}.reason() -> ActionProposal`);
        return result;
      }
      if (method === "summarize") {
        const input = this.getNamedArgValue(expr, "input");
        return model.summarize(input.kind === "void" ? undefined : input);
      }
      if (method === "detect") {
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
      default:
        return { kind: "void" };
    }
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
      switch (op) {
        case "+": return { kind: "number", value: left.value + right.value, unit: left.unit };
        case "-": return { kind: "number", value: left.value - right.value, unit: left.unit };
        case "*": return { kind: "number", value: left.value * right.value, unit: "none" };
        case "/": return { kind: "number", value: left.value / right.value, unit: "none" };
        case "<": return { kind: "bool", value: left.value < right.value };
        case "<=": return { kind: "bool", value: left.value <= right.value };
        case ">": return { kind: "bool", value: left.value > right.value };
        case ">=": return { kind: "bool", value: left.value >= right.value };
        case "==": return { kind: "bool", value: left.value === right.value };
        case "!=": return { kind: "bool", value: left.value !== right.value };
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
