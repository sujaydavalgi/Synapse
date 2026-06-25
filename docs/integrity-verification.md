# Integrity Verification

**Status:** Experimental (verify-time) · **Phase:** Verify, Deploy · **Priority:** P3.1

Verify that declared system artifacts match approved baselines.

## CLI

```bash
spanda integrity rover.sd
spanda integrity rover.sd --baseline approved/rover.sd --json
spanda integrity rover.sd --agent Rover@JetsonOrin --config spanda.toml --json
```

Verify-time `spanda integrity` hashes hardware, missions, safety rules, policies, health policies, capabilities, deploy targets, and package imports. With `--baseline`, each artifact is **Trusted**, **Modified**, or **Unknown** (no baseline). With `--agent`, compares live agent `/v1/status` program hash and hardware profile against the declared deploy target.

## Verified artifacts

| Artifact | Method |
|----------|--------|
| Hardware profiles | Profile hash vs agent report |
| Mission definitions | AST hash vs audit baseline |
| Capability definitions | Registry hash |
| Policies | Policy block hash |
| Safety rules | Safety AST hash |
| Health policies | Health decl hash |
| Package metadata | Registry signature + hash |
| Provider registrations | Provider manifest hash |

## Output

`IntegrityReport` — per-artifact status: **Trusted**, **Modified**, or **Unknown**.

## Foundation

Builds on `spanda-audit`, deploy bundle signatures (`--require-signature`, `--require-hash`), and `spanda certify prove`.

See [tamper-detection.md](./tamper-detection.md) · [audit-provenance.md](./audit-provenance.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md).
