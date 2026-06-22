# spanda-mqtt

Official Spanda package: **MQTT pub/sub transport**

## Import

```spanda
import communication.mqtt;
```

## Live backend

When `spanda-mqtt` is listed in project `spanda.toml` dependencies, the runtime registers
`crates/spanda-transport-mqtt` via `bootstrap_providers_for_packages()` and routes the
comm-bus through the provider registry.

## Status

Spanda-language exports in `src/` are scaffold stubs. Live MQTT I/O is implemented in the
`spanda-transport-mqtt` workspace crate (live hooks in `spanda-transport-routing`).
