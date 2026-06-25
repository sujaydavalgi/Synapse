# Warehouse configuration example

Runnable cascading configuration for a warehouse patrol fleet. Used by `spanda-config` tests and as a reference for project layout.

```bash
cd crates/spanda-config/tests/fixtures/warehouse
cargo run -p spanda -- config validate
cargo run -p spanda -- config report
cargo run -p spanda -- device-tree inspect rover-001
```

From repo root:

```bash
cargo run -p spanda -- config validate --config crates/spanda-config/tests/fixtures/warehouse/spanda.toml
```

See [docs/configuration.md](../../docs/configuration.md).
