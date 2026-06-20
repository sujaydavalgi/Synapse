import type { RuntimeValue } from "../runtime/interpreter.js";
import {
  InMemoryCommBus,
  type DiscoverFilter,
  type DiscoverTarget,
  type PublishedCommMessage,
  type SimNetworkConfig,
  type TransportKind,
  transportAsStr,
} from "../comm/index.js";

export type { TransportKind };

export type TransportConfig = {
  brokerUrl?: string | null;
  nodeName?: string | null;
  namespace?: string | null;
  domainId?: number | null;
  clientId?: string | null;
};

export type AdapterMessage = {
  topic: string;
  messageType: string;
  value: RuntimeValue;
};

export interface TransportAdapter {
  kind(): TransportKind;
  connect(config: TransportConfig): void;
  disconnect(): void;
  isConnected(): boolean;
  publish(topic: string, messageType: string, value: RuntimeValue): void;
  subscribe(topic: string): void;
  receive(topic: string): RuntimeValue | null;
  callService(service: string, serviceType: string, request?: RuntimeValue | null): RuntimeValue;
  sendAction(action: string, actionType: string, goal: RuntimeValue): RuntimeValue;
  published(): AdapterMessage[];
}

type StubState = {
  connected: boolean;
  config: TransportConfig;
  subscriptions: Map<string, RuntimeValue[]>;
  published: AdapterMessage[];
};

function createStubAdapter(kind: TransportKind): TransportAdapter {
  const state: StubState = {
    connected: false,
    config: {},
    subscriptions: new Map(),
    published: [],
  };

  return {
    kind: () => kind,
    connect(config) {
      state.connected = true;
      state.config = config;
    },
    disconnect() {
      state.connected = false;
    },
    isConnected: () => state.connected,
    publish(topic, messageType, value) {
      if (!state.connected) return;
      state.published.push({ topic, messageType, value });
      const buf = state.subscriptions.get(topic);
      if (buf) buf.push(value);
    },
    subscribe(topic) {
      if (!state.connected) return;
      if (!state.subscriptions.has(topic)) state.subscriptions.set(topic, []);
    },
    receive(topic) {
      if (!state.connected) return null;
      const buf = state.subscriptions.get(topic);
      return buf?.shift() ?? null;
    },
    callService(_service, serviceType) {
      return {
        kind: "object",
        typeName: serviceType,
        fields: { ok: { kind: "bool", value: true } },
      };
    },
    sendAction(_action, actionType) {
      return {
        kind: "object",
        typeName: actionType,
        fields: { success: { kind: "bool", value: true } },
      };
    },
    published: () => [...state.published],
  };
}

export class RoutingCommBus {
  private memory = new InMemoryCommBus();
  private ros2 = createStubAdapter("ros2");
  private mqtt = createStubAdapter("mqtt");
  private dds = createStubAdapter("dds");
  private websocket = createStubAdapter("websocket");

  configure(config: TransportConfig): void {
    this.ros2.connect({ nodeName: config.nodeName, namespace: config.namespace, ...config });
    this.mqtt.connect({
      brokerUrl: config.brokerUrl ?? "mqtt://localhost:1883",
      clientId: config.clientId ?? "spanda",
      ...config,
    });
    this.dds.connect({ domainId: config.domainId ?? 0, ...config });
    this.websocket.connect({
      brokerUrl: config.brokerUrl ?? "ws://localhost:9090",
      ...config,
    });
  }

  private adapter(transport: TransportKind): TransportAdapter | null {
    switch (transport) {
      case "ros2":
        return this.ros2;
      case "mqtt":
        return this.mqtt;
      case "dds":
        return this.dds;
      case "websocket":
        return this.websocket;
      default:
        return null;
    }
  }

  registerRobot(name: string): void {
    this.memory.registerRobot(name);
  }

  registerAgent(name: string): void {
    this.memory.registerAgent(name);
  }

  registerDevice(name: string): void {
    this.memory.registerDevice(name);
  }

  publish(
    topicPath: string,
    messageType: string,
    value: RuntimeValue,
    transport: TransportKind,
  ): void {
    this.memory.publish(topicPath, messageType, value, transport);
    this.adapter(transport)?.publish(topicPath, messageType, value);
  }

  subscribe(topicPath: string, handler: string): void {
    this.memory.subscribe(topicPath, handler);
  }

  receive(topicPath: string): RuntimeValue | null {
    return this.memory.receive(topicPath);
  }

  callService(serviceType: string): RuntimeValue {
    return this.memory.callService(serviceType);
  }

  sendAction(actionType: string): RuntimeValue {
    return this.memory.sendAction(actionType);
  }

  publishPeer(
    peer: string,
    topic: string,
    value: RuntimeValue,
    transport: TransportKind,
  ): void {
    this.memory.publishPeer(peer, topic, value, transport);
  }

  discover(target: DiscoverTarget, filter: DiscoverFilter): string[] {
    return this.memory.discover(target, filter);
  }

  publishedMessages(): PublishedCommMessage[] {
    return this.memory.publishedMessages();
  }

  injectFault(fault: string): void {
    this.memory.injectFault(fault);
  }

  adapterPublished(transport: TransportKind): AdapterMessage[] {
    return this.adapter(transport)?.published() ?? [];
  }
}

export { transportAsStr };
