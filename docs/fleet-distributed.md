# Distributed fleet operations

Spanda supports **multi-robot programs** locally (`spanda fleet run`) and **remote orchestration** when fleet agents are registered on the network.

## Local multi-robot

```bash
spanda fleet run examples/showcase/fleet_management/main.sd
```

The interpreter sets up and executes each robot in isolation so coordinator-only robots (no actuators) do not shadow member bindings.

## Remote orchestration

### 1. Start mesh and agents

**Coordinator host:**

```bash
spanda fleet mesh start --bind 127.0.0.1:9700 --token "$FLEET_TOKEN"
```

**Per-robot hosts:**

```bash
spanda fleet agent start --bind 127.0.0.1:9701 --robot RoverA --token "$FLEET_TOKEN"
spanda fleet agent register RoverA http://127.0.0.1:9701 --token "$FLEET_TOKEN"
```

List registered agents:

```bash
spanda fleet agent list
spanda fleet agent list --json
```

### 2. Orchestrate with relay

```bash
spanda fleet orchestrate --remote --json examples/robotics/fleet_field_trial.sd
```

`--remote` relays peer deliveries to registered fleet agents. JSON output includes `remote_relayed` and `remote_failed` counts per fleet block.

### 3. Mesh URL (optional)

When a mesh coordinator is already running elsewhere:

```bash
spanda fleet orchestrate --remote \
  --mesh-url http://coordinator:9700 --mesh-token "$FLEET_TOKEN" \
  fleet_program.sd
```

## OTA deploy (remote rollout)

Remote OTA uses the same agent registry:

```bash
spanda deploy plan --version 1.2.0 program.sd
spanda deploy rollout --remote --require-certify program.sd
spanda deploy rollback --remote program.sd
```

See [phase-18-security-hardening.md](./phase-18-security-hardening.md) for token requirements on non-loopback binds.

## Golden path

```bash
./examples/robotics/golden_path_deploy.sh
```

CI job: `robotics-golden-path`.

## Related

- [fleet-health.md](./fleet-health.md)
- [concurrency.md](./concurrency.md)
- [tier-3-experimental.md](./tier-3-experimental.md)
