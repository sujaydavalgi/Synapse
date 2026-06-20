import { RuntimeError } from "./runtime/interpreter.js";
import type { RuntimeValue } from "./runtime/interpreter.js";

export type SpawnHandle = {
  funcName: string;
  args: RuntimeValue[];
  result: RuntimeValue | null;
};

export type AgentRoute = {
  from: string;
  to: string;
  messageType: string;
};

function runtimeTypeTag(value: RuntimeValue): string {
  switch (value.kind) {
    case "object":
      return `object:${value.typeName}`;
    case "enum":
      return `enum:${value.enumName}::${value.variant}`;
    case "number":
      return `number:${value.unit}`;
    case "string":
      return "string";
    case "bool":
      return "bool";
    case "pose":
      return "pose";
    case "channel":
      return "channel";
    case "task_handle":
      return "task_handle";
    case "future":
      return "future";
    default:
      return value.kind;
  }
}

export class ConcurrencyRuntime {
  private nextChannelId = 1;
  private channels = new Map<number, RuntimeValue[]>();
  private channelTypeTags = new Map<number, string>();
  private nextHandleId = 1;
  private handles = new Map<number, SpawnHandle>();
  private fireAndForgetQueue: number[] = [];
  private agentInboxes = new Map<string, RuntimeValue[]>();
  private agentRoutes: AgentRoute[] = [];

  createChannel(): RuntimeValue {
    const id = this.nextChannelId++;
    this.channels.set(id, []);
    return { kind: "channel", id };
  }

  bindChannelType(channel: RuntimeValue, value: RuntimeValue, line: number): void {
    if (channel.kind !== "channel") {
      throw new RuntimeError("channel type binding requires channel", line);
    }
    const next = runtimeTypeTag(value);
    const existing = this.channelTypeTags.get(channel.id);
    if (existing && existing !== next) {
      throw new RuntimeError(`Channel type mismatch: expected ${existing}, got ${next}`, line);
    }
    this.channelTypeTags.set(channel.id, next);
  }

  send(channel: RuntimeValue, value: RuntimeValue, line: number): void {
    if (channel.kind !== "channel") {
      throw new RuntimeError("send requires a channel", line);
    }
    const buf = this.channels.get(channel.id);
    if (!buf) {
      throw new RuntimeError(`Unknown channel id ${channel.id}`, line);
    }
    const expected = this.channelTypeTags.get(channel.id);
    if (expected) {
      const actual = runtimeTypeTag(value);
      if (expected !== actual) {
        throw new RuntimeError(`Channel type mismatch: expected ${expected}, got ${actual}`, line);
      }
    }
    buf.push(value);
  }

  tryRecv(channel: RuntimeValue, line: number): RuntimeValue | null {
    if (channel.kind !== "channel") {
      throw new RuntimeError("recv requires a channel", line);
    }
    const buf = this.channels.get(channel.id);
    if (!buf) {
      throw new RuntimeError(`Unknown channel id ${channel.id}`, line);
    }
    return buf.shift() ?? null;
  }

  createTaskHandle(funcName: string, args: RuntimeValue[]): RuntimeValue {
    const id = this.nextHandleId++;
    this.handles.set(id, { funcName, args, result: null });
    return { kind: "task_handle", id };
  }

  queueFireAndForget(funcName: string, args: RuntimeValue[]): void {
    const handle = this.createTaskHandle(funcName, args);
    if (handle.kind === "task_handle") {
      this.fireAndForgetQueue.push(handle.id);
    }
  }

  getHandle(id: number): SpawnHandle | undefined {
    return this.handles.get(id);
  }

  setHandleResult(id: number, result: RuntimeValue): void {
    const handle = this.handles.get(id);
    if (handle) handle.result = result;
  }

  drainFireAndForgetQueue(): number[] {
    const queue = [...this.fireAndForgetQueue];
    this.fireAndForgetQueue = [];
    return queue;
  }

  registerAgentRoute(from: string, to: string, messageType: string): void {
    this.agentRoutes.push({ from, to, messageType });
  }

  sendAgent(from: string, to: string, value: RuntimeValue, line: number): void {
    const allowed = this.agentRoutes.some((route) => route.from === from && route.to === to);
    if (!allowed) {
      throw new RuntimeError(`No agent channel from '${from}' to '${to}'`, line);
    }
    const route = this.agentRoutes.find((r) => r.from === from && r.to === to);
    if (route?.messageType) {
      const actual = runtimeTypeTag(value);
      const expected = `object:${route.messageType}`;
      if (actual !== expected && actual !== route.messageType) {
        throw new RuntimeError(
          `Agent message type mismatch: expected ${route.messageType}, got ${actual}`,
          line,
        );
      }
    }
    const inbox = this.agentInboxes.get(to) ?? [];
    inbox.push(value);
    this.agentInboxes.set(to, inbox);
  }

  tryRecvAgent(agent: string): RuntimeValue | null {
    const inbox = this.agentInboxes.get(agent);
    return inbox?.shift() ?? null;
  }
}
