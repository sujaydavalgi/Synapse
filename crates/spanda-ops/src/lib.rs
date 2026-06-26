//! Enterprise operations primitives for Spanda Control Center.
//!
pub mod alerting;
pub mod otlp_metrics;
pub mod otlp_traces;
pub mod pdf_report;
pub mod slack;

pub use alerting::{Alert, AlertChannel, AlertDispatcher, AlertSeverity, AlertStore, AlertType};
pub use otlp_metrics::{
    env_metrics_endpoint, push_otlp_metrics, render_otlp_metrics_json, ControlCenterMetrics,
};
pub use otlp_traces::{
    env_otlp_token, env_trace_auto_push_enabled, env_traces_endpoint, push_otlp_traces,
    render_otlp_traces_json, HttpTraceSpan,
};
pub use pdf_report::render_text_pdf;
pub use slack::slack_webhook_payload;
