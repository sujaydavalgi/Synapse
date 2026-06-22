# spanda-dap

**Debug Adapter Protocol** server for Spanda — VS Code and compatible editors.

## Dependencies

`spanda-driver` (`DebugMachine`), `spanda-debug` (`DebugSession`, `DebugOptions`), `spanda-error`.

## Run

```bash
cargo run -p spanda-dap
```

Wire the VS Code extension in `editor/vscode/` to this binary.

## Related

- [docs/debugging.md](../../docs/debugging.md)
