# Configuration Drift Detection

**Status:** Experimental · **Phase:** Deploy, Operate · **Priority:** P1.1

Detect mismatch between **expected** (approved baseline configuration, declared deploy posture, and program artifacts) and **actual** (live resolved configuration and on-device agent reports).

## CLI

```bash
# Compare approved baseline against live project config
spanda drift --baseline configs/approved/ --config spanda.toml

# Program + live deploy agent check (uses .spanda/deploy-agents.json)
spanda drift rover.sd --agent Rover@JetsonOrin --config spanda.toml

# Program drift against all registered deploy/fleet agents
spanda drift rover.sd --config spanda.toml

# Config subcommand alias
spanda config drift --baseline configs/approved/ rover.sd --json

# Readiness with baseline drift gates
spanda readiness rover.sd --config spanda.toml --baseline configs/approved/
```

## Comparison dimensions

| Dimension | Expected | Actual |
|-----------|----------|--------|
| Configuration | Baseline merged TOML | Live merged TOML |
| Fleet | Baseline fleet tree | Live fleet tree |
| Device | Baseline `DeviceRegistry` | Live device records |
| Provider / Package | Baseline manifests | Live manifests |
| Mapping | Baseline logical map | Live logical map |
| Program | `.sd` sensors/actuators | Live logical map |
| Hardware | `deploy … to <profile>` | Agent `/v1/status` `hardware_profile` |
| Firmware | Device `firmware_version` in config | Agent `/v1/status` `firmware_version` |
| Program hash | SHA-256 of `.sd` file | Agent `/v1/status` `program_hash` |
| Packages | `ResolvedSystemConfig.packages` | Agent `/v1/status` `packages` |

## Agent status fields

Deploy agents (`spanda deploy agent`) and fleet agents (`spanda fleet agent`) expose drift fields on `GET /v1/status`:

- `program_hash` — set on rollout
- `hardware_profile` — from deploy assignment or rollout payload
- `firmware_version` — optional rollout metadata
- `packages` — optional rollout metadata
- `healthy` — agent health flag

## Output

`ConfigDriftReport` — structured findings with `dimension`, `severity`, `message`, and optional `path`. Medium-or-higher severity fails the check (exit code 1).

## Foundation

- Semantic config comparison: `spanda-config::drift`
- Agent snapshot comparison: `expected_agent_states` + `detect_agent_drift`
- Readiness baseline gates: `spanda readiness --baseline`

## Related

[configuration.md](./configuration.md) · [readiness.md](./readiness.md) · [platform-maturity-roadmap.md](./platform-maturity-roadmap.md)
