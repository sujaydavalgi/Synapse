# Spanda Python SDK

Thin HTTP client for the [Control Center](../../docs/control-center.md) REST API v1.

WebSocket streaming: `pip install -e 'packages/sdk-python[stream]'`

```bash
pip install -e packages/sdk-python
export SPANDA_API_KEY=your-key
export SPANDA_CONTROL_CENTER_URL=http://127.0.0.1:8080
python -c "from spanda_sdk import ControlCenterClient; c=ControlCenterClient(); print(c.dashboard())"
```

## API coverage

| Area | Methods |
|------|---------|
| Health | `health()`, `dashboard()` |
| Devices | `list_devices()` |
| Readiness | `readiness_run()` |
| Drift | `drift(baseline_id)` |
| OTA | `ota_plan()`, `ota_execute()`, `ota_status()` |
| Trust | `trust_package(name, version=...)` |
| SRE | `sre_summary()`, `list_incidents()`, `create_incident()`, `ack_incident()`, `resolve_incident()` |
| Config | `list_config_snapshots()`, `save_config_snapshot()`, `list_config_approvals()`, `submit_config_approval()`, `approve_config_approval()`, `reject_config_approval()` |
| Compliance | `compliance_export()`, `list_compliance_evidence()`, `executive_scorecard()`, `digital_thread_query()`, `export_reports()` |
| Ops | `list_alerts()`, `list_audit_mutations()` |
| Gateway | `rpc(method, params)` |

Integration tests: set `SPANDA_SDK_INTEGRATION=1` with a running `spanda control-center serve` (see `scripts/enterprise_ops_smoke.sh`).
