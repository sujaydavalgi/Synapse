import type {
  BehaviorDecl,
  Expr,
  Program,
  RobotDecl,
  SafetyRule,
  SafetyZoneDecl,
  Stmt,
  UnitKind,
  SpandaType,
  TopicDecl,
  ServiceDecl,
  ActionDecl,
} from "../ast/nodes.js";
import type { CapabilityDecl, MatchArm, TraitImplDecl } from "../foundations.js";
import { resolveModuleImport, resolveTypeAlias } from "../foundations.js";
import { resolveStdImport } from "../stdlib.js";
import {
  MessageRegistry,
  isCommCapability,
  transportFromIdent,
} from "../comm/index.js";
import {
  binaryPhysicalOpAllowed,
  isActionProposalType,
  resolveTypeName,
  isSafeActionType,
  typeKindName,
} from "../type-system.js";
import { resolveImport } from "../lib/registry.js";
import { resolveAiImport } from "../ai/registry.js";
import { getSocProfile, validateHalAgainstSoc } from "../soc/index.js";
import { halMemberFromDecl } from "../hal/index.js";
import {
  ACTION_TYPES,
  AI_MODEL_TYPES,
  AI_VALUE_TYPES,
  ACTUATOR_TYPES,
  BUILTIN_FUNCTIONS,
  BUILTIN_METHODS,
  OBJECT_PROPERTIES,
  POSE_PROPERTIES,
  ROBOT_METHODS,
  SCAN_PROPERTIES,
  SENSOR_TYPES,
  SERVICE_TYPES,
  VELOCITY_PROPERTIES,
  TypeCheckError,
  resultUnitForBinary,
  unitsCompatible,
  unitMatchesNamedType,
  type TypeError,
} from "./units.js";
import { isKnownCapability, parseTrustLevel } from "../security/index.js";
import { unitCategory } from "../units/index.js";

type SymbolEntry = {
  name: string;
  roboType: SpandaType;
  kind: "sensor" | "actuator" | "variable" | "behavior" | "topic" | "service" | "action" | "robot" | "ai_model" | "agent" | "safety";
  sensorType?: string;
  actuatorType?: string;
  messageType?: string;
  serviceType?: string;
  actionType?: string;
};

export function typeCheck(program: Program): void {
  check(program);
}

export function check(program: Program): void {
  const checker = new TypeChecker();
  checker.checkProgram(program);
  if (checker.errors.length > 0) {
    throw new TypeCheckError(checker.errors);
  }
}

class TypeChecker {
  errors: TypeError[] = [];
  private symbols = new Map<string, SymbolEntry>();
  private enumVariants = new Map<string, string[]>();
  private variantOwner = new Map<string, string>();
  private structDefs = new Map<string, Array<{ name: string; typeName: string }>>();
  private traitDefs = new Map<string, Map<string, { params: Array<{ name: string; typeName: string }>; returnType: string }>>();
  private agentTraitMethods = new Map<string, Map<string, SpandaType>>();
  private stateMachineStates = new Set<string>();
  private currentRobot: RobotDecl | null = null;
  private messageRegistry = MessageRegistry.new();
  private subscribedTopics = new Set<string>();
  private agentNames = new Set<string>();
  private deviceNames = new Set<string>();
  private peerRobotNames = new Set<string>();

  checkProgram(program: Program): void {
    const imported = new Set<string>();
    for (const imp of program.imports) {
      if (
        !resolveImport(imp.path) &&
        !resolveAiImport(imp.path) &&
        !resolveModuleImport(imp.path) &&
        !resolveStdImport(imp.path)
      ) {
        this.error(`Unknown import '${imp.path}'`, imp.span.start.line, imp.span.start.column);
      } else {
        imported.add(imp.path);
      }
    }

    for (const structDecl of program.structs) {
      this.checkStruct(structDecl);
    }
    for (const enumDecl of program.enums) {
      this.checkEnum(enumDecl);
    }
    for (const traitDecl of program.traits) {
      this.checkTrait(traitDecl);
    }

    this.messageRegistry = MessageRegistry.fromProgram(program.messages, program.structs);
    for (const msg of program.messages) {
      this.checkMessage(msg);
    }

    for (const robot of program.robots) {
      this.checkRobot(robot, imported);
    }
  }

  private checkMessage(decl: import("../comm/index.js").MessageDecl): void {
    for (const field of decl.fields) {
      let known = !!this.messageRegistry.resolveType(field.typeName) || !!resolveTypeAlias(field.typeName);
      if (!known) {
        try {
          resolveTypeName(field.typeName);
          known = true;
        } catch {
          /* unknown */
        }
      }
      if (!known) {
        this.error(
          `Unknown field type '${field.typeName}' in message '${decl.name}'`,
          field.span.start.line,
          field.span.start.column,
        );
      }
    }
    this.symbols.set(decl.name, {
      name: decl.name,
      roboType: { kind: "named", name: decl.name },
      kind: "variable",
    });
  }

  private resolveMessageType(name: string): SpandaType | null {
    return this.messageRegistry.resolveType(name);
  }

  private checkStruct(decl: import("../foundations.js").StructDecl): void {
    for (const field of decl.fields) {
      if (
        !resolveTypeAlias(field.typeName) &&
        !["Pose", "Velocity", "Scan", "String", "Bool", "Path"].includes(field.typeName)
      ) {
        this.error(`Unknown field type '${field.typeName}'`, field.span.start.line, field.span.start.column);
      }
    }
    this.symbols.set(decl.name, {
      name: decl.name,
      roboType: { kind: "named", name: decl.name },
      kind: "variable",
    });
    this.structDefs.set(
      decl.name,
      decl.fields.map((f) => ({ name: f.name, typeName: f.typeName })),
    );
  }

  private checkEnum(decl: import("../foundations.js").EnumDecl): void {
    if (decl.variants.length === 0) {
      this.error(`Enum '${decl.name}' must declare at least one variant`, decl.span.start.line, decl.span.start.column);
    }
    this.symbols.set(decl.name, {
      name: decl.name,
      roboType: { kind: "named", name: decl.name },
      kind: "variable",
    });
    this.enumVariants.set(decl.name, [...decl.variants]);
    for (const variant of decl.variants) {
      const existing = this.variantOwner.get(variant);
      if (existing) {
        this.error(
          `Enum variant '${variant}' already declared in enum '${existing}'`,
          decl.span.start.line,
          decl.span.start.column,
        );
      } else {
        this.variantOwner.set(variant, decl.name);
      }
    }
  }

  private checkTrait(decl: import("../foundations.js").TraitDecl): void {
    if (decl.methods.length === 0) {
      this.error(`Trait '${decl.name}' must declare at least one method`, decl.span.start.line, decl.span.start.column);
    }
    const methodMap = new Map<string, { params: Array<{ name: string; typeName: string }>; returnType: string }>();
    for (const method of decl.methods) {
      methodMap.set(method.name, {
        params: method.params.map((p) => ({ name: p.name, typeName: p.typeName })),
        returnType: method.returnType,
      });
    }
    this.traitDefs.set(decl.name, methodMap);
  }

  private typeNameToSpanda(typeName: string): SpandaType {
    try {
      return resolveTypeName(typeName);
    } catch {
      switch (resolveTypeAlias(typeName)) {
        case "distance":
          return { kind: "number", unit: "m" };
        case "angle":
          return { kind: "number", unit: "rad" };
        case "path":
          return { kind: "trajectory" };
        case "velocity":
          return { kind: "velocity" };
        case "pose":
          return { kind: "pose" };
        default:
          return { kind: "named", name: typeName };
      }
    }
  }

  private checkRobot(robot: RobotDecl, imported: Set<string>): void {
    this.currentRobot = robot;
    this.subscribedTopics.clear();
    this.agentNames.clear();
    this.deviceNames.clear();
    this.peerRobotNames.clear();
    this.symbols.clear();
    this.stateMachineStates.clear();
    this.agentTraitMethods.clear();

    for (const enumName of this.enumVariants.keys()) {
      this.symbols.set(enumName, {
        name: enumName,
        roboType: { kind: "named", name: enumName },
        kind: "variable",
      });
    }
    for (const structName of this.structDefs.keys()) {
      this.symbols.set(structName, {
        name: structName,
        roboType: { kind: "named", name: structName },
        kind: "variable",
      });
    }

    if (robot.soc) {
      if (!getSocProfile(robot.soc.profile)) {
        this.error(`Unknown SoC profile '${robot.soc.profile}'`, robot.soc.span.start.line, robot.soc.span.start.column);
      }
    }

    if (robot.hal && robot.soc) {
      const profile = getSocProfile(robot.soc.profile);
      if (profile) {
        const members = robot.hal.members.map(halMemberFromDecl);
        for (const err of validateHalAgainstSoc(profile, members)) {
          this.error(err.message, robot.hal.span.start.line, robot.hal.span.start.column);
        }
      }
    }

    const halBusNames = new Set(robot.hal?.members.map((m) => m.name) ?? []);

    for (const node of robot.nodes) {
      if (!node.namespace) {
        this.error("Node should specify namespace with 'on \"/namespace\"'", node.span.start.line, node.span.start.column);
      }
    }

    for (const topic of robot.topics) {
      this.checkTopic(topic);
    }

    for (const service of robot.services) {
      this.checkService(service);
    }

    for (const action of robot.actions) {
      this.checkAction(action);
    }

    for (const sensor of robot.sensors) {
      if (!SENSOR_TYPES[sensor.sensorType]) {
        this.error(`Unknown sensor type '${sensor.sensorType}'`, sensor.span.start.line, sensor.span.start.column);
      }
      if (sensor.library) {
        if (!imported.has(sensor.library)) {
          this.error(`Library '${sensor.library}' must be imported before use`, sensor.span.start.line, sensor.span.start.column);
        }
        const lib = resolveImport(sensor.library);
        if (lib && !lib.sensors[sensor.sensorType]) {
          this.error(`Sensor type '${sensor.sensorType}' not provided by library '${sensor.library}'`, sensor.span.start.line, sensor.span.start.column);
        }
      }
      if (sensor.binding?.kind === "hal" && !halBusNames.has(sensor.binding.busName)) {
        this.error(`Unknown HAL bus '${sensor.binding.busName}'`, sensor.span.start.line, sensor.span.start.column);
      }
      this.symbols.set(sensor.name, {
        name: sensor.name,
        roboType: SENSOR_TYPES[sensor.sensorType] ?? { kind: "named", name: sensor.sensorType },
        kind: "sensor",
        sensorType: sensor.sensorType,
      });
    }

    for (const actuator of robot.actuators) {
      if (!ACTUATOR_TYPES[actuator.actuatorType]) {
        this.error(`Unknown actuator type '${actuator.actuatorType}'`, actuator.span.start.line, actuator.span.start.column);
      }
      this.symbols.set(actuator.name, {
        name: actuator.name,
        roboType: ACTUATOR_TYPES[actuator.actuatorType] ?? { kind: "named", name: actuator.actuatorType },
        kind: "actuator",
        actuatorType: actuator.actuatorType,
      });
    }

    for (const bus of robot.buses) {
      if (transportFromIdent(bus.name) === null && bus.transport === "local") {
        this.error(
          `Unknown transport '${bus.name}' in bus declaration`,
          bus.span.start.line,
          bus.span.start.column,
        );
      }
    }

    for (const peer of robot.peerRobots) {
      this.peerRobotNames.add(peer.name);
      this.symbols.set(peer.name, {
        name: peer.name,
        roboType: { kind: "named", name: "PeerRobot" },
        kind: "robot",
      });
    }

    for (const device of robot.devices) {
      if (!["Camera", "IMU", "Lidar", "GPS", "Microphone", "Speaker"].includes(device.deviceType)) {
        this.error(
          `Unknown device type '${device.deviceType}'`,
          device.span.start.line,
          device.span.start.column,
        );
      }
      this.deviceNames.add(device.name);
      this.symbols.set(device.name, {
        name: device.name,
        roboType: { kind: "named", name: device.deviceType },
        kind: "sensor",
        sensorType: device.deviceType,
      });
    }

    if (robot.safety) {
      const saved = new Map(this.symbols);
      for (const rule of robot.safety.rules) {
        this.checkSafetyRule(rule);
      }
      for (const zone of robot.safety.zones) {
        this.checkSafetyZone(zone);
      }
      this.symbols = saved;
    }

    if (robot.ai_models.length > 0) {
      for (const model of robot.ai_models ?? []) {
        this.checkAiModel(model);
      }
    }

    if (robot.safety) {
      this.symbols.set("safety", {
        name: "safety",
        roboType: { kind: "named", name: "Safety" },
        kind: "safety",
      });
    }

    for (const agent of robot.agents) {
      this.checkAgent(agent);
    }

    for (const channel of robot.agentChannels) {
      if (!this.agentNames.has(channel.fromAgent)) {
        this.error(
          `Agent channel source '${channel.fromAgent}' is not declared`,
          channel.span.start.line,
          channel.span.start.column,
        );
      }
      if (!this.agentNames.has(channel.toAgent)) {
        this.error(
          `Agent channel target '${channel.toAgent}' is not declared`,
          channel.span.start.line,
          channel.span.start.column,
        );
      }
    }

    for (const traitImpl of robot.traitImpls) {
      this.checkTraitImpl(traitImpl);
    }

    for (const sm of robot.stateMachines) {
      if (sm.states.length === 0) {
        this.error(
          `State machine '${sm.name}' must declare at least one state`,
          sm.span.start.line,
          sm.span.start.column,
        );
      }
      const stateSet = new Set(sm.states);
      for (const transition of sm.transitions) {
        if (!stateSet.has(transition.from) || !stateSet.has(transition.to)) {
          this.error(
            `Invalid transition ${transition.from} -> ${transition.to} in state machine '${sm.name}'`,
            transition.span.start.line,
            transition.span.start.column,
          );
        }
      }
      for (const state of sm.states) {
        this.stateMachineStates.add(state);
      }
    }

    if (robot.twin) {
      if (robot.twin.mirrors.length === 0) {
        this.error("Digital twin must mirror at least one field", robot.twin.span.start.line, robot.twin.span.start.column);
      }
      const allowedMirrorFields = ["pose", "velocity", "battery", "status", "scan"];
      for (const mirror of robot.twin.mirrors) {
        if (!allowedMirrorFields.includes(mirror)) {
          this.error(`Unknown twin mirror field '${mirror}'`, robot.twin.span.start.line, robot.twin.span.start.column);
        }
      }
      this.symbols.set(robot.twin.name, {
        name: robot.twin.name,
        roboType: { kind: "named", name: "Twin" },
        kind: "variable",
      });
    }

    if (robot.verify) {
      const saved = new Map(this.symbols);
      this.symbols.set("robot", {
        name: "robot",
        roboType: { kind: "named", name: "Robot" },
        kind: "robot",
      });
      for (const rule of robot.verify.rules) {
        const t = this.checkExpr(rule);
        if (t.kind !== "bool") {
          this.error(
            "verify rule must be boolean",
            robot.verify.span.start.line,
            robot.verify.span.start.column,
          );
        }
      }
      this.symbols = saved;
    }

    if (robot.observe) {
      if (robot.observe.sensors.length === 0) {
        this.error(
          "observe block must list at least one sensor",
          robot.observe.span.start.line,
          robot.observe.span.start.column,
        );
      }
      for (const sensorName of robot.observe.sensors) {
        const sym = this.symbols.get(sensorName);
        if (!sym || sym.kind !== "sensor") {
          this.error(
            `observe references unknown sensor '${sensorName}'`,
            robot.observe.span.start.line,
            robot.observe.span.start.column,
          );
        }
      }
      this.symbols.set("fusion", {
        name: "fusion",
        roboType: { kind: "named", name: "SensorFusion" },
        kind: "variable",
      });
    }

    if (robot.identity) {
      if (!robot.identity.fields.some(([k]) => k === "id")) {
        this.error("identity block must declare an 'id' field", robot.identity.span.start.line, robot.identity.span.start.column);
      }
      this.symbols.set("identity", {
        name: "identity",
        roboType: { kind: "named", name: "RobotIdentity" },
        kind: "variable",
      });
    }

    if (robot.audit) {
      if (robot.audit.records.length === 0) {
        this.error("audit block must record at least one field", robot.audit.span.start.line, robot.audit.span.start.column);
      }
      this.symbols.set("audit", {
        name: "audit",
        roboType: { kind: "named", name: "AuditLog" },
        kind: "variable",
      });
      this.symbols.set("mock_ledger", {
        name: "mock_ledger",
        roboType: { kind: "named", name: "MockLedger" },
        kind: "variable",
      });
    }

    for (const secret of robot.secrets ?? []) {
      this.symbols.set(secret.name, {
        name: secret.name,
        roboType: { kind: "named", name: "Secret" },
        kind: "variable",
      });
    }

    if (robot.trust) {
      if (!parseTrustLevel(robot.trust.level)) {
        this.error(`unknown trust level '${robot.trust.level}'`, robot.trust.span.start.line, robot.trust.span.start.column);
      }
    }

    if (robot.permissions) {
      if (robot.permissions.capabilities.length === 0) {
        this.error("permissions block must grant at least one capability", robot.permissions.span.start.line, robot.permissions.span.start.column);
      }
      for (const cap of robot.permissions.capabilities) {
        if (!isKnownCapability(cap)) {
          this.error(`unknown package capability '${cap}'`, robot.permissions.span.start.line, robot.permissions.span.start.column);
        }
      }
    }

    for (const behavior of robot.behaviors) {
      if (behavior.requires) {
        const t = this.checkExpr(behavior.requires);
        if (t.kind !== "bool") {
          this.error("requires clause must be boolean", behavior.span.start.line, behavior.span.start.column);
        }
      }
      if (behavior.ensures) {
        const t = this.checkExpr(behavior.ensures);
        if (t.kind !== "bool") {
          this.error("ensures clause must be boolean", behavior.span.start.line, behavior.span.start.column);
        }
      }
      if (behavior.invariant) {
        const t = this.checkExpr(behavior.invariant);
        if (t.kind !== "bool") {
          this.error("invariant clause must be boolean", behavior.span.start.line, behavior.span.start.column);
        }
      }
      this.symbols.set(behavior.name, {
        name: behavior.name,
        roboType: { kind: "void" },
        kind: "behavior",
      });
      this.checkBehaviorBody(behavior.body);
    }

    for (const task of robot.tasks) {
      if (task.intervalMs <= 0) {
        this.error("task interval must be positive", task.span.start.line, task.span.start.column);
      } else if (task.intervalMs < 1) {
        this.error("task interval must be at least 1ms", task.span.start.line, task.span.start.column);
      }
      if (task.requires) {
        const t = this.checkExpr(task.requires);
        if (t.kind !== "bool") {
          this.error("requires clause must be boolean", task.span.start.line, task.span.start.column);
        }
      }
      if (task.ensures) {
        const t = this.checkExpr(task.ensures);
        if (t.kind !== "bool") {
          this.error("ensures clause must be boolean", task.span.start.line, task.span.start.column);
        }
      }
      if (task.invariant) {
        const t = this.checkExpr(task.invariant);
        if (t.kind !== "bool") {
          this.error("invariant clause must be boolean", task.span.start.line, task.span.start.column);
        }
      }
      this.symbols.set(task.name, {
        name: task.name,
        roboType: { kind: "void" },
        kind: "behavior",
      });
      this.checkBehaviorBody(task.body);
    }

    for (const handler of robot.eventHandlers) {
      const declared = robot.events.some((e) => e.name === handler.eventName);
      if (!declared) {
        this.error(
          `No event declared for handler '${handler.eventName}'`,
          handler.span.start.line,
          handler.span.start.column,
        );
      }
      this.checkBehaviorBody(handler.body);
    }
  }

  private checkTraitImpl(decl: TraitImplDecl): void {
    const traitMethods = this.traitDefs.get(decl.traitName);
    if (!traitMethods) {
      this.error(`Unknown trait '${decl.traitName}'`, decl.span.start.line, decl.span.start.column);
      return;
    }
    const agentSym = this.symbols.get(decl.agentName);
    if (!agentSym || agentSym.kind !== "agent") {
      this.error(
        `Trait impl target '${decl.agentName}' is not a declared agent`,
        decl.span.start.line,
        decl.span.start.column,
      );
      return;
    }
    const registered: Array<[string, SpandaType]> = [];
    for (const method of decl.methods) {
      const expected = traitMethods.get(method.name);
      if (!expected) {
        this.error(
          `Trait '${decl.traitName}' has no method '${method.name}'`,
          method.span.start.line,
          method.span.start.column,
        );
        continue;
      }
      if (method.returnType !== expected.returnType) {
        this.error(
          `Trait method '${method.name}' return type mismatch: expected ${expected.returnType}, got ${method.returnType}`,
          method.span.start.line,
          method.span.start.column,
        );
      }
      if (method.params.length !== expected.params.length) {
        this.error(
          `Trait method '${method.name}' parameter count mismatch`,
          method.span.start.line,
          method.span.start.column,
        );
      }
      for (let i = 0; i < method.params.length; i++) {
        const actual = method.params[i];
        const exp = expected.params[i];
        if (!exp || actual.name !== exp.name || actual.typeName !== exp.typeName) {
          this.error(
            `Trait method '${method.name}' parameter '${exp?.name ?? actual.name}' type mismatch`,
            actual.span.start.line,
            actual.span.start.column,
          );
        }
      }
      const saved = new Map(this.symbols);
      for (const param of method.params) {
        this.symbols.set(param.name, {
          name: param.name,
          roboType: this.typeNameToSpanda(param.typeName),
          kind: "variable",
        });
      }
      for (const stmt of method.body) {
        this.checkStmt(stmt);
      }
      this.symbols = saved;
      registered.push([method.name, this.typeNameToSpanda(method.returnType)]);
    }
    const agentMethods = this.agentTraitMethods.get(decl.agentName) ?? new Map<string, SpandaType>();
    for (const [name, ret] of registered) {
      agentMethods.set(name, ret);
    }
    this.agentTraitMethods.set(decl.agentName, agentMethods);
  }

  private checkTopic(topic: TopicDecl): void {
    if (!this.resolveMessageType(topic.messageType)) {
      this.error(
        `Unknown message type '${topic.messageType}'`,
        topic.span.start.line,
        topic.span.start.column,
      );
    }
    if (topic.topic === null && topic.transport === null && topic.role === "publish") {
      this.error(
        `Topic '${topic.name}' publisher must specify path or transport`,
        topic.span.start.line,
        topic.span.start.column,
      );
    }
    if (topic.role === "subscribe" || topic.role === "both") {
      if (topic.topic) this.subscribedTopics.add(topic.topic);
      this.subscribedTopics.add(topic.name);
    }
    if (topic.qos) {
      if (topic.qos.rateHz !== null && topic.qos.rateHz <= 0) {
        this.error("Topic rate must be positive", topic.qos.span.start.line, topic.qos.span.start.column);
      }
      if (topic.qos.deadlineMs !== null && topic.qos.deadlineMs <= 0) {
        this.error("Topic deadline must be positive", topic.qos.span.start.line, topic.qos.span.start.column);
      }
    }
    if (topic.secure) this.checkSecureBlock(topic.secure);
    this.symbols.set(topic.name, {
      name: topic.name,
      roboType: this.resolveMessageType(topic.messageType) ?? { kind: "void" },
      kind: "topic",
      messageType: topic.messageType,
    });
  }

  private checkService(service: ServiceDecl): void {
    if (service.requestType && service.responseType) {
      if (!this.resolveMessageType(service.requestType)) {
        this.error(
          `Unknown service request type '${service.requestType}'`,
          service.span.start.line,
          service.span.start.column,
        );
      }
      if (!this.resolveMessageType(service.responseType)) {
        this.error(
          `Unknown service response type '${service.responseType}'`,
          service.span.start.line,
          service.span.start.column,
        );
      }
    } else if (service.serviceType) {
      if (!SERVICE_TYPES[service.serviceType]) {
        this.error(
          `Unknown service type '${service.serviceType}'`,
          service.span.start.line,
          service.span.start.column,
        );
      }
    } else {
      this.error(
        `Service '${service.name}' must specify type or request/response`,
        service.span.start.line,
        service.span.start.column,
      );
    }
    if (service.secure) this.checkSecureBlock(service.secure);
    this.symbols.set(service.name, {
      name: service.name,
      roboType: { kind: "named", name: service.name },
      kind: "service",
      serviceType: service.serviceType ?? undefined,
    });
  }

  private checkAction(action: ActionDecl): void {
    if (action.requestType && action.feedbackType && action.resultType) {
      for (const t of [action.requestType, action.feedbackType, action.resultType]) {
        if (!this.resolveMessageType(t)) {
          this.error(`Unknown action type '${t}'`, action.span.start.line, action.span.start.column);
        }
      }
    } else if (action.actionType) {
      if (!ACTION_TYPES[action.actionType]) {
        this.error(
          `Unknown action type '${action.actionType}'`,
          action.span.start.line,
          action.span.start.column,
        );
      }
    } else {
      this.error(
        `Action '${action.name}' must specify type or request/feedback/result`,
        action.span.start.line,
        action.span.start.column,
      );
    }
    if (action.secure) this.checkSecureBlock(action.secure);
    this.symbols.set(action.name, {
      name: action.name,
      roboType: { kind: "named", name: action.name },
      kind: "action",
      actionType: action.actionType ?? undefined,
    });
  }

  private checkSecureBlock(block: import("../foundations.js").SecureBlockDecl): void {
    if (block.minTrust && !parseTrustLevel(block.minTrust)) {
      this.error(
        `unknown trust level '${block.minTrust}' in secure block`,
        block.span.start.line,
        block.span.start.column,
      );
    }
    for (const cap of block.requires) {
      if (!isKnownCapability(cap)) {
        this.error(
          `unknown capability '${cap}' in secure block`,
          block.span.start.line,
          block.span.start.column,
        );
      }
    }
  }

  private checkSafetyRule(rule: SafetyRule): void {
    if (rule.kind === "MaxSpeedRule") {
      const t = this.checkExpr(rule.value);
      if (t.kind !== "number" || !unitsCompatible(t.unit, rule.unit)) {
        this.error(
          `Expected value with unit '${rule.unit}' for ${rule.name}`,
          rule.span.start.line,
          rule.span.start.column,
        );
      }
    } else {
      const t = this.checkExpr(rule.condition);
      if (t.kind !== "bool") {
        this.error("stop_if condition must be boolean", rule.span.start.line, rule.span.start.column);
      }
    }
  }

  private checkSafetyZone(zone: SafetyZoneDecl): void {
    const x = this.checkExpr(zone.x);
    const y = this.checkExpr(zone.y);
    if (x.kind !== "number" || y.kind !== "number") {
      this.error("Zone coordinates must be numeric", zone.span.start.line, zone.span.start.column);
    }
    if (zone.shape === "circle" && zone.radius) {
      const r = this.checkExpr(zone.radius);
      if (r.kind !== "number") {
        this.error("Zone radius must be numeric", zone.span.start.line, zone.span.start.column);
      }
    }
    if (zone.shape === "rect" && zone.width && zone.height) {
      const w = this.checkExpr(zone.width);
      const h = this.checkExpr(zone.height);
      if (w.kind !== "number" || h.kind !== "number") {
        this.error("Zone size must be numeric", zone.span.start.line, zone.span.start.column);
      }
    }
  }

  private checkAiModel(model: import("../ast/nodes.js").AiModelDecl): void {
    if (!AI_MODEL_TYPES[model.modelType]) {
      this.error(
        `Unknown AI model type '${model.modelType}'`,
        model.span.start.line,
        model.span.start.column,
      );
    }
    if (this.symbols.has(model.name)) {
      this.error(
        `Duplicate ai model name '${model.name}'`,
        model.span.start.line,
        model.span.start.column,
      );
    }
    this.symbols.set(model.name, {
      name: model.name,
      roboType: AI_MODEL_TYPES[model.modelType] ?? { kind: "void" },
      kind: "ai_model",
    });
  }

  private checkAgent(agent: import("../ast/nodes.js").AgentDecl): void {
    if (this.symbols.has(agent.name)) {
      this.error(
        `Duplicate agent name '${agent.name}`,
        agent.span.start.line,
        agent.span.start.column,
      );
    }
    for (const modelName of agent.usesAi) {
      const model = this.symbols.get(modelName);
      if (!model || model.kind !== "ai_model") {
        this.error(
          `Agent '${agent.name} references unknown ai model '${modelName}`,
          agent.span.start.line,
          agent.span.start.column,
        );
      }
    }
    for (const tool of agent.tools) {
      if (!this.symbols.has(tool)) {
        this.error(
          `Agent '${agent.name}' references unknown tool '${tool}'`,
          agent.span.start.line,
          agent.span.start.column,
        );
      }
    }
    for (const cap of agent.capabilities) {
      this.checkCapability(agent.name, cap);
    }
    this.agentNames.add(agent.name);
    this.symbols.set(agent.name, {
      name: agent.name,
      roboType: AI_VALUE_TYPES.Agent ?? { kind: "named", name: "Agent" },
      kind: "agent",
    });

    const saved = new Map(this.symbols);
    for (const stmt of agent.planBody) {
      this.checkStmt(stmt);
    }
    this.symbols = saved;
  }

  private checkCapability(agentName: string, cap: CapabilityDecl): void {
    const allowed = [
      "read",
      "propose_motion",
      "summarize",
      "detect",
      "plan",
      "subscribe",
      "publish",
      "call",
      "execute",
      "discover",
    ];
    if (!allowed.includes(cap.action) && !isCommCapability(cap.action)) {
      this.error(`Unknown capability '${cap.action}'`, cap.span.start.line, cap.span.start.column);
      return;
    }
    if (
      cap.action === "read" ||
      cap.action === "subscribe" ||
      cap.action === "publish" ||
      cap.action === "call" ||
      cap.action === "execute"
    ) {
      if (cap.target) {
        const valid =
          this.symbols.has(cap.target) ||
          this.peerRobotNames.has(cap.target) ||
          this.deviceNames.has(cap.target);
        if (!valid) {
          this.error(
            `Agent '${agentName}' capability ${cap.action}(${cap.target}) references unknown resource`,
            cap.span.start.line,
            cap.span.start.column,
          );
        }
      } else if (cap.action === "read" || cap.action === "subscribe" || cap.action === "publish") {
        this.error(
          `Agent '${agentName}' ${cap.action} capability requires a target`,
          cap.span.start.line,
          cap.span.start.column,
        );
      }
    }
  }

  private checkBehaviorBody(body: Stmt[]): void {
    const parentScope = new Map(this.symbols);
    this.symbols = new Map(parentScope);
    this.symbols.set("robot", {
      name: "robot",
      roboType: { kind: "named", name: "Robot" },
      kind: "robot",
    });
    for (const stmt of body) {
      this.checkStmt(stmt);
    }
    this.symbols = parentScope;
  }

  private checkStmt(stmt: Stmt): void {
    switch (stmt.kind) {
      case "VarDecl": {
        const inferred = stmt.init ? this.checkExpr(stmt.init) : null;
        let t: SpandaType;
        if (stmt.typeAnnotation && inferred) {
          this.assertCompatible(
            stmt.typeAnnotation,
            inferred,
            stmt.span.start.line,
            stmt.span.start.column,
          );
          t = stmt.typeAnnotation;
        } else if (stmt.typeAnnotation) {
          t = stmt.typeAnnotation;
        } else if (inferred) {
          t = inferred;
        } else {
          t = { kind: "void" };
        }
        this.symbols.set(stmt.name, {
          name: stmt.name,
          roboType: t,
          kind: "variable",
        });
        break;
      }
      case "IfStmt": {
        const cond = this.checkExpr(stmt.condition);
        if (cond.kind !== "bool") {
          this.error("if condition must be boolean", stmt.span.start.line, stmt.span.start.column);
        }
        for (const s of stmt.thenBranch) this.checkStmt(s);
        if (stmt.elseBranch) for (const s of stmt.elseBranch) this.checkStmt(s);
        break;
      }
      case "LoopStmt": {
        for (const s of stmt.body) this.checkStmt(s);
        break;
      }
      case "PublishStmt": {
        const topic = this.symbols.get(stmt.topicName);
        if (!topic || topic.kind !== "topic") {
          this.error(`Unknown topic '${stmt.topicName}'`, stmt.span.start.line, stmt.span.start.column);
        } else {
          const val = this.checkExpr(stmt.value);
          this.assertCompatible(topic.roboType, val, stmt.span.start.line, stmt.span.start.column);
        }
        break;
      }
      case "ServiceCallStmt": {
        const service = this.symbols.get(stmt.serviceName);
        if (!service || service.kind !== "service") {
          this.error(`Unknown service '${stmt.serviceName}'`, stmt.span.start.line, stmt.span.start.column);
        }
        break;
      }
      case "ActionSendStmt": {
        const action = this.symbols.get(stmt.actionName);
        if (!action || action.kind !== "action") {
          this.error(`Unknown action '${stmt.actionName}'`, stmt.span.start.line, stmt.span.start.column);
        } else {
          const goal = this.checkExpr(stmt.goal);
          if (goal.kind !== "pose" && goal.kind !== "trajectory") {
            this.error("Action goal must be pose or trajectory", stmt.span.start.line, stmt.span.start.column);
          }
        }
        break;
      }
      case "EmergencyStopStmt":
      case "ResetEmergencyStopStmt":
      case "EmitStmt":
        break;
      case "EnterStmt":
        if (!this.stateMachineStates.has(stmt.stateName)) {
          this.error(
            `Unknown state '${stmt.stateName}' for enter statement`,
            stmt.span.start.line,
            stmt.span.start.column,
          );
        }
        break;
      case "RememberStmt":
        this.checkExpr(stmt.value);
        break;
      case "SubscribeStmt": {
        const [topicName] = stmt.target.split(".");
        if (!this.symbols.has(topicName) && !this.peerRobotNames.has(topicName)) {
          this.error(
            `Unknown subscribe target '${stmt.target}'`,
            stmt.span.start.line,
            stmt.span.start.column,
          );
        }
        this.subscribedTopics.add(stmt.target);
        break;
      }
      case "ExecuteStmt": {
        const action = this.symbols.get(stmt.actionName);
        if (!action || action.kind !== "action") {
          this.error(
            `Unknown action '${stmt.actionName}'`,
            stmt.span.start.line,
            stmt.span.start.column,
          );
        } else {
          this.checkExpr(stmt.goal);
        }
        break;
      }
      case "DiscoverStmt":
        break;
      case "ReceiveStmt": {
        const topic = this.symbols.get(stmt.topicName);
        if (!topic || topic.kind !== "topic") {
          this.error(
            `Unknown topic '${stmt.topicName}' for receive`,
            stmt.span.start.line,
            stmt.span.start.column,
          );
        }
        this.symbols.set(stmt.varName, {
          name: stmt.varName,
          roboType: topic?.roboType ?? { kind: "void" },
          kind: "variable",
        });
        break;
      }
      case "ExprStmt":
        this.checkExpr(stmt.expr);
        break;
      case "ReturnStmt":
        if (stmt.value) this.checkExpr(stmt.value);
        break;
    }
  }

  private checkExpr(expr: Expr): SpandaType {
    switch (expr.kind) {
      case "LiteralExpr":
        if (typeof expr.value === "boolean") return { kind: "bool" };
        if (typeof expr.value === "number") return { kind: "number", unit: "none" };
        if (typeof expr.value === "string") return { kind: "string" };
        return { kind: "void" };

      case "UnitLiteralExpr":
        return { kind: "number", unit: expr.unit };

      case "IdentExpr": {
        const enumName = this.variantOwner.get(expr.name);
        if (enumName) {
          return { kind: "enum_variant", enumName, variant: expr.name };
        }
        const sym = this.symbols.get(expr.name);
        if (!sym) {
          this.error(`Undefined identifier '${expr.name}'`, expr.span.start.line, expr.span.start.column);
          return { kind: "void" };
        }
        return sym.roboType;
      }

      case "BinaryExpr": {
        const left = this.checkExpr(expr.left);
        const right = this.checkExpr(expr.right);
        if (
          ["+", "-", "<", "<=", ">", ">=", "==", "!="].includes(expr.op) &&
          !binaryPhysicalOpAllowed(expr.op, left, right)
        ) {
          this.error(
            `Invalid operation '${expr.op}' between incompatible types (${typeKindName(left)}, ${typeKindName(right)})`,
            expr.span.start.line,
            expr.span.start.column,
          );
        }
        const result = resultUnitForBinary(expr.op, left, right);
        if (!result) {
          this.error(
            `Invalid operation '${expr.op}' for types`,
            expr.span.start.line,
            expr.span.start.column,
          );
          return { kind: "void" };
        }
        return result;
      }

      case "UnaryExpr": {
        const operand = this.checkExpr(expr.operand);
        if (expr.op === "not" && operand.kind !== "bool") {
          this.error("Operand of 'not' must be boolean", expr.span.start.line, expr.span.start.column);
        }
        if (expr.op === "-" && operand.kind !== "number") {
          this.error("Operand of '-' must be numeric", expr.span.start.line, expr.span.start.column);
        }
        return expr.op === "not" ? { kind: "bool" } : operand;
      }

      case "MemberExpr":
        return this.checkMember(expr);

      case "CallExpr":
        return this.checkCall(expr);

      case "MatchExpr":
        return this.checkMatch(expr);

      case "StructLiteralExpr":
        return this.checkStructLiteral(expr);

      case "ServiceCallExpr": {
        const service = this.symbols.get(expr.serviceName);
        if (!service || service.kind !== "service") {
          this.error(
            `Unknown service '${expr.serviceName}'`,
            expr.span.start.line,
            expr.span.start.column,
          );
        }
        return { kind: "named", name: "ServiceResponse" };
      }
      case "ExecuteExpr": {
        const action = this.symbols.get(expr.actionName);
        if (!action || action.kind !== "action") {
          this.error(
            `Unknown action '${expr.actionName}'`,
            expr.span.start.line,
            expr.span.start.column,
          );
        } else {
          this.checkExpr(expr.goal);
        }
        return { kind: "named", name: "ActionResult" };
      }
      case "DiscoverExpr":
        return { kind: "named", name: "DiscoveryResult" };

      default:
        return { kind: "void" };
    }
  }

  private checkStructLiteral(expr: import("../ast/nodes.js").StructLiteralExpr): SpandaType {
    const def = this.structDefs.get(expr.typeName);
    if (!def) {
      this.error(`Unknown struct type '${expr.typeName}'`, expr.span.start.line, expr.span.start.column);
      return { kind: "void" };
    }
    const provided = new Set<string>();
    for (const field of expr.fields) {
      if (provided.has(field.name)) {
        this.error(`Duplicate struct field '${field.name}'`, field.span.start.line, field.span.start.column);
      }
      provided.add(field.name);
      const fieldDef = def.find((f) => f.name === field.name);
      if (!fieldDef) {
        this.error(
          `Struct '${expr.typeName}' has no field '${field.name}'`,
          field.span.start.line,
          field.span.start.column,
        );
        continue;
      }
      const expected = this.typeNameToSpanda(fieldDef.typeName);
      const actual = this.checkExpr(field.value);
      this.assertCompatible(expected, actual, field.span.start.line, field.span.start.column);
    }
    for (const { name } of def) {
      if (!provided.has(name)) {
        this.error(
          `Missing struct field '${name}' in '${expr.typeName}' literal`,
          expr.span.start.line,
          expr.span.start.column,
        );
      }
    }
    return { kind: "named", name: expr.typeName };
  }

  private checkMatch(expr: import("../ast/nodes.js").MatchExpr): SpandaType {
    this.checkExpr(expr.scrutinee);
    if (expr.arms.length === 0) {
      this.error("match expression requires at least one arm", expr.span.start.line, expr.span.start.column);
    }
    for (const arm of expr.arms) {
      for (const stmt of arm.body) {
        this.checkStmt(stmt);
      }
    }
    this.checkMatchExhaustiveness(expr.arms, expr.span);
    return { kind: "void" };
  }

  private checkMatchExhaustiveness(arms: MatchArm[], span: import("../ast/nodes.js").Span): void {
    const armNames = new Set(arms.map((a) => a.variant));
    if (armNames.size === 0) return;

    for (const variants of this.enumVariants.values()) {
      const variantSet = new Set(variants);
      if ([...armNames].every((name) => variantSet.has(name))) {
        if (armNames.size < variantSet.size) {
          const missing = variants.filter((v) => !armNames.has(v));
          this.error(
            `Non-exhaustive match: missing variants ${missing.join(", ")}`,
            span.start.line,
            span.start.column,
          );
        }
        return;
      }
    }
  }

  private checkMember(expr: import("../ast/nodes.js").MemberExpr): SpandaType {
    if (expr.object.kind === "IdentExpr") {
      const sym = this.symbols.get(expr.object.name);
      if (sym?.kind === "sensor" && sym.sensorType === "Lidar" && expr.property === "nearest_distance") {
        return { kind: "number", unit: "m" };
      }
    }

    const objType = this.checkExpr(expr.object);

    if (objType.kind === "scan") {
      const prop = SCAN_PROPERTIES[expr.property];
      if (!prop) {
        this.error(`Unknown scan property '${expr.property}'`, expr.span.start.line, expr.span.start.column);
        return { kind: "void" };
      }
      return prop;
    }

    if (objType.kind === "pose") {
      const prop = POSE_PROPERTIES[expr.property];
      if (!prop) {
        this.error(`Unknown pose property '${expr.property}'`, expr.span.start.line, expr.span.start.column);
        return { kind: "void" };
      }
      return prop;
    }

    if (objType.kind === "velocity") {
      const prop = VELOCITY_PROPERTIES[expr.property];
      if (!prop) {
        this.error(`Unknown velocity property '${expr.property}'`, expr.span.start.line, expr.span.start.column);
        return { kind: "void" };
      }
      return prop;
    }

    if (objType.kind === "named") {
      const enumVariants = this.enumVariants.get(objType.name);
      if (enumVariants?.includes(expr.property)) {
        return { kind: "enum_variant", enumName: objType.name, variant: expr.property };
      }
      const structFields = this.structDefs.get(objType.name);
      const structField = structFields?.find((f) => f.name === expr.property);
      if (structField) {
        return this.typeNameToSpanda(structField.typeName);
      }

      const objProps = OBJECT_PROPERTIES[objType.name];
      if (objProps?.[expr.property]) return objProps[expr.property];

      const methods = BUILTIN_METHODS[objType.name];
      if (methods?.[expr.property]) return methods[expr.property].returns;
    }

    this.error(`Unknown member '${expr.property}'`, expr.span.start.line, expr.span.start.column);
    return { kind: "void" };
  }

  private checkCall(expr: import("../ast/nodes.js").CallExpr): SpandaType {
    if (expr.callee.kind === "IdentExpr") {
      const fn = BUILTIN_FUNCTIONS[expr.callee.name];
      if (!fn) {
        this.error(`Unknown function '${expr.callee.name}'`, expr.span.start.line, expr.span.start.column);
        return { kind: "void" };
      }
      for (const arg of expr.namedArgs) {
        const expected = fn.namedParams[arg.name];
        if (!expected) {
          this.error(`Unknown named argument '${arg.name}'`, arg.span.start.line, arg.span.start.column);
          continue;
        }
        const actual = this.checkExpr(arg.value);
        this.assertCompatible(expected, actual, arg.span.start.line, arg.span.start.column);
      }
      return fn.returns;
    }

    if (expr.callee.kind !== "MemberExpr" || expr.callee.object.kind !== "IdentExpr") {
      this.error("Invalid call target", expr.span.start.line, expr.span.start.column);
      return { kind: "void" };
    }

    const member = expr.callee;
    if (member.object.kind !== "IdentExpr") {
      this.error("Invalid call target", expr.span.start.line, expr.span.start.column);
      return { kind: "void" };
    }
    const targetName = member.object.name;
    const sym = this.symbols.get(targetName);
    if (!sym) {
      this.error(`Undefined identifier '${targetName}'`, expr.span.start.line, expr.span.start.column);
      return { kind: "void" };
    }

    if (sym.kind === "robot") {
      const method = ROBOT_METHODS[member.property];
      if (!method) {
        this.error(`Unknown robot method '${member.property}`, expr.span.start.line, expr.span.start.column);
        return { kind: "void" };
      }
      for (let i = 0; i < expr.args.length; i++) {
        const expected = method.params[i];
        if (expected) {
          const actual = this.checkExpr(expr.args[i]);
          this.assertCompatible(expected, actual, expr.span.start.line, expr.span.start.column);
        }
      }
      return method.returns;
    }

    if (sym.kind === "agent") {
      const traitMethods = this.agentTraitMethods.get(targetName);
      if (traitMethods?.has(member.property)) {
        return traitMethods.get(member.property)!;
      }
      const agentMethod = BUILTIN_METHODS.Agent?.[member.property];
      if (!agentMethod) {
        this.error(`Unknown agent method '${member.property}`, expr.span.start.line, expr.span.start.column);
        return { kind: "void" };
      }
      return agentMethod.returns;
    }

    let typeName = "";
    if (sym.kind === "sensor" && sym.sensorType) typeName = sym.sensorType;
    else if (sym.kind === "actuator" && sym.actuatorType) typeName = sym.actuatorType;
    else if (sym.kind === "safety") typeName = "Safety";
    else if (sym.kind === "ai_model" && sym.roboType.kind === "named") typeName = sym.roboType.name;
    else if (sym.roboType.kind === "named") typeName = sym.roboType.name;
    else if (sym.roboType.kind === "scan") typeName = "Scan";

    const methods = BUILTIN_METHODS[typeName];
    const method = methods?.[member.property];
    if (!method) {
      this.error(
        `Unknown method '${member.property}' on ${typeName}`,
        expr.span.start.line,
        expr.span.start.column,
      );
      return { kind: "void" };
    }

    if (typeName === "LLM" && member.property === "drive") {
      this.error(
        "AI models cannot control actuators directly — use reason(), safety.validate(), then actuator.execute()",
        expr.span.start.line,
        expr.span.start.column,
      );
      return { kind: "void" };
    }

    if (method.namedParams) {
      for (const arg of expr.namedArgs) {
        const expected = method.namedParams[arg.name];
        if (!expected) {
          this.error(`Unknown named argument '${arg.name}'`, arg.span.start.line, arg.span.start.column);
          continue;
        }
        if (typeName === "Twin" && arg.name === "field" && arg.value.kind === "IdentExpr") {
          const allowedMirrorFields = ["pose", "velocity", "battery", "status", "scan"];
          if (!allowedMirrorFields.includes(arg.value.name)) {
            this.error(
              `Unknown twin mirror field '${arg.value.name}'`,
              arg.span.start.line,
              arg.span.start.column,
            );
          }
          continue;
        }
        const actual = this.checkExpr(arg.value);
        this.assertCompatible(expected, actual, arg.span.start.line, arg.span.start.column);
      }
    }

    for (const arg of expr.args) {
      const actual = this.checkExpr(arg);
      if (typeName === "Safety" && member.property === "validate" && !isActionProposalType(actual)) {
        this.error(
          "safety.validate() expects ActionProposal",
          expr.span.start.line,
          expr.span.start.column,
        );
      }
      if (typeName === "DifferentialDrive" && member.property === "execute") {
        if (isActionProposalType(actual)) {
          this.error(
            "ActionProposal cannot be passed to actuator.execute() — call safety.validate() first",
            expr.span.start.line,
            expr.span.start.column,
          );
        } else if (!isSafeActionType(actual)) {
          this.error(
            "actuator.execute() requires SafeAction from safety.validate()",
            expr.span.start.line,
            expr.span.start.column,
          );
        }
      }
      if (member.property === "detect" && typeName === "VisionModel") {
        this.assertNamedType(actual, "CameraFrame", expr.span.start.line, expr.span.start.column);
      }
    }

    if (member.property === "read" && typeName === "Lidar") {
      return { kind: "scan" };
    }

    return method.returns;
  }

  private typesCompatible(expected: SpandaType, actual: SpandaType): boolean {
    if (expected.kind === actual.kind) {
      if (expected.kind === "number" && actual.kind === "number") {
        return unitsCompatible(expected.unit, actual.unit);
      }
      if (expected.kind === "named" && actual.kind === "named") {
        return expected.name === actual.name || actual.name.includes(expected.name);
      }
      if (expected.kind === "enum_variant" && actual.kind === "enum_variant") {
        return expected.enumName === actual.enumName && expected.variant === actual.variant;
      }
      if (expected.kind === "generic" && actual.kind === "generic") {
        return (
          expected.name === actual.name &&
          expected.typeArgs.length === actual.typeArgs.length &&
          expected.typeArgs.every((e, i) => this.typesCompatible(e, actual.typeArgs[i]!))
        );
      }
      return true;
    }
    if (expected.kind === "named" && actual.kind === "enum_variant") {
      return expected.name === actual.enumName;
    }
    if (expected.kind === "enum_variant" && actual.kind === "named") {
      return expected.enumName === actual.name;
    }
    if (expected.kind === "named" && actual.kind === "scan" && expected.name.includes("Lidar")) {
      return true;
    }
    if (expected.kind === "scan" && actual.kind === "named") {
      return ["Detection", "CameraFrame", "Completion"].includes(actual.name);
    }
    if (
      expected.kind === "int" &&
      actual.kind === "number" &&
      actual.unit === "none"
    ) {
      return true;
    }
    if (expected.kind === "float" && actual.kind === "number") {
      return true;
    }
    if (
      (expected.kind === "velocity" &&
        actual.kind === "number" &&
        unitCategory(actual.unit) === "velocity") ||
      (actual.kind === "velocity" &&
        expected.kind === "number" &&
        unitCategory(expected.unit) === "velocity")
    ) {
      return true;
    }
    if (expected.kind === "named" && actual.kind === "number") {
      return unitMatchesNamedType(expected.name, actual.unit);
    }
    if (expected.kind === "named" && actual.kind === "string") {
      return expected.name === "Goal";
    }
    if (expected.kind === "string" && actual.kind === "named") {
      return actual.name === "Goal";
    }
    return false;
  }

  private assertNamedType(actual: SpandaType, typeName: string, line: number, column: number): void {
    if (actual.kind === "named" && actual.name === typeName) return;
    this.error(`Expected ${typeName}, got ${actual.kind}`, line, column);
  }

  private assertCompatible(expected: SpandaType, actual: SpandaType, line: number, column: number): void {
    if (expected.kind === "void" && actual.kind === "void") return;
    if (!this.typesCompatible(expected, actual)) {
      if (expected.kind === "number" && actual.kind === "number") {
        this.error(
          `Unit mismatch: expected '${expected.unit}', got '${actual.unit}'`,
          line,
          column,
        );
        return;
      }
      this.error(`Type mismatch: expected ${expected.kind}, got ${actual.kind}`, line, column);
    }
  }

  private error(message: string, line: number, column: number): void {
    this.errors.push({ message, line, column });
  }
}
