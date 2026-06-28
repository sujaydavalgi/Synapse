# SDK publishing

Maintainer how-to for **Rust, Python, TypeScript, and web panel** releases: **[docs/sdk-publishing.md](../docs/sdk-publishing.md)**.

Quick reference:

| Package | Tag | Secret |
|---------|-----|--------|
| Rust `spanda-sdk` | `crates-sdk-vX.Y.Z` | `CRATES_IO_TOKEN` |
| Python `spanda-sdk` | `sdk-python-vX.Y.Z` | `PYPI_API_TOKEN` |
| TypeScript `@davalgi-spanda/sdk` | `npm-sdk-vX.Y.Z` | `NPM_TOKEN` |
| Web `@davalgi-spanda/web` | `npm-web-vX.Y.Z` | `NPM_TOKEN` |

Dry-run: `./scripts/verify_sdk_publish_ready.sh` from repo root.
