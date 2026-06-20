import type { FieldDecl, StructDecl } from "../foundations.js";
import type { SpandaType, Span } from "../ast/nodes.js";
import type { RuntimeValue } from "../runtime/values.js";

export type TransportKind = "local" | "ros2" | "mqtt" | "dds" | "websocket" | "sim";

export function transportFromIdent(s: string): TransportKind | null {
  switch (s) {
    case "local":
      return "local";
    case "ros2":
      return "ros2";
    case "mqtt":
      return "mqtt";
    case "dds":
      return "dds";
    case "websocket":
      return "websocket";
    case "sim":
      return "sim";
    default:
      return null;
  }
}

export function transportAsStr(t: TransportKind): string {
  return t;
}

export type QosReliability = "reliable" | "best_effort";
export type TopicRole = "publish" | "subscribe" | "both";

export type QosDecl = {
  reliability: QosReliability | null;
  rateHz: number | null;
  deadlineMs: number | null;
  history: string | null;
  span: Span;
};

export type MessageDecl = {
  kind: "MessageDecl";
  name: string;
  fields: FieldDecl[];
  version: number | null;
  span: Span;
};

export type MessageSchema = {
  name: string;
  fields: [string, string][];
  version: number | null;
};

export type BusDecl = {
  kind: "BusDecl";
  name: string;
  transport: TransportKind;
  span: Span;
};

export type PeerRobotDecl = {
  kind: "PeerRobotDecl";
  name: string;
  span: Span;
};

export type DeviceDecl = {
  kind: "DeviceDecl";
  name: string;
  deviceType: string;
  span: Span;
};

export type AgentChannelDecl = {
  kind: "AgentChannelDecl";
  fromAgent: string;
  toAgent: string;
  messageType: string;
  span: Span;
};

export type TwinSyncDecl = {
  kind: "TwinSyncDecl";
  telemetry: boolean;
  replay: boolean;
  faults: boolean;
  events: boolean;
  span: Span;
};

export type DiscoverTarget = "robots" | "agents" | "devices";

export type DiscoverFilter = {
  capability: string | null;
};

export type PublishedCommMessage = {
  topicPath: string;
  messageType: string;
  value: RuntimeValue;
  transport: TransportKind;
};

export type SimNetworkConfig = {
  delayMs: number;
  packetLoss: number;
};

export class MessageRegistry {
  private schemas = new Map<string, MessageSchema>();
  private builtin = new Set(["Velocity", "Pose", "Scan", "String"]);

  static new(): MessageRegistry {
    return new MessageRegistry();
  }

  register(decl: MessageDecl): void {
    this.schemas.set(decl.name, {
      name: decl.name,
      fields: decl.fields.map((f) => [f.name, f.typeName]),
      version: decl.version,
    });
  }

  static fromProgram(messages: MessageDecl[], structs: StructDecl[]): MessageRegistry {
    const reg = MessageRegistry.new();
    for (const msg of messages) reg.register(msg);
    for (const s of structs) {
      reg.schemas.set(s.name, {
        name: s.name,
        fields: s.fields.map((f) => [f.name, f.typeName]),
        version: null,
      });
    }
    return reg;
  }

  isKnown(name: string): boolean {
    return this.builtin.has(name) || this.schemas.has(name);
  }

  resolveType(name: string): SpandaType | null {
    switch (name) {
      case "Velocity":
        return { kind: "velocity" };
      case "Pose":
        return { kind: "pose" };
      case "Scan":
        return { kind: "scan" };
      case "String":
        return { kind: "string" };
      case "Command":
      case "Conversation":
      case "Feedback":
      case "Approval":
      case "Intent":
      case "SafeMessage":
      case "VerifiedMessage":
      case "TrustedSource":
      case "ActionProposal":
      case "SafeAction":
      case "CommandMessage":
      case "BatteryRequest":
      case "BatteryStatus":
      case "NavigationFeedback":
      case "NavigationResult":
      case "LidarReading":
      case "LidarScan":
      case "Timestamp":
      case "PathPlan":
        return { kind: "named", name };
      default:
        if (this.schemas.has(name)) return { kind: "named", name };
        return null;
    }
  }
}

export class InMemoryCommBus {
  private subscriptions = new Map<string, string[]>();
  private buffers = new Map<string, RuntimeValue[]>();
  private published: PublishedCommMessage[] = [];
  private discoveredRobots = ["RoverA", "RoverB"];
  private discoveredAgents = ["Vision", "Planner", "Navigator"];
  private discoveredDevices = ["Camera", "IMU", "Lidar"];
  private network: SimNetworkConfig = { delayMs: 0, packetLoss: 0 };
  private faults: string[] = [];

  publish(
    topicPath: string,
    messageType: string,
    value: RuntimeValue,
    transport: TransportKind,
  ): void {
    if (this.faults.includes("NetworkOutage")) return;
    if (this.network.packetLoss > 0) {
      const hash = topicPath.length + messageType.length;
      if (((hash * 0.13) % 1) < this.network.packetLoss) return;
    }
    this.published.push({ topicPath, messageType, value, transport });
    const buf = this.buffers.get(topicPath);
    if (buf) buf.push(value);
  }

  subscribe(topicPath: string, handler: string): void {
    const subs = this.subscriptions.get(topicPath) ?? [];
    subs.push(handler);
    this.subscriptions.set(topicPath, subs);
    if (!this.buffers.has(topicPath)) this.buffers.set(topicPath, []);
  }

  receive(topicPath: string): RuntimeValue | null {
    const buf = this.buffers.get(topicPath);
    return buf?.shift() ?? null;
  }

  callService(serviceType: string): RuntimeValue {
    return {
      kind: "object",
      typeName: serviceType,
      fields: { ok: { kind: "bool", value: true } },
    };
  }

  sendAction(actionType: string): RuntimeValue {
    return {
      kind: "object",
      typeName: actionType,
      fields: { success: { kind: "bool", value: true } },
    };
  }

  discover(target: DiscoverTarget, filter: DiscoverFilter): string[] {
    const base =
      target === "robots"
        ? this.discoveredRobots
        : target === "agents"
          ? this.discoveredAgents
          : this.discoveredDevices;
    if (filter.capability) {
      const cap = filter.capability.toLowerCase();
      return base.filter((n) => n.toLowerCase().includes(cap));
    }
    return [...base];
  }

  registerRobot(name: string): void {
    this.discoveredRobots.push(name);
  }

  registerAgent(name: string): void {
    this.discoveredAgents.push(name);
  }

  registerDevice(name: string): void {
    this.discoveredDevices.push(name);
  }

  publishPeer(
    peer: string,
    topic: string,
    value: RuntimeValue,
    transport: TransportKind,
  ): void {
    const path = `/${peer}/${topic}`;
    this.publish(path, "PeerMessage", value, transport);
  }

  publishedMessages(): PublishedCommMessage[] {
    return [...this.published];
  }

  injectFault(fault: string): void {
    this.faults.push(fault);
  }
}

export const COMM_CAPABILITIES = ["subscribe", "publish", "call", "execute", "discover"] as const;

export function isCommCapability(action: string): boolean {
  return (COMM_CAPABILITIES as readonly string[]).includes(action);
}

export function estimateTopicBandwidthMbps(rateHz: number, messageSizeBytes: number): number {
  return (rateHz * messageSizeBytes * 8) / 1_000_000;
}

export function defaultMessageSize(messageType: string): number {
  switch (messageType) {
    case "Scan":
    case "LidarScan":
    case "LidarReading":
      return 64_000;
    case "Pose":
    case "Velocity":
      return 128;
    case "PathPlan":
    case "NavigationFeedback":
      return 4_096;
    default:
      return 512;
  }
}
