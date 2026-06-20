import { describe, expect, it } from "vitest";
import { tokenize } from "../src/lexer/index.js";
import { parse } from "../src/parser/index.js";
import { typeCheck } from "../src/types/index.js";
import { RoutingCommBus } from "../src/transport/index.js";
import { resolveModuleImport } from "../src/foundations.js";
import { isStdNetworkType, resolveStdNetworkImport } from "../src/network/index.js";
import { resolveTypeName } from "../src/type-system.js";
import {
  buildSymbolIndex,
  lookupDefinition,
  resolveDefinition,
} from "../src/lsp/symbols.js";

describe("transport adapters", () => {
  it("routes ros2 publish to adapter and memory bus", () => {
    const bus = new RoutingCommBus();
    bus.configure({ nodeName: "TestBot" });
    bus.publish("/scan", "Scan", { kind: "scan", nearestDistance: 1.5 }, "ros2");
    expect(bus.publishedMessages()).toHaveLength(1);
    expect(bus.adapterPublished("ros2")).toHaveLength(1);
    expect(bus.adapterPublished("ros2")[0]?.topic).toBe("/scan");
  });

  it("sim transport stays in memory only", () => {
    const bus = new RoutingCommBus();
    bus.publish("/local", "String", { kind: "string", value: "hi" }, "sim");
    expect(bus.publishedMessages()).toHaveLength(1);
    expect(bus.adapterPublished("ros2")).toHaveLength(0);
  });
});

describe("std.network namespace", () => {
  it("registers import path", () => {
    expect(resolveStdNetworkImport("std.network")).toBe(true);
    expect(resolveModuleImport("std.network")).toBe(true);
  });

  it("resolves network types", () => {
    expect(resolveTypeName("std.network.QosProfile")).toEqual({ kind: "named", name: "QosProfile" });
    expect(resolveTypeName("Bandwidth")).toEqual({ kind: "named", name: "Bandwidth" });
    expect(isStdNetworkType("Transport")).toBe(true);
  });

  it("type-checks program importing std.network", () => {
    const source = `import std.network;

message NetPayload {
  profile: QosProfile;
  path: TopicPath;
}

robot NetBot {
  topic data: NetPayload publish on "/data";
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  behavior run() { }
}`;
    expect(() => typeCheck(parse(tokenize(source)))).not.toThrow();
  });
});

describe("LSP symbol index", () => {
  const source = `message Alert { text: String; }

robot Bot {
  topic alerts: Alert publish on "/alerts";
  service Reset { request String; response String; };
  sensor lidar: Lidar on "/scan";
  agent Worker { goal "x"; plan { } }
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  behavior run() { publish alerts with lidar.read(); }
}`;

  it("indexes messages, topics, and agents", () => {
    const index = buildSymbolIndex(parse(tokenize(source)));
    expect(index.symbols.some((s) => s.kind === "message" && s.name === "Alert")).toBe(true);
    expect(index.symbols.some((s) => s.kind === "topic" && s.name === "alerts")).toBe(true);
    expect(index.symbols.some((s) => s.kind === "agent" && s.name === "Worker")).toBe(true);
  });

  it("resolves definition by name", () => {
    const sym = lookupDefinition(buildSymbolIndex(parse(tokenize(source))), "Alert", "message");
    expect(sym?.kind).toBe("message");
  });

  it("resolveDefinition finds topic reference", () => {
    const lines = source.split("\n");
    const runLine = lines.findIndex((l) => l.includes("publish alerts")) + 1;
    const col = lines[runLine - 1]!.indexOf("alerts") + 1;
    const sym = resolveDefinition(source, runLine, col);
    expect(sym?.kind).toBe("topic");
    expect(sym?.name).toBe("alerts");
  });

  it("indexes struct and enum declarations", () => {
    const source = `
struct Box<T> { value: T; }
enum Mode { Idle, Active }
robot R { actuator wheels: DifferentialDrive; behavior run() { wheels.stop(); } }
`;
    const index = buildSymbolIndex(parse(tokenize(source)));
    expect(index.symbols.some((s) => s.kind === "struct" && s.name === "Box")).toBe(true);
    expect(index.symbols.some((s) => s.kind === "enum" && s.name === "Mode")).toBe(true);
  });

  it("indexes hardware profiles and deploy targets", () => {
    const hwSource = `
hardware Board { memory: 2 GB; }
robot R { actuator wheels: DifferentialDrive; behavior run() { wheels.stop(); } }
deploy R to Board;
`;
    const index = buildSymbolIndex(parse(tokenize(hwSource)));
    expect(index.symbols.some((s) => s.kind === "hardware" && s.name === "Board")).toBe(true);
    expect(index.symbols.some((s) => s.kind === "deploy")).toBe(true);
  });
});
