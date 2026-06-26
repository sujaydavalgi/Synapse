# spanda-alert-escalation

On-call rotation and escalation policy templates for Spanda alerting.

## Status

**Experimental** — policy templates + env-driven routing; pairs with `spanda-alert-pagerduty`.

## Recommended tiers

| Severity | Initial route | Escalation (if unacked) |
|----------|---------------|-------------------------|
| `info` | Log / Teams | None |
| `warning` | Teams webhook | PagerDuty after 15m (`SPANDA_ALERT_DEDUP_WINDOW_WARNING_SECS`) |
| `critical` | PagerDuty + incident auto-open | Secondary routing key after 5m |

## Configuration

```bash
export SPANDA_ALERT_PAGERDUTY_URL="https://events.pagerduty.com/v2/enqueue"
export SPANDA_ALERT_PAGERDUTY_ROUTING_KEY="primary-oncall"
export SPANDA_ALERT_TEAMS_URL="https://outlook.office.com/webhook/..."
export SPANDA_ALERT_DEDUP_WINDOW_WARNING_SECS=900
export SPANDA_ALERT_DEDUP_WINDOW_CRITICAL_SECS=0
```

Use `POST /v1/sre/incidents` + PagerDuty bi-directional sync for ack tracking.

## Related

- [spanda-alert-pagerduty](../spanda-alert-pagerduty/README.md)
- [control-center.md](../../../docs/control-center.md)
