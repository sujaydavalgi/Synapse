//! Platform event emission for readiness evaluation.
//!
use spanda_audit::platform_event::names;
use spanda_audit::{AuditRuntime, PlatformEvent};
use serde_json::json;

use crate::entity_readiness::EntityReadinessReport;

/// Record a `ReadinessChanged` platform event for an entity readiness report.
pub fn record_readiness_platform_event(
    audit: &mut AuditRuntime,
    report: &EntityReadinessReport,
) {
    let event = PlatformEvent::new(
        names::READINESS_CHANGED,
        "spanda-readiness",
        json!({
            "entity_type": report.entity_type,
            "readiness_status": report.readiness_status,
            "mission_ready": report.mission_ready,
            "score": report.score,
            "issue_count": report.issues.len(),
            "sources": report.sources,
        }),
    )
    .with_entity_id(report.entity_id.clone());
    let _ = audit.record_platform_event(&event);
    let _ = spanda_telemetry_store::record_platform_event(&event);
}
