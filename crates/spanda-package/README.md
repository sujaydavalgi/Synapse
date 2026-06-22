# spanda-package

Spanda **package manager** — `spanda.toml` manifests, dependency resolution, registry fetch, publish bundles, and hardware metadata validation.

## Design note

`spanda-package` does **not** depend on `spanda-core` (Phase 4 cycle break). Hardware catalog and adapter verify use [`spanda-hardware`](../spanda-hardware/README.md).

## CLI surface

Invoked via `spanda init`, `add`, `install`, `build`, `test`, `publish`, and related subcommands in `spanda-cli`.

## Related

- [docs/packages.md](../../docs/packages.md)
- [docs/spanda-toml.md](../../docs/spanda-toml.md)
- [docs/registry.md](../../docs/registry.md)
