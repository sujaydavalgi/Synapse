/**
 * index module (lexer/index.ts).
 * @module
 */

export type TokenType =
  | "IMPORT"
  | "MODULE"
  | "STRUCT"
  | "ENUM"
  | "TRAIT"
  | "DYN"
  | "IMPL"
  | "FOR"
  | "MATCH"
  | "FN"
  | "EXPORT"
  | "PUBLIC"
  | "PRIVATE"
  | "RETURN"
  | "ASYNC"
  | "AWAIT"
  | "SPAWN"
  | "SELECT"
  | "PARALLEL"
  | "EXTERN"
  | "STATE_MACHINE"
  | "TASK"
  | "SKILL"
  | "EVENT"
  | "TWIN"
  | "STATE"
  | "RESOURCE"
  | "REQUIRES"
  | "ENSURES"
  | "INVARIANT"
  | "CAN"
  | "TRANSITION"
  | "MIRROR"
  | "REPLAY"
  | "EMIT"
  | "ENTER"
  | "ARROW"
  | "FAT_ARROW"
  | "HAL"
  | "SOC"
  | "FROM"
  | "I2C"
  | "SPI"
  | "UART"
  | "GPIO"
  | "PWM"
  | "ADC"
  | "OUT"
  | "IN"
  | "BAUD"
  | "FREQUENCY"
  | "PIN"
  | "ROBOT"
  | "NODE"
  | "TOPIC"
  | "SERVICE"
  | "ACTION"
  | "SENSOR"
  | "ACTUATOR"
  | "SAFETY"
  | "AI_MODEL"
  | "AGENT"
  | "USES"
  | "TOOLS"
  | "GOAL"
  | "PLAN"
  | "MEMORY"
  | "PROVIDER"
  | "BEHAVIOR"
  | "LOOP"
  | "EVERY"
  | "LET"
  | "IF"
  | "ELSE"
  | "STOP_IF"
  | "PUBLISH"
  | "CALL"
  | "SEND_GOAL"
  | "WITH"
  | "ZONE"
  | "CIRCLE"
  | "RECT"
  | "AT"
  | "RADIUS"
  | "SIZE"
  | "EMERGENCY_STOP"
  | "RESET_EMERGENCY_STOP"
  | "REMEMBER"
  | "VERIFY"
  | "OBSERVE"
  | "SECRET"
  | "TRUST"
  | "PERMISSIONS"
  | "SECURE"
  | "ENV"
  | "SIGNED_BY"
  | "HARDWARE"
  | "DEPLOY"
  | "CPU"
  | "STORAGE"
  | "GPU"
  | "BATTERY"
  | "CAPACITY"
  | "SENSORS"
  | "ACTUATORS"
  | "TO"
  | "REQUIRES_HARDWARE"
  | "REQUIRES_NETWORK"
  | "REQUIRES_CONNECTIVITY"
  | "CONNECTIVITY"
  | "GEOFENCE"
  | "CONNECTIVITY_POLICY"
  | "BLUETOOTH"
  | "BLE_SERVICE"
  | "PACKET_LOSS"
  | "SWITCH_IF"
  | "TRUSTED_ONLY"
  | "SIMULATE_COMPATIBILITY"
  | "BUDGET"
  | "FAULT"
  | "MISSION"
  | "NETWORK"
  | "BANDWIDTH"
  | "LATENCY"
  | "TIMING"
  | "MIN_PERIOD"
  | "DURATION"
  | "MESSAGE"
  | "SUBSCRIBE"
  | "EXECUTE"
  | "DISCOVER"
  | "BUS"
  | "DEVICE"
  | "REQUEST"
  | "RESPONSE"
  | "FEEDBACK"
  | "RESULT"
  | "QOS"
  | "RELIABLE"
  | "BEST_EFFORT"
  | "RATE"
  | "HISTORY"
  | "DEADLINE"
  | "WHERE"
  | "INCLUDES"
  | "RECEIVE"
  | "TELEMETRY"
  | "FAULTS"
  | "ON"
  | "TRUE"
  | "FALSE"
  | "AND"
  | "OR"
  | "NOT"
  | "WATCHDOG"
  | "PIPELINE"
  | "RECOVER"
  | "RETRY"
  | "FALLBACK"
  | "ISOLATED"
  | "JITTER"
  | "BACKOFF"
  | "TIMES"
  | "MATCHES"
  | "WHEN"
  | "WHILE"
  | "USE"
  | "PRIORITY"
  | "REGEX_LITERAL"
  | "PERCENT"
  | "IDENT"
  | "NUMBER"
  | "STRING"
  | "UNIT_LITERAL"
  | "LBRACE"
  | "RBRACE"
  | "LBRACKET"
  | "RBRACKET"
  | "LPAREN"
  | "RPAREN"
  | "SEMICOLON"
  | "COLON"
  | "COMMA"
  | "DOT"
  | "ASSIGN"
  | "PLUS"
  | "MINUS"
  | "STAR"
  | "SLASH"
  | "LT"
  | "LTE"
  | "GT"
  | "GTE"
  | "EQ"
  | "NEQ"
  | "EOF";

export type UnitLexeme =
  | "%VWC"
  | "%RH"
  | "µg/m³"
  | "ug/m3"
  | "uS/cm"
  | "mS/cm"
  | "uSv/h"
  | "mSv/h"
  | "S/m"
  | "cd/m²"
  | "cd/m2"
  | "N·m"
  | "kWh"
  | "dBA"
  | "MHz"
  | "km/h"
  | "m/s²"
  | "m/s2"
  | "m/s"
  | "rad/s"
  | "deg/s"
  | "fahrenheit"
  | "celsius"
  | "kelvin"
  | "kHz"
  | "kPa"
  | "kN"
  | "kW"
  | "kV"
  | "mbar"
  | "mph"
  | "gram"
  | "mm"
  | "cm"
  | "km"
  | "ms"
  | "us"
  | "mV"
  | "mA"
  | "min"
  | "deg"
  | "rad"
  | "psi"
  | "bar"
  | "Pa"
  | "Hz"
  | "ft"
  | "in"
  | "kg"
  | "lb"
  | "MW"
  | "m"
  | "s"
  | "h"
  | "g"
  | "N"
  | "W"
  | "V"
  | "A"
  | "rh"
  | "lux"
  | "lx"
  | "nit"
  | "ppm"
  | "ppb"
  | "dB"
  | "uT"
  | "gauss"
  | "rpm"
  | "Nm"
  | "J"
  | "Wh"
  | "uvi"
  | "pH"
  | "NTU"
  | "FNU"
  | "ppt"
  | "psu"
  | "vwc";

export type Token = {
  type: TokenType;
  lexeme: string;
  value: string | number | boolean | null;
  unit?: UnitLexeme;
  line: number;
  column: number;
  offset: number;
};

export class LexerError extends Error {
  constructor(
    message: string,
    public line: number,
    public column: number,
  ) {
    super(message);
    this.name = "LexerError";
  }
}

const KEYWORDS: Record<string, TokenType> = {
  import: "IMPORT",
  module: "MODULE",
  struct: "STRUCT",
  enum: "ENUM",
  trait: "TRAIT",
  dyn: "DYN",
  impl: "IMPL",
  for: "FOR",
  match: "MATCH",
  fn: "FN",
  export: "EXPORT",
  public: "PUBLIC",
  private: "PRIVATE",
  return: "RETURN",
  async: "ASYNC",
  await: "AWAIT",
  spawn: "SPAWN",
  select: "SELECT",
  parallel: "PARALLEL",
  extern: "EXTERN",
  state_machine: "STATE_MACHINE",
  task: "TASK",
  skill: "SKILL",
  event: "EVENT",
  twin: "TWIN",
  state: "STATE",
  resource: "RESOURCE",
  requires: "REQUIRES",
  ensures: "ENSURES",
  invariant: "INVARIANT",
  can: "CAN",
  transition: "TRANSITION",
  mirror: "MIRROR",
  replay: "REPLAY",
  emit: "EMIT",
  enter: "ENTER",
  hal: "HAL",
  soc: "SOC",
  from: "FROM",
  i2c: "I2C",
  spi: "SPI",
  uart: "UART",
  gpio: "GPIO",
  pwm: "PWM",
  adc: "ADC",
  out: "OUT",
  in: "IN",
  baud: "BAUD",
  frequency: "FREQUENCY",
  pin: "PIN",
  robot: "ROBOT",
  node: "NODE",
  topic: "TOPIC",
  service: "SERVICE",
  action: "ACTION",
  sensor: "SENSOR",
  actuator: "ACTUATOR",
  safety: "SAFETY",
  ai_model: "AI_MODEL",
  agent: "AGENT",
  uses: "USES",
  tools: "TOOLS",
  goal: "GOAL",
  plan: "PLAN",
  memory: "MEMORY",
  provider: "PROVIDER",
  behavior: "BEHAVIOR",
  loop: "LOOP",
  every: "EVERY",
  let: "LET",
  if: "IF",
  else: "ELSE",
  stop_if: "STOP_IF",
  publish: "PUBLISH",
  call: "CALL",
  send_goal: "SEND_GOAL",
  with: "WITH",
  zone: "ZONE",
  circle: "CIRCLE",
  rect: "RECT",
  at: "AT",
  radius: "RADIUS",
  size: "SIZE",
  emergency_stop: "EMERGENCY_STOP",
  reset_emergency_stop: "RESET_EMERGENCY_STOP",
  remember: "REMEMBER",
  verify: "VERIFY",
  observe: "OBSERVE",
  secret: "SECRET",
  trust: "TRUST",
  permissions: "PERMISSIONS",
  secure: "SECURE",
  env: "ENV",
  signed_by: "SIGNED_BY",
  hardware: "HARDWARE",
  deploy: "DEPLOY",
  cpu: "CPU",
  storage: "STORAGE",
  gpu: "GPU",
  battery: "BATTERY",
  capacity: "CAPACITY",
  sensors: "SENSORS",
  actuators: "ACTUATORS",
  to: "TO",
  requires_hardware: "REQUIRES_HARDWARE",
  requires_network: "REQUIRES_NETWORK",
  requires_connectivity: "REQUIRES_CONNECTIVITY",
  connectivity: "CONNECTIVITY",
  geofence: "GEOFENCE",
  connectivity_policy: "CONNECTIVITY_POLICY",
  bluetooth: "BLUETOOTH",
  ble_service: "BLE_SERVICE",
  packet_loss: "PACKET_LOSS",
  switch_if: "SWITCH_IF",
  trusted_only: "TRUSTED_ONLY",
  simulate_compatibility: "SIMULATE_COMPATIBILITY",
  budget: "BUDGET",
  fault: "FAULT",
  mission: "MISSION",
  network: "NETWORK",
  bandwidth: "BANDWIDTH",
  latency: "LATENCY",
  timing: "TIMING",
  min_period: "MIN_PERIOD",
  duration: "DURATION",
  message: "MESSAGE",
  subscribe: "SUBSCRIBE",
  execute: "EXECUTE",
  discover: "DISCOVER",
  bus: "BUS",
  device: "DEVICE",
  request: "REQUEST",
  response: "RESPONSE",
  feedback: "FEEDBACK",
  result: "RESULT",
  qos: "QOS",
  reliable: "RELIABLE",
  best_effort: "BEST_EFFORT",
  rate: "RATE",
  history: "HISTORY",
  deadline: "DEADLINE",
  where: "WHERE",
  includes: "INCLUDES",
  receive: "RECEIVE",
  telemetry: "TELEMETRY",
  faults: "FAULTS",
  on: "ON",
  true: "TRUE",
  false: "FALSE",
  and: "AND",
  or: "OR",
  not: "NOT",
  watchdog: "WATCHDOG",
  pipeline: "PIPELINE",
  recover: "RECOVER",
  retry: "RETRY",
  fallback: "FALLBACK",
  isolated: "ISOLATED",
  jitter: "JITTER",
  backoff: "BACKOFF",
  times: "TIMES",
  matches: "MATCHES",
  when: "WHEN",
  while: "WHILE",
  use: "USE",
  priority: "PRIORITY",
};

const UNIT_SUFFIXES: UnitLexeme[] = [
  "%VWC", "%RH", "µg/m³", "ug/m3", "uS/cm", "mS/cm", "uSv/h", "mSv/h", "S/m",
  "cd/m2", "cd/m²", "N·m", "kWh", "dBA", "MHz", "km/h", "m/s2", "m/s²", "m/s", "rad/s", "deg/s",
  "fahrenheit", "celsius", "kelvin",
  "kHz", "kPa", "kN", "kW", "kV", "mbar", "mph", "gram",
  "mm", "cm", "km", "ms", "us", "mV", "mA", "min",
  "deg", "rad", "psi", "bar", "Pa", "Hz",
  "ft", "in", "kg", "lb", "MW", "m", "s", "h", "g", "N", "W", "V", "A",
  "rh", "lux", "lx", "nit", "ppm", "ppb", "dB", "uT", "gauss", "rpm", "Nm", "J", "Wh",
  "uvi", "pH", "NTU", "FNU", "ppt", "psu", "vwc",
];

export function tokenize(source: string): Token[] {
  // Tokenize.
  //
  // Parameters:
  // - `source` — input value
  //
  // Returns:
  // `Token[]`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = tokenize(source);
  const tokens: Token[] = [];
  let line = 1;
  let column = 1;
  let i = 0;
  const loc = () => ({ line, column, offset: i });

  // Repeat while i < source.length.
  while (i < source.length) {
    const ch = source[i];

    // continue when ch equals " " || ch === "\t" || ch === "\r".
    if (ch === " " || ch === "\t" || ch === "\r") {
      i++;
      column++;
      continue;
    }

    // continue when ch equals "\n".
    if (ch === "\n") {
      i++;
      line++;
      column = 1;
      continue;
    }

    // continue when ch equals "/" && source[i + 1] === "/".
    if (ch === "/" && source[i + 1] === "/") {

      // Repeat while i < source.length && source[i] !== "\n".
      while (i < source.length && source[i] !== "\n") {
        i++;
      }
      continue;
    }
    const start = loc();

    // continue when ch equals "[".
    if (ch === "[") {
      tokens.push({ type: "LBRACKET", lexeme: "[", value: null, ...start });
      i++;
      column++;
      continue;
    }

    // continue when ch equals "]".
    if (ch === "]") {
      tokens.push({ type: "RBRACKET", lexeme: "]", value: null, ...start });
      i++;
      column++;
      continue;
    }

    // continue when ch equals "{".
    if (ch === "{") {
      tokens.push({ type: "LBRACE", lexeme: "{", value: null, ...start });
      i++;
      column++;
      continue;
    }

    // continue when ch equals "}".
    if (ch === "}") {
      tokens.push({ type: "RBRACE", lexeme: "}", value: null, ...start });
      i++;
      column++;
      continue;
    }

    // continue when ch equals ".
    if (ch === "(") {
      tokens.push({ type: "LPAREN", lexeme: "(", value: null, ...start });
      i++;
      column++;
      continue;
    }

    // continue when ch equals ")".
    if (ch === ")") {
      tokens.push({ type: "RPAREN", lexeme: ")", value: null, ...start });
      i++;
      column++;
      continue;
    }

    // continue when ch equals ";".
    if (ch === ";") {
      tokens.push({ type: "SEMICOLON", lexeme: ";", value: null, ...start });
      i++;
      column++;
      continue;
    }

    // continue when ch equals ":".
    if (ch === ":") {
      tokens.push({ type: "COLON", lexeme: ":", value: null, ...start });
      i++;
      column++;
      continue;
    }

    // continue when ch equals ",".
    if (ch === ",") {
      tokens.push({ type: "COMMA", lexeme: ",", value: null, ...start });
      i++;
      column++;
      continue;
    }

    // continue when ch equals ".
    if (ch === ".") {
      tokens.push({ type: "DOT", lexeme: ".", value: null, ...start });
      i++;
      column++;
      continue;
    }

    // continue when ch equals "+".
    if (ch === "+") {
      tokens.push({ type: "PLUS", lexeme: "+", value: null, ...start });
      i++;
      column++;
      continue;
    }

    // continue when ch equals "-" && source[i + 1] === ">".
    if (ch === "-" && source[i + 1] === ">") {
      tokens.push({ type: "ARROW", lexeme: "->", value: null, ...start });
      i += 2;
      column += 2;
      continue;
    }

    // continue when ch equals "-".
    if (ch === "-") {
      tokens.push({ type: "MINUS", lexeme: "-", value: null, ...start });
      i++;
      column++;
      continue;
    }

    // continue when ch equals "*".
    if (ch === "*") {
      tokens.push({ type: "STAR", lexeme: "*", value: null, ...start });
      i++;
      column++;
      continue;
    }

    // continue when ch equals "/".
    if (ch === "/") {
      if (regexLiteralContext(tokens.at(-1))) {
        const { lexeme, advanceBy } = lexRegexLiteral(source, i);
        tokens.push({ type: "REGEX_LITERAL", lexeme, value: lexeme, ...start });
        i += advanceBy;
        column += advanceBy;
        continue;
      }
      tokens.push({ type: "SLASH", lexeme: "/", value: null, ...start });
      i++;
      column++;
      continue;
    }

    // continue when ch equals "%".
    if (ch === "%") {
      tokens.push({ type: "PERCENT", lexeme: "%", value: null, ...start });
      i++;
      column++;
      continue;
    }

    // continue when ch equals "<" && source[i + 1] === "=".
    if (ch === "<" && source[i + 1] === "=") {
      tokens.push({ type: "LTE", lexeme: "<=", value: null, ...start });
      i += 2;
      column += 2;
      continue;
    }

    // continue when ch equals "<".
    if (ch === "<") {
      tokens.push({ type: "LT", lexeme: "<", value: null, ...start });
      i++;
      column++;
      continue;
    }

    // continue when ch equals ">" && source[i + 1] === "=".
    if (ch === ">" && source[i + 1] === "=") {
      tokens.push({ type: "GTE", lexeme: ">=", value: null, ...start });
      i += 2;
      column += 2;
      continue;
    }

    // continue when ch equals ">".
    if (ch === ">") {
      tokens.push({ type: "GT", lexeme: ">", value: null, ...start });
      i++;
      column++;
      continue;
    }

    // continue when ch equals "=" && source[i + 1] === "=".
    if (ch === "=" && source[i + 1] === "=") {
      tokens.push({ type: "EQ", lexeme: "==", value: null, ...start });
      i += 2;
      column += 2;
      continue;
    }

    // continue when ch equals "!" && source[i + 1] === "=".
    if (ch === "!" && source[i + 1] === "=") {
      tokens.push({ type: "NEQ", lexeme: "!=", value: null, ...start });
      i += 2;
      column += 2;
      continue;
    }

    // continue when ch equals "=" && source[i + 1] === ">".
    if (ch === "=" && source[i + 1] === ">") {
      tokens.push({ type: "FAT_ARROW", lexeme: "=>", value: null, ...start });
      i += 2;
      column += 2;
      continue;
    }

    // continue when ch equals "=".
    if (ch === "=") {
      tokens.push({ type: "ASSIGN", lexeme: "=", value: null, ...start });
      i++;
      column++;
      continue;
    }

    // continue when ch equals '"'.
    if (ch === '"') {
      i++;
      column++;
      let value = "";

      // Repeat while i < source.length && source[i] !== '"'.
      while (i < source.length && source[i] !== '"') {

        // continue when source[i] equals length.
        if (source[i] === "\\" && i + 1 < source.length) {
          value += source[i + 1];
          i += 2;
          column += 2;

        // Handle any remaining cases.
        } else {
          value += source[i];
          i++;
          column++;
        }
      }

      // continue when i >= source.length.
      if (i >= source.length) {
        throw new LexerError("Unterminated string", line, column);
      }
      i++;
      column++;
      tokens.push({ type: "STRING", lexeme: value, value, ...start });
      continue;
    }

    // continue when ch equals "0" &&.
    if (ch === "0" && (source[i + 1] === "x" || source[i + 1] === "X")) {
      i += 2;
      column += 2;
      let hexStr = "";

      // Repeat while i < source.length && isHexDigit(source[i]).
      while (i < source.length && isHexDigit(source[i])) {
        hexStr += source[i];
        i++;
        column++;
      }
      const num = parseInt(hexStr, 16);
      tokens.push({ type: "NUMBER", lexeme: `0x${hexStr}`, value: num, ...start });
      continue;
    }

    // continue when isDigit equals " && isDigit.
    if (isDigit(ch) || (ch === "." && isDigit(source[i + 1]))) {
      let numStr = "";

      // Repeat while i < source.length && (isDigit(source[i]) || source[i] === ".").
      while (i < source.length && (isDigit(source[i]) || source[i] === ".")) {
        numStr += source[i];
        i++;
        column++;
      }
      const num = parseFloat(numStr);
      let matchedUnit: UnitLexeme | undefined;

      // Repeat while i < source.length && (source[i] === " " || source[i] === "\t").
      while (i < source.length && (source[i] === " " || source[i] === "\t")) {
        i++;
        column++;
      }

      // Iterate over UNIT SUFFIXES.
      for (const suffix of UNIT_SUFFIXES) {

        // continue when length) equals suffix.
        if (source.slice(i, i + suffix.length) === suffix) {
          const next = source[i + suffix.length];

          // continue when next && equals "/").
          if (next && (isIdentChar(next) || next === "/")) continue;
          matchedUnit = suffix;
          i += suffix.length;
          column += suffix.length;
          break;
        }
      }

      // continue when matchedUnit.
      if (matchedUnit) {
        tokens.push({
          type: "UNIT_LITERAL",
          lexeme: `${numStr}${matchedUnit}`,
          value: num,
          unit: matchedUnit,
          ...start,
        });

      // Handle any remaining cases.
      } else {
        tokens.push({ type: "NUMBER", lexeme: numStr, value: num, ...start });
      }
      continue;
    }

    // continue when isIdentStart(ch).
    if (isIdentStart(ch)) {
      let ident = "";

      // Repeat while i < source.length && isIdentChar(source[i]).
      while (i < source.length && isIdentChar(source[i])) {
        ident += source[i];
        i++;
        column++;
      }
      const kw = KEYWORDS[ident];
      tokens.push({
        type: kw ?? "IDENT",
        lexeme: ident,
        value: ident,
        ...start,
      });
      continue;
    }
    throw new LexerError(`Unexpected character '${ch}'`, line, column);
  }
  tokens.push({
    type: "EOF",
    lexeme: "",
    value: null,
    line,
    column,
    offset: i,
  });
  return tokens;
}

function isHexDigit(ch: string): boolean {
  // IsHexDigit.
  //
  // Parameters:
  // - `ch` — input value
  //
  // Returns:
  // `true` or `false`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = isHexDigit(ch);
  return isDigit(ch) || (ch >= "a" && ch <= "f") || (ch >= "A" && ch <= "F");
}

function isDigit(ch: string): boolean {
  // IsDigit.
  //
  // Parameters:
  // - `ch` — input value
  //
  // Returns:
  // `true` or `false`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = isDigit(ch);
  return ch >= "0" && ch <= "9";
}

function isIdentStart(ch: string): boolean {
  // IsIdentStart.
  //
  // Parameters:
  // - `ch` — input value
  //
  // Returns:
  // `true` or `false`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = isIdentStart(ch);
  return (ch >= "a" && ch <= "z") || (ch >= "A" && ch <= "Z") || ch === "_";
}

function isIdentChar(ch: string): boolean {
  // IsIdentChar.
  //
  // Parameters:
  // - `ch` — input value
  //
  // Returns:
  // `true` or `false`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = isIdentChar(ch);
  return isIdentStart(ch) || isDigit(ch);
}

function regexLiteralContext(previous: Token | undefined): boolean {
  // Decide whether `/` starts a regex literal instead of division.
  if (!previous) {
    return true;
  }
  return [
    "ASSIGN",
    "LPAREN",
    "COMMA",
    "SEMICOLON",
    "LBRACE",
    "COLON",
    "MATCHES",
    "WHERE",
    "FAT_ARROW",
    "RETURN",
    "PLUS",
    "MINUS",
    "STAR",
    "LT",
    "LTE",
    "GT",
    "GTE",
    "EQ",
    "NEQ",
    "LET",
  ].includes(previous.type);
}

function lexRegexLiteral(source: string, start: number): { lexeme: string; advanceBy: number } {
  // Lex a `/pattern/flags` regex literal starting at the opening slash.
  let i = start + 1;
  let escaped = false;
  while (i < source.length) {
    const ch = source[i]!;
    if (escaped) {
      escaped = false;
      i++;
      continue;
    }
    if (ch === "\\") {
      escaped = true;
      i++;
      continue;
    }
    if (ch === "/") {
      i++;
      break;
    }
    i++;
  }
  while (i < source.length && "ims".includes(source[i]!)) {
    i++;
  }
  return { lexeme: source.slice(start, i), advanceBy: i - start };
}

export function unitFromLexeme(lexeme: UnitLexeme): import("../ast/nodes.js").UnitKind {
  // UnitFromLexeme.
  //
  // Parameters:
  // - `lexeme` — input value
  //
  // Returns:
  // `import("../ast/nodes.js").UnitKind`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = unitFromLexeme(lexeme);
  if (lexeme === "m/s2" || lexeme === "m/s²") return "m/s²";

  // continue when lexeme equals "cd/m2".
  if (lexeme === "cd/m2") return "cd/m²";

  // continue when lexeme equals "µg/m³".
  if (lexeme === "µg/m³") return "ug/m3";
  return lexeme;
}
