import type { Token } from "../lexer/index.js";
import { unitFromLexeme } from "../lexer/index.js";
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
  EventDecl,
  EventHandlerDecl,
  FieldDecl,
  MatchArm,
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
  IdentityDecl,
  AuditDecl,
  ProvenanceDecl,
  SignedRecordDecl,
  SecretDecl,
  TrustDecl,
  PermissionsDecl,
  SecureBlockDecl,
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
  const parser = new Parser(tokens);
  return parser.parseProgram();
}

class Parser {
  private pos = 0;

  constructor(private tokens: Token[]) {}

  private peek(): Token {
    return this.tokens[this.pos];
  }

  private previous(): Token {
    return this.tokens[this.pos - 1];
  }

  private advance(): Token {
    if (this.peek().type !== "EOF") this.pos++;
    return this.previous();
  }

  private check(type: Token["type"]): boolean {
    return this.peek().type === type;
  }

  private match(...types: Token["type"][]): boolean {
    for (const t of types) {
      if (this.check(t)) {
        this.advance();
        return true;
      }
    }
    return false;
  }

  private expect(type: Token["type"], message: string): Token {
    if (this.check(type)) return this.advance();
    const t = this.peek();
    throw new ParseError(message, t.line, t.column);
  }

  private spanFrom(start: Token, end: Token): Span {
    return {
      start: { line: start.line, column: start.column, offset: start.offset },
      end: { line: end.line, column: end.column, offset: end.offset },
    };
  }

  private parseLabel(message: string): string {
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
      "SUBSCRIBE",
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
    if (labelTypes.includes(this.peek().type)) {
      return this.advance().lexeme;
    }
    const t = this.peek();
    throw new ParseError(message, t.line, t.column);
  }

  private parseBindingIdent(message: string): string {
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
      "SUBSCRIBE",
      "RECEIVE",
      "MESSAGE",
      "RESPONSE",
      "FEEDBACK",
      "RESULT",
      "REQUEST",
      "DEVICE",
      "BUS",
    ];
    if (bindingTypes.includes(this.peek().type)) {
      return this.advance().lexeme;
    }
    const t = this.peek();
    throw new ParseError(message, t.line, t.column);
  }

  parseProgram(): Program {
    const start = this.peek();
    let moduleName: string | null = null;

    if (this.check("MODULE")) {
      this.advance();
      moduleName = this.parseLabel("Expected module name after 'module'");
      this.expect("SEMICOLON", "Expected ';' after module declaration");
    }

    const imports: ImportDecl[] = [];
    const structs: StructDecl[] = [];
    const enums: EnumDecl[] = [];
    const traits: TraitDecl[] = [];
    const messages: MessageDecl[] = [];
    const robots: RobotDecl[] = [];

    while (this.check("IMPORT")) {
      imports.push(this.parseImport());
    }

    while (!this.check("EOF")) {
      if (this.check("STRUCT")) {
        structs.push(this.parseStruct());
      } else if (this.check("ENUM")) {
        enums.push(this.parseEnum());
      } else if (this.check("TRAIT")) {
        traits.push(this.parseTrait());
      } else if (this.check("MESSAGE")) {
        messages.push(this.parseMessage());
      } else if (this.check("ROBOT")) {
        robots.push(this.parseRobot());
      } else {
        const t = this.peek();
        throw new ParseError("Expected struct, enum, trait, message, or robot declaration", t.line, t.column);
      }
    }

    const end = this.previous();
    return {
      kind: "Program",
      moduleName,
      imports,
      structs,
      enums,
      traits,
      messages,
      robots,
      span: this.spanFrom(start, end),
    };
  }

  private parseImport(): ImportDecl {
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
    if (this.check("IDENT")) {
      return this.advance().lexeme;
    }
    if (this.check("EOF") || this.check("DOT") || this.check("SEMICOLON")) {
      const t = this.peek();
      throw new ParseError(message, t.line, t.column);
    }
    return this.advance().lexeme;
  }

  private parseTypeNamePart(message: string): string {
    if (this.check("IDENT")) {
      return this.advance().lexeme;
    }
    return this.parseLabel(message);
  }

  private parseTypeAnnotation(): SpandaType {
    const parts = [this.parseTypeNamePart("Expected type name")];
    while (this.match("DOT")) {
      parts.push(this.parseTypeNamePart("Expected type name after '.'"));
    }
    const qualified = parts.join(".");
    if (this.match("LT")) {
      const args: SpandaType[] = [];
      if (!this.check("GT")) {
        do {
          args.push(this.parseTypeAnnotation());
        } while (this.match("COMMA"));
      }
      this.expect("GT", "Expected '>' to close generic type");
      const base = parts[parts.length - 1] ?? qualified;
      try {
        return resolveGenericType(base, args);
      } catch (e) {
        const t = this.previous();
        throw new ParseError((e as Error).message, t.line, t.column);
      }
    }
    try {
      return resolveTypeName(qualified);
    } catch (e) {
      const t = this.peek();
      throw new ParseError((e as Error).message, t.line, t.column);
    }
  }

  private parseStruct(): StructDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected struct name");
    this.expect("LBRACE", "Expected '{' after struct name");
    const fields: FieldDecl[] = [];
    while (!this.check("RBRACE") && !this.check("EOF")) {
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
    const end = this.expect("RBRACE", "Expected '}' to close struct");
    return {
      kind: "StructDecl",
      name: name.lexeme,
      fields,
      span: this.spanFrom(start, end),
    };
  }

  private parseEnum(): EnumDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected enum name");
    this.expect("LBRACE", "Expected '{' after enum name");
    const variants: string[] = [];
    while (!this.check("RBRACE") && !this.check("EOF")) {
      variants.push(this.expect("IDENT", "Expected enum variant").lexeme);
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
    const start = this.advance();
    const name = this.expect("IDENT", "Expected trait name");
    this.expect("LBRACE", "Expected '{' after trait name");
    const methods: TraitMethodDecl[] = [];
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
    const start = this.advance(); // fn
    const name = this.parseLabel("Expected method name after fn");
    this.expect("LPAREN", "Expected '(' after method name");
    const params: TraitParamDecl[] = [];
    if (!this.check("RPAREN")) {
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
    const stateMachines: StateMachineDecl[] = [];
    const events: EventDecl[] = [];
    const eventHandlers: EventHandlerDecl[] = [];
    let twin: TwinDecl | null = null;
    let verify: VerifyDecl | null = null;
    let observe: ObserveDecl | null = null;
    let identity: IdentityDecl | null = null;
    let audit: AuditDecl | null = null;
    let provenance: ProvenanceDecl | null = null;
    const signedRecords: SignedRecordDecl[] = [];
    const secrets: SecretDecl[] = [];
    let trust: TrustDecl | null = null;
    let permissions: PermissionsDecl | null = null;
    const traitImpls: TraitImplDecl[] = [];
    const buses: BusDecl[] = [];
    const peerRobots: PeerRobotDecl[] = [];
    const devices: DeviceDecl[] = [];
    const agentChannels: AgentChannelDecl[] = [];
    const twinSync = null;

    while (!this.check("RBRACE") && !this.check("EOF")) {
      if (this.check("SOC")) {
        soc = this.parseSoc();
      } else if (this.check("HAL")) {
        hal = this.parseHal();
      } else if (this.check("NODE")) {
        nodes.push(this.parseNode());
      } else if (this.check("TOPIC")) {
        topics.push(this.parseTopic());
      } else if (this.check("SERVICE")) {
        services.push(this.parseService());
      } else if (this.check("ACTION")) {
        actions.push(this.parseAction());
      } else if (this.check("SENSOR")) {
        sensors.push(this.parseSensor());
      } else if (this.check("ACTUATOR")) {
        actuators.push(this.parseActuator());
      } else if (this.check("SAFETY")) {
        safety = this.parseSafety();
      } else if (this.check("AI_MODEL")) {
        ai_models.push(this.parseAiModelDecl());
      } else if (this.check("IDENT") && this.isAgentChannel()) {
        agentChannels.push(this.parseAgentChannel());
      } else if (this.check("AGENT")) {
        if (this.isAgentShorthand()) {
          this.parseAgentShorthand(agents);
        } else {
          agents.push(this.parseAgent());
        }
      } else if (this.check("BEHAVIOR")) {
        behaviors.push(this.parseBehavior());
      } else if (this.check("TASK")) {
        tasks.push(this.parseTask());
      } else if (this.check("STATE_MACHINE")) {
        stateMachines.push(this.parseStateMachine());
      } else if (this.check("EVENT")) {
        events.push(this.parseEvent());
      } else if (this.check("ON")) {
        eventHandlers.push(this.parseEventHandler());
      } else if (this.check("TWIN")) {
        twin = this.parseTwin();
      } else if (this.check("VERIFY")) {
        verify = this.parseVerify();
      } else if (this.check("OBSERVE")) {
        observe = this.parseObserve();
      } else if (this.isRobotMemberKeyword("identity")) {
        identity = this.parseIdentity();
      } else if (this.isRobotMemberKeyword("audit")) {
        audit = this.parseAudit();
      } else if (this.isRobotMemberKeyword("provenance")) {
        provenance = this.parseProvenance();
      } else if (this.isRobotMemberKeyword("record")) {
        signedRecords.push(this.parseSignedRecord());
      } else if (this.check("SECRET")) {
        secrets.push(this.parseSecret());
      } else if (this.check("TRUST")) {
        trust = this.parseTrust();
      } else if (this.check("PERMISSIONS")) {
        permissions = this.parsePermissions();
      } else if (this.check("IMPL")) {
        traitImpls.push(this.parseTraitImpl());
      } else if (this.check("BUS")) {
        buses.push(this.parseBus());
      } else if (this.check("ROBOT")) {
        peerRobots.push(this.parsePeerRobot());
      } else if (this.check("DEVICE")) {
        devices.push(this.parseDevice());
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
      stateMachines,
      events,
      eventHandlers,
      twin,
      verify,
      observe,
      identity,
      audit,
      provenance,
      signedRecords,
      secrets,
      trust,
      permissions,
      traitImpls,
      buses,
      peerRobots,
      devices,
      agentChannels,
      twinSync,
      span: this.spanFrom(start, end),
    };
  }

  private isRobotMemberKeyword(kw: string): boolean {
    return this.check("IDENT") && this.peek().lexeme === kw;
  }

  private isAgentShorthand(): boolean {
    let idx = this.pos + 1;
    if (idx >= this.tokens.length) return false;
    if (this.tokens[idx]?.type !== "IDENT") return false;
    idx += 1;
    return idx < this.tokens.length && this.tokens[idx]?.type === "SEMICOLON";
  }

  private isAgentChannel(): boolean {
    const idx = this.pos;
    return (
      idx + 2 < this.tokens.length &&
      this.tokens[idx]?.type === "IDENT" &&
      this.tokens[idx + 1]?.type === "ARROW" &&
      this.tokens[idx + 2]?.type === "IDENT"
    );
  }

  private parseAgentShorthand(agents: AgentDecl[]): void {
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
    const transportName = this.expect("IDENT", "Expected bus transport name");
    this.expect("SEMICOLON", "Expected ';' after bus declaration");
    const transport = transportFromIdent(transportName.lexeme) ?? "local";
    return {
      kind: "BusDecl",
      name: transportName.lexeme,
      transport,
      span: this.spanFrom(start, this.previous()),
    };
  }

  private parsePeerRobot(): PeerRobotDecl {
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
    const start = this.advance();
    const name = this.expect("IDENT", "Expected message name");
    this.expect("LBRACE", "Expected '{' after message name");
    const fields: FieldDecl[] = [];
    let version: number | null = null;
    while (!this.check("RBRACE") && !this.check("EOF")) {
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
    const tok = this.expect("NUMBER", "Expected number");
    return tok.value as number;
  }

  private parseObserve(): ObserveDecl {
    const start = this.expect("OBSERVE", "Expected 'observe'");
    this.expect("LBRACE", "Expected '{' after observe");
    const sensors: string[] = [];
    while (!this.check("RBRACE") && !this.check("EOF")) {
      const sensorTok = this.expect("IDENT", "Expected sensor name in observe block");
      sensors.push(sensorTok.lexeme);
      this.expect("SEMICOLON", "Expected ';' after observe sensor");
    }
    const end = this.expect("RBRACE", "Expected '}' to close observe block");
    return { kind: "ObserveDecl", sensors, span: this.spanFrom(start, end) };
  }

  private parseIdentity(): IdentityDecl {
    const start = this.advance();
    const typeName = this.expect("IDENT", "Expected identity type name").lexeme;
    this.expect("LBRACE", "Expected '{' after identity type");
    const fields: [string, string][] = [];
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
    const start = this.advance();
    const name = this.expect("IDENT", "Expected audit name").lexeme;
    this.expect("LBRACE", "Expected '{' after audit name");
    const records: Expr[] = [];
    while (!this.check("RBRACE") && !this.check("EOF")) {
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
    const start = this.advance();
    const name = this.expect("IDENT", "Expected provenance name").lexeme;
    this.expect("LBRACE", "Expected '{' after provenance name");
    let hashAlgo = "sha256";
    let signedBy = "";
    while (!this.check("RBRACE") && !this.check("EOF")) {
      const field = this.expect("IDENT", "Expected provenance field").lexeme;
      this.expect("COLON", "Expected ':' in provenance field");
      if (field === "hash") {
        hashAlgo = this.expect("IDENT", "Expected hash algorithm").lexeme;
      } else if (field === "signed_by") {
        signedBy = Parser.exprPathString(this.parseExpr());
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
    const start = this.advance();
    const eventName = this.expect("IDENT", "Expected signed record event name").lexeme;
    this.expect("SIGNED_BY", "Expected 'signed_by' after record event");
    const signedBy = Parser.exprPathString(this.parseExpr());
    this.expect("SEMICOLON", "Expected ';' after signed record declaration");
    return { kind: "SignedRecordDecl", eventName, signedBy, span: this.spanFrom(start, this.previous()) };
  }

  private parseSecret(): SecretDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected secret name").lexeme;
    this.expect("FROM", "Expected 'from' after secret name");
    let source: SecretDecl["source"];
    if (this.match("ENV")) {
      this.expect("LPAREN", "Expected '(' after env");
      const varName = this.expect("STRING", "Expected env var name").value as string;
      this.expect("RPAREN", "Expected ')' after env var");
      source = { source: "env", var: varName };
    } else if (this.check("STRING")) {
      source = { source: "literal", value: this.advance().value as string };
    } else {
      const t = this.peek();
      throw new ParseError("Expected env(...) or string literal for secret source", t.line, t.column);
    }
    this.expect("SEMICOLON", "Expected ';' after secret declaration");
    return { kind: "SecretDecl", name, source, span: this.spanFrom(start, this.previous()) };
  }

  private parseTrust(): TrustDecl {
    const start = this.advance();
    const level = this.parseLabel("Expected trust level");
    this.expect("SEMICOLON", "Expected ';' after trust declaration");
    return { kind: "TrustDecl", level, span: this.spanFrom(start, this.previous()) };
  }

  private parseDottedCapability(): string {
    const first = this.parseLabel("Expected capability name");
    if (this.match("DOT")) {
      const second = this.parseLabel("Expected capability suffix");
      return `${first}.${second}`;
    }
    return first;
  }

  private parsePermissions(): PermissionsDecl {
    const start = this.advance();
    this.expect("LBRACKET", "Expected '[' after permissions");
    const capabilities: string[] = [];
    if (!this.check("RBRACKET")) {
      do {
        capabilities.push(this.parseDottedCapability());
      } while (this.match("COMMA"));
    }
    this.expect("RBRACKET", "Expected ']' to close permissions");
    this.expect("SEMICOLON", "Expected ';' after permissions declaration");
    return { kind: "PermissionsDecl", capabilities, span: this.spanFrom(start, this.previous()) };
  }

  private parseSecureBlock(): SecureBlockDecl {
    const start = this.advance();
    this.expect("LBRACE", "Expected '{' after secure");
    let signed = false;
    let minTrust: string | null = null;
    const requires: string[] = [];
    while (!this.check("RBRACE") && !this.check("EOF")) {
      const field = this.parseLabel("Expected secure field name");
      this.expect("ASSIGN", "Expected '=' in secure field");
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
      this.expect("SEMICOLON", "Expected ';' after secure field");
    }
    const end = this.expect("RBRACE", "Expected '}' to close secure block");
    return { signed, minTrust, requires, span: this.spanFrom(start, end) };
  }

  private parseConfigValueString(): string {
    const tok = this.advance();
    if (tok.type === "STRING") return tok.value as string;
    if (tok.type === "IDENT") return tok.lexeme;
    throw new ParseError("Expected string or identifier in config value", tok.line, tok.column);
  }

  private static exprPathString(expr: Expr): string {
    if (expr.kind === "IdentExpr") return expr.name;
    if (expr.kind === "MemberExpr") {
      return `${Parser.exprPathString(expr.object)}.${expr.property}`;
    }
    return "";
  }

  private parseVerify(): VerifyDecl {
    const start = this.expect("VERIFY", "Expected 'verify'");
    this.expect("LBRACE", "Expected '{' after verify");
    const rules = [];
    while (!this.check("RBRACE") && !this.check("EOF")) {
      rules.push(this.parseExpr());
      this.expect("SEMICOLON", "Expected ';' after verify rule");
    }
    const end = this.expect("RBRACE", "Expected '}' to close verify block");
    return { kind: "VerifyDecl", rules, span: this.spanFrom(start, end) };
  }

  private parseTraitImpl(): TraitImplDecl {
    const start = this.expect("IMPL", "Expected 'impl'");
    const traitName = this.parseLabel("Expected trait name after 'impl'");
    this.expect("FOR", "Expected 'for' after trait name");
    const agentName = this.parseLabel("Expected agent name after 'for'");
    this.expect("LBRACE", "Expected '{' after trait impl header");
    const methods = [];
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
    const start = this.advance(); // fn
    const name = this.parseLabel("Expected method name");
    this.expect("LPAREN", "Expected '(' after method name");
    const params: TraitParamDecl[] = [];
    if (!this.check("RPAREN")) {
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
    const start = this.advance();
    const profile = this.expect("IDENT", "Expected SoC profile name");
    this.expect("SEMICOLON", "Expected ';' after soc declaration");
    const end = this.previous();
    return { kind: "SocDecl", profile: profile.lexeme, span: this.spanFrom(start, end) };
  }

  private parseHal(): HalBlock {
    const start = this.advance();
    this.expect("LBRACE", "Expected '{' after hal");
    const members: HalMemberDecl[] = [];
    while (!this.check("RBRACE") && !this.check("EOF")) {
      members.push(this.parseHalMember());
    }
    const end = this.expect("RBRACE", "Expected '}' to close hal block");
    return { kind: "HalBlock", members, span: this.spanFrom(start, end) };
  }

  private parseHalMember(): HalMemberDecl {
    const start = this.peek();

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

    if (this.match("SPI")) {
      const name = this.parseBindingIdent("Expected SPI bus name");
      this.expect("AT", "Expected 'at' after SPI bus name");
      const busTok = this.expect("NUMBER", "Expected SPI bus number");
      let csPin: number | null = null;
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

    if (this.match("GPIO")) {
      const name = this.parseBindingIdent("Expected GPIO name");
      let direction: "in" | "out" = "out";
      if (this.match("OUT")) direction = "out";
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
    const tok = this.peek();
    if (tok.type === "UNIT_LITERAL" && tok.unit === "Hz") {
      this.advance();
      return tok.value as number;
    }
    if (tok.type === "NUMBER") {
      this.advance();
      if (this.check("IDENT") && this.peek().lexeme === "Hz") {
        this.advance();
      }
      return tok.value as number;
    }
    throw new ParseError("Expected frequency like 50 Hz", tok.line, tok.column);
  }

  private parseNode(): NodeDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected node name");
    let namespace: string | null = null;
    if (this.match("ON")) {
      const ns = this.expect("STRING", "Expected namespace string after 'on'");
      namespace = ns.value as string;
    }
    this.expect("SEMICOLON", "Expected ';' after node declaration");
    const end = this.previous();
    return { kind: "NodeDecl", name: name.lexeme, namespace, span: this.spanFrom(start, end) };
  }

  private parseTopic(): TopicDecl {
    const start = this.advance();
    const name = this.parseLabel("Expected topic name");
    this.expect("COLON", "Expected ':' after topic name");
    const messageType = this.parseLabel("Expected message type");

    let role: TopicRole = "both";
    let topicPath: string | null = null;
    let qos: QosDecl | null = null;
    let transport: TransportKind | null = null;

    if (this.match("PUBLISH")) {
      role = "publish";
      if (this.match("ON")) {
        if (this.check("STRING")) {
          topicPath = this.advance().value as string;
        } else {
          const ident = this.expect("IDENT", "Expected transport or topic path");
          transport = transportFromIdent(ident.lexeme);
        }
      }
    } else if (this.match("SUBSCRIBE")) {
      role = "subscribe";
      if (this.match("ON")) {
        if (this.check("STRING")) {
          topicPath = this.advance().value as string;
        } else {
          const ident = this.expect("IDENT", "Expected transport or topic path");
          transport = transportFromIdent(ident.lexeme);
        }
      }
    }

    if (this.check("LBRACE")) {
      qos = this.parseQosBlock();
    }

    if (this.match("ON") && topicPath === null && transport === null) {
      if (this.check("STRING")) {
        topicPath = this.advance().value as string;
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
    const start = this.peek();
    this.expect("LBRACE", "Expected '{' for topic QoS block");
    let reliability: QosDecl["reliability"] = null;
    let rateHz: number | null = null;
    let deadlineMs: number | null = null;
    let history: string | null = null;
    while (!this.check("RBRACE") && !this.check("EOF")) {
      if (this.match("QOS")) {
        if (this.match("RELIABLE")) {
          reliability = "reliable";
        } else if (this.match("BEST_EFFORT")) {
          reliability = "best_effort";
        }
        this.expect("SEMICOLON", "Expected ';' after qos reliability");
      } else if (this.match("RATE")) {
        rateHz = this.parseFrequencyHz();
        this.expect("SEMICOLON", "Expected ';' after rate");
      } else if (this.match("DEADLINE")) {
        deadlineMs = this.parseDuration();
        this.expect("SEMICOLON", "Expected ';' after deadline");
      } else if (this.match("HISTORY")) {
        history = this.expect("IDENT", "Expected history policy").lexeme;
        this.expect("SEMICOLON", "Expected ';' after history");
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
    const start = this.advance();
    const name = this.expect("IDENT", "Expected service name");

    if (this.check("LBRACE")) {
      this.advance();
      let requestType: string | null = null;
      let responseType: string | null = null;
      while (!this.check("RBRACE") && !this.check("EOF")) {
        if (this.match("REQUEST")) {
          requestType = this.expect("IDENT", "Expected request type").lexeme;
          this.expect("SEMICOLON", "Expected ';' after request type");
        } else if (this.match("RESPONSE")) {
          responseType = this.expect("IDENT", "Expected response type").lexeme;
          this.expect("SEMICOLON", "Expected ';' after response type");
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
    const start = this.advance();
    const name = this.expect("IDENT", "Expected action name");

    if (this.check("LBRACE")) {
      this.advance();
      let requestType: string | null = null;
      let feedbackType: string | null = null;
      let resultType: string | null = null;
      while (!this.check("RBRACE") && !this.check("EOF")) {
        if (this.match("REQUEST")) {
          requestType = this.expect("IDENT", "Expected request type").lexeme;
          this.expect("SEMICOLON", "Expected ';' after request type");
        } else if (this.match("FEEDBACK")) {
          feedbackType = this.expect("IDENT", "Expected feedback type").lexeme;
          this.expect("SEMICOLON", "Expected ';' after feedback type");
        } else if (this.match("RESULT")) {
          resultType = this.expect("IDENT", "Expected result type").lexeme;
          this.expect("SEMICOLON", "Expected ';' after result type");
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
    const start = this.advance();
    const name = this.expect("IDENT", "Expected sensor name");
    this.expect("COLON", "Expected ':' after sensor name");
    const sensorType = this.expect("IDENT", "Expected sensor type");

    let library: string | null = null;
    if (this.match("FROM")) {
      const vendor = this.expect("IDENT", "Expected library vendor in from clause");
      this.expect("DOT", "Expected '.' in library path");
      const module = this.expect("IDENT", "Expected library module in from clause");
      library = `${vendor.lexeme}.${module.lexeme}`;
    }

    let binding: SensorBinding | null = null;
    if (this.match("ON")) {
      if (this.check("STRING")) {
        const topicTok = this.advance();
        binding = { kind: "topic", path: topicTok.value as string };
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
    const start = this.advance();
    this.expect("LBRACE", "Expected '{' after safety");
    const rules: SafetyRule[] = [];
    const zones: SafetyZoneDecl[] = [];

    while (!this.check("RBRACE") && !this.check("EOF")) {
      if (this.check("STOP_IF")) {
        rules.push(this.parseStopIfRule());
      } else if (this.check("ZONE")) {
        zones.push(this.parseSafetyZone());
      } else if (this.check("IDENT")) {
        rules.push(this.parseMaxSpeedRule());
      } else {
        const t = this.peek();
        throw new ParseError("Expected safety rule or zone", t.line, t.column);
      }
    }

    const end = this.expect("RBRACE", "Expected '}' to close safety block");
    return { kind: "SafetyBlock", rules, zones, span: this.spanFrom(start, end) };
  }

  private parseAiModelDecl(): AiModelDecl {
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
    const entries: AiConfigEntry[] = [];
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
    if (this.check("IDENT") || this.check("PROVIDER")) {
      return this.advance().lexeme;
    }
    const t = this.peek();
    throw new ParseError("Expected config key", t.line, t.column);
  }

  private parseConfigValue(): string | number | boolean {
    if (this.match("STRING")) {
      return this.previous().value as string;
    }
    if (this.match("TRUE")) {
      return true;
    }
    if (this.match("FALSE")) {
      return false;
    }
    if (this.match("NUMBER") || this.match("UNIT_LITERAL")) {
      return this.previous().value as number;
    }
    const t = this.peek();
    throw new ParseError("Expected config value", t.line, t.column);
  }

  private parseAgent(): AgentDecl {
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

    while (!this.check("RBRACE") && !this.check("EOF")) {
      if (this.match("USES")) {
        const modelName = this.expect("IDENT", "Expected model name after uses");
        usesAi.push(modelName.lexeme);
        this.expect("SEMICOLON", "Expected ';' after uses");
      } else if (this.match("MEMORY")) {
        const kindTok = this.expect("IDENT", "Expected memory kind");
        if (kindTok.lexeme !== "short_term" && kindTok.lexeme !== "long_term") {
          throw new ParseError("Memory kind must be short_term or long_term", kindTok.line, kindTok.column);
        }
        memoryKind = kindTok.lexeme;
        this.expect("SEMICOLON", "Expected ';' after memory");
      } else if (this.match("TOOLS")) {
        this.expect("LBRACKET", "Expected '[' after tools");
        if (!this.check("RBRACKET")) {
          do {
            const tool = this.expect("IDENT", "Expected tool name");
            tools.push(tool.lexeme);
          } while (this.match("COMMA"));
        }
        this.expect("RBRACKET", "Expected ']' after tools list");
        this.expect("SEMICOLON", "Expected ';' after tools");
      } else if (this.match("SKILL")) {
        skills.push(this.expect("IDENT", "Expected skill name").lexeme);
        this.expect("SEMICOLON", "Expected ';' after skill");
      } else if (this.match("CAN")) {
        this.expect("LBRACKET", "Expected '[' after can");
        if (!this.check("RBRACKET")) {
          do {
            capabilities.push(this.parseCapability());
          } while (this.match("COMMA"));
        }
        this.expect("RBRACKET", "Expected ']' after capability list");
        this.expect("SEMICOLON", "Expected ';' after can");
      } else if (this.match("GOAL")) {
        const goalTok = this.expect("STRING", "Expected goal string");
        goal = goalTok.value as string;
        this.expect("SEMICOLON", "Expected ';' after goal");
      } else if (this.match("PLAN")) {
        this.expect("LBRACE", "Expected '{' after plan");
        planBody = this.parseBlock();
        this.expect("RBRACE", "Expected '}' to close plan");
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
    const start = this.advance();
    const name = this.expect("IDENT", "Expected zone name");
    let shape: "circle" | "rect" = "circle";
    if (this.match("CIRCLE")) shape = "circle";
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

    if (shape === "circle") {
      this.expect("RADIUS", "Expected 'radius' for circle zone");
      radius = this.parseExpr();
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
    const start = this.peek();
    const name = this.advance();
    this.expect("ASSIGN", "Expected '=' in safety rule");
    const value = this.parseExpr();
    let unit: UnitKind;
    if (value.kind === "UnitLiteralExpr") {
      unit = value.unit;
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
    const start = this.advance();
    const condition = this.parseExpr();
    this.expect("SEMICOLON", "Expected ';' after stop_if rule");
    const end = this.previous();
    return { kind: "StopIfRule", condition, span: this.spanFrom(start, end) };
  }

  private parseContractClauses(): {
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
    const start = this.advance();
    const name = this.expect("IDENT", "Expected task name");
    this.expect("EVERY", "Expected 'every' after task name");
    const intervalMs = this.parseDuration();
    const { requires, ensures, invariant } = this.parseContractClauses();
    this.expect("LBRACE", "Expected '{' after task signature");
    const body = this.parseBlock();
    const end = this.expect("RBRACE", "Expected '}' to close task");
    return {
      kind: "TaskDecl",
      name: name.lexeme,
      intervalMs,
      requires,
      ensures,
      invariant,
      body,
      span: this.spanFrom(start, end),
    };
  }

  private parseStateMachine(): StateMachineDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected state machine name");
    this.expect("LBRACE", "Expected '{' after state machine name");
    const states: string[] = [];
    const transitions: TransitionDecl[] = [];
    while (!this.check("RBRACE") && !this.check("EOF")) {
      if (this.match("STATE")) {
        states.push(this.expect("IDENT", "Expected state name").lexeme);
        this.expect("SEMICOLON", "Expected ';' after state");
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
    const start = this.advance();
    const name = this.expect("IDENT", "Expected event name");
    const fields: FieldDecl[] = [];
    if (this.check("LBRACE")) {
      this.advance();
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

  private parseEventHandler(): EventHandlerDecl {
    const start = this.advance(); // on
    const eventName = this.expect("IDENT", "Expected event name after on");
    this.expect("LBRACE", "Expected '{' after event handler");
    const body = this.parseBlock();
    const end = this.expect("RBRACE", "Expected '}' to close event handler");
    return {
      kind: "EventHandlerDecl",
      eventName: eventName.lexeme,
      body,
      span: this.spanFrom(start, end),
    };
  }

  private parseTwin(): TwinDecl {
    const start = this.advance();
    const name = this.expect("IDENT", "Expected twin name");
    this.expect("LBRACE", "Expected '{' after twin name");
    const mirrors: string[] = [];
    let replay = false;
    while (!this.check("RBRACE") && !this.check("EOF")) {
      if (this.match("MIRROR")) {
        mirrors.push(this.parseLabel("Expected mirror field"));
        this.expect("SEMICOLON", "Expected ';' after mirror");
      } else if (this.match("REPLAY")) {
        replay = this.match("TRUE");
        if (!replay) {
          this.expect("FALSE", "Expected true or false after replay");
        }
        this.expect("SEMICOLON", "Expected ';' after replay");
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
    const start = this.peek();
    let action: string;
    if (this.match("PLAN")) {
      action = "plan";
    } else if (
      this.check("IDENT") ||
      this.check("PUBLISH") ||
      this.check("SUBSCRIBE") ||
      this.check("CALL") ||
      this.check("EXECUTE") ||
      this.check("DISCOVER")
    ) {
      action = this.advance().lexeme;
    } else {
      const t = this.peek();
      throw new ParseError("Expected capability action", t.line, t.column);
    }
    let target: string | null = null;
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
    const stmts: Stmt[] = [];
    while (!this.check("RBRACE") && !this.check("EOF")) {
      stmts.push(this.parseStmt());
    }
    return stmts;
  }

  private parseStmt(): Stmt {
    const start = this.peek();

    if (this.match("LET")) {
      const name = this.parseLocalName("Expected variable name");
      const typeAnnotation = this.match("COLON") ? this.parseTypeAnnotation() : null;
      const init = this.match("ASSIGN") ? this.parseExpr() : null;
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

    if (this.match("IF")) {
      const condition = this.parseExpr();
      this.expect("LBRACE", "Expected '{' after if condition");
      const thenBranch = this.parseBlock();
      this.expect("RBRACE", "Expected '}' after if block");

      let elseBranch: Stmt[] | null = null;
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

    if (this.match("PUBLISH")) {
      const topicName = this.parseSubscribeTarget();
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

    if (this.match("SUBSCRIBE")) {
      const target = this.parseSubscribeTarget();
      this.expect("SEMICOLON", "Expected ';' after subscribe");
      const end = this.previous();
      return { kind: "SubscribeStmt", target, span: this.spanFrom(start, end) };
    }

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

    if (this.match("DISCOVER")) {
      const target = this.parseDiscoverTarget();
      const filter = this.parseDiscoverFilter();
      this.expect("SEMICOLON", "Expected ';' after discover");
      const end = this.previous();
      return { kind: "DiscoverStmt", target, filter, span: this.spanFrom(start, end) };
    }

    if (this.match("RECEIVE")) {
      const topicName = this.expect("IDENT", "Expected topic name after receive").lexeme;
      this.expect("TO", "Expected 'to' after topic in receive");
      const varName = this.expect("IDENT", "Expected variable name").lexeme;
      this.expect("SEMICOLON", "Expected ';' after receive");
      const end = this.previous();
      return { kind: "ReceiveStmt", topicName, varName, span: this.spanFrom(start, end) };
    }

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

    if (this.match("EMERGENCY_STOP")) {
      this.expect("SEMICOLON", "Expected ';' after emergency_stop");
      const end = this.previous();
      return { kind: "EmergencyStopStmt", span: this.spanFrom(start, end) };
    }

    if (this.match("RESET_EMERGENCY_STOP")) {
      this.expect("SEMICOLON", "Expected ';' after reset_emergency_stop");
      const end = this.previous();
      return { kind: "ResetEmergencyStopStmt", span: this.spanFrom(start, end) };
    }

    if (this.match("EMIT")) {
      const eventName = this.parseLabel("Expected event name after emit");
      this.expect("SEMICOLON", "Expected ';' after emit statement");
      const end = this.previous();
      return { kind: "EmitStmt", eventName, span: this.spanFrom(start, end) };
    }

    if (this.match("ENTER")) {
      const stateName = this.parseLabel("Expected state name after enter");
      this.expect("SEMICOLON", "Expected ';' after enter statement");
      const end = this.previous();
      return { kind: "EnterStmt", stateName, span: this.spanFrom(start, end) };
    }

    if (this.match("REMEMBER")) {
      const keyTok = this.expect("STRING", "Expected memory key string");
      const key = keyTok.value as string;
      this.expect("COMMA", "Expected ',' after memory key");
      const value = this.parseExpr();
      this.expect("SEMICOLON", "Expected ';' after remember statement");
      const end = this.previous();
      return { kind: "RememberStmt", key, value, span: this.spanFrom(start, end) };
    }

    const expr = this.parseExpr();
    this.expect("SEMICOLON", "Expected ';' after expression");
    const end = this.previous();
    return { kind: "ExprStmt", expr, span: this.spanFrom(start, end) };
  }

  private parseSubscribeTarget(): string {
    const first = this.parseLabel("Expected subscribe target");
    if (this.match("DOT")) {
      const second = this.parseLabel("Expected member after '.'");
      return `${first}.${second}`;
    }
    return first;
  }

  private parseDiscoverTarget(): DiscoverTarget {
    const name = this.expect("IDENT", "Expected discover target").lexeme;
    if (name === "robots") return "robots";
    if (name === "agents") return "agents";
    if (name === "devices") return "devices";
    const t = this.previous();
    throw new ParseError(
      `Expected robots, agents, or devices in discover, got '${name}'`,
      t.line,
      t.column,
    );
  }

  private parseDiscoverFilter(): DiscoverFilter | null {
    if (!this.match("WHERE")) return null;
    this.expect("IDENT", "Expected 'capability' in discover filter");
    this.expect("INCLUDES", "Expected 'includes' in discover filter");
    const capability = this.expect("IDENT", "Expected capability name").lexeme;
    return { capability };
  }

  private parseDuration(): number {
    const tok = this.peek();
    if (tok.type === "UNIT_LITERAL" && tok.unit === "ms") {
      this.advance();
      return tok.value as number;
    }
    if (tok.type === "UNIT_LITERAL" && tok.unit === "s") {
      this.advance();
      return (tok.value as number) * 1000;
    }
    if (tok.type === "NUMBER") {
      this.advance();
      if (this.check("IDENT") && this.peek().lexeme === "ms") {
        this.advance();
        return tok.value as number;
      }
    }
    throw new ParseError("Expected duration like 50ms", tok.line, tok.column);
  }

  private parseUnitSuffix(): UnitKind {
    const unit = this.tryParseUnitSuffix();
    if (!unit) {
      const t = this.peek();
      throw new ParseError("Expected unit suffix", t.line, t.column);
    }
    return unit;
  }

  private tryParseUnitSuffix(): UnitKind | null {
    if (this.check("UNIT_LITERAL")) {
      const t = this.advance();
      return unitFromLexeme(t.unit!);
    }

    if (this.check("IDENT") && this.peek().lexeme === "m" && this.tokens[this.pos + 1]?.type === "SLASH" && this.tokens[this.pos + 2]?.lexeme === "s") {
      this.advance();
      this.advance();
      this.advance();
      return "m/s";
    }

    if (this.check("IDENT") && this.peek().lexeme === "rad" && this.tokens[this.pos + 1]?.type === "SLASH" && this.tokens[this.pos + 2]?.lexeme === "s") {
      this.advance();
      this.advance();
      this.advance();
      return "rad/s";
    }

    if (this.check("IDENT")) {
      const lexeme = this.peek().lexeme;
      if (isUnitIdent(lexeme)) {
        this.advance();
        return unitFromLexeme(lexeme as import("../lexer/index.js").UnitLexeme);
      }
    }

    return null;
  }

  private parseExpr(): Expr {
    return this.parseOr();
  }

  private parseOr(): Expr {
    let left = this.parseAnd();
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
    let left = this.parseComparison();
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
    let left = this.parseAdditive();
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
    let left = this.parseMultiplicative();
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
    let left = this.parseUnary();
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
    let expr = this.parsePrimary();

    while (true) {
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
      } else if (this.match("LPAREN")) {
        const args: Expr[] = [];
        const namedArgs: NamedArg[] = [];

        if (!this.check("RPAREN")) {
          do {
            if (this.isNamedArgStart()) {
              const name = this.parseNamedArgName();
              this.advance(); // colon
              const value = this.parseExpr();
              namedArgs.push({
                name,
                value,
                span: this.spanFrom(this.previous(), this.previous()),
              });
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
      } else if (this.check("LBRACE")) {
        if (expr.kind === "IdentExpr" && /^[A-Z]/.test(expr.name)) {
          this.advance();
          const fields: import("../ast/nodes.js").StructFieldInit[] = [];
          if (!this.check("RBRACE")) {
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
      } else {
        break;
      }
    }

    return expr;
  }

  private parsePrimary(): Expr {
    const start = this.peek();

    if (this.match("MATCH")) {
      const scrutinee = this.parseExpr();
      this.expect("LBRACE", "Expected '{' after match scrutinee");
      const arms: MatchArm[] = [];
      while (!this.check("RBRACE") && !this.check("EOF")) {
        const armStart = this.peek();
        const variant = this.parseLabel("Expected match arm variant");
        this.expect("FAT_ARROW", "Expected '=>' in match arm");
        let body: Stmt[];
        if (this.check("LBRACE")) {
          this.advance();
          body = this.parseBlock();
          this.expect("RBRACE", "Expected '}' to close match arm");
        } else {
          body = [this.parseStmt()];
        }
        arms.push({
          variant,
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

    if (this.match("ROBOT")) {
      const tok = this.previous();
      return {
        kind: "IdentExpr",
        name: "robot",
        span: this.spanFrom(start, tok),
      };
    }
    if (this.match("SAFETY")) {
      const tok = this.previous();
      return {
        kind: "IdentExpr",
        name: "safety",
        span: this.spanFrom(start, tok),
      };
    }
    if (this.match("TRUE")) {
      return {
        kind: "LiteralExpr",
        value: true,
        span: this.spanFrom(start, this.previous()),
      };
    }
    if (this.match("FALSE")) {
      return {
        kind: "LiteralExpr",
        value: false,
        span: this.spanFrom(start, this.previous()),
      };
    }
    if (this.match("NUMBER")) {
      const tok = this.previous();
      const unit = this.tryParseUnitSuffix();
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
    if (this.match("UNIT_LITERAL")) {
      const tok = this.previous();
      return {
        kind: "UnitLiteralExpr",
        value: tok.value as number,
        unit: unitFromLexeme(tok.unit!),
        span: this.spanFrom(start, tok),
      };
    }
    if (this.match("STRING")) {
      return {
        kind: "LiteralExpr",
        value: this.previous().value as string,
        span: this.spanFrom(start, this.previous()),
      };
    }
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
        "SUBSCRIBE",
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
    if (this.match("LPAREN")) {
      const expr = this.parseExpr();
      const end = this.expect("RPAREN", "Expected ')' after expression");
      return { ...expr, span: this.spanFrom(start, end) };
    }

    const t = this.peek();
    throw new ParseError("Expected expression", t.line, t.column);
  }

  private parsePropertyName(): Token {
    const lexeme = this.parseLabel("Expected property name after '.'");
    const end = this.previous();
    return { type: "IDENT", lexeme, value: null, line: end.line, column: end.column, offset: end.offset };
  }

  private isNamedArgStart(): boolean {
    const next = this.tokens[this.pos + 1];
    if (next?.type !== "COLON") return false;
    return this.check("IDENT") || this.check("FROM") || this.check("GOAL") || this.check("TO");
  }

  private parseNamedArgName(): string {
    if (this.match("FROM")) return "from";
    if (this.match("TO")) return "to";
    if (this.match("GOAL")) return "goal";
    return this.advance().lexeme;
  }
}

export { parse as parseTokens };

function isUnitIdent(lexeme: string): boolean {
  return ["m", "s", "ms", "rad", "deg", "Hz"].includes(lexeme);
}
