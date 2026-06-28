//! Canonical REST v1 route registry for OpenAPI parity tests.
//!

/// HTTP method and OpenAPI path template for a Control Center REST route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApiRoute {
    pub method: &'static str,
    pub path: &'static str,
}

/// Every REST `/v1/*` handler (excluding WebSocket `/v1/stream/telemetry` and JSON-RPC `/v1/rpc`).
pub const REST_V1_ROUTES: &[ApiRoute] = &[
    ApiRoute {
        method: "GET",
        path: "/v1/health",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/tenant",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/version",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/openapi.json",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/dashboard",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/audit/mutations",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/audit/mutations/export",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/robots",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/fleets",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/device-tree",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/device-reports",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/failover/chains",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/devices",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/devices/{id}",
    },
    ApiRoute {
        method: "PATCH",
        path: "/v1/devices/{id}",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/devices/discover",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/devices/{id}/provision",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/devices/{id}/assign",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/devices/{id}/quarantine",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/devices/{id}/trust",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/fleet/agents",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/readiness/run",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/alerts",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/alerts/test",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/secrets",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/rbac/matrix",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/provision",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/config/snapshots",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/config/snapshots",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/config/approvals",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/config/approvals",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/config/approvals/{id}/approve",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/config/approvals/{id}/reject",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/discovery",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/health/summary",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/assurance/summary",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/diagnosis/summary",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/drift",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/drift/scans",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/drift/scan",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/ota/status",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/ota/plan",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/ota/execute",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/trust/package",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/trust/program",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/entities",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/entities/graph",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/entities/traceability",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/entities/query",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/entities/{id}",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/entities/{id}/relationships",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/entities/{id}/health",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/entities/{id}/readiness",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/entities/{id}/trust",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/programs/readiness",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/programs/assure",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/programs/diagnose",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/programs/recovery/heal",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/programs/verify/hardware",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/programs/verify/capabilities",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/programs/verify/mission",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/programs/simulation",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/programs/replay",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/sre/summary",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/sre/incidents",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/sre/incidents",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/sre/incidents/{id}/ack",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/sre/incidents/{id}/resolve",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/observability/traces",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/observability/otlp/traces",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/observability/otlp/metrics",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/observability/backend",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/observability/otlp/export",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/observability/otlp/export-metrics",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/operator/quarantine",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/operator/mission/approvals",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/operator/mission/approve",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/compliance/export",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/compliance/export",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/compliance/evidence",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/digital-thread/query",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/executive/scorecard",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/analytics/readiness",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/compliance/profiles",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/reports/schedules",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/reports/schedules",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/reports/export",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/integrations/pagerduty/webhook",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/hri/sessions",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/hri/collaboration",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/hri/context",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/hri/sessions",
    },
    ApiRoute {
        method: "POST",
        path: "/v1/hri/sessions/{id}/annotate",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/hri/sessions/{id}/replay",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/humans",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/humans/readiness",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/humans/twins",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/humans/{id}/readiness",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/wearables",
    },
    ApiRoute {
        method: "GET",
        path: "/v1/human-health/policy",
    },
];
