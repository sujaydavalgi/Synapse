# spanda-transport-routing

**`RoutingCommBus`** — routes publish/subscribe/service calls across in-memory simulation and live transport adapters (ROS2, MQTT, DDS, WebSocket).

## Modules

| Module | Purpose |
|--------|---------|
| `lib.rs` | `RoutingCommBus`, adapter registration, wire encode/decode integration |
| `transport_live` | ROS2/MQTT live publish hooks (`try_ros2_publish`, `mqtt_live_enabled`, …) |
| `live_bridges` | `LiveMqttBridge`, `LiveDdsBridge`, `LiveWebsocketBridge` with `RuntimeValue` conversion |

## Migration (Phase 17)

Removed from `spanda-core`:

- `spanda_core::transport_live` → `spanda_transport_routing::transport_live`
- `spanda_core::transport_mqtt` / `transport_dds` / `transport_websocket` → `spanda_transport_routing::live_bridges` or the `spanda-transport-*` crates

`spanda_core::transport` still re-exports `RoutingCommBus` for backward compatibility.

## Related

- [spanda-transport](../spanda-transport/README.md) — adapter traits and wire frames
- [spanda-providers](../spanda-providers/README.md) — official package bootstrap
