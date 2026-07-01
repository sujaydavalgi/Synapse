//! Entity health platform event emission tests.

use spanda_audit::platform_event::names;
use spanda_readiness::{
    record_entity_health_platform_events, reset_health_status_cache_for_tests,
    EntityHealthDiagnostic, EntityHealthMetrics, EntityHealthReport,
};
use spanda_runtime::platform_event_runtime::{
    set_platform_event_runtime, PlatformEventRuntime,
};
use std::sync::{Arc, Mutex};

struct CapturePlatformEvents {
    events: Mutex<Vec<(String, serde_json::Value)>>,
}

impl PlatformEventRuntime for CapturePlatformEvents {
    fn record_platform_event(&self, event: &spanda_audit::PlatformEvent) {
        self.events.lock().unwrap().push((
            event.event_type.as_str().to_string(),
            event.payload.clone(),
        ));
    }
}

fn sample_report(entity_id: &str, health_status: &str) -> EntityHealthReport {
    EntityHealthReport {
        entity_id: entity_id.into(),
        entity_type: "robot".into(),
        health_status: health_status.into(),
        lifecycle_state: "active".into(),
        diagnostics: vec![EntityHealthDiagnostic {
            category: "health".into(),
            severity: "warning".into(),
            message: "Entity health is degraded".into(),
        }],
        metrics: EntityHealthMetrics::default(),
        children_checked: 0,
        sources: vec!["entity_snapshot".into()],
    }
}

#[test]
fn health_changed_emits_only_on_status_transition() {
    reset_health_status_cache_for_tests();
    let capture = Arc::new(CapturePlatformEvents {
        events: Mutex::new(Vec::new()),
    });
    let _ = set_platform_event_runtime(Arc::clone(&capture) as Arc<dyn PlatformEventRuntime>);

    record_entity_health_platform_events(&sample_report("rover-001", "healthy"));
    record_entity_health_platform_events(&sample_report("rover-001", "healthy"));
    record_entity_health_platform_events(&sample_report("rover-001", "degraded"));

    let events = capture.events.lock().unwrap();
    let health_changed: Vec<_> = events
        .iter()
        .filter(|(event_type, _)| event_type == names::HEALTH_CHANGED)
        .collect();
    assert_eq!(health_changed.len(), 2);

    let first = &health_changed[0].1;
    assert_eq!(first["from"], "unknown");
    assert_eq!(first["to"], "healthy");
    assert_eq!(first["reason"], "Entity health is degraded");

    let second = &health_changed[1].1;
    assert_eq!(second["from"], "healthy");
    assert_eq!(second["to"], "degraded");
}
