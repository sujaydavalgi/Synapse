# spanda-transport

Transport **adapter traits**, wire-frame encode/decode, TLS/session helpers, and stub transport state.

Adapter implementations live in sibling crates:

- `spanda-transport-ros2`
- `spanda-transport-mqtt`
- `spanda-transport-dds`
- `spanda-transport-websocket`

Routing and `RoutingCommBus` live in [`spanda-transport-routing`](../spanda-transport-routing/README.md).

`spanda_core::transport` re-exports this crate plus routing for backward compatibility.
