# spanda-providers

**Official package bootstrap** — registers transport, sensor, navigation, fleet, and domain provider stubs when official packages are installed.

## Responsibilities

- `bootstrap_default_providers()` — default in-memory + stub transports
- `bootstrap_providers_for_packages()` — project-scoped registration from lockfile/manifest
- Package stubs for all 20 official packages under `packages/registry/`
- `TransportAdapterProvider` bridge for legacy `TransportAdapter` types

`spanda-core::providers` is a thin facade re-exporting this crate plus `spanda_runtime::classification`.

## Related

- [docs/provider-interfaces.md](../../docs/provider-interfaces.md)
- [docs/official-packages.md](../../docs/official-packages.md)
