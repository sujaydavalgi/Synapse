# spanda-cli

Native `spanda` binary — check, verify, run, sim, fleet, deploy, package manager, and tooling commands.

## Build

From the repo root:

```bash
cargo build -p spanda-cli --release
# → target/release/spanda
```

Or `npm run build:rust`.

## Dependencies

Imports workspace crates directly (Phase 15+): `spanda-driver`, `spanda-hardware`, `spanda-ota`, `spanda-fleet`, `spanda-format`, `spanda-lint`, `spanda-codegen`, `spanda-docs`, `spanda-certify`, and others. Does **not** depend on `spanda-core`.

Optional native codegen: `spanda-llvm` for `ir` / `llvm-ir` / `compile-native` subcommands.

## Entry point

`src/main.rs` — command routing; deploy/OTA in `deploy_ota.rs`, swarm in `swarm_cli.rs`, certify in `certify_cli.rs`, packages in `package.rs`.

## Related

- [docs/getting-started.md](../../docs/getting-started.md)
- [docs/man/](../../docs/man/) — CLI reference
