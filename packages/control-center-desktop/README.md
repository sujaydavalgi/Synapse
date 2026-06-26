# Spanda Control Center (desktop)

Tauri v2 desktop shell for the Spanda Control Center. The UI reuses `ControlCenterPanel` from `@spanda/web`; the API backend is expected to run separately via `spanda control-center serve` (or any compatible `spanda-api` deployment).

## Security note

The desktop `src-tauri/Cargo.lock` may report [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html) (`glib` &lt; 0.20) on Linux builds. This repo patches `glib`/`glib-sys`/`glib-macros` from the gtk-rs `0.18` git branch (VariantStrIter backport) via `[patch.crates-io]` in `Cargo.toml`. Upstream gtk-rs 0.20+ adoption in Tauri is tracked for v3; the Control Center web/API path does not depend on `glib`.

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 20+
- Platform Tauri dependencies: [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

## Quick start

1. Start the Control Center API (from repo root):

```bash
cargo run -p spanda -- control-center serve --bind 127.0.0.1:8080
```

2. Install workspace dependencies (once):

```bash
npm install
```

3. Run the desktop app in dev mode:

```bash
npm run dev --workspace=@spanda/control-center-desktop
```

Optional: point the UI at a different API URL:

```bash
VITE_CONTROL_CENTER_URL=http://127.0.0.1:9090 npm run dev --workspace=@spanda/control-center-desktop
```

## Build

Generate platform icons from the bundled PNG (first time only):

```bash
npm exec tauri icon --manifest-path packages/control-center-desktop/src-tauri/Cargo.toml packages/control-center-desktop/src-tauri/icons/icon.png
```

Production bundle:

```bash
npm run build --workspace=@spanda/control-center-desktop
```

## Smoke check

```bash
./scripts/control_center_desktop_smoke.sh
```

This runs `cargo check` on the Tauri crate (no GUI required).

## Architecture

| Layer | Package / crate |
|-------|-----------------|
| React UI | `packages/web` (`ControlCenterPanel`) |
| Desktop shell | `packages/control-center-desktop` (Vite + Tauri) |
| API | `spanda-api` via `spanda control-center serve` |

The desktop app does not embed the Rust API server; operators typically run the API locally or against a fleet endpoint.

## Auto-update

The Tauri shell includes `tauri-plugin-updater`. In development builds, `active` defaults to `false`. For production releases:

1. Generate signing keys: `npm run tauri signer generate -- -w ~/.tauri/spanda-updater.key`
2. Set `TAURI_UPDATER_PUBKEY` at build time (injected via `src-tauri/build.rs`)
3. Set `SPANDA_DESKTOP_UPDATER_ACTIVE=1` (or `TAURI_UPDATER_ACTIVE=true`) when building with `TAURI_BUILD=1`
4. Publish signed artifacts from `.github/workflows/desktop-release.yml` (tag `desktop-v*`)

See [docs/desktop-release-runbook.md](../../docs/desktop-release-runbook.md).

Until the first production release is published, operators update via platform installers from CI (`TAURI_BUILD=1` on macOS).

Optional macOS codesign/notarization after bundle: set `APPLE_SIGNING_IDENTITY` and `APPLE_NOTARIZE_PROFILE`, then run `./scripts/sign_tauri_macos.sh` (also wired in CI when secrets are present).

## Status

**Experimental** — dev workflow, CI signing scaffold, and env-gated updater wiring are shipped. **First signed production release** is pending maintainer tags and Apple/registry secrets. Stable promotion: [docs/stable-hardening-enterprise-ops.md](../../docs/stable-hardening-enterprise-ops.md).
