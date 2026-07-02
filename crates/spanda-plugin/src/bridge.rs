//! Map platform events to plugin lifecycle hooks.

use crate::hooks::PluginHook;
use spanda_audit::platform_event::names;

/// Map a canonical platform event type to a plugin hook when applicable.
pub fn hook_for_platform_event(event_type: &str) -> Option<PluginHook> {
    match event_type {
        names::ENTITY_CREATED
        | names::ENTITY_UPDATED
        | names::ENTITY_TAGGED
        | names::ENTITY_RELATED
        | names::ENTITY_DELETED => Some(PluginHook::OnEntityEvent),
        names::HEALTH_CHANGED
        | names::HEALTH_CHECK_FAILED
        | names::DEGRADED_MODE_ENTERED => Some(PluginHook::OnHealthChanged),
        names::READINESS_CHANGED | names::READINESS_GATE_FAILED => {
            Some(PluginHook::OnReadinessCompleted)
        }
        names::RECOVERY_COMPLETED | names::RECOVERY_FAILED | names::RECOVERY_TRIGGERED => {
            Some(PluginHook::OnRecoveryCompleted)
        }
        names::TRUST_UPDATED | names::TRUST_GATE_FAILED => Some(PluginHook::OnDiagnosisCompleted),
        _ => None,
    }
}

/// Map report-related API paths to the report hook.
pub fn hook_for_report_request(path: &str) -> Option<PluginHook> {
    if path.contains("/reports") || path.contains("/compliance") {
        Some(PluginHook::OnReportRequested)
    } else {
        None
    }
}
