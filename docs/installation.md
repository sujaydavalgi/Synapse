# Installing Spanda

Spanda ships prebuilt installable packages for **Linux**, **macOS**, and **Windows**. You can also build from source.

---

## Prebuilt binaries (recommended)

Download the latest release from [GitHub Releases](https://github.com/Davalgi/Spanda/releases).

Each release includes:

| Artifact | Platform |
|----------|----------|
| `spanda-cli-installer.sh` | Linux and macOS (auto-detects arch) |
| `spanda-cli-installer.ps1` | Windows (PowerShell) |
| `spanda-cli-x86_64-pc-windows-msvc.msi` | Windows installer |
| `spanda-cli-*.tar.xz` | Linux/macOS archives (per architecture) |
| `spanda-cli-*.zip` | Windows archive |
| `spanda-cli.rb` | Homebrew formula (manual or tap install) |

Release tags must match the workspace version in `Cargo.toml` (for example `v0.1.0`).

### Linux and macOS — shell installer

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/Davalgi/Spanda/releases/download/v0.1.0/spanda-cli-installer.sh | sh
```

The installer places `spanda` in `~/.cargo/bin` by default. Ensure that directory is on your `PATH`.

### macOS — Homebrew

Download `spanda-cli.rb` from the release and install locally:

```bash
brew install ./spanda-cli.rb
```

Or extract a platform archive and copy `spanda` to a directory on your `PATH`.

### Windows — MSI installer

Download `spanda-cli-x86_64-pc-windows-msvc.msi` from the release and run the installer.

### Windows — PowerShell installer

```powershell
irm https://github.com/Davalgi/Spanda/releases/download/v0.1.0/spanda-cli-installer.ps1 | iex
```

### Manual archive install

1. Download the archive for your platform (`tar.xz` on Linux/macOS, `.zip` on Windows).
2. Extract the archive.
3. Move `spanda` (or `spanda.exe`) to a directory on your `PATH`.

Supported targets:

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

Verify:

```bash
spanda --version
spanda check examples/hello_world.sd
```

---

## Quick install script

From a clone:

```bash
./scripts/install.sh
```

This runs `cargo install --path crates/spanda-cli --locked` and installs the `spanda` binary to `~/.cargo/bin`.

---

## Build from source

For contributors or unreleased builds:

### Prerequisites

- **Node.js** 18+ (TypeScript tooling and tests)
- **Rust** stable (native CLI)
- **npm**

### Steps

```bash
git clone https://github.com/Davalgi/Spanda.git
cd Spanda
npm install
npm run build:rust    # → target/release/spanda
npm test
```

Add `target/release` to your `PATH`, or run via `npm run spanda:native -- <command>`.

---

## Package maintainers

Releases are built with [cargo-dist](https://github.com/axodotdev/cargo-dist). Push a semver tag matching `Cargo.toml` (for example `v0.1.0`) to trigger `.github/workflows/release.yml`.

Local packaging:

```bash
./scripts/package-release.sh v0.1.0        # current host only
./scripts/package-release.sh v0.1.0 --all  # all targets (needs cross tools below)
```

`--all` cross-compiles Linux and Windows artifacts from your machine and requires:

```bash
cargo install --locked cargo-zigbuild cargo-xwin
brew install zig   # or: python3 -m pip install ziglang
rustup target add aarch64-apple-darwin aarch64-unknown-linux-gnu \
  x86_64-apple-darwin x86_64-unknown-linux-gnu x86_64-pc-windows-msvc
```

Or install automatically:

```bash
./scripts/package-release.sh v0.1.0 --all --install-cross
```

For official releases, prefer pushing a semver tag and letting GitHub Actions build every platform natively.

Regenerate dist CI after config changes:

```bash
dist generate
```

Configuration lives in `dist-workspace.toml`.
