/**
 * Unified trigger registry mirroring the Rust `spanda-runtime` trigger model.
 * @module
 */

import type { Expr, Span, SpandaType, Stmt } from "../ast/nodes.js";
import type { EventHandlerDecl } from "../foundations.js";
import type { TaskPriority } from "../foundations.js";

export const MAX_TRIGGERS_PER_TICK = 64;

export type TriggerKind =
  | { kind: "event"; name: string }
  | { kind: "message"; topic: string }
  | { kind: "timer"; interval_ms: number }
  | { kind: "condition"; expr: Expr; level: boolean }
  | { kind: "hardware"; event: string }
  | { kind: "connectivity"; domain: string; event: string }
  | { kind: "geofence"; name: string; phase: string }
  | { kind: "kill_switch"; name: string }
  | { kind: "sensor_event"; sensor: string; event: string }
  | { kind: "message_match"; field: string }
  | { kind: "log_match" };

export type TriggerHandlerDecl = {
  kind: "TriggerHandlerDecl";
  triggerKind: TriggerKind;
  priority: TaskPriority;
  returnType: SpandaType;
  body: Stmt[];
  span: Span;
};

export type RegisteredTrigger = {
  id: number;
  name: string;
  kind: TriggerKind;
  priority: TaskPriority;
  body: Stmt[];
  agent?: string;
};

export type TriggerTimerSchedule = {
  trigger_id: number;
  interval_ms: number;
  next_due_ms: number;
};

export function priorityRank(priority: TaskPriority): number {
  switch (priority) {
    case "critical":
      return 0;
    case "high":
      return 1;
    case "normal":
      return 2;
    case "low":
      return 3;
  }
}

export function triggerDisplayName(kind: TriggerKind, agent?: string): string {
  let base: string;
  switch (kind.kind) {
    case "event":
      base = `event:${kind.name}`;
      break;
    case "message":
      base = `message:${kind.topic}`;
      break;
    case "timer":
      base = `timer:${kind.interval_ms}ms`;
      break;
    case "condition":
      base = kind.level ? "while" : "when";
      break;
    case "hardware":
      base = `hardware:${kind.event}`;
      break;
    case "connectivity":
      base = `connectivity:${kind.domain}.${kind.event}`;
      break;
    case "geofence":
      base = `geofence:${kind.name}:${kind.phase}`;
      break;
    case "kill_switch":
      base = `kill_switch:${kind.name}`;
      break;
    case "sensor_event":
      base = `sensor:${kind.sensor}.${kind.event}`;
      break;
    case "message_match":
      base = `message_match:${kind.field}`;
      break;
    case "log_match":
      base = "log_match";
      break;
  }
  return agent ? `${agent}/${base}` : base;
}

export function triggerCategoryLabel(kind: TriggerKind): string {
  switch (kind.kind) {
    case "event":
      return "event";
    case "message":
      return "message";
    case "timer":
      return "timer";
    case "condition":
      return "condition";
    case "hardware":
      return "hardware";
    case "connectivity":
      return "connectivity";
    case "geofence":
      return "geofence";
    case "kill_switch":
      return "kill_switch";
    case "sensor_event":
      return "sensor_event";
    case "message_match":
      return "message_match";
    case "log_match":
      return "log_match";
  }
}

export function triggerKindFromLegacyEventName(eventName: string): TriggerKind {
  if (eventName.startsWith("message:")) {
    return { kind: "message", topic: eventName.slice("message:".length) };
  }
  if (eventName.startsWith("geofence:")) {
    const parts = eventName.split(":");
    return { kind: "geofence", name: parts[1] ?? "", phase: parts[2] ?? "" };
  }
  if (eventName.startsWith("kill_switch:")) {
    return { kind: "kill_switch", name: eventName.slice("kill_switch:".length) };
  }
  if (eventName.startsWith("hardware.")) {
    return { kind: "hardware", event: eventName.slice("hardware.".length) };
  }
  if (eventName.startsWith("message.")) {
    return { kind: "message_match", field: eventName };
  }
  if (eventName === "log") {
    return { kind: "log_match" };
  }
  const dot = eventName.indexOf(".");
  if (dot > 0) {
    const domain = eventName.slice(0, dot).toLowerCase();
    const event = eventName.slice(dot + 1).toLowerCase();
    if (
      domain === "network" ||
      domain === "cellular" ||
      domain === "bluetooth" ||
      domain === "wifi" ||
      domain === "gps" ||
      domain === "gnss"
    ) {
      return { kind: "connectivity", domain, event };
    }
    if (event === "fix" || event === "reading" || event === "update") {
      return { kind: "sensor_event", sensor: eventName.slice(0, dot), event };
    }
    return { kind: "connectivity", domain, event };
  }
  return { kind: "event", name: eventName };
}

export function triggerHandlerFromLegacy(handler: EventHandlerDecl): TriggerHandlerDecl {
  return {
    kind: "TriggerHandlerDecl",
    triggerKind: triggerKindFromLegacyEventName(handler.eventName),
    priority: "normal",
    returnType: handler.returnType,
    body: handler.body,
    span: handler.span,
  };
}

export class ConditionTriggerState {
  private wasActive = new Set<number>();

  shouldFire(triggerId: number, active: boolean): boolean {
    const was = this.wasActive.has(triggerId);
    if (active) {
      this.wasActive.add(triggerId);
      return !was;
    }
    this.wasActive.delete(triggerId);
    return false;
  }

  clear(): void {
    this.wasActive.clear();
  }
}

export class TriggerRegistry {
  private handlers: RegisteredTrigger[] = [];
  private eventIndex = new Map<string, number>();
  private nextId = 0;

  clear(): void {
    this.handlers = [];
    this.eventIndex.clear();
    this.nextId = 0;
  }

  register(decl: TriggerHandlerDecl, agent?: string): void {
    const name = triggerDisplayName(decl.triggerKind, agent);
    const id = this.nextId++;
    if (decl.triggerKind.kind === "event") {
      this.eventIndex.set(decl.triggerKind.name, id);
    }
    this.handlers.push({
      id,
      name,
      kind: decl.triggerKind,
      priority: decl.priority,
      body: decl.body,
      agent,
    });
  }

  registerLegacyEvent(handler: EventHandlerDecl): void {
    this.register(triggerHandlerFromLegacy(handler));
  }

  get(id: number): RegisteredTrigger | undefined {
    return this.handlers.find((handler) => handler.id === id);
  }

  handlersForEvent(eventName: string): RegisteredTrigger[] {
    return this.handlers.filter(
      (handler) => handler.kind.kind === "event" && handler.kind.name === eventName,
    );
  }

  handlersForMessage(topicName: string, topicPath: string): RegisteredTrigger[] {
    return this.handlers.filter((handler) => {
      if (handler.kind.kind !== "message") {
        return false;
      }
      const topic = handler.kind.topic;
      return (
        topic === topicName ||
        topic === topicPath ||
        topicPath.endsWith(`/${topic}`) ||
        `/${topic}` === topicPath
      );
    });
  }

  handlersForConnectivity(domain: string, event: string): RegisteredTrigger[] {
    return this.handlers.filter(
      (handler) =>
        handler.kind.kind === "connectivity" &&
        handler.kind.domain === domain &&
        handler.kind.event === event,
    );
  }

  timerHandlers(): RegisteredTrigger[] {
    return this.handlers.filter((handler) => handler.kind.kind === "timer");
  }

  conditionHandlers(): RegisteredTrigger[] {
    return this.handlers.filter((handler) => handler.kind.kind === "condition");
  }

  static sortedByPriority(handlers: RegisteredTrigger[]): RegisteredTrigger[] {
    return [...handlers].sort((left, right) => priorityRank(left.priority) - priorityRank(right.priority));
  }
}

export function timerScheduleFromHandler(handler: RegisteredTrigger): TriggerTimerSchedule | null {
  if (handler.kind.kind !== "timer") {
    return null;
  }
  return {
    trigger_id: handler.id,
    interval_ms: handler.kind.interval_ms,
    next_due_ms: 0,
  };
}
