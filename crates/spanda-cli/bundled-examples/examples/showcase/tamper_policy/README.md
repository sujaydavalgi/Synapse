# Tamper policy showcase

Declarative `tamper_policy` branches dispatch recovery actions on runtime security faults. **Critical** destructive actions (stop, safe mode, kill switch) require operator approval unless `SPANDA_OPERATOR_APPROVAL=1`.

## Commands

```bash
spanda tamper-check examples/showcase/tamper_policy/rover.sd
spanda sim examples/showcase/tamper_policy/rover.sd --inject-security-faults
```

With operator approval for destructive responses:

```bash
export SPANDA_OPERATOR_APPROVAL=1
spanda sim examples/showcase/tamper_policy/rover.sd --inject-security-faults
```

## One command

```bash
spanda demo trust
```

Smoke: `scripts/tamper_policy_smoke.sh`

See [docs/tamper-detection.md](../../../docs/tamper-detection.md).
