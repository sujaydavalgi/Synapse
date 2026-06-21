/**
 * checker module (types/checker.ts).
 * @module
 */

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
import type { CapabilityDecl, ExternFnDecl, MatchArm, ModuleFnDecl, TraitImplDecl } from "../foundations.js";
import { resolveModuleImport, resolveTypeAlias } from "../foundations.js";
import { resolveFfiImport } from "../ffi/registry.js";
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
import type { ModuleRegistry } from "../modules/index.js";
import { getSocProfile, validateHalAgainstSoc } from "../soc/index.js";
import {
  validateTaskTiming,
  validateTaskPriority,
  validatePipeline,
  validateWatchdog,
  validateResourceBudget,
  validateRecover,
} from "../reliability.js";
import { compileRegex, RegexError } from "../regex.js";
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

/** Built-in hardware profiles (mirrors Rust `hardware::builtin_profiles`). */
const BUILTIN_HARDWARE_PROFILES = new Set([
  "RoverV1",
  "RoverV2",
  "JetsonOrin",
  "RaspberryPi5",
  "ESP32",
]);

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
  // TypeCheck.
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

  // const result = typeCheck(program);
  check(program);
}

export function check(program: Program): void {
  // Check input.
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

  // const result = check(program);
  checkWithRegistry(program, undefined);
}

export function checkWithRegistry(program: Program, registry: ModuleRegistry | undefined): void {
  // CheckWithRegistry.
  //
  // Parameters:
  // - `program` — input value
  // - `registry` — input value
  //
  // Returns:
  // Nothing.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = checkWithRegistry(program, registry);
  const checker = new TypeChecker(registry);
  checker.checkProgram(program);

  // continue when checker.errors.length > 0.
  if (checker.errors.length > 0) {
    throw new TypeCheckError(checker.errors);
  }
}

class TypeChecker {
  errors: TypeError[] = [];
  private symbols = new Map<string, SymbolEntry>();
  private enumVariants = new Map<string, string[]>();
  private variantOwner = new Map<string, string>();
  private enumPayloadFields = new Map<string, string[]>();
  private structTypeParams = new Map<string, string[]>();
  private structDefs = new Map<string, Array<{ name: string; typeName: string }>>();
  private traitDefs = new Map<string, Map<string, { params: Array<{ name: string; typeName: string }>; returnType: string }>>();
  private agentTraitMethods = new Map<string, Map<string, SpandaType>>();
  private agentTraits = new Map<string, Set<string>>();
  private stateMachineStates = new Set<string>();
  private currentRobot: RobotDecl | null = null;
  private messageRegistry = MessageRegistry.new();
  private subscribedTopics = new Set<string>();
  private agentNames = new Set<string>();
  private deviceNames = new Set<string>();
  private peerRobotNames = new Set<string>();
  private moduleFunctions = new Map<string, ModuleFnDecl>();
  private externFunctions = new Map<string, ExternFnDecl>();
  private typeParamScope = new Map<string, SpandaType>();
  private channelPayloadTypes = new Map<string, SpandaType>();
  private activeAgent: string | null = null;

  constructor(private moduleRegistry?: ModuleRegistry) {}

  checkProgram(program: Program): void {
    // CheckProgram.
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

    // const result = checkProgram(program);

    this.moduleFunctions.clear();
    this.externFunctions.clear();
    const imported = new Set<string>();
    for (const imp of program.imports) {
      const registryExport = this.moduleRegistry?.exportsFor(imp.path);
      if (
        !resolveImport(imp.path) &&
        !resolveAiImport(imp.path) &&
        !resolveModuleImport(imp.path) &&
        !resolveStdImport(imp.path) &&
        !resolveFfiImport(imp.path) &&
        !registryExport
      ) {
        this.error(`Unknown import '${imp.path}'`, imp.span.start.line, imp.span.start.column);
      } else {
        imported.add(imp.path);
        if (registryExport) {
          for (const [fname, fdecl] of registryExport.functions) {
            this.moduleFunctions.set(fname, fdecl);
          }
        }
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

    this.checkExternFunctions(program.externFunctions);
    this.checkModuleFunctions(program.functions);

    for (const test of program.tests) {
      for (const stmt of test.body) {
        this.checkStmt(stmt);
      }
    }

    this.messageRegistry = MessageRegistry.fromProgram(program.messages, program.structs);
    for (const msg of program.messages) {
      this.checkMessage(msg);
    }

    for (const rule of program.validateRules) {
      try {
        compileRegex(rule.pattern);
      } catch (err) {
        if (err instanceof RegexError) {
          this.error(err.message, err.line, err.column);
        } else {
          throw err;
        }
      }
    }

    for (const robot of program.robots) {
      this.checkRobot(robot, imported);
    }

    this.checkHardwareProgram(program);
  }

  private checkHardwareProgram(program: Program): void {
    // CheckHardwareProgram.
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

    // const result = checkHardwareProgram(program);
    const profileNames = new Set(program.hardwareProfiles.map((p) => p.name));
    const robotNames = new Set(program.robots.map((r) => r.name));
    const seenProfiles = new Set<string>();

    // Process each hardwareProfile.
    for (const profile of program.hardwareProfiles) {

      // continue when seenProfiles.has(profile.name).
      if (seenProfiles.has(profile.name)) {
        this.error(
          `Duplicate hardware profile '${profile.name}'`,
          profile.span.start.line,
          profile.span.start.column,
        );
      }
      seenProfiles.add(profile.name);
    }

    // Process each deployment.
    for (const deploy of program.deployments) {

      // continue when robotName) is falsy.
      if (!robotNames.has(deploy.robotName)) {
        this.error(
          `Deploy references unknown robot '${deploy.robotName}'`,
          deploy.span.start.line,
          deploy.span.start.column,
        );
      }

      // Process each target.
      for (const target of deploy.targets) {

        // continue when has is falsy.
        if (!profileNames.has(target) && !BUILTIN_HARDWARE_PROFILES.has(target)) {
          this.error(
            `Deploy target '${target}' is not a declared or built-in hardware profile`,
            deploy.span.start.line,
            deploy.span.start.column,
          );
        }
      }
    }
}

  private checkMessage(decl: import("../comm/index.js").MessageDecl): void {    // Process each field.
    for (const field of decl.fields) {
      let known = !!this.messageRegistry.resolveType(field.typeName) || !!resolveTypeAlias(field.typeName);

      // continue when known is falsy.
      if (!known) {

        // Try the operation and handle failures below.
        try {
          resolveTypeName(field.typeName);
          known = true;
        } catch {
          /* unknown */
        }
      }

      // continue when known is falsy.
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
    // ResolveMessageType.
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

    // const result = resolveMessageType(name);
    return this.messageRegistry.resolveType(name);
}

  private checkStruct(decl: import("../foundations.js").StructDecl): void {
    // CheckStruct.
    //
    // Parameters:
    // - `decl` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkStruct(decl);
    const typeParams = decl.typeParams ?? [];

    // continue when typeParams.length > 0.
    if (typeParams.length > 0) {
      this.structTypeParams.set(decl.name, typeParams);
    }

    // Process each field.
    for (const field of decl.fields) {
      const allowedGeneric = typeParams.includes(field.typeName);

      // continue when value.
      if (
        !allowedGeneric &&
        !resolveTypeAlias(field.typeName) &&
        !["Pose", "Velocity", "Scan", "String", "Bool", "Path", "Int", "Float"].includes(field.typeName) &&
        !field.typeName.includes("<")
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

  private enumPayloadKey(enumName: string, variant: string): string {
    // EnumPayloadKey.
    //
    // Parameters:
    // - `enumName` — input value
    // - `variant` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = enumPayloadKey(enumName, variant);
    return `${enumName}\0${variant}`;
}

  private checkEnum(decl: import("../foundations.js").EnumDecl): void {
    // CheckEnum.
    //
    // Parameters:
    // - `decl` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkEnum(decl);
    if (decl.variants.length === 0) {
      this.error(`Enum '${decl.name}' must declare at least one variant`, decl.span.start.line, decl.span.start.column);
    }
    this.symbols.set(decl.name, {
      name: decl.name,
      roboType: { kind: "named", name: decl.name },
      kind: "variable",
    });
    const variantNames = decl.variants.map((v) => v.name);
    this.enumVariants.set(decl.name, variantNames);

    // Process each variant.
    for (const variant of decl.variants) {

      // continue when variant.fieldTypes.length > 0.
      if (variant.fieldTypes.length > 0) {
        this.enumPayloadFields.set(
          this.enumPayloadKey(decl.name, variant.name),
          variant.fieldTypes,
        );
      }
      const existing = this.variantOwner.get(variant.name);

      // continue when existing.
      if (existing) {
        this.error(
          `Enum variant '${variant.name}' already declared in enum '${existing}'`,
          decl.span.start.line,
          decl.span.start.column,
        );

      // Handle any remaining cases.
      } else {
        this.variantOwner.set(variant.name, decl.name);
      }
    }
}

  private checkTrait(decl: import("../foundations.js").TraitDecl): void {    // continue when length equals 0.
    if (decl.methods.length === 0) {
      this.error(`Trait '${decl.name}' must declare at least one method`, decl.span.start.line, decl.span.start.column);
    }
    const methodMap = new Map<string, { params: Array<{ name: string; typeName: string }>; returnType: string }>();

    // Process each method.
    for (const method of decl.methods) {
      methodMap.set(method.name, {
        params: method.params.map((p) => ({ name: p.name, typeName: p.typeName })),
        returnType: method.returnType,
      });
    }
    this.traitDefs.set(decl.name, methodMap);
}

  private typeNameToSpanda(typeName: string): SpandaType {
    // TypeNameToSpanda.
    //
    // Parameters:
    // - `typeName` — input value
    //
    // Returns:
    // SpandaType.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = typeNameToSpanda(typeName);
    try {
      return resolveTypeName(typeName);
    } catch {

      // Branch on resolveTypeAlias.
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

  private validateTypeAnnotation(ty: SpandaType, line: number, column: number): void {
    // ValidateTypeAnnotation.
    //
    // Parameters:
    // - `ty` — input value
    // - `line` — input value
    // - `column` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = validateTypeAnnotation(ty, line, column);
    if (ty.kind === "named") {

      // continue when value.
      if (
        this.structDefs.has(ty.name) ||
        this.typeParamScope.has(ty.name) ||
        this.enumVariants.has(ty.name)
      ) {
        return;
      }

      // Try the operation and handle failures below.
      try {
        resolveTypeName(ty.name);
      } catch {
        this.error(`Unknown type '${ty.name}'`, line, column);
      }
      return;
    }

    // continue when kind equals "generic".
    if (ty.kind === "generic") {

      // Apply each command-line argument.
      for (const arg of ty.typeArgs) {
        this.validateTypeAnnotation(arg, line, column);
      }
      return;
    }

    // continue when kind equals "trait object".
    if (ty.kind === "trait_object") {

      // continue when traitName) is falsy.
      if (!this.traitDefs.has(ty.traitName)) {
        this.error(`Unknown trait '${ty.traitName}'`, line, column);
      }
    }
}

  private resolveTypeAnn(ty: SpandaType): SpandaType {
    // ResolveTypeAnn.
    //
    // Parameters:
    // - `ty` — input value
    //
    // Returns:
    // SpandaType.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = resolveTypeAnn(ty);
    if (ty.kind === "named" && this.typeParamScope.has(ty.name)) {
      return this.typeParamScope.get(ty.name)!;
    }

    // continue when kind equals "generic".
    if (ty.kind === "generic") {
      return {
        kind: "generic",
        name: ty.name,
        typeArgs: ty.typeArgs.map((a) => this.resolveTypeAnn(a)),
      };
    }
    return ty;
}

  private static futureType(inner: SpandaType): SpandaType {
    // FutureType.
    //
    // Parameters:
    // - `inner` — input value
    //
    // Returns:
    // SpandaType.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = futureType(inner);
    return { kind: "generic", name: "Future", typeArgs: [inner] };
}

  private static taskHandleType(inner: SpandaType): SpandaType {
    // TaskHandleType.
    //
    // Parameters:
    // - `inner` — input value
    //
    // Returns:
    // SpandaType.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = taskHandleType(inner);
    return { kind: "generic", name: "TaskHandle", typeArgs: [inner] };
}

  private checkModuleFunctions(functions: ModuleFnDecl[]): void {
    // CheckModuleFunctions.
    //
    // Parameters:
    // - `functions` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkModuleFunctions(functions);
    for (const func of functions) {
      const savedScope = new Map(this.typeParamScope);
      this.typeParamScope.clear();

      // Process each typeParam.
      for (const tp of func.typeParams) {
        this.typeParamScope.set(tp, { kind: "named", name: tp });
      }

      // Bind each parameter before continuing.
      for (const param of func.params) {
        this.validateTypeAnnotation(param.typeAnn, param.span.start.line, param.span.start.column);
        const resolved = this.resolveTypeAnn(param.typeAnn);
        this.symbols.set(param.name, {
          name: param.name,
          roboType: resolved,
          kind: "variable",
        });
      }

      // Execute each statement in sequence.
      for (const stmt of func.body) {
        this.checkStmt(stmt);
      }

      // Bind each parameter before continuing.
      for (const param of func.params) {
        this.symbols.delete(param.name);
      }

      // continue when visibility equals visibility === "public".
      if (func.visibility === "export" || func.visibility === "public") {
        this.moduleFunctions.set(func.name, func);
      }
      this.resolveTypeAnn(func.returnType);
      this.typeParamScope = savedScope;
    }
}

  private checkExternFunctions(functions: ExternFnDecl[]): void {
    // CheckExternFunctions.
    //
    // Parameters:
    // - `functions` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkExternFunctions(functions);
    for (const func of functions) {

      // Bind each parameter before continuing.
      for (const param of func.params) {
        this.validateTypeAnnotation(param.typeAnn, param.span.start.line, param.span.start.column);
      }
      this.resolveTypeAnn(func.returnType);
      this.externFunctions.set(func.name, func);
    }
}

  private checkRobot(robot: RobotDecl, imported: Set<string>): void {
    // CheckRobot.
    //
    // Parameters:
    // - `robot` — input value
    // - `imported` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkRobot(robot, imported);
    this.currentRobot = robot;
    this.subscribedTopics.clear();
    this.agentNames.clear();
    this.deviceNames.clear();
    this.peerRobotNames.clear();
    this.symbols.clear();
    this.stateMachineStates.clear();
    this.agentTraitMethods.clear();

    // Process each key.
    for (const enumName of this.enumVariants.keys()) {
      this.symbols.set(enumName, {
        name: enumName,
        roboType: { kind: "named", name: enumName },
        kind: "variable",
      });
    }

    // Process each key.
    for (const structName of this.structDefs.keys()) {
      this.symbols.set(structName, {
        name: structName,
        roboType: { kind: "named", name: structName },
        kind: "variable",
      });
    }

    // continue when robot.soc.
    if (robot.soc) {

      // continue when profile) is falsy.
      if (!getSocProfile(robot.soc.profile)) {
        this.error(`Unknown SoC profile '${robot.soc.profile}'`, robot.soc.span.start.line, robot.soc.span.start.column);
      }
    }

    // continue when robot.hal && robot.soc.
    if (robot.hal && robot.soc) {
      const profile = getSocProfile(robot.soc.profile);

      // continue when profile.
      if (profile) {
        const members = robot.hal.members.map(halMemberFromDecl);

        // Iterate over validateHalAgainstSoc.
        for (const err of validateHalAgainstSoc(profile, members)) {
          this.error(err.message, robot.hal.span.start.line, robot.hal.span.start.column);
        }
      }
    }
    const halBusNames = new Set(robot.hal?.members.map((m) => m.name) ?? []);

    // Visit each AST node.
    for (const node of robot.nodes) {

      // continue when namespace is falsy.
      if (!node.namespace) {
        this.error("Node should specify namespace with 'on \"/namespace\"'", node.span.start.line, node.span.start.column);
      }
    }

    // Process each topic.
    for (const topic of robot.topics) {
      this.checkTopic(topic);
    }

    // Process each service.
    for (const service of robot.services) {
      this.checkService(service);
    }

    // Process each action.
    for (const action of robot.actions) {
      this.checkAction(action);
    }

    // Process each sensor.
    for (const sensor of robot.sensors) {

      // continue when sensorType] is falsy.
      if (!SENSOR_TYPES[sensor.sensorType]) {
        this.error(`Unknown sensor type '${sensor.sensorType}'`, sensor.span.start.line, sensor.span.start.column);
      }

      // continue when sensor.library.
      if (sensor.library) {

        // continue when library) is falsy.
        if (!imported.has(sensor.library)) {
          this.error(`Library '${sensor.library}' must be imported before use`, sensor.span.start.line, sensor.span.start.column);
        }
        const lib = resolveImport(sensor.library);

        // continue when lib && !lib.sensors[sensor.sensorType].
        if (lib && !lib.sensors[sensor.sensorType]) {
          this.error(`Sensor type '${sensor.sensorType}' not provided by library '${sensor.library}'`, sensor.span.start.line, sensor.span.start.column);
        }
      }

      // continue when kind equals busName).
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

    // Process each actuator.
    for (const actuator of robot.actuators) {

      // continue when actuatorType] is falsy.
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

    // Process each buse.
    for (const bus of robot.buses) {

      // continue when name) equals transport === "local".
      if (transportFromIdent(bus.name) === null && bus.transport === "local") {
        this.error(
          `Unknown transport '${bus.name}' in bus declaration`,
          bus.span.start.line,
          bus.span.start.column,
        );
      }
    }

    // Process each peerRobot.
    for (const peer of robot.peerRobots) {
      this.peerRobotNames.add(peer.name);
      this.symbols.set(peer.name, {
        name: peer.name,
        roboType: { kind: "named", name: "PeerRobot" },
        kind: "robot",
      });
    }

    // Process each device.
    for (const device of robot.devices) {

      // continue when deviceType) is falsy.
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

    // continue when robot.safety.
    if (robot.safety) {
      const saved = new Map(this.symbols);
      this.symbols.set("robot", {
        name: "robot",
        roboType: { kind: "named", name: "Robot" },
        kind: "robot",
      });

      // Process each rule.
      for (const rule of robot.safety.rules) {
        this.checkSafetyRule(rule);
      }

      // Process each zone.
      for (const zone of robot.safety.zones) {
        this.checkSafetyZone(zone);
      }
      this.symbols = saved;
    }

    // continue when robot.ai models.length > 0.
    if (robot.ai_models.length > 0) {

      // Iterate over ai models ?? [].
      for (const model of robot.ai_models ?? []) {
        this.checkAiModel(model);
      }
    }

    // continue when robot.safety.
    if (robot.safety) {
      this.symbols.set("safety", {
        name: "safety",
        roboType: { kind: "named", name: "Safety" },
        kind: "safety",
      });
    }

    // Process each agent.
    for (const agent of robot.agents) {
      this.checkAgent(agent);
    }

    // Process each agentChannel.
    for (const channel of robot.agentChannels) {

      // continue when fromAgent) is falsy.
      if (!this.agentNames.has(channel.fromAgent)) {
        this.error(
          `Agent channel source '${channel.fromAgent}' is not declared`,
          channel.span.start.line,
          channel.span.start.column,
        );
      }

      // continue when toAgent) is falsy.
      if (!this.agentNames.has(channel.toAgent)) {
        this.error(
          `Agent channel target '${channel.toAgent}' is not declared`,
          channel.span.start.line,
          channel.span.start.column,
        );
      }
    }

    // Process each traitImpl.
    for (const traitImpl of robot.traitImpls) {
      this.checkTraitImpl(traitImpl);
    }

    // Process each stateMachine.
    for (const sm of robot.stateMachines) {

      // continue when length equals 0.
      if (sm.states.length === 0) {
        this.error(
          `State machine '${sm.name}' must declare at least one state`,
          sm.span.start.line,
          sm.span.start.column,
        );
      }
      const stateSet = new Set(sm.states);

      // Process each transition.
      for (const transition of sm.transitions) {

        // continue when to) is falsy.
        if (!stateSet.has(transition.from) || !stateSet.has(transition.to)) {
          this.error(
            `Invalid transition ${transition.from} -> ${transition.to} in state machine '${sm.name}'`,
            transition.span.start.line,
            transition.span.start.column,
          );
        }
      }

      // Process each state.
      for (const state of sm.states) {
        this.stateMachineStates.add(state);
      }
    }

    // continue when robot.twin.
    if (robot.twin) {

      // continue when length equals 0.
      if (robot.twin.mirrors.length === 0) {
        this.error("Digital twin must mirror at least one field", robot.twin.span.start.line, robot.twin.span.start.column);
      }
      const allowedMirrorFields = [
        "pose",
        "velocity",
        "battery",
        "status",
        "scan",
        ...robot.sensors.map((s) => s.name),
        ...robot.actuators.map((a) => a.name),
      ];

      // Process each mirror.
      for (const mirror of robot.twin.mirrors) {

        // continue when includes is falsy.
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

    // continue when robot.verify.
    if (robot.verify) {
      const saved = new Map(this.symbols);
      this.symbols.set("robot", {
        name: "robot",
        roboType: { kind: "named", name: "Robot" },
        kind: "robot",
      });

      // Process each rule.
      for (const rule of robot.verify.rules) {
        const t = this.checkExpr(rule);

        // continue when kind differs from "bool".
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

    // continue when robot.observe.
    if (robot.observe) {

      // continue when length equals 0.
      if (robot.observe.sensors.length === 0) {
        this.error(
          "observe block must list at least one sensor",
          robot.observe.span.start.line,
          robot.observe.span.start.column,
        );
      }

      // Process each sensor.
      for (const sensorName of robot.observe.sensors) {
        const sym = this.symbols.get(sensorName);

        // continue when kind differs from "sensor".
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

    // continue when robot.identity.
    if (robot.identity) {

      // continue when some equals "id").
      if (!robot.identity.fields.some(([k]) => k === "id")) {
        this.error("identity block must declare an 'id' field", robot.identity.span.start.line, robot.identity.span.start.column);
      }
      this.symbols.set("identity", {
        name: "identity",
        roboType: { kind: "named", name: "RobotIdentity" },
        kind: "variable",
      });
    }

    // continue when robot.audit.
    if (robot.audit) {

      // continue when length equals 0.
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

    // Iterate over secrets ?? [].
    for (const secret of robot.secrets ?? []) {
      this.symbols.set(secret.name, {
        name: secret.name,
        roboType: { kind: "named", name: "Secret" },
        kind: "variable",
      });
    }

    // continue when robot.trust.
    if (robot.trust) {

      // continue when level) is falsy.
      if (!parseTrustLevel(robot.trust.level)) {
        this.error(`unknown trust level '${robot.trust.level}'`, robot.trust.span.start.line, robot.trust.span.start.column);
      }
    }

    // continue when robot.permissions.
    if (robot.permissions) {

      // continue when length equals 0.
      if (robot.permissions.capabilities.length === 0) {
        this.error("permissions block must grant at least one capability", robot.permissions.span.start.line, robot.permissions.span.start.column);
      }

      // Process each capabilitie.
      for (const cap of robot.permissions.capabilities) {

        // continue when isKnownCapability is falsy.
        if (!isKnownCapability(cap)) {
          this.error(`unknown package capability '${cap}'`, robot.permissions.span.start.line, robot.permissions.span.start.column);
        }
      }
    }

    // Process each behavior.
    for (const behavior of robot.behaviors) {

      // continue when behavior.requires.
      if (behavior.requires) {
        const t = this.checkExpr(behavior.requires);

        // continue when kind differs from "bool".
        if (t.kind !== "bool") {
          this.error("requires clause must be boolean", behavior.span.start.line, behavior.span.start.column);
        }
      }

      // continue when behavior.ensures.
      if (behavior.ensures) {
        const t = this.checkExpr(behavior.ensures);

        // continue when kind differs from "bool".
        if (t.kind !== "bool") {
          this.error("ensures clause must be boolean", behavior.span.start.line, behavior.span.start.column);
        }
      }

      // continue when behavior.invariant.
      if (behavior.invariant) {
        const t = this.checkExpr(behavior.invariant);

        // continue when kind differs from "bool".
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

    // Process each task.
    for (const task of robot.tasks) {
      for (const diag of validateTaskTiming(task)) {
        this.error(diag.message, diag.line, diag.column);
      }
      for (const diag of validateTaskPriority(task)) {
        this.error(diag.message, diag.line, diag.column);
      }

      if (task.budget) {
        for (const diag of validateResourceBudget(task.budget, task.span)) {
          this.error(diag.message, diag.line, diag.column);
        }
      }

      // continue when task.intervalMs <= 0.
      if (task.intervalMs <= 0) {
        this.error("task interval must be positive", task.span.start.line, task.span.start.column);

      // Otherwise, continue when task.intervalMs < 1.
      } else if (task.intervalMs < 1) {
        this.error("task interval must be at least 1ms", task.span.start.line, task.span.start.column);
      }

      // continue when task.requires.
      if (task.requires) {
        const t = this.checkExpr(task.requires);

        // continue when kind differs from "bool".
        if (t.kind !== "bool") {
          this.error("requires clause must be boolean", task.span.start.line, task.span.start.column);
        }
      }

      // continue when task.ensures.
      if (task.ensures) {
        const t = this.checkExpr(task.ensures);

        // continue when kind differs from "bool".
        if (t.kind !== "bool") {
          this.error("ensures clause must be boolean", task.span.start.line, task.span.start.column);
        }
      }

      // continue when task.invariant.
      if (task.invariant) {
        const t = this.checkExpr(task.invariant);

        // continue when kind differs from "bool".
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

    const taskNames = robot.tasks.map((task) => task.name);

    for (const pipeline of robot.pipelines) {
      for (const diag of validatePipeline(pipeline)) {
        this.error(diag.message, diag.line, diag.column);
      }
      this.checkBehaviorBody(pipeline.body);
    }

    for (const watchdog of robot.watchdogs) {
      for (const diag of validateWatchdog(watchdog, taskNames)) {
        this.error(diag.message, diag.line, diag.column);
      }
      this.checkBehaviorBody(watchdog.body);
    }

    for (const mode of robot.modes) {
      this.checkBehaviorBody(mode.body);
    }

    for (const retry of robot.retries) {
      this.checkBehaviorBody(retry.body);
      this.checkBehaviorBody(retry.fallback);
    }

    for (const recover of robot.recovers) {
      for (const diag of validateRecover(recover)) {
        this.error(diag.message, diag.line, diag.column);
      }
      this.checkBehaviorBody(recover.body);
    }

    // Invoke each registered handler.
    for (const handler of robot.eventHandlers) {
      const isTriggerHandler =
        handler.eventName === "log" ||
        handler.eventName.startsWith("hardware.") ||
        handler.eventName.startsWith("message.") ||
        handler.eventName.startsWith("geofence:") ||
        /^[a-z]+\.[a-z_]+$/.test(handler.eventName);
      const declared = robot.events.some((e) => e.name === handler.eventName);

      // continue when declared is falsy.
      if (!isTriggerHandler && !declared) {
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
    // CheckTraitImpl.
    //
    // Parameters:
    // - `decl` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkTraitImpl(decl);
    const traitMethods = this.traitDefs.get(decl.traitName);

    // continue when traitMethods is falsy.
    if (!traitMethods) {
      this.error(`Unknown trait '${decl.traitName}'`, decl.span.start.line, decl.span.start.column);
      return;
    }
    const agentSym = this.symbols.get(decl.agentName);

    // continue when kind differs from "agent".
    if (!agentSym || agentSym.kind !== "agent") {
      this.error(
        `Trait impl target '${decl.agentName}' is not a declared agent`,
        decl.span.start.line,
        decl.span.start.column,
      );
      return;
    }
    const registered: Array<[string, SpandaType]> = [];

    // Process each method.
    for (const method of decl.methods) {
      const expected = traitMethods.get(method.name);

      // continue when expected is falsy.
      if (!expected) {
        this.error(
          `Trait '${decl.traitName}' has no method '${method.name}'`,
          method.span.start.line,
          method.span.start.column,
        );
        continue;
      }

      // continue when returnType differs from returnType.
      if (method.returnType !== expected.returnType) {
        this.error(
          `Trait method '${method.name}' return type mismatch: expected ${expected.returnType}, got ${method.returnType}`,
          method.span.start.line,
          method.span.start.column,
        );
      }

      // continue when length differs from length.
      if (method.params.length !== expected.params.length) {
        this.error(
          `Trait method '${method.name}' parameter count mismatch`,
          method.span.start.line,
          method.span.start.column,
        );
      }

      // Loop with index variable i.
      for (let i = 0; i < method.params.length; i++) {
        const actual = method.params[i];
        const exp = expected.params[i];

        // continue when name differs from typeName.
        if (!exp || actual.name !== exp.name || actual.typeName !== exp.typeName) {
          this.error(
            `Trait method '${method.name}' parameter '${exp?.name ?? actual.name}' type mismatch`,
            actual.span.start.line,
            actual.span.start.column,
          );
        }
      }
      const saved = new Map(this.symbols);

      // Bind each parameter before continuing.
      for (const param of method.params) {
        this.symbols.set(param.name, {
          name: param.name,
          roboType: this.typeNameToSpanda(param.typeName),
          kind: "variable",
        });
      }

      // Execute each statement in sequence.
      for (const stmt of method.body) {
        this.checkStmt(stmt);
      }
      this.symbols = saved;
      registered.push([method.name, this.typeNameToSpanda(method.returnType)]);
    }
    const agentMethods = this.agentTraitMethods.get(decl.agentName) ?? new Map<string, SpandaType>();

    // Iterate over the collection.
    for (const [name, ret] of registered) {
      agentMethods.set(name, ret);
    }
    this.agentTraitMethods.set(decl.agentName, agentMethods);
    const traits = this.agentTraits.get(decl.agentName) ?? new Set<string>();
    traits.add(decl.traitName);
    this.agentTraits.set(decl.agentName, traits);
}

  private checkTopic(topic: TopicDecl): void {
    // CheckTopic.
    //
    // Parameters:
    // - `topic` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkTopic(topic);
    if (!this.resolveMessageType(topic.messageType)) {
      this.error(
        `Unknown message type '${topic.messageType}'`,
        topic.span.start.line,
        topic.span.start.column,
      );
    }

    // continue when topic equals role === "publish".
    if (topic.topic === null && topic.transport === null && topic.role === "publish") {
      this.error(
        `Topic '${topic.name}' publisher must specify path or transport`,
        topic.span.start.line,
        topic.span.start.column,
      );
    }

    // continue when role equals role === "both".
    if (topic.role === "subscribe" || topic.role === "both") {

      // continue when topic.topic) this.subscribedTopics.add(topic.topic.
      if (topic.topic) this.subscribedTopics.add(topic.topic);
      this.subscribedTopics.add(topic.name);
    }

    // continue when topic.qos.
    if (topic.qos) {

      // continue when rateHz differs from rateHz <= 0.
      if (topic.qos.rateHz !== null && topic.qos.rateHz <= 0) {
        this.error("Topic rate must be positive", topic.qos.span.start.line, topic.qos.span.start.column);
      }

      // continue when deadlineMs differs from deadlineMs <= 0.
      if (topic.qos.deadlineMs !== null && topic.qos.deadlineMs <= 0) {
        this.error("Topic deadline must be positive", topic.qos.span.start.line, topic.qos.span.start.column);
      }
    }

    // continue when topic.secure) this.checkSecureBlock(topic.secure.
    if (topic.secure) this.checkSecureBlock(topic.secure);
    this.symbols.set(topic.name, {
      name: topic.name,
      roboType: this.resolveMessageType(topic.messageType) ?? { kind: "void" },
      kind: "topic",
      messageType: topic.messageType,
    });
}

  private checkService(service: ServiceDecl): void {
    // CheckService.
    //
    // Parameters:
    // - `service` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkService(service);
    if (service.requestType && service.responseType) {

      // continue when requestType) is falsy.
      if (!this.resolveMessageType(service.requestType)) {
        this.error(
          `Unknown service request type '${service.requestType}'`,
          service.span.start.line,
          service.span.start.column,
        );
      }

      // continue when responseType) is falsy.
      if (!this.resolveMessageType(service.responseType)) {
        this.error(
          `Unknown service response type '${service.responseType}'`,
          service.span.start.line,
          service.span.start.column,
        );
      }

    // Otherwise, continue when service.serviceType.
    } else if (service.serviceType) {

      // continue when serviceType] is falsy.
      if (!SERVICE_TYPES[service.serviceType]) {
        this.error(
          `Unknown service type '${service.serviceType}'`,
          service.span.start.line,
          service.span.start.column,
        );
      }

    // Handle any remaining cases.
    } else {
      this.error(
        `Service '${service.name}' must specify type or request/response`,
        service.span.start.line,
        service.span.start.column,
      );
    }

    // continue when service.secure) this.checkSecureBlock(service.secure.
    if (service.secure) this.checkSecureBlock(service.secure);
    this.symbols.set(service.name, {
      name: service.name,
      roboType: { kind: "named", name: service.name },
      kind: "service",
      serviceType: service.serviceType ?? undefined,
    });
}

  private checkAction(action: ActionDecl): void {
    // CheckAction.
    //
    // Parameters:
    // - `action` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkAction(action);
    if (action.requestType && action.feedbackType && action.resultType) {

      // Iterate over resultType].
      for (const t of [action.requestType, action.feedbackType, action.resultType]) {

        // continue when resolveMessageType is falsy.
        if (!this.resolveMessageType(t)) {
          this.error(`Unknown action type '${t}'`, action.span.start.line, action.span.start.column);
        }
      }

    // Otherwise, continue when action.actionType.
    } else if (action.actionType) {

      // continue when actionType] is falsy.
      if (!ACTION_TYPES[action.actionType]) {
        this.error(
          `Unknown action type '${action.actionType}'`,
          action.span.start.line,
          action.span.start.column,
        );
      }

    // Handle any remaining cases.
    } else {
      this.error(
        `Action '${action.name}' must specify type or request/feedback/result`,
        action.span.start.line,
        action.span.start.column,
      );
    }

    // continue when action.secure) this.checkSecureBlock(action.secure.
    if (action.secure) this.checkSecureBlock(action.secure);
    this.symbols.set(action.name, {
      name: action.name,
      roboType: { kind: "named", name: action.name },
      kind: "action",
      actionType: action.actionType ?? undefined,
    });
}

  private checkSecureBlock(block: import("../foundations.js").SecureBlockDecl): void {    // continue when block.minTrust && !parseTrustLevel(block.minTrust).
    if (block.minTrust && !parseTrustLevel(block.minTrust)) {
      this.error(
        `unknown trust level '${block.minTrust}' in secure block`,
        block.span.start.line,
        block.span.start.column,
      );
    }

    // Process each require.
    for (const cap of block.requires) {

      // continue when isKnownCapability is falsy.
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
    // CheckSafetyRule.
    //
    // Parameters:
    // - `rule` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkSafetyRule(rule);
    if (rule.kind === "MaxSpeedRule") {
      const t = this.checkExpr(rule.value);

      // continue when kind differs from unit).
      if (t.kind !== "number" || !unitsCompatible(t.unit, rule.unit)) {
        this.error(
          `Expected value with unit '${rule.unit}' for ${rule.name}`,
          rule.span.start.line,
          rule.span.start.column,
        );
      }

    // Handle any remaining cases.
    } else {
      const t = this.checkExpr(rule.condition);

      // continue when kind differs from "bool".
      if (t.kind !== "bool") {
        this.error("stop_if condition must be boolean", rule.span.start.line, rule.span.start.column);
      }
    }
}

  private checkSafetyZone(zone: SafetyZoneDecl): void {
    // CheckSafetyZone.
    //
    // Parameters:
    // - `zone` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkSafetyZone(zone);
    const x = this.checkExpr(zone.x);
    const y = this.checkExpr(zone.y);

    // continue when kind differs from kind !== "number".
    if (x.kind !== "number" || y.kind !== "number") {
      this.error("Zone coordinates must be numeric", zone.span.start.line, zone.span.start.column);
    }

    // continue when shape equals radius.
    if (zone.shape === "circle" && zone.radius) {
      const r = this.checkExpr(zone.radius);

      // continue when kind differs from "number".
      if (r.kind !== "number") {
        this.error("Zone radius must be numeric", zone.span.start.line, zone.span.start.column);
      }
    }

    // continue when shape equals height.
    if (zone.shape === "rect" && zone.width && zone.height) {
      const w = this.checkExpr(zone.width);
      const h = this.checkExpr(zone.height);

      // continue when kind differs from kind !== "number".
      if (w.kind !== "number" || h.kind !== "number") {
        this.error("Zone size must be numeric", zone.span.start.line, zone.span.start.column);
      }
    }
}

  private checkAiModel(model: import("../ast/nodes.js").AiModelDecl): void {
    // CheckAiModel.
    //
    // Parameters:
    // - `model` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkAiModel(model);
    if (!AI_MODEL_TYPES[model.modelType]) {
      this.error(
        `Unknown AI model type '${model.modelType}'`,
        model.span.start.line,
        model.span.start.column,
      );
    }

    // continue when this.symbols.has(model.name).
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
    // CheckAgent.
    //
    // Parameters:
    // - `agent` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkAgent(agent);

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
    const prevAgent = this.activeAgent;
    this.activeAgent = agent.name;
    for (const stmt of agent.planBody) {
      this.checkStmt(stmt);
    }
    this.activeAgent = prevAgent;
    this.symbols = saved;
  }

  private checkCapability(agentName: string, cap: CapabilityDecl): void {
    // CheckCapability.
    //
    // Parameters:
    // - `agentName` — input value
    // - `cap` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkCapability(agentName, cap);
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

    // continue when action) is falsy.
    if (!allowed.includes(cap.action) && !isCommCapability(cap.action)) {
      this.error(`Unknown capability '${cap.action}'`, cap.span.start.line, cap.span.start.column);
      return;
    }

    // continue when value.
    if (
      cap.action === "read" ||
      cap.action === "subscribe" ||
      cap.action === "publish" ||
      cap.action === "call" ||
      cap.action === "execute"
    ) {

      // continue when cap.target.
      if (cap.target) {
        const valid =
          this.symbols.has(cap.target) ||
          this.peerRobotNames.has(cap.target) ||
          this.deviceNames.has(cap.target);

        // continue when valid is falsy.
        if (!valid) {
          this.error(
            `Agent '${agentName}' capability ${cap.action}(${cap.target}) references unknown resource`,
            cap.span.start.line,
            cap.span.start.column,
          );
        }

      // Otherwise, continue when action equals action === "publish".
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
    // CheckBehaviorBody.
    //
    // Parameters:
    // - `body` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkBehaviorBody(body);
    const parentScope = new Map(this.symbols);
    this.symbols = new Map(parentScope);
    this.symbols.set("robot", {
      name: "robot",
      roboType: { kind: "named", name: "Robot" },
      kind: "robot",
    });

    // Execute each statement in sequence.
    for (const stmt of body) {
      this.checkStmt(stmt);
    }
    this.symbols = parentScope;
}

  private checkStmt(stmt: Stmt): void {
    // CheckStmt.
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

    // const result = checkStmt(stmt);
    switch (stmt.kind) {
      case "VarDecl": {

        // continue when stmt.typeAnnotation.
        if (stmt.typeAnnotation) {
          this.validateTypeAnnotation(
            stmt.typeAnnotation,
            stmt.span.start.line,
            stmt.span.start.column,
          );
        }

        // continue when value.
        if (
          stmt.typeAnnotation?.kind === "trait_object" &&
          stmt.init?.kind === "IdentExpr"
        ) {
          const traitName = stmt.typeAnnotation.traitName;
          const agent = stmt.init.name;
          const traits = this.agentTraits.get(agent);

          // continue when has is falsy.
          if (!traits?.has(traitName)) {
            this.error(
              `Agent '${agent}' does not implement trait '${traitName}'`,
              stmt.span.start.line,
              stmt.span.start.column,
            );
          }
        }
        const traitAgentOk =
          stmt.typeAnnotation?.kind === "trait_object" &&
          stmt.init?.kind === "IdentExpr" &&
          this.agentTraits.get(stmt.init.name)?.has(stmt.typeAnnotation.traitName);
        const inferred = stmt.init ? this.checkExpr(stmt.init) : null;
        let t: SpandaType;

        // continue when stmt.typeAnnotation && inferred && traitAgentOk.
        if (stmt.typeAnnotation && inferred && traitAgentOk) {
          t = stmt.typeAnnotation;

        // Otherwise, continue when stmt.typeAnnotation && inferred.
        } else if (stmt.typeAnnotation && inferred) {
          this.assertCompatible(
            stmt.typeAnnotation,
            inferred,
            stmt.span.start.line,
            stmt.span.start.column,
          );
          t = stmt.typeAnnotation;

        // Otherwise, continue when stmt.typeAnnotation.
        } else if (stmt.typeAnnotation) {
          t = stmt.typeAnnotation;

        // Otherwise, continue when inferred.
        } else if (inferred) {
          t = inferred;

        // Handle any remaining cases.
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

        // continue when kind differs from "bool".
        if (cond.kind !== "bool") {
          this.error("if condition must be boolean", stmt.span.start.line, stmt.span.start.column);
        }

        // Iterate over thenBranch.
        for (const s of stmt.thenBranch) this.checkStmt(s);

        // continue when stmt.elseBranch) for (const s of stmt.elseBranch) this.checkStmt(s.
        if (stmt.elseBranch) for (const s of stmt.elseBranch) this.checkStmt(s);
        break;
      }
      case "LoopStmt": {

        // Iterate over body.
        for (const s of stmt.body) this.checkStmt(s);
        break;
      }
      case "PublishStmt": {
        const topic = this.symbols.get(stmt.topicName);

        // continue when kind differs from "topic".
        if (!topic || topic.kind !== "topic") {
          this.error(`Unknown topic '${stmt.topicName}'`, stmt.span.start.line, stmt.span.start.column);

        // Handle any remaining cases.
        } else {
          const val = this.checkExpr(stmt.value);
          this.assertCompatible(topic.roboType, val, stmt.span.start.line, stmt.span.start.column);
        }
        break;
      }
      case "ServiceCallStmt": {
        const service = this.symbols.get(stmt.serviceName);

        // continue when kind differs from "service".
        if (!service || service.kind !== "service") {
          this.error(`Unknown service '${stmt.serviceName}'`, stmt.span.start.line, stmt.span.start.column);
        }
        break;
      }
      case "ActionSendStmt": {
        const action = this.symbols.get(stmt.actionName);

        // continue when kind differs from "action".
        if (!action || action.kind !== "action") {
          this.error(`Unknown action '${stmt.actionName}'`, stmt.span.start.line, stmt.span.start.column);

        // Handle any remaining cases.
        } else {
          const goal = this.checkExpr(stmt.goal);

          // continue when kind differs from kind !== "trajectory".
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

        // continue when stateName) is falsy.
        if (!this.stateMachineStates.has(stmt.stateName)) {
          this.error(
            `Unknown state '${stmt.stateName}' for enter statement`,
            stmt.span.start.line,
            stmt.span.start.column,
          );
        }
        break;
      case "EnterModeStmt":
      case "StopAllActuatorsStmt":
      case "RunPipelineStmt":
      case "UseFallbackStmt":
        break;
      case "RememberStmt":
        this.checkExpr(stmt.value);
        break;
      case "SubscribeStmt": {
        const [topicName] = stmt.target.split(".");

        if (
          stmt.target.includes(".") &&
          !this.symbols.has(topicName) &&
          !this.peerRobotNames.has(topicName)
        ) {
          this.error(
            `Unknown subscribe target '${stmt.target}'`,
            stmt.span.start.line,
            stmt.span.start.column,
          );
        }
        if (stmt.filter) {
          try {
            compileRegex(stmt.filter.pattern);
          } catch (err) {
            if (err instanceof RegexError) {
              this.error(err.message, err.line, err.column);
            } else {
              throw err;
            }
          }
        }
        this.subscribedTopics.add(stmt.target);
        break;
      }
      case "ExecuteStmt": {
        const action = this.symbols.get(stmt.actionName);

        // continue when kind differs from "action".
        if (!action || action.kind !== "action") {
          this.error(
            `Unknown action '${stmt.actionName}'`,
            stmt.span.start.line,
            stmt.span.start.column,
          );

        // Handle any remaining cases.
        } else {
          this.checkExpr(stmt.goal);
        }
        break;
      }
      case "DiscoverStmt":
        break;
      case "ReceiveStmt": {
        const root = stmt.topicName.split(".")[0] ?? stmt.topicName;
        const topic = this.symbols.get(stmt.topicName);

        // continue when kind differs from has.
        if ((!topic || topic.kind !== "topic") && !this.peerRobotNames.has(root)) {
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

        // continue when stmt.value) this.checkExpr(stmt.value.
        if (stmt.value) this.checkExpr(stmt.value);
        break;
      case "SpawnStmt": {

        // continue when kind equals name).
        if (stmt.callee.kind === "IdentExpr" && !this.moduleFunctions.has(stmt.callee.name)) {
          this.error(
            `Unknown spawn target '${stmt.callee.name}'`,
            stmt.span.start.line,
            stmt.span.start.column,
          );
        }

        // Apply each command-line argument.
        for (const arg of stmt.args) {
          this.checkExpr(arg);
        }
        break;
      }
      case "SelectStmt":

        // Process each arm.
        for (const arm of stmt.arms) {
          this.checkExpr(arm.channel);

          // Iterate over body.
          for (const s of arm.body) {
            this.checkStmt(s);
          }
        }
        break;
      case "ParallelStmt":

        // Iterate over body.
        for (const s of stmt.body) {
          this.checkStmt(s);
        }
        this.symbols.set("_parallel", {
          name: "_parallel",
          roboType: { kind: "named", name: "ParallelResults" },
          kind: "variable",
        });
        break;
    }
}

  private checkExpr(expr: Expr): SpandaType {
    // CheckExpr.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // SpandaType.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkExpr(expr);
    switch (expr.kind) {
      case "LiteralExpr": {
        if (typeof expr.value === "object" && expr.value !== null && "source" in expr.value) {
          try {
            compileRegex(expr.value);
          } catch (err) {
            if (err instanceof RegexError) {
              this.error(err.message, err.line, err.column);
            } else {
              throw err;
            }
          }
          return { kind: "regex" };
        }

        // continue when value equals "boolean".
        if (typeof expr.value === "boolean") return { kind: "bool" };

        // continue when value equals "number".
        if (typeof expr.value === "number") return { kind: "number", unit: "none" };

        // continue when value equals "string".
        if (typeof expr.value === "string") return { kind: "string" };
        return { kind: "void" };
      }
      case "UnitLiteralExpr":
        return { kind: "number", unit: expr.unit };
      case "IdentExpr": {
        const enumName = this.variantOwner.get(expr.name);

        // continue when enumName.
        if (enumName) {
          return { kind: "enum_variant", enumName, variant: expr.name };
        }
        const sym = this.symbols.get(expr.name);

        // continue when sym is falsy.
        if (!sym) {
          this.error(`Undefined identifier '${expr.name}'`, expr.span.start.line, expr.span.start.column);
          return { kind: "void" };
        }
        return sym.roboType;
      }
      case "BinaryExpr": {
        const left = this.checkExpr(expr.left);
        const right = this.checkExpr(expr.right);

        // continue when value.
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

        // continue when result is falsy.
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

        // continue when op equals kind !== "bool".
        if (expr.op === "not" && operand.kind !== "bool") {
          this.error("Operand of 'not' must be boolean", expr.span.start.line, expr.span.start.column);
        }

        // continue when op equals kind !== "number".
        if (expr.op === "-" && operand.kind !== "number") {
          this.error("Operand of '-' must be numeric", expr.span.start.line, expr.span.start.column);
        }
        return expr.op === "not" ? { kind: "bool" } : operand;
      }
      case "MemberExpr":
        return this.checkMember(expr);
      case "CallExpr":
        return this.checkCall(expr);
      case "AwaitExpr": {
        const inner = this.checkExpr(expr.operand);

        // continue when kind equals name === "Future".
        if (inner.kind === "generic" && inner.name === "Future") {
          const t = inner.typeArgs[0];

          // continue when t.
          if (t) return t;
        }
        this.error("await requires a Future value", expr.span.start.line, expr.span.start.column);
        return { kind: "void" };
      }
      case "SpawnExpr": {

        // continue when kind equals name).
        if (expr.callee.kind === "IdentExpr" && !this.moduleFunctions.has(expr.callee.name)) {
          this.error(
            `Unknown spawn target '${expr.callee.name}'`,
            expr.span.start.line,
            expr.span.start.column,
          );
        }

        // Apply each command-line argument.
        for (const arg of expr.args) {
          this.checkExpr(arg);
        }

        // continue when kind equals "IdentExpr".
        if (expr.callee.kind === "IdentExpr") {
          const moduleFn = this.moduleFunctions.get(expr.callee.name);

          // continue when moduleFn.
          if (moduleFn) {
            const ret = this.resolveTypeAnn(moduleFn.returnType);
            return TypeChecker.taskHandleType(ret);
          }
        }
        return TypeChecker.taskHandleType({ kind: "void" });
      }
      case "MatchExpr":
        return this.checkMatch(expr);
      case "StructLiteralExpr":
        return this.checkStructLiteral(expr);
      case "ServiceCallExpr": {
        const service = this.symbols.get(expr.serviceName);

        // continue when kind differs from "service".
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

        // continue when kind differs from "action".
        if (!action || action.kind !== "action") {
          this.error(
            `Unknown action '${expr.actionName}'`,
            expr.span.start.line,
            expr.span.start.column,
          );

        // Handle any remaining cases.
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
    // CheckStructLiteral.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // SpandaType.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkStructLiteral(expr);
    const [baseName, typeArgNames] = splitInstantiatedTypeName(expr.typeName);
    const def = this.structDefs.get(baseName);

    // continue when def is falsy.
    if (!def) {
      this.error(`Unknown struct type '${expr.typeName}'`, expr.span.start.line, expr.span.start.column);
      return { kind: "void" };
    }
    const typeParams = this.structTypeParams.get(baseName) ?? [];

    // continue when length differs from length.
    if (typeParams.length !== typeArgNames.length) {
      this.error(
        `Struct '${baseName}' expects ${typeParams.length} type argument(s), got ${typeArgNames.length}`,
        expr.span.start.line,
        expr.span.start.column,
      );
      return { kind: "void" };
    }
    const substitutions = new Map<string, string>();
    typeParams.forEach((param, index) => {
      const arg = typeArgNames[index];

      // continue when arg) substitutions.set(param, arg.
      if (arg) substitutions.set(param, arg);
    });
    const provided = new Set<string>();

    // Process each field.
    for (const field of expr.fields) {

      // continue when provided.has(field.name).
      if (provided.has(field.name)) {
        this.error(`Duplicate struct field '${field.name}'`, field.span.start.line, field.span.start.column);
      }
      provided.add(field.name);
      const fieldDef = def.find((f) => f.name === field.name);

      // continue when fieldDef is falsy.
      if (!fieldDef) {
        this.error(
          `Struct '${baseName}' has no field '${field.name}'`,
          field.span.start.line,
          field.span.start.column,
        );
        continue;
      }
      const expected = this.typeNameToSpanda(instantiateTypeName(fieldDef.typeName, substitutions));
      const actual = this.checkExpr(field.value);
      this.assertCompatible(expected, actual, field.span.start.line, field.span.start.column);
    }

    // Iterate over the collection.
    for (const { name } of def) {

      // continue when has is falsy.
      if (!provided.has(name)) {
        this.error(
          `Missing struct field '${name}' in '${expr.typeName}' literal`,
          expr.span.start.line,
          expr.span.start.column,
        );
      }
    }

    // continue when length equals 0.
    if (typeArgNames.length === 0) {
      return { kind: "named", name: baseName };
    }
    return {
      kind: "generic",
      name: baseName,
      typeArgs: typeArgNames.map((arg) => this.typeNameToSpanda(arg)),
    };
}

  private checkMatch(expr: import("../ast/nodes.js").MatchExpr): SpandaType {
    // CheckMatch.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // SpandaType.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkMatch(expr);
    const scrutineeType = this.checkExpr(expr.scrutinee);

    // continue when length equals 0.
    if (expr.arms.length === 0) {
      this.error("match expression requires at least one arm", expr.span.start.line, expr.span.start.column);
    }
    const scrutineeEnum = scrutineeType.kind === "named" ? scrutineeType.name : null;

    // Process each arm.
    for (const arm of expr.arms) {
      const bindings = arm.bindings ?? [];

      // continue when bindings.length > 0.
      if (bindings.length > 0) {

        // continue when scrutineeEnum.
        if (scrutineeEnum) {
          const fieldTypes = this.enumPayloadFields.get(this.enumPayloadKey(scrutineeEnum, arm.variant));

          // continue when fieldTypes.
          if (fieldTypes) {

            // continue when length differs from length.
            if (bindings.length !== fieldTypes.length) {
              this.error(
                `Match arm '${arm.variant}' expects ${fieldTypes.length} binding(s), got ${bindings.length}`,
                arm.span.start.line,
                arm.span.start.column,
              );
            }

            // Loop with index variable i.
            for (let i = 0; i < bindings.length; i++) {
              const typeName = fieldTypes[i];
              const binding = bindings[i];

              // continue when typeName && binding.
              if (typeName && binding) {
                this.symbols.set(binding, {
                  name: binding,
                  roboType: this.typeNameToSpanda(typeName),
                  kind: "variable",
                });
              }
            }

          // Handle any remaining cases.
          } else {
            this.error(
              `Variant '${arm.variant}' has no payload bindings`,
              arm.span.start.line,
              arm.span.start.column,
            );
          }
        }
      }

      // Execute each statement in sequence.
      for (const stmt of arm.body) {
        this.checkStmt(stmt);
      }

      // Process each binding.
      for (const binding of bindings) {
        this.symbols.delete(binding);
      }
    }
    this.checkMatchExhaustiveness(expr.arms, expr.span);
    return { kind: "void" };
}

  private checkMatchExhaustiveness(arms: MatchArm[], span: import("../ast/nodes.js").Span): void {
    // CheckMatchExhaustiveness.
    //
    // Parameters:
    // - `arms` — input value
    // - `span` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkMatchExhaustiveness(arms, span);
    const armNames = new Set(arms.map((a) => a.variant));

    // continue when size equals 0.
    if (armNames.size === 0) return;

    // continue when armNames.has("Ok") || armNames.has("Err").
    if (armNames.has("Ok") || armNames.has("Err")) {

      // Iterate over ["Ok", "Err"].
      for (const required of ["Ok", "Err"]) {

        // continue when has is falsy.
        if (!armNames.has(required)) {
          this.error(
            `Non-exhaustive match on Result: missing '${required}' arm`,
            span.start.line,
            span.start.column,
          );
        }
      }
      return;
    }

    // continue when armNames.has("Some") || armNames.has("None").
    if (armNames.has("Some") || armNames.has("None")) {

      // Iterate over ["Some", "None"].
      for (const required of ["Some", "None"]) {

        // continue when has is falsy.
        if (!armNames.has(required)) {
          this.error(
            `Non-exhaustive match on Option: missing '${required}' arm`,
            span.start.line,
            span.start.column,
          );
        }
      }
      return;
    }

    // Process each value.
    for (const variants of this.enumVariants.values()) {
      const variantSet = new Set(variants);

      // continue when [...armNames].every((name) => variantSet.has(name)).
      if ([...armNames].every((name) => variantSet.has(name))) {

        // continue when armNames.size < variantSet.size.
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
    // CheckMember.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // SpandaType.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkMember(expr);
    if (expr.object.kind === "IdentExpr") {
      const sym = this.symbols.get(expr.object.name);

      // continue when kind equals property === "nearest distance".
      if (sym?.kind === "sensor" && sym.sensorType === "Lidar" && expr.property === "nearest_distance") {
        return { kind: "number", unit: "m" };
      }
    }
    const objType = this.checkExpr(expr.object);

    // continue when kind equals "scan".
    if (objType.kind === "scan") {
      const prop = SCAN_PROPERTIES[expr.property];

      // continue when prop is falsy.
      if (!prop) {
        this.error(`Unknown scan property '${expr.property}'`, expr.span.start.line, expr.span.start.column);
        return { kind: "void" };
      }
      return prop;
    }

    // continue when kind equals "pose".
    if (objType.kind === "pose") {
      const prop = POSE_PROPERTIES[expr.property];

      // continue when prop is falsy.
      if (!prop) {
        this.error(`Unknown pose property '${expr.property}'`, expr.span.start.line, expr.span.start.column);
        return { kind: "void" };
      }
      return prop;
    }

    // continue when kind equals "velocity".
    if (objType.kind === "velocity") {
      const prop = VELOCITY_PROPERTIES[expr.property];

      // continue when prop is falsy.
      if (!prop) {
        this.error(`Unknown velocity property '${expr.property}'`, expr.span.start.line, expr.span.start.column);
        return { kind: "void" };
      }
      return prop;
    }

    // continue when kind equals "generic".
    if (objType.kind === "generic") {
      const structFields = this.structDefs.get(objType.name);
      const typeParams = this.structTypeParams.get(objType.name) ?? [];
      const structField = structFields?.find((f) => f.name === expr.property);

      // continue when structField.
      if (structField) {
        const substitutions = new Map<string, string>();
        typeParams.forEach((param, index) => {
          const arg = objType.typeArgs[index];

          // continue when kind equals name.
          if (arg?.kind === "named") substitutions.set(param, arg.name);

          // Otherwise, continue when kind equals set.
          else if (arg?.kind === "int") substitutions.set(param, "Int");

          // Otherwise, continue when kind equals set.
          else if (arg?.kind === "float") substitutions.set(param, "Float");

          // Otherwise, continue when kind equals set.
          else if (arg?.kind === "bool") substitutions.set(param, "Bool");

          // Otherwise, continue when kind equals set.
          else if (arg?.kind === "string") substitutions.set(param, "String");
        });
        return this.typeNameToSpanda(instantiateTypeName(structField.typeName, substitutions));
      }
    }

    // continue when kind equals "named".
    if (objType.kind === "named") {
      const enumVariants = this.enumVariants.get(objType.name);

      // check membership before continuing.
      if (enumVariants?.includes(expr.property)) {
        return { kind: "enum_variant", enumName: objType.name, variant: expr.property };
      }
      const structFields = this.structDefs.get(objType.name);
      const structField = structFields?.find((f) => f.name === expr.property);

      // continue when structField.
      if (structField) {
        return this.typeNameToSpanda(structField.typeName);
      }
      const objProps = OBJECT_PROPERTIES[objType.name];

      // continue when objProps?.[expr.property].
      if (objProps?.[expr.property]) return objProps[expr.property];
      const methods = BUILTIN_METHODS[objType.name];

      // continue when methods?.[expr.property].
      if (methods?.[expr.property]) return methods[expr.property].returns;
    }

    // continue when kind equals "trait object".
    if (objType.kind === "trait_object") {
      const traitMethods = this.traitDefs.get(objType.traitName);
      const method = traitMethods?.get(expr.property);

      // continue when method.
      if (method) {
        return this.typeNameToSpanda(method.returnType);
      }
      this.error(
        `Unknown trait method '${expr.property}' on '${objType.traitName}'`,
        expr.span.start.line,
        expr.span.start.column,
      );
      return { kind: "void" };
    }

    // continue when kind equals "IdentExpr".
    if (expr.object.kind === "IdentExpr") {
      const sym = this.symbols.get(expr.object.name);

      // continue when kind equals "trait object".
      if (sym?.roboType.kind === "trait_object") {
        const traitMethods = this.traitDefs.get(sym.roboType.traitName);
        const method = traitMethods?.get(expr.property);

        // continue when method.
        if (method) {
          return this.typeNameToSpanda(method.returnType);
        }
        this.error(
          `Unknown trait method '${expr.property}' on '${sym.roboType.traitName}'`,
          expr.span.start.line,
          expr.span.start.column,
        );
        return { kind: "void" };
      }
    }
    this.error(`Unknown member '${expr.property}'`, expr.span.start.line, expr.span.start.column);
    return { kind: "void" };
}

  private checkResultOptionCtor(
    name: string,
    args: Expr[],
    span: import("../ast/nodes.js").Span,
  ): SpandaType {
    // CheckResultOptionCtor.
    //
    // Parameters:
    // - `name` — input value
    // - `args` — input value
    // - `span` — input value
    //
    // Returns:
    // SpandaType.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkResultOptionCtor(name, args, span);
    if (name === "Ok" || name === "Some") {
      const arg = args[0];

      // continue when arg is falsy.
      if (!arg) {
        this.error(`'${name}' requires a value argument`, span.start.line, span.start.column);
        return { kind: "void" };
      }
      const inner = this.checkExpr(arg);

      // continue when name equals "Ok".
      if (name === "Ok") {
        return {
          kind: "generic",
          name: "Result",
          typeArgs: [inner, { kind: "named", name: "Error" }],
        };
      }
      return { kind: "generic", name: "Option", typeArgs: [inner] };
    }

    // continue when name equals "Err".
    if (name === "Err") {
      const inner = args[0] ? this.checkExpr(args[0]) : { kind: "named" as const, name: "Error" };
      return {
        kind: "generic",
        name: "Result",
        typeArgs: [{ kind: "void" }, inner],
      };
    }
    return { kind: "generic", name: "Option", typeArgs: [{ kind: "void" }] };
}

  private checkCall(expr: import("../ast/nodes.js").CallExpr): SpandaType {
    // CheckCall.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // SpandaType.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = checkCall(expr);
    if (expr.callee.kind === "IdentExpr") {
      const name = expr.callee.name;
      const moduleFn = this.moduleFunctions.get(name);

      // continue when moduleFn.
      if (moduleFn) {
        const saved = new Map(this.typeParamScope);

        // Loop with index variable i.
        for (let i = 0; i < moduleFn.typeParams.length; i++) {
          const tp = moduleFn.typeParams[i];
          const arg = expr.args[i];

          // continue when tp && arg.
          if (tp && arg) {
            this.typeParamScope.set(tp, this.checkExpr(arg));
          }
        }

        // Loop with index variable i.
        for (let i = 0; i < expr.args.length; i++) {
          const param = moduleFn.params[i];

          // continue when param.
          if (param) {
            const expected = this.resolveTypeAnn(param.typeAnn);
            const actual = this.checkExpr(expr.args[i]!);
            this.assertCompatible(expected, actual, expr.span.start.line, expr.span.start.column);
          }
        }
        const ret = this.resolveTypeAnn(moduleFn.returnType);
        this.typeParamScope = saved;
        return moduleFn.isAsync ? TypeChecker.futureType(ret) : ret;
      }
      const externFn = this.externFunctions.get(name);

      // continue when externFn.
      if (externFn) {

        // Loop with index variable i.
        for (let i = 0; i < expr.args.length; i++) {
          const param = externFn.params[i];

          // continue when param.
          if (param) {
            const expected = this.resolveTypeAnn(param.typeAnn);
            const actual = this.checkExpr(expr.args[i]!);
            this.assertCompatible(expected, actual, expr.span.start.line, expr.span.start.column);
          }
        }
        return this.resolveTypeAnn(externFn.returnType);
      }

      // continue when name equals "assert".
      if (name === "assert") {
        const arg = expr.args[0];

        // continue when arg.
        if (arg) {
          const t = this.checkExpr(arg);

          // continue when kind differs from "bool".
          if (t.kind !== "bool") {
            this.error("assert requires a boolean condition", expr.span.start.line, expr.span.start.column);
          }
        }
        return { kind: "void" };
      }

      // continue when name equals "channel".
      if (name === "channel") {
        return { kind: "named", name: "Channel" };
      }

      // continue when name equals "send".
      if (name === "send") {

        // continue when expr.args.length < 2.
        if (expr.args.length < 2) {
          this.error("send requires (channel, value)", expr.span.start.line, expr.span.start.column);
          return { kind: "void" };
        }
        const channelTy = this.checkExpr(expr.args[0]!);

        // continue when kind differs from name !== "Channel".
        if (channelTy.kind !== "named" || channelTy.name !== "Channel") {
          this.error("send first argument must be Channel", expr.span.start.line, expr.span.start.column);
        }
        const payloadTy = this.checkExpr(expr.args[1]!);

        // continue when kind equals "IdentExpr".
        if (expr.args[0]?.kind === "IdentExpr") {
          const channelName = expr.args[0].name;
          const existing = this.channelPayloadTypes.get(channelName);

          // continue when existing.
          if (existing) {
            this.assertCompatible(existing, payloadTy, expr.span.start.line, expr.span.start.column);

          // Handle any remaining cases.
          } else {
            this.channelPayloadTypes.set(channelName, payloadTy);
          }
        }
        return { kind: "void" };
      }

      // continue when name equals "recv".
      if (name === "recv") {

        // continue when expr.args.length < 1.
        if (expr.args.length < 1) {
          this.error("recv requires (channel)", expr.span.start.line, expr.span.start.column);
          return { kind: "void" };
        }

        // continue when kind equals "IdentExpr".
        if (expr.args[0]?.kind === "IdentExpr") {
          const existing = this.channelPayloadTypes.get(expr.args[0].name);

          // continue when existing.
          if (existing) return existing;
        }
        return { kind: "void" };
      }

      // continue when name equals "join".
      if (name === "join") {

        // continue when expr.args.length < 1.
        if (expr.args.length < 1) {
          this.error("join requires (handle)", expr.span.start.line, expr.span.start.column);
          return { kind: "void" };
        }
        const joined = this.checkExpr(expr.args[0]!);

        // continue when kind equals name === "TaskHandle").
        if (joined.kind === "generic" && (joined.name === "Future" || joined.name === "TaskHandle")) {
          return joined.typeArgs[0] ?? { kind: "void" };
        }
        this.error("join requires a Future or TaskHandle value", expr.span.start.line, expr.span.start.column);
        return { kind: "void" };
      }

      // continue when name equals "send agent".
      if (name === "send_agent") {

        // continue when expr.args.length < 2.
        if (expr.args.length < 2) {
          this.error("send_agent requires (to, value)", expr.span.start.line, expr.span.start.column);
        }

        // continue when activeAgent is falsy.
        if (!this.activeAgent) {
          this.error("send_agent requires active agent context", expr.span.start.line, expr.span.start.column);
        }

        // continue when expr.args[1]) this.checkExpr(expr.args[1].
        if (expr.args[1]) this.checkExpr(expr.args[1]);
        return { kind: "void" };
      }

      // continue when name equals "recv agent".
      if (name === "recv_agent") {

        // continue when activeAgent is falsy.
        if (!this.activeAgent) {
          this.error("recv_agent requires active agent context", expr.span.start.line, expr.span.start.column);
        }
        return { kind: "void" };
      }

      // continue when name equals "peer send".
      if (name === "peer_send") {

        // continue when expr.args.length < 3.
        if (expr.args.length < 3) {
          this.error("peer_send requires (peer, topic, value)", expr.span.start.line, expr.span.start.column);
        }

        // continue when expr.args[2]) this.checkExpr(expr.args[2]!.
        if (expr.args[2]) this.checkExpr(expr.args[2]!);
        return { kind: "void" };
      }

      // continue when name equals "Ok" || name === "Err" || name === "Some" || name === "None".
      if (name === "Ok" || name === "Err" || name === "Some" || name === "None") {
        return this.checkResultOptionCtor(name, expr.args, expr.span);
      }
      const enumName = this.variantOwner.get(name);

      // continue when enumName.
      if (enumName) {
        const fieldTypes = this.enumPayloadFields.get(this.enumPayloadKey(enumName, name));

        // continue when fieldTypes.
        if (fieldTypes) {

          // continue when length differs from length.
          if (expr.args.length !== fieldTypes.length) {
            this.error(
              `Variant '${name}' expects ${fieldTypes.length} payload argument(s), got ${expr.args.length}`,
              expr.span.start.line,
              expr.span.start.column,
            );
          }

          // Loop with index variable i.
          for (let i = 0; i < expr.args.length; i++) {
            const typeName = fieldTypes[i];
            const arg = expr.args[i];

            // continue when typeName && arg.
            if (typeName && arg) {
              const expected = this.typeNameToSpanda(typeName);
              const actual = this.checkExpr(arg);
              this.assertCompatible(expected, actual, expr.span.start.line, expr.span.start.column);
            }
          }
          return { kind: "named", name: enumName };
        }
        const variants = this.enumVariants.get(enumName);

        // check membership before continuing.
        if (variants?.includes(name)) {

          // continue when expr.args.length > 0.
          if (expr.args.length > 0) {
            this.error(`Unit variant '${name}' takes no arguments`, expr.span.start.line, expr.span.start.column);
          }
          return { kind: "named", name: enumName };
        }
      }
      const fn = BUILTIN_FUNCTIONS[name];

      // continue when fn is falsy.
      if (!fn) {
        this.error(`Unknown function '${name}'`, expr.span.start.line, expr.span.start.column);
        return { kind: "void" };
      }

      // Apply each command-line argument.
      for (const arg of expr.namedArgs) {
        const expected = fn.namedParams[arg.name];

        // continue when expected is falsy.
        if (!expected) {
          this.error(`Unknown named argument '${arg.name}'`, arg.span.start.line, arg.span.start.column);
          continue;
        }
        const actual = this.checkExpr(arg.value);
        this.assertCompatible(expected, actual, arg.span.start.line, arg.span.start.column);
      }
      return fn.returns;
    }

    if (expr.callee.kind === "MemberExpr") {
      const member = expr.callee;
      const objType = this.checkExpr(member.object);
      if (objType.kind === "string" || objType.kind === "regex") {
        const typeName = objType.kind === "string" ? "String" : "Regex";
        const method = BUILTIN_METHODS[typeName]?.[member.property];
        if (method) {
          for (let i = 0; i < expr.args.length; i++) {
            const expected = method.params[i];
            if (expected) {
              const actual = this.checkExpr(expr.args[i]!);
              this.assertCompatible(expected, actual, expr.span.start.line, expr.span.start.column);
            }
          }
          return method.returns;
        }
      }
    }

    if (expr.callee.kind !== "MemberExpr") {
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

    // continue when sym is falsy.
    if (!sym) {
      this.error(`Undefined identifier '${targetName}'`, expr.span.start.line, expr.span.start.column);
      return { kind: "void" };
    }

    // continue when kind equals "robot".
    if (sym.kind === "robot") {
      const method = ROBOT_METHODS[member.property];

      // continue when method is falsy.
      if (!method) {
        this.error(`Unknown robot method '${member.property}`, expr.span.start.line, expr.span.start.column);
        return { kind: "void" };
      }

      // Loop with index variable i.
      for (let i = 0; i < expr.args.length; i++) {
        const expected = method.params[i];

        // continue when expected.
        if (expected) {
          const actual = this.checkExpr(expr.args[i]);
          this.assertCompatible(expected, actual, expr.span.start.line, expr.span.start.column);
        }
      }
      return method.returns;
    }

    // continue when kind equals "agent".
    if (sym.kind === "agent") {
      const traitMethods = this.agentTraitMethods.get(targetName);

      // continue when traitMethods?.has(member.property).
      if (traitMethods?.has(member.property)) {
        return traitMethods.get(member.property)!;
      }
      const agentMethod = BUILTIN_METHODS.Agent?.[member.property];

      // continue when agentMethod is falsy.
      if (!agentMethod) {
        this.error(`Unknown agent method '${member.property}`, expr.span.start.line, expr.span.start.column);
        return { kind: "void" };
      }
      return agentMethod.returns;
    }

    // continue when kind equals "trait object".
    if (sym.roboType.kind === "trait_object") {
      const traitMethods = this.traitDefs.get(sym.roboType.traitName);
      const method = traitMethods?.get(member.property);

      // continue when method.
      if (method) {
        return this.typeNameToSpanda(method.returnType);
      }
      this.error(
        `Unknown trait method '${member.property}' on '${sym.roboType.traitName}'`,
        expr.span.start.line,
        expr.span.start.column,
      );
      return { kind: "void" };
    }
    let typeName = "";

    // continue when kind equals sensorType.
    if (sym.kind === "sensor" && sym.sensorType) typeName = sym.sensorType;

    // Otherwise, continue when kind equals actuatorType.
    else if (sym.kind === "actuator" && sym.actuatorType) typeName = sym.actuatorType;

    // Otherwise, continue when kind equals "safety".
    else if (sym.kind === "safety") typeName = "Safety";

    // Otherwise, continue when kind equals kind === "named".
    else if (sym.kind === "ai_model" && sym.roboType.kind === "named") typeName = sym.roboType.name;

    // Otherwise, continue when kind equals "named".
    else if (sym.roboType.kind === "named") typeName = sym.roboType.name;

    // Otherwise, continue when kind equals "scan".
    else if (sym.roboType.kind === "scan") typeName = "Scan";
    const methods = BUILTIN_METHODS[typeName];
    const method = methods?.[member.property];

    // continue when method is falsy.
    if (!method) {
      this.error(
        `Unknown method '${member.property}' on ${typeName}`,
        expr.span.start.line,
        expr.span.start.column,
      );
      return { kind: "void" };
    }

    // continue when typeName equals property === "drive".
    if (typeName === "LLM" && member.property === "drive") {
      this.error(
        "AI models cannot control actuators directly — use reason(), safety.validate(), then actuator.execute()",
        expr.span.start.line,
        expr.span.start.column,
      );
      return { kind: "void" };
    }

    // continue when method.namedParams.
    if (method.namedParams) {

      // Apply each command-line argument.
      for (const arg of expr.namedArgs) {
        const expected = method.namedParams[arg.name];

        // continue when expected is falsy.
        if (!expected) {
          this.error(`Unknown named argument '${arg.name}'`, arg.span.start.line, arg.span.start.column);
          continue;
        }

        // continue when typeName equals kind === "IdentExpr".
        if (typeName === "Twin" && arg.name === "field" && arg.value.kind === "IdentExpr") {
          const allowedMirrorFields = [
            "pose",
            "velocity",
            "battery",
            "status",
            "scan",
            ...(this.currentRobot?.sensors.map((s) => s.name) ?? []),
            ...(this.currentRobot?.actuators.map((a) => a.name) ?? []),
          ];

          // continue when name) is falsy.
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

    // Apply each command-line argument.
    for (const arg of expr.args) {
      const actual = this.checkExpr(arg);

      // continue when typeName equals property === "validate" && !isActionProposalType.
      if (typeName === "Safety" && member.property === "validate" && !isActionProposalType(actual)) {
        this.error(
          "safety.validate() expects ActionProposal",
          expr.span.start.line,
          expr.span.start.column,
        );
      }

      // continue when typeName equals property === "execute".
      if (typeName === "DifferentialDrive" && member.property === "execute") {

        // continue when isActionProposalType(actual).
        if (isActionProposalType(actual)) {
          this.error(
            "ActionProposal cannot be passed to actuator.execute() — call safety.validate() first",
            expr.span.start.line,
            expr.span.start.column,
          );

        // Otherwise, continue when isSafeActionType is falsy.
        } else if (!isSafeActionType(actual)) {
          this.error(
            "actuator.execute() requires SafeAction from safety.validate()",
            expr.span.start.line,
            expr.span.start.column,
          );
        }
      }

      // continue when property equals "detect" && typeName === "VisionModel".
      if (member.property === "detect" && typeName === "VisionModel") {
        this.assertNamedType(actual, "CameraFrame", expr.span.start.line, expr.span.start.column);
      }
    }

    // continue when property equals "read" && typeName === "Lidar".
    if (member.property === "read" && typeName === "Lidar") {
      return { kind: "scan" };
    }
    return method.returns;
}

  private typesCompatible(expected: SpandaType, actual: SpandaType): boolean {
    // TypesCompatible.
    //
    // Parameters:
    // - `expected` — input value
    // - `actual` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = typesCompatible(expected, actual);
    if (expected.kind === actual.kind) {

      // continue when kind equals kind === "number".
      if (expected.kind === "number" && actual.kind === "number") {
        return unitsCompatible(expected.unit, actual.unit);
      }

      // continue when kind equals kind === "named".
      if (expected.kind === "named" && actual.kind === "named") {
        return expected.name === actual.name || actual.name.includes(expected.name);
      }

      // continue when kind equals kind === "enum variant".
      if (expected.kind === "enum_variant" && actual.kind === "enum_variant") {
        return expected.enumName === actual.enumName && expected.variant === actual.variant;
      }

      // continue when kind equals kind === "generic".
      if (expected.kind === "generic" && actual.kind === "generic") {
        return (
          expected.name === actual.name &&
          expected.typeArgs.length === actual.typeArgs.length &&
          expected.typeArgs.every((e, i) => this.typesCompatible(e, actual.typeArgs[i]!))
        );
      }

      // continue when kind equals kind === "trait object".
      if (expected.kind === "trait_object" && actual.kind === "trait_object") {
        return expected.traitName === actual.traitName;
      }
      return true;
    }

    // continue when kind equals kind === "enum variant".
    if (expected.kind === "named" && actual.kind === "enum_variant") {
      return expected.name === actual.enumName;
    }

    // continue when kind equals kind === "named".
    if (expected.kind === "enum_variant" && actual.kind === "named") {
      return expected.enumName === actual.name;
    }

    // continue when kind equals includes.
    if (expected.kind === "named" && actual.kind === "scan" && expected.name.includes("Lidar")) {
      return true;
    }

    // continue when kind equals kind === "named".
    if (expected.kind === "scan" && actual.kind === "named") {
      return ["Detection", "CameraFrame", "Completion"].includes(actual.name);
    }

    // continue when value.
    if (
      expected.kind === "int" &&
      actual.kind === "number" &&
      actual.unit === "none"
    ) {
      return true;
    }

    // continue when kind equals kind === "number".
    if (expected.kind === "float" && actual.kind === "number") {
      return true;
    }

    // continue when value.
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

    // continue when kind equals kind === "number".
    if (expected.kind === "named" && actual.kind === "number") {
      return unitMatchesNamedType(expected.name, actual.unit);
    }

    // continue when kind equals kind === "string".
    if (expected.kind === "named" && actual.kind === "string") {
      return expected.name === "Goal";
    }

    // continue when kind equals kind === "named".
    if (expected.kind === "string" && actual.kind === "named") {
      return actual.name === "Goal";
    }
    return false;
}

  private assertNamedType(actual: SpandaType, typeName: string, line: number, column: number): void {
    // AssertNamedType.
    //
    // Parameters:
    // - `actual` — input value
    // - `typeName` — input value
    // - `line` — input value
    // - `column` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = assertNamedType(actual, typeName, line, column);
    if (actual.kind === "named" && actual.name === typeName) return;
    this.error(`Expected ${typeName}, got ${actual.kind}`, line, column);
}

  private assertCompatible(expected: SpandaType, actual: SpandaType, line: number, column: number): void {    // continue when kind equals kind === "void".
    if (expected.kind === "void" && actual.kind === "void") return;

    // continue when typesCompatible is falsy.
    if (!this.typesCompatible(expected, actual)) {

      // continue when kind equals kind === "number".
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

  private error(message: string, line: number, column: number): void {    // Call push on this instance.
    this.errors.push({ message, line, column });
}
}

function splitInstantiatedTypeName(typeName: string): [string, string[]] {
  // SplitInstantiatedTypeName.
  //
  // Parameters:
  // - `typeName` — input value
  //
  // Returns:
  // `[string, string[]]`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = splitInstantiatedTypeName(typeName);
  const lt = typeName.indexOf("<");

  // continue when lt >= 0 && typeName.endsWith(">").
  if (lt >= 0 && typeName.endsWith(">")) {
    const base = typeName.slice(0, lt).trim();
    const args = typeName
      .slice(lt + 1, -1)
      .split(",")
      .map((part) => part.trim())
      .filter(Boolean);
    return [base, args];
  }
  return [typeName, []];
}

function instantiateTypeName(typeName: string, substitutions: Map<string, string>): string {
  // InstantiateTypeName.
  //
  // Parameters:
  // - `typeName` — input value
  // - `substitutions` — input value
  //
  // Returns:
  // Text result.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = instantiateTypeName(typeName, substitutions);
  return substitutions.get(typeName) ?? typeName;
}
