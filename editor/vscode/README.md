# Spanda VS Code Extension

Language support for Spanda (`.sd`) with bundled LSP diagnostics and debug adapter wiring.

## Install from Marketplace

Search **Spanda** in the VS Code Extensions view (`spanda-lang.spanda-vscode`), or:

```bash
code --install-extension spanda-lang.spanda-vscode
```

## Prerequisites

Install the native Spanda CLI so `check` and `verify` diagnostics work:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/Davalgi/Spanda/releases/download/v0.1.0/spanda-cli-installer.sh | sh
```

Or build from source: `cargo build -p spanda-cli -p spanda-dap --release`

## Features

| Feature | How |
|---------|-----|
| Syntax highlighting | Automatic for `.sd` files |
| Type diagnostics | LSP → `spanda check` |
| Verify diagnostics | LSP → `spanda verify` (warnings/errors in Problems panel) |
| Deploy target autocomplete | Type `deploy Robot to ` — suggests `RoverV1`, `JetsonOrin`, … |
| Verify with picker | Command Palette → **Spanda: Verify Deploy Target…** |
| Debug | F5 with Spanda debug configuration — steps through `behavior`, `task every`, and `every` triggers via `spanda-dap` |

## Settings

| Setting | Description |
|---------|-------------|
| `spanda.cliPath` | Path to `spanda` binary (default: `spanda` on PATH) |
| `spanda.languageServerPath` | Override bundled LSP server (monorepo dev only) |

## Debug workflow

1. Open a `.sd` file with `behavior` or `task every` blocks
2. Set breakpoints in the gutter
3. Run **Debug: Start Debugging** (Spanda configuration)
4. Use Step Over to advance through periodic tasks

See [docs/debugging.md](../../docs/debugging.md) and [docs/killer-demo.md](../../docs/killer-demo.md).

## Build VSIX locally

```bash
./scripts/bundle-vscode-extension.sh
cd editor/vscode && npm run package
code --install-extension spanda-vscode-0.1.0.vsix
```

## Publish to Marketplace (maintainers)

1. Create publisher `spanda-lang` on [Visual Studio Marketplace](https://marketplace.visualstudio.com/manage)
2. Generate a Personal Access Token with **Marketplace → Manage**
3. `npx vsce login spanda-lang` (or set `VSCE_PAT`)
4. From `editor/vscode`: `npm run publish:marketplace`

`vscode:prepublish` runs `bundle-vscode-extension.sh` automatically before `vsce package` / `vsce publish` — no separate bundle step needed.

CI builds and verifies the VSIX on every push (`.github/workflows/vscode-extension-ci.yml`). Marketplace publish runs on tag release when `VSCE_PAT` is set.

Verify a local VSIX build without publishing:

```bash
./scripts/verify_vscode_vsix.sh
```

## Monorepo development

```bash
npm run build --workspace=@spanda/lsp
cd editor/vscode && npm run build
# Press F5 in editor/vscode for Extension Development Host
```
