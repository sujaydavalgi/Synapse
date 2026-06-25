# Package Trust Framework

**Status:** Experimental · **Phase:** Verify, Build · **Priority:** P0.4

Improve ecosystem trust with transparent scoring for registry packages.

## CLI

```bash
spanda trust spanda-mqtt
spanda trust spanda-mqtt --version 0.1.0 --json
spanda trust spanda-mqtt --project examples/showcase/rover
```

## Evaluation factors

| Factor | Weight | Signal |
|--------|--------|--------|
| Registry listed | 20 | Package appears in local/remote registry index |
| Official framework | 15 | Listed in official Spanda framework packages |
| License | 10 | Permissive license (Apache-2.0, MIT, BSD-3-Clause) |
| Maintained | 10 | At least one published version |
| Checksum | 15 | SHA-256 checksum in registry index |
| Signed | 20 | Ed25519 registry signature verified with trust key |
| Safety metadata | 10 | Vendored `spanda.toml` `[safety]` level |

## Output

`TrustScoreReport` (0–100) with factor breakdown, tier (`trusted` / `acceptable` / `low`), and recommendations. Pass threshold: **60**.

## Integration

- `spanda explain --config` includes a `package_trust` section for configured packages
- `spanda deploy gate --config` runs a `package_trust` gate when packages are declared in `spanda.toml`
- Feeds composite `TrustScore` in tamper framework (future)

## Crate

`spanda-package::trust` — composes registry metadata, `registry_sign`, and vendored safety manifests.

See [trust-framework.md](./trust-framework.md) · [registry.md](./registry.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
