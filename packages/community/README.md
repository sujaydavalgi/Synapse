# Community packages

Use this directory as a template for **community-maintained** Spanda packages that extend official registry scaffolds.

## Start from an official scaffold

1. Copy `packages/registry/spanda-ledger/` (or another official package) into your own repo or fork.
2. Update `spanda.toml` `name`, `version`, and capability requirements.
3. Implement `.sd` module exports and wire provider dispatch in your runtime integration tests.
4. Publish a tarball and add it to your private registry index.

## Reference golden path

The official `spanda-ledger` package includes a runnable example:

```bash
./scripts/ledger_golden_path.sh
```

See [packages/registry/spanda-ledger/README.md](../registry/spanda-ledger/README.md) and [future-blockchain-support.md](../../docs/future-blockchain-support.md).
