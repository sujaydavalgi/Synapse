# spanda-ledger

Official Spanda package: **Audit ledger anchoring**

## Import

```spanda
import provenance.ledger;
```

## Capabilities

This package requires runtime capabilities declared in `spanda.toml` (`audit.append`).

## Community example

Runnable anchor workflow for community package authors:

```bash
./scripts/ledger_golden_path.sh
```

Project: `examples/anchor_audit/` (local path dependency on this package).

## Status

Scaffold package — implements the lean-core provider contract surface.
Core retains compatibility shims until callers migrate to this package.

See [packages/community/README.md](../../community/README.md) for fork guidance.
