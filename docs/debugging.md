# Debugging Spanda in VS Code

Step through `behavior`, `task every`, and top-level `every` trigger loops with the **Spanda debug adapter** (`spanda-dap`) and the VS Code extension.

## Prerequisites

```bash
cargo build -p spanda-dap -p spanda-cli --release
export PATH="$PWD/target/release:$PATH"
```

Install the [Spanda VS Code extension](../editor/vscode/README.md) (marketplace or local VSIX).

## Launch configuration

The extension contributes a **Spanda** debug type. Default `launch.json` entry:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "spanda",
      "request": "launch",
      "name": "Spanda: Debug current file",
      "program": "${file}"
    }
  ]
}
```

## Workflow (killer demo)

1. Open `examples/showcase/killer_demo.sd`
2. Set a breakpoint inside `behavior patrol()` or `task control_loop`
3. Press **F5** (Start Debugging)
4. Use **Step Over** (`next`) to advance through `task every 50ms` / `loop every 100ms` iterations
5. Inspect locals in the Variables panel when paused

CLI equivalent (breakpoint smoke test):

```bash
spanda debug examples/showcase/killer_demo.sd --break 60
```

## What works

| Capability | Status |
|------------|--------|
| Breakpoints on line numbers | Supported |
| Step over (`next`) | Advances periodic tasks, loop bodies, and `every` trigger ticks |
| Step in / step out | Experimental |
| Variables / locals | Per-frame snapshot at pause |
| `task every` loops | Step over pauses between task ticks when breakpoints hit |
| `every` trigger bodies | Debug session enters from behavior, task, or `every` trigger body (Phase 35) |

## Troubleshooting

| Issue | Fix |
|-------|-----|
| `spanda-dap not found` | Build `spanda-dap`; set `spanda.cliPath` to `target/release/spanda` |
| No pause on `task every` | Set breakpoint inside task body; use Step Over after first hit |
| LSP works but debug fails | Debug uses `spanda-dap`, separate from LSP |

## Related

- [killer-demo.md](./killer-demo.md) — flagship program for demos
- [editor/vscode/README.md](../editor/vscode/README.md) — extension install
- `crates/spanda-dap/` — DAP server source
