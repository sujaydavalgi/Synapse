export type TokenType =
  | "IMPORT"
  | "MODULE"
  | "STRUCT"
  | "ENUM"
  | "TRAIT"
  | "IMPL"
  | "FOR"
  | "MATCH"
  | "FN"
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
  impl: "IMPL",
  for: "FOR",
  match: "MATCH",
  fn: "FN",
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
  const tokens: Token[] = [];
  let line = 1;
  let column = 1;
  let i = 0;

  const loc = () => ({ line, column, offset: i });

  while (i < source.length) {
    const ch = source[i];

    if (ch === " " || ch === "\t" || ch === "\r") {
      i++;
      column++;
      continue;
    }

    if (ch === "\n") {
      i++;
      line++;
      column = 1;
      continue;
    }

    if (ch === "/" && source[i + 1] === "/") {
      while (i < source.length && source[i] !== "\n") {
        i++;
      }
      continue;
    }

    const start = loc();

    if (ch === "[") {
      tokens.push({ type: "LBRACKET", lexeme: "[", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === "]") {
      tokens.push({ type: "RBRACKET", lexeme: "]", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === "{") {
      tokens.push({ type: "LBRACE", lexeme: "{", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === "}") {
      tokens.push({ type: "RBRACE", lexeme: "}", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === "(") {
      tokens.push({ type: "LPAREN", lexeme: "(", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === ")") {
      tokens.push({ type: "RPAREN", lexeme: ")", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === ";") {
      tokens.push({ type: "SEMICOLON", lexeme: ";", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === ":") {
      tokens.push({ type: "COLON", lexeme: ":", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === ",") {
      tokens.push({ type: "COMMA", lexeme: ",", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === ".") {
      tokens.push({ type: "DOT", lexeme: ".", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === "+") {
      tokens.push({ type: "PLUS", lexeme: "+", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === "-" && source[i + 1] === ">") {
      tokens.push({ type: "ARROW", lexeme: "->", value: null, ...start });
      i += 2;
      column += 2;
      continue;
    }
    if (ch === "-") {
      tokens.push({ type: "MINUS", lexeme: "-", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === "*") {
      tokens.push({ type: "STAR", lexeme: "*", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === "/") {
      tokens.push({ type: "SLASH", lexeme: "/", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === "<" && source[i + 1] === "=") {
      tokens.push({ type: "LTE", lexeme: "<=", value: null, ...start });
      i += 2;
      column += 2;
      continue;
    }
    if (ch === "<") {
      tokens.push({ type: "LT", lexeme: "<", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === ">" && source[i + 1] === "=") {
      tokens.push({ type: "GTE", lexeme: ">=", value: null, ...start });
      i += 2;
      column += 2;
      continue;
    }
    if (ch === ">") {
      tokens.push({ type: "GT", lexeme: ">", value: null, ...start });
      i++;
      column++;
      continue;
    }
    if (ch === "=" && source[i + 1] === "=") {
      tokens.push({ type: "EQ", lexeme: "==", value: null, ...start });
      i += 2;
      column += 2;
      continue;
    }
    if (ch === "!" && source[i + 1] === "=") {
      tokens.push({ type: "NEQ", lexeme: "!=", value: null, ...start });
      i += 2;
      column += 2;
      continue;
    }
    if (ch === "=" && source[i + 1] === ">") {
      tokens.push({ type: "FAT_ARROW", lexeme: "=>", value: null, ...start });
      i += 2;
      column += 2;
      continue;
    }
    if (ch === "=") {
      tokens.push({ type: "ASSIGN", lexeme: "=", value: null, ...start });
      i++;
      column++;
      continue;
    }

    if (ch === '"') {
      i++;
      column++;
      let value = "";
      while (i < source.length && source[i] !== '"') {
        if (source[i] === "\\" && i + 1 < source.length) {
          value += source[i + 1];
          i += 2;
          column += 2;
        } else {
          value += source[i];
          i++;
          column++;
        }
      }
      if (i >= source.length) {
        throw new LexerError("Unterminated string", line, column);
      }
      i++;
      column++;
      tokens.push({ type: "STRING", lexeme: value, value, ...start });
      continue;
    }

    if (ch === "0" && (source[i + 1] === "x" || source[i + 1] === "X")) {
      i += 2;
      column += 2;
      let hexStr = "";
      while (i < source.length && isHexDigit(source[i])) {
        hexStr += source[i];
        i++;
        column++;
      }
      const num = parseInt(hexStr, 16);
      tokens.push({ type: "NUMBER", lexeme: `0x${hexStr}`, value: num, ...start });
      continue;
    }

    if (isDigit(ch) || (ch === "." && isDigit(source[i + 1]))) {
      let numStr = "";
      while (i < source.length && (isDigit(source[i]) || source[i] === ".")) {
        numStr += source[i];
        i++;
        column++;
      }
      const num = parseFloat(numStr);

      let matchedUnit: UnitLexeme | undefined;
      while (i < source.length && (source[i] === " " || source[i] === "\t")) {
        i++;
        column++;
      }
      for (const suffix of UNIT_SUFFIXES) {
        if (source.slice(i, i + suffix.length) === suffix) {
          const next = source[i + suffix.length];
          if (next && (isIdentChar(next) || next === "/")) continue;
          matchedUnit = suffix;
          i += suffix.length;
          column += suffix.length;
          break;
        }
      }

      if (matchedUnit) {
        tokens.push({
          type: "UNIT_LITERAL",
          lexeme: `${numStr}${matchedUnit}`,
          value: num,
          unit: matchedUnit,
          ...start,
        });
      } else {
        tokens.push({ type: "NUMBER", lexeme: numStr, value: num, ...start });
      }
      continue;
    }

    if (isIdentStart(ch)) {
      let ident = "";
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
  return isDigit(ch) || (ch >= "a" && ch <= "f") || (ch >= "A" && ch <= "F");
}

function isDigit(ch: string): boolean {
  return ch >= "0" && ch <= "9";
}

function isIdentStart(ch: string): boolean {
  return (ch >= "a" && ch <= "z") || (ch >= "A" && ch <= "Z") || ch === "_";
}

function isIdentChar(ch: string): boolean {
  return isIdentStart(ch) || isDigit(ch);
}

export function unitFromLexeme(lexeme: UnitLexeme): import("../ast/nodes.js").UnitKind {
  if (lexeme === "m/s2" || lexeme === "m/s²") return "m/s²";
  if (lexeme === "cd/m2") return "cd/m²";
  if (lexeme === "µg/m³") return "ug/m3";
  return lexeme;
}
