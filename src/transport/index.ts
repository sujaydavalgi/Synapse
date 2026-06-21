/**
 * index module (transport/index.ts).
 * @module
 */

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
  // CreateStubAdapter.
  //
  // Parameters:
  // - `kind` — input value
  //
  // Returns:
  // `TransportAdapter`.
  //
  // Options:
  // None.
  //
  // Example:

  // const result = createStubAdapter(kind);
  const state: StubState = {
    connected: false,
    config: {},
    subscriptions: new Map(),
    published: [],
  };
  return {
    kind: () => kind,
    connect(config) {
      //
      // Parameters:
      // - `config` — input value
      //
      // Returns:
      //
      // Options:
      // None.
      //
      // Example:
      state.connected = true;
      state.config = config;
    },
    disconnect() {
      //
      // Parameters:
      // None.
      //
      // Returns:
      //
      // Options:
      // None.
      //
      // Example:
      state.connected = false;
    },
    isConnected: () => state.connected,
    publish(topic, messageType, value) {
      //
      // Parameters:
      // - `topic` — input value
      // - `messageType` — input value
      // - `value` — input value
      //
      // Returns:
      //
      // Options:
      // None.
      //
      // Example:

      // continue when connected is falsy.
      if (!state.connected) return;
      state.published.push({ topic, messageType, value });
      const buf = state.subscriptions.get(topic);

      // continue when buf) buf.push(value.
      if (buf) buf.push(value);
    },
    subscribe(topic) {
      //
      // Parameters:
      // - `topic` — input value
      //
      // Returns:
      //
      // Options:
      // None.
      //
      // Example:

      // continue when connected is falsy.
      if (!state.connected) return;

      // continue when set is falsy.
      if (!state.subscriptions.has(topic)) state.subscriptions.set(topic, []);
    },
    receive(topic) {
      //
      // Parameters:
      // - `topic` — input value
      //
      // Returns:
      //
      // Options:
      // None.
      //
      // Example:

      // continue when connected is falsy.
      if (!state.connected) return null;
      const buf = state.subscriptions.get(topic);
      return buf?.shift() ?? null;
    },
    callService(_service, serviceType) {
      //
      // Parameters:
      // - `_service` — input value
      // - `serviceType` — input value
      //
      // Returns:
      //
      // Options:
      // None.
      //
      // Example:
      return {
        kind: "object",
        typeName: serviceType,
        fields: { ok: { kind: "bool", value: true } },
      };
    },
    sendAction(_action, actionType) {
      //
      // Parameters:
      // - `_action` — input value
      // - `actionType` — input value
      //
      // Returns:
      //
      // Options:
      // None.
      //
      // Example:
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
    // Configure.
    //
    // Parameters:
    // - `config` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = configure(config);

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
    // Adapter.
    //
    // Parameters:
    // - `transport` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = adapter(transport);
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
    // RegisterRobot.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = registerRobot(name);

    this.memory.registerRobot(name);
  }

  registerAgent(name: string): void {
    // RegisterAgent.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = registerAgent(name);

    this.memory.registerAgent(name);
  }

  registerDevice(name: string): void {
    // RegisterDevice.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = registerDevice(name);

    this.memory.registerDevice(name);
  }

  publish(
    topicPath: string,
    messageType: string,
    value: RuntimeValue,
    transport: TransportKind,
  ): void {
    // Publish.
    //
    // Parameters:
    // - `topicPath` — input value
    // - `messageType` — input value
    // - `value` — input value
    // - `transport` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = publish(topicPath, messageType, value, transport);

    this.memory.publish(topicPath, messageType, value, transport);
    this.adapter(transport)?.publish(topicPath, messageType, value);
  }

  subscribe(topicPath: string, handler: string): void {
    // Subscribe.
    //
    // Parameters:
    // - `topicPath` — input value
    // - `handler` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = subscribe(topicPath, handler);

    this.memory.subscribe(topicPath, handler);
  }

  receive(topicPath: string): RuntimeValue | null {
    // Receive.
    //
    // Parameters:
    // - `topicPath` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = receive(topicPath);

    return this.memory.receive(topicPath);
  }

  callService(serviceType: string): RuntimeValue {
    // CallService.
    //
    // Parameters:
    // - `serviceType` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = callService(serviceType);

    return this.memory.callService(serviceType);
  }

  sendAction(actionType: string): RuntimeValue {
    // SendAction.
    //
    // Parameters:
    // - `actionType` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = sendAction(actionType);

    return this.memory.sendAction(actionType);
  }

  publishPeer(
    peer: string,
    topic: string,
    value: RuntimeValue,
    transport: TransportKind,
  ): void {
    // PublishPeer.
    //
    // Parameters:
    // - `peer` — input value
    // - `topic` — input value
    // - `value` — input value
    // - `transport` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = publishPeer(peer, topic, value, transport);

    this.memory.publishPeer(peer, topic, value, transport);
  }

  discover(target: DiscoverTarget, filter: DiscoverFilter): string[] {
    // Discover.
    //
    // Parameters:
    // - `target` — input value
    // - `filter` — input value
    //
    // Returns:
    // string[].
    //
    // Options:
    // None.
    //
    // Example:

    // const result = discover(target, filter);

    return this.memory.discover(target, filter);
  }

  publishedMessages(): PublishedCommMessage[] {
    // PublishedMessages.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // PublishedCommMessage[].
    //
    // Options:
    // None.
    //
    // Example:

    // const result = publishedMessages();

    return this.memory.publishedMessages();
  }

  injectFault(fault: string): void {
    // InjectFault.
    //
    // Parameters:
    // - `fault` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // const result = injectFault(fault);

    this.memory.injectFault(fault);
  }

  activeFaults(): string[] {
    // Return active comm-bus simulation faults.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Fault name list.
    //
    // Options:
    // None.
    //
    // Example:
    // const faults = activeFaults();

    return this.memory.activeFaults();
  }

  adapterPublished(transport: TransportKind): AdapterMessage[] {
    // AdapterPublished.
    //
    // Parameters:
    // - `transport` — input value
    //
    // Returns:
    // AdapterMessage[].
    //
    // Options:
    // None.
    //
    // Example:

    // const result = adapterPublished(transport);

    return this.adapter(transport)?.published() ?? [];
  }
}

export { transportAsStr };
