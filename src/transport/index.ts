/**
 * index module (transport/index.ts).
 * @module
 */

import type { RuntimeValue } from "../runtime/interpreter.js";
import {
  InMemoryCommBus,
  type CommEnvelope,
  type DiscoverFilter,
  type DiscoverTarget,
  type PublishedCommMessage,
  type SimNetworkConfig,
  type TransportKind,
  transportAsStr,
} from "../comm/index.js";
import {
  TlsTransportSession,
  defaultTransportSecurity,
  effectiveTransportPolicy,
  transportSecurityFromBusFields,
  urlRequiresTls,
  resolveBrokerUrl,
  type SecureCommPolicy,
  type TransportSecurityConfig,
} from "./transport-security.js";
import { decodeWireValue, encodeWireValue, type FullTransportConfig } from "./transport-wire.js";
import { LiveMqttBridge, liveMqttEnabled } from "./live-mqtt.js";
import { LiveWebsocketBridge, liveWebsocketEnabled } from "./live-websocket.js";
import { LiveDdsBridge, liveDdsEnabled } from "./live-dds.js";
import type { ProviderRegistry, TransportProvider } from "../providers/registry.js";
import { notifyProviderCall } from "../runtime/provider-observer.js";

export type { TransportKind, SecureCommPolicy, TransportSecurityConfig, FullTransportConfig };
export { TlsTransportSession, defaultTransportSecurity, transportSecurityFromBusFields, resolveBrokerUrl };

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
  config: Partial<FullTransportConfig>;
  subscriptions: Map<string, RuntimeValue[]>;
  published: AdapterMessage[];
};

/** Create an in-memory transport adapter stub for tests and package bootstrap. */
export function createTransportStub(kind: TransportKind): TransportProvider & TransportAdapter {
  // Description:
  //     CreateTransportStub.
  //
  // Inputs:
  //     kind: TransportKind
  //         Caller-supplied kind.
  //
  // Outputs:
  //     result: TransportProvider & TransportAdapter
  //         Return value from `createTransportStub`.
  //
  // Example:
  //     const result = createTransportStub(kind);

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

export type TransportConfig = FullTransportConfig;

export class RoutingCommBus {
  private memory = new InMemoryCommBus();
  private ros2 = createTransportStub("ros2");
  private mqtt = createTransportStub("mqtt");
  private dds = createTransportStub("dds");
  private websocket = createTransportStub("websocket");
  private config: FullTransportConfig = {
    security: defaultTransportSecurity(),
    tls: new TlsTransportSession(),
  };
  private liveMqtt: LiveMqttBridge | null = null;
  private liveWebsocket: LiveWebsocketBridge | null = null;
  private liveDds: LiveDdsBridge | null = null;
  private providerRegistry: ProviderRegistry | null = null;
  private registryBacked = new Set<TransportKind>();
  private registryKeys = new Map<TransportKind, string>();

  attachProviderRegistry(registry: ProviderRegistry): void {
    this.providerRegistry = registry;
  }

  markRegistryBacked(kind: TransportKind, key: string): void {
    this.registryBacked.add(kind);
    this.registryKeys.set(kind, key);
  }

  clearRegistryBacked(): void {
    this.registryBacked.clear();
    this.registryKeys.clear();
  }

  isRegistryBacked(kind: TransportKind): boolean {
    return this.registryBacked.has(kind);
  }

  private usesRegistryTransport(kind: TransportKind): boolean {
    // Description:
    //     UsesRegistryTransport.
    //
    // Inputs:
    //     kind: TransportKind
    //         Caller-supplied kind.
    //
    // Outputs:
    //     result: boolean
    //         Return value from `usesRegistryTransport`.
    //
    // Example:

    //     const result = usesRegistryTransport(kind);

    return this.registryBacked.has(kind) && this.providerRegistry !== null;
  }

  private withRegistryTransport<R>(
    kind: TransportKind,
    fn: (provider: TransportProvider) => R,
  ): R | undefined {
    if (!this.usesRegistryTransport(kind) || !this.providerRegistry) return undefined;
    const key = this.registryKeys.get(kind);
    if (!key) return undefined;
    return this.providerRegistry.withTransport(key, fn);
  }

  private initLiveTransports(config: FullTransportConfig): void {
    // Description:
    //     InitLiveTransports.
    //
    // Inputs:
    //     config: FullTransportConfig
    //         Caller-supplied config.
    //
    // Outputs:
    //     None.
    //
    // Example:
    //     const result = initLiveTransports(config);

    // Connect optional live transport bridges when env flags are enabled.
    if (liveMqttEnabled()) {
      this.liveMqtt = new LiveMqttBridge();
      void this.liveMqtt
        .connect(config.brokerUrl ?? "mqtt://localhost:1883", config.clientId ?? "spanda")
        .catch(() => {
          this.liveMqtt = null;
        });
    }
    if (liveWebsocketEnabled()) {
      this.liveWebsocket = new LiveWebsocketBridge();
      void this.liveWebsocket.connect(config.brokerUrl ?? "ws://localhost:9090").catch(() => {
        this.liveWebsocket = null;
      });
    }
    if (liveDdsEnabled()) {
      this.liveDds = new LiveDdsBridge();
      this.liveDds.connect(config.domainId ?? 0);
    }
  }

  private liveBridgePublish(transport: TransportKind, topic: string, payload: string): void {
    // Description:
    //     LiveBridgePublish.
    //
    // Inputs:
    //     transport: TransportKind
    //         Caller-supplied transport.
    //     topic: string
    //         Caller-supplied topic.
    //     payload: string
    //         Caller-supplied payload.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     const result = liveBridgePublish(transport, topic, payload);

    if (transport === "mqtt") this.liveMqtt?.publish(topic, payload);
    if (transport === "websocket") this.liveWebsocket?.publish(topic, payload);
    if (transport === "dds") this.liveDds?.publish(topic, payload);
  }

  private liveBridgeReceive(transport: TransportKind, topic: string): RuntimeValue | null {
    // Description:
    //     LiveBridgeReceive.
    //
    // Inputs:
    //     transport: TransportKind
    //         Caller-supplied transport.
    //     topic: string
    //         Caller-supplied topic.
    //
    // Outputs:
    //     result: RuntimeValue | null
    //         Return value from `liveBridgeReceive`.
    //
    // Example:

    //     const result = liveBridgeReceive(transport, topic);

    if (transport === "mqtt") return this.liveMqtt?.receive(topic) ?? null;
    if (transport === "websocket") return this.liveWebsocket?.receive(topic) ?? null;
    if (transport === "dds") return this.liveDds?.receive(topic) ?? null;
    return null;
  }

  configure(config: Partial<FullTransportConfig>): void {
    // Configure transport adapters and negotiate TLS wire encryption.
    //
    // Parameters:
    // - `config` — transport and security configuration
    //
    // Returns:
    // Nothing; throws when security validation fails.
    //
    // Options:
    // None.
    //
    // Example:

    // bus.configure({ nodeName: "Rover", security: busSecurity, tls: new TlsTransportSession() });

    const merged: FullTransportConfig = {
      ...this.config,
      ...config,
      security: config.security ?? this.config.security,
      tls: config.tls ?? this.config.tls,
    };
    if (
      urlRequiresTls(merged.brokerUrl) &&
      merged.security.encryption === "none"
    ) {
      merged.security = { ...merged.security, encryption: "required" };
    }
    merged.tls.connect(merged.security, merged.brokerUrl);
    this.config = merged;
    this.initLiveTransports(merged);
    this.ros2.connect(merged);
    this.mqtt.connect({
      ...merged,
      brokerUrl: merged.brokerUrl ?? "mqtt://localhost:1883",
      clientId: merged.clientId ?? "spanda",
    });
    this.dds.connect({ ...merged, domainId: merged.domainId ?? 0 });
    this.websocket.connect({
      ...merged,
      brokerUrl: merged.brokerUrl ?? "ws://localhost:9090",
    });
  }

  private adapter(transport: TransportKind): TransportAdapter | null {
    // Description:
    //     Adapter.
    //
    // Inputs:
    //     transport: TransportKind
    //         Caller-supplied transport.
    //
    // Outputs:
    //     result: TransportAdapter | null
    //         Return value from `adapter`.
    //
    // Example:
    //     const result = adapter(transport);
    // Description:
    //     Adapter.
    //
    // Inputs:
    //     transport: TransportKind
    //         Caller-supplied transport.
    //
    // Outputs:
    //     result: TransportAdapter | null
    //         Return value from `adapter`.
    //
    // Example:
    //     const result = adapter(transport);

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
    sourceId?: string | null,
  ): void {
    // Publish to in-memory bus and external adapter with optional wire encryption.
    //
    // Parameters:
    // - `topicPath` — topic path
    // - `messageType` — message type name
    // - `value` — payload runtime value
    // - `transport` — active transport kind
    // - `sourceId` — optional publisher identity for trusted-source enforcement
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // bus.publish("/motion", "Velocity", val, "mqtt", "Navigator");

    this.memory.publish(topicPath, messageType, value, transport, sourceId);
    if (this.usesRegistryTransport(transport)) {
      const started = performance.now();
      let failed = false;
      this.withRegistryTransport(transport, (provider) => {
        if (provider.isConnected()) {
          try {
            const wireValue = encodeWireValue(
              this.config,
              topicPath,
              messageType,
              value,
              sourceId,
              transport,
            );
            provider.publish(topicPath, messageType, wireValue);
          } catch {
            failed = true;
            provider.publish(topicPath, messageType, value);
          }
        }
      });
      notifyProviderCall(transport, "transport", performance.now() - started, failed);
      return;
    }
    const adapter = this.adapter(transport);
    if (!adapter) return;
    try {
      const wireValue = encodeWireValue(
        this.config,
        topicPath,
        messageType,
        value,
        sourceId,
        transport,
      );
      if (wireValue.kind === "string") {
        this.liveBridgePublish(transport, topicPath, wireValue.value);
      }
      adapter.publish(topicPath, messageType, wireValue);
    } catch {
      adapter.publish(topicPath, messageType, value);
    }
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

  receiveEnvelope(topicPath: string): CommEnvelope | null {
    // Receive the next in-memory envelope including publisher source_id.
    //
    // Parameters:
    // - `topicPath` — topic path
    //
    // Returns:
    // CommEnvelope or null.
    //
    // Options:
    // None.
    //
    // Example:

    // const env = bus.receiveEnvelope("/motion");

    return this.memory.receiveEnvelope(topicPath);
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

    return this.receiveEnvelope(topicPath)?.value ?? null;
  }

  pollInbound(transport: TransportKind): Array<[string, CommEnvelope]> {
    // Poll external transport adapters for inbound messages on subscribed topics.
    //
    // Parameters:
    // - `transport` — primary transport kind to poll
    //
    // Returns:
    // Topic path and envelope pairs pushed into the in-memory bus.
    //
    // Options:
    // None.
    //
    // Example:

    // const inbound = bus.pollInbound("mqtt");

    const paths = this.memory.subscriptionPaths();
    const inbound: Array<[string, CommEnvelope]> = [];
    const kinds = new Set<TransportKind>([transport, "ros2", "mqtt", "dds", "websocket"]);
    for (const path of paths) {
      for (const kind of kinds) {
        if (this.usesRegistryTransport(kind)) {
          const raw = this.withRegistryTransport(kind, (provider) =>
            provider.isConnected() ? provider.receive(path) : null,
          );
          if (raw) {
            try {
              const decoded = decodeWireValue(this.config, raw);
              const envelope: CommEnvelope = {
                value: decoded.value,
                sourceId: decoded.sourceId,
              };
              this.memory.pushInbound(path, envelope.value, envelope.sourceId);
              inbound.push([path, envelope]);
            } catch {
              const envelope: CommEnvelope = { value: raw, sourceId: null };
              this.memory.pushInbound(path, raw, null);
              inbound.push([path, envelope]);
            }
          }
          continue;
        }
        const adapter = this.adapter(kind);
        if (!adapter?.isConnected()) continue;
        const raw = adapter.receive(path);
        if (!raw) {
          const live = this.liveBridgeReceive(kind, path);
          if (live) {
            try {
              const decoded = decodeWireValue(this.config, live);
              const envelope: CommEnvelope = {
                value: decoded.value,
                sourceId: decoded.sourceId,
              };
              this.memory.pushInbound(path, envelope.value, envelope.sourceId);
              inbound.push([path, envelope]);
            } catch {
              const envelope: CommEnvelope = { value: live, sourceId: null };
              this.memory.pushInbound(path, live, null);
              inbound.push([path, envelope]);
            }
          }
          continue;
        }
        try {
          const decoded = decodeWireValue(this.config, raw);
          const envelope: CommEnvelope = {
            value: decoded.value,
            sourceId: decoded.sourceId,
          };
          this.memory.pushInbound(path, envelope.value, envelope.sourceId);
          inbound.push([path, envelope]);
        } catch {
          const envelope: CommEnvelope = { value: raw, sourceId: null };
          this.memory.pushInbound(path, raw, null);
          inbound.push([path, envelope]);
        }
      }
    }
    return inbound;
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
    sourceId?: string | null,
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

    this.memory.publishPeer(peer, topic, value, transport, sourceId);
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

  reconnectTransport(transport: TransportKind): void {
    // Connect the active transport adapter and resubscribe all topic paths.
    //
    // Parameters:
    // - `transport` — transport kind to activate after connectivity failover
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:

    // reconnectTransport("dds");

    const adapter = this.adapter(transport);
    if (!adapter) return;

    // Tear down stub adapters that are no longer the active transport.
    for (const kind of ["ros2", "mqtt", "dds", "websocket"] as TransportKind[]) {
      if (kind !== transport) {
        this.adapter(kind)?.disconnect();
      }
    }

    // Connect the target adapter when it is not already live.
    if (!adapter.isConnected()) {
      adapter.connect(this.config);
    }

    // Resubscribe every topic path on the newly active adapter.
    for (const path of this.memory.subscriptionPaths()) {
      adapter.subscribe(path);
    }
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

export { transportAsStr, effectiveTransportPolicy };
