//! Enterprise operations primitives for Spanda Control Center.
//!
pub mod alerting;
pub mod incidents;
pub mod otlp_metrics;
pub mod sre;
pub mod otlp_traces;
pub mod pagerduty;
pub mod pdf_report;
pub mod slack;
pub mod teams;

pub use alerting::{Alert, AlertChannel, AlertDispatcher, AlertSeverity, AlertStore, AlertType};
pub use incidents::{
    Incident, IncidentSeverity, IncidentStatus, IncidentStore,
};
pub use sre::{
    health_trends_summary, mtbf_hint_ms, slo_burn_rate_fast_threshold,
    slo_burn_rate_summary, slo_burn_rate_window_hours, slo_status, slo_target_percent,
};
pub use otlp_metrics::{
    env_metrics_endpoint, push_otlp_metrics, render_otlp_metrics_json, ControlCenterMetrics,
};
pub use otlp_traces::{
    env_otlp_token, env_trace_auto_push_enabled, env_traces_endpoint, observability_backend_summary,
    push_otlp_traces, render_otlp_traces_json, HttpTraceSpan,
};
pub use pdf_report::render_text_pdf;
pub use slack::slack_webhook_payload;
