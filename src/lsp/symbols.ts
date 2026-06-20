import type { Program, Span } from "../ast/nodes.js";
import { tokenize } from "../lexer/index.js";
import { parse } from "../parser/index.js";

export type SymbolKind =
  | "message"
  | "struct"
  | "enum"
  | "topic"
  | "service"
  | "action"
  | "robot"
  | "agent"
  | "sensor"
  | "actuator"
  | "event"
  | "behavior"
  | "bus"
  | "device"
  | "hardware"
  | "deploy";

export type SpandaSymbol = {
  name: string;
  kind: SymbolKind;
  span: Span;
  detail?: string;
  container?: string;
};

export type SymbolIndex = {
  symbols: SpandaSymbol[];
  byName: Map<string, SpandaSymbol[]>;
};

function addSymbol(index: SymbolIndex, sym: SpandaSymbol): void {
  index.symbols.push(sym);
  const existing = index.byName.get(sym.name) ?? [];
  existing.push(sym);
  index.byName.set(sym.name, existing);
}

export function buildSymbolIndex(program: Program): SymbolIndex {
  const index: SymbolIndex = { symbols: [], byName: new Map() };

  for (const msg of program.messages) {
    addSymbol(index, {
      name: msg.name,
      kind: "message",
      span: msg.span,
      detail: msg.fields.map((f) => `${f.name}: ${f.typeName}`).join(", "),
    });
  }

  for (const structDecl of program.structs) {
    const typeParams = structDecl.typeParams?.length
      ? `<${structDecl.typeParams.join(", ")}>`
      : "";
    addSymbol(index, {
      name: structDecl.name,
      kind: "struct",
      span: structDecl.span,
      detail: `${typeParams}{ ${structDecl.fields.map((f) => `${f.name}: ${f.typeName}`).join(", ")} }`,
    });
  }

  for (const enumDecl of program.enums) {
    addSymbol(index, {
      name: enumDecl.name,
      kind: "enum",
      span: enumDecl.span,
      detail: enumDecl.variants.map((v) => v.name).join(" | "),
    });
  }

  for (const profile of program.hardwareProfiles) {
    addSymbol(index, {
      name: profile.name,
      kind: "hardware",
      span: profile.span,
      detail: profile.cpu ?? undefined,
    });
  }

  for (const deploy of program.deployments) {
    addSymbol(index, {
      name: `${deploy.robotName}→${deploy.targets.join(",")}`,
      kind: "deploy",
      span: deploy.span,
      detail: deploy.targets.join(", "),
    });
  }

  for (const robot of program.robots) {
    addSymbol(index, { name: robot.name, kind: "robot", span: robot.span });

    for (const bus of robot.buses) {
      addSymbol(index, {
        name: bus.name,
        kind: "bus",
        span: bus.span,
        detail: bus.transport,
        container: robot.name,
      });
    }

    for (const device of robot.devices) {
      addSymbol(index, {
        name: device.name,
        kind: "device",
        span: device.span,
        detail: device.deviceType,
        container: robot.name,
      });
    }

    for (const topic of robot.topics) {
      addSymbol(index, {
        name: topic.name,
        kind: "topic",
        span: topic.span,
        detail: topic.messageType ?? undefined,
        container: robot.name,
      });
    }

    for (const service of robot.services) {
      addSymbol(index, {
        name: service.name,
        kind: "service",
        span: service.span,
        detail: service.serviceType ?? undefined,
        container: robot.name,
      });
    }

    for (const action of robot.actions) {
      addSymbol(index, {
        name: action.name,
        kind: "action",
        span: action.span,
        detail: action.actionType ?? undefined,
        container: robot.name,
      });
    }

    for (const sensor of robot.sensors) {
      addSymbol(index, {
        name: sensor.name,
        kind: "sensor",
        span: sensor.span,
        detail: sensor.sensorType,
        container: robot.name,
      });
    }

    for (const actuator of robot.actuators) {
      addSymbol(index, {
        name: actuator.name,
        kind: "actuator",
        span: actuator.span,
        detail: actuator.actuatorType,
        container: robot.name,
      });
    }

    for (const agent of robot.agents) {
      addSymbol(index, {
        name: agent.name,
        kind: "agent",
        span: agent.span,
        container: robot.name,
      });
    }

    for (const event of robot.events) {
      addSymbol(index, {
        name: event.name,
        kind: "event",
        span: event.span,
        container: robot.name,
      });
    }

    for (const behavior of robot.behaviors) {
      addSymbol(index, {
        name: behavior.name,
        kind: "behavior",
        span: behavior.span,
        container: robot.name,
      });
    }
  }

  return index;
}

export function indexSource(source: string): SymbolIndex {
  return buildSymbolIndex(parse(tokenize(source)));
}

export function symbolAtPosition(
  index: SymbolIndex,
  line: number,
  column: number,
): SpandaSymbol | null {
  for (const sym of index.symbols) {
    const { start, end } = sym.span;
    if (
      line >= start.line &&
      line <= end.line &&
      (line !== start.line || column >= start.column) &&
      (line !== end.line || column <= end.column)
    ) {
      return sym;
    }
  }
  return null;
}

export function lookupDefinition(
  index: SymbolIndex,
  name: string,
  kind?: SymbolKind,
): SpandaSymbol | null {
  const candidates = index.byName.get(name);
  if (!candidates?.length) return null;
  if (kind) {
    return candidates.find((c) => c.kind === kind) ?? candidates[0]!;
  }
  return candidates[0]!;
}

export function wordAtPosition(source: string, line: number, column: number): string | null {
  const lines = source.split("\n");
  const text = lines[line - 1];
  if (!text) return null;
  const col = Math.max(0, column - 1);
  if (col >= text.length) return null;

  const ident = /[A-Za-z_][A-Za-z0-9_]*/g;
  let match: RegExpExecArray | null;
  while ((match = ident.exec(text)) !== null) {
    const start = match.index;
    const end = start + match[0].length;
    if (col >= start && col <= end) {
      return match[0];
    }
  }
  return null;
}

const DEFINITION_KIND_PRIORITY: SymbolKind[] = [
  "topic",
  "service",
  "action",
  "message",
  "agent",
  "sensor",
  "actuator",
  "behavior",
  "event",
  "bus",
  "device",
  "robot",
];

export function resolveDefinition(
  source: string,
  line: number,
  column: number,
): SpandaSymbol | null {
  const index = indexSource(source);
  const word = wordAtPosition(source, line, column);
  if (word) {
    const candidates = index.byName.get(word);
    if (candidates?.length) {
      for (const kind of DEFINITION_KIND_PRIORITY) {
        const found = candidates.find((c) => c.kind === kind);
        if (found) return found;
      }
      return candidates[0]!;
    }
  }
  return symbolAtPosition(index, line, column);
}

export function formatHover(sym: SpandaSymbol): string {
  const header = `**${sym.kind}** \`${sym.name}\``;
  const parts = [header];
  if (sym.container) parts.push(`in robot \`${sym.container}\``);
  if (sym.detail) parts.push(sym.detail);
  return parts.join("\n\n");
}
