# Publishing `@davalgi-spanda/web`

The Control Center React panel ships from `packages/web` as **`@davalgi-spanda/web`** (npm org `@davalgi-spanda` — `@spanda` scope is unavailable).

## Install

```bash
npm install @davalgi-spanda/web
```

## Versioning

Follow semver aligned with the Spanda release (`package.json` → `version`). Bump **minor** for additive UI/API client changes; **major** for breaking prop or export changes.

## Local dry-run

```bash
cd packages/web
npm run build
npm pack
```

## CI publish

`.github/workflows/publish-npm-web.yml` runs `npm pack --dry-run` on every PR and publishes on tags:

```text
npm-web-v0.4.0
```

Requires GitHub secret **`NPM_TOKEN`** (same as `@davalgi-spanda/sdk`). Granular tokens expire within **90 days** — rotate before expiry.

## Exports

- `@davalgi-spanda/web/ControlCenterPanel` — React panel (used by Tauri desktop shell)
- `@davalgi-spanda/web/index.css` — panel styles

## Scope

This package contains the playground IDE and `ControlCenterPanel`. It does **not** include the Tauri desktop shell (`@spanda/control-center-desktop`).

See also [docs/sdk-publishing.md](../../docs/sdk-publishing.md).
