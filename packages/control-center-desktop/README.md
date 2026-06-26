# Spanda Control Center (desktop)

Tauri v2 desktop shell for the Spanda Control Center. The UI reuses `ControlCenterPanel` from `@spanda/web`; the API backend is expected to run separately via `spanda control-center serve` (or any compatible `spanda-api` deployment).

## Security note

The desktop `src-tauri/Cargo.lock` may report [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html) (`glib` &lt; 0.20) via the GTK/Tauri stack on Linux builds. Upstream fix requires gtk-rs 0.20+ when Tauri adopts it; the Control Center web/API path does not depend on `glib`.

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

## Auto-update (experimental)

The Tauri shell includes `tauri-plugin-updater` with `active: false` by default. To enable signed updates in production:

1. Generate signing keys (`tauri signer generate`).
2. Set `plugins.updater.pubkey` in `src-tauri/tauri.conf.json`.
3. Set `plugins.updater.active` to `true` and configure release endpoints.

Until then, operators update via platform installers from CI (`TAURI_BUILD=1` on macOS).

## Status

**Experimental** — scaffold, dev workflow, and updater plugin wiring; production installers and signed auto-update are not yet published.
