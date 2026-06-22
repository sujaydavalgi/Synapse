# spanda-node

Node.js **N-API bindings** for Spanda — `check`, `run`, `verify`, `sir`, and `fmt` from JavaScript/TypeScript.

## Dependencies

Imports workspace crates directly (Phase 16+): `spanda-driver`, `spanda-format`, `spanda-hardware`, `spanda-error`. Does not depend on `spanda-core`.

## Build

```bash
npm run build:native   # from repo root
```

Wrapped by `@spanda/native` in `packages/native/`.

## Related

- [spanda-wasm](../spanda-wasm/README.md) — browser bindings
- [packages/lsp/](../../packages/lsp/) — language server
