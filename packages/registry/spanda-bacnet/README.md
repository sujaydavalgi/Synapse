# spanda-bacnet

BACnet building automation bridge for Smart Spaces blueprints.

## Runtime

Provider dispatch (`iot.bacnet.read_point`) routes through `spanda-providers` → `iot_hub` → `iot_live` when:

- `SPANDA_LIVE_BACNET=1`
- `SPANDA_BACNET_CMD` shell template (`{device}`, `{object_id}`), or
- Python bridge handler `bacnet_read_point` (mock without hardware)

## Smoke

```bash
spanda check packages/registry/spanda-bacnet/tests/smoke.sd
./scripts/smart_spaces_live_iot_smoke.sh
```

## Example

```bash
export SPANDA_LIVE_BACNET=1
export SPANDA_BACNET_CMD='echo live:{device}:{object_id}'
spanda control-center smart-spaces readiness --facility-id tower-demo
```
