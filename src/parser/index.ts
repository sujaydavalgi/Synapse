/**
 * index module (parser/index.ts).
 * @module
 */

import type { Token } from "../lexer/index.js";
import { unitFromLexeme } from "../lexer/index.js";
import { regexFromLexeme } from "../regex.js";
import { resolveGenericType, resolveTypeName } from "../type-system.js";
import type { SpandaType } from "../ast/nodes.js";
import type {
  ActuatorDecl,
  ActionDecl,
  BehaviorDecl,
  BinaryOp,
  Expr,
  HalBlock,
  HalMemberDecl,
  ImportDecl,
  NamedArg,
  NodeDecl,
  Program,
  RobotDecl,
  SafetyBlock,
  SafetyRule,
  SafetyZoneDecl,
  AiModelDecl,
  AiConfigEntry,
  AgentDecl,
  SensorBinding,
  SensorDecl,
  ServiceDecl,
  SocDecl,
  Span,
  Stmt,
  TopicDecl,
  UnitKind,
} from "../ast/nodes.js";
import type {
  AgentChannelDecl,
  BusDecl,
  DeviceDecl,
  DiscoverFilter,
  DiscoverTarget,
  MessageDecl,
  PeerRobotDecl,
  QosDecl,
  TopicRole,
  TransportKind,
} from "../comm/index.js";
import { transportFromIdent } from "../comm/index.js";
import type {
  CapabilityDecl,
  EnumDecl,
  EnumVariantDecl,
  EventDecl,
  EventHandlerDecl,
  FieldDecl,
  MatchArm,
  ModuleFnDecl,
  ExternFnDecl,
  TestDecl,
  SelectArm,
  ModuleParamDecl,
  Visibility,
  StateMachineDecl,
  StructDecl,
  TaskDecl,
  TraitDecl,
  TraitImplDecl,
  TraitMethodDecl,
  TraitParamDecl,
  TransitionDecl,
  TwinDecl,
  VerifyDecl,
  ObserveDecl,
  WorldModelDecl,
  HardwareDecl,
  DeployDecl,
  RequiresHardwareDecl,
  RequiresNetworkDecl,
  RequiresConnectivityDecl,
  GeofenceDecl,
  ConnectivityPolicyDecl,
  BluetoothConfigDecl,
  BleServiceDecl,
  SimulateCompatibilityDecl,
  MissionDecl,
  ResourceBudgetDecl,
  TaskPriority,
  PipelineDecl,
  WatchdogDecl,
  ModeDecl,
  RetryDecl,
  RecoverDecl,
  ValidateRuleDecl,
  SubscribeFilterDecl,
  BridgeKind,
  IdentityDecl,
  AuditDecl,
  ProvenanceDecl,
  SignedRecordDecl,
  SecretDecl,
  TrustDecl,
  PermissionsDecl,
  SecureBlockDecl,
  SecureCommPolicyDecl,
  TrustBoundaryDecl,
  SwarmPolicy,
} from "../foundations.js";

export class ParseError extends Error {
  constructor(
    message: string,
    public line: number,
    public column: number,
  ) {
    super(message);
    this.name = "ParseError";
  }
}

export function parse(tokens: Token[]): Program {
  // Parse input.
  //
  // Parameters:
  // - `tokens` — input value
  //
  // Returns:
  // `Program`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = parse(tokens);
  const parser = new Parser(tokens);
  return parser.parseProgram();
}

class Parser {
  private pos = 0;
  private typeParamNames = new Set<string>();

  constructor(private tokens: Token[]) {}

  private peek(): Token {
    // Peek.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Token.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = peek();
    return this.tokens[this.pos];
}

  private previous(): Token {
    // Previous.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Token.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = previous();
    return this.tokens[this.pos - 1];
}

  private advance(): Token {
    // Advance.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Token.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = advance();
    if (this.peek().type !== "EOF") this.pos++;
    return this.previous();
}

  private check(type: Token["type"]): boolean {
    // Check input.
    //
    // Parameters:
    // - `type` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = check(type);
    return this.peek().type === type;
}

  private match(...types: Token["type"][]): boolean {
    // Match.
    //
    // Parameters:
    // - `...types` — rest arguments
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = match(...types);
    for (const t of types) {

      // continue when this.check(t).
      if (this.check(t)) {
        this.advance();
        return true;
      }
    }
    return false;
}

  private expect(type: Token["type"], message: string): Token {
    // Expect.
    //
    // Parameters:
    // - `type` — input value
    // - `message` — input value
    //
    // Returns:
    // Token.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = expect(type, message);
    if (this.check(type)) return this.advance();
    const t = this.peek();
    throw new ParseError(message, t.line, t.column);
}

  private spanFrom(start: Token, end: Token): Span {
    // SpanFrom.
    //
    // Parameters:
    // - `start` — input value
    // - `end` — input value
    //
    // Returns:
    // Span.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = spanFrom(start, end);
    return {
      start: { line: start.line, column: start.column, offset: start.offset },
      end: { line: end.line, column: end.column, offset: end.offset },
    };
}

  private parseLabel(message: string): string {
    // ParseLabel.
    //
    // Parameters:
    // - `message` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseLabel(message);
    const labelTypes: Token["type"][] = [
      "IDENT",
      "PLAN",
      "TWIN",
      "SKILL",
      "MATCH",
      "STATE",
      "EVENT",
      "TASK",
      "ACTION",
      "GOAL",
      "MEMORY",
      "ON",
      "REPLAY",
      "MIRROR",
      "ENTER",
      "EMIT",
      "EXECUTE",
      "DISCOVER",
      "FROM",
      "TO",
      "SUBSCRIBE",
      "MATCHES",
      "RECEIVE",
      "MESSAGE",
      "RESPONSE",
      "FEEDBACK",
      "RESULT",
      "REQUEST",
      "DEVICE",
      "BUS",
      "QOS",
      "RELIABLE",
      "BEST_EFFORT",
      "RATE",
      "HISTORY",
      "DEADLINE",
      "TELEMETRY",
      "FAULTS",
      "VERIFY",
      "REQUIRES",
    ];

    // check membership before continuing.
    if (labelTypes.includes(this.peek().type)) {
      return this.advance().lexeme;
    }
    const t = this.peek();
    throw new ParseError(message, t.line, t.column);
}

  private parseBindingIdent(message: string): string {
    // ParseBindingIdent.
    //
    // Parameters:
    // - `message` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseBindingIdent(message);
    const bindingTypes: Token["type"][] = [
      "IDENT",
      "PLAN",
      "TWIN",
      "SKILL",
      "MATCH",
      "STATE",
      "EVENT",
      "TASK",
      "ACTION",
      "GOAL",
      "MEMORY",
      "ON",
      "REPLAY",
      "MIRROR",
      "ENTER",
      "EMIT",
      "MISSION",
      "DURATION",
      "NETWORK",
      "BANDWIDTH",
      "LATENCY",
      "TIMING",
      "BUDGET",
      "FAULT",
      "EXECUTE",
      "DISCOVER",
      "FROM",
      "TO",
      "SUBSCRIBE",
      "MATCHES",
      "RECEIVE",
      "MESSAGE",
      "RESPONSE",
      "FEEDBACK",
      "RESULT",
      "REQUEST",
      "DEVICE",
      "BUS",
    ];

    // check membership before continuing.
    if (bindingTypes.includes(this.peek().type)) {
      return this.advance().lexeme;
    }
    const t = this.peek();
    throw new ParseError(message, t.line, t.column);
}

  parseProgram(): Program {
    // ParseProgram.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Program.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseProgram();

    const start = this.peek();
    let moduleName: string | null = null;

    if (this.check("MODULE")) {
      this.advance();
      moduleName = this.parseDottedName("Expected module name after 'module'");
      this.expect("SEMICOLON", "Expected ';' after module declaration");
    }

    const imports: ImportDecl[] = [];
    const functions: ModuleFnDecl[] = [];
    const externFunctions: ExternFnDecl[] = [];
    const tests: TestDecl[] = [];
    const structs: StructDecl[] = [];
    const enums: EnumDecl[] = [];
    const traits: TraitDecl[] = [];
    const messages: MessageDecl[] = [];
    const validateRules: ValidateRuleDecl[] = [];
    const robots: RobotDecl[] = [];
    const hardwareProfiles: HardwareDecl[] = [];
    const deployments: DeployDecl[] = [];
    let requiresHardware: RequiresHardwareDecl | null = null;
    let requiresNetwork: RequiresNetworkDecl | null = null;
    let requiresConnectivity: RequiresConnectivityDecl | null = null;
    const geofences: GeofenceDecl[] = [];
    const fleets: import("../foundations.js").FleetDecl[] = [];
    const swarms: import("../foundations.js").SwarmDecl[] = [];
    const programSafetyZones: import("../foundations.js").ProgramSafetyZoneDecl[] = [];
    const certifications: import("../foundations.js").CertifyDecl[] = [];
    const connectivityPolicies: ConnectivityPolicyDecl[] = [];
    const bleServices: BleServiceDecl[] = [];
    let simulateCompatibility: SimulateCompatibilityDecl | null = null;

    while (this.check("IMPORT")) {
      imports.push(this.parseImport());
    }

    while (!this.check("EOF")) {
      if (this.isModuleFnStart()) {
        functions.push(this.parseModuleFn());
      } else if (this.match("EXTERN")) {
        externFunctions.push(this.parseExternFn());
      } else if (this.check("IDENT") && this.peek().lexeme === "test") {
        tests.push(this.parseTest());
      } else if (this.check("STRUCT")) {
        structs.push(this.parseStruct());
      } else if (this.check("ENUM")) {
        enums.push(this.parseEnum());
      } else if (this.check("TRAIT")) {
        traits.push(this.parseTrait());
      } else if (this.check("HARDWARE")) {
        hardwareProfiles.push(this.parseHardware());
      } else if (this.check("DEPLOY")) {
        deployments.push(this.parseDeploy());
      } else if (this.check("REQUIRES_HARDWARE")) {
        requiresHardware = this.parseRequiresHardware();
      } else if (this.check("REQUIRES_NETWORK")) {
        requiresNetwork = this.parseRequiresNetwork();
      } else if (this.check("REQUIRES_CONNECTIVITY")) {
        requiresConnectivity = this.parseRequiresConnectivity();
      } else if (this.check("GEOFENCE")) {
        geofences.push(this.parseGeofence());
      } else if (this.check("FLEET")) {
        fleets.push(this.parseFleet());
      } else if (this.check("SWARM")) {
        swarms.push(this.parseSwarm());
      } else if (this.check("SAFETY_ZONE")) {
        programSafetyZones.push(this.parseProgramSafetyZone());
      } else if (this.check("CERTIFY")) {
        certifications.push(this.parseCertify());
      } else if (this.check("CONNECTIVITY_POLICY")) {
        connectivityPolicies.push(this.parseConnectivityPolicy());
      } else if (this.check("BLE_SERVICE")) {
        bleServices.push(this.parseBleService());
      } else if (this.check("SIMULATE_COMPATIBILITY")) {
        simulateCompatibility = this.parseSimulateCompatibility();
      } else if (this.check("MESSAGE")) {
        messages.push(this.parseMessage());
      } else if (this.check("IDENT") && this.peek().lexeme === "validate") {
        validateRules.push(this.parseValidateRule());
      } else if (this.check("ROBOT")) {
        robots.push(this.parseRobot());
      } else {
        const t = this.peek();
        throw new ParseError(
          "Expected struct, enum, trait, hardware, deploy, validate, or robot declaration",
          t.line,
          t.column,
        );
      }
    }

    const end = this.previous();
    return {
      kind: "Program",
      moduleName,
      imports,
      functions,
      tests,
      externFunctions,
      structs,
      enums,
      traits,
      hardwareProfiles,
      deployments,
      requiresHardware,
      requiresNetwork,
      requiresConnectivity,
      geofences,
      fleets,
      swarms,
      programSafetyZones,
      certifications,
      connectivityPolicies,
      bleServices,
      simulateCompatibility,
      messages,
      validateRules,
      robots,
      span: this.spanFrom(start, end),
    };
  }

  private parseDottedName(message: string): string {
    // ParseDottedName.
    //
    // Parameters:
    // - `message` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseDottedName(message);
    const parts = [this.parseImportSegment(message)];

    // Repeat while this.match("DOT").
    while (this.match("DOT")) {
      parts.push(this.parseImportSegment("Expected name after '.'"));
    }
    return parts.join(".");
}

  private isModuleFnStart(): boolean {
    // IsModuleFnStart.
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

    // const result = isModuleFnStart();
    return (
      this.check("EXPORT") ||
      this.check("PUBLIC") ||
      this.check("PRIVATE") ||
      this.check("ASYNC") ||
      this.check("FN")
    );
}

  private parseTypeParams(): string[] {
    // ParseTypeParams.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // string[].
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseTypeParams();
    if (!this.match("LT")) return [];
    const params: string[] = [];

    // continue when check is falsy.
    if (!this.check("GT")) {

      // Evaluate do.
      do {
        params.push(this.parseLabel("Expected type parameter name"));
      } while (this.match("COMMA"));
    }
    this.expect("GT", "Expected '>' after type parameters");
    return params;
}

  private parseModuleFn(): ModuleFnDecl {
    // ParseModuleFn.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // ModuleFnDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseModuleFn();
    const start = this.peek();
    let visibility: Visibility = "private";

    // continue when this.match("EXPORT").
    if (this.match("EXPORT")) visibility = "export";

    // Otherwise, continue when else if (this.match("PUBLIC").
    else if (this.match("PUBLIC")) visibility = "public";

    // Otherwise, continue when else if (this.match("PRIVATE").
    else if (this.match("PRIVATE")) visibility = "private";
    const isAsync = this.match("ASYNC");
    this.expect("FN", "Expected 'fn' in module function");
    const name = this.parseLabel("Expected function name");
    const typeParams = this.parseTypeParams();
    const savedTypeParams = this.typeParamNames;
    this.typeParamNames = new Set(typeParams);
    this.expect("LPAREN", "Expected '(' after function name");
    const params: ModuleParamDecl[] = [];

    // continue when check is falsy.
    if (!this.check("RPAREN")) {

      // Evaluate do.
      do {
        const pstart = this.peek();
        const pname = this.parseLabel("Expected parameter name");
        this.expect("COLON", "Expected ':' after parameter name");
        const typeAnn = this.parseTypeAnnotation();
        params.push({
          name: pname,
          typeAnn,
          span: this.spanFrom(pstart, this.previous()),
        });
      } while (this.match("COMMA"));
    }
    this.expect("RPAREN", "Expected ')' after parameters");
    this.expect("ARROW", "Expected '->' after function parameters");
    const returnType = this.parseTypeAnnotation();
    this.expect("LBRACE", "Expected '{' after function return type");
    const body = this.parseBlock();
    const end = this.expect("RBRACE", "Expected '}' to close function");
    this.typeParamNames = savedTypeParams;
    return {
      kind: "ModuleFnDecl",
      name,
      visibility,
      typeParams,
      params,
      returnType,
      isAsync,
      body,
      span: this.spanFrom(start, end),
    };
}

  private parseExternFn(): ExternFnDecl {
    // ParseExternFn.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // ExternFnDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseExternFn();
    const start = this.previous();
    let library: string | null = null;
    let bridge: BridgeKind = "native";

    // continue when this.match("STRING").
    if (this.match("STRING")) {
      library = this.previous().value as string;

    // Otherwise, continue when this.check("IDENT").
    } else if (this.check("IDENT")) {
      const lex = this.peek().lexeme;

      // continue when lex equals "python".
      if (lex === "python") {
        this.advance();
        library = "python";
        bridge = "python";

      // Otherwise, continue when lex equals "cpp".
      } else if (lex === "cpp") {
        this.advance();
        library = "cpp";
        bridge = "cpp";
      }
    }
    this.expect("FN", "Expected 'fn' in extern declaration");
    const name = this.parseLabel("Expected extern function name");
    this.expect("LPAREN", "Expected '(' after extern function name");
    const params: ModuleParamDecl[] = [];

    // continue when check is falsy.
    if (!this.check("RPAREN")) {

      // Evaluate do.
      do {
        const pstart = this.peek();
        const pname = this.parseLabel("Expected parameter name");
        this.expect("COLON", "Expected ':' after parameter name");
        const typeAnn = this.parseTypeAnnotation();
        params.push({
          name: pname,
          typeAnn,
          span: this.spanFrom(pstart, this.previous()),
        });
      } while (this.match("COMMA"));
    }
    this.expect("RPAREN", "Expected ')' after extern parameters");
    this.expect("ARROW", "Expected '->' after extern parameters");
    const returnType = this.parseTypeAnnotation();
    const end = this.expect("SEMICOLON", "Expected ';' after extern declaration");
    return {
      kind: "ExternFnDecl",
      name,
      library,
      bridge,
      params,
      returnType,
      span: this.spanFrom(start, end),
    };
}

  private parseTest(): TestDecl {
    // ParseTest.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // TestDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseTest();
    const start = this.advance();
    const nameTok = this.expect("STRING", "Expected test name string");
    const name = nameTok.value as string;
    this.expect("LBRACE", "Expected '{' after test name");
    const body = this.parseBlock();
    const end = this.expect("RBRACE", "Expected '}' to close test");
    return {
      kind: "TestDecl",
      name,
      body,
      span: this.spanFrom(start, end),
    };
}

  private parseImport(): ImportDecl {
    // ParseImport.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // ImportDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseImport();
    const start = this.advance();
    const vendor = this.parseImportSegment("Expected library vendor name");
    this.expect("DOT", "Expected '.' in import path");
    const module = this.parseImportSegment("Expected library module name");
    this.expect("SEMICOLON", "Expected ';' after import");
    const end = this.previous();
    return {
      kind: "ImportDecl",
      path: `${vendor}.${module}`,
      span: this.spanFrom(start, end),
    };
}

  private parseImportSegment(message: string): string {
    // ParseImportSegment.
    //
    // Parameters:
    // - `message` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseImportSegment(message);
    if (this.check("IDENT")) {
      return this.advance().lexeme;
    }

    // continue when this.check("EOF") || this.check("DOT") || this.check("SEMICOLON").
    if (this.check("EOF") || this.check("DOT") || this.check("SEMICOLON")) {
      const t = this.peek();
      throw new ParseError(message, t.line, t.column);
    }
    return this.advance().lexeme;
}

  private parseTypeNamePart(message: string): string {
    // ParseTypeNamePart.
    //
    // Parameters:
    // - `message` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseTypeNamePart(message);
    if (this.check("IDENT")) {
      return this.advance().lexeme;
    }
    return this.parseLabel(message);
}

  private finishGenericTypeName(base: string): string {
    // FinishGenericTypeName.
    //
    // Parameters:
    // - `base` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = finishGenericTypeName(base);
    this.expect("LT", "Expected '<' to open generic type");
    const args: string[] = [];

    // continue when check is falsy.
    if (!this.check("GT")) {

      // Evaluate do.
      do {
        args.push(this.parseTypeName());
      } while (this.match("COMMA"));
    }
    this.expect("GT", "Expected '>' to close generic type");
    return `${base}<${args.join(", ")}>`;
}

  private parseTypeName(): string {
    // ParseTypeName.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseTypeName();
    let name = this.parseTypeNamePart("Expected type name");

    // continue when this.check("LT").
    if (this.check("LT")) {
      name = this.finishGenericTypeName(name);
    }
    return name;
}

  private parseTypeAnnotation(): SpandaType {
    // ParseTypeAnnotation.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // SpandaType.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseTypeAnnotation();
    if (this.match("DYN")) {
      const traitName = this.parseTypeNamePart("Expected trait name after dyn");
      return { kind: "trait_object", traitName };
    }
    const parts = [this.parseTypeNamePart("Expected type name")];

    // Repeat while this.match("DOT").
    while (this.match("DOT")) {
      parts.push(this.parseTypeNamePart("Expected type name after '.'"));
    }
    const qualified = parts.join(".");

    // continue when this.match("LT").
    if (this.match("LT")) {
      const args: SpandaType[] = [];

      // continue when check is falsy.
      if (!this.check("GT")) {

        // Evaluate do.
        do {
          args.push(this.parseTypeAnnotation());
        } while (this.match("COMMA"));
      }
      this.expect("GT", "Expected '>' to close generic type");
      const base = parts[parts.length - 1] ?? qualified;

      // Try the operation and handle failures below.
      try {
        return resolveGenericType(base, args);
      } catch (e) {
        const t = this.previous();
        throw new ParseError((e as Error).message, t.line, t.column);
      }
    }

    // Try the operation and handle failures below.
    try {
      return resolveTypeName(qualified);
    } catch (e) {

      // continue when this.typeParamNames.has(qualified) || this.typeParamNames.has(parts[pa.
      if (this.typeParamNames.has(qualified) || this.typeParamNames.has(parts[parts.length - 1] ?? qualified)) {
        return { kind: "named", name: parts[parts.length - 1] ?? qualified };
      }
      const t = this.peek();
      throw new ParseError((e as Error).message, t.line, t.column);
    }
}

  private parseStruct(): StructDecl {
    // ParseStruct.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // StructDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseStruct();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected struct name");
    const typeParams = this.parseTypeParams();
    this.expect("LBRACE", "Expected '{' after struct name");
    const fields: FieldDecl[] = [];

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {
      const fieldStart = this.peek();
      const fieldName = this.expect("IDENT", "Expected field name");
      this.expect("COLON", "Expected ':' after field name");
      const typeName = this.parseTypeName();
      this.expect("SEMICOLON", "Expected ';' after field");
      fields.push({
        name: fieldName.lexeme,
        typeName,
        span: this.spanFrom(fieldStart, this.previous()),
      });
    }
    const end = this.expect("RBRACE", "Expected '}' to close struct");
    return {
      kind: "StructDecl",
      name: name.lexeme,
      typeParams: typeParams.length > 0 ? typeParams : undefined,
      fields,
      span: this.spanFrom(start, end),
    };
}

  private parseEnum(): EnumDecl {
    // ParseEnum.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // EnumDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseEnum();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected enum name");
    this.expect("LBRACE", "Expected '{' after enum name");
    const variants: EnumVariantDecl[] = [];

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {
      const variantStart = this.peek();
      const variantName = this.expect("IDENT", "Expected enum variant");
      const fieldTypes: string[] = [];

      // continue when this.match("LPAREN").
      if (this.match("LPAREN")) {

        // Repeat while !this.check("RPAREN") && !this.check("EOF").
        while (!this.check("RPAREN") && !this.check("EOF")) {
          fieldTypes.push(this.parseTypeName());

          // continue when match is falsy.
          if (!this.match("COMMA")) break;
        }
        this.expect("RPAREN", "Expected ')' after enum variant fields");
      }
      variants.push({
        name: variantName.lexeme,
        fieldTypes,
        span: this.spanFrom(variantStart, this.previous()),
      });

      // continue when this.match("COMMA").
      if (this.match("COMMA")) continue;
    }
    const end = this.expect("RBRACE", "Expected '}' to close enum");
    return {
      kind: "EnumDecl",
      name: name.lexeme,
      variants,
      span: this.spanFrom(start, end),
    };
}

  private parseTrait(): TraitDecl {
    // ParseTrait.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // TraitDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseTrait();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected trait name");
    this.expect("LBRACE", "Expected '{' after trait name");
    const methods: TraitMethodDecl[] = [];

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {
      methods.push(this.parseTraitMethod());
    }
    const end = this.expect("RBRACE", "Expected '}' to close trait");
    return {
      kind: "TraitDecl",
      name: name.lexeme,
      methods,
      span: this.spanFrom(start, end),
    };
}

  private parseTraitMethod(): TraitMethodDecl {
    // ParseTraitMethod.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // TraitMethodDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseTraitMethod();
    const start = this.advance(); // fn
    const name = this.parseLabel("Expected method name after fn");
    this.expect("LPAREN", "Expected '(' after method name");
    const params: TraitParamDecl[] = [];

    // continue when check is falsy.
    if (!this.check("RPAREN")) {

      // Evaluate do.
      do {
        const paramStart = this.peek();
        const paramName = this.parseLabel("Expected parameter name");
        this.expect("COLON", "Expected ':' after parameter name");
        const typeName = this.expect("IDENT", "Expected parameter type");
        params.push({
          name: paramName,
          typeName: typeName.lexeme,
          span: this.spanFrom(paramStart, this.previous()),
        });
      } while (this.match("COMMA"));
    }
    this.expect("RPAREN", "Expected ')' after parameters");
    this.expect("ARROW", "Expected '->' after trait method parameters");
    const returnType = this.expect("IDENT", "Expected return type");
    this.expect("SEMICOLON", "Expected ';' after trait method");
    return {
      name,
      params,
      returnType: returnType.lexeme,
      span: this.spanFrom(start, this.previous()),
    };
}

  private parseRobot(): RobotDecl {
    // ParseRobot.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // RobotDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseRobot();
    const start = this.expect("ROBOT", "Expected 'robot'");
    const nameTok = this.expect("IDENT", "Expected robot name");
    this.expect("LBRACE", "Expected '{' after robot name");
    let soc: SocDecl | null = null;
    let hal: HalBlock | null = null;
    const nodes: NodeDecl[] = [];
    const topics: TopicDecl[] = [];
    const services: ServiceDecl[] = [];
    const actions: ActionDecl[] = [];
    const sensors: SensorDecl[] = [];
    const actuators: ActuatorDecl[] = [];
    let safety: SafetyBlock | null = null;
    const ai_models: AiModelDecl[] = [];
    const agents: AgentDecl[] = [];
    const behaviors: BehaviorDecl[] = [];
    const tasks: TaskDecl[] = [];
    const pipelines: PipelineDecl[] = [];
    const watchdogs: WatchdogDecl[] = [];
    const modes: ModeDecl[] = [];
    const retries: RetryDecl[] = [];
    const recovers: RecoverDecl[] = [];
    let mission: MissionDecl | null = null;
    const stateMachines: StateMachineDecl[] = [];
    const events: EventDecl[] = [];
    const eventHandlers: EventHandlerDecl[] = [];
    let twin: TwinDecl | null = null;
    let verify: VerifyDecl | null = null;
    let observe: ObserveDecl | null = null;
    let worldModel: WorldModelDecl | null = null;
    let identity: IdentityDecl | null = null;
    let audit: AuditDecl | null = null;
    let provenance: ProvenanceDecl | null = null;
    const signedRecords: SignedRecordDecl[] = [];
    const secrets: SecretDecl[] = [];
    let trust: TrustDecl | null = null;
    let permissions: PermissionsDecl | null = null;
    let requiresConnectivity: RequiresConnectivityDecl | null = null;
    let bluetooth: BluetoothConfigDecl | null = null;
    const traitImpls: TraitImplDecl[] = [];
    const buses: BusDecl[] = [];
    const peerRobots: PeerRobotDecl[] = [];
    const devices: DeviceDecl[] = [];
    const agentChannels: AgentChannelDecl[] = [];
    const twinSync = null;
    let secureComm: SecureCommPolicyDecl | null = null;
    const trustBoundaries: TrustBoundaryDecl[] = [];

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {

      // continue when this.check("SOC").
      if (this.check("SOC")) {
        soc = this.parseSoc();

      // Otherwise, continue when this.check("HAL").
      } else if (this.check("HAL")) {
        hal = this.parseHal();

      // Otherwise, continue when this.check("NODE").
      } else if (this.check("NODE")) {
        nodes.push(this.parseNode());

      // Otherwise, continue when this.check("TOPIC").
      } else if (this.check("TOPIC")) {
        topics.push(this.parseTopic());

      // Otherwise, continue when this.check("SERVICE").
      } else if (this.check("SERVICE")) {
        services.push(this.parseService());

      // Otherwise, continue when this.check("ACTION").
      } else if (this.check("ACTION")) {
        actions.push(this.parseAction());

      // Otherwise, continue when this.check("SENSOR").
      } else if (this.check("SENSOR")) {
        sensors.push(this.parseSensor());

      // Otherwise, continue when this.check("ACTUATOR").
      } else if (this.check("ACTUATOR")) {
        actuators.push(this.parseActuator());

      // Otherwise, continue when this.check("SAFETY").
      } else if (this.check("SAFETY")) {
        safety = this.parseSafety();

      // Otherwise, continue when this.check("AI MODEL").
      } else if (this.check("AI_MODEL")) {
        ai_models.push(this.parseAiModelDecl());

      // Otherwise, continue when this.check("IDENT") && this.isAgentChannel().
      } else if (this.check("IDENT") && this.isAgentChannel()) {
        agentChannels.push(this.parseAgentChannel());

      // Otherwise, continue when this.check("AGENT").
      } else if (this.check("AGENT")) {

        // continue when this.isAgentShorthand().
        if (this.isAgentShorthand()) {
          this.parseAgentShorthand(agents);

        // Handle any remaining cases.
        } else {
          agents.push(this.parseAgent());
        }

      // Otherwise, continue when this.check("BEHAVIOR").
      } else if (this.check("BEHAVIOR")) {
        behaviors.push(this.parseBehavior());

      // Otherwise, continue when this.check("TASK").
      } else if (this.check("TASK")) {
        tasks.push(this.parseTask());

      // Otherwise, continue when this.check("PIPELINE").
      } else if (this.check("PIPELINE")) {
        pipelines.push(this.parsePipeline());

      // Otherwise, continue when this.check("WATCHDOG").
      } else if (this.check("WATCHDOG")) {
        watchdogs.push(this.parseWatchdog());

      // Otherwise, continue when this.check("RETRY").
      } else if (this.check("RETRY")) {
        retries.push(this.parseRetry());

      // Otherwise, continue when this.check("RECOVER").
      } else if (this.check("RECOVER")) {
        recovers.push(this.parseRecover());

      // Otherwise, continue when this.isRobotMemberKeyword("mode").
      } else if (this.isRobotMemberKeyword("mode")) {
        modes.push(this.parseMode());

      // Otherwise, continue when this.check("STATE MACHINE").
      } else if (this.check("STATE_MACHINE")) {
        stateMachines.push(this.parseStateMachine());

      // Otherwise, continue when this.check("EVENT").
      } else if (this.check("EVENT")) {
        events.push(this.parseEvent());

      // Otherwise, continue when this.check("ON").
      } else if (this.check("ON")) {
        eventHandlers.push(this.parseOnTrigger());

      // Otherwise, continue when this.check("TWIN").
      } else if (this.check("TWIN")) {
        twin = this.parseTwin();

      // Otherwise, continue when this.check("VERIFY").
      } else if (this.check("VERIFY")) {
        verify = this.parseVerify();

      // Otherwise, continue when this.check("OBSERVE").
      } else if (this.check("OBSERVE")) {
        observe = this.parseObserve();

      // Otherwise, continue when this.isRobotMemberKeyword("world_model").
      } else if (this.isRobotMemberKeyword("world_model")) {
        worldModel = this.parseWorldModel();

      // Otherwise, continue when this.isRobotMemberKeyword("identity").
      } else if (this.isRobotMemberKeyword("identity")) {
        identity = this.parseIdentity();

      // Otherwise, continue when this.isRobotMemberKeyword("audit").
      } else if (this.isRobotMemberKeyword("audit")) {
        audit = this.parseAudit();

      // Otherwise, continue when this.isRobotMemberKeyword("provenance").
      } else if (this.isRobotMemberKeyword("provenance")) {
        provenance = this.parseProvenance();

      // Otherwise, continue when this.isRobotMemberKeyword("record").
      } else if (this.isRobotMemberKeyword("record")) {
        signedRecords.push(this.parseSignedRecord());

      // Otherwise, continue when this.isRobotMemberKeyword("secrets").
      } else if (this.isRobotMemberKeyword("secrets")) {
        secrets.push(...this.parseSecretsBlock());

      // Otherwise, continue when this.check("SECRET").
      } else if (this.check("SECRET")) {
        secrets.push(this.parseSecret());

      // Otherwise, continue when this.isRobotMemberKeyword("secure_comm").
      } else if (this.isRobotMemberKeyword("secure_comm")) {
        secureComm = this.parseSecureCommPolicy();

      // Otherwise, continue when this.isRobotMemberKeyword("trust_boundary").
      } else if (this.isRobotMemberKeyword("trust_boundary")) {
        trustBoundaries.push(this.parseTrustBoundary());

      // Otherwise, continue when this.check("TRUST").
      } else if (this.check("TRUST")) {
        trust = this.parseTrust();

      // Otherwise, continue when this.check("PERMISSIONS").
      } else if (this.check("PERMISSIONS")) {
        permissions = this.parsePermissions();

      // Otherwise, continue when this.check("REQUIRES_CONNECTIVITY").
      } else if (this.check("REQUIRES_CONNECTIVITY")) {
        requiresConnectivity = this.parseRequiresConnectivity();

      // Otherwise, continue when this.check("BLUETOOTH").
      } else if (this.check("BLUETOOTH")) {
        bluetooth = this.parseBluetoothConfig();

      // Otherwise, continue when this.check("MISSION").
      } else if (this.check("MISSION")) {
        mission = this.parseMission();

      // Otherwise, continue when this.check("IMPL").
      } else if (this.check("IMPL")) {
        traitImpls.push(this.parseTraitImpl());

      // Otherwise, continue when this.check("BUS").
      } else if (this.check("BUS")) {
        buses.push(this.parseBus());

      // Otherwise, continue when this.check("ROBOT").
      } else if (this.check("ROBOT")) {
        peerRobots.push(this.parsePeerRobot());

      // Otherwise, continue when this.check("DEVICE").
      } else if (this.check("DEVICE")) {
        devices.push(this.parseDevice());

      // Handle any remaining cases.
      } else {
        const t = this.peek();
        throw new ParseError("Expected robot member declaration", t.line, t.column);
      }
    }
    const end = this.expect("RBRACE", "Expected '}' to close robot block");
    return {
      kind: "RobotDecl",
      name: nameTok.lexeme,
      soc,
      hal,
      nodes,
      topics,
      services,
      actions,
      sensors,
      actuators,
      safety,
      ai_models,
      agents,
      behaviors,
      tasks,
      pipelines,
      watchdogs,
      modes,
      retries,
      recovers,
      mission,
      stateMachines,
      events,
      eventHandlers,
      twin,
      verify,
      observe,
      worldModel,
      identity,
      audit,
      provenance,
      signedRecords,
      secrets,
      trust,
      permissions,
      requiresConnectivity,
      bluetooth,
      traitImpls,
      buses,
      peerRobots,
      devices,
      agentChannels,
      twinSync,
      secureComm,
      trustBoundaries,
      span: this.spanFrom(start, end),
    };
}

  private isRobotMemberKeyword(kw: string): boolean {
    // IsRobotMemberKeyword.
    //
    // Parameters:
    // - `kw` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = isRobotMemberKeyword(kw);
    return this.check("IDENT") && this.peek().lexeme === kw;
}

  private isAgentShorthand(): boolean {
    // IsAgentShorthand.
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

    // const result = isAgentShorthand();
    let idx = this.pos + 1;

    // continue when idx >= this.tokens.length.
    if (idx >= this.tokens.length) return false;

    // continue when type differs from "IDENT".
    if (this.tokens[idx]?.type !== "IDENT") return false;
    idx += 1;
    return idx < this.tokens.length && this.tokens[idx]?.type === "SEMICOLON";
}

  private isAgentChannel(): boolean {
    // IsAgentChannel.
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

    // const result = isAgentChannel();
    const idx = this.pos;
    return (
      idx + 2 < this.tokens.length &&
      this.tokens[idx]?.type === "IDENT" &&
      this.tokens[idx + 1]?.type === "ARROW" &&
      this.tokens[idx + 2]?.type === "IDENT"
    );
}

  private parseAgentShorthand(agents: AgentDecl[]): void {
    // ParseAgentShorthand.
    //
    // Parameters:
    // - `agents` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseAgentShorthand(agents);
    const start = this.advance();
    const name = this.expect("IDENT", "Expected agent name");
    this.expect("SEMICOLON", "Expected ';' after agent reference");
    agents.push({
      kind: "AgentDecl",
      name: name.lexeme,
      usesAi: [],
      memoryKind: null,
      tools: [],
      skills: [],
      capabilities: [],
      goal: "",
      planBody: [],
      span: this.spanFrom(start, this.previous()),
    });
}

  private parseBus(): BusDecl {
    const start = this.advance();
    const busName = this.expect("IDENT", "Expected bus name");

    if (this.check("LBRACE")) {
      this.advance();
      let transportName = busName.lexeme;
      let brokerUrl: string | null = null;
      let encryption: string | null = null;
      let authentication: string | null = null;
      let integrity: string | null = null;

      while (!this.check("RBRACE") && !this.check("EOF")) {
        const key = this.parseConfigKeyToken();
        this.expect("COLON", "Expected ':' in bus field");
        if (key === "transport") {
          transportName = this.parseConfigValueString();
        } else if (key === "url") {
          brokerUrl = this.parseConfigValueString();
        } else if (key === "encryption") {
          encryption = this.parseLabel("Expected encryption mode");
        } else if (key === "authentication") {
          authentication = this.parseLabel("Expected authentication mode");
        } else if (key === "integrity") {
          integrity = this.parseLabel("Expected integrity mode");
        } else {
          const t = this.peek();
          throw new ParseError(`Unknown bus field '${key}'`, t.line, t.column);
        }
        this.expect("SEMICOLON", "Expected ';' after bus field");
      }
      const end = this.expect("RBRACE", "Expected '}' to close bus block");
      this.match("SEMICOLON");
      const transport = transportFromIdent(transportName) ?? "local";
      return {
        kind: "BusDecl",
        name: busName.lexeme,
        transport,
        transportName,
        brokerUrl,
        encryption,
        authentication,
        integrity,
        span: this.spanFrom(start, end),
      };
    }

    this.expect("SEMICOLON", "Expected ';' after bus declaration");
    const transport = transportFromIdent(busName.lexeme) ?? "local";
    return {
      kind: "BusDecl",
      name: busName.lexeme,
      transport,
      span: this.spanFrom(start, this.previous()),
    };
}

  private parsePeerRobot(): PeerRobotDecl {
    // ParsePeerRobot.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // PeerRobotDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parsePeerRobot();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected peer robot name");
    this.expect("SEMICOLON", "Expected ';' after peer robot");
    return {
      kind: "PeerRobotDecl",
      name: name.lexeme,
      span: this.spanFrom(start, this.previous()),
    };
}

  private parseDevice(): DeviceDecl {
    // ParseDevice.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // DeviceDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseDevice();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected device name");
    this.expect("COLON", "Expected ':' after device name");
    const deviceType = this.expect("IDENT", "Expected device type");
    this.expect("SEMICOLON", "Expected ';' after device declaration");
    return {
      kind: "DeviceDecl",
      name: name.lexeme,
      deviceType: deviceType.lexeme,
      span: this.spanFrom(start, this.previous()),
    };
}

  private parseAgentChannel(): AgentChannelDecl {
    // ParseAgentChannel.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // AgentChannelDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseAgentChannel();
    const start = this.peek();
    const fromAgent = this.expect("IDENT", "Expected source agent").lexeme;
    this.expect("ARROW", "Expected '->' in agent channel");
    const toAgent = this.expect("IDENT", "Expected target agent").lexeme;
    this.expect("SEMICOLON", "Expected ';' after agent channel");
    return {
      kind: "AgentChannelDecl",
      fromAgent,
      toAgent,
      messageType: "",
      span: this.spanFrom(start, this.previous()),
    };
}

  private parseMessage(): MessageDecl {
    // ParseMessage.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // MessageDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseMessage();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected message name");
    this.expect("LBRACE", "Expected '{' after message name");
    const fields: FieldDecl[] = [];
    let version: number | null = null;

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {

      // continue when lexeme equals "version".
      if (this.check("IDENT") && this.peek().lexeme === "version") {
        this.advance();
        this.expect("COLON", "Expected ':' after version");
        version = this.parseNumberValue();
        this.expect("SEMICOLON", "Expected ';' after version");
        continue;
      }
      const fieldStart = this.peek();
      const fieldName = this.expect("IDENT", "Expected field name");
      this.expect("COLON", "Expected ':' after field name");
      const typeName = this.expect("IDENT", "Expected field type");
      this.expect("SEMICOLON", "Expected ';' after field");
      fields.push({
        name: fieldName.lexeme,
        typeName: typeName.lexeme,
        span: this.spanFrom(fieldStart, this.previous()),
      });
    }
    const end = this.expect("RBRACE", "Expected '}' to close message");
    return {
      kind: "MessageDecl",
      name: name.lexeme,
      fields,
      version,
      span: this.spanFrom(start, end),
    };
}

  private parseNumberValue(): number {
    // ParseNumberValue.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseNumberValue();
    const tok = this.expect("NUMBER", "Expected number");
    return tok.value as number;
}

  private parseStorageAmount(): number {
    // ParseStorageAmount.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseStorageAmount();
    const value = this.parseNumberValue();

    // continue when this.check("IDENT").
    if (this.check("IDENT")) {
      const unit = this.peek().lexeme;
      let mb = value;

      // continue when unit equals "GB" || unit === "Gb".
      if (unit === "GB" || unit === "Gb") mb = value * 1024;

      // Otherwise, continue when else if equals "MB" || unit === "Mb".
      else if (unit === "MB" || unit === "Mb") mb = value;

      // Otherwise, continue when else if equals "TB" || unit === "Tb".
      else if (unit === "TB" || unit === "Tb") mb = value * 1024 * 1024;

      // continue when unit equals "GB" || unit === "MB" || unit === "TB" || unit === "Gb" || unit === "Mb" || unit === "Tb".
      if (unit === "GB" || unit === "MB" || unit === "TB" || unit === "Gb" || unit === "Mb" || unit === "Tb") {
        this.advance();
      }
      return mb;
    }
    return value;
}

  private parseNetworkAmount(): number {
    // ParseNetworkAmount.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseNetworkAmount();
    const value = this.parseNumberValue();

    // continue when this.check("IDENT").
    if (this.check("IDENT")) {
      const unit = this.peek().lexeme;

      // continue when unit equals "Mbps" || unit === "mbps".
      if (unit === "Mbps" || unit === "mbps") {
        this.advance();
        return value;
      }

      // continue when unit equals "Gbps" || unit === "gbps".
      if (unit === "Gbps" || unit === "gbps") {
        this.advance();
        return value * 1000;
      }
    }
    return value;
}

  private parseEnergyWhValue(): number {
    // ParseEnergyWhValue.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseEnergyWhValue();
    const tok = this.peek();

    // continue when type equals unit === "Wh".
    if (tok.type === "UNIT_LITERAL" && tok.unit === "Wh") {
      this.advance();
      return tok.value as number;
    }
    const value = this.parseNumberValue();

    // continue when lexeme equals "Wh".
    if (this.check("IDENT") && this.peek().lexeme === "Wh") {
      this.advance();
    }
    return value;
}

  private parsePowerWValue(): number {
    // ParsePowerWValue.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parsePowerWValue();
    const tok = this.peek();

    // continue when type equals unit === "W".
    if (tok.type === "UNIT_LITERAL" && tok.unit === "W") {
      this.advance();
      return tok.value as number;
    }
    const value = this.parseNumberValue();

    // continue when lexeme equals "W".
    if (this.check("IDENT") && this.peek().lexeme === "W") {
      this.advance();
    }
    return value;
}

  private parseHardwareTypeList(kind: string): string[] {
    // ParseHardwareTypeList.
    //
    // Parameters:
    // - `kind` — input value
    //
    // Returns:
    // string[].
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseHardwareTypeList(kind);
    this.expect("LBRACKET", `Expected '[' after ${kind}`);
    const items: string[] = [];

    // continue when check is falsy.
    if (!this.check("RBRACKET")) {

      // Evaluate do.
      do {
        items.push(this.parseLabel(`Expected ${kind} type name`));
      } while (this.match("COMMA") && !this.check("RBRACKET"));
    }
    this.expect("RBRACKET", `Expected ']' after ${kind} list`);
    this.expect("SEMICOLON", `Expected ';' after ${kind} list`);
    return items;
}

  private parseHardware(): HardwareDecl {
    // ParseHardware.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HardwareDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseHardware();
    const start = this.advance();
    const name = this.parseLabel("Expected hardware profile name");
    this.expect("LBRACE", "Expected '{' after hardware name");
    let cpu: string | null = null;
    let memoryMb: number | null = null;
    let storageMb: number | null = null;
    let gpuTops: number | null = null;
    let gpuRequired = false;
    let sensors: string[] = [];
    let actuators: string[] = [];
    let connectivity: string[] = [];
    let batteryWh: number | null = null;
    let networkBandwidthMbps: number | null = null;
    let networkLatencyMs: number | null = null;
    let minControlPeriodMs: number | null = null;
    let powerDrawW: number | null = null;

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {

      // continue when this.match("CPU").
      if (this.match("CPU")) {
        this.expect("COLON", "Expected ':' after cpu");
        cpu = this.parseLabel("Expected CPU identifier");
        this.expect("SEMICOLON", "Expected ';' after cpu");

      // Otherwise, continue when this.match("MEMORY").
      } else if (this.match("MEMORY")) {
        this.expect("COLON", "Expected ':' after memory");
        memoryMb = this.parseStorageAmount();
        this.expect("SEMICOLON", "Expected ';' after memory");

      // Otherwise, continue when this.match("STORAGE").
      } else if (this.match("STORAGE")) {
        this.expect("COLON", "Expected ':' after storage");
        storageMb = this.parseStorageAmount();
        this.expect("SEMICOLON", "Expected ';' after storage");

      // Otherwise, continue when this.match("GPU").
      } else if (this.match("GPU")) {
        this.expect("COLON", "Expected ':' after gpu");

        // continue when this.match("TRUE").
        if (this.match("TRUE")) {
          gpuRequired = true;

        // Handle any remaining cases.
        } else {
          gpuTops = this.parseNumberValue();

          // continue when lexeme equals "TOPS".
          if (this.check("IDENT") && this.peek().lexeme === "TOPS") {
            this.advance();
          }
        }
        this.expect("SEMICOLON", "Expected ';' after gpu");

      // Otherwise, continue when this.match("SENSORS").
      } else if (this.match("SENSORS")) {
        sensors = this.parseHardwareTypeList("sensors");

      // Otherwise, continue when this.match("ACTUATORS").
      } else if (this.match("ACTUATORS")) {
        actuators = this.parseHardwareTypeList("actuators");

      // Otherwise, continue when this.match("CONNECTIVITY").
      } else if (this.match("CONNECTIVITY")) {
        connectivity = this.parseHardwareTypeList("connectivity");

      // Otherwise, continue when this.match("BATTERY").
      } else if (this.match("BATTERY")) {
        this.expect("LBRACE", "Expected '{' after battery");

        // Repeat while !this.check("RBRACE") && !this.check("EOF").
        while (!this.check("RBRACE") && !this.check("EOF")) {
          this.expect("CAPACITY", "Expected capacity in battery block");
          this.expect("COLON", "Expected ':' after capacity");
          batteryWh = this.parseEnergyWhValue();
          this.expect("SEMICOLON", "Expected ';' after capacity");
        }
        this.expect("RBRACE", "Expected '}' to close battery block");

      // Otherwise, continue when this.match("NETWORK").
      } else if (this.match("NETWORK")) {
        this.expect("LBRACE", "Expected '{' after network");

        // Repeat while !this.check("RBRACE") && !this.check("EOF").
        while (!this.check("RBRACE") && !this.check("EOF")) {

          // continue when this.match("BANDWIDTH").
          if (this.match("BANDWIDTH")) {
            this.expect("COLON", "Expected ':' after bandwidth");
            networkBandwidthMbps = this.parseNetworkAmount();
            this.expect("SEMICOLON", "Expected ';' after bandwidth");

          // Otherwise, continue when this.match("LATENCY").
          } else if (this.match("LATENCY")) {
            this.expect("COLON", "Expected ':' after latency");
            networkLatencyMs = this.parseDuration();
            this.expect("SEMICOLON", "Expected ';' after latency");

          // Handle any remaining cases.
          } else {
            const t = this.peek();
            throw new ParseError("Expected bandwidth or latency in network block", t.line, t.column);
          }
        }
        this.expect("RBRACE", "Expected '}' to close network block");

      // Otherwise, continue when this.match("TIMING").
      } else if (this.match("TIMING")) {
        this.expect("LBRACE", "Expected '{' after timing");

        // Repeat while !this.check("RBRACE") && !this.check("EOF").
        while (!this.check("RBRACE") && !this.check("EOF")) {
          this.expect("MIN_PERIOD", "Expected min_period in timing block");
          this.expect("COLON", "Expected ':' after min_period");
          minControlPeriodMs = this.parseDuration();
          this.expect("SEMICOLON", "Expected ';' after min_period");
        }
        this.expect("RBRACE", "Expected '}' to close timing block");

      // Otherwise, continue when this.match("RESOURCE").
      } else if (this.match("RESOURCE")) {
        this.expect("COLON", "Expected ':' after resource");
        powerDrawW = this.parsePowerWValue();
        this.expect("SEMICOLON", "Expected ';' after resource power");

      // Handle any remaining cases.
      } else {
        const t = this.peek();
        throw new ParseError("Expected hardware profile member", t.line, t.column);
      }
    }
    const end = this.expect("RBRACE", "Expected '}' to close hardware block");
    return {
      kind: "HardwareDecl",
      name,
      cpu,
      memoryMb,
      storageMb,
      gpuTops,
      gpuRequired,
      sensors,
      actuators,
      connectivity,
      batteryWh,
      networkBandwidthMbps,
      networkLatencyMs,
      minControlPeriodMs,
      powerDrawW,
      span: this.spanFrom(start, end),
    };
}

  private parseDeploy(): DeployDecl {
    // ParseDeploy.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // DeployDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseDeploy();
    const start = this.advance();
    const robotName = this.parseLabel("Expected robot name after deploy");
    this.expect("TO", "Expected 'to' after deploy robot name");
    const targets: string[] = [];

    // continue when this.match("LBRACKET").
    if (this.match("LBRACKET")) {

      // continue when check is falsy.
      if (!this.check("RBRACKET")) {

        // Evaluate do.
        do {
          targets.push(this.parseLabel("Expected hardware target name"));
        } while (this.match("COMMA"));
      }
      this.expect("RBRACKET", "Expected ']' after deploy targets");

    // Handle any remaining cases.
    } else {
      targets.push(this.parseLabel("Expected hardware target name"));
    }
    this.expect("SEMICOLON", "Expected ';' after deploy statement");
    const end = this.previous();
    return { kind: "DeployDecl", robotName, targets, span: this.spanFrom(start, end) };
}

  private parseRequiresHardware(): RequiresHardwareDecl {
    // ParseRequiresHardware.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // RequiresHardwareDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseRequiresHardware();
    const start = this.advance();
    this.expect("LBRACE", "Expected '{' after requires_hardware");
    let memoryMbMin: number | null = null;
    let storageMbMin: number | null = null;
    let gpuTopsMin: number | null = null;
    let gpuRequired = false;
    let sensors: string[] = [];
    let actuators: string[] = [];

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {

      // continue when this.match("MEMORY").
      if (this.match("MEMORY")) {
        this.expect("GTE", "Expected '>=' after memory in requires_hardware");
        memoryMbMin = this.parseStorageAmount();
        this.expect("SEMICOLON", "Expected ';' after memory requirement");

      // Otherwise, continue when this.match("STORAGE").
      } else if (this.match("STORAGE")) {
        this.expect("GTE", "Expected '>=' after storage in requires_hardware");
        storageMbMin = this.parseStorageAmount();
        this.expect("SEMICOLON", "Expected ';' after storage requirement");

      // Otherwise, continue when this.match("GPU").
      } else if (this.match("GPU")) {

        // continue when this.match("GTE").
        if (this.match("GTE")) {
          gpuTopsMin = this.parseNumberValue();

          // continue when lexeme equals "TOPS".
          if (this.check("IDENT") && this.peek().lexeme === "TOPS") {
            this.advance();
          }

        // Handle any remaining cases.
        } else {
          this.expect("COLON", "Expected ':' or '>=' after gpu");

          // continue when this.match("TRUE").
          if (this.match("TRUE")) {
            gpuRequired = true;

          // Handle any remaining cases.
          } else {
            gpuTopsMin = this.parseNumberValue();

            // continue when lexeme equals "TOPS".
            if (this.check("IDENT") && this.peek().lexeme === "TOPS") {
              this.advance();
            }
          }
        }
        this.expect("SEMICOLON", "Expected ';' after gpu requirement");

      // Otherwise, continue when this.match("SENSORS").
      } else if (this.match("SENSORS")) {
        sensors = this.parseHardwareTypeList("sensors");

      // Otherwise, continue when this.match("ACTUATORS").
      } else if (this.match("ACTUATORS")) {
        actuators = this.parseHardwareTypeList("actuators");

      // Handle any remaining cases.
      } else {
        const t = this.peek();
        throw new ParseError("Expected requires_hardware member", t.line, t.column);
      }
    }
    const end = this.expect("RBRACE", "Expected '}' to close requires_hardware");
    return {
      kind: "RequiresHardwareDecl",
      memoryMbMin,
      storageMbMin,
      gpuTopsMin,
      gpuRequired,
      sensors,
      actuators,
      span: this.spanFrom(start, end),
    };
}

  private parseRequiresNetwork(): RequiresNetworkDecl {
    // ParseRequiresNetwork.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // RequiresNetworkDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseRequiresNetwork();
    const start = this.advance();
    this.expect("LBRACE", "Expected '{' after requires_network");
    let bandwidthMbpsMin: number | null = null;
    let latencyMsMax: number | null = null;

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {

      // continue when this.match("BANDWIDTH").
      if (this.match("BANDWIDTH")) {
        this.expect("GTE", "Expected '>=' after bandwidth");
        bandwidthMbpsMin = this.parseNetworkAmount();
        this.expect("SEMICOLON", "Expected ';' after bandwidth requirement");

      // Otherwise, continue when this.match("LATENCY").
      } else if (this.match("LATENCY")) {
        this.expect("LTE", "Expected '<=' after latency");
        latencyMsMax = this.parseDuration();
        this.expect("SEMICOLON", "Expected ';' after latency requirement");

      // Handle any remaining cases.
      } else {
        const t = this.peek();
        throw new ParseError("Expected bandwidth or latency in requires_network", t.line, t.column);
      }
    }
    const end = this.expect("RBRACE", "Expected '}' to close requires_network");
    return {
      kind: "RequiresNetworkDecl",
      bandwidthMbpsMin,
      latencyMsMax,
      span: this.spanFrom(start, end),
    };
}

  private parseSignedNumberValue(): number {
    // Parse a numeric literal that may be prefixed with minus.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Signed numeric value.
    //
    // Options:
    // None.
    //
    // Example:
    // const lon = parseSignedNumberValue();

    let sign = 1;
    if (this.match("MINUS")) {
      sign = -1;
    }
    return sign * this.parseNumberValue();
  }

  private parseConnectivityLink(message: string): string {
    // Parse a connectivity link identifier (wifi, cellular, bluetooth, network).
    //
    // Parameters:
    // - `message` — parse error when the token is invalid
    //
    // Returns:
    // Link name string.
    //
    // Options:
    // None.
    //
    // Example:
    // const link = parseConnectivityLink("Expected link name");

    if (this.check("BLUETOOTH")) {
      this.advance();
      return "bluetooth";
    }
    if (this.check("NETWORK")) {
      this.advance();
      return "network";
    }
    return this.parseLabel(message);
  }

  private parseTriggerDomain(): string {
    // Parse the domain prefix in dot-notation triggers (gps, network, bluetooth).
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Domain identifier string.
    //
    // Options:
    // None.
    //
    // Example:
    // const domain = parseTriggerDomain();

    if (this.check("BLUETOOTH")) {
      this.advance();
      return "bluetooth";
    }
    if (this.check("NETWORK")) {
      this.advance();
      return "network";
    }
    return this.expect("IDENT", "Expected trigger domain").lexeme;
  }

  private parseRequiresConnectivity(): RequiresConnectivityDecl {
    // Parse a requires_connectivity verification block.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Parsed connectivity requirements.
    //
    // Options:
    // None.
    //
    // Example:
    // const req = parseRequiresConnectivity();

    const start = this.advance();
    this.expect("LBRACE", "Expected '{' after requires_connectivity");
    const channels: Array<[string, import("../foundations.js").ConnectivityRequirement]> = [];
    let latencyMsMax: number | null = null;
    let bandwidthMbpsMin: number | null = null;
    let packetLossPctMax: number | null = null;

    while (!this.check("RBRACE") && !this.check("EOF")) {
      if (this.match("LATENCY")) {
        this.expect("LTE", "Expected '<=' after latency");
        latencyMsMax = this.parseDuration();
        this.expect("SEMICOLON", "Expected ';' after latency");
      } else if (this.match("BANDWIDTH")) {
        this.expect("GTE", "Expected '>=' after bandwidth");
        bandwidthMbpsMin = this.parseNetworkAmount();
        this.expect("SEMICOLON", "Expected ';' after bandwidth");
      } else if (this.match("PACKET_LOSS")) {
        this.expect("LTE", "Expected '<=' after packet_loss");
        packetLossPctMax = this.parseNumberValue();
        if (this.check("PERCENT")) {
          this.advance();
        } else if (this.check("IDENT") && this.peek().lexeme === "%") {
          this.advance();
        }
        this.expect("SEMICOLON", "Expected ';' after packet_loss");
      } else if (this.check("IDENT")) {
        const key = this.advance().lexeme;
        this.expect("COLON", "Expected ':' after connectivity key");
        let level: import("../foundations.js").ConnectivityRequirement;
        if (this.check("IDENT") && this.peek().lexeme === "required") {
          this.advance();
          level = "required";
        } else if (this.check("IDENT") && this.peek().lexeme === "optional") {
          this.advance();
          level = "optional";
        } else {
          const t = this.peek();
          throw new ParseError(
            "Expected required or optional after connectivity key",
            t.line,
            t.column,
          );
        }
        this.expect("SEMICOLON", "Expected ';' after connectivity requirement");
        channels.push([key, level]);
      } else {
        const t = this.peek();
        throw new ParseError(
          "Expected connectivity channel or metric in requires_connectivity",
          t.line,
          t.column,
        );
      }
    }
    const end = this.expect("RBRACE", "Expected '}' to close requires_connectivity");
    return {
      kind: "RequiresConnectivityDecl",
      channels,
      latencyMsMax,
      bandwidthMbpsMin,
      packetLossPctMax,
      span: this.spanFrom(start, end),
    };
  }

  private parseGeofence(): GeofenceDecl {
    // Parse a WGS84 geofence zone declaration.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Parsed geofence declaration.
    //
    // Options:
    // None.
    //
    // Example:
    // const fence = parseGeofence();

    const start = this.advance();
    const name = this.expect("IDENT", "Expected geofence name").lexeme;
    this.expect("LBRACE", "Expected '{' after geofence name");
    let centerLat = 0;
    let centerLon = 0;
    let radiusM = 0;

    while (!this.check("RBRACE") && !this.check("EOF")) {
      if (this.check("IDENT") && this.peek().lexeme === "center") {
        this.advance();
        this.expect("COLON", "Expected ':' after center");
        this.expect("IDENT", "Expected geo");
        this.expect("LPAREN", "Expected '(' after geo");
        centerLat = this.parseSignedNumberValue();
        this.expect("COMMA", "Expected ',' in geo()");
        centerLon = this.parseSignedNumberValue();
        this.expect("RPAREN", "Expected ')' after geo coordinates");
        this.expect("SEMICOLON", "Expected ';' after center");
      } else if (this.match("RADIUS")) {
        this.expect("COLON", "Expected ':' after radius");
        radiusM = this.parseNumberValue();
        if (this.check("IDENT") && this.peek().lexeme === "m") {
          this.advance();
        }
        this.expect("SEMICOLON", "Expected ';' after radius");
      } else {
        const t = this.peek();
        throw new ParseError("Expected center or radius in geofence block", t.line, t.column);
      }
    }
    const end = this.expect("RBRACE", "Expected '}' to close geofence");
    return {
      kind: "GeofenceDecl",
      name,
      centerLat,
      centerLon,
      radiusM,
      span: this.spanFrom(start, end),
    };
  }

  private parseConnectivityPolicy(): ConnectivityPolicyDecl {
    // Parse a multi-link connectivity failover policy.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Parsed connectivity policy.
    //
    // Options:
    // None.
    //
    // Example:
    // const policy = parseConnectivityPolicy();

    const start = this.advance();
    const name = this.expect("IDENT", "Expected connectivity policy name").lexeme;
    this.expect("LBRACE", "Expected '{' after policy name");
    let preferred = "";
    let fallback = "";
    let emergency: string | null = null;
    let switchIfLatencyMs: number | null = null;
    let switchIfPacketLossPct: number | null = null;

    while (!this.check("RBRACE") && !this.check("EOF")) {
      if (this.check("IDENT") && this.peek().lexeme === "preferred") {
        this.advance();
        this.expect("COLON", "Expected ':' after preferred");
        preferred = this.parseConnectivityLink("Expected link name");
        this.expect("SEMICOLON", "Expected ';' after preferred");
      } else if (this.match("FALLBACK")) {
        this.expect("COLON", "Expected ':' after fallback");
        fallback = this.parseConnectivityLink("Expected link name");
        this.expect("SEMICOLON", "Expected ';' after fallback");
      } else if (this.check("IDENT") && this.peek().lexeme === "emergency") {
        this.advance();
        this.expect("COLON", "Expected ':' after emergency");
        emergency = this.parseConnectivityLink("Expected link name");
        this.expect("SEMICOLON", "Expected ';' after emergency");
      } else if (this.match("SWITCH_IF")) {
        if (this.match("LATENCY")) {
          this.expect("GT", "Expected '>' after latency");
          switchIfLatencyMs = this.parseDuration();
        } else if (this.match("PACKET_LOSS")) {
          this.expect("GT", "Expected '>' after packet_loss");
          switchIfPacketLossPct = this.parseNumberValue();
          if (this.check("PERCENT")) {
            this.advance();
          } else if (this.check("IDENT") && this.peek().lexeme === "%") {
            this.advance();
          }
        } else {
          const t = this.peek();
          throw new ParseError("Expected latency or packet_loss after switch_if", t.line, t.column);
        }
        this.expect("SEMICOLON", "Expected ';' after switch_if");
      } else {
        const t = this.peek();
        throw new ParseError("Expected policy member in connectivity_policy", t.line, t.column);
      }
    }
    const end = this.expect("RBRACE", "Expected '}' to close connectivity_policy");
    return {
      kind: "ConnectivityPolicyDecl",
      name,
      preferred,
      fallback,
      emergency,
      switchIfLatencyMs,
      switchIfPacketLossPct,
      span: this.spanFrom(start, end),
    };
  }

  private parseBleService(): BleServiceDecl {
    // Parse a BLE GATT service declaration.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Parsed BLE service block.
    //
    // Options:
    // None.
    //
    // Example:
    // const svc = parseBleService();

    const start = this.advance();
    const name = this.expect("IDENT", "Expected BLE service name").lexeme;
    this.expect("LBRACE", "Expected '{' after ble_service name");
    let uuid = "";

    while (!this.check("RBRACE") && !this.check("EOF")) {
      if (this.check("IDENT") && this.peek().lexeme === "uuid") {
        this.advance();
        this.expect("COLON", "Expected ':' after uuid");
        uuid = this.expect("STRING", "Expected UUID string").value as string;
        this.expect("SEMICOLON", "Expected ';' after uuid");
      } else {
        const t = this.peek();
        throw new ParseError("Expected uuid in ble_service block", t.line, t.column);
      }
    }
    const end = this.expect("RBRACE", "Expected '}' to close ble_service");
    return {
      kind: "BleServiceDecl",
      name,
      uuid,
      span: this.spanFrom(start, end),
    };
  }

  private parseBluetoothConfig(): BluetoothConfigDecl {
    // Parse a robot-level Bluetooth scan and pairing configuration.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Parsed bluetooth block.
    //
    // Options:
    // None.
    //
    // Example:
    // const bt = parseBluetoothConfig();

    const start = this.advance();
    this.expect("LBRACE", "Expected '{' after bluetooth");
    let scanPattern: import("../regex.js").RegexPattern | null = null;
    let pairMode: string | null = null;

    while (!this.check("RBRACE") && !this.check("EOF")) {
      if (this.check("IDENT") && this.peek().lexeme === "scan") {
        this.advance();
        this.expect("FOR", "Expected 'for' after scan");
        this.expect("IDENT", "Expected 'devices'");
        this.expect("WHERE", "Expected 'where' in bluetooth scan");
        this.expect("IDENT", "Expected 'name'");
        this.expect("MATCHES", "Expected 'matches' in bluetooth scan");
        scanPattern = this.parseRegexLiteral();
        this.expect("SEMICOLON", "Expected ';' after scan");
      } else if (this.check("IDENT") && this.peek().lexeme === "pair") {
        this.advance();
        if (this.match("TRUSTED_ONLY")) {
          pairMode = "trusted_only";
        } else {
          pairMode = this.parseLabel("Expected pair mode");
        }
        this.expect("SEMICOLON", "Expected ';' after pair");
      } else {
        const t = this.peek();
        throw new ParseError("Expected scan or pair in bluetooth block", t.line, t.column);
      }
    }
    const end = this.expect("RBRACE", "Expected '}' to close bluetooth");
    return {
      kind: "BluetoothConfigDecl",
      scanPattern,
      pairMode,
      span: this.spanFrom(start, end),
    };
  }

  private parseSimulateCompatibility(): SimulateCompatibilityDecl {
    // ParseSimulateCompatibility.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // SimulateCompatibilityDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseSimulateCompatibility();
    const start = this.advance();
    this.expect("LBRACE", "Expected '{' after simulate_compatibility");
    const faults: { faultType: string; atOffsetMs?: number; durationMs?: number; span: Span }[] = [];

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {
      this.expect("FAULT", "Expected fault declaration in simulate_compatibility");
      const faultStart = this.peek();
      const faultType = this.parseLabel("Expected fault type name");
      let atOffsetMs: number | undefined;
      let durationMs: number | undefined;
      if (this.check("AT") || (this.check("IDENT") && this.peek().lexeme === "at")) {
        this.advance();
        if (this.check("IDENT") && this.peek().lexeme.startsWith("T+")) {
          const offset = this.advance().lexeme;
          const secsStr = offset.slice(2).replace(/s$/, "");
          const secs = Number.parseFloat(secsStr);
          if (!Number.isNaN(secs)) {
            atOffsetMs = secs * 1000;
          }
        } else if (this.check("IDENT") && this.peek().lexeme === "T") {
          this.advance();
          if (this.match("PLUS")) {
            if (this.check("UNIT_LITERAL")) {
              const lex = this.advance().lexeme;
              const secsStr = lex.replace(/s$/, "");
              const secs = Number.parseFloat(secsStr);
              if (!Number.isNaN(secs)) {
                atOffsetMs = secs * 1000;
              }
            } else if (this.check("NUMBER")) {
              const secs = Number.parseFloat(this.advance().lexeme);
              if (!Number.isNaN(secs)) {
                atOffsetMs = secs * 1000;
              }
            } else {
              const num = this.parseLabel("Expected fault offset");
              const secsStr = num.replace(/s$/, "");
              const secs = Number.parseFloat(secsStr);
              if (!Number.isNaN(secs)) {
                atOffsetMs = secs * 1000;
              }
            }
          }
        }
      } else if (this.match("DURATION")) {
        durationMs = this.parseDuration();
      }
      this.expect("SEMICOLON", "Expected ';' after fault");
      faults.push({ faultType, atOffsetMs, durationMs, span: this.spanFrom(faultStart, this.previous()) });
    }
    const end = this.expect("RBRACE", "Expected '}' to close simulate_compatibility");
    return { kind: "SimulateCompatibilityDecl", faults, span: this.spanFrom(start, end) };
}

  private parseObserve(): ObserveDecl {
    // ParseObserve.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // ObserveDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseObserve();
    const start = this.expect("OBSERVE", "Expected 'observe'");
    this.expect("LBRACE", "Expected '{' after observe");
    const sensors: string[] = [];

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {
      const sensorTok = this.expect("IDENT", "Expected sensor name in observe block");
      sensors.push(sensorTok.lexeme);
      this.expect("SEMICOLON", "Expected ';' after observe sensor");
    }
    const end = this.expect("RBRACE", "Expected '}' to close observe block");
    return { kind: "ObserveDecl", sensors, span: this.spanFrom(start, end) };
  }

  private parseWorldModel(): WorldModelDecl {
    // Parse world_model block on a robot declaration.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Parsed world_model declaration.
    //
    // Options:
    // None.
    //
    // Example:
    // const result = parseWorldModel();

    const start = this.expect("IDENT", "Expected 'world_model'");
    this.expect("LBRACE", "Expected '{' after world_model");
    let enabled = true;
    while (!this.check("RBRACE") && !this.check("EOF")) {
      const flag = this.expect("IDENT", "Expected world_model flag");
      this.expect("SEMICOLON", "Expected ';' after world_model flag");
      if (flag.lexeme === "enabled") {
        enabled = true;
      } else if (flag.lexeme === "disabled") {
        enabled = false;
      } else {
        throw new ParseError(
          `Unknown world_model flag '${flag.lexeme}' (use enabled or disabled)`,
          flag.line,
          flag.column,
        );
      }
    }
    const end = this.expect("RBRACE", "Expected '}' to close world_model block");
    return { kind: "WorldModelDecl", enabled, span: this.spanFrom(start, end) };
  }

  private parseIdentity(): IdentityDecl {
    // ParseIdentity.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // IdentityDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseIdentity();
    const start = this.advance();
    const typeName = this.expect("IDENT", "Expected identity type name").lexeme;
    this.expect("LBRACE", "Expected '{' after identity type");
    const fields: [string, string][] = [];

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {
      const key = this.expect("IDENT", "Expected identity field").lexeme;
      this.expect("COLON", "Expected ':' in identity field");
      const value = this.parseConfigValueString();
      this.expect("SEMICOLON", "Expected ';' after identity field");
      fields.push([key, value]);
    }
    const end = this.expect("RBRACE", "Expected '}' to close identity");
    return {
      kind: "IdentityDecl",
      typeName,
      fields,
      span: this.spanFrom(start, end),
    };
}

  private parseAudit(): AuditDecl {
    // ParseAudit.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // AuditDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseAudit();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected audit name").lexeme;
    this.expect("LBRACE", "Expected '{' after audit name");
    const records: Expr[] = [];

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {

      // continue when lexeme equals "record").
      if (!(this.check("IDENT") && this.peek().lexeme === "record")) {
        const t = this.peek();
        throw new ParseError("Expected 'record' in audit block", t.line, t.column);
      }
      this.advance();
      records.push(this.parseExpr());
      this.expect("SEMICOLON", "Expected ';' after audit record field");
    }
    const end = this.expect("RBRACE", "Expected '}' to close audit");
    return { kind: "AuditDecl", name, records, span: this.spanFrom(start, end) };
}

  private parseProvenance(): ProvenanceDecl {
    // ParseProvenance.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // ProvenanceDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseProvenance();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected provenance name").lexeme;
    this.expect("LBRACE", "Expected '{' after provenance name");
    let hashAlgo = "sha256";
    let signedBy = "";

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {
      const field = this.expect("IDENT", "Expected provenance field").lexeme;
      this.expect("COLON", "Expected ':' in provenance field");

      // continue when field equals "hash".
      if (field === "hash") {
        hashAlgo = this.expect("IDENT", "Expected hash algorithm").lexeme;

      // Otherwise, continue when field equals "signed by".
      } else if (field === "signed_by") {
        signedBy = Parser.exprPathString(this.parseExpr());

      // Handle any remaining cases.
      } else {
        const t = this.peek();
        throw new ParseError(`Unknown provenance field '${field}'`, t.line, t.column);
      }
      this.expect("SEMICOLON", "Expected ';' after provenance field");
    }
    const end = this.expect("RBRACE", "Expected '}' to close provenance");
    return { kind: "ProvenanceDecl", name, hashAlgo, signedBy, span: this.spanFrom(start, end) };
}

  private parseSignedRecord(): SignedRecordDecl {
    // ParseSignedRecord.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // SignedRecordDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseSignedRecord();
    const start = this.advance();
    const eventName = this.expect("IDENT", "Expected signed record event name").lexeme;
    this.expect("SIGNED_BY", "Expected 'signed_by' after record event");
    const signedBy = Parser.exprPathString(this.parseExpr());
    this.expect("SEMICOLON", "Expected ';' after signed record declaration");
    return { kind: "SignedRecordDecl", eventName, signedBy, span: this.spanFrom(start, this.previous()) };
}

  private parseSecret(): SecretDecl {
    // ParseSecret.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // SecretDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseSecret();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected secret name").lexeme;
    this.expect("FROM", "Expected 'from' after secret name");
    let source: SecretDecl["source"];

    // continue when this.match("ENV").
    if (this.match("ENV")) {
      this.expect("LPAREN", "Expected '(' after env");
      const varName = this.expect("STRING", "Expected env var name").value as string;
      this.expect("RPAREN", "Expected ')' after env var");
      source = { source: "env", var: varName };

    // Otherwise, continue when this.check("IDENT") && lexeme === "file".
    } else if (this.check("IDENT") && this.peek().lexeme === "file") {
      this.advance();
      source = { source: "file", path: this.parseConfigValueString() };

    // Otherwise, continue when this.check("STRING").
    } else if (this.check("STRING")) {
      source = { source: "literal", value: this.advance().value as string };

    // Handle any remaining cases.
    } else {
      const t = this.peek();
      throw new ParseError("Expected env(...) or string literal for secret source", t.line, t.column);
    }
    this.expect("SEMICOLON", "Expected ';' after secret declaration");
    return { kind: "SecretDecl", name, source, span: this.spanFrom(start, this.previous()) };
}

  private parseTrust(): TrustDecl {
    // ParseTrust.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // TrustDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseTrust();
    const start = this.advance();
    const level = this.parseLabel("Expected trust level");
    this.expect("SEMICOLON", "Expected ';' after trust declaration");
    return { kind: "TrustDecl", level, span: this.spanFrom(start, this.previous()) };
}

  private parseDottedCapability(): string {
    // ParseDottedCapability.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseDottedCapability();
    const first = this.parseCapabilityDomain();

    let cap = first;
    while (this.match("DOT")) {
      cap = `${cap}.${this.parseCapabilitySuffix()}`;
    }
    return cap;
  }

  private parseCapabilitySuffix(): string {
    if (this.check("IDENT") && this.peek().lexeme === "scan") {
      this.advance();
      return "scan";
    }
    if (this.check("IDENT") && this.peek().lexeme === "pair") {
      this.advance();
      return "pair";
    }
    if (this.check("IDENT") && this.peek().lexeme === "connect") {
      this.advance();
      return "connect";
    }
    if (this.check("IDENT") && this.peek().lexeme === "status") {
      this.advance();
      return "status";
    }
    if (this.check("IDENT") && this.peek().lexeme === "failover") {
      this.advance();
      return "failover";
    }
    if (this.check("IDENT") && this.peek().lexeme === "read") {
      this.advance();
      return "read";
    }
    if (this.check("PUBLISH")) {
      this.advance();
      return "publish";
    }
    if (this.check("SUBSCRIBE")) {
      this.advance();
      return "subscribe";
    }
    return this.parseLabel("Expected capability suffix");
  }

  private parseCapabilityDomain(): string {
    // Parse the domain prefix in dotted capability names (network.status, gps.read).
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Capability domain string.
    //
    // Options:
    // None.
    //
    // Example:
    // const domain = parseCapabilityDomain();

    if (this.check("NETWORK")) {
      this.advance();
      return "network";
    }
    if (this.check("BLUETOOTH")) {
      this.advance();
      return "bluetooth";
    }
    return this.parseLabel("Expected capability name");
  }

  private parsePermissions(): PermissionsDecl {
    // ParsePermissions.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // PermissionsDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parsePermissions();
    const start = this.advance();
    this.expect("LBRACKET", "Expected '[' after permissions");
    const capabilities: string[] = [];

    // continue when check is falsy.
    if (!this.check("RBRACKET")) {

      // Evaluate do.
      do {
        capabilities.push(this.parseDottedCapability());
      } while (this.match("COMMA"));
    }
    this.expect("RBRACKET", "Expected ']' to close permissions");
    this.expect("SEMICOLON", "Expected ';' after permissions declaration");
    return { kind: "PermissionsDecl", capabilities, span: this.spanFrom(start, this.previous()) };
}

  private parseSecretsBlock(): SecretDecl[] {
    const start = this.advance();
    this.expect("LBRACE", "Expected '{' after secrets");
    const secrets: SecretDecl[] = [];
    while (!this.check("RBRACE") && !this.check("EOF")) {
      const name = this.expect("IDENT", "Expected secret name").lexeme;
      this.expect("FROM", "Expected 'from' after secret name");
      let source: SecretDecl["source"];
      if (this.match("ENV")) {
        const varName = this.parseConfigValueString();
        source = { source: "env", var: varName };
      } else if (this.check("IDENT") && this.peek().lexeme === "file") {
        this.advance();
        source = { source: "file", path: this.parseConfigValueString() };
      } else if (this.check("STRING")) {
        source = { source: "literal", value: this.advance().value as string };
      } else {
        const t = this.peek();
        throw new ParseError("Expected env, file, or string literal for secret source", t.line, t.column);
      }
      this.expect("SEMICOLON", "Expected ';' after secret");
      secrets.push({
        kind: "SecretDecl",
        name,
        source,
        span: this.spanFrom(start, this.previous()),
      });
    }
    this.expect("RBRACE", "Expected '}' to close secrets block");
    this.match("SEMICOLON");
    return secrets;
  }

  private parseSecureCommPolicy(): SecureCommPolicyDecl {
    const start = this.advance();
    this.expect("LBRACE", "Expected '{' after secure_comm");
    let encryption: string | null = null;
    let authentication: string | null = null;
    let integrity: string | null = null;
    while (!this.check("RBRACE") && !this.check("EOF")) {
      const key = this.parseConfigKeyToken();
      this.expect("COLON", "Expected ':' in secure_comm field");
      const value = this.parseLabel("Expected secure_comm field value");
      if (key === "encryption") encryption = value;
      else if (key === "authentication") authentication = value;
      else if (key === "integrity") integrity = value;
      else {
        const t = this.peek();
        throw new ParseError(`Unknown secure_comm field '${key}'`, t.line, t.column);
      }
      this.expect("SEMICOLON", "Expected ';' after secure_comm field");
    }
    const end = this.expect("RBRACE", "Expected '}' to close secure_comm");
    this.match("SEMICOLON");
    return {
      kind: "SecureCommPolicyDecl",
      encryption,
      authentication,
      integrity,
      span: this.spanFrom(start, end),
    };
  }

  private parseTrustBoundary(): TrustBoundaryDecl {
    const start = this.advance();
    const name = this.parseLabel("Expected trust boundary name");
    this.expect("SEMICOLON", "Expected ';' after trust_boundary declaration");
    return {
      kind: "TrustBoundaryDecl",
      name,
      span: this.spanFrom(start, this.previous()),
    };
  }

  private parseSecureBlock(): SecureBlockDecl {
    const start = this.advance();
    this.expect("LBRACE", "Expected '{' after secure");
    let signed = false;
    let minTrust: string | null = null;
    const requires: string[] = [];
    let encryption: string | null = null;
    let authentication: string | null = null;
    let integrity: string | null = null;
    const trustedSources: string[] = [];
    let rejectUntrusted = false;

    while (!this.check("RBRACE") && !this.check("EOF")) {
      const field = this.parseLabel("Expected secure field name");

      if (field === "trusted_sources") {
        this.expect("LBRACKET", "Expected '[' after trusted_sources");
        if (!this.check("RBRACKET")) {
          do {
            trustedSources.push(this.parseLabel("Expected trusted source name"));
          } while (this.match("COMMA"));
        }
        this.expect("RBRACKET", "Expected ']' after trusted_sources");
      } else if (this.check("ASSIGN")) {
        this.advance();
        if (field === "signed") {
          signed = this.match("TRUE");
          if (!signed) this.expect("FALSE", "Expected true or false for signed");
        } else if (field === "min_trust") {
          minTrust = this.parseLabel("Expected trust level");
        } else if (field === "requires") {
          this.expect("LBRACKET", "Expected '[' after requires");
          if (!this.check("RBRACKET")) {
            do {
              requires.push(this.parseDottedCapability());
            } while (this.match("COMMA"));
          }
          this.expect("RBRACKET", "Expected ']' after requires");
        } else {
          const t = this.peek();
          throw new ParseError(`Unknown secure field '${field}'`, t.line, t.column);
        }
      } else if (field === "reject_untrusted") {
        rejectUntrusted = this.match("TRUE");
        if (!rejectUntrusted) {
          this.expect("FALSE", "Expected true or false for reject_untrusted");
          rejectUntrusted = false;
        }
      } else {
        const value = this.parseLabel("Expected secure field value");
        if (field === "encryption") encryption = value;
        else if (field === "authentication") authentication = value;
        else if (field === "integrity") integrity = value;
        else if (field === "signed") {
          signed = value === "required" || value === "true";
        } else {
          const t = this.peek();
          throw new ParseError(`Unknown secure field '${field}'`, t.line, t.column);
        }
      }
      this.expect("SEMICOLON", "Expected ';' after secure field");
    }
    const end = this.expect("RBRACE", "Expected '}' to close secure block");
    return {
      signed,
      minTrust,
      requires,
      encryption,
      authentication,
      integrity,
      trustedSources,
      rejectUntrusted,
      span: this.spanFrom(start, end),
    };
  }

  private parseConfigValueString(): string {
    // ParseConfigValueString.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseConfigValueString();
    const tok = this.advance();

    // continue when type equals "STRING".
    if (tok.type === "STRING") return tok.value as string;

    // continue when type equals "IDENT".
    if (tok.type === "IDENT") return tok.lexeme;
    throw new ParseError("Expected string or identifier in config value", tok.line, tok.column);
}

  private static exprPathString(expr: Expr): string {
    // ExprPathString.
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

    // const result = exprPathString(expr);
    if (expr.kind === "IdentExpr") return expr.name;

    // continue when kind equals "MemberExpr".
    if (expr.kind === "MemberExpr") {
      return `${Parser.exprPathString(expr.object)}.${expr.property}`;
    }
    return "";
}

  private parseVerify(): VerifyDecl {
    // ParseVerify.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // VerifyDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseVerify();
    const start = this.expect("VERIFY", "Expected 'verify'");
    this.expect("LBRACE", "Expected '{' after verify");
    const rules = [];

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {
      rules.push(this.parseExpr());
      this.expect("SEMICOLON", "Expected ';' after verify rule");
    }
    const end = this.expect("RBRACE", "Expected '}' to close verify block");
    return { kind: "VerifyDecl", rules, span: this.spanFrom(start, end) };
}

  private parseTraitImpl(): TraitImplDecl {
    // ParseTraitImpl.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // TraitImplDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseTraitImpl();
    const start = this.expect("IMPL", "Expected 'impl'");
    const traitName = this.parseLabel("Expected trait name after 'impl'");
    this.expect("FOR", "Expected 'for' after trait name");
    const agentName = this.parseLabel("Expected agent name after 'for'");
    this.expect("LBRACE", "Expected '{' after trait impl header");
    const methods = [];

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {
      methods.push(this.parseTraitImplMethod());
    }
    const end = this.expect("RBRACE", "Expected '}' to close trait impl");
    return {
      kind: "TraitImplDecl",
      traitName,
      agentName,
      methods,
      span: this.spanFrom(start, end),
    };
}

  private parseTraitImplMethod(): import("../foundations.js").TraitImplMethodDecl {
    // ParseTraitImplMethod.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // import("../foundations.js").TraitImplMethodDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseTraitImplMethod();
    const start = this.advance(); // fn
    const name = this.parseLabel("Expected method name");
    this.expect("LPAREN", "Expected '(' after method name");
    const params: TraitParamDecl[] = [];

    // continue when check is falsy.
    if (!this.check("RPAREN")) {

      // Evaluate do.
      do {
        const paramStart = this.peek();
        const paramName = this.parseLabel("Expected parameter name");
        this.expect("COLON", "Expected ':' after parameter name");
        const typeName = this.expect("IDENT", "Expected parameter type");
        params.push({
          name: paramName,
          typeName: typeName.lexeme,
          span: this.spanFrom(paramStart, this.previous()),
        });
      } while (this.match("COMMA"));
    }
    this.expect("RPAREN", "Expected ')' after parameters");
    this.expect("ARROW", "Expected '->' after trait impl parameters");
    const returnType = this.expect("IDENT", "Expected return type");
    this.expect("LBRACE", "Expected '{' after trait impl method signature");
    const body = this.parseBlock();
    const end = this.expect("RBRACE", "Expected '}' to close trait impl method");
    return {
      name,
      params,
      returnType: returnType.lexeme,
      body,
      span: this.spanFrom(start, end),
    };
}

  private parseSoc(): SocDecl {
    // ParseSoc.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // SocDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseSoc();
    const start = this.advance();
    const profile = this.expect("IDENT", "Expected SoC profile name");
    this.expect("SEMICOLON", "Expected ';' after soc declaration");
    const end = this.previous();
    return { kind: "SocDecl", profile: profile.lexeme, span: this.spanFrom(start, end) };
}

  private parseHal(): HalBlock {
    // ParseHal.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HalBlock.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseHal();
    const start = this.advance();
    this.expect("LBRACE", "Expected '{' after hal");
    const members: HalMemberDecl[] = [];

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {
      members.push(this.parseHalMember());
    }
    const end = this.expect("RBRACE", "Expected '}' to close hal block");
    return { kind: "HalBlock", members, span: this.spanFrom(start, end) };
}

  private parseHalMember(): HalMemberDecl {
    // ParseHalMember.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HalMemberDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseHalMember();
    const start = this.peek();

    // continue when this.match("I2C").
    if (this.match("I2C")) {
      const name = this.parseBindingIdent("Expected I2C bus name");
      this.expect("AT", "Expected 'at' after I2C bus name");
      const addrTok = this.expect("NUMBER", "Expected I2C address");
      this.expect("SEMICOLON", "Expected ';' after I2C declaration");
      return {
        kind: "HalI2cDecl",
        name,
        address: addrTok.value as number,
        span: this.spanFrom(start, this.previous()),
      };
    }

    // continue when this.match("SPI").
    if (this.match("SPI")) {
      const name = this.parseBindingIdent("Expected SPI bus name");
      this.expect("AT", "Expected 'at' after SPI bus name");
      const busTok = this.expect("NUMBER", "Expected SPI bus number");
      let csPin: number | null = null;

      // continue when this.match("PIN").
      if (this.match("PIN")) {
        const cs = this.expect("NUMBER", "Expected CS pin number");
        csPin = cs.value as number;
      }
      this.expect("SEMICOLON", "Expected ';' after SPI declaration");
      return {
        kind: "HalSpiDecl",
        name,
        bus: busTok.value as number,
        csPin,
        span: this.spanFrom(start, this.previous()),
      };
    }

    // continue when this.match("GPIO").
    if (this.match("GPIO")) {
      const name = this.parseBindingIdent("Expected GPIO name");
      let direction: "in" | "out" = "out";

      // continue when this.match("OUT").
      if (this.match("OUT")) direction = "out";

      // Otherwise, continue when else if (this.match("IN").
      else if (this.match("IN")) direction = "in";
      this.expect("PIN", "Expected 'pin' keyword");
      const pinTok = this.expect("NUMBER", "Expected GPIO pin number");
      this.expect("SEMICOLON", "Expected ';' after GPIO declaration");
      return {
        kind: "HalGpioDecl",
        name,
        direction,
        pin: pinTok.value as number,
        span: this.spanFrom(start, this.previous()),
      };
    }

    // continue when this.match("PWM").
    if (this.match("PWM")) {
      const name = this.parseBindingIdent("Expected PWM name");
      this.expect("ON", "Expected 'on' after PWM name");
      this.expect("PIN", "Expected 'pin' after on");
      const pinTok = this.expect("NUMBER", "Expected PWM pin");
      this.expect("FREQUENCY", "Expected 'frequency' after PWM pin");
      const freq = this.parseFrequencyHz();
      this.expect("SEMICOLON", "Expected ';' after PWM declaration");
      return {
        kind: "HalPwmDecl",
        name,
        pin: pinTok.value as number,
        frequencyHz: freq,
        span: this.spanFrom(start, this.previous()),
      };
    }

    // continue when this.match("UART").
    if (this.match("UART")) {
      const name = this.parseBindingIdent("Expected UART name");
      this.expect("ON", "Expected 'on' after UART name");
      const device = this.expect("STRING", "Expected UART device path");
      this.expect("BAUD", "Expected 'baud' after UART device");
      const baudTok = this.expect("NUMBER", "Expected baud rate");
      this.expect("SEMICOLON", "Expected ';' after UART declaration");
      return {
        kind: "HalUartDecl",
        name,
        device: device.value as string,
        baud: baudTok.value as number,
        span: this.spanFrom(start, this.previous()),
      };
    }

    // continue when this.match("ADC").
    if (this.match("ADC")) {
      const name = this.parseBindingIdent("Expected ADC name");
      this.expect("ON", "Expected 'on' after ADC name");
      this.expect("IDENT", "Expected 'channel' keyword"); // channel as ident
      const chTok = this.expect("NUMBER", "Expected ADC channel number");
      this.expect("SEMICOLON", "Expected ';' after ADC declaration");
      return {
        kind: "HalAdcDecl",
        name,
        channel: chTok.value as number,
        span: this.spanFrom(start, this.previous()),
      };
    }
    const t = this.peek();
    throw new ParseError("Expected HAL member (i2c, spi, gpio, pwm, uart, adc)", t.line, t.column);
}

  private parseFrequencyHz(): number {
    // ParseFrequencyHz.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseFrequencyHz();
    const tok = this.peek();

    // continue when type equals unit === "Hz".
    if (tok.type === "UNIT_LITERAL" && tok.unit === "Hz") {
      this.advance();
      return tok.value as number;
    }

    // continue when type equals "NUMBER".
    if (tok.type === "NUMBER") {
      this.advance();

      // continue when lexeme equals "Hz".
      if (this.check("IDENT") && this.peek().lexeme === "Hz") {
        this.advance();
      }
      return tok.value as number;
    }
    throw new ParseError("Expected frequency like 50 Hz", tok.line, tok.column);
}

  private parseNode(): NodeDecl {
    // ParseNode.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // NodeDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseNode();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected node name");
    let namespace: string | null = null;

    // continue when this.match("ON").
    if (this.match("ON")) {
      const ns = this.expect("STRING", "Expected namespace string after 'on'");
      namespace = ns.value as string;
    }
    this.expect("SEMICOLON", "Expected ';' after node declaration");
    const end = this.previous();
    return { kind: "NodeDecl", name: name.lexeme, namespace, span: this.spanFrom(start, end) };
}

  private parseTopic(): TopicDecl {
    // ParseTopic.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // TopicDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseTopic();
    const start = this.advance();
    const name = this.parseLabel("Expected topic name");
    this.expect("COLON", "Expected ':' after topic name");
    const messageType = this.parseLabel("Expected message type");
    let role: TopicRole = "both";
    let topicPath: string | null = null;
    let qos: QosDecl | null = null;
    let transport: TransportKind | null = null;

    // continue when this.match("PUBLISH").
    if (this.match("PUBLISH")) {
      role = "publish";

      // continue when this.match("ON").
      if (this.match("ON")) {

        // continue when this.check("STRING").
        if (this.check("STRING")) {
          topicPath = this.advance().value as string;

        // Handle any remaining cases.
        } else {
          const ident = this.expect("IDENT", "Expected transport or topic path");
          transport = transportFromIdent(ident.lexeme);
        }
      }

    // Otherwise, continue when this.match("SUBSCRIBE").
    } else if (this.match("SUBSCRIBE")) {
      role = "subscribe";

      // continue when this.match("ON").
      if (this.match("ON")) {

        // continue when this.check("STRING").
        if (this.check("STRING")) {
          topicPath = this.advance().value as string;

        // Handle any remaining cases.
        } else {
          const ident = this.expect("IDENT", "Expected transport or topic path");
          transport = transportFromIdent(ident.lexeme);
        }
      }
    }

    // continue when this.check("LBRACE").
    if (this.check("LBRACE")) {
      qos = this.parseQosBlock();
    }

    // continue when match equals null && transport === null.
    if (this.match("ON") && topicPath === null && transport === null) {

      // continue when this.check("STRING").
      if (this.check("STRING")) {
        topicPath = this.advance().value as string;

      // Handle any remaining cases.
      } else {
        const ident = this.expect("IDENT", "Expected transport name after on");
        transport = transportFromIdent(ident.lexeme);
      }
    }
    const secure = this.check("SECURE") ? this.parseSecureBlock() : null;
    this.expect("SEMICOLON", "Expected ';' after topic declaration");
    return {
      kind: "TopicDecl",
      name,
      messageType,
      topic: topicPath,
      role,
      qos,
      transport,
      secure,
      span: this.spanFrom(start, this.previous()),
    };
}

  private parseQosBlock(): QosDecl {
    // ParseQosBlock.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // QosDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseQosBlock();
    const start = this.peek();
    this.expect("LBRACE", "Expected '{' for topic QoS block");
    let reliability: QosDecl["reliability"] = null;
    let rateHz: number | null = null;
    let deadlineMs: number | null = null;
    let history: string | null = null;

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {

      // continue when this.match("QOS").
      if (this.match("QOS")) {

        // continue when this.match("RELIABLE").
        if (this.match("RELIABLE")) {
          reliability = "reliable";

        // Otherwise, continue when this.match("BEST EFFORT").
        } else if (this.match("BEST_EFFORT")) {
          reliability = "best_effort";
        }
        this.expect("SEMICOLON", "Expected ';' after qos reliability");

      // Otherwise, continue when this.match("RATE").
      } else if (this.match("RATE")) {
        rateHz = this.parseFrequencyHz();
        this.expect("SEMICOLON", "Expected ';' after rate");

      // Otherwise, continue when this.match("DEADLINE").
      } else if (this.match("DEADLINE")) {
        deadlineMs = this.parseDuration();
        this.expect("SEMICOLON", "Expected ';' after deadline");

      // Otherwise, continue when this.match("HISTORY").
      } else if (this.match("HISTORY")) {
        history = this.expect("IDENT", "Expected history policy").lexeme;
        this.expect("SEMICOLON", "Expected ';' after history");

      // Handle any remaining cases.
      } else {
        const t = this.peek();
        throw new ParseError("Expected qos, rate, deadline, or history in topic block", t.line, t.column);
      }
    }
    const end = this.expect("RBRACE", "Expected '}' to close QoS block");
    return {
      reliability,
      rateHz,
      deadlineMs,
      history,
      span: this.spanFrom(start, end),
    };
}

  private parseService(): ServiceDecl {
    // ParseService.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // ServiceDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseService();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected service name");

    // continue when this.check("LBRACE").
    if (this.check("LBRACE")) {
      this.advance();
      let requestType: string | null = null;
      let responseType: string | null = null;

      // Repeat while !this.check("RBRACE") && !this.check("EOF").
      while (!this.check("RBRACE") && !this.check("EOF")) {

        // continue when this.match("REQUEST").
        if (this.match("REQUEST")) {
          requestType = this.expect("IDENT", "Expected request type").lexeme;
          this.expect("SEMICOLON", "Expected ';' after request type");

        // Otherwise, continue when this.match("RESPONSE").
        } else if (this.match("RESPONSE")) {
          responseType = this.expect("IDENT", "Expected response type").lexeme;
          this.expect("SEMICOLON", "Expected ';' after response type");

        // Handle any remaining cases.
        } else {
          const t = this.peek();
          throw new ParseError("Expected request or response in service block", t.line, t.column);
        }
      }
      this.expect("RBRACE", "Expected '}' to close service");
      const secure = this.check("SECURE") ? this.parseSecureBlock() : null;
      this.expect("SEMICOLON", "Expected ';' after service declaration");
      return {
        kind: "ServiceDecl",
        name: name.lexeme,
        serviceType: null,
        requestType,
        responseType,
        secure,
        span: this.spanFrom(start, this.previous()),
      };
    }
    this.expect("COLON", "Expected ':' after service name");
    const serviceType = this.expect("IDENT", "Expected service type");
    const secure = this.check("SECURE") ? this.parseSecureBlock() : null;
    this.expect("SEMICOLON", "Expected ';' after service declaration");
    return {
      kind: "ServiceDecl",
      name: name.lexeme,
      serviceType: serviceType.lexeme,
      requestType: null,
      responseType: null,
      secure,
      span: this.spanFrom(start, this.previous()),
    };
}

  private parseAction(): ActionDecl {
    // ParseAction.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // ActionDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseAction();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected action name");

    // continue when this.check("LBRACE").
    if (this.check("LBRACE")) {
      this.advance();
      let requestType: string | null = null;
      let feedbackType: string | null = null;
      let resultType: string | null = null;

      // Repeat while !this.check("RBRACE") && !this.check("EOF").
      while (!this.check("RBRACE") && !this.check("EOF")) {

        // continue when this.match("REQUEST").
        if (this.match("REQUEST")) {
          requestType = this.expect("IDENT", "Expected request type").lexeme;
          this.expect("SEMICOLON", "Expected ';' after request type");

        // Otherwise, continue when this.match("FEEDBACK").
        } else if (this.match("FEEDBACK")) {
          feedbackType = this.expect("IDENT", "Expected feedback type").lexeme;
          this.expect("SEMICOLON", "Expected ';' after feedback type");

        // Otherwise, continue when this.match("RESULT").
        } else if (this.match("RESULT")) {
          resultType = this.expect("IDENT", "Expected result type").lexeme;
          this.expect("SEMICOLON", "Expected ';' after result type");

        // Handle any remaining cases.
        } else {
          const t = this.peek();
          throw new ParseError("Expected request, feedback, or result in action block", t.line, t.column);
        }
      }
      this.expect("RBRACE", "Expected '}' to close action");
      const secure = this.check("SECURE") ? this.parseSecureBlock() : null;
      this.expect("SEMICOLON", "Expected ';' after action declaration");
      return {
        kind: "ActionDecl",
        name: name.lexeme,
        actionType: null,
        requestType,
        feedbackType,
        resultType,
        secure,
        span: this.spanFrom(start, this.previous()),
      };
    }
    this.expect("COLON", "Expected ':' after action name");
    const actionType = this.expect("IDENT", "Expected action type");
    const secure = this.check("SECURE") ? this.parseSecureBlock() : null;
    this.expect("SEMICOLON", "Expected ';' after action declaration");
    return {
      kind: "ActionDecl",
      name: name.lexeme,
      actionType: actionType.lexeme,
      requestType: null,
      feedbackType: null,
      resultType: null,
      secure,
      span: this.spanFrom(start, this.previous()),
    };
}

  private parseSensor(): SensorDecl {
    // ParseSensor.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // SensorDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseSensor();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected sensor name");
    this.expect("COLON", "Expected ':' after sensor name");
    const sensorType = this.expect("IDENT", "Expected sensor type");
    let library: string | null = null;

    // continue when this.match("FROM").
    if (this.match("FROM")) {
      const vendor = this.expect("IDENT", "Expected library vendor in from clause");
      this.expect("DOT", "Expected '.' in library path");
      const module = this.expect("IDENT", "Expected library module in from clause");
      library = `${vendor.lexeme}.${module.lexeme}`;
    }
    let binding: SensorBinding | null = null;

    // continue when this.match("ON").
    if (this.match("ON")) {

      // continue when this.check("STRING").
      if (this.check("STRING")) {
        const topicTok = this.advance();
        binding = { kind: "topic", path: topicTok.value as string };

      // Handle any remaining cases.
      } else {
        const busName = this.parseBindingIdent("Expected HAL bus name or topic string after 'on'");
        binding = { kind: "hal", busName };
      }
    }
    this.expect("SEMICOLON", "Expected ';' after sensor declaration");
    const end = this.previous();
    return {
      kind: "SensorDecl",
      name: name.lexeme,
      sensorType: sensorType.lexeme,
      library,
      binding,
      span: this.spanFrom(start, end),
    };
}

  private parseActuator(): ActuatorDecl {
    // ParseActuator.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // ActuatorDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseActuator();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected actuator name");
    this.expect("COLON", "Expected ':' after actuator name");
    const actuatorType = this.expect("IDENT", "Expected actuator type");
    this.expect("SEMICOLON", "Expected ';' after actuator declaration");
    const end = this.previous();
    return {
      kind: "ActuatorDecl",
      name: name.lexeme,
      actuatorType: actuatorType.lexeme,
      span: this.spanFrom(start, end),
    };
}

  private parseSafety(): SafetyBlock {
    // ParseSafety.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // SafetyBlock.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseSafety();
    const start = this.advance();
    this.expect("LBRACE", "Expected '{' after safety");
    const rules: SafetyRule[] = [];
    const zones: SafetyZoneDecl[] = [];

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {

      // continue when this.check("STOP IF").
      if (this.check("STOP_IF")) {
        rules.push(this.parseStopIfRule());

      // Otherwise, continue when this.check("ZONE").
      } else if (this.check("ZONE")) {
        zones.push(this.parseSafetyZone());

      // Otherwise, continue when this.check("IDENT").
      } else if (this.check("IDENT")) {
        rules.push(this.parseMaxSpeedRule());

      // Handle any remaining cases.
      } else {
        const t = this.peek();
        throw new ParseError("Expected safety rule or zone", t.line, t.column);
      }
    }
    const end = this.expect("RBRACE", "Expected '}' to close safety block");
    return { kind: "SafetyBlock", rules, zones, span: this.spanFrom(start, end) };
}

  private parseAiModelDecl(): AiModelDecl {
    // ParseAiModelDecl.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // AiModelDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseAiModelDecl();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected ai model name");
    this.expect("COLON", "Expected ':' after ai model name");
    const modelType = this.expect("IDENT", "Expected ai model type");
    this.expect("LBRACE", "Expected '{' after ai model type");
    const config = this.parseAiConfigEntries();
    const end = this.expect("RBRACE", "Expected 'GNUC to close ai model config");
    return {
      kind: "AiModelDecl",
      name: name.lexeme,
      modelType: modelType.lexeme,
      config,
      span: this.spanFrom(start, end),
    };
}

  private parseAiConfigEntries(): AiConfigEntry[] {
    // ParseAiConfigEntries.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // AiConfigEntry[].
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseAiConfigEntries();
    const entries: AiConfigEntry[] = [];

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {
      const entryStart = this.peek();
      const keyTok = this.parseConfigKeyToken();
      this.expect("COLON", "Expected ':' in ai model config");
      const value = this.parseConfigValue();
      this.expect("SEMICOLON", "Expected ';' after ai model config entry");
      entries.push({
        key: keyTok,
        value,
        span: this.spanFrom(entryStart, this.previous()),
      });
    }
    return entries;
}

  private parseConfigKeyToken(): string {
    // ParseConfigKeyToken.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseConfigKeyToken();
    if (this.check("IDENT") || this.check("PROVIDER") || this.check("MEMORY")) {
      return this.advance().lexeme;
    }
    const t = this.peek();
    throw new ParseError("Expected config key", t.line, t.column);
}

  private parseConfigValue(): string | number | boolean {
    // ParseConfigValue.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // string | number | boolean.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseConfigValue();
    if (this.match("STRING")) {
      return this.previous().value as string;
    }

    // continue when this.match("TRUE").
    if (this.match("TRUE")) {
      return true;
    }

    // continue when this.match("FALSE").
    if (this.match("FALSE")) {
      return false;
    }

    // continue when this.check("UNIT LITERAL").
    if (this.check("UNIT_LITERAL")) {
      return this.advance().value as number;
    }

    // continue when this.check("NUMBER").
    if (this.check("NUMBER")) {
      const value = this.parseNumberValue();

      // continue when this.check("IDENT").
      if (this.check("IDENT")) {
        const unit = this.peek().lexeme;

        // continue when unit equals "GB" || unit === "MB" || unit === "TB" || unit === "Gb" || unit === "Mb" || unit === "Tb".
        if (unit === "GB" || unit === "MB" || unit === "TB" || unit === "Gb" || unit === "Mb" || unit === "Tb") {
          this.advance();
        }
      }
      return value;
    }
    const t = this.peek();
    throw new ParseError("Expected config value", t.line, t.column);
}

  private parseAgent(): AgentDecl {
    // ParseAgent.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // AgentDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseAgent();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected agent name");
    this.expect("LBRACE", "Expected '{' after agent name");
    const usesAi: string[] = [];
    let memoryKind: "short_term" | "long_term" | null = null;
    const tools: string[] = [];
    const skills: string[] = [];
    const capabilities: CapabilityDecl[] = [];
    let goal = "";
    let planBody: Stmt[] = [];

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {

      // continue when this.match("USES").
      if (this.match("USES")) {
        const modelName = this.expect("IDENT", "Expected model name after uses");
        usesAi.push(modelName.lexeme);
        this.expect("SEMICOLON", "Expected ';' after uses");

      // Otherwise, continue when this.match("MEMORY").
      } else if (this.match("MEMORY")) {
        const kindTok = this.expect("IDENT", "Expected memory kind");

        // continue when lexeme differs from lexeme !== "long term".
        if (kindTok.lexeme !== "short_term" && kindTok.lexeme !== "long_term") {
          throw new ParseError("Memory kind must be short_term or long_term", kindTok.line, kindTok.column);
        }
        memoryKind = kindTok.lexeme;
        this.expect("SEMICOLON", "Expected ';' after memory");

      // Otherwise, continue when this.match("TOOLS").
      } else if (this.match("TOOLS")) {
        this.expect("LBRACKET", "Expected '[' after tools");

        // continue when check is falsy.
        if (!this.check("RBRACKET")) {

          // Evaluate do.
          do {
            const tool = this.expect("IDENT", "Expected tool name");
            tools.push(tool.lexeme);
          } while (this.match("COMMA"));
        }
        this.expect("RBRACKET", "Expected ']' after tools list");
        this.expect("SEMICOLON", "Expected ';' after tools");

      // Otherwise, continue when this.match("SKILL").
      } else if (this.match("SKILL")) {
        skills.push(this.expect("IDENT", "Expected skill name").lexeme);
        this.expect("SEMICOLON", "Expected ';' after skill");

      // Otherwise, continue when this.match("CAN").
      } else if (this.match("CAN")) {
        this.expect("LBRACKET", "Expected '[' after can");

        // continue when check is falsy.
        if (!this.check("RBRACKET")) {

          // Evaluate do.
          do {
            capabilities.push(this.parseCapability());
          } while (this.match("COMMA"));
        }
        this.expect("RBRACKET", "Expected ']' after capability list");
        this.expect("SEMICOLON", "Expected ';' after can");

      // Otherwise, continue when this.match("GOAL").
      } else if (this.match("GOAL")) {
        const goalTok = this.expect("STRING", "Expected goal string");
        goal = goalTok.value as string;
        this.expect("SEMICOLON", "Expected ';' after goal");

      // Otherwise, continue when this.match("PLAN").
      } else if (this.match("PLAN")) {
        this.expect("LBRACE", "Expected '{' after plan");
        planBody = this.parseBlock();
        this.expect("RBRACE", "Expected '}' to close plan");

      // Handle any remaining cases.
      } else {
        const t = this.peek();
        throw new ParseError("Expected agent member", t.line, t.column);
      }
    }
    const end = this.expect("RBRACE", "Expected '}' to close agent block");
    return {
      kind: "AgentDecl",
      name: name.lexeme,
      usesAi,
      memoryKind,
      tools,
      skills,
      capabilities,
      goal,
      planBody,
      span: this.spanFrom(start, end),
    };
}

  private parseSafetyZone(): SafetyZoneDecl {
    // ParseSafetyZone.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // SafetyZoneDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseSafetyZone();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected zone name");
    let shape: "circle" | "rect" = "circle";

    // continue when this.match("CIRCLE").
    if (this.match("CIRCLE")) shape = "circle";

    // Otherwise, continue when else if (this.match("RECT").
    else if (this.match("RECT")) shape = "rect";
    else throw new ParseError("Expected 'circle' or 'rect' after zone name", this.peek().line, this.peek().column);
    this.expect("AT", "Expected 'at' in zone declaration");
    this.expect("LPAREN", "Expected '(' after 'at'");
    const x = this.parseExpr();
    this.expect("COMMA", "Expected ',' between coordinates");
    const y = this.parseExpr();
    this.expect("RPAREN", "Expected ')' after coordinates");
    let radius: Expr | null = null;
    let width: Expr | null = null;
    let height: Expr | null = null;

    // continue when shape equals "circle".
    if (shape === "circle") {
      this.expect("RADIUS", "Expected 'radius' for circle zone");
      radius = this.parseExpr();

    // Handle any remaining cases.
    } else {
      this.expect("SIZE", "Expected 'size' for rect zone");
      this.expect("LPAREN", "Expected '(' after 'size'");
      width = this.parseExpr();
      this.expect("COMMA", "Expected ',' between size dimensions");
      height = this.parseExpr();
      this.expect("RPAREN", "Expected ')' after size");
    }
    this.expect("SEMICOLON", "Expected ';' after zone declaration");
    const end = this.previous();
    return {
      kind: "SafetyZoneDecl",
      name: name.lexeme,
      shape,
      x,
      y,
      radius,
      width,
      height,
      span: this.spanFrom(start, end),
    };
}

  private parseMaxSpeedRule(): SafetyRule {
    // ParseMaxSpeedRule.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // SafetyRule.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseMaxSpeedRule();
    const start = this.peek();
    const name = this.advance();
    this.expect("ASSIGN", "Expected '=' in safety rule");
    const value = this.parseExpr();
    let unit: UnitKind;

    // continue when kind equals "UnitLiteralExpr".
    if (value.kind === "UnitLiteralExpr") {
      unit = value.unit;

    // Handle any remaining cases.
    } else {
      unit = this.parseUnitSuffix();
    }
    this.expect("SEMICOLON", "Expected ';' after safety rule");
    const end = this.previous();
    return {
      kind: "MaxSpeedRule",
      name: name.lexeme,
      value,
      unit,
      span: this.spanFrom(start, end),
    };
}

  private parseStopIfRule(): SafetyRule {
    // ParseStopIfRule.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // SafetyRule.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseStopIfRule();
    const start = this.advance();
    const condition = this.parseExpr();
    this.expect("SEMICOLON", "Expected ';' after stop_if rule");
    const end = this.previous();
    return { kind: "StopIfRule", condition, span: this.spanFrom(start, end) };
}

  private parseContractClauses(): {
    // ParseContractClauses.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // .
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseContractClauses();
    requires: Expr | null;
    ensures: Expr | null;
    invariant: Expr | null;
} {
    let requires: Expr | null = null;
    let ensures: Expr | null = null;
    let invariant: Expr | null = null;
    while (!this.check("LBRACE") && !this.check("EOF")) {
      if (this.match("REQUIRES")) {
        requires = this.parseExpr();
      } else if (this.match("ENSURES")) {
        ensures = this.parseExpr();
      } else if (this.match("INVARIANT")) {
        invariant = this.parseExpr();
      } else {
        break;
      }
    }
    return { requires, ensures, invariant };
  }

  private parseBehavior(): BehaviorDecl {
    // ParseBehavior.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // BehaviorDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseBehavior();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected behavior name");
    this.expect("LPAREN", "Expected '(' after behavior name");
    this.expect("RPAREN", "Expected ')' after behavior parameters");
    const { requires, ensures, invariant } = this.parseContractClauses();
    this.expect("LBRACE", "Expected '{' after behavior signature");
    const body = this.parseBlock();
    const end = this.expect("RBRACE", "Expected '}' to close behavior");
    return {
      kind: "BehaviorDecl",
      name: name.lexeme,
      requires,
      ensures,
      invariant,
      body,
      span: this.spanFrom(start, end),
    };
}

  private parseTask(): TaskDecl {
    // ParseTask.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // TaskDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseTask();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected task name");
    let priority: TaskPriority = "normal";
    let deadlineMs: number | null = null;
    let jitterMsMax: number | null = null;
    let isolated = false;

    if (this.check("IDENT")) {
      const maybe = this.peek().lexeme;
      if (maybe === "critical" || maybe === "high" || maybe === "normal" || maybe === "low") {
        this.advance();
        priority = maybe;
      }
    }

    if (this.match("PRIORITY")) {
      const level = this.expect("IDENT", "Expected priority level");
      if (
        level.lexeme === "critical" ||
        level.lexeme === "high" ||
        level.lexeme === "normal" ||
        level.lexeme === "low"
      ) {
        priority = level.lexeme;
      } else {
        throw new ParseError(
          `Invalid priority '${level.lexeme}'; use critical, high, normal, or low`,
          level.line,
          level.column,
        );
      }
    }

    const intervalMs = this.check("EVERY") ? (this.advance(), this.parseDuration()) : 10;

    if (this.match("DEADLINE")) {
      deadlineMs = this.parseDuration();
    }
    if (this.match("JITTER")) {
      this.expect("LTE", "Expected '<=' after jitter");
      jitterMsMax = this.parseDuration();
    }
    if (this.match("ISOLATED")) {
      isolated = true;
    }

    const { requires, ensures, invariant } = this.parseContractClauses();
    this.expect("LBRACE", "Expected '{' after task signature");
    let budget: ResourceBudgetDecl | null = null;

    if (this.check("BUDGET")) {
      budget = this.parseBudget();
    }
    const body = this.parseBlock();
    const end = this.expect("RBRACE", "Expected '}' to close task");
    return {
      kind: "TaskDecl",
      name: name.lexeme,
      priority,
      intervalMs,
      deadlineMs,
      jitterMsMax,
      isolated,
      requires,
      ensures,
      invariant,
      budget,
      body,
      span: this.spanFrom(start, end),
    };
}

  private parseBudget(): ResourceBudgetDecl {
    // ParseBudget.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // ResourceBudgetDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseBudget();
    const start = this.advance();
    this.expect("LBRACE", "Expected '{' after budget");
    let batteryPctMax: number | null = null;
    let memoryMbMax: number | null = null;
    let cpuPctMax: number | null = null;
    let gpuPctMax: number | null = null;
    let networkMbpsMax: number | null = null;
    let storageMbMax: number | null = null;

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {

      // continue when this.match("BATTERY").
      if (this.match("BATTERY")) {
        this.expect("LTE", "Expected '<=' after battery in budget");
        batteryPctMax = this.parsePercentValue();
        this.expect("SEMICOLON", "Expected ';' after battery budget");

      // Otherwise, continue when this.match("MEMORY").
      } else if (this.match("MEMORY")) {
        this.expect("LTE", "Expected '<=' after memory in budget");
        memoryMbMax = this.parseStorageAmount();
        this.expect("SEMICOLON", "Expected ';' after memory budget");

      // Otherwise, continue when this.match("CPU").
      } else if (this.match("CPU")) {
        this.expect("LTE", "Expected '<=' after cpu in budget");
        cpuPctMax = this.parsePercentValue();
        this.expect("SEMICOLON", "Expected ';' after cpu budget");

      // Otherwise, continue when this.match("GPU").
      } else if (this.match("GPU")) {
        this.expect("LTE", "Expected '<=' after gpu in budget");
        gpuPctMax = this.parsePercentValue();
        this.expect("SEMICOLON", "Expected ';' after gpu budget");

      // Otherwise, continue when this.match("NETWORK").
      } else if (this.match("NETWORK")) {
        this.expect("LTE", "Expected '<=' after network in budget");
        networkMbpsMax = this.parseNetworkAmount();
        this.expect("SEMICOLON", "Expected ';' after network budget");

      // Otherwise, continue when this.match("STORAGE").
      } else if (this.match("STORAGE")) {
        this.expect("LTE", "Expected '<=' after storage in budget");
        storageMbMax = this.parseStorageAmount();
        this.expect("SEMICOLON", "Expected ';' after storage budget");

      // Handle any remaining cases.
      } else {
        const t = this.peek();
        throw new ParseError("Expected budget constraint", t.line, t.column);
      }
    }
    const end = this.expect("RBRACE", "Expected '}' to close budget");
    return {
      kind: "ResourceBudgetDecl",
      batteryPctMax,
      memoryMbMax,
      cpuPctMax,
      gpuPctMax,
      networkMbpsMax,
      storageMbMax,
      span: this.spanFrom(start, end),
    };
}

  private parsePipeline(): PipelineDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected pipeline name");
    this.expect("BUDGET", "Expected 'budget' after pipeline name");
    const budgetMs = this.parseDuration();
    this.expect("LBRACE", "Expected '{' after pipeline budget");
    const body = this.parseBlock();
    const end = this.expect("RBRACE", "Expected '}' to close pipeline");
    return {
      kind: "PipelineDecl",
      name: name.lexeme,
      budgetMs,
      body,
      span: this.spanFrom(start, end),
    };
  }

  private parseWatchdog(): WatchdogDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected watchdog name");
    let target: string | null = null;
    if (this.check("IDENT") && this.peek().lexeme !== "timeout") {
      target = this.advance().lexeme;
    }
    if (this.check("IDENT") && this.peek().lexeme === "timeout") {
      this.advance();
    } else {
      const t = this.peek();
      throw new ParseError("Expected 'timeout' in watchdog declaration", t.line, t.column);
    }
    const timeoutMs = this.parseDuration();
    this.expect("LBRACE", "Expected '{' after watchdog timeout");
    const body = this.parseBlock();
    const end = this.expect("RBRACE", "Expected '}' to close watchdog");
    return {
      kind: "WatchdogDecl",
      name: name.lexeme,
      target,
      timeoutMs,
      body,
      span: this.spanFrom(start, end),
    };
  }

  private parseMode(): ModeDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected mode name");
    this.expect("LBRACE", "Expected '{' after mode name");
    const body = this.parseBlock();
    const end = this.expect("RBRACE", "Expected '}' to close mode");
    return {
      kind: "ModeDecl",
      name: name.lexeme,
      body,
      span: this.spanFrom(start, end),
    };
  }

  private parseRetry(): RetryDecl {
    const start = this.advance();
    const attemptsTok = this.expect("NUMBER", "Expected retry attempt count");
    const attempts = typeof attemptsTok.value === "number" ? attemptsTok.value : Number(attemptsTok.value);
    if (!Number.isFinite(attempts) || attempts < 1) {
      throw new ParseError("Retry attempts must be a positive number", attemptsTok.line, attemptsTok.column);
    }
    this.expect("TIMES", "Expected 'times' after retry count");
    this.expect("BACKOFF", "Expected 'backoff' in retry declaration");
    const backoffMs = this.parseDuration();
    this.expect("LBRACE", "Expected '{' after retry backoff");
    const body = this.parseBlock();
    this.expect("RBRACE", "Expected '}' to close retry body");
    let fallback: Stmt[] = [];
    if (this.match("FALLBACK")) {
      this.expect("LBRACE", "Expected '{' after fallback");
      fallback = this.parseBlock();
      this.expect("RBRACE", "Expected '}' to close fallback");
    }
    return {
      kind: "RetryDecl",
      attempts: Math.floor(attempts),
      backoffMs,
      body,
      fallback,
      span: this.spanFrom(start, this.previous()),
    };
  }

  private parseRecover(): RecoverDecl {
    const start = this.advance();
    this.expect("FROM", "Expected 'from' after recover");
    const errorName = this.expect("IDENT", "Expected error name");
    this.expect("LBRACE", "Expected '{' after recover error");
    const body = this.parseBlock();
    const end = this.expect("RBRACE", "Expected '}' to close recover");
    return {
      kind: "RecoverDecl",
      errorName: errorName.lexeme,
      body,
      span: this.spanFrom(start, end),
    };
  }

  private parseValidateRule(): ValidateRuleDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected validate rule name");
    this.expect("LBRACE", "Expected '{' after validate name");
    this.expect("IDENT", "Expected 'value' in validate rule");
    this.expect("MATCHES", "Expected 'matches' in validate rule");
    const pattern = this.parseRegexLiteral();
    this.expect("SEMICOLON", "Expected ';' after validate pattern");
    const end = this.expect("RBRACE", "Expected '}' to close validate rule");
    return {
      kind: "ValidateRuleDecl",
      name: name.lexeme,
      pattern,
      span: this.spanFrom(start, end),
    };
  }

  private parseRegexLiteral(): import("../regex.js").RegexPattern {
    const tok = this.expect("REGEX_LITERAL", "Expected regex literal");
    return regexFromLexeme(tok.lexeme, this.spanFrom(tok, tok));
  }

  private parsePercentValue(): number {
    // ParsePercentValue.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parsePercentValue();
    const value = this.parseNumberValue();

    // continue when lexeme equals "%".
    if (this.check("IDENT") && this.peek().lexeme === "%") {
      this.advance();

    // Otherwise, continue when this.match("PERCENT").
    } else if (this.match("PERCENT")) {
      /* consumed */
    }
    return value;
}

  private parseMission(): MissionDecl {
    const start = this.advance();
    const name = this.check("IDENT") ? this.advance().lexeme : null;
    this.expect("LBRACE", "Expected '{' after mission");
    let durationHours: number | null = null;
    const steps: string[] = [];

    while (!this.check("RBRACE") && !this.check("EOF")) {
      if (this.match("DURATION")) {
        this.expect("COLON", "Expected ':' after duration");
        durationHours = this.parseDurationHours();
        this.expect("SEMICOLON", "Expected ';' after duration");
      } else {
        const step = this.parseLabel("Expected mission step name");
        this.expect("SEMICOLON", "Expected ';' after mission step");
        steps.push(step);
      }
    }
    const end = this.expect("RBRACE", "Expected '}' to close mission");
    if (durationHours === null && steps.length === 0) {
      const t = this.peek();
      throw new ParseError("mission block requires duration or at least one step", t.line, t.column);
    }
    return {
      kind: "MissionDecl",
      name,
      durationHours,
      steps,
      span: this.spanFrom(start, end),
    };
  }

  private parseFleet(): import("../foundations.js").FleetDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected fleet name").lexeme;
    this.expect("LBRACE", "Expected '{' after fleet name");
    const members: string[] = [];
    while (!this.check("RBRACE") && !this.check("EOF")) {
      const member = this.expect("IDENT", "Expected robot name in fleet block").lexeme;
      this.expect("SEMICOLON", "Expected ';' after fleet member");
      members.push(member);
    }
    const end = this.expect("RBRACE", "Expected '}' to close fleet");
    return { kind: "FleetDecl", name, members, span: this.spanFrom(start, end) };
  }

  private parseSwarm(): import("../foundations.js").SwarmDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected swarm name").lexeme;
    this.expect("LBRACE", "Expected '{' after swarm name");
    let fleetName: string | null = null;
    let policy: SwarmPolicy = "round_robin";
    while (!this.check("RBRACE") && !this.check("EOF")) {
      if (this.check("FLEET")) {
        this.advance();
        fleetName = this.expect("IDENT", "Expected fleet name after 'fleet'").lexeme;
        this.expect("SEMICOLON", "Expected ';' after fleet name");
      } else if (this.check("POLICY")) {
        this.advance();
        const policyName = this.expect("IDENT", "Expected swarm policy name").lexeme;
        this.expect("SEMICOLON", "Expected ';' after swarm policy");
        const allowed: SwarmPolicy[] = ["round_robin", "broadcast", "leader_follow"];
        const parsed = allowed.includes(policyName as SwarmPolicy)
          ? (policyName as SwarmPolicy)
          : undefined;
        if (!parsed) {
          const tok = this.previous();
          throw new ParseError(
            `Unknown swarm policy '${policyName}' (expected round_robin, broadcast, or leader_follow)`,
            tok.line,
            tok.column,
          );
        }
        policy = parsed;
      } else {
        const tok = this.peek();
        throw new ParseError("Expected fleet or policy in swarm block", tok.line, tok.column);
      }
    }
    const end = this.expect("RBRACE", "Expected '}' to close swarm");
    if (!fleetName) {
      throw new ParseError(`swarm '${name}' requires a fleet reference`, end.line, end.column);
    }
    return {
      kind: "SwarmDecl",
      name,
      fleetName,
      policy,
      span: this.spanFrom(start, end),
    };
  }

  private parseProgramSafetyZone(): import("../foundations.js").ProgramSafetyZoneDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected safety zone name").lexeme;
    this.expect("LBRACE", "Expected '{' after safety zone name");
    let maxSpeedMps: number | null = null;
    while (!this.check("RBRACE") && !this.check("EOF")) {
      if (this.check("IDENT") && this.peek().lexeme === "max_speed") {
        this.advance();
        maxSpeedMps = this.exprToMps(this.parseExpr());
        this.expect("SEMICOLON", "Expected ';' after max_speed");
      } else {
        const t = this.peek();
        throw new ParseError("Expected max_speed in safety_zone block", t.line, t.column);
      }
    }
    const end = this.expect("RBRACE", "Expected '}' to close safety_zone");
    return {
      kind: "ProgramSafetyZoneDecl",
      name,
      maxSpeedMps,
      span: this.spanFrom(start, end),
    };
  }

  private parseCertify(): import("../foundations.js").CertifyDecl {
    const start = this.advance();
    const standardName = this.expect("IDENT", "Expected certification standard after 'certify'").lexeme;
    const standard = (["ISO13849", "IEC61508", "ISO26262"] as const).find((s) => s === standardName);
    if (!standard) {
      throw new ParseError(
        `Unknown certification standard '${standardName}' (expected ISO13849, IEC61508, or ISO26262)`,
        start.line,
        start.column,
      );
    }
    let level: string | null = null;
    if (this.check("LBRACE")) {
      this.advance();
      while (!this.check("RBRACE") && !this.check("EOF")) {
        if (this.check("IDENT") && this.peek().lexeme === "level") {
          this.advance();
          level = this.expect("IDENT", "Expected certification level after 'level'").lexeme;
          this.expect("SEMICOLON", "Expected ';' after certification level");
        } else {
          const t = this.peek();
          throw new ParseError("Expected 'level <Level>;' in certify block", t.line, t.column);
        }
      }
      this.expect("RBRACE", "Expected '}' to close certify block");
    } else {
      this.expect("SEMICOLON", "Expected ';' after certification standard");
    }
    const end = this.previous();
    return { kind: "CertifyDecl", standard, level, span: this.spanFrom(start, end) };
  }

  private exprToMps(expr: Expr): number {
    if (expr.kind !== "UnitLiteralExpr") {
      throw new ParseError("max_speed requires a numeric velocity literal", expr.span.start.line, expr.span.start.column);
    }
    if (expr.unit === "m/s") return expr.value;
    if (expr.unit === "km/h") return expr.value / 3.6;
    if (expr.unit === "mph") return expr.value * 0.44704;
    throw new ParseError("max_speed requires a velocity unit (m/s, km/h, mph)", expr.span.start.line, expr.span.start.column);
  }

  private parseDurationHours(): number {
    // ParseDurationHours.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseDurationHours();
    const tok = this.peek();

    if (tok.type === "UNIT_LITERAL") {
      this.advance();
      const n = tok.value as number;
      const unit = tok.unit;
      if (unit === "h") return n;
      if (unit === "min") return n / 60;
      if (unit === "s") return n / 3600;
    }

    // continue when type equals unit === "h".
    if (tok.type === "UNIT_LITERAL" && tok.unit === "h") {
      this.advance();
      return tok.value as number;
    }
    const value = this.parseNumberValue();

    // continue when this.check("IDENT").
    if (this.check("IDENT")) {
      const unit = this.peek().lexeme;
      let hours = value;

      // continue when unit equals "h" || unit === "hr" || unit === "hrs" || unit === "hour" || unit === "hours".
      if (unit === "h" || unit === "hr" || unit === "hrs" || unit === "hour" || unit === "hours") {
        hours = value;

      // Otherwise, continue when unit equals "min" || unit === "mins" || unit === "minute" || unit === "minutes".
      } else if (unit === "min" || unit === "mins" || unit === "minute" || unit === "minutes") {
        hours = value / 60;

      // Otherwise, continue when unit equals "s" || unit === "sec" || unit === "secs".
      } else if (unit === "s" || unit === "sec" || unit === "secs") {
        hours = value / 3600;
      }

      // continue when value.
      if (
        unit === "h" ||
        unit === "hr" ||
        unit === "hrs" ||
        unit === "hour" ||
        unit === "hours" ||
        unit === "min" ||
        unit === "mins" ||
        unit === "minute" ||
        unit === "minutes" ||
        unit === "s" ||
        unit === "sec" ||
        unit === "secs"
      ) {
        this.advance();
      }
      return hours;
    }
    return value;
}

  private parseStateMachine(): StateMachineDecl {
    // ParseStateMachine.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // StateMachineDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseStateMachine();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected state machine name");
    this.expect("LBRACE", "Expected '{' after state machine name");
    const states: string[] = [];
    const transitions: TransitionDecl[] = [];

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {

      // continue when this.match("STATE").
      if (this.match("STATE")) {
        states.push(this.expect("IDENT", "Expected state name").lexeme);
        this.expect("SEMICOLON", "Expected ';' after state");

      // Otherwise, continue when this.match("TRANSITION").
      } else if (this.match("TRANSITION")) {
        const from = this.expect("IDENT", "Expected source state");
        this.expect("ARROW", "Expected '->' in transition");
        const to = this.expect("IDENT", "Expected target state");
        this.expect("SEMICOLON", "Expected ';' after transition");
        transitions.push({
          from: from.lexeme,
          to: to.lexeme,
          span: this.spanFrom(from, this.previous()),
        });

      // Handle any remaining cases.
      } else {
        const t = this.peek();
        throw new ParseError("Expected state or transition in state machine", t.line, t.column);
      }
    }
    const end = this.expect("RBRACE", "Expected '}' to close state machine");
    return {
      kind: "StateMachineDecl",
      name: name.lexeme,
      states,
      transitions,
      span: this.spanFrom(start, end),
    };
}

  private parseEvent(): EventDecl {
    // ParseEvent.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // EventDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseEvent();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected event name");
    const fields: FieldDecl[] = [];

    // continue when this.check("LBRACE").
    if (this.check("LBRACE")) {
      this.advance();

      // Repeat while !this.check("RBRACE") && !this.check("EOF").
      while (!this.check("RBRACE") && !this.check("EOF")) {
        const fieldStart = this.peek();
        const fieldName = this.expect("IDENT", "Expected event field name");
        this.expect("COLON", "Expected ':' after event field name");
        const typeName = this.expect("IDENT", "Expected event field type");
        this.expect("SEMICOLON", "Expected ';' after event field");
        fields.push({
          name: fieldName.lexeme,
          typeName: typeName.lexeme,
          span: this.spanFrom(fieldStart, this.previous()),
        });
      }
      this.expect("RBRACE", "Expected '}' to close event");
    }
    this.expect("SEMICOLON", "Expected ';' after event");
    return {
      kind: "EventDecl",
      name: name.lexeme,
      fields,
      span: this.spanFrom(start, this.previous()),
    };
}

  private parseOnTrigger(): EventHandlerDecl {
    const start = this.advance();
    let eventName: string;

    if (this.check("IDENT") && this.peek().lexeme === "log") {
      this.advance();
      this.expect("MATCHES", "Expected 'matches' after log");
      this.parseRegexLiteral();
      eventName = "log";
    } else if (this.check("MESSAGE")) {
      this.advance();
      this.expect("DOT", "Expected '.' after message in trigger");
      const fieldPart = this.expect("IDENT", "Expected field name after message.").lexeme;
      eventName = `message.${fieldPart}`;
      this.expect("MATCHES", "Expected 'matches' after message field");
      this.parseRegexLiteral();
    } else if (this.check("IDENT") && this.peek().lexeme.includes(".")) {
      eventName = this.advance().lexeme;
      this.expect("MATCHES", "Expected 'matches' after message field");
      this.parseRegexLiteral();
    } else if (this.check("HARDWARE")) {
      this.advance();
      eventName = `hardware.${this.expect("IDENT", "Expected hardware event name").lexeme}`;
    } else if (this.check("GEOFENCE")) {
      this.advance();
      const fenceName = this.expect("IDENT", "Expected geofence name").lexeme;
      let phase: string;
      if (this.match("EXITED")) {
        phase = "exited";
      } else if (this.match("ENTERED")) {
        phase = "entered";
      } else if (this.check("IDENT")) {
        phase = this.advance().lexeme.toLowerCase();
      } else {
        const t = this.peek();
        throw new ParseError("Expected entered or exited after geofence name", t.line, t.column);
      }
      eventName = `geofence:${fenceName}:${phase}`;
    } else if (
      this.check("IDENT") ||
      this.check("BLUETOOTH") ||
      this.check("NETWORK")
    ) {
      const domain = this.parseTriggerDomain();
      if (this.match("DOT")) {
        const event = this.expect("IDENT", "Expected event after '.'").lexeme.toLowerCase();
        eventName = `${domain}.${event}`;
      } else {
        eventName = domain;
      }
    } else {
      eventName = this.expect("IDENT", "Expected trigger target name").lexeme;
    }

    this.expect("LBRACE", "Expected '{' after trigger signature");
    const body = this.parseBlock();
    const end = this.expect("RBRACE", "Expected '}' to close trigger handler");
    return {
      kind: "EventHandlerDecl",
      eventName,
      body,
      span: this.spanFrom(start, end),
    };
  }

  private parseTwin(): TwinDecl {
    // ParseTwin.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // TwinDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseTwin();
    const start = this.advance();
    const name = this.expect("IDENT", "Expected twin name");
    this.expect("LBRACE", "Expected '{' after twin name");
    const mirrors: string[] = [];
    let replay = false;

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {

      // continue when this.match("MIRROR").
      if (this.match("MIRROR")) {
        mirrors.push(this.parseLabel("Expected mirror field"));
        this.expect("SEMICOLON", "Expected ';' after mirror");

      // Otherwise, continue when this.match("REPLAY").
      } else if (this.match("REPLAY")) {
        replay = this.match("TRUE");

        // continue when replay is falsy.
        if (!replay) {
          this.expect("FALSE", "Expected true or false after replay");
        }
        this.expect("SEMICOLON", "Expected ';' after replay");

      // Handle any remaining cases.
      } else {
        const t = this.peek();
        throw new ParseError("Expected mirror or replay in twin block", t.line, t.column);
      }
    }
    const end = this.expect("RBRACE", "Expected '}' to close twin");
    return {
      kind: "TwinDecl",
      name: name.lexeme,
      mirrors,
      replay,
      span: this.spanFrom(start, end),
    };
}

  private parseCapability(): CapabilityDecl {
    // ParseCapability.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // CapabilityDecl.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseCapability();
    const start = this.peek();
    let action: string;

    // continue when this.match("PLAN").
    if (this.match("PLAN")) {
      action = "plan";

    // Otherwise, continue when value.
    } else if (
      this.check("IDENT") ||
      this.check("PUBLISH") ||
      this.check("SUBSCRIBE") ||
      this.check("CALL") ||
      this.check("EXECUTE") ||
      this.check("DISCOVER")
    ) {
      action = this.advance().lexeme;

    // Handle any remaining cases.
    } else {
      const t = this.peek();
      throw new ParseError("Expected capability action", t.line, t.column);
    }
    let target: string | null = null;

    // continue when this.match("LPAREN").
    if (this.match("LPAREN")) {
      target = this.expect("IDENT", "Expected capability target").lexeme;
      this.expect("RPAREN", "Expected ')' after capability target");
    }
    return {
      action,
      target,
      span: this.spanFrom(start, this.previous()),
    };
}

  private parseLocalName(message: string): Token {
    // ParseLocalName.
    //
    // Parameters:
    // - `message` — input value
    //
    // Returns:
    // Token.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseLocalName(message);
    const lexeme = this.parseBindingIdent(message);
    return {
      type: "IDENT",
      lexeme,
      value: lexeme,
      line: this.previous().line,
      column: this.previous().column,
      offset: this.previous().offset,
    };
}

  private parseBlock(): Stmt[] {
    // ParseBlock.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Stmt[].
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseBlock();
    const stmts: Stmt[] = [];

    // Repeat while !this.check("RBRACE") && !this.check("EOF").
    while (!this.check("RBRACE") && !this.check("EOF")) {
      stmts.push(this.parseStmt());
    }
    return stmts;
}

  private parseStmt(): Stmt {
    // ParseStmt.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Stmt.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseStmt();
    const start = this.peek();

    if (this.check("IDENT") && this.peek().lexeme === "stop_all_actuators") {
      this.advance();
      this.expect("LPAREN", "Expected '(' after stop_all_actuators");
      this.expect("RPAREN", "Expected ')' after stop_all_actuators");
      this.expect("SEMICOLON", "Expected ';' after stop_all_actuators");
      return { kind: "StopAllActuatorsStmt", span: this.spanFrom(start, this.previous()) };
    }

    if (this.check("IDENT") && this.peek().lexeme === "run_pipeline") {
      this.advance();
      const name = this.expect("IDENT", "Expected pipeline name after run_pipeline");
      this.expect("SEMICOLON", "Expected ';' after run_pipeline");
      return {
        kind: "RunPipelineStmt",
        name: name.lexeme,
        span: this.spanFrom(start, this.previous()),
      };
    }

    if (this.check("IDENT") && this.peek().lexeme === "navigate") {
      this.advance();
      this.expect("LBRACE", "Expected '{' after navigate");
      let goal: import("../ast/nodes.js").Expr | null = null;
      let linear: import("../ast/nodes.js").Expr | null = null;
      let angular: import("../ast/nodes.js").Expr | null = null;
      while (!this.check("RBRACE") && !this.check("EOF")) {
        const fieldName = this.match("GOAL")
          ? "goal"
          : this.check("IDENT")
            ? this.advance().lexeme
            : null;
        if (!fieldName) {
          const t = this.peek();
          throw new ParseError("Expected field name in navigate block", t.line, t.column);
        }
        this.expect("COLON", "Expected ':' after navigate field");
        if (fieldName === "goal") {
          goal = this.parseExpr();
        } else if (fieldName === "linear") {
          linear = this.parseExpr();
        } else if (fieldName === "angular") {
          angular = this.parseExpr();
        } else {
          throw new ParseError(`Unknown navigate field '${fieldName}'`, this.previous().line, this.previous().column);
        }
        this.expect("SEMICOLON", "Expected ';' after navigate field");
      }
      const end = this.expect("RBRACE", "Expected '}' to close navigate block");
      if (!goal) {
        throw new ParseError("navigate block requires goal: ...", start.line, start.column);
      }
      return {
        kind: "NavigateStmt",
        goal,
        linear,
        angular,
        span: this.spanFrom(start, end),
      };
    }

    if (this.match("USE")) {
      const resource = this.expect("IDENT", "Expected fallback resource name");
      this.expect("SEMICOLON", "Expected ';' after use statement");
      return {
        kind: "UseFallbackStmt",
        resource: resource.lexeme,
        span: this.spanFrom(start, this.previous()),
      };
    }

    // continue when this.match("LET").
    if (this.match("LET")) {
      const name = this.parseLocalName("Expected variable name");
      const typeAnnotation = this.match("COLON") ? this.parseTypeAnnotation() : null;
      const init = this.match("ASSIGN") ? this.parseExpr() : null;

      // continue when typeAnnotation && !init is falsy.
      if (!typeAnnotation && !init) {
        const t = this.peek();
        throw new ParseError("Expected type annotation or initializer in let declaration", t.line, t.column);
      }
      this.expect("SEMICOLON", "Expected ';' after let declaration");
      const end = this.previous();
      return {
        kind: "VarDecl",
        name: name.lexeme,
        typeAnnotation,
        init,
        span: this.spanFrom(start, end),
      };
    }

    // continue when this.match("IF").
    if (this.match("IF")) {
      const condition = this.parseExpr();
      this.expect("LBRACE", "Expected '{' after if condition");
      const thenBranch = this.parseBlock();
      this.expect("RBRACE", "Expected '}' after if block");
      let elseBranch: Stmt[] | null = null;

      // continue when this.match("ELSE").
      if (this.match("ELSE")) {
        this.expect("LBRACE", "Expected '{' after else");
        elseBranch = this.parseBlock();
        this.expect("RBRACE", "Expected '}' after else block");
      }
      const end = this.previous();
      return {
        kind: "IfStmt",
        condition,
        thenBranch,
        elseBranch,
        span: this.spanFrom(start, end),
      };
    }

    // continue when this.match("LOOP").
    if (this.match("LOOP")) {
      this.expect("EVERY", "Expected 'every' after loop");
      const interval = this.parseDuration();
      this.expect("LBRACE", "Expected '{' after loop interval");
      const body = this.parseBlock();
      const end = this.expect("RBRACE", "Expected '}' to close loop");
      return {
        kind: "LoopStmt",
        intervalMs: interval,
        body,
        span: this.spanFrom(start, end),
      };
    }

    // continue when this.match("PUBLISH").
    if (this.match("PUBLISH")) {
      const topicName = this.parseSubscribeTarget();

      // continue when this.match("LPAREN").
      if (this.match("LPAREN")) {
        const value = this.parseExpr();
        this.expect("RPAREN", "Expected ')' after publish value");
        this.expect("SEMICOLON", "Expected ';' after publish statement");
        const end = this.previous();
        return {
          kind: "PublishStmt",
          topicName,
          value,
          span: this.spanFrom(start, end),
        };
      }
      this.expect("WITH", "Expected 'with' or '(' after topic name");
      const value = this.parseExpr();
      this.expect("SEMICOLON", "Expected ';' after publish statement");
      const end = this.previous();
      return {
        kind: "PublishStmt",
        topicName,
        value,
        span: this.spanFrom(start, end),
      };
    }

    // continue when this.match("SUBSCRIBE").
    if (this.match("SUBSCRIBE")) {
      const target = this.parseSubscribeTarget();
      let filter: SubscribeFilterDecl | null = null;
      if (this.match("WHERE")) {
        const field = this.parseDottedName("Expected filter field after where");
        this.expect("MATCHES", "Expected 'matches' in subscribe filter");
        const pattern = this.parseRegexLiteral();
        filter = {
          field,
          pattern,
          span: this.spanFrom(start, this.previous()),
        };
      }
      this.expect("SEMICOLON", "Expected ';' after subscribe");
      const end = this.previous();
      return { kind: "SubscribeStmt", target, filter, span: this.spanFrom(start, end) };
    }

    // continue when this.match("EXECUTE").
    if (this.match("EXECUTE")) {
      const actionName = this.expect("IDENT", "Expected action name after execute").lexeme;
      const goal = this.match("LPAREN")
        ? (() => {
            const g = this.parseExpr();
            this.expect("RPAREN", "Expected ')' after execute goal");
            return g;
          })()
        : this.parseExpr();
      this.expect("SEMICOLON", "Expected ';' after execute");
      const end = this.previous();
      return {
        kind: "ExecuteStmt",
        actionName,
        goal,
        span: this.spanFrom(start, end),
      };
    }

    // continue when this.match("DISCOVER").
    if (this.match("DISCOVER")) {
      const target = this.parseDiscoverTarget();
      const filter = this.parseDiscoverFilter();
      this.expect("SEMICOLON", "Expected ';' after discover");
      const end = this.previous();
      return { kind: "DiscoverStmt", target, filter, span: this.spanFrom(start, end) };
    }

    // continue when this.match("RECEIVE").
    if (this.match("RECEIVE")) {
      const topicName = this.parseSubscribeTarget();
      this.expect("TO", "Expected 'to' after topic in receive");
      const varName = this.expect("IDENT", "Expected variable name").lexeme;
      this.expect("SEMICOLON", "Expected ';' after receive");
      const end = this.previous();
      return { kind: "ReceiveStmt", topicName, varName, span: this.spanFrom(start, end) };
    }

    // continue when this.match("CALL").
    if (this.match("CALL")) {
      const serviceName = this.expect("IDENT", "Expected service name after call");
      this.expect("LPAREN", "Expected '(' after service name");
      this.expect("RPAREN", "Expected ')' after service arguments");
      this.expect("SEMICOLON", "Expected ';' after service call");
      const end = this.previous();
      return {
        kind: "ServiceCallStmt",
        serviceName: serviceName.lexeme,
        span: this.spanFrom(start, end),
      };
    }

    // continue when this.match("SEND GOAL").
    if (this.match("SEND_GOAL")) {
      const actionName = this.expect("IDENT", "Expected action name after send_goal");
      this.expect("WITH", "Expected 'with' after action name");
      const goal = this.parseExpr();
      this.expect("SEMICOLON", "Expected ';' after send_goal statement");
      const end = this.previous();
      return {
        kind: "ActionSendStmt",
        actionName: actionName.lexeme,
        goal,
        span: this.spanFrom(start, end),
      };
    }

    // continue when this.match("EMERGENCY STOP").
    if (this.match("EMERGENCY_STOP")) {
      this.expect("SEMICOLON", "Expected ';' after emergency_stop");
      const end = this.previous();
      return { kind: "EmergencyStopStmt", span: this.spanFrom(start, end) };
    }

    // continue when this.match("RESET EMERGENCY STOP").
    if (this.match("RESET_EMERGENCY_STOP")) {
      this.expect("SEMICOLON", "Expected ';' after reset_emergency_stop");
      const end = this.previous();
      return { kind: "ResetEmergencyStopStmt", span: this.spanFrom(start, end) };
    }

    // continue when this.match("EMIT").
    if (this.match("EMIT")) {
      const eventName = this.parseLabel("Expected event name after emit");
      this.expect("SEMICOLON", "Expected ';' after emit statement");
      const end = this.previous();
      return { kind: "EmitStmt", eventName, span: this.spanFrom(start, end) };
    }

    // continue when this.match("ENTER").
    if (this.match("ENTER")) {
      const target = this.parseLabel("Expected state or mode name after enter");
      this.expect("SEMICOLON", "Expected ';' after enter statement");
      const end = this.previous();
      if (
        target.endsWith("_mode") ||
        target === "normal" ||
        target === "degraded" ||
        target === "emergency"
      ) {
        const mode = target.endsWith("_mode") ? target.slice(0, -"_mode".length) : target;
        return { kind: "EnterModeStmt", mode, span: this.spanFrom(start, end) };
      }
      return { kind: "EnterStmt", stateName: target, span: this.spanFrom(start, end) };
    }

    // continue when this.match("REMEMBER").
    if (this.match("REMEMBER")) {
      const keyTok = this.expect("STRING", "Expected memory key string");
      const key = keyTok.value as string;
      this.expect("COMMA", "Expected ',' after memory key");
      const value = this.parseExpr();
      this.expect("SEMICOLON", "Expected ';' after remember statement");
      const end = this.previous();
      return { kind: "RememberStmt", key, value, span: this.spanFrom(start, end) };
    }

    // continue when this.match("RETURN").
    if (this.match("RETURN")) {
      const value = this.check("SEMICOLON") ? null : this.parseExpr();
      this.expect("SEMICOLON", "Expected ';' after return");
      const end = this.previous();
      return { kind: "ReturnStmt", value, span: this.spanFrom(start, end) };
    }

    // continue when this.match("SPAWN").
    if (this.match("SPAWN")) {
      const calleeName = this.parseLabel("Expected function name after spawn");
      const callee: Expr = {
        kind: "IdentExpr",
        name: calleeName,
        span: this.spanFrom(start, this.previous()),
      };
      const args: Expr[] = [];

      // continue when this.match("LPAREN").
      if (this.match("LPAREN")) {

        // continue when check is falsy.
        if (!this.check("RPAREN")) {

          // Evaluate do.
          do {
            args.push(this.parseExpr());
          } while (this.match("COMMA"));
        }
        this.expect("RPAREN", "Expected ')' after spawn arguments");
      }
      this.expect("SEMICOLON", "Expected ';' after spawn");
      const end = this.previous();
      return { kind: "SpawnStmt", callee, args, span: this.spanFrom(start, end) };
    }

    // continue when this.match("PARALLEL").
    if (this.match("PARALLEL")) {
      this.expect("LBRACE", "Expected '{' after parallel");
      const body = this.parseBlock();
      const end = this.expect("RBRACE", "Expected '}' to close parallel");
      this.expect("SEMICOLON", "Expected ';' after parallel block");
      return { kind: "ParallelStmt", body, span: this.spanFrom(start, end) };
    }

    // continue when this.match("SELECT").
    if (this.match("SELECT")) {
      this.expect("LBRACE", "Expected '{' after select");
      const arms: SelectArm[] = [];

      // Repeat while !this.check("RBRACE") && !this.check("EOF").
      while (!this.check("RBRACE") && !this.check("EOF")) {
        const armStart = this.peek();
        const recv = this.expect("IDENT", "Expected 'recv' in select arm");

        // continue when lexeme differs from "recv".
        if (recv.lexeme !== "recv") {
          throw new ParseError("Expected 'recv' in select arm", recv.line, recv.column);
        }
        this.expect("LPAREN", "Expected '(' after recv");
        const channel = this.parseExpr();
        this.expect("RPAREN", "Expected ')' after channel");
        this.expect("FAT_ARROW", "Expected '=>' in select arm");
        let body: Stmt[];

        // continue when this.check("LBRACE").
        if (this.check("LBRACE")) {
          this.advance();
          body = this.parseBlock();
          this.expect("RBRACE", "Expected '}' to close select arm");

        // Handle any remaining cases.
        } else {
          body = [this.parseStmt()];
        }
        arms.push({
          channel,
          body,
          span: this.spanFrom(armStart, this.previous()),
        });
      }
      const end = this.expect("RBRACE", "Expected '}' to close select");
      this.expect("SEMICOLON", "Expected ';' after select");
      return { kind: "SelectStmt", arms, span: this.spanFrom(start, end) };
    }
    const expr = this.parseExpr();
    this.expect("SEMICOLON", "Expected ';' after expression");
    const end = this.previous();
    return { kind: "ExprStmt", expr, span: this.spanFrom(start, end) };
}

  private parseSubscribeTarget(): string {
    // ParseSubscribeTarget.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseSubscribeTarget();
    const first = this.parseLabel("Expected subscribe target");

    // continue when this.match("DOT").
    if (this.match("DOT")) {
      const second = this.parseLabel("Expected member after '.'");
      return `${first}.${second}`;
    }
    return first;
}

  private parseDiscoverTarget(): DiscoverTarget {
    // ParseDiscoverTarget.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // DiscoverTarget.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseDiscoverTarget();
    const name = this.expect("IDENT", "Expected discover target").lexeme;

    // continue when name equals "robots".
    if (name === "robots") return "robots";

    // continue when name equals "agents".
    if (name === "agents") return "agents";

    // continue when name equals "devices".
    if (name === "devices") return "devices";
    const t = this.previous();
    throw new ParseError(
      `Expected robots, agents, or devices in discover, got '${name}'`,
      t.line,
      t.column,
    );
}

  private parseDiscoverFilter(): DiscoverFilter | null {
    // ParseDiscoverFilter.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseDiscoverFilter();
    if (!this.match("WHERE")) return null;
    this.expect("IDENT", "Expected 'capability' in discover filter");
    this.expect("INCLUDES", "Expected 'includes' in discover filter");
    const capability = this.expect("IDENT", "Expected capability name").lexeme;
    return { capability };
}

  private parseDuration(): number {
    // ParseDuration.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseDuration();
    const tok = this.peek();

    // continue when type equals unit === "ms".
    if (tok.type === "UNIT_LITERAL" && tok.unit === "ms") {
      this.advance();
      return tok.value as number;
    }

    // continue when type equals unit === "s".
    if (tok.type === "UNIT_LITERAL" && tok.unit === "s") {
      this.advance();
      return (tok.value as number) * 1000;
    }

    // continue when type equals "NUMBER".
    if (tok.type === "NUMBER") {
      this.advance();

      // continue when lexeme equals "ms".
      if (this.check("IDENT") && this.peek().lexeme === "ms") {
        this.advance();
        return tok.value as number;
      }
    }
    throw new ParseError("Expected duration like 50ms", tok.line, tok.column);
}

  private parseUnitSuffix(): UnitKind {
    // ParseUnitSuffix.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // UnitKind.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseUnitSuffix();
    const unit = this.tryParseUnitSuffix();

    // continue when unit is falsy.
    if (!unit) {
      const t = this.peek();
      throw new ParseError("Expected unit suffix", t.line, t.column);
    }
    return unit;
}

  private tryParseUnitSuffix(): UnitKind | null {
    // TryParseUnitSuffix.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = tryParseUnitSuffix();
    if (this.check("UNIT_LITERAL")) {
      const t = this.advance();
      return unitFromLexeme(t.unit!);
    }

    // continue when lexeme equals lexeme === "s".
    if (this.check("IDENT") && this.peek().lexeme === "m" && this.tokens[this.pos + 1]?.type === "SLASH" && this.tokens[this.pos + 2]?.lexeme === "s") {
      this.advance();
      this.advance();
      this.advance();
      return "m/s";
    }

    // continue when lexeme equals lexeme === "s".
    if (this.check("IDENT") && this.peek().lexeme === "rad" && this.tokens[this.pos + 1]?.type === "SLASH" && this.tokens[this.pos + 2]?.lexeme === "s") {
      this.advance();
      this.advance();
      this.advance();
      return "rad/s";
    }

    // continue when this.check("IDENT").
    if (this.check("IDENT")) {
      const lexeme = this.peek().lexeme;

      // continue when isUnitIdent(lexeme).
      if (isUnitIdent(lexeme)) {
        this.advance();
        return unitFromLexeme(lexeme as import("../lexer/index.js").UnitLexeme);
      }
    }
    return null;
}

  private parseExpr(): Expr {
    // ParseExpr.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Expr.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseExpr();
    return this.parseOr();
}

  private parseOr(): Expr {
    // ParseOr.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Expr.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseOr();
    let left = this.parseAnd();

    // Repeat while this.match("OR").
    while (this.match("OR")) {
      const opStart = this.previous();
      const right = this.parseAnd();
      left = {
        kind: "BinaryExpr",
        op: "or",
        left,
        right,
        span: this.spanFrom(
          { ...opStart, type: "OR" },
          this.previous(),
        ),
      };
    }
    return left;
}

  private parseAnd(): Expr {
    // ParseAnd.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Expr.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseAnd();
    let left = this.parseComparison();

    // Repeat while this.match("AND").
    while (this.match("AND")) {
      const opStart = this.previous();
      const right = this.parseComparison();
      left = {
        kind: "BinaryExpr",
        op: "and",
        left,
        right,
        span: this.spanFrom(opStart, this.previous()),
      };
    }
    return left;
}

  private parseComparison(): Expr {
    // ParseComparison.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Expr.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseComparison();
    let left = this.parseAdditive();

    // Repeat while value.
    while (
      this.match("LT", "LTE", "GT", "GTE", "EQ", "NEQ")
    ) {
      const opTok = this.previous();
      const op = opTok.lexeme as BinaryOp;
      const right = this.parseAdditive();
      left = {
        kind: "BinaryExpr",
        op,
        left,
        right,
        span: this.spanFrom(opTok, this.previous()),
      };
    }
    return left;
}

  private parseAdditive(): Expr {
    // ParseAdditive.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Expr.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseAdditive();
    let left = this.parseMultiplicative();

    // Repeat while this.match("PLUS", "MINUS").
    while (this.match("PLUS", "MINUS")) {
      const opTok = this.previous();
      const op = opTok.lexeme as BinaryOp;
      const right = this.parseMultiplicative();
      left = {
        kind: "BinaryExpr",
        op,
        left,
        right,
        span: this.spanFrom(opTok, this.previous()),
      };
    }
    return left;
}

  private parseMultiplicative(): Expr {
    // ParseMultiplicative.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Expr.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseMultiplicative();
    let left = this.parseUnary();

    // Repeat while this.match("STAR", "SLASH").
    while (this.match("STAR", "SLASH")) {
      const opTok = this.previous();
      const op = opTok.lexeme as BinaryOp;
      const right = this.parseUnary();
      left = {
        kind: "BinaryExpr",
        op,
        left,
        right,
        span: this.spanFrom(opTok, this.previous()),
      };
    }
    return left;
}

  private parseUnary(): Expr {
    // ParseUnary.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Expr.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parseUnary();
    if (this.match("SPAWN")) {
      const start = this.previous();
      const calleeName = this.parseLabel("Expected function name after spawn");
      const callee: Expr = {
        kind: "IdentExpr",
        name: calleeName,
        span: this.spanFrom(start, this.previous()),
      };
      const args: Expr[] = [];

      // continue when this.match("LPAREN").
      if (this.match("LPAREN")) {

        // continue when check is falsy.
        if (!this.check("RPAREN")) {

          // Evaluate do.
          do {
            args.push(this.parseExpr());
          } while (this.match("COMMA"));
        }
        this.expect("RPAREN", "Expected ')' after spawn arguments");
      }
      return { kind: "SpawnExpr", callee, args, span: this.spanFrom(start, this.previous()) };
    }

    // continue when this.match("AWAIT").
    if (this.match("AWAIT")) {
      const start = this.previous();
      const operand = this.parseUnary();
      return {
        kind: "AwaitExpr",
        operand,
        span: this.spanFrom(start, this.previous()),
      };
    }

    // continue when this.match("MINUS", "NOT").
    if (this.match("MINUS", "NOT")) {
      const opTok = this.previous();
      const op = (opTok.type === "NOT" ? "not" : "-") as import("../ast/nodes.js").UnaryOp;
      const operand = this.parseUnary();
      return {
        kind: "UnaryExpr",
        op,
        operand,
        span: this.spanFrom(opTok, this.previous()),
      };
    }
    return this.parsePostfix();
}

  private parsePostfix(): Expr {
    // ParsePostfix.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Expr.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parsePostfix();
    let expr = this.parsePrimary();

    // Repeat while true.
    while (true) {

      // continue when this.match("DOT").
      if (this.match("DOT")) {
        const prop = this.parsePropertyName();
        expr = {
          kind: "MemberExpr",
          object: expr,
          property: prop.lexeme,
          span: this.spanFrom(
            { ...prop, type: "DOT" },
            prop,
          ),
        };

      // Otherwise, continue when this.match("LPAREN").
      } else if (this.match("LPAREN")) {
        const args: Expr[] = [];
        const namedArgs: NamedArg[] = [];

        // continue when check is falsy.
        if (!this.check("RPAREN")) {

          // Evaluate do.
          do {

            // continue when this.isNamedArgStart().
            if (this.isNamedArgStart()) {
              const name = this.parseNamedArgName();
              this.advance(); // colon
              const value = this.parseExpr();
              namedArgs.push({
                name,
                value,
                span: this.spanFrom(this.previous(), this.previous()),
              });

            // Handle any remaining cases.
            } else {
              args.push(this.parseExpr());
            }
          } while (this.match("COMMA"));
        }
        const end = this.expect("RPAREN", "Expected ')' after arguments");
        expr = {
          kind: "CallExpr",
          callee: expr,
          args,
          namedArgs,
          span: this.spanFrom(
            { line: expr.span.start.line, column: expr.span.start.column, offset: 0, type: "LPAREN", lexeme: "(", value: null },
            end,
          ),
        };

      // Otherwise, continue when this.check("LT").
      } else if (this.check("LT")) {

        // continue when kind equals name).
        if (expr.kind === "IdentExpr" && /^[A-Z]/.test(expr.name)) {
          const full = this.finishGenericTypeName(expr.name);
          expr = {
            kind: "IdentExpr",
            name: full,
            span: this.spanFrom(
              { line: expr.span.start.line, column: expr.span.start.column, offset: 0, type: "IDENT", lexeme: full, value: null },
              this.previous(),
            ),
          };
          continue;
        }
        break;

      // Otherwise, continue when this.check("LBRACE").
      } else if (this.check("LBRACE")) {

        // continue when kind equals name).
        if (expr.kind === "IdentExpr" && /^[A-Z]/.test(expr.name)) {
          this.advance();
          const fields: import("../ast/nodes.js").StructFieldInit[] = [];

          // continue when check is falsy.
          if (!this.check("RBRACE")) {

            // Evaluate do.
            do {
              const fieldStart = this.peek();
              const fieldName = this.parseLabel("Expected struct field name");
              this.expect("COLON", "Expected ':' after struct field name");
              const value = this.parseExpr();
              fields.push({
                name: fieldName,
                value,
                span: this.spanFrom(fieldStart, this.previous()),
              });
            } while (this.match("COMMA"));
          }
          const end = this.expect("RBRACE", "Expected '}' to close struct literal");
          expr = {
            kind: "StructLiteralExpr",
            typeName: expr.name,
            fields,
            span: this.spanFrom(
              { line: expr.span.start.line, column: expr.span.start.column, offset: 0, type: "IDENT", lexeme: expr.name, value: null },
              end,
            ),
          };
          continue;
        }
        break;

      // Handle any remaining cases.
      } else {
        break;
      }
    }
    return expr;
}

  private parsePrimary(): Expr {
    // ParsePrimary.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Expr.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = parsePrimary();
    const start = this.peek();

    // continue when this.match("MATCH").
    if (this.match("MATCH")) {
      const scrutinee = this.parseExpr();
      this.expect("LBRACE", "Expected '{' after match scrutinee");
      const arms: MatchArm[] = [];

      // Repeat while !this.check("RBRACE") && !this.check("EOF").
      while (!this.check("RBRACE") && !this.check("EOF")) {
        const armStart = this.peek();
        const variant = this.parseLabel("Expected match arm variant");
        const bindings: string[] = [];

        // continue when this.match("LPAREN").
        if (this.match("LPAREN")) {

          // Repeat while !this.check("RPAREN") && !this.check("EOF").
          while (!this.check("RPAREN") && !this.check("EOF")) {
            bindings.push(this.parseLabel("Expected binding name"));

            // continue when match is falsy.
            if (!this.match("COMMA")) break;
          }
          this.expect("RPAREN", "Expected ')' after match bindings");
        }
        this.expect("FAT_ARROW", "Expected '=>' in match arm");
        let body: Stmt[];

        // continue when this.check("LBRACE").
        if (this.check("LBRACE")) {
          this.advance();
          body = this.parseBlock();
          this.expect("RBRACE", "Expected '}' to close match arm");

        // Handle any remaining cases.
        } else {
          body = [this.parseStmt()];
        }
        arms.push({
          variant,
          bindings: bindings.length > 0 ? bindings : undefined,
          body,
          span: this.spanFrom(armStart, this.previous()),
        });
      }
      const end = this.expect("RBRACE", "Expected '}' to close match");
      return {
        kind: "MatchExpr",
        scrutinee,
        arms,
        span: this.spanFrom(start, end),
      };
    }

    // continue when this.match("CALL").
    if (this.match("CALL")) {
      const serviceName = this.expect("IDENT", "Expected service name after call").lexeme;
      this.expect("LPAREN", "Expected '(' after service name");
      this.expect("RPAREN", "Expected ')' after service arguments");
      return {
        kind: "ServiceCallExpr",
        serviceName,
        span: this.spanFrom(start, this.previous()),
      };
    }

    // continue when this.match("EXECUTE").
    if (this.match("EXECUTE")) {
      const actionName = this.expect("IDENT", "Expected action name after execute").lexeme;
      const goal = this.match("LPAREN")
        ? (() => {
            const g = this.parseExpr();
            this.expect("RPAREN", "Expected ')' after execute goal");
            return g;
          })()
        : this.parseExpr();
      return {
        kind: "ExecuteExpr",
        actionName,
        goal,
        span: this.spanFrom(start, this.previous()),
      };
    }

    // continue when this.match("DISCOVER").
    if (this.match("DISCOVER")) {
      const target = this.parseDiscoverTarget();
      const filter = this.parseDiscoverFilter();
      return {
        kind: "DiscoverExpr",
        target,
        filter,
        span: this.spanFrom(start, this.previous()),
      };
    }

    // continue when this.match("ROBOT").
    if (this.match("ROBOT")) {
      const tok = this.previous();
      return {
        kind: "IdentExpr",
        name: "robot",
        span: this.spanFrom(start, tok),
      };
    }

    if (this.match("FLEET")) {
      const tok = this.previous();
      return { kind: "IdentExpr", name: "fleet", span: this.spanFrom(start, tok) };
    }

    if (this.match("MISSION")) {
      const tok = this.previous();
      return { kind: "IdentExpr", name: "mission", span: this.spanFrom(start, tok) };
    }

    // continue when this.match("SAFETY").
    if (this.match("SAFETY")) {
      const tok = this.previous();
      return {
        kind: "IdentExpr",
        name: "safety",
        span: this.spanFrom(start, tok),
      };
    }

    // continue when this.match("TRUE").
    if (this.match("TRUE")) {
      return {
        kind: "LiteralExpr",
        value: true,
        span: this.spanFrom(start, this.previous()),
      };
    }

    // continue when this.match("FALSE").
    if (this.match("FALSE")) {
      return {
        kind: "LiteralExpr",
        value: false,
        span: this.spanFrom(start, this.previous()),
      };
    }

    if (this.match("REGEX_LITERAL")) {
      const tok = this.previous();
      return {
        kind: "LiteralExpr",
        value: regexFromLexeme(tok.lexeme, this.spanFrom(tok, tok)),
        span: this.spanFrom(start, tok),
      };
    }

    // continue when this.match("NUMBER").
    if (this.match("NUMBER")) {
      const tok = this.previous();
      const unit = this.tryParseUnitSuffix();

      // continue when unit.
      if (unit) {
        return {
          kind: "UnitLiteralExpr",
          value: tok.value as number,
          unit,
          span: this.spanFrom(start, this.previous()),
        };
      }
      return {
        kind: "LiteralExpr",
        value: tok.value as number,
        span: this.spanFrom(start, tok),
      };
    }

    // continue when this.match("UNIT LITERAL").
    if (this.match("UNIT_LITERAL")) {
      const tok = this.previous();
      return {
        kind: "UnitLiteralExpr",
        value: tok.value as number,
        unit: unitFromLexeme(tok.unit!),
        span: this.spanFrom(start, tok),
      };
    }

    // continue when this.match("STRING").
    if (this.match("STRING")) {
      return {
        kind: "LiteralExpr",
        value: this.previous().value as string,
        span: this.spanFrom(start, this.previous()),
      };
    }

    // continue when value.
    if (
      this.match(
        "IDENT",
        "ACTION",
        "STATE",
        "PLAN",
        "GOAL",
        "SKILL",
        "EVENT",
        "TASK",
        "TWIN",
        "MATCH",
        "MISSION",
        "DURATION",
        "NETWORK",
        "BANDWIDTH",
        "LATENCY",
        "TIMING",
        "BUDGET",
        "FAULT",
        "EXECUTE",
        "DISCOVER",
        "FROM",
        "TO",
        "SUBSCRIBE",
        "MATCHES",
        "RECEIVE",
        "MESSAGE",
        "RESPONSE",
        "FEEDBACK",
        "RESULT",
        "REQUEST",
        "DEVICE",
        "BUS",
      )
    ) {
      const tok = this.previous();
      return {
        kind: "IdentExpr",
        name: tok.lexeme,
        span: this.spanFrom(start, tok),
      };
    }

    // continue when this.match("LPAREN").
    if (this.match("LPAREN")) {
      const expr = this.parseExpr();
      const end = this.expect("RPAREN", "Expected ')' after expression");
      return { ...expr, span: this.spanFrom(start, end) };
    }
    const t = this.peek();
    throw new ParseError("Expected expression", t.line, t.column);
}

  private parsePropertyName(): Token {    // Compute lexeme for the following logic.
    const lexeme = this.parseLabel("Expected property name after '.'");
    const end = this.previous();
    return { type: "IDENT", lexeme, value: null, line: end.line, column: end.column, offset: end.offset };
}

  private isNamedArgStart(): boolean {    // Compute next for the following logic.
    const next = this.tokens[this.pos + 1];

    // continue when type differs from "COLON".
    if (next?.type !== "COLON") return false;
    return this.check("IDENT") || this.check("FROM") || this.check("GOAL") || this.check("TO");
}

  private parseNamedArgName(): string {    // continue when this.match("FROM").
    if (this.match("FROM")) return "from";

    // continue when this.match("TO").
    if (this.match("TO")) return "to";

    // continue when this.match("GOAL").
    if (this.match("GOAL")) return "goal";
    return this.advance().lexeme;
}
}

export { parse as parseTokens };

function isUnitIdent(lexeme: string): boolean {
  // IsUnitIdent.
  //
  // Parameters:
  // - `lexeme` — input value
  //
  // Returns:
  // `true` or `false`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = isUnitIdent(lexeme);
  return ["m", "s", "ms", "rad", "deg", "Hz"].includes(lexeme);
}
