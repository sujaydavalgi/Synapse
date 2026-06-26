# OTLP collector HA deployment

Guide for running Spanda Control Center observability exports against a highly available OpenTelemetry collector tier.

## Architecture

```text
Control Center ──POST /v1/observability/otlp/export──► OTLP Collector (HA)
        │                        │
        │                        ├──► Jaeger (traces)
        │                        └──► Prometheus/Mimir (metrics)
        └── GET /v1/observability/backend
```

## Environment

| Variable | Purpose |
|----------|---------|
| `SPANDA_OTLP_TRACES_ENDPOINT` | OTLP HTTP traces URL (e.g. `http://collector:4318/v1/traces`) |
| `SPANDA_OTLP_METRICS_ENDPOINT` | OTLP HTTP metrics URL |
| `SPANDA_OTLP_TOKEN` | Optional bearer token for collector auth |

## HA collector pattern

1. Deploy **two or more** OpenTelemetry Collector instances behind a load balancer.
2. Point Control Center at the LB VIP, not a single pod.
3. Enable collector `retry_on_failure` and `sending_queue` exporters.
4. Run Grafana with `spanda-grafana-dashboards` templates against the metrics backend.

## Health checks

- `GET /v1/observability/backend` — configured endpoints summary
- `GET /v1/observability/otlp/traces` — trace preview before export
- `POST /v1/observability/otlp/export` — push traces (requires Deploy role)

## Failure modes

| Symptom | Mitigation |
|---------|------------|
| Collector 503 | LB routes to healthy replica; increase `sending_queue` size |
| Token expiry | Rotate `SPANDA_OTLP_TOKEN` via secret vault |
| Trace gaps | Enable correlation IDs (`X-Correlation-ID`) end-to-end |

## Related

- [control-center.md](./control-center.md)
- [packages/registry/spanda-grafana-dashboards](../packages/registry/spanda-grafana-dashboards/README.md)
