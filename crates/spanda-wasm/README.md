# spanda-wasm

**WebAssembly bindings** for the Spanda playground — `wasm_check`, `wasm_run`, `wasm_verify`, `wasm_ir`, `wasm_fmt`.

## Dependencies

`spanda-driver`, `spanda-format`, `spanda-hardware`, `spanda-error` (no `spanda-core`).

## Build

```bash
npm run build:wasm
```

Consumed by `@spanda/web` (`packages/web/`).
