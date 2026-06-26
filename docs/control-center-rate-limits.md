# Control Center API rate limits

Spanda Control Center uses an in-memory token bucket per API key (`SPANDA_API_RATE_LIMIT_PER_MINUTE`). When exceeded, clients receive HTTP **429** with `Retry-After`.

## Recommended tiers

| Tier | Devices | `SPANDA_API_RATE_LIMIT_PER_MINUTE` | Notes |
|------|---------|-----------------------------------|-------|
| Dev / lab | 1–10 | `0` (disabled) | Default for local `control-center serve` |
| Pilot | 10–100 | `120` | ~2 req/s sustained per API key |
| Production | 100–1000 | `300` | Dashboard + drift scans + OTA planners |
| Fleet | 1000+ | `600` | Multiple operator keys; split read vs mutate keys |

## Load-test defaults

Before promoting to **Stable**, run:

```bash
./scripts/enterprise_ops_smoke.sh
SPANDA_API_RATE_LIMIT_PER_MINUTE=300 cargo run -p spanda -- control-center serve --bind 127.0.0.1:8080
```

Hammer read endpoints (`/v1/dashboard`, `/v1/devices`) from CI or `hey`/`wrk` and confirm 429 behavior above the configured limit.

## Client behavior

- Honor `Retry-After` on 429 responses.
- Use WebSocket `/v1/stream/telemetry` for live updates instead of polling REST.
- Prefer batch operators (`/v1/drift/scan`, `/v1/ota/plan`) over per-device loops.

## Related

- [control-center.md](./control-center.md)
- [stable-hardening-enterprise-ops.md](./stable-hardening-enterprise-ops.md)
