# spanda-audit-siem

SIEM export adapter for Spanda Control Center mutation audit records.

## Status

**Experimental** — export is built into Control Center (`GET /v1/audit/mutations/export`).

## Export formats

| Format | Query | Use case |
|--------|-------|----------|
| JSONL | `?format=jsonl` | Splunk HEC, Elastic ingest pipelines |
| CEF | `?format=cef` | ArcSight, Sentinel, QRadar CEF receivers |

```bash
curl -H "Authorization: Bearer $SPANDA_API_KEY" \
  "http://127.0.0.1:8080/v1/audit/mutations/export?format=cef"
```

Records originate from the append-only JSONL file at `SPANDA_MUTATION_AUDIT_PATH` (default `.spanda/control-center-mutations.jsonl`).

## Forwarding

Configure your SIEM collector to poll or tail the export endpoint on a schedule, or mirror the JSONL file with your log shipper (Fluent Bit, Vector, Filebeat).
